mod cli;
mod sync;

use std::path::{Path, PathBuf};

use cli::get_config;
use sync::sync_dirs;
use thiserror::Error;

pub struct DirSyncConfig {
    pub src_dir: PathBuf,
    pub dst_dir: PathBuf,
    pub dry_run: bool,
}

impl DirSyncConfig {
    fn src_path(&self, path: &Path) -> PathBuf {
        let mut buf = self.src_dir.clone();
        buf.push(path);
        buf
    }

    fn dst_path(&self, path: &Path) -> PathBuf {
        let mut buf = self.dst_dir.clone();
        buf.push(path);
        buf
    }
}

#[derive(Debug, Error)]
pub enum DirSyncError {
    #[error("Source and destination paths point to the same directory")]
    SameDirectory,
    #[error("Destination path is missing the .dirsync file")]
    SyncFileMissing,
    #[error("Source path in .dirsync file does not match the source path")]
    SyncPathMismatch,
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

fn check_path(path: &Path) -> Result<(), DirSyncError> {
    path.exists()
        .then_some(())
        .ok_or_else(|| DirSyncError::NonExistentPath(path.to_owned()))
}

fn check_sync(src: &Path, dst: &Path) -> Result<(), DirSyncError> {
    if src.canonicalize()? == dst.canonicalize()? {
        return Err(DirSyncError::SameDirectory);
    }
    let dirsync_path = dst.join(".dirsync");    
    if !dirsync_path.exists() {
        return Err(DirSyncError::SyncFileMissing);
    }

    let dirsync_src_path = std::fs::read_to_string(&dirsync_path)?;
    if dirsync_src_path != src.to_string_lossy() {
        return Err(DirSyncError::SyncPathMismatch);
    }

    Ok(())
}

fn validate(config: &DirSyncConfig) -> Result<(), DirSyncError> {
    check_path(&config.src_dir)?;
    check_path(&config.dst_dir)?;
    check_sync(&config.src_dir, &config.dst_dir)?;
    Ok(())
}

fn main() -> Result<(), DirSyncError> {
    let config = get_config()?;
    validate(&config)?;
    sync_dirs(&config)
}
