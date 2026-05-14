//! Private launch bridge for the playable runtime.

use std::path::PathBuf;

use crate::{
    audio::LiveAudioMode,
    input::InputProfile,
    platform::{AudioOutput, ControlProfile, RunMode, RuntimeConfig},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RuntimeHost<B = InstalledRuntimeBackend> {
    backend: B,
}

impl RuntimeHost<InstalledRuntimeBackend> {
    pub(crate) fn current() -> Self {
        Self::with_backend(InstalledRuntimeBackend)
    }
}

impl<B> RuntimeHost<B> {
    pub(crate) fn with_backend(backend: B) -> Self {
        Self { backend }
    }
}

impl<B: RuntimeBackend> RuntimeHost<B> {
    pub(crate) fn run_cli(&self) -> anyhow::Result<()> {
        self.backend.run_command(RuntimeCommand::AcceptedCli)
    }

    pub(crate) fn run_help(&self) -> anyhow::Result<()> {
        self.backend.run_command(RuntimeCommand::Help)
    }

    pub(crate) fn run_rom_report(&self, path: Option<PathBuf>) -> anyhow::Result<()> {
        self.backend.run_command(RuntimeCommand::RomReport { path })
    }

    pub(crate) fn run_verify_roms(&self, path: PathBuf) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::VerifyRoms { path })
    }

    pub(crate) fn run_fidelity_scenario_list(&self) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::FidelityScenarioList)
    }

    pub(crate) fn run_fidelity_scenario_input_writer(&self, path: PathBuf) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::FidelityScenarioInputWriter { path })
    }

    pub(crate) fn run(&self, config: &RuntimeConfig) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::from_config(config))
    }
}

pub(crate) trait RuntimeBackend {
    fn run_command(&self, command: RuntimeCommand) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeCommand {
    AcceptedCli,
    Help,
    RomReport {
        path: Option<PathBuf>,
    },
    VerifyRoms {
        path: PathBuf,
    },
    FidelityScenarioList,
    FidelityScenarioInputWriter {
        path: PathBuf,
    },
    WgpuLive {
        input_profile: InputProfile,
        audio_mode: LiveAudioMode,
        cmos_path: Option<PathBuf>,
    },
    WgpuLiveSmoke {
        input_profile: InputProfile,
        cmos_path: Option<PathBuf>,
    },
}

impl RuntimeCommand {
    fn from_config(config: &RuntimeConfig) -> Self {
        match config.mode {
            RunMode::Interactive => Self::WgpuLive {
                input_profile: input_profile(config.controls),
                audio_mode: audio_mode(config.audio),
                cmos_path: config.cmos_path.clone(),
            },
            RunMode::Smoke => Self::WgpuLiveSmoke {
                input_profile: input_profile(config.controls),
                cmos_path: config.cmos_path.clone(),
            },
        }
    }
}

fn audio_mode(output: AudioOutput) -> LiveAudioMode {
    match output {
        AudioOutput::Disabled => LiveAudioMode::Disabled,
        AudioOutput::Null => LiveAudioMode::Null,
    }
}

fn input_profile(profile: ControlProfile) -> InputProfile {
    match profile {
        ControlProfile::Planetoid => InputProfile::Planetoid,
        ControlProfile::Cabinet => InputProfile::Cabinet,
        ControlProfile::Test => InputProfile::Test,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct InstalledRuntimeBackend;

impl RuntimeBackend for InstalledRuntimeBackend {
    fn run_command(&self, command: RuntimeCommand) -> anyhow::Result<()> {
        match command {
            RuntimeCommand::AcceptedCli => crate::accepted_behavior::run_runtime(),
            RuntimeCommand::Help => {
                print!("{}", help_text());
                Ok(())
            }
            RuntimeCommand::RomReport { path } => crate::rom_report::run(path.as_deref()),
            RuntimeCommand::VerifyRoms { path } => crate::rom_report::run_verify(&path),
            RuntimeCommand::FidelityScenarioList => crate::fidelity_scenarios::run_list(),
            RuntimeCommand::FidelityScenarioInputWriter { path } => {
                crate::fidelity_scenarios::run_write_inputs(&path)
            }
            RuntimeCommand::WgpuLive {
                input_profile,
                audio_mode,
                cmos_path,
            } => crate::wgpu_presenter::run_wgpu_live(
                input_profile,
                audio_mode,
                cmos_path.as_deref(),
            ),
            RuntimeCommand::WgpuLiveSmoke {
                input_profile,
                cmos_path,
            } => {
                let report = crate::wgpu_presenter::run_wgpu_live_smoke(
                    input_profile,
                    cmos_path.as_deref(),
                )?;
                print!("{}", report.to_text());
                Ok(())
            }
        }
    }
}

pub(crate) fn run_cli() -> anyhow::Result<()> {
    RuntimeHost::current().run_cli()
}

pub(crate) fn run_help() -> anyhow::Result<()> {
    RuntimeHost::current().run_help()
}

pub(crate) fn run_rom_report(path: Option<PathBuf>) -> anyhow::Result<()> {
    RuntimeHost::current().run_rom_report(path)
}

pub(crate) fn run_verify_roms(path: PathBuf) -> anyhow::Result<()> {
    RuntimeHost::current().run_verify_roms(path)
}

pub(crate) fn run_fidelity_scenario_list() -> anyhow::Result<()> {
    RuntimeHost::current().run_fidelity_scenario_list()
}

pub(crate) fn run_fidelity_scenario_input_writer(path: PathBuf) -> anyhow::Result<()> {
    RuntimeHost::current().run_fidelity_scenario_input_writer(path)
}

pub(crate) fn run(config: &RuntimeConfig) -> anyhow::Result<()> {
    RuntimeHost::current().run(config)
}

pub(crate) fn help_text() -> &'static str {
    concat!(
        "defender\n",
        "  cargo run\n",
        "  cargo run -- --live-smoke\n",
        "  cargo run -- --mute\n",
        "  cargo run -- --input-profile planetoid\n",
        "  cargo run -- --input-profile cabinet\n",
        "  cargo run -- --cmos-path ~/.local/state/defender/red-label-cmos.bin\n",
        "  cargo run -- --rom-report\n",
        "  cargo run -- --rom-report /path/to/roms\n",
        "  cargo run -- --verify-roms /path/to/roms\n",
        "  cargo run -- --fidelity-trace 300\n",
        "  cargo run -- --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'\n",
        "  cargo run -- --fidelity-trace-inputs-file /path/to/inputs.txt\n",
        "  cargo run -- --fidelity-check-trace /path/to/inputs.txt /path/to/expected.tsv\n",
        "  cargo run -- --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current\n",
        "  cargo run -- --fidelity-list-scenarios\n",
        "  cargo run -- --fidelity-write-scenario-inputs docs/fidelity/fixtures/local/reference\n",
        "  cargo run -- --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local/reference\n",
        "\n",
        "Runtime assets are embedded in the binary for copy-only deployment.\n",
        "Live play uses the windowed wgpu backend.\n",
        "Live audio routes accepted sound commands through a non-blocking null backend; ",
        "--mute disables that runtime path.\n",
    )
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        fs,
        path::PathBuf,
        rc::Rc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::platform::{AudioOutput, ControlProfile, RunMode, RuntimeConfig};
    use crate::{audio::LiveAudioMode, input::InputProfile};

    use super::{
        InstalledRuntimeBackend, RuntimeBackend, RuntimeCommand, RuntimeHost, audio_mode,
        help_text, input_profile,
    };

    #[derive(Debug, Clone, Default)]
    struct RecordingBackend {
        calls: Rc<RefCell<Vec<RuntimeCommand>>>,
    }

    impl RuntimeBackend for RecordingBackend {
        fn run_command(&self, command: RuntimeCommand) -> anyhow::Result<()> {
            self.calls.borrow_mut().push(command);
            Ok(())
        }
    }

    #[test]
    fn runtime_host_adapts_clean_config_to_launch_command() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });
        let config = RuntimeConfig {
            controls: ControlProfile::Cabinet,
            audio: AudioOutput::Disabled,
            mode: RunMode::Interactive,
            cmos_path: Some(PathBuf::from("scores.bin")),
        };

