use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::attract::{SceneKind, attract_cycle, attract_scene, high_score_scene, logo_scene};
use crate::audio::{AudioManager, SoundCue};
use crate::demo::gameplay_demo_cycle;
use crate::game::World;
use crate::live::run_live;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    AudioDemo,
    Gameplay { frames: usize },
    PlayDemo { play_audio: bool, sleep: bool },
    PlayAttract { play_audio: bool, sleep: bool },
    PlayLive { play_audio: bool },
    Scene { kind: SceneKind },
    RomReport { path: PathBuf },
    Help,
}

#[derive(Debug, Clone)]
struct PlaybackBeat {
    text: String,
    cue: Option<SoundCue>,
    hold_ms: u64,
}

pub fn run() -> Result<()> {
    match parse_args(env::args().skip(1))? {
        Command::AudioDemo => {
            AudioManager::new().play_demo();
            Ok(())
        }
        Command::Gameplay { frames } => run_demo(frames),
        Command::PlayDemo { play_audio, sleep } => run_play_demo(play_audio, sleep),
        Command::PlayAttract { play_audio, sleep } => run_play_attract(play_audio, sleep),
        Command::PlayLive { play_audio } => run_live(play_audio),
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

fn run_play_attract(play_audio: bool, sleep: bool) -> Result<()> {
    play_sequence(
        attract_cycle().into_iter().map(|beat| PlaybackBeat {
            text: beat.scene().text(),
            cue: beat.cue,
            hold_ms: beat.hold_ms,
        }),
        play_audio,
        sleep,
    )
}

fn run_play_demo(play_audio: bool, sleep: bool) -> Result<()> {
    play_sequence(
        gameplay_demo_cycle().into_iter().map(|beat| PlaybackBeat {
            text: crate::render::render(&beat.world),
            cue: beat.cue,
            hold_ms: beat.hold_ms,
        }),
        play_audio,
        sleep,
    )
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

fn play_sequence<I>(beats: I, play_audio: bool, sleep: bool) -> Result<()>
where
    I: IntoIterator<Item = PlaybackBeat>,
{
    let audio = AudioManager::new();
    let mut stdout = io::stdout();

    for beat in beats {
        writeln!(stdout, "\x1B[2J\x1B[H{}", beat.text).context("writing playback frame")?;
        stdout.flush().context("flushing playback frame")?;

        if play_audio && let Some(cue) = beat.cue {
            audio.play_cue_blocking(cue);
        }

        if sleep {
            let elapsed_ms = beat.cue.map(SoundCue::duration_ms).unwrap_or(0);
            let remaining_ms = beat.hold_ms.saturating_sub(elapsed_ms);
            if remaining_ms > 0 {
                thread::sleep(Duration::from_millis(remaining_ms));
            }
        }
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
        "--play-demo" => {
            let mut play_audio = true;
            let mut sleep = true;

            for arg in args {
                match arg.as_str() {
                    "--mute" => play_audio = false,
                    "--no-sleep" => sleep = false,
                    other => bail!("unsupported --play-demo option: {other}"),
                }
            }

            Ok(Command::PlayDemo { play_audio, sleep })
        }
        "--play-attract" => {
            let mut play_audio = true;
            let mut sleep = true;

            for arg in args {
                match arg.as_str() {
                    "--mute" => play_audio = false,
                    "--no-sleep" => sleep = false,
                    other => bail!("unsupported --play-attract option: {other}"),
                }
            }

            Ok(Command::PlayAttract { play_audio, sleep })
        }
        "--play-live" => {
            let mut play_audio = true;

            for arg in args {
                match arg.as_str() {
                    "--mute" => play_audio = false,
                    other => bail!("unsupported --play-live option: {other}"),
                }
            }

            Ok(Command::PlayLive { play_audio })
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
    println!("  cargo run -- --play-demo");
    println!("  cargo run -- --play-demo --mute --no-sleep");
    println!("  cargo run -- --play-attract");
    println!("  cargo run -- --play-attract --mute --no-sleep");
    println!("  cargo run -- --play-live");
    println!("  cargo run -- --play-live --mute");
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
    fn parse_args_reads_play_attract_with_switches() {
        let command = parse_args(vec![
            String::from("--play-attract"),
            String::from("--mute"),
            String::from("--no-sleep"),
        ])
        .expect("parse args");
        assert_eq!(
            command,
            Command::PlayAttract {
                play_audio: false,
                sleep: false
            }
        );
    }

    #[test]
    fn parse_args_reads_play_demo_with_switches() {
        let command = parse_args(vec![
            String::from("--play-demo"),
            String::from("--mute"),
            String::from("--no-sleep"),
        ])
        .expect("parse args");
        assert_eq!(
            command,
            Command::PlayDemo {
                play_audio: false,
                sleep: false
            }
        );
    }

    #[test]
    fn parse_args_reads_play_live_with_mute() {
        let command = parse_args(vec![String::from("--play-live"), String::from("--mute")])
            .expect("parse args");
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

    #[test]
    fn parse_args_rejects_unknown_play_attract_option() {
        let error = parse_args(vec![
            String::from("--play-attract"),
            String::from("--warp-speed"),
        ])
        .expect_err("parse args");
        assert!(
            error
                .to_string()
                .contains("unsupported --play-attract option")
        );
    }

    #[test]
    fn parse_args_rejects_unknown_play_demo_option() {
        let error = parse_args(vec![
            String::from("--play-demo"),
            String::from("--warp-speed"),
        ])
        .expect_err("parse args");
        assert!(error.to_string().contains("unsupported --play-demo option"));
    }

    #[test]
    fn parse_args_rejects_unknown_play_live_option() {
        let error = parse_args(vec![
            String::from("--play-live"),
            String::from("--no-sleep"),
        ])
        .expect_err("parse args");
        assert!(error.to_string().contains("unsupported --play-live option"));
    }
}
