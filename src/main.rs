use std::{cell::Cell, fs::File, io::stdout, path::PathBuf};

use clap::Parser;
use crossterm::{
    cursor::SetCursorStyle,
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEvent,
        KeyEventKind, MouseButton, MouseEventKind,
    },
    execute,
};
use devicons::FileIcon;
use futures::{StreamExt, stream::Fuse};
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
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    border::render_vertical_border, cmdline::Cmdline, cursor::Cursor, filesystem::Filetree,
    lualine::Lualine,
};

mod border;
mod cmdline;
mod cursor;
mod filesystem;
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

/// Internal editor events,
/// for background running tasks to make their
/// results available to the main thread.
pub enum EditorEvent {
    FolderLoaded {
        id: filesystem::FolderId,
        files: Vec<filesystem::File>,
        folders: Vec<filesystem::Folder>,
    },
}

#[derive(Debug)]
pub struct App {
    // Global app settings
    cursor_margin_y: usize,
    scroll_tick: usize,
    exit: bool,
    mode: Mode,
    cmdline: Cmdline,
    lualine: Lualine,
    filetree: Filetree,

    // Event channels
    term_events: Fuse<EventStream>,
    editor_events: Receiver<EditorEvent>,
    editor_sender: Sender<EditorEvent>,

    // Per editor buffer state
    cursor: Cursor,
    rope: Rope,
    screen_y: Cell<usize>,
    scroll_y: Cell<usize>,
    icon: Option<FileIcon>,
}

impl App {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(64);

        Self {
            cursor_margin_y: 5,
            scroll_tick: 3,
            exit: false,
            mode: Mode::Normal,
            cmdline: Cmdline::default(),
            lualine: Lualine::default(),
            filetree: Filetree::new(sender.clone()),
            term_events: EventStream::new().fuse(),
            editor_events: receiver,
            editor_sender: sender,
            cursor: Cursor::default(),
            rope: Rope::default(),
            screen_y: Cell::new(0),
            scroll_y: Cell::new(0),
            icon: None,
        }
    }
}

#[derive(Debug, clap::Parser)]
struct Args {
    file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("debug.log").unwrap(),
    )
    .unwrap();

    let mut app = App::new();
    app.filetree.load_root();

    if let Some(file) = Args::parse().file {
        let icon = FileIcon::from(&file);
        let content = std::fs::read_to_string(&file).unwrap();
        app.rope = Rope::from(content);
        app.icon = Some(icon);
    }

    execute!(stdout(), EnableMouseCapture).unwrap();
    let terminal = ratatui::init();
    let result = app.run(terminal).await;
    ratatui::restore();
    execute!(stdout(), DisableMouseCapture).unwrap();
    result
}

impl App {
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        self.set_cursor_style(SetCursorStyle::SteadyBlock);
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await;
        }
        Ok(())
    }

    pub async fn handle_events(&mut self) {
        tokio::select! {
            Some(Ok(event)) = self.term_events.next() => {
                self.handle_term_event(event);
            }
            Some(event) = self.editor_events.recv() => {
                self.handle_editor_event(event).await;
            }
        }
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
        if self.filetree.open {
            self.filetree.width + 1
        } else {
            0
        }
    }

    async fn handle_editor_event(&mut self, event: EditorEvent) {
        match event {
            EditorEvent::FolderLoaded { id, files, folders } => {
                self.filetree.init_folder(id, files, folders);
            }
        }
    }

    fn handle_term_event(&mut self, event: Event) {
        match event {
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
            KeyCode::Char('f') => self.filetree.open = !self.filetree.open,
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
            Constraint::Length(if self.filetree.open {
                self.filetree.width as u16
            } else {
                0
            }), // file tree
            Constraint::Length(if self.filetree.open { 1 } else { 0 }), // file tree border
            Constraint::Length(2),                                      // margin
            Constraint::Length(self.numbers_gutter_width() as u16),
            Constraint::Length(2), // margin
            Constraint::Fill(1),
        ])
        .areas(main);

        if self.filetree.open {
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

        // Render the file tree (if open)
        if self.filetree.open {
            self.filetree.render(filetree, buf);
        }

        // Render the lualine
        self.lualine.render(lualine, buf, self);

        // Render the cmdline if open
        self.cmdline.render(area, buf);
    }
}
