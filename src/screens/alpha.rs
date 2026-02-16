use crossterm::event::Event;
use ratatui::prelude::*;

use crate::{Screen, State, Widget};

/// alpha.nvim home page widget
#[derive(Debug)]
pub struct AlphaScreen {}

impl AlphaScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for AlphaScreen {
    fn handle(&mut self, event: Event, state: &mut State) {
        // TODO
    }
}

impl Widget for AlphaScreen {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &State) {
        // TODO
    }

    fn contains(&self, _: Position) -> bool {
        true
    }
}
