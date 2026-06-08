//! Platform-facing runtime configuration and launch boundaries.

use std::{fmt, path::PathBuf};

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
    Device,
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
    pub actor_script_path: Option<PathBuf>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            controls: ControlProfile::Planetoid,
            audio: AudioOutput::Device,
            mode: RunMode::Interactive,
            cmos_path: None,
            actor_script_path: None,
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
    dispatch_cli_classification(RuntimeCliClassifier::classify(args))
}

fn dispatch_cli_classification(classification: CliClassification) -> anyhow::Result<()> {
    match classification {
        CliClassification::Runtime(config) => crate::runtime::run(&config),
        CliClassification::ActorScriptCheck(request) => {
            crate::runtime::run_actor_script_check(request.path)
        }
        CliClassification::ActorSmoke => crate::runtime::run_actor_smoke(),
        CliClassification::ActorAttractSmoke => crate::runtime::run_actor_attract_smoke(),
        CliClassification::ActorPostGameSmoke => crate::runtime::run_actor_post_game_smoke(),
        CliClassification::ActorWgpuSmoke => crate::runtime::run_actor_wgpu_smoke(),
        CliClassification::Help => crate::runtime::run_help(),
        CliClassification::Error(error) => Err(error.into()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliClassification {
    Runtime(RuntimeConfig),
    ActorScriptCheck(ActorScriptCheckRequest),
    ActorSmoke,
    ActorAttractSmoke,
    ActorPostGameSmoke,
    ActorWgpuSmoke,
    Help,
    Error(CleanCliError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActorScriptCheckRequest {
    path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeCliClassifier;

impl RuntimeCliClassifier {
    fn classify<I>(args: I) -> CliClassification
    where
        I: IntoIterator<Item = String>,
    {
        let mut config = RuntimeConfig::default();
        let mut args = args.into_iter();
        let mut live_option_seen = false;

        while let Some(arg) = args.next() {
            match Self::apply_arg(&arg, &mut args, &mut config, live_option_seen) {
                ArgClassification::Runtime => live_option_seen = true,
                ArgClassification::ActorScriptCheck(request) => {
                    return CliClassification::ActorScriptCheck(request);
                }
                ArgClassification::ActorSmoke => return CliClassification::ActorSmoke,
                ArgClassification::ActorAttractSmoke => {
                    return CliClassification::ActorAttractSmoke;
                }
                ArgClassification::ActorPostGameSmoke => {
                    return CliClassification::ActorPostGameSmoke;
                }
                ArgClassification::ActorWgpuSmoke => return CliClassification::ActorWgpuSmoke,
                ArgClassification::Help => return CliClassification::Help,
                ArgClassification::Error(error) => return CliClassification::Error(error),
            }
        }

        if config.mode == RunMode::Smoke && config.actor_script_path.is_some() {
            return CliClassification::Error(CleanCliError::ActorScriptWithLiveSmoke);
        }

        CliClassification::Runtime(config)
    }

    fn apply_arg<I>(
        arg: &str,
        args: &mut I,
        config: &mut RuntimeConfig,
        live_option_seen: bool,
    ) -> ArgClassification
    where
        I: Iterator<Item = String>,
    {
        match arg {
            "--live-smoke" => {
                config.mode = RunMode::Smoke;
                ArgClassification::Runtime
            }
            "--actor-live" => {
                config.mode = RunMode::Interactive;
                ArgClassification::Runtime
            }
            "--actor-smoke" => no_arg_command(
                args,
                live_option_seen,
                "--actor-smoke",
                CleanCliError::TooManyActorSmokeArgs,
                ArgClassification::ActorSmoke,
            ),
            "--actor-attract-smoke" => no_arg_command(
                args,
                live_option_seen,
                "--actor-attract-smoke",
                CleanCliError::TooManyActorAttractSmokeArgs,
                ArgClassification::ActorAttractSmoke,
            ),
            "--actor-post-game-smoke" => no_arg_command(
                args,
                live_option_seen,
                "--actor-post-game-smoke",
                CleanCliError::TooManyActorPostGameSmokeArgs,
                ArgClassification::ActorPostGameSmoke,
            ),
            "--actor-wgpu-smoke" => no_arg_command(
                args,
                live_option_seen,
                "--actor-wgpu-smoke",
                CleanCliError::TooManyActorWgpuSmokeArgs,
                ArgClassification::ActorWgpuSmoke,
            ),
            "--actor-script-check" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--actor-script-check",
                    ));
                }
                let Some(path) = args.next() else {
                    return ArgClassification::Error(CleanCliError::MissingActorScriptCheckPath);
                };
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::TooManyActorScriptCheckArgs);
                }
                ArgClassification::ActorScriptCheck(ActorScriptCheckRequest {
                    path: PathBuf::from(path),
                })
            }
            "--help" | "-h" => ArgClassification::Help,
            "--renderer" | "--presentation" => {
                ArgClassification::Error(CleanCliError::RemovedRendererSelection)
            }
            "--input-profile" => {
                let Some(value) = args.next() else {
                    return ArgClassification::Error(CleanCliError::MissingInputProfile);
                };
                let Some(controls) = parse_control_profile(&value) else {
                    return ArgClassification::Error(CleanCliError::UnknownInputProfile(value));
                };
                config.controls = controls;
                ArgClassification::Runtime
            }
            "--mute" => {
                config.audio = AudioOutput::Disabled;
                ArgClassification::Runtime
            }
            "--cmos-path" => {
                let Some(value) = args.next() else {
                    return ArgClassification::Error(CleanCliError::MissingCmosPath);
                };
                config.cmos_path = Some(PathBuf::from(value));
                ArgClassification::Runtime
            }
            "--actor-script" => {
                let Some(value) = args.next() else {
                    return ArgClassification::Error(CleanCliError::MissingActorScriptPath);
                };
                config.actor_script_path = Some(PathBuf::from(value));
                ArgClassification::Runtime
            }
            _ => ArgClassification::Error(CleanCliError::UnknownArgument(String::from(arg))),
        }
    }
}

