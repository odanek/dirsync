use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use filetime::FileTime;
use walkdir::{DirEntry, WalkDir};

use crate::{DirSyncConfig, DirSyncError};

fn is_not_ignored(entry: &DirEntry) -> bool {
    entry.file_name() != ".DS_Store"
}

fn concat_path(prefix: &Path, suffix: &Path) -> PathBuf {
    let mut buf = prefix.to_owned();
    buf.push(suffix);
    buf
}

fn get_paths(dir: &Path) -> Result<Vec<PathBuf>, DirSyncError> {
    let mut paths = Vec::new();
    let mut invalid_paths = Vec::new();

    WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(is_not_ignored)
        .filter_map(|v| v.ok())
        .for_each(|x| {
            if let Ok(suffix) = x.path().strip_prefix(dir) {
                paths.push(suffix.to_owned());
            } else {
                invalid_paths.push(x.path().to_owned());
            }
        });

    if invalid_paths.is_empty() {
        Ok(paths)
    } else {
        Err(DirSyncError::InvalidPaths(invalid_paths))
    }
}

fn is_modified(config: &DirSyncConfig, path: &Path) -> Result<bool, DirSyncError> {
    let src_file = concat_path(&config.src_dir, path);
    let dst_file = concat_path(&config.dst_dir, path);

    if src_file.is_dir() && dst_file.is_dir() {
        return Ok(false);
    }

    let src_metadata = src_file.metadata()?;
    let src_ts = FileTime::from_last_modification_time(&src_metadata).unix_seconds();

    let dst_metadata = dst_file.metadata()?;
    let dst_ts = FileTime::from_last_modification_time(&dst_metadata).unix_seconds();

    if src_ts < dst_ts {
        Err(DirSyncError::DstNewerThanSrc(dst_file))
    } else {
        Ok(src_ts != dst_ts)
    }
}

fn copy_file(config: &DirSyncConfig, path: &Path) -> Result<(), DirSyncError> {
    let src_file = concat_path(&config.src_dir, path);
    let dst_file = concat_path(&config.dst_dir, path);

    fs::copy(src_file, dst_file)?;

    Ok(())
}

pub fn sync_dirs(config: &DirSyncConfig) -> Result<(), DirSyncError> {
    println!("Syncing {:?} -> {:?}", config.src_dir, config.dst_dir);

    let src_paths = get_paths(&config.src_dir)?;
    let dst_paths = get_paths(&config.dst_dir)?;

    let src_set: HashSet<&PathBuf> = src_paths.iter().collect();
    let dst_set: HashSet<&PathBuf> = dst_paths.iter().collect();

    let mut added_paths = Vec::new();
    let mut modified_paths = Vec::new();
    let removed_paths = dst_set.difference(&src_set);

    for path in &src_paths {
        if dst_set.contains(path) {
            let modified = is_modified(config, path)?;
            if modified {
                modified_paths.push(path);
            }
        } else {
            added_paths.push(path);
        }
    }

    println!("=== ADDED");
    for path in added_paths {
        println!("{}", path.display());
        copy_file(config, path)?;
    }

    println!("=== MODIFIED");
    for path in modified_paths {
        println!("{}", path.display());
        copy_file(config, path)?;
    }

    println!("=== REMOVED");
    for path in removed_paths {
        println!("{}", path.display());
    }

    // TODO Pozor na zmenu soubor -> adresar a naopak
    // TODO Pozor kdyz smazu adresar tak uz nemusim mazat soubory a adresare v nem

    Ok(())
}
