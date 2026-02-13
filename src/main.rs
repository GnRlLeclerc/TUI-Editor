use std::{
    fs::File,
    io::{BufReader, stdout},
    path::PathBuf,
};

use clap::Parser;
use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
};
use log::LevelFilter;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::{Paragraph, Widget},
};
use ropey::Rope;
use simplelog::{Config, WriteLogger};

use crate::cursor::Cursor;

mod cursor;

#[derive(Debug, Default)]
pub struct App {
    cursor: Cursor,
    rope: Rope,
    exit: bool,
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

    if let Some(file) = Args::parse().file {
        app.rope = Rope::from_reader(BufReader::new(File::open(file).unwrap())).unwrap();
    }

    ratatui::run(|terminal| app.run(terminal)).unwrap();
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        self.set_cursor_style(SetCursorStyle::BlinkingBar);
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
        frame.set_cursor_position(self.cursor.position());
    }

    fn set_cursor_style(&self, style: SetCursorStyle) {
        if let Err(e) = execute!(stdout(), style) {
            log::error!("Failed to set cursor style: {}", e);
        }
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exit(),
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

        Paragraph::new(Text::from(
            (0..line_count.min(self.rope.len_lines()))
                .map(|line| {
                    let line = self.rope.line(line);
                    let line = line
                        .slice(0..line_length.min(line.len_chars()))
                        .as_str()
                        .unwrap();

                    Line::from(line)
                })
                .collect::<Vec<_>>(),
        ))
        .render(area, buf);
    }
}
