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

    pub(crate) fn run_actor_script_check(&self, path: PathBuf) -> anyhow::Result<()> {
        self.backend
            .run_command(RuntimeCommand::ActorScriptCheck { path })
    }

    pub(crate) fn run_actor_smoke(&self) -> anyhow::Result<()> {
        self.backend.run_command(RuntimeCommand::ActorSmoke)
    }

    pub(crate) fn run_actor_attract_smoke(&self) -> anyhow::Result<()> {
        self.backend.run_command(RuntimeCommand::ActorAttractSmoke)
    }

    pub(crate) fn run_actor_post_game_smoke(&self) -> anyhow::Result<()> {
        self.backend.run_command(RuntimeCommand::ActorPostGameSmoke)
    }

    pub(crate) fn run_actor_wgpu_smoke(&self) -> anyhow::Result<()> {
        self.backend.run_command(RuntimeCommand::ActorWgpuSmoke)
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
    ActorScriptCheck {
        path: PathBuf,
    },
    ActorSmoke,
    ActorAttractSmoke,
    ActorPostGameSmoke,
    ActorWgpuSmoke,
    ActorWgpuLive {
        input_profile: LiveInputProfile,
        audio_mode: LiveAudioMode,
        cmos_path: Option<PathBuf>,
        actor_script_path: Option<PathBuf>,
    },
    WgpuLiveSmoke {
        input_profile: LiveInputProfile,
        cmos_path: Option<PathBuf>,
    },
}

impl RuntimeCommand {
    fn from_config(config: &RuntimeConfig) -> Self {
        match config.mode {
            RunMode::Interactive => Self::ActorWgpuLive {
                input_profile: input_profile(config.controls),
                audio_mode: audio_mode(config.audio),
                cmos_path: config.cmos_path.clone(),
                actor_script_path: config.actor_script_path.clone(),
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
            RuntimeCommand::ActorScriptCheck { path } => {
                let report = crate::live_wgpu::run_actor_script_check(&path)?;
                print!("{}", report.to_text());
                Ok(())
            }
            RuntimeCommand::ActorSmoke => crate::actor_smoke::run(),
            RuntimeCommand::ActorAttractSmoke => crate::actor_smoke::run_attract_cycle(),
            RuntimeCommand::ActorPostGameSmoke => crate::actor_smoke::run_post_game(),
            RuntimeCommand::ActorWgpuSmoke => {
                let report = crate::live_wgpu::run_actor_wgpu_smoke()?;
                print!("{}", report.to_text());
                Ok(())
            }
            RuntimeCommand::ActorWgpuLive {
                input_profile,
                audio_mode,
                cmos_path,
                actor_script_path,
            } => crate::live_wgpu::run_actor_live(
                input_profile,
                audio_mode,
                cmos_path.as_deref(),
                actor_script_path.as_deref(),
            ),
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

pub(crate) fn run_help() -> anyhow::Result<()> {
    RuntimeHost::current().run_help()
}

pub(crate) fn run_actor_script_check(path: PathBuf) -> anyhow::Result<()> {
    RuntimeHost::current().run_actor_script_check(path)
}

pub(crate) fn run_actor_smoke() -> anyhow::Result<()> {
    RuntimeHost::current().run_actor_smoke()
}

pub(crate) fn run_actor_attract_smoke() -> anyhow::Result<()> {
    RuntimeHost::current().run_actor_attract_smoke()
}

pub(crate) fn run_actor_post_game_smoke() -> anyhow::Result<()> {
    RuntimeHost::current().run_actor_post_game_smoke()
}

pub(crate) fn run_actor_wgpu_smoke() -> anyhow::Result<()> {
    RuntimeHost::current().run_actor_wgpu_smoke()
}

pub(crate) fn run(config: &RuntimeConfig) -> anyhow::Result<()> {
    RuntimeHost::current().run(config)
}

pub(crate) fn help_text() -> &'static str {
    concat!(
        "defender\n",
        "  cargo run\n",
        "  cargo run -- --actor-live\n",
        "  cargo run -- --actor-script /path/to/driver.script\n",
        "  cargo run -- --live-smoke\n",
        "  cargo run -- --actor-script-check /path/to/driver.script\n",
        "  cargo run -- --actor-smoke\n",
        "  cargo run -- --actor-attract-smoke\n",
        "  cargo run -- --actor-post-game-smoke\n",
        "  cargo run -- --actor-wgpu-smoke\n",
        "  cargo run -- --mute\n",
        "  cargo run -- --input-profile planetoid\n",
        "  cargo run -- --input-profile cabinet\n",
        "  cargo run -- --cmos-path ~/.local/state/defender/cabinet-cmos.bin\n",
        "\n",
        "Runtime assets are embedded in the binary for copy-only deployment.\n",
        "Live play uses the windowed wgpu backend.\n",
        "Live audio routes accepted sound commands through a non-blocking synthesized device backend; ",
        "--mute disables that runtime path.\n",
    )
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

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
    fn runtime_host_adapts_default_config_to_actor_launch_command() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });
        let config = RuntimeConfig {
            controls: ControlProfile::Cabinet,
            audio: AudioOutput::Disabled,
            mode: RunMode::Interactive,
            cmos_path: Some(PathBuf::from("scores.bin")),
            actor_script_path: Some(PathBuf::from("driver.script")),
        };

        host.run(&config).expect("runtime host should run backend");

        assert_eq!(
            calls.borrow().as_slice(),
            &[RuntimeCommand::ActorWgpuLive {
                input_profile: LiveInputProfile::Cabinet,
                audio_mode: LiveAudioMode::Disabled,
                cmos_path: Some(PathBuf::from("scores.bin")),
                actor_script_path: Some(PathBuf::from("driver.script")),
            }]
        );
    }

