use std::cell::Cell;

use crate::{
    State, Widget,
    state::{FileSystem, FolderId},
};
use ratatui::{prelude::*, widgets::Widget as RatatuiWidget};

pub struct FileTree {
    area: Cell<Rect>,
}

impl FileTree {
    pub fn new() -> Self {
        Self {
            area: Cell::new(Rect::default()),
        }
    }

    /// Recursively display files, folders and their children
    fn recurse_lines<'a>(
        &self,
        id: FolderId,
        filesystem: &'a FileSystem,
        lines: &mut Vec<Line<'a>>,
        remaining: &mut u16,
        depth: usize,
    ) {
        let folder = &filesystem.folders[id];
        for folder_id in &folder.child_folders {
            if *remaining == 0 {
                return;
            }

            let folder = &filesystem.folders[*folder_id];
            if folder.hidden() {
                continue;
            }
            lines.push(folder.line(depth));

            if folder.open {
                self.recurse_lines(*folder_id, filesystem, lines, remaining, depth + 1);
            }

            *remaining = remaining.saturating_sub(1);
        }

        for file_id in &folder.child_files {
            if *remaining == 0 {
                return;
            }

            let file = &filesystem.files[*file_id];
            lines.push(file.line(depth));
            *remaining = remaining.saturating_sub(1);
        }
    }
}

impl Widget for FileTree {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &State) {
        let mut lines = vec![];
        let mut remaining = area.height;
        let filesystem = &state.filesystem;
        self.recurse_lines(filesystem.root, filesystem, &mut lines, &mut remaining, 0);

        Text::from(lines).render(area, buf);

        self.area.set(area);
    }

    fn contains(&self, pos: Position) -> bool {
        self.area.get().contains(pos)
    }
}
