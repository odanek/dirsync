use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use walkdir::{DirEntry, WalkDir};

use crate::{DirSyncConfig, DirSyncError};

fn is_not_ignored(entry: &DirEntry) -> bool {
    entry.file_name() != ".DS_Store"
}

fn get_paths(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(is_not_ignored)
        .filter_map(|v| v.ok())
        .for_each(|x| {
            paths.push(x.path().strip_prefix(dir).unwrap().to_owned());
        });

    paths
}

pub fn sync_dirs(config: &DirSyncConfig) -> Result<(), DirSyncError> {    
    let source_paths = get_paths(&config.source_dir);
    let target_paths = get_paths(&config.target_dir);

    let source_set: HashSet<&PathBuf> = source_paths.iter().collect();
    let target_set: HashSet<&PathBuf> = target_paths.iter().collect();

    let added_files = source_set.difference(&target_set);
    let removed_files = target_set.difference(&source_set);

    println!("=== ADDED");
    for path in added_files {
        println!("{}", path.display());
    }

    println!("=== REMOVED");
    for path in removed_files {
        println!("{}", path.display());
    }

    Ok(())
}
