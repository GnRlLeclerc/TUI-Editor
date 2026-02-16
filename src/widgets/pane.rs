use std::cell::Cell;

use crate::{State, Widget, cursor::Cursor, state::FileId, utils::number_digits};

use ratatui::{
    prelude::*,
    widgets::{Paragraph, Widget as RatatuiWidget},
};

/// Single pane widget, linked to a single file
pub struct Pane {
    file: FileId,
    cursor: Cursor,
    scroll_y: Cell<usize>,

    // Memoized values from the rendering pass
    area: Cell<Rect>,
    gutter_width: Cell<u16>,
}

impl Pane {
    pub fn new(file: FileId) -> Self {
        Self {
            area: Cell::new(Rect::default()),
            gutter_width: Cell::new(0),
            cursor: Cursor::default(),
            file,
            scroll_y: Cell::new(0),
        }
    }

    /// Computes the absolute cursor position to display
    /// on the screen from the inner relative cursor position.
    pub fn cursor_position(&self) -> Position {
        let area = self.area.get();
        let x = (self.gutter_width.get() + 1 + area.left()).saturating_add(self.cursor.x as u16);
        let y = (self.cursor.y - self.scroll_y.get()) as u16 + area.top();
        Position::new(x, y)
    }
}

impl Widget for Pane {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &State) {
        // If the file does not exist or has no buffer, silently render nothing
        // (this should not happen)
        let buffer = match state.filesystem.files.get(self.file) {
            Some(file) => match &file.buffer {
                Some(buffer) => buffer,
                None => return,
            },
            None => return,
        };

        let cursor_margin_y = state.config.cursor_margin_y;
        let line_length = area.width as usize;
        let line_count = area.height as usize;

        // Autoscroll at rendering time, depending on the cursor position
        if self.cursor.y < self.scroll_y.get() + cursor_margin_y {
            self.scroll_y
                .set(self.cursor.y.saturating_sub(cursor_margin_y));
        } else if self.cursor.y + cursor_margin_y >= self.scroll_y.get() + line_count {
            self.scroll_y
                .set(self.cursor.y + 1 + cursor_margin_y - line_count);
        }

        let gutter_width = 4.max(number_digits(buffer.len_lines()));
        self.gutter_width.set(gutter_width as u16);
        let [gutter_area, _, buffer_area] = Layout::horizontal([
            Constraint::Length(gutter_width as u16),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        // Render the text area
        Paragraph::new(Text::from(
            (self.scroll_y.get()..buffer.len_lines().min(line_count + self.scroll_y.get()))
                .map(|line| {
                    let mut remaining = line_length;
                    let line = buffer.line(line);
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
        .render(buffer_area, buf);

        // Render the gutter
        Text::from_iter(
            (self.scroll_y.get()..buffer.len_lines().min(line_count + self.scroll_y.get())).map(
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
        .render(gutter_area, buf);

        self.area.set(area);
    }

    fn contains(&self, pos: Position) -> bool {
        self.area.get().contains(pos)
    }
}
