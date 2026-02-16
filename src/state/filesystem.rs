use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use ropey::Rope;
use slotmap::{SlotMap, new_key_type};

mod file;
mod folder;

pub use file::File;
pub use folder::Folder;
use tokio::sync::mpsc::Sender;

use super::EditorEvent;

new_key_type! {
    pub struct FileId;
    pub struct FolderId;
}

/// All loaded files and folders
#[derive(Debug)]
pub struct FileSystem {
    /// Root folder
    pub root: FolderId,
    pub folders: SlotMap<FolderId, Folder>,
    pub files: SlotMap<FileId, File>,
    /// Shortcut to know which files currently contain open buffers.
    pub open_buffers: HashSet<FileId>,
    /// Opened files that are outside the filetree.
    /// or whose filetree has not yet been loaded
    pub file_paths: HashMap<PathBuf, FileId>,

    /// From paths of directories that have been opened once
    /// (and that are being watched) to the corresponding node ids.
    /// We don't store all paths to id mappings because of renaming and deletion.
    /// File watch events are dispatched by parent folder name.
    pub folder_paths: HashMap<PathBuf, FolderId>,
}

impl FileSystem {
    pub fn new(root_path: PathBuf) -> Self {
        let mut folders = SlotMap::with_key();
        let root = folders.insert(Folder::new(root_path));

        Self {
            root,
            folders,
            files: SlotMap::with_key(),
            open_buffers: HashSet::new(),
            file_paths: HashMap::new(),
            folder_paths: HashMap::new(),
        }
    }

    /// Load the contents of a folder asynchronously in the background
    pub fn load_folder(&self, sender: Sender<EditorEvent>, id: FolderId) {
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

    /// Initialize the contents of a folder that is being opened for the first time.
    pub fn init_folder(&mut self, id: FolderId, files: Vec<File>, folders: Vec<Folder>) {
        // Avoid overwriting existing children
        if self.folders[id].init {
            return;
        }
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
        self.folder_paths.insert(self.folders[id].path.clone(), id);
    }
}

fn compare_names(a: &Path, b: &Path) -> Ordering {
    a.file_name()
        .unwrap_or_default()
        .cmp(b.file_name().unwrap_or_default())
}
