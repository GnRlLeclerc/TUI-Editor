use crossterm::event::Event;
use ratatui::prelude::*;

use crate::{
    Screen, State, Widget,
    widgets::{Border, FileTree, Lualine, Panes},
};

/// The file editor screen, with a filetree
#[derive(Debug)]
pub struct EditorScreen {
    filetree: FileTree,
    tree_open: bool,
    tree_width: u16,

    /// Border between the filetree and the panes
    border: Border,

    /// File editor panes
    panes: Panes,

    /// Lualine at the bottom
    lualine: Lualine,
}

impl EditorScreen {
    pub fn new() -> Self {
        Self {
            filetree: FileTree::new(),
            tree_open: true,
            tree_width: 30,
            border: Border::vertical(),
            panes: Panes::new(),
            lualine: Lualine::new(),
        }
    }
}

impl Screen for EditorScreen {
    fn handle(&mut self, event: Event, state: &mut State) {
        // TODO:
    }
}

impl Widget for EditorScreen {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &State) {
        let [main, lualine] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        match self.tree_open {
            true => {
                let [tree, border, panes] = Layout::horizontal([
                    Constraint::Length(self.tree_width),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .areas(main);

                self.filetree.render(tree, buf, state);
                self.border.render(border, buf, state);
                self.panes.render(panes, buf, state);
            }
            false => self.panes.render(main, buf, state),
        }

        self.lualine.render(lualine, buf, state);
    }

    /// Always true when the screen is active
    fn contains(&self, _: Position) -> bool {
        true
    }
}
