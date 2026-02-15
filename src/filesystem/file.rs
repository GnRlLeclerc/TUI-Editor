use std::path::PathBuf;

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
}

impl File {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
