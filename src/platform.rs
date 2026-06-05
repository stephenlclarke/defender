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
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            controls: ControlProfile::Planetoid,
            audio: AudioOutput::Device,
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
    dispatch_cli_classification(RuntimeCliClassifier::classify(args))
}

fn dispatch_cli_classification(classification: CliClassification) -> anyhow::Result<()> {
    match classification {
        CliClassification::RomReport(request) => crate::runtime::run_rom_report(request.path),
        CliClassification::VerifyRoms(request) => crate::runtime::run_verify_roms(request.path),
        CliClassification::GameSmoke => crate::runtime::run_game_smoke(),
        CliClassification::FidelityTrace(request) => {
            crate::runtime::run_fidelity_trace(request.frame_count)
        }
        CliClassification::FidelityTraceInputs(request) => {
            crate::runtime::run_fidelity_trace_inputs(request.script)
        }
        CliClassification::FidelityTraceInputsFile(request) => {
            crate::runtime::run_fidelity_trace_inputs_file(request.path)
        }
        CliClassification::FidelityTraceCheck(request) => {
            crate::runtime::run_fidelity_trace_check(request.inputs_path, request.expected_path)
        }
        CliClassification::FidelityTraceFixtureDirectory(request) => {
            crate::runtime::run_fidelity_trace_check_dir(request.path)
        }
        CliClassification::FidelityReferenceTraceFixtureDirectory(request) => {
            crate::runtime::run_fidelity_reference_trace_check_dir(request.path)
        }
        CliClassification::FidelityScenarioList => crate::runtime::run_fidelity_scenario_list(),
        CliClassification::FidelityScenarioInputWriter(request) => {
            crate::runtime::run_fidelity_scenario_input_writer(request.path)
        }
        CliClassification::Runtime(config) => crate::runtime::run(&config),
        CliClassification::Help => crate::runtime::run_help(),
        CliClassification::Error(error) => Err(error.into()),
        CliClassification::ActorSmoke => crate::runtime::run_actor_smoke(),
        CliClassification::ActorWgpuSmoke => crate::runtime::run_actor_wgpu_smoke(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliClassification {
    RomReport(RomReportRequest),
    VerifyRoms(VerifyRomsRequest),
    FidelityTrace(FidelityTraceRequest),
    FidelityTraceInputs(FidelityTraceInputsRequest),
    FidelityTraceInputsFile(FidelityTraceInputsFileRequest),
    FidelityTraceCheck(FidelityTraceCheckRequest),
    FidelityTraceFixtureDirectory(FidelityTraceFixtureDirectoryRequest),
    FidelityReferenceTraceFixtureDirectory(FidelityReferenceTraceFixtureDirectoryRequest),
    FidelityScenarioList,
    FidelityScenarioInputWriter(ScenarioInputWriterRequest),
    Runtime(RuntimeConfig),
    GameSmoke,
    ActorSmoke,
    ActorWgpuSmoke,
    Help,
    Error(CleanCliError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RomReportRequest {
    path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifyRomsRequest {
    path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FidelityTraceRequest {
    frame_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FidelityTraceInputsRequest {
    script: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FidelityTraceInputsFileRequest {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FidelityTraceCheckRequest {
    inputs_path: PathBuf,
    expected_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FidelityTraceFixtureDirectoryRequest {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FidelityReferenceTraceFixtureDirectoryRequest {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScenarioInputWriterRequest {
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
                ArgClassification::RomReport(request) => {
                    return CliClassification::RomReport(request);
                }
                ArgClassification::VerifyRoms(request) => {
                    return CliClassification::VerifyRoms(request);
                }
                ArgClassification::FidelityTrace(request) => {
                    return CliClassification::FidelityTrace(request);
                }
                ArgClassification::FidelityTraceInputs(request) => {
                    return CliClassification::FidelityTraceInputs(request);
                }
                ArgClassification::FidelityTraceInputsFile(request) => {
                    return CliClassification::FidelityTraceInputsFile(request);
                }
                ArgClassification::FidelityTraceCheck(request) => {
                    return CliClassification::FidelityTraceCheck(request);
                }
                ArgClassification::FidelityTraceFixtureDirectory(request) => {
                    return CliClassification::FidelityTraceFixtureDirectory(request);
                }
                ArgClassification::FidelityReferenceTraceFixtureDirectory(request) => {
                    return CliClassification::FidelityReferenceTraceFixtureDirectory(request);
                }
                ArgClassification::FidelityScenarioList => {
                    return CliClassification::FidelityScenarioList;
                }
                ArgClassification::FidelityScenarioInputWriter(request) => {
                    return CliClassification::FidelityScenarioInputWriter(request);
                }
                ArgClassification::GameSmoke => return CliClassification::GameSmoke,
                ArgClassification::ActorSmoke => return CliClassification::ActorSmoke,
                ArgClassification::ActorWgpuSmoke => return CliClassification::ActorWgpuSmoke,
                ArgClassification::Help => return CliClassification::Help,
                ArgClassification::Error(error) => {
                    return CliClassification::Error(error);
                }
            }
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
            "--game-smoke" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--game-smoke",
                    ));
                }
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::TooManyGameSmokeArgs);
                }
                ArgClassification::GameSmoke
            }
            "--actor-smoke" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--actor-smoke",
                    ));
                }
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::TooManyActorSmokeArgs);
                }
                ArgClassification::ActorSmoke
            }
            "--actor-wgpu-smoke" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--actor-wgpu-smoke",
                    ));
                }
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::TooManyActorWgpuSmokeArgs);
                }
                ArgClassification::ActorWgpuSmoke
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
            "--rom-report" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--rom-report",
                    ));
                }
                let path = match args.next() {
                    Some(value) if value.starts_with('-') => {
                        return ArgClassification::Error(CleanCliError::RomReportPathCannotBeFlag(
                            value,
                        ));
                    }
                    Some(value) => Some(PathBuf::from(value)),
                    None => None,
                };
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::TooManyRomReportArgs);
                }
                ArgClassification::RomReport(RomReportRequest { path })
            }
            "--verify-roms" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--verify-roms",
                    ));
                }
                let Some(path) = args.next() else {
                    return ArgClassification::Error(CleanCliError::MissingVerifyRomsPath);
                };
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::TooManyVerifyRomsArgs);
                }
                ArgClassification::VerifyRoms(VerifyRomsRequest {
                    path: PathBuf::from(path),
                })
            }
            "--fidelity-trace" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--fidelity-trace",
                    ));
                }
                let frame_count = match args.next() {
                    Some(value) => match parse_fidelity_trace_frame_count(&value) {
                        Ok(frame_count) => frame_count,
                        Err(error) => return ArgClassification::Error(error),
                    },
                    None => 1,
                };
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::TooManyFidelityTraceArgs);
                }
                ArgClassification::FidelityTrace(FidelityTraceRequest { frame_count })
            }
            "--fidelity-trace-inputs" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--fidelity-trace-inputs",
                    ));
                }
                let Some(script) = args.next() else {
                    return ArgClassification::Error(
                        CleanCliError::FidelityTraceInputsMissingScript,
                    );
                };
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::FidelityTraceInputsExtraArgs);
                }
                ArgClassification::FidelityTraceInputs(FidelityTraceInputsRequest { script })
            }
            "--fidelity-trace-inputs-file" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--fidelity-trace-inputs-file",
                    ));
                }
                let Some(path) = args.next() else {
                    return ArgClassification::Error(
                        CleanCliError::FidelityTraceInputsFileMissingPath,
                    );
                };
                if args.next().is_some() {
                    return ArgClassification::Error(
                        CleanCliError::FidelityTraceInputsFileExtraArgs,
                    );
                }
                ArgClassification::FidelityTraceInputsFile(FidelityTraceInputsFileRequest {
                    path: PathBuf::from(path),
                })
            }
            "--fidelity-check-trace" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--fidelity-check-trace",
                    ));
                }
                let Some(inputs_path) = args.next() else {
                    return ArgClassification::Error(CleanCliError::FidelityCheckTraceMissingPaths);
                };
                let Some(expected_path) = args.next() else {
                    return ArgClassification::Error(CleanCliError::FidelityCheckTraceMissingPaths);
                };
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::FidelityCheckTraceExtraArgs);
                }
                ArgClassification::FidelityTraceCheck(FidelityTraceCheckRequest {
                    inputs_path: PathBuf::from(inputs_path),
                    expected_path: PathBuf::from(expected_path),
                })
            }
            "--fidelity-check-trace-dir" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--fidelity-check-trace-dir",
                    ));
                }
                let Some(path) = args.next() else {
                    return ArgClassification::Error(
                        CleanCliError::FidelityCheckTraceDirMissingPath,
                    );
                };
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::FidelityCheckTraceDirExtraArgs);
                }
                ArgClassification::FidelityTraceFixtureDirectory(
                    FidelityTraceFixtureDirectoryRequest {
                        path: PathBuf::from(path),
                    },
                )
            }
            "--fidelity-check-reference-trace-dir" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--fidelity-check-reference-trace-dir",
                    ));
                }
                let Some(path) = args.next() else {
                    return ArgClassification::Error(
                        CleanCliError::FidelityCheckReferenceTraceDirMissingPath,
                    );
                };
                if args.next().is_some() {
                    return ArgClassification::Error(
                        CleanCliError::FidelityCheckReferenceTraceDirExtraArgs,
                    );
                }
                ArgClassification::FidelityReferenceTraceFixtureDirectory(
                    FidelityReferenceTraceFixtureDirectoryRequest {
                        path: PathBuf::from(path),
                    },
                )
            }
            "--fidelity-list-scenarios" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--fidelity-list-scenarios",
                    ));
                }
                if args.next().is_some() {
                    return ArgClassification::Error(CleanCliError::FidelityListScenariosExtraArgs);
                }
                ArgClassification::FidelityScenarioList
            }
            "--fidelity-write-scenario-inputs" => {
                if live_option_seen {
                    return ArgClassification::Error(CleanCliError::LiveOptionsWithCommand(
                        "--fidelity-write-scenario-inputs",
                    ));
                }
                let Some(path) = args.next() else {
                    return ArgClassification::Error(
                        CleanCliError::FidelityWriteScenarioInputsMissingPath,
                    );
                };
                if args.next().is_some() {
                    return ArgClassification::Error(
                        CleanCliError::FidelityWriteScenarioInputsExtraArgs,
                    );
                }
                ArgClassification::FidelityScenarioInputWriter(ScenarioInputWriterRequest {
                    path: PathBuf::from(path),
                })
            }
            _ => ArgClassification::Error(CleanCliError::UnknownArgument(String::from(arg))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArgClassification {
    RomReport(RomReportRequest),
    VerifyRoms(VerifyRomsRequest),
    FidelityTrace(FidelityTraceRequest),
    FidelityTraceInputs(FidelityTraceInputsRequest),
    FidelityTraceInputsFile(FidelityTraceInputsFileRequest),
    FidelityTraceCheck(FidelityTraceCheckRequest),
    FidelityTraceFixtureDirectory(FidelityTraceFixtureDirectoryRequest),
    FidelityReferenceTraceFixtureDirectory(FidelityReferenceTraceFixtureDirectoryRequest),
    FidelityScenarioList,
    FidelityScenarioInputWriter(ScenarioInputWriterRequest),
    GameSmoke,
    ActorSmoke,
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
    RemovedRendererSelection,
    UnknownArgument(String),
    LiveOptionsWithCommand(&'static str),
    RomReportPathCannotBeFlag(String),
    TooManyRomReportArgs,
    MissingVerifyRomsPath,
    TooManyVerifyRomsArgs,
    TooManyGameSmokeArgs,
    TooManyActorSmokeArgs,
    TooManyActorWgpuSmokeArgs,
    InvalidFidelityTraceFrameCount { value: String, error: String },
    NonPositiveFidelityTraceFrameCount,
    TooManyFidelityTraceArgs,
    FidelityTraceInputsMissingScript,
    FidelityTraceInputsExtraArgs,
    FidelityTraceInputsFileMissingPath,
    FidelityTraceInputsFileExtraArgs,
    FidelityCheckTraceMissingPaths,
    FidelityCheckTraceExtraArgs,
    FidelityCheckTraceDirMissingPath,
    FidelityCheckTraceDirExtraArgs,
    FidelityCheckReferenceTraceDirMissingPath,
    FidelityCheckReferenceTraceDirExtraArgs,
    FidelityListScenariosExtraArgs,
    FidelityWriteScenarioInputsMissingPath,
    FidelityWriteScenarioInputsExtraArgs,
}

impl fmt::Display for CleanCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInputProfile => {
                write!(
                    formatter,
                    "--input-profile requires one of: planetoid, cabinet, test"
                )
            }
            Self::UnknownInputProfile(value) => {
                write!(formatter, "unknown input profile: {value}")
            }
            Self::MissingCmosPath => write!(formatter, "--cmos-path requires a file path"),
            Self::RemovedRendererSelection => {
                write!(
                    formatter,
                    "renderer selection was removed; live play is wgpu-only"
                )
            }
            Self::UnknownArgument(value) => write!(formatter, "unknown argument: {value}"),
            Self::LiveOptionsWithCommand(command) => {
                write!(formatter, "live options cannot be combined with {command}")
            }
            Self::RomReportPathCannotBeFlag(value) => {
                write!(
                    formatter,
                    "--rom-report optional path cannot be another flag: {value}"
                )
            }
            Self::TooManyRomReportArgs => {
                write!(formatter, "--rom-report only accepts one optional path")
            }
            Self::MissingVerifyRomsPath => {
                write!(formatter, "--verify-roms requires a ROM directory path")
            }
            Self::TooManyVerifyRomsArgs => {
                write!(formatter, "--verify-roms only accepts one directory path")
            }
            Self::TooManyGameSmokeArgs => {
                write!(
                    formatter,
                    "--game-smoke does not accept additional arguments"
                )
            }
            Self::TooManyActorSmokeArgs => {
                write!(
                    formatter,
                    "--actor-smoke does not accept additional arguments"
                )
            }
            Self::TooManyActorWgpuSmokeArgs => {
                write!(
                    formatter,
                    "--actor-wgpu-smoke does not accept additional arguments"
                )
            }
            Self::InvalidFidelityTraceFrameCount { value, error } => {
                write!(
                    formatter,
                    "invalid --fidelity-trace frame count: {value}: {error}"
                )
            }
            Self::NonPositiveFidelityTraceFrameCount => {
                write!(
                    formatter,
                    "--fidelity-trace frame count must be greater than zero"
                )
            }
            Self::TooManyFidelityTraceArgs => {
                write!(
                    formatter,
                    "--fidelity-trace only accepts one optional frame count"
                )
            }
            Self::FidelityTraceInputsMissingScript => {
                write!(
                    formatter,
                    "--fidelity-trace-inputs requires a semicolon-separated input script"
                )
            }
            Self::FidelityTraceInputsExtraArgs => {
                write!(
                    formatter,
                    "--fidelity-trace-inputs only accepts one input script"
                )
            }
            Self::FidelityTraceInputsFileMissingPath => {
                write!(
                    formatter,
                    "--fidelity-trace-inputs-file requires a trace input script path"
                )
            }
            Self::FidelityTraceInputsFileExtraArgs => {
                write!(
                    formatter,
                    "--fidelity-trace-inputs-file only accepts one path"
                )
            }
            Self::FidelityCheckTraceMissingPaths => {
                write!(
                    formatter,
                    "--fidelity-check-trace requires an input script path and expected trace path"
                )
            }
            Self::FidelityCheckTraceExtraArgs => {
                write!(
                    formatter,
                    "--fidelity-check-trace only accepts an input script path and expected trace path"
                )
            }
            Self::FidelityCheckTraceDirMissingPath => {
                write!(
                    formatter,
                    "--fidelity-check-trace-dir requires a fixture directory path"
                )
            }
            Self::FidelityCheckTraceDirExtraArgs => {
                write!(
                    formatter,
                    "--fidelity-check-trace-dir only accepts one fixture directory path"
                )
            }
            Self::FidelityCheckReferenceTraceDirMissingPath => {
                write!(
                    formatter,
                    "--fidelity-check-reference-trace-dir requires a reference fixture directory path"
                )
            }
            Self::FidelityCheckReferenceTraceDirExtraArgs => {
                write!(
                    formatter,
                    "--fidelity-check-reference-trace-dir only accepts one reference fixture directory path"
                )
            }
            Self::FidelityListScenariosExtraArgs => {
                write!(
                    formatter,
                    "--fidelity-list-scenarios does not accept extra arguments"
                )
            }
            Self::FidelityWriteScenarioInputsMissingPath => {
                write!(
                    formatter,
                    "--fidelity-write-scenario-inputs requires an output directory path"
                )
            }
            Self::FidelityWriteScenarioInputsExtraArgs => {
                write!(
                    formatter,
                    "--fidelity-write-scenario-inputs only accepts one output directory path"
                )
            }
        }
    }
}

