#[derive(Debug)]
pub struct Config {
    /// Minimum number of lines between the cursor and the top/bottom of the screen.
    pub cursor_margin_y: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { cursor_margin_y: 5 }
    }
}
