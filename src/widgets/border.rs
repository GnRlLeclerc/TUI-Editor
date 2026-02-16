use std::cell::Cell;

use crate::{State, Widget};

use ratatui::prelude::*;

enum Orientation {
    Horizontal,
    Vertical,
}

pub struct Border {
    area: Cell<Rect>,
    orientation: Orientation,
}

impl Border {
    pub fn horizontal() -> Self {
        Self {
            area: Cell::new(Rect::default()),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn vertical() -> Self {
        Self {
            area: Cell::new(Rect::default()),
            orientation: Orientation::Vertical,
        }
    }
}

impl Widget for Border {
    fn render(&self, area: Rect, buf: &mut Buffer, _: &State) {
        let style = Style::default().dark_gray();

        match self.orientation {
            Orientation::Horizontal => {
                for x in area.left()..area.right() {
                    buf.set_string(x, area.top(), "─", style);
                }
            }
            Orientation::Vertical => {
                for y in area.top()..area.bottom() {
                    buf.set_string(area.left(), y, "│", style);
                }
            }
        }

        self.area.set(area);
    }

    fn contains(&self, pos: Position) -> bool {
        self.area.get().contains(pos)
    }
}
