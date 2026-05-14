//! Private launch bridge for the playable runtime.

use crate::platform::RuntimeConfig;

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
    pub(crate) fn run(&self, config: &RuntimeConfig) -> anyhow::Result<()> {
        self.backend.run(config)
    }
}

pub(crate) trait RuntimeBackend {
    fn run(&self, config: &RuntimeConfig) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct InstalledRuntimeBackend;

impl RuntimeBackend for InstalledRuntimeBackend {
    fn run(&self, _config: &RuntimeConfig) -> anyhow::Result<()> {
        crate::accepted_behavior::run_runtime()
    }
}

pub(crate) fn run(config: &RuntimeConfig) -> anyhow::Result<()> {
    RuntimeHost::current().run(config)
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    use crate::platform::{AudioOutput, ControlProfile, RunMode, RuntimeConfig};

    use super::{InstalledRuntimeBackend, RuntimeBackend, RuntimeHost};

    #[derive(Debug, Clone, Default)]
    struct RecordingBackend {
        calls: Rc<RefCell<Vec<RuntimeConfig>>>,
    }

    impl RuntimeBackend for RecordingBackend {
        fn run(&self, config: &RuntimeConfig) -> anyhow::Result<()> {
            self.calls.borrow_mut().push(config.clone());
            Ok(())
        }
    }

    #[test]
    fn runtime_host_forwards_clean_config_to_backend() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let host = RuntimeHost::with_backend(RecordingBackend {
            calls: Rc::clone(&calls),
        });
        let config = RuntimeConfig {
            controls: ControlProfile::Cabinet,
            audio: AudioOutput::Disabled,
            mode: RunMode::Smoke,
            cmos_path: Some(PathBuf::from("scores.bin")),
        };

        host.run(&config).expect("runtime host should run backend");

        let observed = calls.borrow();
        assert_eq!(observed.as_slice(), std::slice::from_ref(&config));
    }

    #[test]
    fn current_runtime_uses_installed_backend() {
        assert_eq!(
            RuntimeHost::current(),
            RuntimeHost::with_backend(InstalledRuntimeBackend)
        );
    }
}
