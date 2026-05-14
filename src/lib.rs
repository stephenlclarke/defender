//! Clean Defender rewrite entry point.
//!
//! New source lives in this `src/` directory. The converted implementation is
//! parked under `src_legacy/` and remains wired only as the gameplay oracle and
//! runtime bridge while the rewrite proves equivalent behavior.

mod accepted;

pub mod audio;
pub mod game;
pub mod oracle;
pub mod platform;
pub mod renderer;
pub mod systems;

// Legacy bridge modules are hidden from the supported clean API surface while
// the rewrite still uses them for the CLI, oracle, fixtures, and smoke tests.
// Parked low-level modules tolerate dead code after removal from public tool
// facades; later rewrite cycles should delete them when their evidence is no
// longer needed.
#[doc(hidden)]
#[path = "../src_legacy/accepted_behavior.rs"]
pub(crate) mod accepted_behavior;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/app.rs"]
pub(crate) mod app;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/assets.rs"]
pub(crate) mod assets;
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
#[doc(hidden)]
#[path = "../src_legacy/board.rs"]
pub(crate) mod board;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/cmos_storage.rs"]
pub(crate) mod cmos_storage;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/fidelity.rs"]
pub(crate) mod fidelity;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/input.rs"]
pub(crate) mod input;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/live.rs"]
pub(crate) mod live;
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
#[doc(hidden)]
#[path = "../src_legacy/machine.rs"]
pub(crate) mod machine;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/machine_process.rs"]
pub(crate) mod machine_process;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/machine_state.rs"]
pub(crate) mod machine_state;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/pia.rs"]
pub(crate) mod pia;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/red_label.rs"]
pub(crate) mod red_label;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/red_label_memory.rs"]
pub(crate) mod red_label_memory;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/red_label_message.rs"]
pub(crate) mod red_label_message;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/red_label_wave.rs"]
pub(crate) mod red_label_wave;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/rom.rs"]
pub(crate) mod rom;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/sound.rs"]
pub(crate) mod sound;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/video.rs"]
pub(crate) mod video;
#[doc(hidden)]
#[path = "../src_legacy/wgpu_presenter.rs"]
pub(crate) mod wgpu_presenter;

