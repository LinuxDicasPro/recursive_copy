use std::{io, path::PathBuf};
use walkdir_minimal::WalkError;

#[derive(Debug)]
pub enum CopyError {
    Io(io::Error),
    Walk(WalkError),
    DepthExceeded(PathBuf),
    SymlinkLoop(PathBuf),
    SrcNotFound(PathBuf),
    DestNotDir(PathBuf),
    NotSupported(PathBuf),
}

impl From<io::Error> for CopyError {
    fn from(e: io::Error) -> Self {
        CopyError::Io(e)
    }
}
