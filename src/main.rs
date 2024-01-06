mod cli;
mod sync;

use std::path::{PathBuf, Path};

use cli::get_config;
use sync::sync_dirs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DirSyncError {
    #[error("Missing value for argument {0}")]
    MissingArgument(String),
    #[error("Path does not exist {0}")]
    InvalidPath(PathBuf),
}

pub struct DirSyncConfig {
    source_dir: PathBuf,
    target_dir: PathBuf,
}

fn check_path(path: &Path) -> Result<(), DirSyncError> {
    if !path.exists() {
        return Err(DirSyncError::InvalidPath(path.to_owned()));
    }
    Ok(())
}

fn main() -> Result<(), DirSyncError> {
    let config = get_config()?;
    check_path(&config.source_dir)?;
    check_path(&config.target_dir)?;
    sync_dirs(&config)
}
