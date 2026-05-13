//! Clean Defender rewrite entry point.
//!
//! New source lives in this `src/` directory. The converted implementation is
//! parked under `src_legacy/` and remains wired only as the gameplay oracle and
//! compatibility runtime while the rewrite proves equivalent behavior.

pub mod game;
pub mod oracle;
pub mod platform;
pub mod renderer;
pub mod systems;

#[path = "../src_legacy/app.rs"]
pub mod app;
#[path = "../src_legacy/assets.rs"]
pub mod assets;
#[path = "../src_legacy/audio.rs"]
pub mod audio;
#[path = "../src_legacy/board.rs"]
pub mod board;
#[path = "../src_legacy/cmos_storage.rs"]
pub mod cmos_storage;
#[path = "../src_legacy/fidelity.rs"]
pub mod fidelity;
#[path = "../src_legacy/input.rs"]
pub mod input;
#[path = "../src_legacy/live.rs"]
pub mod live;
#[path = "../src_legacy/machine.rs"]
pub mod machine;
#[path = "../src_legacy/machine_process.rs"]
pub mod machine_process;
#[path = "../src_legacy/machine_state.rs"]
pub mod machine_state;
#[path = "../src_legacy/pia.rs"]
pub mod pia;
#[path = "../src_legacy/red_label.rs"]
pub mod red_label;
#[path = "../src_legacy/red_label_memory.rs"]
pub mod red_label_memory;
#[path = "../src_legacy/red_label_message.rs"]
pub mod red_label_message;
#[path = "../src_legacy/red_label_trace_samples.rs"]
pub(crate) mod red_label_trace_samples;
#[path = "../src_legacy/red_label_wave.rs"]
pub mod red_label_wave;
#[path = "../src_legacy/rom.rs"]
pub mod rom;
#[path = "../src_legacy/sound.rs"]
pub mod sound;
#[path = "../src_legacy/terminal.rs"]
pub mod terminal;
#[path = "../src_legacy/video.rs"]
pub mod video;
#[path = "../src_legacy/wgpu_presenter.rs"]
pub mod wgpu_presenter;

pub use game::{
    Direction, GameEvent, GameEvents, GameFrame, GameInput, GamePhase, GameSnapshot, GameState,
    PlayerSnapshot, ScoreSnapshot, SoundEvent, WorldVector,
};
pub use oracle::GameplayOracle;
pub use platform::{AudioOutput, ControlProfile, RunMode, RuntimeConfig};
pub use renderer::{
    Color, GpuRendererSettings, RenderLayer, RenderLayerCounts, RenderScene, RenderSceneSummary,
    SceneSprite, SpriteId, SurfaceSize,
};
pub use systems::{FixedStepAccumulator, FrameRate, GameSimulation, advance_one_frame};

#[cfg(test)]
#[path = "../src_legacy/test_support.rs"]
pub(crate) mod test_support;

#[cfg(test)]
mod public_api_tests {
    #[test]
    fn clean_contracts_have_oracle_path() {
        let mut oracle = crate::GameplayOracle::new();
        let frame = oracle.step(crate::GameInput::NONE);

        assert_eq!(frame.state.frame, 1);
        assert_eq!(frame.state.phase, crate::GamePhase::Attract);
    }

    #[test]
    fn legacy_machine_state_contracts_remain_available_for_oracle_tests() {
        let direct_phase = crate::machine_state::GamePhase::Attract;
        let compatibility_phase: crate::machine::GamePhase = direct_phase;
        let direct_phase_again: crate::machine_state::GamePhase = compatibility_phase;
        assert_eq!(direct_phase_again, crate::machine_state::GamePhase::Attract);

        let direct = crate::machine_state::CompatibilityState::default();
        let compatibility: crate::machine::CompatibilityState = direct;
        assert!(!compatibility.xyzzy_active);
    }

    #[test]
    fn legacy_machine_process_contracts_remain_available_for_oracle_tests() {
        let direct =
            crate::machine_process::RedLabelScheduledProcess::from_source_disp(0xA05F, 0xC123);
        let compatibility: crate::machine::RedLabelScheduledProcess = direct;
        let direct_again: crate::machine_process::RedLabelScheduledProcess = compatibility;

        assert_eq!(direct_again.process_address, 0xA05F);
        assert_eq!(direct_again.routine_address, 0xC123);
    }
}
