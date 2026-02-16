use ratatui::prelude::*;
use ratatui::{style::Color, widgets::Widget as RatatuiWidget};
use ropey::Rope;

use crate::cursor::Cursor;
use crate::state::Mode;
use crate::utils::whitespace_padding;
use crate::{State, Widget};

/// Lualine equivalent
#[derive(Debug, Clone, Default)]
pub struct Lualine;

impl Lualine {
    pub fn new() -> Self {
        Self {}
    }

    fn temp_render_from_cursor_and_rope(
        &self,
        area: Rect,
        buf: &mut Buffer,
        color: Color,
        cursor: &Cursor,
        rope: &Rope,
    ) {
        let row = cursor.y + 1;
        let col = cursor.x + 1;

        // Right part
        let text = format!(
            "  {}  {}{}:{}{} ",
            if cursor.y == 0 {
                "Top".to_string()
            } else if cursor.y == rope.len_lines() - 1 {
                "Bot".to_string()
            } else {
                let percent = (cursor.y * 100) / rope.len_lines();
                let padding = if percent < 10 { " " } else { "" };
                format!("{}{}%", padding, percent)
            },
            whitespace_padding(row, 3),
            row,
            col,
            whitespace_padding(col, 2),
        );
        Line::from(vec![
            Span::from("").fg(color).on_black(),
            Span::from(text).black().bg(color),
        ])
        .alignment(HorizontalAlignment::Right)
        .render(area, buf);
    }
}

impl Widget for Lualine {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &State) {
        let text = state.mode.text();
        let color = state.mode.color();

        // Left part
        Line::from(vec![
            Span::from(text).black().bg(color),
            Span::from("").fg(color).on_black(),
        ])
        .render(area, buf);

        // Right part (TODO)
        // self.temp_render_from_cursor_and_rope(area, buf, color, &state.cursor, &state.rope);
    }

    /// Lualine is not click-sensitive
    fn contains(&self, _: Position) -> bool {
        false
    }
}

impl Mode {
    fn color(&self) -> Color {
        match self {
            Mode::Normal => Color::Blue,
            Mode::Insert => Color::Green,
            Mode::Visual => Color::Magenta,
            Mode::Command => Color::Yellow,
        }
    }

    fn text(&self) -> &'static str {
        match self {
            Mode::Normal => " NORMAL ",
            Mode::Insert => " INSERT ",
            Mode::Visual => " VISUAL ",
            Mode::Command => " COMMAND ",
        }
    }
}
