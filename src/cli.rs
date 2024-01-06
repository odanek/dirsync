use clap::{Arg, Command, ArgMatches};

use crate::{DirSyncError, DirSyncConfig};

pub fn cli() -> Command {
    Command::new("DirSync")
        .author("Ondrej Danek, ondrej.danek@gmail.com")
        .version("0.1.0")
        .about("Directory synchronization utility")
        .arg(
            Arg::new("source_dir")
                .index(1)
                .required(true)
                .help("Source directory"),
        )
        .arg(
            Arg::new("target_dir")
                .index(2)
                .required(true)
                .help("Target directory"),
        )
}

fn get_arg<'a, T: Clone + Send + Sync + 'static>(matches: &'a ArgMatches, name: &'_ str) -> Result<&'a T, DirSyncError> {
    matches.get_one::<T>(name).ok_or(DirSyncError::MissingArgument(name.to_owned()))
}

pub fn get_config() -> Result<DirSyncConfig, DirSyncError> {
    let matches = cli().get_matches();
    let source_dir = get_arg::<String>(&matches, "source_dir")?.clone();
    let target_dir = get_arg::<String>(&matches, "target_dir")?.clone();

    Ok(DirSyncConfig {
        source_dir,
        target_dir
    })
}