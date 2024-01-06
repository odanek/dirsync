mod cli;
mod sync;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DirSyncError {
    #[error("Missing value for argument {0}")]
    MissingArgument(String)
}

pub struct DirSyncConfig {
    source_dir: String,
    target_dir: String,
}

fn main() -> Result<(), DirSyncError> {
    let config = cli::get_config()?;
    sync::sync_dirs(&config)?;
    Ok(())
}
