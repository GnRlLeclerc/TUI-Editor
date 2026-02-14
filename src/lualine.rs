use ratatui::prelude::*;
use ratatui::{style::Color, widgets::Widget};

use crate::utils::whitespace_padding;
use crate::{App, Mode};

#[derive(Debug, Clone)]
pub struct LineConfig {
    pub normal_color: Color,
    pub insert_color: Color,
    pub visual_color: Color,
}

impl Default for LineConfig {
    fn default() -> Self {
        Self {
            normal_color: Color::Blue,
            insert_color: Color::Yellow,
            visual_color: Color::Magenta,
        }
    }
}

/// Lualine equivalent
#[derive(Debug, Clone, Default)]
pub struct Lualine {
    config: LineConfig,
}

impl Lualine {
    pub fn render(&self, area: Rect, buf: &mut Buffer, app: &App) {
        // Left part
        let text = self.text_for_mode(app.mode);
        let color = self.color_for_mode(app.mode);

        Line::from(vec![
            Span::from(text).black().bg(color),
            Span::from("").fg(color).on_black(),
        ])
        .render(area, buf);

        let row = app.cursor.y + 1;
        let col = app.cursor.x + 1;

        // Right part
        let text = format!(
            "  {}  {}{}:{}{} ",
            if app.cursor.y == 0 {
                "Top".to_string()
            } else if app.cursor.y == app.rope.len_lines() - 1 {
                "Bot".to_string()
            } else {
                let percent = (app.cursor.y * 100) / app.rope.len_lines();
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

    fn color_for_mode(&self, mode: Mode) -> Color {
        match mode {
            Mode::Normal => self.config.normal_color,
            Mode::Insert => self.config.insert_color,
            Mode::Visual => self.config.visual_color,
        }
    }

    fn text_for_mode(&self, mode: Mode) -> &'static str {
        match mode {
            Mode::Normal => " NORMAL ",
            Mode::Insert => " INSERT ",
            Mode::Visual => " VISUAL ",
        }
    }
}