fn no_arg_command<I>(
    args: &mut I,
    live_option_seen: bool,
    command: &'static str,
    too_many_args: CleanCliError,
    classification: ArgClassification,
) -> ArgClassification
where
    I: Iterator<Item = String>,
{
    if live_option_seen {
        return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(command));
    }
    if args.next().is_some() {
        return ArgClassification::Error(too_many_args);
    }

    classification
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArgClassification {
    ActorScriptCheck(ActorScriptCheckRequest),
    ActorSmoke,
    ActorAttractSmoke,
    ActorPostGameSmoke,
    ActorWgpuSmoke,
    Runtime,
    Help,
    Error(CleanCliError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CleanCliError {
    MissingInputProfile,
    UnknownInputProfile(String),
    MissingCmosPath,
    MissingActorScriptPath,
    ActorScriptWithLiveSmoke,
    RemovedRendererSelection,
    UnknownArgument(String),
    LiveOptionsWithCommand(&'static str),
    MissingActorScriptCheckPath,
    TooManyActorScriptCheckArgs,
    TooManyActorSmokeArgs,
    TooManyActorAttractSmokeArgs,
    TooManyActorPostGameSmokeArgs,
    TooManyActorWgpuSmokeArgs,
}

impl fmt::Display for CleanCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInputProfile => write!(
                formatter,
                "--input-profile requires one of: planetoid, cabinet, test"
            ),
            Self::UnknownInputProfile(value) => write!(formatter, "unknown input profile: {value}"),
            Self::MissingCmosPath => write!(formatter, "--cmos-path requires a file path"),
            Self::MissingActorScriptPath => write!(
                formatter,
                "--actor-script requires a sectioned actor driver script path"
            ),
            Self::ActorScriptWithLiveSmoke => write!(
                formatter,
                "--actor-script is only supported by interactive actor live play and --actor-script-check; --live-smoke uses the built-in actor smoke script"
            ),
            Self::RemovedRendererSelection => write!(
                formatter,
                "renderer selection was removed; live play is wgpu-only"
            ),
            Self::UnknownArgument(value) => write!(formatter, "unknown argument: {value}"),
            Self::LiveOptionsWithCommand(command) => {
                write!(formatter, "live options cannot be combined with {command}")
            }
            Self::MissingActorScriptCheckPath => {
                write!(formatter, "--actor-script-check requires a script path")
            }
            Self::TooManyActorScriptCheckArgs => {
                write!(
                    formatter,
                    "--actor-script-check only accepts one script path"
                )
            }
            Self::TooManyActorSmokeArgs => {
                write!(
                    formatter,
                    "--actor-smoke does not accept additional arguments"
                )
            }
            Self::TooManyActorAttractSmokeArgs => write!(
                formatter,
                "--actor-attract-smoke does not accept additional arguments"
            ),
            Self::TooManyActorPostGameSmokeArgs => write!(
                formatter,
                "--actor-post-game-smoke does not accept additional arguments"
            ),
            Self::TooManyActorWgpuSmokeArgs => write!(
                formatter,
                "--actor-wgpu-smoke does not accept additional arguments"
            ),
        }
    }
}

