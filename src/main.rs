mod cli;
mod sync;

use std::path::{Path, PathBuf};

use cli::get_config;
use sync::sync_dirs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DirSyncError {
    #[error("Source and destination paths point to the same directory")]
    SameDirectory,
    #[error("Missing value for argument {0}")]
    MissingArgument(String),
    #[error("Path does not exist {0}")]
    NonExistentPath(PathBuf),
    #[error("IO error encountered")]
    IoError(#[from] std::io::Error),
    #[error("Invalid paths detected")]
    InvalidPaths(Vec<PathBuf>),
    #[error("Destination file {0} is newer than source file")]
    DstNewerThanSrc(PathBuf),
}

pub struct DirSyncConfig {
    pub src_dir: PathBuf,
    pub dst_dir: PathBuf,
    pub dry_run: bool,
}

fn check_path(path: &Path) -> Result<(), DirSyncError> {
    if !path.exists() {
        Err(DirSyncError::NonExistentPath(path.to_owned()))
    } else {
        Ok(())
    }
}

fn main() -> Result<(), DirSyncError> {
    let config = get_config()?;
    check_path(&config.src_dir)?;
    check_path(&config.dst_dir)?;
    sync_dirs(&config)
}
