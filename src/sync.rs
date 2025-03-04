use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use filetime::{FileTime, set_file_times};
use walkdir::{DirEntry, WalkDir};

use crate::{DirSyncConfig, DirSyncError};

fn is_not_ignored(entry: &DirEntry) -> bool {
    let Some(file_name) = entry.file_name().to_str() else {
        println!(
            "WARNING Path {} cannot be converted to string",
            entry.path().display()
        );
        return true;
    };
    file_name != ".DS_Store"
        && file_name != "_nosync"
        && file_name != "node_modules"
        && !file_name.starts_with("._")
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
                if suffix.to_str() != Some(".dirsync") {
                    paths.push(suffix.to_owned());
                }
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
        println!("WARNING {} is newer than source", dst_file.display());
    }
    Ok(src_ts != dst_ts)
}

fn copy_modification_time(src_file: &Path, dst_file: &Path) -> Result<(), DirSyncError> {
    let src_metadata = src_file.metadata()?;
    set_file_times(
        dst_file,
        FileTime::from_last_access_time(&src_metadata),
        FileTime::from_last_modification_time(&src_metadata),
    )?;
    Ok(())
}

fn add_file(config: &DirSyncConfig, path: &Path) -> Result<(), DirSyncError> {
    let src_file = config.src_path(path);
    let dst_file = config.dst_path(path);

    if src_file.is_dir() {
        println!("+D   {}", dst_file.display());
        fs::create_dir(dst_file)?;
    } else {
        println!("+F   {}", dst_file.display());
        fs::copy(&src_file, &dst_file)?;
        copy_modification_time(&src_file, &dst_file)?;
    }

    Ok(())
}

fn update_file(config: &DirSyncConfig, path: &Path) -> Result<(), DirSyncError> {
    let src_file = config.src_path(path);
    let dst_file = config.dst_path(path);

    if src_file.is_dir() {
        println!("F->D {}", dst_file.display());
        fs::remove_file(&dst_file)?;
        fs::create_dir(&dst_file)?;
    } else {
        if dst_file.is_dir() {
            println!("D->F {}", dst_file.display());
            fs::remove_dir(&dst_file)?;
        } else {
            println!("UF   {}", dst_file.display());

            let meta = dst_file.metadata()?;
            if meta.permissions().readonly() {
                fs::remove_file(&dst_file)?
            }
        }

        fs::copy(&src_file, &dst_file)?;
        copy_modification_time(&src_file, &dst_file)?;
    }

    Ok(())
}

fn remove_file(config: &DirSyncConfig, path: &Path) -> Result<(), DirSyncError> {
    let dst_file = config.dst_path(path);

    if dst_file.is_dir() {
        println!("-D   {}", dst_file.display());
        fs::remove_dir(dst_file)?
    } else {
        println!("-F   {}", dst_file.display());
        fs::remove_file(dst_file)?
    }

    Ok(())
}

pub fn sync_dirs(config: &DirSyncConfig) -> Result<(), DirSyncError> {
    println!(
        "Syncing {} -> {}",
        config.src_dir.display(),
        config.dst_dir.display()
    );

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