impl std::error::Error for CleanCliError {}

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

    use super::{
        ActorScriptCheckRequest, AudioOutput, CleanCliError, CliClassification, ControlProfile,
        RunMode, RuntimeCliClassifier, RuntimeConfig,
    };

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| String::from(*value)).collect()
    }

    #[test]
    fn runtime_config_defaults_to_interactive_planetoid() {
        let config = RuntimeConfig::default();

        assert_eq!(config.controls, ControlProfile::Planetoid);
        assert_eq!(config.audio, AudioOutput::Device);
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
            RuntimeCliClassifier::classify(args(&[])),
            CliClassification::Runtime(RuntimeConfig::default())
        );
    }

    #[test]
    fn clean_cli_owns_interactive_configuration_flags() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&[
                "--input-profile",
                "cabinet",
                "--mute",
                "--cmos-path",
                "scores.bin",
                "--actor-script",
                "driver.script",
            ])),
            CliClassification::Runtime(RuntimeConfig {
                controls: ControlProfile::Cabinet,
                audio: AudioOutput::Disabled,
                mode: RunMode::Interactive,
                cmos_path: Some(PathBuf::from("scores.bin")),
                actor_script_path: Some(PathBuf::from("driver.script")),
            })
        );
    }

    #[test]
    fn clean_cli_owns_live_smoke_mode() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--live-smoke"])),
            CliClassification::Runtime(RuntimeConfig::smoke())
        );
    }

    #[test]
    fn clean_cli_owns_help_and_smoke_commands() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--help"])),
            CliClassification::Help
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--actor-smoke"])),
            CliClassification::ActorSmoke
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--actor-attract-smoke"])),
            CliClassification::ActorAttractSmoke
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--actor-post-game-smoke"])),
            CliClassification::ActorPostGameSmoke
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--actor-wgpu-smoke"])),
            CliClassification::ActorWgpuSmoke
        );
    }

    #[test]
    fn clean_cli_rejects_retired_game_smoke_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--game-smoke"])),
            CliClassification::Error(CleanCliError::UnknownArgument(String::from("--game-smoke")))
        );
    }

    #[test]
    fn clean_cli_owns_actor_script_check_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--actor-script-check", "driver.script"])),
            CliClassification::ActorScriptCheck(ActorScriptCheckRequest {
                path: PathBuf::from("driver.script"),
            })
        );
    }

    #[test]
    fn clean_cli_rejects_live_options_with_headless_commands() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--mute", "--actor-smoke"])),
            CliClassification::Error(CleanCliError::LiveOptionsWithCommand("--actor-smoke"))
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--live-smoke", "--actor-script", "driver"])),
            CliClassification::Error(CleanCliError::ActorScriptWithLiveSmoke)
        );
    }

    #[test]
    fn clean_cli_rejects_missing_values_and_extra_args() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--input-profile"])),
            CliClassification::Error(CleanCliError::MissingInputProfile)
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--input-profile", "unknown"])),
            CliClassification::Error(CleanCliError::UnknownInputProfile(String::from("unknown")))
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--cmos-path"])),
            CliClassification::Error(CleanCliError::MissingCmosPath)
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--actor-script"])),
            CliClassification::Error(CleanCliError::MissingActorScriptPath)
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--actor-script-check"])),
            CliClassification::Error(CleanCliError::MissingActorScriptCheckPath)
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--actor-wgpu-smoke", "extra"])),
            CliClassification::Error(CleanCliError::TooManyActorWgpuSmokeArgs)
        );
    }

    #[test]
    fn clean_cli_rejects_removed_or_unknown_options() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--renderer", "terminal"])),
            CliClassification::Error(CleanCliError::RemovedRendererSelection)
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--rom-report"])),
            CliClassification::Error(CleanCliError::UnknownArgument(String::from("--rom-report")))
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--fidelity-trace"])),
            CliClassification::Error(CleanCliError::UnknownArgument(String::from(
                "--fidelity-trace"
            )))
        );
    }

    #[test]
    fn clean_cli_error_messages_are_operator_facing() {
        assert_eq!(
            CleanCliError::LiveOptionsWithCommand("--actor-smoke").to_string(),
            "live options cannot be combined with --actor-smoke"
        );
        assert_eq!(
            CleanCliError::RemovedRendererSelection.to_string(),
            "renderer selection was removed; live play is wgpu-only"
        );
        assert_eq!(
            CleanCliError::UnknownArgument(String::from("--rom-report")).to_string(),
            "unknown argument: --rom-report"
        );
    }
}
