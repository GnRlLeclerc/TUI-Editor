use ratatui::layout::Position;
use ropey::Rope;

/// Cursor with position, relative to the parent element
#[derive(Debug, Default)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,

    /// Memorize the preferred x position
    /// when moving up and down.
    /// Resets when moving laterally.
    pub preferred_x: usize,
}

impl Cursor {
    pub fn position(&self) -> Position {
        Position {
            x: self.x as u16,
            y: self.y as u16,
        }
    }

    /// Insert a char at the current cursor position
    pub fn insert_char(&mut self, rope: &mut Rope, c: char) {
        rope.insert_char(self.cursor_char_index(rope), c);
        self.move_right(rope);
    }

    /// Delete the char before the cursor
    pub fn delete_prev_char(&mut self, rope: &mut Rope) {
        let index = self.cursor_char_index(rope);
        if index > 0 {
            rope.remove(index - 1..index);
            self.move_left(rope);
        }
    }

    /// Delete the char after the cursor
    pub fn delete_next_char(&mut self, rope: &mut Rope) {
        let index = self.cursor_char_index(rope);
        if index < rope.len_chars() {
            rope.remove(index..index + 1);
        }
    }

    pub fn move_left(&mut self, rope: &Rope) {
        if self.x > 0 {
            self.x -= 1;
            self.preferred_x = self.x;
        } else if self.y > 0 {
            self.y -= 1;
            self.move_line_end(rope);
        }
    }

    pub fn move_right(&mut self, rope: &Rope) {
        let last = self.last_valid_line_index(rope);
        if self.x < last {
            self.x += 1;
        } else if self.y < rope.len_lines() - 1 {
            self.y += 1;
            self.x = 0;
        }

        self.preferred_x = self.x;
    }

    pub fn move_up(&mut self, rope: &Rope) {
        if self.y > 0 {
            self.y -= 1;
            self.move_to_preferred_x(rope);
        }
    }

    pub fn move_down(&mut self, rope: &Rope) {
        if self.y < rope.len_lines() - 1 {
            self.y += 1;
            self.move_to_preferred_x(rope);
        }
    }

    pub fn move_line_end(&mut self, rope: &Rope) {
        self.x = self.last_valid_line_index(rope);
        self.preferred_x = self.x;
    }

    pub fn move_line_start(&mut self, _: &Rope) {
        self.x = 0;
        self.preferred_x = 0;
    }

    /// Set the cursor position (from a click)
    pub fn set_position(&mut self, x: usize, y: usize, rope: &Rope) {
        let lines = rope.len_lines();
        if y >= lines {
            self.y = lines - 1;
            self.move_line_end(rope);
        } else {
            self.y = y;
            self.x = x.min(self.last_valid_line_index(rope));
            self.preferred_x = self.x;
        }
    }

    // ********************************************************************* //
    //                                Helpers                                //
    // ********************************************************************* //

    /// Get the char index at the cursor position
    fn cursor_char_index(&self, rope: &Rope) -> usize {
        rope.line_to_char(self.y) + self.x
    }

    /// Returns the last "valid" cursor position in the line.
    /// This is the position right before potential \n or \r\n chars.
    fn last_valid_line_index(&self, rope: &Rope) -> usize {
        let line = rope.line(self.y);
        let mut length = line.len_chars();

        if length > 0 && line.char(length - 1) == '\n' {
            length -= 1;
            if length > 0 && line.char(length - 1) == '\r' {
                length -= 1;
            }
        }

        length
    }

    fn move_to_preferred_x(&mut self, rope: &Rope) {
        self.x = self.preferred_x.min(self.last_valid_line_index(rope));
    }
}
