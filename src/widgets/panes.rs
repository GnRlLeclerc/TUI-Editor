use std::cell::Cell;

use crate::{State, Widget};

use ratatui::prelude::*;

/// Group of editor panes
#[derive(Debug)]
pub struct Panes {
    area: Cell<Rect>,
}

impl Panes {
    pub fn new() -> Self {
        Self {
            area: Cell::new(Rect::default()),
        }
    }
}

impl Widget for Panes {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &State) {}

    fn contains(&self, pos: Position) -> bool {
        self.area.get().contains(pos)
    }
}
