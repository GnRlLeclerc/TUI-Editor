use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
};
use futures::StreamExt;
use ratatui::prelude::*;
use std::{io::stdout, path::PathBuf};

use crate::{
    Widget,
    screens::{AlphaScreen, EditorScreen},
    state::{EditorEvent, Screen, State},
};

#[derive(Debug)]
pub struct App {
    /// Global app state
    state: State,

    // Screens
    editor: EditorScreen,
    alpha: AlphaScreen,
}

impl App {
    pub fn new(path: PathBuf) -> Self {
        Self {
            state: State::new(path),
            editor: EditorScreen::new(),
            alpha: AlphaScreen::new(),
        }
    }

    /// Run the event loop until exit
    pub async fn run(&mut self) -> std::io::Result<()> {
        execute!(stdout(), EnableMouseCapture)?;
        let mut terminal = ratatui::init();
        while !self.state.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await;
        }
        ratatui::restore();
        execute!(stdout(), DisableMouseCapture)
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let buffer = frame.buffer_mut();

        match self.state.screen {
            Screen::Alpha => self.alpha.render(area, buffer, &self.state),
            Screen::Editor => self.editor.render(area, buffer, &self.state),
        }

        let position = self.state.cursor_pos.get();
        frame.set_cursor_position(position);
    }

    pub async fn handle_events(&mut self) {
        let events = &mut self.state.events;
        tokio::select! {
            Some(Ok(event)) = events.term_events.next() => {
                self.handle_term_event(event);
            }
            Some(event) = events.editor_events.recv() => {
                self.handle_editor_event(event).await;
            }
        }
    }

    async fn handle_editor_event(&mut self, event: EditorEvent) {
        match event {
            EditorEvent::FolderLoaded { id, files, folders } => {
                self.state.filesystem.init_folder(id, files, folders);
            }
        }
    }

    fn handle_term_event(&mut self, event: Event) {
        // TODO: delegate to screens, which will delegate based on focus / hitboxes
    }
}
