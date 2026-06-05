//! Private launch bridge for the playable runtime.

use std::path::PathBuf;

use crate::{
    audio::LiveAudioMode,
    live_wgpu::LiveInputProfile,
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

    pub(crate) fn run_game_smoke(&self) -> anyhow::Result<()> {
        self.backend.run_command(RuntimeCommand::GameSmoke)
    }

    pub(crate) fn run_actor_smoke(&self) -> anyhow::Result<()> {
        self.backend.run_command(RuntimeCommand::ActorSmoke)
    }

    pub(crate) fn run_actor_wgpu_smoke(&self) -> anyhow::Result<()> {
        self.backend.run_command(RuntimeCommand::ActorWgpuSmoke)
    }

    pub(crate) fn run_fidelity_trace(&self, frame_count: usize) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::FidelityTrace { frame_count })
    }

    pub(crate) fn run_fidelity_trace_inputs(&self, script: String) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::FidelityTraceInputs { script })
    }

    pub(crate) fn run_fidelity_trace_inputs_file(&self, path: PathBuf) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::FidelityTraceInputsFile { path })
    }

    pub(crate) fn run_fidelity_trace_check(
        &self,
        inputs_path: PathBuf,
        expected_path: PathBuf,
    ) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::FidelityTraceCheck {
                inputs_path,
                expected_path,
            })
    }

    pub(crate) fn run_fidelity_trace_check_dir(&self, path: PathBuf) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::FidelityTraceFixtureDirectory { path })
    }

    pub(crate) fn run_fidelity_reference_trace_check_dir(
        &self,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::FidelityReferenceTraceFixtureDirectory { path })
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
    Help,
    RomReport {
        path: Option<PathBuf>,
    },
    VerifyRoms {
        path: PathBuf,
    },
    GameSmoke,
    ActorSmoke,
    ActorWgpuSmoke,
    FidelityTrace {
        frame_count: usize,
    },
    FidelityTraceInputs {
        script: String,
    },
    FidelityTraceInputsFile {
        path: PathBuf,
    },
    FidelityTraceCheck {
        inputs_path: PathBuf,
        expected_path: PathBuf,
    },
    FidelityTraceFixtureDirectory {
        path: PathBuf,
    },
    FidelityReferenceTraceFixtureDirectory {
        path: PathBuf,
    },
    FidelityScenarioList,
    FidelityScenarioInputWriter {
        path: PathBuf,
    },
    WgpuLive {
        input_profile: LiveInputProfile,
        audio_mode: LiveAudioMode,
        cmos_path: Option<PathBuf>,
    },
    WgpuLiveSmoke {
        input_profile: LiveInputProfile,
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
        AudioOutput::Device => LiveAudioMode::Device,
        AudioOutput::Null => LiveAudioMode::Null,
    }
}

