use crate::{DirSyncError, DirSyncConfig};

pub fn sync_dirs(config: &DirSyncConfig) -> Result<(), DirSyncError> {
    println!("Syncing {} {}", config.source_dir, config.target_dir);
    Ok(())
}
