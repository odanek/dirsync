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
    let src_file = config.src_path(path);
    let dst_file = config.dst_path(path);

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

fn add_file(config: &DirSyncConfig, path: &Path) -> Result<(), DirSyncError> {
    let src_file = config.src_path(path);
    let dst_file = config.dst_path(path);

    if src_file.is_dir() {
        println!("CD   {:?}", dst_file);
        fs::create_dir(dst_file)?;
    } else {
        println!("CF   {:?}", dst_file);
        fs::copy(src_file, dst_file)?;    
    }

    Ok(())
}

fn update_file(config: &DirSyncConfig, path: &Path) -> Result<(), DirSyncError> {
    let src_file = config.src_path(path);
    let dst_file = config.dst_path(path);

    if src_file.is_dir() {
        println!("F->D {:?}", dst_file);
        fs::remove_file(&dst_file)?;
        fs::create_dir(&dst_file)?;
    } else {
        if dst_file.is_dir() {
            println!("D->F {:?}", dst_file);
            fs::remove_dir(&dst_file)?;
        } else {
            println!("UF   {:?}", dst_file);
        }
        fs::copy(&src_file, &dst_file)?;
    }

    Ok(())
}

fn remove_file(config: &DirSyncConfig, path: &Path) -> Result<(), DirSyncError> {
    let full_path = config.dst_path(path);        

    if full_path.is_dir() {
        println!("RD   {:?}", full_path);
        fs::remove_dir(full_path)?
    } else {
        println!("RF   {:?}", full_path);
        fs::remove_file(full_path)?
    }

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
    let removed_paths: Vec<&PathBuf> = dst_paths.iter().filter(|p| !src_set.contains(p)).collect();

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

    println!("=== Added ===");
    for path in &added_paths {
        println!("{}", path.display());        
    }
    println!();

    println!("=== Modified ===");
    for path in &modified_paths {
        println!("{}", path.display());
    }
    println!();

    println!("=== Removed ===");
    for path in &removed_paths {
        println!("{}", path.display());
    }
    println!();

    if !config.dry_run {
        println!("=== Executing ===");
        for path in removed_paths.iter().rev() {
            remove_file(config, path)?;
        }
    
        for path in &modified_paths {
            update_file(config, path)?;
        }
    
        for path in &added_paths {
            add_file(config, path)?;
        }    
    }

    Ok(())
}