fn input_profile(profile: ControlProfile) -> LiveInputProfile {
    match profile {
        ControlProfile::Planetoid => LiveInputProfile::Planetoid,
        ControlProfile::Cabinet => LiveInputProfile::Cabinet,
        ControlProfile::Test => LiveInputProfile::Test,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct InstalledRuntimeBackend;

impl RuntimeBackend for InstalledRuntimeBackend {
    fn run_command(&self, command: RuntimeCommand) -> anyhow::Result<()> {
        match command {
            RuntimeCommand::Help => {
                print!("{}", help_text());
                Ok(())
            }
            RuntimeCommand::RomReport { path } => run_rom_report_command(path),
            RuntimeCommand::VerifyRoms { path } => run_verify_roms_command(path),
            RuntimeCommand::GameSmoke => crate::game_smoke::run(),
            RuntimeCommand::ActorSmoke => crate::actor_smoke::run(),
            RuntimeCommand::ActorWgpuSmoke => {
                let report = crate::live_wgpu::run_actor_wgpu_smoke()?;
                print!("{}", report.to_text());
                Ok(())
            }
            RuntimeCommand::FidelityTrace { frame_count } => {
                run_fidelity_trace_command(frame_count)
            }
            RuntimeCommand::FidelityTraceInputs { script } => {
                run_fidelity_trace_inputs_command(script)
            }
            RuntimeCommand::FidelityTraceInputsFile { path } => {
                run_fidelity_trace_inputs_file_command(path)
            }
            RuntimeCommand::FidelityTraceCheck {
                inputs_path,
                expected_path,
            } => run_fidelity_trace_check_command(inputs_path, expected_path),
            RuntimeCommand::FidelityTraceFixtureDirectory { path } => {
                run_fidelity_trace_check_dir_command(path)
            }
            RuntimeCommand::FidelityReferenceTraceFixtureDirectory { path } => {
                run_fidelity_reference_trace_check_dir_command(path)
            }
            RuntimeCommand::FidelityScenarioList => run_fidelity_scenario_list_command(),
            RuntimeCommand::FidelityScenarioInputWriter { path } => {
                run_fidelity_scenario_input_writer_command(path)
            }
            RuntimeCommand::WgpuLive {
                input_profile,
                audio_mode,
                cmos_path,
            } => crate::live_wgpu::run(input_profile, audio_mode, cmos_path.as_deref()),
            RuntimeCommand::WgpuLiveSmoke {
                input_profile,
                cmos_path,
            } => {
                let report = crate::live_wgpu::run_smoke(input_profile, cmos_path.as_deref())?;
                print!("{}", report.to_text());
                Ok(())
            }
        }
    }
}

#[cfg(feature = "legacy-tools")]
fn run_rom_report_command(path: Option<PathBuf>) -> anyhow::Result<()> {
    crate::rom_report::run(path.as_deref())
}

#[cfg(not(feature = "legacy-tools"))]
fn run_rom_report_command(_path: Option<PathBuf>) -> anyhow::Result<()> {
    Err(legacy_tools_disabled("--rom-report"))
}

#[cfg(feature = "legacy-tools")]
fn run_verify_roms_command(path: PathBuf) -> anyhow::Result<()> {
    crate::rom_report::run_verify(&path)
}

#[cfg(not(feature = "legacy-tools"))]
fn run_verify_roms_command(_path: PathBuf) -> anyhow::Result<()> {
    Err(legacy_tools_disabled("--verify-roms"))
}

#[cfg(feature = "legacy-tools")]
fn run_fidelity_trace_command(frame_count: usize) -> anyhow::Result<()> {
    crate::fidelity_traces::run_trace(frame_count)
}

#[cfg(not(feature = "legacy-tools"))]
fn run_fidelity_trace_command(_frame_count: usize) -> anyhow::Result<()> {
    Err(legacy_tools_disabled("--fidelity-trace"))
}

#[cfg(feature = "legacy-tools")]
fn run_fidelity_trace_inputs_command(script: String) -> anyhow::Result<()> {
    crate::fidelity_traces::run_trace_inputs(&script)
}

#[cfg(not(feature = "legacy-tools"))]
fn run_fidelity_trace_inputs_command(_script: String) -> anyhow::Result<()> {
    Err(legacy_tools_disabled("--fidelity-trace-inputs"))
}

#[cfg(feature = "legacy-tools")]
fn run_fidelity_trace_inputs_file_command(path: PathBuf) -> anyhow::Result<()> {
    crate::fidelity_traces::run_trace_inputs_file(&path)
}

#[cfg(not(feature = "legacy-tools"))]
fn run_fidelity_trace_inputs_file_command(_path: PathBuf) -> anyhow::Result<()> {
    Err(legacy_tools_disabled("--fidelity-trace-inputs-file"))
}

#[cfg(feature = "legacy-tools")]
fn run_fidelity_trace_check_command(
    inputs_path: PathBuf,
    expected_path: PathBuf,
) -> anyhow::Result<()> {
    crate::fidelity_traces::run_check_trace(&inputs_path, &expected_path)
}

#[cfg(not(feature = "legacy-tools"))]
fn run_fidelity_trace_check_command(
    _inputs_path: PathBuf,
    _expected_path: PathBuf,
) -> anyhow::Result<()> {
    Err(legacy_tools_disabled("--fidelity-check-trace"))
}

#[cfg(feature = "legacy-tools")]
fn run_fidelity_trace_check_dir_command(path: PathBuf) -> anyhow::Result<()> {
    crate::fidelity_traces::run_check_trace_dir(&path)
}

#[cfg(not(feature = "legacy-tools"))]
fn run_fidelity_trace_check_dir_command(_path: PathBuf) -> anyhow::Result<()> {
    Err(legacy_tools_disabled("--fidelity-check-trace-dir"))
}

#[cfg(feature = "legacy-tools")]
fn run_fidelity_reference_trace_check_dir_command(path: PathBuf) -> anyhow::Result<()> {
    crate::fidelity_traces::run_check_reference_trace_dir(&path)
}

#[cfg(not(feature = "legacy-tools"))]
fn run_fidelity_reference_trace_check_dir_command(_path: PathBuf) -> anyhow::Result<()> {
    Err(legacy_tools_disabled(
        "--fidelity-check-reference-trace-dir",
    ))
}

#[cfg(feature = "legacy-tools")]
fn run_fidelity_scenario_list_command() -> anyhow::Result<()> {
    crate::fidelity_scenarios::run_list()
}

#[cfg(not(feature = "legacy-tools"))]
fn run_fidelity_scenario_list_command() -> anyhow::Result<()> {
    Err(legacy_tools_disabled("--fidelity-list-scenarios"))
}

#[cfg(feature = "legacy-tools")]
fn run_fidelity_scenario_input_writer_command(path: PathBuf) -> anyhow::Result<()> {
    crate::fidelity_scenarios::run_write_inputs(&path)
}

#[cfg(not(feature = "legacy-tools"))]
fn run_fidelity_scenario_input_writer_command(_path: PathBuf) -> anyhow::Result<()> {
    Err(legacy_tools_disabled("--fidelity-write-scenario-inputs"))
}

#[cfg(not(feature = "legacy-tools"))]
fn legacy_tools_disabled(command: &'static str) -> anyhow::Error {
    anyhow::anyhow!(
        "{command} is developer legacy tooling; rebuild with --features legacy-tools to use it"
    )
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

pub(crate) fn run_game_smoke() -> anyhow::Result<()> {
    RuntimeHost::current().run_game_smoke()
}

pub(crate) fn run_actor_smoke() -> anyhow::Result<()> {
    RuntimeHost::current().run_actor_smoke()
}

pub(crate) fn run_actor_wgpu_smoke() -> anyhow::Result<()> {
    RuntimeHost::current().run_actor_wgpu_smoke()
}

pub(crate) fn run_fidelity_trace(frame_count: usize) -> anyhow::Result<()> {
    RuntimeHost::current().run_fidelity_trace(frame_count)
}

pub(crate) fn run_fidelity_trace_inputs(script: String) -> anyhow::Result<()> {
    RuntimeHost::current().run_fidelity_trace_inputs(script)
}

pub(crate) fn run_fidelity_trace_inputs_file(path: PathBuf) -> anyhow::Result<()> {
    RuntimeHost::current().run_fidelity_trace_inputs_file(path)
}

pub(crate) fn run_fidelity_trace_check(
    inputs_path: PathBuf,
    expected_path: PathBuf,
) -> anyhow::Result<()> {
    RuntimeHost::current().run_fidelity_trace_check(inputs_path, expected_path)
}

pub(crate) fn run_fidelity_trace_check_dir(path: PathBuf) -> anyhow::Result<()> {
    RuntimeHost::current().run_fidelity_trace_check_dir(path)
}

pub(crate) fn run_fidelity_reference_trace_check_dir(path: PathBuf) -> anyhow::Result<()> {
    RuntimeHost::current().run_fidelity_reference_trace_check_dir(path)
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
        "  cargo run -- --game-smoke\n",
        "  cargo run -- --actor-smoke\n",
        "  cargo run -- --actor-wgpu-smoke\n",
        "  cargo run -- --mute\n",
        "  cargo run -- --input-profile planetoid\n",
        "  cargo run -- --input-profile cabinet\n",
        "  cargo run -- --cmos-path ~/.local/state/defender/red-label-cmos.bin\n",
        "  cargo run --features legacy-tools -- --rom-report\n",
        "  cargo run --features legacy-tools -- --rom-report /path/to/roms\n",
        "  cargo run --features legacy-tools -- --verify-roms /path/to/roms\n",
        "  cargo run --features legacy-tools -- --fidelity-trace 300\n",
        "  cargo run --features legacy-tools -- --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'\n",
        "  cargo run --features legacy-tools -- --fidelity-trace-inputs-file /path/to/inputs.txt\n",
        "  cargo run --features legacy-tools -- --fidelity-check-trace /path/to/inputs.txt /path/to/expected.tsv\n",
        "  cargo run --features legacy-tools -- --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current\n",
        "  cargo run --features legacy-tools -- --fidelity-list-scenarios\n",
        "  cargo run --features legacy-tools -- --fidelity-write-scenario-inputs docs/fidelity/fixtures/local/reference\n",
        "  cargo run --features legacy-tools -- --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local/reference\n",
        "\n",
        "Runtime assets are embedded in the binary for copy-only deployment.\n",
        "Live play uses the windowed wgpu backend.\n",
        "ROM and accepted-trace commands are explicit legacy developer tooling.\n",
        "Live audio routes accepted sound commands through a non-blocking synthesized device backend; ",
        "--mute disables that runtime path.\n",
    )
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    #[cfg(feature = "legacy-tools")]
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::platform::{AudioOutput, ControlProfile, RunMode, RuntimeConfig};
    use crate::{audio::LiveAudioMode, live_wgpu::LiveInputProfile};

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
                input_profile: LiveInputProfile::Cabinet,
                audio_mode: LiveAudioMode::Disabled,
                cmos_path: Some(PathBuf::from("scores.bin")),
            }]
        );
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
    fn runtime_host_launches_game_smoke_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_game_smoke()
            .expect("runtime host should run clean game smoke command");

        let observed = calls.borrow();
        assert_eq!(observed.as_slice(), &[RuntimeCommand::GameSmoke]);
    }

    #[test]
    fn runtime_host_launches_actor_smoke_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_actor_smoke()
            .expect("runtime host should run actor smoke command");

        let observed = calls.borrow();
        assert_eq!(observed.as_slice(), &[RuntimeCommand::ActorSmoke]);
    }

    #[test]
    fn runtime_host_launches_actor_wgpu_smoke_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_actor_wgpu_smoke()
            .expect("runtime host should run actor wgpu smoke command");

        let observed = calls.borrow();
        assert_eq!(observed.as_slice(), &[RuntimeCommand::ActorWgpuSmoke]);
    }

    #[test]
    fn runtime_host_launches_fidelity_trace_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_fidelity_trace(300)
            .expect("runtime host should run fidelity trace command");

        let observed = calls.borrow();
        assert_eq!(
            observed.as_slice(),
            &[RuntimeCommand::FidelityTrace { frame_count: 300 }]
        );
    }

    #[test]
    fn runtime_host_launches_fidelity_trace_inputs_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_fidelity_trace_inputs(String::from("none"))
            .expect("runtime host should run inline trace inputs command");
        host.run_fidelity_trace_inputs_file(PathBuf::from("inputs.txt"))
            .expect("runtime host should run file trace inputs command");

        let observed = calls.borrow();
        assert_eq!(
            observed.as_slice(),
            &[
                RuntimeCommand::FidelityTraceInputs {
                    script: String::from("none"),
                },
                RuntimeCommand::FidelityTraceInputsFile {
                    path: PathBuf::from("inputs.txt"),
                },
            ]
        );
    }

    #[test]
    fn runtime_host_launches_fidelity_trace_check_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_fidelity_trace_check(PathBuf::from("inputs.txt"), PathBuf::from("expected.tsv"))
            .expect("runtime host should run trace check command");

        let observed = calls.borrow();
        assert_eq!(
            observed.as_slice(),
            &[RuntimeCommand::FidelityTraceCheck {
                inputs_path: PathBuf::from("inputs.txt"),
                expected_path: PathBuf::from("expected.tsv"),
            }]
        );
    }

    #[test]
    fn runtime_host_launches_fidelity_trace_fixture_directory_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_fidelity_trace_check_dir(PathBuf::from("fixtures"))
            .expect("runtime host should run trace fixture directory command");

        let observed = calls.borrow();
        assert_eq!(
            observed.as_slice(),
            &[RuntimeCommand::FidelityTraceFixtureDirectory {
                path: PathBuf::from("fixtures"),
            }]
        );
    }

    #[test]
    fn runtime_host_launches_fidelity_reference_trace_fixture_directory_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_fidelity_reference_trace_check_dir(PathBuf::from("fixtures"))
            .expect("runtime host should run reference trace fixture directory command");

        let observed = calls.borrow();
        assert_eq!(
            observed.as_slice(),
            &[RuntimeCommand::FidelityReferenceTraceFixtureDirectory {
                path: PathBuf::from("fixtures"),
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
                input_profile: LiveInputProfile::Planetoid,
                audio_mode: LiveAudioMode::Device,
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
                input_profile: LiveInputProfile::Test,
                cmos_path: Some(PathBuf::from("smoke_cmos.bin")),
            }
        );
    }

    #[test]
    fn clean_audio_outputs_map_to_runtime_audio_modes() {
        assert_eq!(audio_mode(AudioOutput::Disabled), LiveAudioMode::Disabled);
        assert_eq!(audio_mode(AudioOutput::Device), LiveAudioMode::Device);
        assert_eq!(audio_mode(AudioOutput::Null), LiveAudioMode::Null);
    }

    #[test]
    fn clean_control_profiles_map_to_runtime_input_profiles() {
        assert_eq!(
            input_profile(ControlProfile::Planetoid),
            LiveInputProfile::Planetoid
        );
        assert_eq!(
            input_profile(ControlProfile::Cabinet),
            LiveInputProfile::Cabinet
        );
        assert_eq!(input_profile(ControlProfile::Test), LiveInputProfile::Test);
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

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn installed_backend_runs_clean_rom_listing_report() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_rom_report(None)
            .expect("installed backend should run clean ROM listing report");
    }

    #[cfg(feature = "legacy-tools")]
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
    fn installed_backend_runs_clean_game_smoke() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_game_smoke()
            .expect("installed backend should run clean game smoke");
    }

    #[test]
    fn installed_backend_runs_actor_smoke() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_actor_smoke()
            .expect("installed backend should run actor smoke");
    }

    #[test]
    fn installed_backend_runs_actor_wgpu_smoke() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_actor_wgpu_smoke()
            .expect("installed backend should run actor wgpu smoke");
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn installed_backend_runs_clean_fidelity_scenario_list() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_fidelity_scenario_list()
            .expect("installed backend should run clean scenario list");
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn installed_backend_runs_clean_fidelity_trace() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_fidelity_trace(1)
            .expect("installed backend should run clean fidelity trace");
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn installed_backend_runs_clean_fidelity_trace_inputs() {
        let path = unique_temp_dir("defender-clean-runtime-trace-inputs");
        fs::create_dir_all(&path).expect("create temp dir");
        let script_path = path.join("inputs.txt");
        fs::write(&script_path, "none\n").expect("write trace input script");

        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_fidelity_trace_inputs(String::from("none"))
            .expect("installed backend should run clean inline trace inputs");
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_fidelity_trace_inputs_file(script_path)
            .expect("installed backend should run clean file trace inputs");
        let _ = fs::remove_dir_all(path);
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn installed_backend_runs_clean_fidelity_trace_check() {
        let path = unique_temp_dir("defender-clean-runtime-trace-check");
        fs::create_dir_all(&path).expect("create temp dir");
        let inputs_path = path.join("inputs.txt");
        let expected_path = path.join("expected.tsv");
        fs::write(&inputs_path, "none\n").expect("write trace input script");
        fs::write(&expected_path, one_frame_idle_trace_text()).expect("write expected trace");

        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_fidelity_trace_check(inputs_path, expected_path)
            .expect("installed backend should run clean trace check");
        let _ = fs::remove_dir_all(path);
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn installed_backend_runs_clean_fidelity_trace_fixture_directory() {
        let path = unique_temp_dir("defender-clean-runtime-trace-fixtures");
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("boot.inputs.txt"), "none\n").expect("write fixture input");
        fs::write(path.join("boot.expected.tsv"), one_frame_idle_trace_text())
            .expect("write expected trace");

        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_fidelity_trace_check_dir(path.clone())
            .expect("installed backend should run clean trace fixture directory");
        let _ = fs::remove_dir_all(path);
    }

    #[cfg(feature = "legacy-tools")]
    #[test]
    fn installed_backend_runs_clean_fidelity_reference_trace_fixture_directory() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_fidelity_reference_trace_check_dir(PathBuf::from(
                "docs/fidelity/fixtures/local/reference",
            ))
            .expect("installed backend should run clean reference trace fixture directory");
    }

    #[cfg(feature = "legacy-tools")]
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

    #[cfg(not(feature = "legacy-tools"))]
    #[test]
    fn installed_backend_keeps_legacy_tooling_out_of_default_runtime() {
        let error = RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_rom_report(None)
            .expect_err("default runtime should not compile legacy ROM tooling");

        assert!(error.to_string().contains("legacy tooling"));
        assert!(error.to_string().contains("--features legacy-tools"));
    }

    #[test]
    fn clean_help_text_preserves_current_cli_contract() {
        let text = help_text();

        assert!(text.starts_with("defender\n  cargo run\n"));
        assert!(text.contains("--rom-report"));
        assert!(text.contains("--input-profile planetoid"));
        assert!(text.contains("--input-profile cabinet"));
        assert!(text.contains("--live-smoke"));
        assert!(text.contains("--game-smoke"));
        assert!(text.contains("--actor-smoke"));
        assert!(text.contains("--actor-wgpu-smoke"));
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
