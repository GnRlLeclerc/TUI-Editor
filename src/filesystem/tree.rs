use std::{
    cmp::Ordering,
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use ratatui::prelude::*;
use ratatui::widgets::Widget;
use slotmap::{SlotMap, new_key_type};
use tokio::sync::mpsc::Sender;

use super::{File, Folder};
use crate::EditorEvent;

new_key_type! {
    pub struct FileId;
    pub struct FolderId;
}

#[derive(Debug)]
pub struct Filetree {
    /// Whether the filetree view is currently open in the UI
    pub open: bool,
    /// Widget width
    pub width: usize,
    root: FolderId,
    sender: Sender<EditorEvent>,

    folders: SlotMap<FolderId, Folder>,
    files: SlotMap<FileId, File>,

    /// From paths of directories that have been opened once
    /// (and that are being watched) to the corresponding node ids.
    /// We don't store all paths to id mappings because of renaming and deletion.
    /// File watch events are dispatched by parent folder name.
    paths: HashMap<PathBuf, FolderId>,
}

impl Filetree {
    pub fn new(sender: Sender<EditorEvent>) -> Self {
        let path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let folder = Folder::new(path);
        let mut folders = SlotMap::with_key();
        let root = folders.insert(folder);

        Self {
            open: false,
            width: 40,
            root,
            sender,
            folders,
            files: SlotMap::with_key(),
            paths: HashMap::new(),
        }
    }

    /// Initialize the contents of a folder that is being opened for the first time.
    pub fn init_folder(&mut self, id: FolderId, files: Vec<File>, folders: Vec<Folder>) {
        let file_ids = files
            .into_iter()
            .map(|file| self.files.insert(file))
            .collect::<Vec<_>>();
        let folder_ids = folders
            .into_iter()
            .map(|folder| self.folders.insert(folder))
            .collect::<Vec<_>>();

        self.folders[id].child_files = file_ids;
        self.folders[id].child_folders = folder_ids;
        self.folders[id].init = true;
        self.paths.insert(self.folders[id].path.clone(), id);
    }

    pub fn load_root(&self) {
        self.load_folder(self.root);
    }

    /// Load the contents of a folder asynchronously in the background
    pub fn load_folder(&self, id: FolderId) {
        let sender = self.sender.clone();
        let path = self.folders[id].path.clone();
        tokio::spawn(async move {
            let mut files: Vec<File> = vec![];
            let mut folders: Vec<Folder> = vec![];

            match tokio::fs::read_dir(&path).await {
                Ok(mut entries) => {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.is_dir() {
                            folders.push(Folder::new(path));
                        } else {
                            files.push(File::new(path));
                        }
                    }

                    files.sort_by(|a, b| compare_names(&a.path, &b.path));
                    folders.sort_by(|a, b| compare_names(&a.path, &b.path));

                    if let Err(err) = sender
                        .send(EditorEvent::FolderLoaded { id, files, folders })
                        .await
                    {
                        log::error!("Failed to send folder loaded event: {}", err);
                    }
                }
                Err(err) => {
                    log::error!("Failed to read directory {}: {}", path.display(), err);
                }
            }
        });
    }
}
}

fn compare_names(a: &Path, b: &Path) -> Ordering {
    a.file_name()
        .unwrap_or_default()
        .cmp(b.file_name().unwrap_or_default())
}
