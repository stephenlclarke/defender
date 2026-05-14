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

pub(crate) fn run(config: &RuntimeConfig) -> anyhow::Result<()> {
    RuntimeHost::current().run(config)
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    use crate::platform::{AudioOutput, ControlProfile, RunMode, RuntimeConfig};
    use crate::{audio::LiveAudioMode, input::InputProfile};

    use super::{
        InstalledRuntimeBackend, RuntimeBackend, RuntimeCommand, RuntimeHost, audio_mode,
        input_profile,
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
}