#[doc(hidden)]
#[path = "../src_legacy/readme_media.rs"]
pub mod readme_media;

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
#[path = "../src_legacy/oracle_equivalence_tests.rs"]
mod oracle_equivalence_tests;

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
    fn readme_media_facade_is_legacy_owned_and_doc_hidden() {
        let lib_rs = include_str!("lib.rs");
        let marker = "#[path = \"../src_legacy/readme_media.rs\"]";
        let Some(marker_start) = lib_rs.find(marker) else {
            panic!("missing README media facade path");
        };

        assert!(
            lib_rs[..marker_start].ends_with("#[doc(hidden)]\n"),
            "README media facade must be hidden from supported API docs"
        );
        assert!(
            lib_rs[marker_start..].starts_with(&format!("{marker}\npub mod readme_media;")),
            "README media facade must be owned by src_legacy"
        );
        assert!(
            !lib_rs.contains("pub mod readme_media {\n"),
            "README media facade details must stay out of clean src/lib.rs"
        );
    }

    #[test]
    fn compatibility_namespace_is_retired() {
        let lib_rs = include_str!("lib.rs");
        let example_rs = include_str!("../examples/generate_readme_media.rs");
        let compatibility_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("src_legacy/{}.rs", "compatibility"));
        let old_module_declaration = format!("pub mod {}", "compatibility");
        let old_path_attribute = format!("#[path = \"../src_legacy/{}.rs\"]", "compatibility");
        let old_tool_path = format!("defender::{}", "compatibility");

        assert!(!compatibility_path.exists());
        assert!(!lib_rs.contains(&old_module_declaration));
        assert!(!lib_rs.contains(&old_path_attribute));
        assert!(!example_rs.contains(&old_tool_path));
        assert!(example_rs.contains("defender::readme_media"));
    }

    #[test]
    fn legacy_equivalence_tests_use_crate_private_oracle_wiring() {
        let oracle_equivalence_tests_rs = include_str!("../src_legacy/oracle_equivalence_tests.rs");

        for forbidden in [
            "compatibility::",
            "compatibility::{",
            "crate::compatibility",
        ] {
            assert!(
                !oracle_equivalence_tests_rs.contains(forbidden),
                "legacy equivalence tests must use crate-private oracle wiring instead of {forbidden}"
            );
        }

        for required in ["input::{", "machine::{"] {
            assert!(
                oracle_equivalence_tests_rs.contains(required),
                "legacy equivalence tests must keep explicit crate-private {required} imports"
            );
        }
    }

    #[test]
    fn clean_runtime_and_oracle_use_accepted_facade() {
        let platform_rs = include_str!("platform.rs");
        assert!(platform_rs.contains("crate::accepted::run_runtime()"));
        assert!(!platform_rs.contains("crate::compatibility::"));
        assert!(!platform_rs.contains("crate::app::run()"));

        let oracle_rs = include_str!("oracle.rs");
        assert!(oracle_rs.contains("crate::accepted::"));
        for forbidden in [
            "crate::compatibility::",
            "crate::input::",
            "crate::machine::",
            "crate::machine_state::",
            "crate::red_label::",
            "crate::video::",
        ] {
            assert!(
                !oracle_rs.contains(forbidden),
                "clean oracle boundary must use accepted facade instead of {forbidden}"
            );
        }

        for forbidden in ["red_label", "RED_LABEL", "source routine", "assembler"] {
            assert!(
                !oracle_rs.contains(forbidden),
                "clean oracle source must not expose legacy terminology {forbidden}"
            );
        }

        let accepted_rs = include_str!("accepted.rs");
        assert!(accepted_rs.contains("crate::accepted_behavior::"));
        for forbidden in [
            "crate::compatibility::",
            "crate::input::",
            "crate::machine::",
            "crate::machine_state::",
            "crate::red_label::",
            "crate::video::",
            "crate::app::",
        ] {
            assert!(
                !accepted_rs.contains(forbidden),
                "accepted facade must use accepted_behavior adapter instead of {forbidden}"
            );
        }
    }

    #[test]
    fn legacy_modules_are_crate_private_at_root() {
        let lib_rs = include_str!("lib.rs");
        let legacy_modules = [
            "accepted_behavior",
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
            "red_label_wave",
            "rom",
            "sound",
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
            assert!(
                lib_rs[marker_start..].starts_with(&format!("{marker}\npub(crate) mod {module};")),
                "legacy module {module} must be crate-private at the root"
            );
        }
    }

    #[test]
    fn generated_trace_samples_are_machine_oracle_private() {
        let lib_rs = include_str!("lib.rs");
        let machine_rs = include_str!("../src_legacy/machine.rs");
        let fixture_module = format!("{}_trace_samples", "red_label");
        let fixture_path = format!("../src_legacy/{fixture_module}.rs");
        let root_module = format!("mod {fixture_module};");
        let machine_module = format!("#[path = \"{fixture_module}.rs\"]\nmod {fixture_module};");

        assert!(
            !lib_rs.contains(&fixture_path),
            "generated trace samples must not be wired from clean crate root"
        );
        assert!(
            !lib_rs.contains(&root_module),
            "generated trace samples must stay private to the legacy machine oracle"
        );
        assert!(
            machine_rs.contains(&machine_module),
            "legacy machine oracle must own its generated trace sample fixture module"
        );
    }

    #[test]
    fn legacy_terminal_session_is_not_active_crate_wiring() {
        let lib_rs = include_str!("lib.rs");
        let module_declaration = format!("{} {};", "mod", "terminal");

        assert!(
            !lib_rs.contains("#[path = \"../src_legacy/terminal.rs\"]"),
            "terminal session code must stay parked outside active crate wiring"
        );
        assert!(
            !lib_rs.contains(&module_declaration),
            "terminal session code must not be compiled as an active root module"
        );
    }
}
