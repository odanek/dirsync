use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, builder::PathBufValueParser};

use crate::{DirSyncConfig, DirSyncError};

pub fn cli() -> Command {
    Command::new("DirSync")
        .author("Ondrej Danek, ondrej.danek@gmail.com")
        .version("0.1.0")
        .about("Directory synchronization utility")
        .arg(
            Arg::new("dry_run")
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("Dry run without executing any changes"),
        )
        .arg(
            Arg::new("src_dir")
                .index(1)
                .value_parser(PathBufValueParser::new())
                .required(true)
                .help("Source directory"),
        )
        .arg(
            Arg::new("dst_dir")
                .index(2)
                .value_parser(PathBufValueParser::new())
                .required(true)
                .help("Destination directory"),
        )
}

fn get_arg<'a, T: Clone + Send + Sync + 'static>(
    matches: &'a ArgMatches,
    name: &'_ str,
) -> Result<&'a T, DirSyncError> {
    matches
        .get_one::<T>(name)
        .ok_or(DirSyncError::MissingArgument(name.to_owned()))
}

pub fn get_config() -> Result<DirSyncConfig, DirSyncError> {
    let matches = cli().get_matches();
    let src_dir = get_arg::<PathBuf>(&matches, "src_dir")?.clone();
    let dst_dir = get_arg::<PathBuf>(&matches, "dst_dir")?.clone();
    let dry_run = *(get_arg::<bool>(&matches, "dry_run")?);

    Ok(DirSyncConfig {
        src_dir,
        dst_dir,
        dry_run,
    })
}
