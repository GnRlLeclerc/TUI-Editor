use crossterm::event::Event;

mod alpha;
mod editor;

pub use alpha::AlphaScreen;
pub use editor::EditorScreen;

use crate::{state::State, widgets::Widget};

/// Editor screen trait
/// A screen is a single "page" of the app, and contains widgets
pub trait Screen: Widget {
    /// Handle terminal events
    /// (dispatch them to child widgets, which may modify the global state)
    fn handle(&mut self, event: Event, state: &mut State);
}
