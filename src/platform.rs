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
    crate::app::run()
}

#[cfg(test)]
mod tests {
    use super::{AudioOutput, ControlProfile, RunMode, RuntimeConfig};

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
    fn runtime_entrypoint_delegates_to_compatibility_runtime() {
        super::run().expect("compatibility runtime should run help under tests");
    }
}
