use std::{cell::Cell, fs::File, io::stdout, path::PathBuf};

use clap::Parser;
use crossterm::{
    cursor::SetCursorStyle,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        MouseButton, MouseEventKind,
    },
    execute,
};
use devicons::FileIcon;
use log::LevelFilter;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    style::Stylize,
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget},
};
use ropey::Rope;
use simplelog::{Config, WriteLogger};

use crate::{border::render_vertical_border, cmdline::Cmdline, cursor::Cursor, lualine::Lualine};

mod border;
mod cmdline;
mod cursor;
mod filetree;
mod lualine;
mod utils;

/// Editor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
}

#[derive(Debug, Default)]
pub struct App {
    // Global app settings
    cursor_margin_y: usize,
    scroll_tick: usize,
    exit: bool,
    mode: Mode,
    cmdline: Cmdline,
    lualine: Lualine,
    filetree_width: usize,
    filetree_open: bool,

    // Per editor buffer state
    cursor: Cursor,
    rope: Rope,
    screen_y: Cell<usize>,
    scroll_y: Cell<usize>,
    icon: Option<FileIcon>,
}

#[derive(Debug, clap::Parser)]
struct Args {
    file: Option<PathBuf>,
}

fn main() {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("debug.log").unwrap(),
    )
    .unwrap();

    let mut app = App::default();
    app.scroll_tick = 3;
    app.cursor_margin_y = 5;
    app.filetree_width = 25;

    if let Some(file) = Args::parse().file {
        let icon = FileIcon::from(&file);
        let content = std::fs::read_to_string(&file).unwrap();
        app.rope = Rope::from(content);
        app.icon = Some(icon);
    }

    execute!(stdout(), EnableMouseCapture).unwrap();
    ratatui::run(|terminal| app.run(terminal)).unwrap();
    execute!(stdout(), DisableMouseCapture).unwrap();
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        self.set_cursor_style(SetCursorStyle::SteadyBlock);
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());

        // Draw cmdline cursor
        if self.cmdline.draw_cursor(frame) {
            return;
        }

        // Draw active buffer cursor
        let mut position = self.cursor.position();
        position.x += self.x_margin() as u16 + self.filetree_offset() as u16;
        position.y = position.y.saturating_sub(self.scroll_y.get() as u16);
        frame.set_cursor_position(position);
    }

    fn numbers_gutter_width(&self) -> usize {
        4.max((self.rope.len_lines() as f32).log10() as usize)
    }

    fn x_margin(&self) -> usize {
        2 + self.numbers_gutter_width() + 2
    }

    fn set_cursor_style(&self, style: SetCursorStyle) {
        if let Err(e) = execute!(stdout(), style) {
            log::error!("Failed to set cursor style: {}", e);
        }
    }
    fn filetree_offset(&self) -> usize {
        if self.filetree_open {
            self.filetree_width + 1
        } else {
            0
        }
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            Event::Mouse(mouse_event) => match mouse_event.kind {
                MouseEventKind::Down(button) => {
                    if button == MouseButton::Left {
                        let x = mouse_event.column as usize;
                        let y = mouse_event.row as usize;
                        self.cursor.set_position(
                            x - self.x_margin() - self.filetree_offset(),
                            y + self.scroll_y.get(),
                            &self.rope,
                        );
                    }
                }
                MouseEventKind::ScrollUp => {
                    self.scroll_y
                        .set(self.scroll_y.get().saturating_sub(self.scroll_tick));

                    if self.cursor.y + self.cursor_margin_y
                        > self.scroll_y.get() + self.screen_y.get()
                    {
                        let n = self.cursor.y + self.cursor_margin_y
                            - (self.scroll_y.get() + self.screen_y.get());
                        self.cursor.move_up_n(&self.rope, n);
                    }
                }
                MouseEventKind::ScrollDown => {
                    self.scroll_y
                        .set(self.scroll_y.get().saturating_add(self.scroll_tick));

                    if self.cursor.y < self.scroll_y.get() + self.cursor_margin_y {
                        let n = self.scroll_y.get() + self.cursor_margin_y - self.cursor.y;
                        self.cursor.move_down_n(&self.rope, n);
                    }
                }
                _ => {}
            },
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.code == KeyCode::Tab {
            self.exit();
        }

        if self.cmdline.handle_key_event(key_event) {
            return;
        }

        match self.mode {
            Mode::Insert => self.handle_insert_mode_key_event(key_event),
            Mode::Normal => self.handle_normal_mode_key_event(key_event),
            Mode::Visual => self.handle_visual_mode_key_event(key_event),
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        match mode {
            Mode::Insert => self.set_cursor_style(SetCursorStyle::SteadyBar),
            Mode::Normal | Mode::Visual => self.set_cursor_style(SetCursorStyle::SteadyBlock),
        }
    }

    fn handle_normal_mode_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('f') => self.filetree_open = !self.filetree_open,
            KeyCode::Char('i') => self.set_mode(Mode::Insert),
            KeyCode::Char('h') => self.cursor.move_left(&self.rope),
            KeyCode::Char('j') => self.cursor.move_down(&self.rope),
            KeyCode::Char('k') => self.cursor.move_up(&self.rope),
            KeyCode::Char('l') => self.cursor.move_right(&self.rope),
            KeyCode::Char('0') => self.cursor.move_line_start(&self.rope),
            KeyCode::Char('$') => self.cursor.move_line_end(&self.rope),
            KeyCode::Char('v') => self.set_mode(Mode::Visual),
            KeyCode::Char('a') => {
                self.cursor.move_right(&self.rope);
                self.set_mode(Mode::Insert);
            }
            KeyCode::Char('A') => {
                self.cursor.move_line_end(&self.rope);
                self.set_mode(Mode::Insert);
            }
            KeyCode::Char('I') => {
                self.cursor.move_line_start(&self.rope);
                self.set_mode(Mode::Insert);
            }
            KeyCode::Char(':') => self.cmdline.open(),
            _ => {}
        }
    }

    fn handle_visual_mode_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.set_mode(Mode::Normal),
            KeyCode::Char('i') => self.set_mode(Mode::Insert),
            KeyCode::Char('h') => self.cursor.move_left(&self.rope),
            KeyCode::Char('j') => self.cursor.move_down(&self.rope),
            KeyCode::Char('k') => self.cursor.move_up(&self.rope),
            KeyCode::Char('l') => self.cursor.move_right(&self.rope),
            KeyCode::Char('0') => self.cursor.move_line_start(&self.rope),
            KeyCode::Char('$') => self.cursor.move_line_end(&self.rope),
            _ => {}
        }
    }

    fn handle_insert_mode_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.set_mode(Mode::Normal),
            KeyCode::Char(c) => self.cursor.insert_char(&mut self.rope, c),
            KeyCode::Enter => self.cursor.insert_char(&mut self.rope, '\n'),
            KeyCode::Backspace => self.cursor.delete_prev_char(&mut self.rope),
            KeyCode::Delete => self.cursor.delete_next_char(&mut self.rope),
            KeyCode::Right => self.cursor.move_right(&self.rope),
            KeyCode::Left => self.cursor.move_left(&self.rope),
            KeyCode::Up => self.cursor.move_up(&self.rope),
            KeyCode::Down => self.cursor.move_down(&self.rope),
            KeyCode::Home => self.cursor.move_line_start(&self.rope),
            KeyCode::End => self.cursor.move_line_end(&self.rope),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line_length = area.width as usize;
        let line_count = area.height as usize;
        self.screen_y.set(line_count);

        // Autoscroll at rendering time, depending on the cursor position
        if self.cursor.y < self.scroll_y.get() + self.cursor_margin_y {
            self.scroll_y
                .set(self.cursor.y.saturating_sub(self.cursor_margin_y));
        } else if self.cursor.y + self.cursor_margin_y >= self.scroll_y.get() + line_count {
            self.scroll_y
                .set(self.cursor.y + 1 + self.cursor_margin_y - line_count);
        }

        let [main, lualine] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1), // lualine
        ])
        .areas(area);

        let [filetree, border, _, gutter, _, buffer] = Layout::horizontal([
            Constraint::Length(if self.filetree_open {
                self.filetree_width as u16
            } else {
                0
            }), // file tree
            Constraint::Length(if self.filetree_open { 1 } else { 0 }), // file tree border
            Constraint::Length(2),                                      // margin
            Constraint::Length(self.numbers_gutter_width() as u16),
            Constraint::Length(2), // margin
            Constraint::Fill(1),
        ])
        .areas(main);

        if self.filetree_open {
            render_vertical_border(border, buf);
        }

        // Render the text area
        Paragraph::new(Text::from(
            (self.scroll_y.get()..self.rope.len_lines().min(line_count + self.scroll_y.get()))
                .map(|line| {
                    let mut remaining = line_length;
                    let line = self.rope.line(line);
                    Line::from_iter(line.chunks().map_while(|chunk| {
                        if remaining == 0 {
                            return None;
                        }

                        let n = chunk.chars().count().min(remaining);
                        remaining -= n;

                        Some(&chunk[..n])
                    }))
                })
                .collect::<Vec<_>>(),
        ))
        .render(buffer, buf);

        // Render the gutter
        Text::from_iter(
            (self.scroll_y.get()..self.rope.len_lines().min(line_count + self.scroll_y.get())).map(
                |line| {
                    if line == self.cursor.y {
                        return Line::from(Span::raw((line + 1).to_string()).cyan())
                            .alignment(HorizontalAlignment::Right);
                    }
                    let relative = if line < self.cursor.y {
                        self.cursor.y - line
                    } else {
                        line - self.cursor.y
                    };

                    Line::from(Span::raw(relative.to_string()).dark_gray())
                        .alignment(HorizontalAlignment::Right)
                },
            ),
        )
        .render(gutter, buf);

        // Render the lualine
        self.lualine.render(lualine, buf, self);

        // Render the cmdline if open
        self.cmdline.render(area, buf);
    }
}
