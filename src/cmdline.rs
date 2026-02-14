use std::cell::Cell;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Widget};
use ropey::Rope;

/// Command line input
#[derive(Debug, Default)]
pub struct Cmdline {
    command: Rope,
    cursor: usize,
    cursor_position: Cell<Position>,
    open: bool,
}

impl Cmdline {
    /// Handle a key event. Returns true if the event was handled, false otherwise.
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if !self.open {
            return false;
        }

        match key_event.code {
            KeyCode::Esc => self.close(),
            KeyCode::Enter => self.execute(),
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.remove_char(self.cursor - 1);
                    self.cursor -= 1;
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor < self.command.len_chars() {
                    self.cursor += 1;
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.command.len_chars() {
                    self.remove_char(self.cursor);
                }
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.command.len_chars(),
            KeyCode::Char(c) => {
                self.command.insert_char(self.cursor, c);
                self.cursor += 1;
            }
            _ => {}
        }

        true
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    /// Draws the cursor if the command line is open.
    /// Returns true if the cursor was drawn, false otherwise.
    pub fn draw_cursor(&self, frame: &mut Frame) -> bool {
        if !self.open {
            return false;
        }
        frame.set_cursor_position(self.cursor_position.get());
        true
    }

    fn close(&mut self) {
        self.open = false;
        self.command = Rope::new();
        self.cursor = 0;
    }

    fn execute(&mut self) {
        // TODO: execute the command
        self.close();
    }

    fn remove_char(&mut self, idx: usize) {
        self.command.remove(idx..idx + 1);
    }
}

impl Widget for &Cmdline {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.open {
            return;
        }

        let [middle_line] = Layout::vertical([Constraint::Length(3)])
            .flex(Flex::Center)
            .areas(area);

        let width = 60.max(1 + 3 + self.command.chars().count() as u16 + 2);

        let [middle] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::Center)
            .areas(middle_line);

        // Set cursor position from the computed layout
        let cursor_y = middle.top() + 1;
        let cursor_x = middle.left() + 1 + 3 + self.cursor as u16;
        self.cursor_position.set(Position {
            x: cursor_x,
            y: cursor_y,
        });

        Clear::default().render(middle, buf);

        Paragraph::new(Text::from(Line::from(vec![
            Span::styled(" > ", Style::default().bold().blue()),
            Span::raw(&self.command),
        ])))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().magenta())
                .title_alignment(HorizontalAlignment::Center)
                .title(" Cmdline "),
        )
        .render(middle, buf);
    }
}
