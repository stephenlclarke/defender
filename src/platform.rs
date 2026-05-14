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
    dispatch_cli_classification(RuntimeCliClassifier::classify(args))
}

fn dispatch_cli_classification(classification: CliClassification) -> anyhow::Result<()> {
    match classification {
        CliClassification::CleanRomReport(request) => crate::runtime::run_rom_report(request.path),
        CliClassification::CleanFidelityScenarioList => {
            crate::runtime::run_fidelity_scenario_list()
        }
        CliClassification::HistoricalCommand(command) => {
            let _command = command;
            crate::runtime::run_cli()
        }
        CliClassification::CompatibilityFallback(arg) => {
            let _first_arg = arg.first_arg;
            crate::runtime::run_cli()
        }
        CliClassification::CleanRuntime(config) => crate::runtime::run(&config),
        CliClassification::CleanHelp => crate::runtime::run_help(),
        CliClassification::CleanError(error) => Err(error.into()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliClassification {
    CleanRomReport(RomReportRequest),
    CleanFidelityScenarioList,
    HistoricalCommand(HistoricalCliCommand),
    CompatibilityFallback(CompatibilityCliArg),
    CleanRuntime(RuntimeConfig),
    CleanHelp,
    CleanError(CleanCliError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HistoricalCliCommand {
    VerifyRoms,
    FidelityTrace,
    FidelityTraceInputs,
    FidelityTraceInputsFile,
    FidelityCheckTrace,
    FidelityCheckTraceDir,
    FidelityWriteScenarioInputs,
    FidelityCheckReferenceTraceDir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompatibilityCliArg {
    first_arg: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RomReportRequest {
    path: Option<PathBuf>,
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
                ArgClassification::CleanRuntime => live_option_seen = true,
                ArgClassification::CleanRomReport(request) => {
                    return CliClassification::CleanRomReport(request);
                }
                ArgClassification::CleanFidelityScenarioList => {
                    return CliClassification::CleanFidelityScenarioList;
                }
                ArgClassification::CleanHelp => return CliClassification::CleanHelp,
                ArgClassification::CleanError(error) => {
                    return CliClassification::CleanError(error);
                }
                ArgClassification::HistoricalCommand(command) => {
                    return CliClassification::HistoricalCommand(command);
                }
                ArgClassification::CompatibilityFallback(first_arg) => {
                    return CliClassification::CompatibilityFallback(CompatibilityCliArg {
                        first_arg,
                    });
                }
            }
        }

        CliClassification::CleanRuntime(config)
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
                ArgClassification::CleanRuntime
            }
            "--help" | "-h" => ArgClassification::CleanHelp,
            "--input-profile" => {
                let Some(value) = args.next() else {
                    return ArgClassification::CleanError(CleanCliError::MissingInputProfile);
                };
                let Some(controls) = parse_control_profile(&value) else {
                    return ArgClassification::CleanError(CleanCliError::UnknownInputProfile(
                        value,
                    ));
                };
                config.controls = controls;
                ArgClassification::CleanRuntime
            }
            "--mute" => {
                config.audio = AudioOutput::Disabled;
                ArgClassification::CleanRuntime
            }
            "--cmos-path" => {
                let Some(value) = args.next() else {
                    return ArgClassification::CleanError(CleanCliError::MissingCmosPath);
                };
                config.cmos_path = Some(PathBuf::from(value));
                ArgClassification::CleanRuntime
            }
            "--rom-report" => {
                if live_option_seen {
                    return ArgClassification::CleanError(CleanCliError::LiveOptionsWithCommand(
                        "--rom-report",
                    ));
                }
                let path = match args.next() {
                    Some(value) if value.starts_with('-') => {
                        return ArgClassification::CleanError(
                            CleanCliError::RomReportPathCannotBeFlag(value),
                        );
                    }
                    Some(value) => Some(PathBuf::from(value)),
                    None => None,
                };
                if args.next().is_some() {
                    return ArgClassification::CleanError(CleanCliError::TooManyRomReportArgs);
                }
                ArgClassification::CleanRomReport(RomReportRequest { path })
            }
            "--fidelity-list-scenarios" => {
                if live_option_seen {
                    return ArgClassification::CleanError(CleanCliError::LiveOptionsWithCommand(
                        "--fidelity-list-scenarios",
                    ));
                }
                if args.next().is_some() {
                    return ArgClassification::CleanError(
                        CleanCliError::FidelityListScenariosExtraArgs,
                    );
                }
                ArgClassification::CleanFidelityScenarioList
            }
            _ => historical_cli_command(arg)
                .map(ArgClassification::HistoricalCommand)
                .unwrap_or_else(|| ArgClassification::CompatibilityFallback(String::from(arg))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArgClassification {
    CleanRomReport(RomReportRequest),
    CleanFidelityScenarioList,
    HistoricalCommand(HistoricalCliCommand),
    CompatibilityFallback(String),
    CleanRuntime,
    CleanHelp,
    CleanError(CleanCliError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CleanCliError {
    MissingInputProfile,
    UnknownInputProfile(String),
    MissingCmosPath,
    LiveOptionsWithCommand(&'static str),
    RomReportPathCannotBeFlag(String),
    TooManyRomReportArgs,
    FidelityListScenariosExtraArgs,
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
            Self::FidelityListScenariosExtraArgs => {
                write!(
                    formatter,
                    "--fidelity-list-scenarios does not accept extra arguments"
                )
            }
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

fn historical_cli_command(arg: &str) -> Option<HistoricalCliCommand> {
    match arg {
        "--verify-roms" => Some(HistoricalCliCommand::VerifyRoms),
        "--fidelity-trace" => Some(HistoricalCliCommand::FidelityTrace),
        "--fidelity-trace-inputs" => Some(HistoricalCliCommand::FidelityTraceInputs),
        "--fidelity-trace-inputs-file" => Some(HistoricalCliCommand::FidelityTraceInputsFile),
        "--fidelity-check-trace" => Some(HistoricalCliCommand::FidelityCheckTrace),
        "--fidelity-check-trace-dir" => Some(HistoricalCliCommand::FidelityCheckTraceDir),
        "--fidelity-write-scenario-inputs" => {
            Some(HistoricalCliCommand::FidelityWriteScenarioInputs)
        }
        "--fidelity-check-reference-trace-dir" => {
            Some(HistoricalCliCommand::FidelityCheckReferenceTraceDir)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        AudioOutput, CleanCliError, CliClassification, CompatibilityCliArg, ControlProfile,
        HistoricalCliCommand, RomReportRequest, RunMode, RuntimeCliClassifier, RuntimeConfig,
    };

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
            RuntimeCliClassifier::classify(args(&[])),
            CliClassification::CleanRuntime(RuntimeConfig::default())
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
            CliClassification::CleanRuntime(RuntimeConfig {
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
            CliClassification::CleanRuntime(RuntimeConfig::smoke())
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
            CliClassification::CleanRuntime(RuntimeConfig {
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
            CliClassification::CleanRuntime(RuntimeConfig {
                controls: ControlProfile::Test,
                audio: AudioOutput::Null,
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
                CliClassification::CleanHelp
            );
        }
    }

    #[test]
    fn clean_cli_delegates_historical_commands() {
        for (arg, command) in [
            ("--verify-roms", HistoricalCliCommand::VerifyRoms),
            ("--fidelity-trace", HistoricalCliCommand::FidelityTrace),
            (
                "--fidelity-trace-inputs",
                HistoricalCliCommand::FidelityTraceInputs,
            ),
            (
                "--fidelity-trace-inputs-file",
                HistoricalCliCommand::FidelityTraceInputsFile,
            ),
            (
                "--fidelity-check-trace",
                HistoricalCliCommand::FidelityCheckTrace,
            ),
            (
                "--fidelity-check-trace-dir",
                HistoricalCliCommand::FidelityCheckTraceDir,
            ),
            (
                "--fidelity-write-scenario-inputs",
                HistoricalCliCommand::FidelityWriteScenarioInputs,
            ),
            (
                "--fidelity-check-reference-trace-dir",
                HistoricalCliCommand::FidelityCheckReferenceTraceDir,
            ),
        ] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&[arg])),
                CliClassification::HistoricalCommand(command)
            );
        }
    }

    #[test]
    fn clean_cli_delegates_historical_commands_after_clean_live_flags() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--live-smoke", "--fidelity-trace", "1"])),
            CliClassification::HistoricalCommand(HistoricalCliCommand::FidelityTrace)
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--mute", "--verify-roms", "roms"])),
            CliClassification::HistoricalCommand(HistoricalCliCommand::VerifyRoms)
        );
    }

    #[test]
    fn clean_cli_owns_rom_report_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--rom-report"])),
            CliClassification::CleanRomReport(RomReportRequest { path: None })
        );
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--rom-report", "roms"])),
            CliClassification::CleanRomReport(RomReportRequest {
                path: Some(PathBuf::from("roms")),
            })
        );
    }

    #[test]
    fn clean_cli_owns_fidelity_scenario_listing_command() {
        assert_eq!(
            RuntimeCliClassifier::classify(args(&["--fidelity-list-scenarios"])),
            CliClassification::CleanFidelityScenarioList
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
                CliClassification::CleanError(error)
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
                CliClassification::CleanError(error)
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
                CliClassification::CleanError(error)
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
            CleanCliError::FidelityListScenariosExtraArgs.to_string(),
            "--fidelity-list-scenarios does not accept extra arguments"
        );
    }

    #[test]
    fn clean_cli_delegates_unsupported_args() {
        for values in [vec!["--live-smoke", "--unknown"], vec!["--unknown"]] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::CompatibilityFallback(CompatibilityCliArg {
                    first_arg: String::from("--unknown"),
                })
            );
        }
    }

    #[test]
    fn clean_cli_delegates_removed_renderer_selection_as_compatibility() {
        for values in [vec!["--renderer", "wgpu"], vec!["--presentation", "wgpu"]] {
            assert_eq!(
                RuntimeCliClassifier::classify(args(&values)),
                CliClassification::CompatibilityFallback(CompatibilityCliArg {
                    first_arg: String::from(values[0]),
                })
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
    fn accepted_cli_entrypoint_delegates_unsupported_args() {
        super::run_with_args(args(&["--unknown"]))
            .expect("unsupported clean CLI args should delegate to accepted CLI");
    }

    #[test]
    fn clean_rom_report_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&["--rom-report"]))
            .expect("clean ROM report CLI should run through configured runtime");
    }

    #[test]
    fn clean_fidelity_scenario_listing_cli_entrypoint_accepts_supported_args() {
        super::run_with_args(args(&["--fidelity-list-scenarios"]))
            .expect("clean scenario listing CLI should run through configured runtime");
    }

    #[test]
    fn accepted_cli_entrypoint_delegates_historical_commands() {
        super::run_with_args(args(&["--verify-roms"]))
            .expect("historical CLI commands should delegate to accepted CLI");
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
