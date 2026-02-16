use crossterm::event::EventStream;
use futures::{StreamExt, stream::Fuse};
use tokio::sync::mpsc::{Receiver, Sender};

use super::{File, Folder, FolderId};

/// Internal editor events,
/// for background running tasks to make their
/// results available to the main thread.
#[derive(Debug)]
pub enum EditorEvent {
    FolderLoaded {
        id: FolderId,
        files: Vec<File>,
        folders: Vec<Folder>,
    },
}

/// Event channel listeners
#[derive(Debug)]
pub struct Events {
    pub term_events: Fuse<EventStream>,
    pub editor_events: Receiver<EditorEvent>,
    pub editor_sender: Sender<EditorEvent>,
}

impl Events {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(64);

        Self {
            term_events: EventStream::new().fuse(),
            editor_events: receiver,
            editor_sender: sender,
        }
    }
}