    #[test]
    fn runtime_host_launches_direct_commands_separately() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });

        host.run_help().expect("help command");
        host.run_actor_script_check(PathBuf::from("driver.script"))
            .expect("actor script check command");
        host.run_actor_smoke().expect("actor smoke command");
        host.run_actor_attract_smoke()
            .expect("actor attract smoke command");
        host.run_actor_post_game_smoke()
            .expect("actor post-game smoke command");
        host.run_actor_wgpu_smoke()
            .expect("actor wgpu smoke command");

        assert_eq!(
            calls.borrow().as_slice(),
            &[
                RuntimeCommand::Help,
                RuntimeCommand::ActorScriptCheck {
                    path: PathBuf::from("driver.script"),
                },
                RuntimeCommand::ActorSmoke,
                RuntimeCommand::ActorAttractSmoke,
                RuntimeCommand::ActorPostGameSmoke,
                RuntimeCommand::ActorWgpuSmoke,
            ]
        );
    }

    #[test]
    fn smoke_config_uses_wgpu_live_smoke_launch() {
        let config = RuntimeConfig {
            controls: ControlProfile::Test,
            audio: AudioOutput::Null,
            mode: RunMode::Smoke,
            cmos_path: Some(PathBuf::from("smoke_cmos.bin")),
            actor_script_path: None,
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
    fn installed_backend_runs_clean_help() {
        RuntimeHost::with_backend(InstalledRuntimeBackend)
            .run_help()
            .expect("installed backend should run clean help");
    }

    #[test]
    fn clean_help_text_preserves_current_cli_contract() {
        let text = help_text();

        assert!(text.starts_with("defender\n  cargo run\n"));
        assert!(text.contains("--actor-live"));
        assert!(text.contains("--actor-script /path/to/driver.script"));
        assert!(text.contains("--input-profile planetoid"));
        assert!(text.contains("--input-profile cabinet"));
        assert!(text.contains("--live-smoke"));
        assert!(!text.contains("--game-smoke"));
        assert!(text.contains("--actor-script-check /path/to/driver.script"));
        assert!(text.contains("--actor-smoke"));
        assert!(text.contains("--actor-attract-smoke"));
        assert!(text.contains("--actor-post-game-smoke"));
        assert!(text.contains("--actor-wgpu-smoke"));
        assert!(text.contains("--mute"));
        assert!(!text.contains("--memory-report"));
        assert!(!text.contains("--fidelity"));
        assert!(text.contains("copy-only deployment"));
        assert!(text.contains("uses the windowed wgpu"));
    }
}
