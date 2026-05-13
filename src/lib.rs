//! Clean Defender rewrite entry point.
//!
//! New source lives in this `src/` directory. The converted implementation is
//! parked under `src_legacy/` and remains wired only as the gameplay oracle and
//! compatibility runtime while the rewrite proves equivalent behavior.

pub mod audio;
pub mod game;
pub mod oracle;
pub mod platform;
pub mod renderer;
pub mod systems;

// Compatibility modules are hidden from the supported clean API surface while
// the rewrite still uses them for the CLI, oracle, fixtures, and smoke tests.
#[doc(hidden)]
#[path = "../src_legacy/app.rs"]
pub mod app;
#[doc(hidden)]
#[path = "../src_legacy/assets.rs"]
pub mod assets;
#[doc(hidden)]
#[path = "../src_legacy/board.rs"]
pub mod board;
#[doc(hidden)]
#[path = "../src_legacy/cmos_storage.rs"]
pub mod cmos_storage;
#[doc(hidden)]
#[path = "../src_legacy/fidelity.rs"]
pub mod fidelity;
#[doc(hidden)]
#[path = "../src_legacy/input.rs"]
pub mod input;
#[doc(hidden)]
#[path = "../src_legacy/live.rs"]
pub mod live;
#[doc(hidden)]
#[path = "../src_legacy/machine.rs"]
pub mod machine;
#[doc(hidden)]
#[path = "../src_legacy/machine_process.rs"]
pub mod machine_process;
#[doc(hidden)]
#[path = "../src_legacy/machine_state.rs"]
pub mod machine_state;
#[doc(hidden)]
#[path = "../src_legacy/pia.rs"]
pub mod pia;
#[doc(hidden)]
#[path = "../src_legacy/red_label.rs"]
pub mod red_label;
#[doc(hidden)]
#[path = "../src_legacy/red_label_memory.rs"]
pub mod red_label_memory;
#[doc(hidden)]
#[path = "../src_legacy/red_label_message.rs"]
pub mod red_label_message;
#[doc(hidden)]
#[path = "../src_legacy/red_label_trace_samples.rs"]
pub(crate) mod red_label_trace_samples;
#[doc(hidden)]
#[path = "../src_legacy/red_label_wave.rs"]
pub mod red_label_wave;
#[doc(hidden)]
#[path = "../src_legacy/rom.rs"]
pub mod rom;
#[doc(hidden)]
#[path = "../src_legacy/sound.rs"]
pub mod sound;
#[doc(hidden)]
#[path = "../src_legacy/terminal.rs"]
pub mod terminal;
#[doc(hidden)]
#[path = "../src_legacy/video.rs"]
pub mod video;
#[doc(hidden)]
#[path = "../src_legacy/wgpu_presenter.rs"]
pub mod wgpu_presenter;

pub use game::{
    Direction, GameEvent, GameEvents, GameFrame, GameInput, GamePhase, GameSnapshot, GameState,
    PlayerSnapshot, ScoreSnapshot, SoundEvent, WorldVector,
};
pub use oracle::GameplayOracle;
pub use platform::{AudioOutput, ControlProfile, RunMode, RuntimeConfig};
pub use renderer::{
    AtlasRegion, Color, FontAtlas, GpuRendererSettings, NativeRenderPipeline,
    NativeRendererResources, NativeSceneRenderer, PaletteResource, RenderLayer, RenderLayerCounts,
    RenderScene, RenderSceneSummary, SceneDrawPlan, SceneRaster, SceneRasterError,
    SceneRasterUpload, SceneSprite, SpriteId, SurfaceSize, TextureAtlas,
};
pub use systems::{
    FixedStepAccumulator, FrameRate, GameSimulation, PlayerActionTriggers, PlayerControlFrame,
    PlayerControlIntent, PlayerControlSystem, PlayerMotionFrame, PlayerMotionState,
    PlayerMotionSystem, ProjectileLaunchOutcome, ProjectileState, ProjectileSystem, ScreenPosition,
    VerticalControl, advance_one_frame,
};

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
    fn binary_entrypoint_uses_clean_platform_runtime_boundary() {
        let main_rs = include_str!("main.rs");
        let legacy_call = format!("{}::{}::{}()", "defender", "app", "run");

        assert!(main_rs.contains("defender::platform::run()"));
        assert!(!main_rs.contains(&legacy_call));
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

    #[test]
    fn legacy_compatibility_modules_are_hidden_from_supported_api_docs() {
        let lib_rs = include_str!("lib.rs");
        let legacy_modules = [
            "app",
            "assets",
            "board",
            "cmos_storage",
            "fidelity",
            "input",
            "live",
            "machine",
            "machine_process",
            "machine_state",
            "pia",
            "red_label",
            "red_label_memory",
            "red_label_message",
            "red_label_trace_samples",
            "red_label_wave",
            "rom",
            "sound",
            "terminal",
            "video",
            "wgpu_presenter",
        ];

        for module in legacy_modules {
            let marker = format!("#[path = \"../src_legacy/{module}.rs\"]");
            let Some(marker_start) = lib_rs.find(&marker) else {
                panic!("missing legacy module path for {module}");
            };
            assert!(
                lib_rs[..marker_start].ends_with("#[doc(hidden)]\n"),
                "legacy module {module} must be hidden from supported API docs"
            );
        }
    }
}
