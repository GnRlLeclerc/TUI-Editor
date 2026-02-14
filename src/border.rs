use ratatui::prelude::*;

pub fn render_vertical_border(area: Rect, buf: &mut Buffer) {
    let style = Style::default().dark_gray();
    for y in area.top()..area.bottom() {
        buf.set_string(area.left(), y, "│", style);
    }
}

pub fn render_horizontal_border(area: Rect, buf: &mut Buffer) {
    let style = Style::default().dark_gray();
    for x in area.left()..area.right() {
        buf.set_string(x, area.top(), "─", style);
    }
}
