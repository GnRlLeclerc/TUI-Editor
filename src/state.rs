use std::path::PathBuf;

pub use events::{EditorEvent, Events};
pub use filesystem::{File, FileId, FileSystem, Folder, FolderId};

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
}

impl State {
    pub fn new(root_path: PathBuf) -> Self {
        let screen = Screen::default();
        let mode = Mode::default();
        let events = Events::new();
        let filesystem = FileSystem::new(root_path);

        // Load the root folder asynchronously
        filesystem.load_folder(events.editor_sender.clone(), filesystem.root);

        Self {
            screen,
            mode,
            events,
            filesystem,
        }
    }
}
