use super::{FileId, FolderId};
use std::path::PathBuf;

use ratatui::prelude::*;

#[derive(Debug)]
pub struct Folder {
    pub path: PathBuf,

    pub child_files: Vec<FileId>,
    pub child_folders: Vec<FolderId>,

    /// Current open state in UI
    pub open: bool,
    /// Whether the folder has already been loaded once
    pub init: bool,
}

impl Folder {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            child_files: vec![],
            child_folders: vec![],
            open: false,
            init: false,
        }
    }

    /// Returns a ratatui line to display the folder
    pub fn line(&self, depth: usize) -> Line<'_> {
        Line::from(vec![
            Span::raw("  ".repeat(depth)),
            Span::raw(if self.open { " " } else { " " }).gray(),
            Span::raw(if self.open { " " } else { " " }).blue(),
            Span::raw(self.path.file_name().unwrap_or_default().to_string_lossy()).blue(),
        ])
    }
}
