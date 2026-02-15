use std::path::{Path, PathBuf};

use devicons::FileIcon;
use hex_color::HexColor;
use ratatui::prelude::*;

#[derive(Debug)]
pub struct Devicon {
    text: String,
    style: Style,
}

impl Devicon {
    pub fn new(path: &Path) -> Self {
        let icon = FileIcon::from(path);

        let mut style = Style::default();
        if let Ok(color) = HexColor::parse(icon.color) {
            style = style.fg(Color::Rgb(color.r, color.g, color.b));
        }

        Self {
            text: format!("{} ", icon.icon),
            style,
        }
    }

    pub fn span(&self) -> Span<'_> {
        Span::styled(&self.text, self.style)
    }
}

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    icon: Devicon,
}

impl File {
    pub fn new(path: PathBuf) -> Self {
        let icon = Devicon::new(&path);

        Self { path, icon }
    }

    /// Returns a ratatui line to display the file
    pub fn line(&self, depth: usize) -> Line<'_> {
        Line::from(vec![
            Span::raw("  ".repeat(depth + 1)),
            self.icon.span(),
            Span::raw(self.path.file_name().unwrap_or_default().to_string_lossy()),
        ])
    }
}