        host.run(&config).expect("runtime host should run backend");

        let observed = calls.borrow();
        assert_eq!(
            observed.as_slice(),
            &[RuntimeCommand::WgpuLive {
                input_profile: InputProfile::Cabinet,
                audio_mode: LiveAudioMode::Disabled,
                cmos_path: Some(PathBuf::from("scores.bin")),
            }]
        );
    }

    #[test]
    fn runtime_host_launches_default_cli_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_cli()
            .expect("runtime host should run default CLI command");

        let observed = calls.borrow();
        assert_eq!(observed.as_slice(), &[RuntimeCommand::AcceptedCli]);
    }

    #[test]
    fn runtime_host_launches_help_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_help()
            .expect("runtime host should run help command");

        let observed = calls.borrow();
        assert_eq!(observed.as_slice(), &[RuntimeCommand::Help]);
    }

    #[test]
    fn runtime_host_launches_rom_report_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_rom_report(Some(PathBuf::from("roms")))
            .expect("runtime host should run ROM report command");

        let observed = calls.borrow();
        assert_eq!(
            observed.as_slice(),
            &[RuntimeCommand::RomReport {
                path: Some(PathBuf::from("roms")),
            }]
        );
    }

    #[test]
    fn runtime_host_launches_verify_roms_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_verify_roms(PathBuf::from("roms"))
            .expect("runtime host should run verify ROMs command");

        let observed = calls.borrow();
        assert_eq!(
            observed.as_slice(),
            &[RuntimeCommand::VerifyRoms {
                path: PathBuf::from("roms"),
            }]
        );
    }

    #[test]
    fn runtime_host_launches_fidelity_scenario_list_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_fidelity_scenario_list()
            .expect("runtime host should run scenario list command");

        let observed = calls.borrow();
        assert_eq!(observed.as_slice(), &[RuntimeCommand::FidelityScenarioList]);
    }

    #[test]
    fn runtime_host_launches_fidelity_scenario_input_writer_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_fidelity_scenario_input_writer(PathBuf::from("inputs"))
            .expect("runtime host should run scenario input writer command");

        let observed = calls.borrow();
        assert_eq!(
            observed.as_slice(),
            &[RuntimeCommand::FidelityScenarioInputWriter {
                path: PathBuf::from("inputs"),
            }]
        );
    }

    #[test]
    fn default_config_uses_wgpu_live_launch() {
        assert_eq!(
            RuntimeCommand::from_config(&RuntimeConfig::default()),
            RuntimeCommand::WgpuLive {
                input_profile: InputProfile::Planetoid,
                audio_mode: LiveAudioMode::Null,
                cmos_path: None,
            }
        );
    }

    #[test]
    fn smoke_config_uses_wgpu_live_smoke_launch() {
        let config = RuntimeConfig {
            controls: ControlProfile::Test,
            audio: AudioOutput::Null,
            mode: RunMode::Smoke,
            cmos_path: Some(PathBuf::from("smoke_cmos.bin")),
        };

        assert_eq!(
            RuntimeCommand::from_config(&config),
            RuntimeCommand::WgpuLiveSmoke {
                input_profile: InputProfile::Test,
                cmos_path: Some(PathBuf::from("smoke_cmos.bin")),
            }
        );
    }

    #[test]
    fn clean_audio_outputs_map_to_runtime_audio_modes() {
        assert_eq!(audio_mode(AudioOutput::Disabled), LiveAudioMode::Disabled);
        assert_eq!(audio_mode(AudioOutput::Null), LiveAudioMode::Null);
    }

    #[test]
    fn clean_control_profiles_map_to_runtime_input_profiles() {
        assert_eq!(
            input_profile(ControlProfile::Planetoid),
            InputProfile::Planetoid
        );
        assert_eq!(
            input_profile(ControlProfile::Cabinet),
            InputProfile::Cabinet
        );
        assert_eq!(input_profile(ControlProfile::Test), InputProfile::Test);
    }

    #[test]
    fn current_runtime_uses_installed_backend() {
        assert_eq!(
            RuntimeHost::current(),
            RuntimeHost::with_backend(InstalledRuntimeBackend)
        );
    }

    #[test]
    fn installed_backend_runs_config_driven_wgpu_smoke() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run(&RuntimeConfig::smoke())
            .expect("installed backend should run config-driven smoke");
    }

    #[test]
    fn installed_backend_runs_config_driven_wgpu_live() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run(&RuntimeConfig::default())
            .expect("installed backend should run config-driven live");
    }

    #[test]
    fn installed_backend_runs_clean_help() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_help()
            .expect("installed backend should run clean help");
    }

    #[test]
    fn installed_backend_runs_clean_rom_listing_report() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_rom_report(None)
            .expect("installed backend should run clean ROM listing report");
    }

    #[test]
    fn installed_backend_runs_clean_verify_roms_report() {
        let path = unique_temp_dir("defender-clean-runtime-verify-roms");
        fs::create_dir_all(&path).expect("create temp ROM dir");

        let error = RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_verify_roms(path.clone())
            .expect_err("installed backend should report incomplete ROM set");

        assert!(error.to_string().contains("ROM set"));
        assert!(error.to_string().contains("Missing:"));
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn installed_backend_runs_clean_fidelity_scenario_list() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_fidelity_scenario_list()
            .expect("installed backend should run clean scenario list");
    }

    #[test]
    fn installed_backend_runs_clean_fidelity_scenario_input_writer() {
        let path = unique_temp_dir("defender-clean-runtime-scenario-inputs");
        let _ = fs::remove_dir_all(&path);

        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_fidelity_scenario_input_writer(path.clone())
            .expect("installed backend should run clean scenario input writer");

        assert!(path.join("attract_boot.inputs.txt").is_file());
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn clean_help_text_preserves_current_cli_contract() {
        let text = help_text();

        assert!(text.starts_with("defender\n  cargo run\n"));
        assert!(text.contains("--rom-report"));
        assert!(text.contains("--input-profile planetoid"));
        assert!(text.contains("--input-profile cabinet"));
        assert!(text.contains("--live-smoke"));
        assert!(text.contains("--mute"));
        assert!(text.contains("--verify-roms"));
        assert!(text.contains("--fidelity-trace 300"));
        assert!(text.contains("--fidelity-trace-inputs"));
        assert!(text.contains("--fidelity-trace-inputs-file"));
        assert!(text.contains("--fidelity-check-trace"));
        assert!(text.contains("--fidelity-check-trace-dir"));
        assert!(text.contains("--fidelity-list-scenarios"));
        assert!(text.contains("--fidelity-write-scenario-inputs"));
        assert!(text.contains("--fidelity-check-reference-trace-dir"));
        assert!(text.contains("docs/fidelity/fixtures/local/rust-current"));
        assert!(text.contains("docs/fidelity/fixtures/local/reference"));
        assert!(text.contains("copy-only deployment"));
        assert!(text.contains("uses the windowed wgpu"));
        assert!(!text.contains("Kitty graphics"));
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }
}
