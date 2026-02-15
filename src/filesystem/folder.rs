use super::{FileId, FolderId};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Folder {
    pub path: PathBuf,

    pub child_files: Vec<FileId>,
    pub child_folders: Vec<FolderId>,

    /// Current open state in UI
    pub open: bool,
    /// Whether the folder has already been loaded once
    pub init: bool,
}

impl Folder {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            child_files: vec![],
            child_folders: vec![],
            open: false,
            init: false,
        }
    }
}