impl std::error::Error for CleanCliError {}

fn parse_fidelity_trace_frame_count(value: &str) -> Result<usize, CleanCliError> {
    let frame_count =
        value
            .parse::<usize>()
            .map_err(|error| CleanCliError::InvalidFidelityTraceFrameCount {
                value: String::from(value),
                error: error.to_string(),
            })?;
    if frame_count == 0 {
        return Err(CleanCliError::NonPositiveFidelityTraceFrameCount);
    }

    Ok(frame_count)
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

    #[cfg(feature = "legacy-tools")]
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        AudioOutput, CleanCliError, CliClassification, ControlProfile,
        FidelityReferenceTraceFixtureDirectoryRequest, FidelityTraceCheckRequest,
        FidelityTraceFixtureDirectoryRequest, FidelityTraceInputsFileRequest,
        FidelityTraceInputsRequest, FidelityTraceRequest, RomReportRequest, RunMode,
        RuntimeCliClassifier, RuntimeConfig, ScenarioInputWriterRequest, VerifyRomsRequest,
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
            ])),
            CliClassification::Runtime(RuntimeConfig {
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
            RuntimeCliClassifier::classify(args(&["--live-smoke"])),
            CliClassification::Runtime(RuntimeConfig::smoke())
        );
    }

    #[test]
    fn clean_cli_owns_game_smoke_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--game-smoke"])),
            CliClassification::GameSmoke
        );
    }

    #[test]
    fn clean_cli_owns_actor_smoke_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--actor-smoke"])),
            CliClassification::ActorSmoke
        );
    }

    #[test]
    fn clean_cli_owns_actor_wgpu_smoke_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--actor-wgpu-smoke"])),
            CliClassification::ActorWgpuSmoke
        );
    }

    #[test]
    fn clean_cli_owns_live_smoke_configuration_flags() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&[
                "--input-profile",
                "cabinet",
                "--mute",
                "--cmos-path",
                "scores.bin",
                "--live-smoke",
            ])),
            CliClassification::Runtime(RuntimeConfig {
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
            RuntimeCliClassifier::classify(args(&["--live-smoke", "--input-profile", "test"])),
            CliClassification::Runtime(RuntimeConfig {
                controls: ControlProfile::Test,
                audio: AudioOutput::Device,
                mode: RunMode::Smoke,
                cmos_path: None,
            })
        );
    }

    #[test]
    fn clean_cli_owns_help_launch() {
        for values in [vec!["--help"], vec!["-h"], vec!["--mute", "--help"]] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Help
            );
        }
    }

    #[test]
    fn clean_cli_has_no_historical_commands_left() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--fidelity-check-reference-trace-dir"])),
            CliClassification::Error(CleanCliError::FidelityCheckReferenceTraceDirMissingPath)
        );
    }

    #[test]
    fn clean_cli_rejects_reference_fixture_directory_after_clean_live_flags() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&[
                "--live-smoke",
                "--fidelity-check-reference-trace-dir",
                "fixtures",
            ])),
            CliClassification::Error(CleanCliError::LiveOptionsWithCommand(
                "--fidelity-check-reference-trace-dir",
            ))
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&[
                "--mute",
                "--fidelity-check-reference-trace-dir",
                "fixtures",
            ])),
            CliClassification::Error(CleanCliError::LiveOptionsWithCommand(
                "--fidelity-check-reference-trace-dir",
            ))
        );
    }

    #[test]
    fn clean_cli_owns_rom_report_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--rom-report"])),
            CliClassification::RomReport(RomReportRequest { path: None })
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--rom-report", "roms"])),
            CliClassification::RomReport(RomReportRequest {
                path: Some(PathBuf::from("roms")),
            })
        );
    }

    #[test]
    fn clean_cli_owns_verify_roms_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--verify-roms", "roms"])),
            CliClassification::VerifyRoms(VerifyRomsRequest {
                path: PathBuf::from("roms"),
            })
        );
    }

    #[test]
    fn clean_cli_owns_fidelity_trace_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--fidelity-trace"])),
            CliClassification::FidelityTrace(FidelityTraceRequest { frame_count: 1 })
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--fidelity-trace", "300"])),
            CliClassification::FidelityTrace(FidelityTraceRequest { frame_count: 300 })
        );
    }

    #[test]
    fn clean_cli_owns_fidelity_trace_input_commands() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--fidelity-trace-inputs", "none"])),
            CliClassification::FidelityTraceInputs(FidelityTraceInputsRequest {
                script: String::from("none"),
            })
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--fidelity-trace-inputs-file", "inputs.txt"])),
            CliClassification::FidelityTraceInputsFile(FidelityTraceInputsFileRequest {
                path: PathBuf::from("inputs.txt"),
            })
        );
    }

    #[test]
    fn clean_cli_owns_fidelity_trace_check_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&[
                "--fidelity-check-trace",
                "inputs.txt",
                "expected.tsv",
            ])),
            CliClassification::FidelityTraceCheck(FidelityTraceCheckRequest {
                inputs_path: PathBuf::from("inputs.txt"),
                expected_path: PathBuf::from("expected.tsv"),
            })
        );
    }

    #[test]
    fn clean_cli_owns_fidelity_trace_fixture_directory_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--fidelity-check-trace-dir", "fixtures"])),
            CliClassification::FidelityTraceFixtureDirectory(
                FidelityTraceFixtureDirectoryRequest {
                    path: PathBuf::from("fixtures"),
                },
            )
        );
    }

    #[test]
    fn clean_cli_owns_fidelity_reference_trace_fixture_directory_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&[
                "--fidelity-check-reference-trace-dir",
                "fixtures",
            ])),
            CliClassification::FidelityReferenceTraceFixtureDirectory(
                FidelityReferenceTraceFixtureDirectoryRequest {
                    path: PathBuf::from("fixtures"),
                },
            )
        );
    }

    #[test]
    fn clean_cli_owns_fidelity_scenario_listing_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--fidelity-list-scenarios"])),
            CliClassification::FidelityScenarioList
        );
    }

    #[test]
    fn clean_cli_owns_fidelity_scenario_input_writer_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&[
                "--fidelity-write-scenario-inputs",
                "scenario-inputs",
            ])),
            CliClassification::FidelityScenarioInputWriter(ScenarioInputWriterRequest {
                path: PathBuf::from("scenario-inputs"),
            })
        );
    }

    #[test]
    fn clean_cli_rejects_malformed_live_args() {
        for (values, error) in [
            (
                vec!["--live-smoke", "--input-profile"],
                CleanCliError::MissingInputProfile,
            ),
            (
                vec!["--live-smoke", "--input-profile", "unknown"],
                CleanCliError::UnknownInputProfile(String::from("unknown")),
            ),
            (
                vec!["--live-smoke", "--cmos-path"],
                CleanCliError::MissingCmosPath,
            ),
            (vec!["--input-profile"], CleanCliError::MissingInputProfile),
            (
                vec!["--input-profile", "unknown"],
                CleanCliError::UnknownInputProfile(String::from("unknown")),
            ),
            (vec!["--cmos-path"], CleanCliError::MissingCmosPath),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_game_smoke_args() {
        for (values, error) in [
            (
                vec!["--mute", "--game-smoke"],
                CleanCliError::LiveOptionsWithCommand("--game-smoke"),
            ),
            (
                vec!["--game-smoke", "--mute"],
                CleanCliError::TooManyGameSmokeArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_actor_smoke_args() {
        for (values, error) in [
            (
                vec!["--mute", "--actor-smoke"],
                CleanCliError::LiveOptionsWithCommand("--actor-smoke"),
            ),
            (
                vec!["--actor-smoke", "--mute"],
                CleanCliError::TooManyActorSmokeArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_actor_wgpu_smoke_args() {
        for (values, error) in [
            (
                vec!["--mute", "--actor-wgpu-smoke"],
                CleanCliError::LiveOptionsWithCommand("--actor-wgpu-smoke"),
            ),
            (
                vec!["--actor-wgpu-smoke", "--mute"],
                CleanCliError::TooManyActorWgpuSmokeArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_rom_report_args() {
        for (values, error) in [
            (
                vec!["--mute", "--rom-report"],
                CleanCliError::LiveOptionsWithCommand("--rom-report"),
            ),
            (
                vec!["--rom-report", "--verify-roms"],
                CleanCliError::RomReportPathCannotBeFlag(String::from("--verify-roms")),
            ),
            (
                vec!["--rom-report", "roms", "extra"],
                CleanCliError::TooManyRomReportArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_verify_roms_args() {
        for (values, error) in [
            (
                vec!["--mute", "--verify-roms", "roms"],
                CleanCliError::LiveOptionsWithCommand("--verify-roms"),
            ),
            (vec!["--verify-roms"], CleanCliError::MissingVerifyRomsPath),
            (
                vec!["--verify-roms", "roms", "extra"],
                CleanCliError::TooManyVerifyRomsArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_fidelity_trace_args() {
        for (values, error) in [
            (
                vec!["--mute", "--fidelity-trace", "1"],
                CleanCliError::LiveOptionsWithCommand("--fidelity-trace"),
            ),
            (
                vec!["--fidelity-trace", "wat"],
                CleanCliError::InvalidFidelityTraceFrameCount {
                    value: String::from("wat"),
                    error: String::from("invalid digit found in string"),
                },
            ),
            (
                vec!["--fidelity-trace", "0"],
                CleanCliError::NonPositiveFidelityTraceFrameCount,
            ),
            (
                vec!["--fidelity-trace", "1", "extra"],
                CleanCliError::TooManyFidelityTraceArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_fidelity_trace_input_args() {
        for (values, error) in [
            (
                vec!["--mute", "--fidelity-trace-inputs", "none"],
                CleanCliError::LiveOptionsWithCommand("--fidelity-trace-inputs"),
            ),
            (
                vec!["--fidelity-trace-inputs"],
                CleanCliError::FidelityTraceInputsMissingScript,
            ),
            (
                vec!["--fidelity-trace-inputs", "none", "extra"],
                CleanCliError::FidelityTraceInputsExtraArgs,
            ),
            (
                vec!["--mute", "--fidelity-trace-inputs-file", "inputs.txt"],
                CleanCliError::LiveOptionsWithCommand("--fidelity-trace-inputs-file"),
            ),
            (
                vec!["--fidelity-trace-inputs-file"],
                CleanCliError::FidelityTraceInputsFileMissingPath,
            ),
            (
                vec!["--fidelity-trace-inputs-file", "inputs.txt", "extra"],
                CleanCliError::FidelityTraceInputsFileExtraArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_fidelity_trace_check_args() {
        for (values, error) in [
            (
                vec![
                    "--mute",
                    "--fidelity-check-trace",
                    "inputs.txt",
                    "expected.tsv",
                ],
                CleanCliError::LiveOptionsWithCommand("--fidelity-check-trace"),
            ),
            (
                vec!["--fidelity-check-trace"],
                CleanCliError::FidelityCheckTraceMissingPaths,
            ),
            (
                vec!["--fidelity-check-trace", "inputs.txt"],
                CleanCliError::FidelityCheckTraceMissingPaths,
            ),
            (
                vec![
                    "--fidelity-check-trace",
                    "inputs.txt",
                    "expected.tsv",
                    "extra",
                ],
                CleanCliError::FidelityCheckTraceExtraArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_fidelity_trace_fixture_directory_args() {
        for (values, error) in [
            (
                vec!["--mute", "--fidelity-check-trace-dir", "fixtures"],
                CleanCliError::LiveOptionsWithCommand("--fidelity-check-trace-dir"),
            ),
            (
                vec!["--fidelity-check-trace-dir"],
                CleanCliError::FidelityCheckTraceDirMissingPath,
            ),
            (
                vec!["--fidelity-check-trace-dir", "fixtures", "extra"],
                CleanCliError::FidelityCheckTraceDirExtraArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_fidelity_reference_trace_fixture_directory_args() {
        for (values, error) in [
            (
                vec!["--mute", "--fidelity-check-reference-trace-dir", "fixtures"],
                CleanCliError::LiveOptionsWithCommand("--fidelity-check-reference-trace-dir"),
            ),
            (
                vec!["--fidelity-check-reference-trace-dir"],
                CleanCliError::FidelityCheckReferenceTraceDirMissingPath,
            ),
            (
                vec!["--fidelity-check-reference-trace-dir", "fixtures", "extra"],
                CleanCliError::FidelityCheckReferenceTraceDirExtraArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_fidelity_scenario_listing_args() {
        for (values, error) in [
            (
                vec!["--mute", "--fidelity-list-scenarios"],
                CleanCliError::LiveOptionsWithCommand("--fidelity-list-scenarios"),
            ),
            (
                vec!["--fidelity-list-scenarios", "extra"],
                CleanCliError::FidelityListScenariosExtraArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_rejects_malformed_fidelity_scenario_input_writer_args() {
        for (values, error) in [
            (
                vec!["--mute", "--fidelity-write-scenario-inputs", "inputs"],
                CleanCliError::LiveOptionsWithCommand("--fidelity-write-scenario-inputs"),
            ),
            (
                vec!["--fidelity-write-scenario-inputs"],
                CleanCliError::FidelityWriteScenarioInputsMissingPath,
            ),
            (
                vec!["--fidelity-write-scenario-inputs", "inputs", "extra"],
                CleanCliError::FidelityWriteScenarioInputsExtraArgs,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(error)
            );
        }
    }

    #[test]
    fn clean_cli_error_messages_are_stable() {
        assert_eq!(
            CleanCliError::MissingInputProfile.to_string(),
            "--input-profile requires one of: planetoid, cabinet, test"
        );
        assert_eq!(
            CleanCliError::UnknownInputProfile(String::from("invalid")).to_string(),
            "unknown input profile: invalid"
        );
        assert_eq!(
            CleanCliError::MissingCmosPath.to_string(),
            "--cmos-path requires a file path"
        );
        assert_eq!(
            CleanCliError::RemovedRendererSelection.to_string(),
            "renderer selection was removed; live play is wgpu-only"
        );
        assert_eq!(
            CleanCliError::UnknownArgument(String::from("--unknown")).to_string(),
            "unknown argument: --unknown"
        );
        assert_eq!(
            CleanCliError::LiveOptionsWithCommand("--rom-report").to_string(),
            "live options cannot be combined with --rom-report"
        );
        assert_eq!(
            CleanCliError::RomReportPathCannotBeFlag(String::from("--verify-roms")).to_string(),
            "--rom-report optional path cannot be another flag: --verify-roms"
        );
        assert_eq!(
            CleanCliError::TooManyRomReportArgs.to_string(),
            "--rom-report only accepts one optional path"
        );
        assert_eq!(
            CleanCliError::MissingVerifyRomsPath.to_string(),
            "--verify-roms requires a ROM directory path"
        );
        assert_eq!(
            CleanCliError::TooManyVerifyRomsArgs.to_string(),
            "--verify-roms only accepts one directory path"
        );
        assert_eq!(
            CleanCliError::TooManyGameSmokeArgs.to_string(),
            "--game-smoke does not accept additional arguments"
        );
        assert_eq!(
            CleanCliError::TooManyActorSmokeArgs.to_string(),
            "--actor-smoke does not accept additional arguments"
        );
        assert_eq!(
            CleanCliError::TooManyActorWgpuSmokeArgs.to_string(),
            "--actor-wgpu-smoke does not accept additional arguments"
        );
        assert_eq!(
            CleanCliError::InvalidFidelityTraceFrameCount {
                value: String::from("wat"),
                error: String::from("invalid digit found in string"),
            }
            .to_string(),
            "invalid --fidelity-trace frame count: wat: invalid digit found in string"
        );
        assert_eq!(
            CleanCliError::NonPositiveFidelityTraceFrameCount.to_string(),
            "--fidelity-trace frame count must be greater than zero"
        );
        assert_eq!(
            CleanCliError::TooManyFidelityTraceArgs.to_string(),
            "--fidelity-trace only accepts one optional frame count"
        );
        assert_eq!(
            CleanCliError::FidelityTraceInputsMissingScript.to_string(),
            "--fidelity-trace-inputs requires a semicolon-separated input script"
        );
        assert_eq!(
            CleanCliError::FidelityTraceInputsExtraArgs.to_string(),
            "--fidelity-trace-inputs only accepts one input script"
        );
        assert_eq!(
            CleanCliError::FidelityTraceInputsFileMissingPath.to_string(),
            "--fidelity-trace-inputs-file requires a trace input script path"
        );
        assert_eq!(
            CleanCliError::FidelityTraceInputsFileExtraArgs.to_string(),
            "--fidelity-trace-inputs-file only accepts one path"
        );
        assert_eq!(
            CleanCliError::FidelityCheckTraceMissingPaths.to_string(),
            "--fidelity-check-trace requires an input script path and expected trace path"
        );
        assert_eq!(
            CleanCliError::FidelityCheckTraceExtraArgs.to_string(),
            "--fidelity-check-trace only accepts an input script path and expected trace path"
        );
        assert_eq!(
            CleanCliError::FidelityCheckTraceDirMissingPath.to_string(),
            "--fidelity-check-trace-dir requires a fixture directory path"
        );
        assert_eq!(
            CleanCliError::FidelityCheckTraceDirExtraArgs.to_string(),
            "--fidelity-check-trace-dir only accepts one fixture directory path"
        );
        assert_eq!(
            CleanCliError::FidelityCheckReferenceTraceDirMissingPath.to_string(),
            "--fidelity-check-reference-trace-dir requires a reference fixture directory path"
        );
        assert_eq!(
            CleanCliError::FidelityCheckReferenceTraceDirExtraArgs.to_string(),
            "--fidelity-check-reference-trace-dir only accepts one reference fixture directory path"
        );
        assert_eq!(
            CleanCliError::FidelityListScenariosExtraArgs.to_string(),
            "--fidelity-list-scenarios does not accept extra arguments"
        );
        assert_eq!(
            CleanCliError::FidelityWriteScenarioInputsMissingPath.to_string(),
            "--fidelity-write-scenario-inputs requires an output directory path"
        );
        assert_eq!(
            CleanCliError::FidelityWriteScenarioInputsExtraArgs.to_string(),
            "--fidelity-write-scenario-inputs only accepts one output directory path"
        );
    }

    #[test]
    fn clean_cli_rejects_unsupported_args() {
        for values in [vec!["--live-smoke", "--unknown"], vec!["--unknown"]] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(CleanCliError::UnknownArgument(String::from("--unknown")))
            );
        }
    }

    #[test]
    fn clean_cli_rejects_removed_renderer_selection() {
        for values in [vec!["--renderer", "wgpu"], vec!["--presentation", "wgpu"]] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::Error(CleanCliError::RemovedRendererSelection)
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
    fn game_smoke_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&["--game-smoke"]))
            .expect("clean game-smoke CLI should run through configured runtime");
    }

    #[test]
    fn actor_smoke_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&["--actor-smoke"]))
            .expect("actor smoke CLI should run through configured runtime");
    }

    #[test]
    fn actor_wgpu_smoke_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&["--actor-wgpu-smoke"]))
            .expect("actor wgpu smoke CLI should run through configured runtime");
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
    fn clean_help_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&["--help"]))
            .expect("clean help CLI should run through configured runtime");
    }

    #[test]
    fn clean_cli_entrypoint_rejects_malformed_live_args() {
        let error = super::run_with_args(args(&["--input-profile"]))
            .expect_err("malformed clean live args should return clean CLI error");

        assert_eq!(
            error.to_string(),
            "--input-profile requires one of: planetoid, cabinet, test"
        );
    }

    #[test]
    fn clean_cli_entrypoint_rejects_unsupported_args() {
        let error = super::run_with_args(args(&["--unknown"]))
            .expect_err("unsupported clean CLI args should return clean CLI error");

        assert_eq!(error.to_string(), "unknown argument: --unknown");
    }

    #[test]
    fn clean_cli_entrypoint_rejects_removed_renderer_selection() {
        let error = super::run_with_args(args(&["--renderer", "wgpu"]))
            .expect_err("removed renderer selection should return clean CLI error");

        assert_eq!(
            error.to_string(),
            "renderer selection was removed; live play is wgpu-only"
        );
    }

    #[cfg(not(feature = "legacy-tools"))]
    #[test]
    fn clean_cli_legacy_tool_entrypoints_require_explicit_feature() {
        for command in [
            &["--rom-report"][..],
            &["--verify-roms", "roms"],
            &["--fidelity-list-scenarios"],
            &["--fidelity-trace", "1"],
            &["--fidelity-trace-inputs", "none"],
            &["--fidelity-trace-inputs-file", "inputs.txt"],
            &["--fidelity-check-trace", "inputs.txt", "expected.tsv"],
            &["--fidelity-check-trace-dir", "fixtures"],
            &["--fidelity-check-reference-trace-dir", "reference"],
            &["--fidelity-write-scenario-inputs", "out"],
        ] {
            let error = super::run_with_args(args(command))
                .expect_err("legacy developer tooling must be feature-gated");
            let message = error.to_string();

            assert!(message.contains("developer legacy tooling"));
            assert!(message.contains("--features legacy-tools"));
        }
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn clean_rom_report_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&["--rom-report"]))
            .expect("clean ROM report CLI should run through configured runtime");
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn clean_verify_roms_cli_entrypoint_rejects_incomplete_rom_set_through_clean_runtime() {
        let path = unique_temp_dir("defender-clean-platform-verify-roms");
        fs::create_dir_all(&path).expect("create temp ROM dir");
        let path_arg = path.display().to_string();

        let error = super::run_with_args(args(&["--verify-roms", path_arg.as_str()]))
            .expect_err("incomplete ROM set should return clean verification report");

        assert!(error.to_string().contains("ROM set"));
        assert!(error.to_string().contains("Missing:"));
        let _ = fs::remove_dir_all(path);
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn clean_fidelity_scenario_listing_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&["--fidelity-list-scenarios"]))
            .expect("clean scenario listing CLI should run through configured runtime");
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn clean_fidelity_trace_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&["--fidelity-trace", "1"]))
            .expect("clean fidelity trace CLI should run through configured runtime");
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn clean_fidelity_trace_input_cli_entrypoints_accept_supported_args() {
        let path = unique_temp_dir("defender-clean-platform-trace-inputs");
        fs::create_dir_all(&path).expect("create temp dir");
        let script_path = path.join("inputs.txt");
        fs::write(&script_path, "none\n").expect("write trace input script");
        let path_arg = script_path.display().to_string();

        super::run_with_args(args(&["--fidelity-trace-inputs", "none"]))
            .expect("clean inline trace inputs CLI should run through configured runtime");
        super::run_with_args(args(&["--fidelity-trace-inputs-file", path_arg.as_str()]))
            .expect("clean file trace inputs CLI should run through configured runtime");
        let _ = fs::remove_dir_all(path);
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn clean_fidelity_trace_check_cli_entrypoint_accepts_supported_args() {
        let path = unique_temp_dir("defender-clean-platform-trace-check");
        fs::create_dir_all(&path).expect("create temp dir");
        let inputs_path = path.join("inputs.txt");
        let expected_path = path.join("expected.tsv");
        fs::write(&inputs_path, "none\n").expect("write trace input script");
        fs::write(&expected_path, one_frame_idle_trace_text()).expect("write expected trace");
        let inputs_arg = inputs_path.display().to_string();
        let expected_arg = expected_path.display().to_string();

        super::run_with_args(args(&[
            "--fidelity-check-trace",
            inputs_arg.as_str(),
            expected_arg.as_str(),
        ]))
        .expect("clean trace check CLI should run through configured runtime");
        let _ = fs::remove_dir_all(path);
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn clean_fidelity_trace_fixture_directory_cli_entrypoint_accepts_supported_args() {
        let path = unique_temp_dir("defender-clean-platform-trace-fixtures");
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("boot.inputs.txt"), "none\n").expect("write fixture input");
        fs::write(path.join("boot.expected.tsv"), one_frame_idle_trace_text())
            .expect("write expected trace");
        let path_arg = path.display().to_string();

        super::run_with_args(args(&["--fidelity-check-trace-dir", path_arg.as_str()]))
            .expect("clean trace fixture directory CLI should run through configured runtime");
        let _ = fs::remove_dir_all(path);
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn clean_fidelity_reference_trace_fixture_directory_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&[
            "--fidelity-check-reference-trace-dir",
            "docs/fidelity/fixtures/local/reference",
        ]))
        .expect("clean reference trace fixture directory CLI should run through runtime");
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn clean_fidelity_scenario_input_writer_cli_entrypoint_accepts_supported_args() {
        let path = unique_temp_dir("defender-clean-platform-scenario-inputs");
        let _ = fs::remove_dir_all(&path);
        let path_arg = path.display().to_string();

        super::run_with_args(args(&[
            "--fidelity-write-scenario-inputs",
            path_arg.as_str(),
        ]))
        .expect("clean scenario input writer CLI should run through configured runtime");

        assert!(path.join("attract_boot.inputs.txt").is_file());
        let _ = fs::remove_dir_all(path);
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

    #[cfg(feature = "legacy-tools")]
    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }

    #[cfg(feature = "legacy-tools")]
    fn one_frame_idle_trace_text() -> &'static str {
        concat!(
            "frame\tinput_bits\tinput_in0\tinput_in1\tinput_in2\tphase\t",
            "p1_score\tp2_score\twave\tlives\tsmart_bombs\tseed\thseed\tlseed\t",
            "object_table_crc32\tprocess_table_crc32\tsuper_process_table_crc32\t",
            "shell_table_crc32\tvideo_crc32\tsound_commands\tevents\n",
            "1\t0x0000\t0x00\t0x00\t0x00\tattract\t0\t0\t0\t0\t0\t",
            "0x00\t0x00\t0x00\t0xE15D8394\t0xC4C53DA1\t0x05B7E865\t",
            "0x41D912FF\t0x157E98C7\t-\t-\n",
        )
    }
}
