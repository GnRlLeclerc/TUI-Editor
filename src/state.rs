use std::{cell::Cell, io::stdout, path::PathBuf};

pub use config::Config;
use crossterm::{cursor::SetCursorStyle, execute};
pub use events::{EditorEvent, Events};
pub use filesystem::{File, FileId, FileSystem, Folder, FolderId};
use ratatui::layout::Position;

mod config;
mod events;
mod filesystem;

/// Currently displayed screen
#[derive(Debug, Default)]
pub enum Screen {
    /// Home page
    #[default]
    Alpha,
    /// Text editor
    Editor,
}

/// Editor mode
#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
    /// Cmdline widget open
    Command,
}

#[derive(Debug)]
pub struct State {
    pub screen: Screen,
    pub mode: Mode,
    pub events: Events,
    pub filesystem: FileSystem,
    pub config: Config,

    /// Cursor position determined at rendering time by the widgets
    /// This variable is read after rendering to update the cursor position
    pub cursor_pos: Cell<Position>,
    /// Cursor style determined at rendering time by the widgets
    /// This variable is read at rendering time for the widget that owns
    /// the focus to decide whether the cursor style needs to be changed
    pub cursor_style: Cell<SetCursorStyle>,

    pub exit: bool,
}

impl State {
    pub fn new(root_path: PathBuf) -> Self {
        let screen = Screen::default();
        let mode = Mode::default();
        let events = Events::new();
        let filesystem = FileSystem::new(root_path);
        let config = Config::default();

        // Load the root folder asynchronously
        filesystem.load_folder(events.editor_sender.clone(), filesystem.root);

        Self {
            screen,
            mode,
            events,
            filesystem,
            config,
            cursor_pos: Cell::new(Position::default()),
            cursor_style: Cell::new(SetCursorStyle::SteadyBlock),
            exit: false,
        }
    }

    /// Change the cursor style.
    pub fn set_cursor_style(&self, style: SetCursorStyle) {
        if self.cursor_style.get() == style {
            return;
        }
        if let Err(e) = execute!(stdout(), style) {
            log::error!("Failed to set cursor style: {}", e);
        } else {
            self.cursor_style.set(style);
        }
    }
}
