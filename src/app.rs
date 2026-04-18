use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::attract::{SceneKind, attract_scene, high_score_scene, logo_scene};
use crate::audio::AudioManager;
use crate::game::World;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    AudioDemo,
    Gameplay { frames: usize },
    Scene { kind: SceneKind },
    RomReport { path: PathBuf },
    Help,
}

pub fn run() -> Result<()> {
    match parse_args(env::args().skip(1))? {
        Command::AudioDemo => {
            AudioManager::new().play_demo();
            Ok(())
        }
        Command::Gameplay { frames } => run_demo(frames),
        Command::Scene { kind } => run_scene(kind),
        Command::RomReport { path } => run_rom_report(&path),
        Command::Help => {
            print_help();
            Ok(())
        }
    }
}

fn run_demo(frames: usize) -> Result<()> {
    let mut world = World::bootstrap();
    for _ in 1..frames {
        world.step();
    }
    println!("{}", crate::render::render(&world));
    Ok(())
}

fn run_scene(kind: SceneKind) -> Result<()> {
    let text = match kind {
        SceneKind::Logo => logo_scene().text(),
        SceneKind::Attract => {
            let mut world = World::bootstrap();
            for _ in 0..4 {
                world.step();
            }
            attract_scene(&world).text()
        }
        SceneKind::HighScore => high_score_scene().text(),
    };

    println!("{text}");
    Ok(())
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
        return Ok(Command::Scene {
            kind: SceneKind::Logo,
        });
    };

    match first.as_str() {
        "--help" | "-h" => Ok(Command::Help),
        "--audio-demo" => {
            if args.next().is_some() {
                bail!("--audio-demo does not accept extra arguments");
            }
            Ok(Command::AudioDemo)
        }
        "--scene" => {
            let Some(value) = args.next() else {
                bail!("--scene requires one of: logo, attract, high-score");
            };
            if args.next().is_some() {
                bail!("--scene only accepts one value");
            }

            let Some(kind) = SceneKind::parse(&value) else {
                bail!("unsupported scene: {value}");
            };

            Ok(Command::Scene { kind })
        }
        "--frames" => {
            let Some(value) = args.next() else {
                bail!("--frames requires a positive integer");
            };
            if args.next().is_some() {
                bail!("--frames only accepts one value");
            }

            let parsed = value
                .parse::<usize>()
                .context("--frames requires a positive integer")?;

            Ok(Command::Gameplay {
                frames: parsed.max(1),
            })
        }
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

fn print_help() {
    println!("defender");
    println!("  cargo run");
    println!("  cargo run -- --audio-demo");
    println!("  cargo run -- --scene logo");
    println!("  cargo run -- --scene attract");
    println!("  cargo run -- --scene high-score");
    println!("  cargo run -- --frames 8");
    println!("  cargo run -- --rom-report assets/roms/defender");
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::attract::SceneKind;

    use super::{Command, parse_args};

    #[test]
    fn parse_args_defaults_to_logo_scene() {
        let command = parse_args(Vec::<String>::new()).expect("parse args");
        assert_eq!(
            command,
            Command::Scene {
                kind: SceneKind::Logo
            }
        );
    }

    #[test]
    fn parse_args_reads_frame_override() {
        let command =
            parse_args(vec![String::from("--frames"), String::from("8")]).expect("parse args");
        assert_eq!(command, Command::Gameplay { frames: 8 });
    }

    #[test]
    fn parse_args_clamps_zero_frames_to_one() {
        let command =
            parse_args(vec![String::from("--frames"), String::from("0")]).expect("parse args");
        assert_eq!(command, Command::Gameplay { frames: 1 });
    }

    #[test]
    fn parse_args_reads_scene_selection() {
        let command =
            parse_args(vec![String::from("--scene"), String::from("attract")]).expect("parse args");
        assert_eq!(
            command,
            Command::Scene {
                kind: SceneKind::Attract
            }
        );
    }

    #[test]
    fn parse_args_reads_audio_demo() {
        let command = parse_args(vec![String::from("--audio-demo")]).expect("parse args");
        assert_eq!(command, Command::AudioDemo);
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
    fn parse_args_rejects_unknown_scene() {
        let error = parse_args(vec![String::from("--scene"), String::from("warp")])
            .expect_err("parse args");
        assert!(error.to_string().contains("unsupported scene"));
    }

    #[test]
    fn parse_args_rejects_extra_audio_demo_arguments() {
        let error = parse_args(vec![String::from("--audio-demo"), String::from("extra")])
            .expect_err("parse args");
        assert!(
            error
                .to_string()
                .contains("does not accept extra arguments")
        );
    }
}
