use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::live::run_live;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    PlayLive { play_audio: bool },
    RomReport { path: PathBuf },
    Help,
}

pub fn run() -> Result<()> {
    match parse_args(env::args().skip(1))? {
        Command::PlayLive { play_audio } => run_live(play_audio),
        Command::RomReport { path } => run_rom_report(&path),
        Command::Help => {
            print_help();
            Ok(())
        }
    }
}

fn run_rom_report(path: &Path) -> Result<()> {
    let report = crate::rom::scan_dir(path)
        .with_context(|| format!("failed to inspect ROM directory {}", path.display()))?;

    println!("{}", report.summary_line());

    if !report.missing.is_empty() {
        println!("Missing: {}", report.missing.join(", "));
    }

    if !report.unexpected.is_empty() {
        println!("Unexpected: {}", report.unexpected.join(", "));
    }

    Ok(())
}

fn parse_args<I>(args: I) -> Result<Command>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Ok(Command::PlayLive { play_audio: true });
    };

    match first.as_str() {
        "--help" | "-h" => Ok(Command::Help),
        "--mute" => parse_live_options(args, false),
        "--rom-report" => {
            let path = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("assets/roms/defender"));

            if args.next().is_some() {
                bail!("--rom-report only accepts one optional path");
            }

            Ok(Command::RomReport { path })
        }
        other => bail!("unknown argument: {other}"),
    }
}

fn parse_live_options<I>(args: I, play_audio: bool) -> Result<Command>
where
    I: IntoIterator<Item = String>,
{
    let extras = args.into_iter().collect::<Vec<_>>();
    if !extras.is_empty() {
        bail!("unsupported live-mode option: {}", extras.join(" "));
    }

    Ok(Command::PlayLive { play_audio })
}

fn print_help() {
    println!("defender");
    println!("  cargo run");
    println!("  cargo run -- --mute");
    println!("  cargo run -- --rom-report assets/roms/defender");
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Command, parse_args};

    #[test]
    fn parse_args_defaults_to_live_play() {
        let command = parse_args(Vec::<String>::new()).expect("parse args");
        assert_eq!(command, Command::PlayLive { play_audio: true });
    }

    #[test]
    fn parse_args_reads_live_mute_mode() {
        let command = parse_args(vec![String::from("--mute")]).expect("parse args");
        assert_eq!(command, Command::PlayLive { play_audio: false });
    }

    #[test]
    fn parse_args_uses_default_rom_directory() {
        let command = parse_args(vec![String::from("--rom-report")]).expect("parse args");
        assert_eq!(
            command,
            Command::RomReport {
                path: PathBuf::from("assets/roms/defender")
            }
        );
    }

    #[test]
    fn parse_args_uses_explicit_rom_directory() {
        let command = parse_args(vec![
            String::from("--rom-report"),
            String::from("/tmp/defender"),
        ])
        .expect("parse args");
        assert_eq!(
            command,
            Command::RomReport {
                path: PathBuf::from("/tmp/defender")
            }
        );
    }

    #[test]
    fn parse_args_rejects_unknown_flags() {
        let error = parse_args(vec![String::from("--unknown")]).expect_err("parse args");
        assert!(error.to_string().contains("unknown argument"));
    }

    #[test]
    fn parse_args_rejects_extra_live_options() {
        let error = parse_args(vec![String::from("--mute"), String::from("--rom-report")])
            .expect_err("parse args");
        assert!(error.to_string().contains("unsupported live-mode option"));
    }
}
