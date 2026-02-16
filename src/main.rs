use std::{fs::File, path::PathBuf};

use clap::Parser;
use log::LevelFilter;
use simplelog::{Config, WriteLogger};

pub use screens::Screen;
pub use state::State;
pub use widgets::Widget;

use crate::app::App;

mod app;
mod cursor;
mod screens;
mod state;
mod utils;
mod widgets;

#[derive(Debug, clap::Parser)]
struct Args {
    file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("debug.log").unwrap(),
    )
    .unwrap();

    let file = Args::parse().file.unwrap();
    let mut app = App::new(file);

    app.run().await
}

// impl App {
//     fn handle_term_event(&mut self, event: Event) {
//         match event {
//             // it's important to check that the event is a key press event as
//             // crossterm also emits key release and repeat events on Windows.
//             Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
//                 self.handle_key_event(key_event)
//             }
//             Event::Mouse(mouse_event) => match mouse_event.kind {
//                 MouseEventKind::Down(button) => {
//                     if button == MouseButton::Left {
//                         let x = mouse_event.column as usize;
//                         let y = mouse_event.row as usize;
//                         self.cursor.set_position(
//                             x - self.x_margin() - self.filetree_offset(),
//                             y + self.scroll_y.get(),
//                             &self.rope,
//                         );
//                     }
//                 }
//                 MouseEventKind::ScrollUp => {
//                     self.scroll_y
//                         .set(self.scroll_y.get().saturating_sub(self.scroll_tick));
//
//                     if self.cursor.y + self.cursor_margin_y
//                         > self.scroll_y.get() + self.screen_y.get()
//                     {
//                         let n = self.cursor.y + self.cursor_margin_y
//                             - (self.scroll_y.get() + self.screen_y.get());
//                         self.cursor.move_up_n(&self.rope, n);
//                     }
//                 }
//                 MouseEventKind::ScrollDown => {
//                     self.scroll_y
//                         .set(self.scroll_y.get().saturating_add(self.scroll_tick));
//
//                     if self.cursor.y < self.scroll_y.get() + self.cursor_margin_y {
//                         let n = self.scroll_y.get() + self.cursor_margin_y - self.cursor.y;
//                         self.cursor.move_down_n(&self.rope, n);
//                     }
//                 }
//                 _ => {}
//             },
//             _ => {}
//         };
//     }
//
//     fn handle_key_event(&mut self, key_event: KeyEvent) {
//         if key_event.code == KeyCode::Tab {
//             self.exit();
//         }
//
//         if self.cmdline.handle_key_event(key_event) {
//             return;
//         }
//
//         match self.mode {
//             Mode::Insert => self.handle_insert_mode_key_event(key_event),
//             Mode::Normal => self.handle_normal_mode_key_event(key_event),
//             Mode::Visual => self.handle_visual_mode_key_event(key_event),
//         }
//     }
//
//     fn set_mode(&mut self, mode: Mode) {
//         self.mode = mode;
//         match mode {
//             Mode::Insert => self.set_cursor_style(SetCursorStyle::SteadyBar),
//             Mode::Normal | Mode::Visual => self.set_cursor_style(SetCursorStyle::SteadyBlock),
//         }
//     }
//
//     fn handle_normal_mode_key_event(&mut self, key_event: KeyEvent) {
//         match key_event.code {
//             KeyCode::Char('f') => self.filetree.open = !self.filetree.open,
//             KeyCode::Char('i') => self.set_mode(Mode::Insert),
//             KeyCode::Char('h') => self.cursor.move_left(&self.rope),
//             KeyCode::Char('j') => self.cursor.move_down(&self.rope),
//             KeyCode::Char('k') => self.cursor.move_up(&self.rope),
//             KeyCode::Char('l') => self.cursor.move_right(&self.rope),
//             KeyCode::Char('0') => self.cursor.move_line_start(&self.rope),
//             KeyCode::Char('$') => self.cursor.move_line_end(&self.rope),
//             KeyCode::Char('v') => self.set_mode(Mode::Visual),
//             KeyCode::Char('a') => {
//                 self.cursor.move_right(&self.rope);
//                 self.set_mode(Mode::Insert);
//             }
//             KeyCode::Char('A') => {
//                 self.cursor.move_line_end(&self.rope);
//                 self.set_mode(Mode::Insert);
//             }
//             KeyCode::Char('I') => {
//                 self.cursor.move_line_start(&self.rope);
//                 self.set_mode(Mode::Insert);
//             }
//             KeyCode::Char(':') => self.cmdline.open(),
//             _ => {}
//         }
//     }
//
//     fn handle_visual_mode_key_event(&mut self, key_event: KeyEvent) {
//         match key_event.code {
//             KeyCode::Esc => self.set_mode(Mode::Normal),
//             KeyCode::Char('i') => self.set_mode(Mode::Insert),
//             KeyCode::Char('h') => self.cursor.move_left(&self.rope),
//             KeyCode::Char('j') => self.cursor.move_down(&self.rope),
//             KeyCode::Char('k') => self.cursor.move_up(&self.rope),
//             KeyCode::Char('l') => self.cursor.move_right(&self.rope),
//             KeyCode::Char('0') => self.cursor.move_line_start(&self.rope),
//             KeyCode::Char('$') => self.cursor.move_line_end(&self.rope),
//             _ => {}
//         }
//     }
//
//     fn handle_insert_mode_key_event(&mut self, key_event: KeyEvent) {
//         match key_event.code {
//             KeyCode::Esc => self.set_mode(Mode::Normal),
//             KeyCode::Char(c) => self.cursor.insert_char(&mut self.rope, c),
//             KeyCode::Enter => self.cursor.insert_char(&mut self.rope, '\n'),
//             KeyCode::Backspace => self.cursor.delete_prev_char(&mut self.rope),
//             KeyCode::Delete => self.cursor.delete_next_char(&mut self.rope),
//             KeyCode::Right => self.cursor.move_right(&self.rope),
//             KeyCode::Left => self.cursor.move_left(&self.rope),
//             KeyCode::Up => self.cursor.move_up(&self.rope),
//             KeyCode::Down => self.cursor.move_down(&self.rope),
//             KeyCode::Home => self.cursor.move_line_start(&self.rope),
//             KeyCode::End => self.cursor.move_line_end(&self.rope),
//             _ => {}
//         }
//     }
//
//     fn exit(&mut self) {
//         self.exit = true;
//     }
// }
