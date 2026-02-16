use ratatui::prelude::*;

use crate::State;
pub use border::Border;
pub use cmdline::Cmdline;
pub use filetree::FileTree;
pub use lualine::Lualine;
pub use pane::Pane;
pub use panes::Panes;

mod border;
mod cmdline;
mod filetree;
mod lualine;
mod pane;
mod panes;

/// Editor widget trait
pub trait Widget {
    /// Render the widget with the given global state.
    fn render(&self, area: Rect, buf: &mut Buffer, state: &State);

    /// Check for mouse position hits.
    fn contains(&self, pos: Position) -> bool;
}
