//! Platform-facing runtime configuration and launch boundaries.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ControlProfile {
    #[default]
    Planetoid,
    Cabinet,
    Test,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AudioOutput {
    Disabled,
    #[default]
    Null,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RunMode {
    #[default]
    Interactive,
    Smoke,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub controls: ControlProfile,
    pub audio: AudioOutput,
    pub mode: RunMode,
    pub cmos_path: Option<PathBuf>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            controls: ControlProfile::Planetoid,
            audio: AudioOutput::Null,
            mode: RunMode::Interactive,
            cmos_path: None,
        }
    }
}

impl RuntimeConfig {
    pub fn smoke() -> Self {
        Self {
            mode: RunMode::Smoke,
            ..Self::default()
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    run_with_args(runtime_args())
}

pub fn run_with_config(config: RuntimeConfig) -> anyhow::Result<()> {
    crate::runtime::run(&config)
}

#[cfg(all(not(test), not(coverage)))]
fn runtime_args() -> impl Iterator<Item = String> {
    std::env::args().skip(1)
}

#[cfg(any(test, coverage))]
fn runtime_args() -> impl Iterator<Item = String> {
    std::iter::empty()
}

fn run_with_args<I>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = String>,
{
    match launch_from_args(args) {
        CliLaunch::AcceptedCli => crate::runtime::run_cli(),
        CliLaunch::CleanRuntime(config) => crate::runtime::run(&config),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliLaunch {
    AcceptedCli,
    CleanRuntime(RuntimeConfig),
}

fn launch_from_args<I>(args: I) -> CliLaunch
where
    I: IntoIterator<Item = String>,
{
    let mut config = RuntimeConfig::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--live-smoke" => config.mode = RunMode::Smoke,
            "--help" | "-h" => return CliLaunch::AcceptedCli,
            "--input-profile" => {
                let Some(value) = args.next() else {
                    return CliLaunch::AcceptedCli;
                };
                let Some(controls) = parse_control_profile(&value) else {
                    return CliLaunch::AcceptedCli;
                };
                config.controls = controls;
            }
            "--mute" => config.audio = AudioOutput::Disabled,
            "--cmos-path" => {
                let Some(value) = args.next() else {
                    return CliLaunch::AcceptedCli;
                };
                config.cmos_path = Some(PathBuf::from(value));
            }
            _ => return CliLaunch::AcceptedCli,
        }
    }

    CliLaunch::CleanRuntime(config)
}

fn parse_control_profile(value: &str) -> Option<ControlProfile> {
    match value {
        "planetoid" => Some(ControlProfile::Planetoid),
        "cabinet" => Some(ControlProfile::Cabinet),
        "test" => Some(ControlProfile::Test),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{AudioOutput, CliLaunch, ControlProfile, RunMode, RuntimeConfig};

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| String::from(*value)).collect()
    }

    #[test]
    fn runtime_config_defaults_to_interactive_planetoid() {
        let config = RuntimeConfig::default();

        assert_eq!(config.controls, ControlProfile::Planetoid);
        assert_eq!(config.audio, AudioOutput::Null);
        assert_eq!(config.mode, RunMode::Interactive);
        assert!(config.cmos_path.is_none());
    }

    #[test]
    fn smoke_config_selects_smoke_mode() {
        assert_eq!(RuntimeConfig::smoke().mode, RunMode::Smoke);
    }

    #[test]
    fn clean_cli_owns_default_interactive_launch() {
        assert_eq!(
            super::launch_from_args(args(&[])),
            CliLaunch::CleanRuntime(RuntimeConfig::default())
        );
    }

    #[test]
    fn clean_cli_owns_interactive_configuration_flags() {
        assert_eq!(
            super::launch_from_args(args(&[
                "--input-profile",
                "cabinet",
                "--mute",
                "--cmos-path",
                "scores.bin",
            ])),
            CliLaunch::CleanRuntime(RuntimeConfig {
                controls: ControlProfile::Cabinet,
                audio: AudioOutput::Disabled,
                mode: RunMode::Interactive,
                cmos_path: Some(PathBuf::from("scores.bin")),
            })
        );
    }

    #[test]
    fn clean_cli_owns_live_smoke_launch() {
        assert_eq!(
            super::launch_from_args(args(&["--live-smoke"])),
            CliLaunch::CleanRuntime(RuntimeConfig::smoke())
        );
    }

    #[test]
    fn clean_cli_owns_live_smoke_configuration_flags() {
        assert_eq!(
            super::launch_from_args(args(&[
                "--input-profile",
                "cabinet",
                "--mute",
                "--cmos-path",
                "scores.bin",
                "--live-smoke",
            ])),
            CliLaunch::CleanRuntime(RuntimeConfig {
                controls: ControlProfile::Cabinet,
                audio: AudioOutput::Disabled,
                mode: RunMode::Smoke,
                cmos_path: Some(PathBuf::from("scores.bin")),
            })
        );
    }

    #[test]
    fn clean_cli_accepts_test_profile_for_live_smoke() {
        assert_eq!(
            super::launch_from_args(args(&["--live-smoke", "--input-profile", "test"])),
            CliLaunch::CleanRuntime(RuntimeConfig {
                controls: ControlProfile::Test,
                audio: AudioOutput::Null,
                mode: RunMode::Smoke,
                cmos_path: None,
            })
        );
    }

    #[test]
    fn clean_cli_delegates_help_to_accepted_cli() {
        for values in [vec!["--help"], vec!["-h"], vec!["--mute", "--help"]] {
            assert_eq!(
                super::launch_from_args(args(&values)),
                CliLaunch::AcceptedCli
            );
        }
    }

    #[test]
    fn clean_cli_delegates_historical_commands() {
        assert_eq!(
            super::launch_from_args(args(&["--fidelity-trace", "1"])),
            CliLaunch::AcceptedCli
        );
        assert_eq!(
            super::launch_from_args(args(&["--live-smoke", "--fidelity-trace", "1"])),
            CliLaunch::AcceptedCli
        );
    }

    #[test]
    fn clean_cli_delegates_malformed_live_args() {
        for values in [
            vec!["--live-smoke", "--input-profile"],
            vec!["--live-smoke", "--input-profile", "unknown"],
            vec!["--live-smoke", "--cmos-path"],
            vec!["--live-smoke", "--unknown"],
            vec!["--input-profile"],
            vec!["--input-profile", "unknown"],
            vec!["--cmos-path"],
            vec!["--unknown"],
        ] {
            assert_eq!(
                super::launch_from_args(args(&values)),
                CliLaunch::AcceptedCli
            );
        }
    }

    #[test]
    fn runtime_entrypoint_runs_clean_default_live_config() {
        super::run().expect("runtime bridge should run clean default live under tests");
    }

    #[test]
    fn live_smoke_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&["--live-smoke", "--input-profile", "test", "--mute"]))
            .expect("clean live-smoke CLI should run through configured runtime");
    }

    #[test]
    fn interactive_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&[
            "--input-profile",
            "test",
            "--mute",
            "--cmos-path",
            "scores.bin",
        ]))
        .expect("clean interactive CLI should run through configured runtime");
    }

    #[test]
    fn configured_runtime_entrypoint_accepts_clean_interactive_config() {
        super::run_with_config(RuntimeConfig::default())
            .expect("configured runtime bridge should run live under tests");
    }

    #[test]
    fn configured_runtime_entrypoint_accepts_clean_smoke_config() {
        super::run_with_config(RuntimeConfig::smoke())
            .expect("configured runtime bridge should run smoke under tests");
    }
}
