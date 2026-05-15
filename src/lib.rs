//! Clean Defender rewrite entry point.
//!
//! New source lives in this `src/` directory. The converted implementation is
//! parked under `src_legacy/` and remains wired only as the gameplay oracle and
//! runtime bridge while the rewrite proves equivalent behavior.

mod accepted;

pub mod audio;
pub mod fidelity;
mod fidelity_scenarios;
mod fidelity_traces;
pub mod game;
mod oracle;
pub mod platform;
pub mod renderer;
mod rom_report;
mod runtime;
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
#[path = "../src_legacy/input.rs"]
pub(crate) mod input;
#[allow(dead_code)]
#[doc(hidden)]
#[path = "../src_legacy/fidelity.rs"]
pub(crate) mod legacy_fidelity;
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

pub use fidelity::GameplayEquivalenceSignature;
pub use game::{
    Direction, Game, GameEvent, GameEvents, GameFrame, GameInput, GamePhase, GameSnapshot,
    GameState, PlayerSnapshot, ScoreSnapshot, SoundEvent, WorldVector,
};
pub use platform::{AudioOutput, ControlProfile, RunMode, RuntimeConfig};
pub use renderer::{
    AtlasRegion, Color, FontAtlas, GpuRendererSettings, NativeRenderPipeline,
    NativeRendererResources, NativeSceneRenderer, PaletteResource, RenderLayer, RenderLayerCounts,
    RenderScene, RenderSceneSummary, SceneDrawPlan, SceneProjectionUniforms, SceneRaster,
    SceneRasterError, SceneRasterUpload, SceneSprite, SpriteDrawBatch, SpriteDrawInstance,
    SpriteId, SurfaceSize, TextureAtlas, ViewportLayout, WgpuPassPlan, WgpuViewportCommand,
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
    fn clean_contracts_have_public_game_simulation() {
        let mut game = crate::Game::new();
        let frame = crate::advance_one_frame(&mut game, crate::GameInput::NONE);

        assert_eq!(frame.state.frame, 1);
        assert_eq!(frame.state.phase, crate::GamePhase::Attract);
        assert_eq!(frame.scene.summary().layers.hud, 1);
    }

    #[test]
    fn gameplay_oracle_is_internal_fidelity_wiring() {
        let lib_rs = include_str!("lib.rs");
        let mut oracle = crate::oracle::GameplayOracle::new();
        let frame = oracle.step(crate::GameInput::NONE);
        let public_module = format!("pub {} oracle;", "mod");
        let public_export = format!("pub use {}::GameplayOracle;", "oracle");

        assert_eq!(frame.state.frame, 1);
        assert!(lib_rs.contains("mod oracle;"));
        assert!(!lib_rs.contains(&public_module));
        assert!(!lib_rs.contains(&public_export));
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
    fn clean_runtime_and_oracle_use_quarantined_adapters() {
        let platform_rs = include_str!("platform.rs");
        let accepted_runtime_call = format!("crate::{}::{}()", "accepted", "run_runtime");
        let app_runtime_call = format!("crate::{}::{}()", "app", "run");

        assert!(platform_rs.contains("crate::runtime::run(&config)"));
        assert!(platform_rs.contains("crate::runtime::run_help()"));
        assert!(platform_rs.contains("crate::runtime::run_rom_report(request.path)"));
        assert!(platform_rs.contains("crate::runtime::run_verify_roms(request.path)"));
        assert!(platform_rs.contains("crate::runtime::run_fidelity_trace(request.frame_count)"));
        assert!(platform_rs.contains("crate::runtime::run_fidelity_trace_inputs(request.script)"));
        assert!(
            platform_rs.contains("crate::runtime::run_fidelity_trace_inputs_file(request.path)")
        );
        assert!(platform_rs.contains(
            "crate::runtime::run_fidelity_trace_check(request.inputs_path, request.expected_path)"
        ));
        assert!(platform_rs.contains("crate::runtime::run_fidelity_trace_check_dir(request.path)"));
        assert!(
            platform_rs
                .contains("crate::runtime::run_fidelity_reference_trace_check_dir(request.path)")
        );
        assert!(platform_rs.contains("crate::runtime::run_fidelity_scenario_list()"));
        assert!(
            platform_rs
                .contains("crate::runtime::run_fidelity_scenario_input_writer(request.path)")
        );
        assert!(platform_rs.contains("RuntimeCliClassifier::classify(args)"));
        assert!(platform_rs.contains("fn dispatch_cli_classification"));
        assert!(platform_rs.contains("CliClassification::RomReport(request)"));
        assert!(platform_rs.contains("CliClassification::VerifyRoms(request)"));
        assert!(platform_rs.contains("CliClassification::FidelityTrace(request)"));
        assert!(platform_rs.contains("CliClassification::FidelityTraceInputs(request)"));
        assert!(platform_rs.contains("CliClassification::FidelityTraceInputsFile(request)"));
        assert!(platform_rs.contains("CliClassification::FidelityTraceCheck(request)"));
        assert!(platform_rs.contains("CliClassification::FidelityTraceFixtureDirectory(request)"));
        assert!(
            platform_rs
                .contains("CliClassification::FidelityReferenceTraceFixtureDirectory(request)")
        );
        assert!(platform_rs.contains("CliClassification::FidelityScenarioList"));
        assert!(platform_rs.contains("CliClassification::FidelityScenarioInputWriter"));
        assert!(platform_rs.contains("struct RomReportRequest"));
        assert!(platform_rs.contains("struct VerifyRomsRequest"));
        assert!(platform_rs.contains("struct FidelityTraceRequest"));
        assert!(platform_rs.contains("struct FidelityTraceInputsRequest"));
        assert!(platform_rs.contains("struct FidelityTraceInputsFileRequest"));
        assert!(platform_rs.contains("struct FidelityTraceCheckRequest"));
        assert!(platform_rs.contains("struct FidelityTraceFixtureDirectoryRequest"));
        assert!(platform_rs.contains("struct FidelityReferenceTraceFixtureDirectoryRequest"));
        assert!(platform_rs.contains("struct ScenarioInputWriterRequest"));
        assert!(platform_rs.contains("\"--rom-report\" =>"));
        assert!(platform_rs.contains("\"--verify-roms\" =>"));
        assert!(platform_rs.contains("\"--fidelity-trace\" =>"));
        assert!(platform_rs.contains("\"--fidelity-trace-inputs\" =>"));
        assert!(platform_rs.contains("\"--fidelity-trace-inputs-file\" =>"));
        assert!(platform_rs.contains("CleanCliError::RomReportPathCannotBeFlag"));
        assert!(platform_rs.contains("CleanCliError::TooManyRomReportArgs"));
        assert!(platform_rs.contains("CleanCliError::MissingVerifyRomsPath"));
        assert!(platform_rs.contains("CleanCliError::TooManyVerifyRomsArgs"));
        assert!(platform_rs.contains("CleanCliError::InvalidFidelityTraceFrameCount"));
        assert!(platform_rs.contains("CleanCliError::NonPositiveFidelityTraceFrameCount"));
        assert!(platform_rs.contains("CleanCliError::TooManyFidelityTraceArgs"));
        assert!(platform_rs.contains("CleanCliError::FidelityTraceInputsMissingScript"));
        assert!(platform_rs.contains("CleanCliError::FidelityTraceInputsExtraArgs"));
        assert!(platform_rs.contains("CleanCliError::FidelityTraceInputsFileMissingPath"));
        assert!(platform_rs.contains("CleanCliError::FidelityTraceInputsFileExtraArgs"));
        assert!(platform_rs.contains("CleanCliError::FidelityCheckTraceMissingPaths"));
        assert!(platform_rs.contains("CleanCliError::FidelityCheckTraceExtraArgs"));
        assert!(platform_rs.contains("CleanCliError::FidelityCheckTraceDirMissingPath"));
        assert!(platform_rs.contains("CleanCliError::FidelityCheckTraceDirExtraArgs"));
        assert!(platform_rs.contains("CleanCliError::FidelityCheckReferenceTraceDirMissingPath"));
        assert!(platform_rs.contains("CleanCliError::FidelityCheckReferenceTraceDirExtraArgs"));
        assert!(platform_rs.contains("CleanCliError::FidelityListScenariosExtraArgs"));
        assert!(platform_rs.contains("CleanCliError::FidelityWriteScenarioInputsMissingPath"));
        assert!(platform_rs.contains("CleanCliError::FidelityWriteScenarioInputsExtraArgs"));
        assert!(platform_rs.contains("CleanCliError::LiveOptionsWithCommand"));
        assert!(platform_rs.contains("\"--verify-roms\""));
        assert!(platform_rs.contains("\"--fidelity-trace\""));
        assert!(platform_rs.contains("\"--fidelity-trace-inputs\""));
        assert!(platform_rs.contains("\"--fidelity-trace-inputs-file\""));
        assert!(platform_rs.contains("\"--fidelity-check-trace\""));
        assert!(platform_rs.contains("\"--fidelity-check-trace-dir\""));
        assert!(platform_rs.contains("\"--fidelity-check-reference-trace-dir\""));
        assert!(platform_rs.contains("\"--fidelity-list-scenarios\""));
        assert!(platform_rs.contains("\"--fidelity-write-scenario-inputs\""));
        assert!(!platform_rs.contains("VerifyRoms,"));
        assert!(!platform_rs.contains("HistoricalCliCommand"));
        assert!(!platform_rs.contains("CliClassification::HistoricalCommand"));
        assert!(!platform_rs.contains("historical_cli_command"));
        assert!(!platform_rs.contains("Some(HistoricalCliCommand::Trace)"));
        assert!(!platform_rs.contains("Some(HistoricalCliCommand::TraceInputs)"));
        assert!(!platform_rs.contains("Some(HistoricalCliCommand::TraceInputsFile)"));
        assert!(!platform_rs.contains("Some(HistoricalCliCommand::CompareTrace)"));
        assert!(!platform_rs.contains("Some(HistoricalCliCommand::FixtureDirectory)"));
        assert!(!platform_rs.contains("FidelityWriteScenarioInputs,"));
        assert!(platform_rs.contains("CliClassification::Runtime(config)"));
        assert!(platform_rs.contains("CliClassification::Help"));
        assert!(platform_rs.contains("CliClassification::Error(error)"));
        assert!(platform_rs.contains("enum CleanCliError"));
        assert!(platform_rs.contains("CleanCliError::MissingInputProfile"));
        assert!(platform_rs.contains("CleanCliError::UnknownInputProfile"));
        assert!(platform_rs.contains("CleanCliError::MissingCmosPath"));
        assert!(platform_rs.contains("CleanCliError::RemovedRendererSelection"));
        assert!(platform_rs.contains("CleanCliError::UnknownArgument"));
        assert!(platform_rs.contains("RuntimeConfig::default()"));
        assert!(platform_rs.contains("config.mode = RunMode::Smoke"));
        assert!(platform_rs.contains("\"--help\" | \"-h\" => ArgClassification::Help"));
        assert!(platform_rs.contains("ArgClassification::Error"));
        assert!(platform_rs.contains("\"--live-smoke\""));
        assert!(platform_rs.contains("RuntimeConfig::smoke()"));
        assert!(!platform_rs.contains(&accepted_runtime_call));
        assert!(!platform_rs.contains("crate::compatibility::"));
        assert!(!platform_rs.contains("CompatibilityFallback"));
        assert!(!platform_rs.contains("CompatibilityCliArg"));
        assert!(!platform_rs.contains("crate::runtime::run_cli()"));
        assert!(!platform_rs.contains(&app_runtime_call));

        let runtime_rs = include_str!("runtime.rs");
        assert!(runtime_rs.contains("crate::wgpu_presenter::run_wgpu_live("));
        assert!(runtime_rs.contains("crate::wgpu_presenter::run_wgpu_live_smoke"));
        assert!(runtime_rs.contains("RuntimeCommand::Help"));
        assert!(runtime_rs.contains("RuntimeCommand::RomReport { path }"));
        assert!(runtime_rs.contains("RuntimeCommand::VerifyRoms { path }"));
        assert!(runtime_rs.contains("RuntimeCommand::FidelityTrace { frame_count }"));
        assert!(runtime_rs.contains("RuntimeCommand::FidelityTraceInputs { script }"));
        assert!(runtime_rs.contains("RuntimeCommand::FidelityTraceInputsFile { path }"));
        assert!(runtime_rs.contains("RuntimeCommand::FidelityTraceCheck {"));
        assert!(runtime_rs.contains("RuntimeCommand::FidelityTraceFixtureDirectory { path }"));
        assert!(
            runtime_rs.contains("RuntimeCommand::FidelityReferenceTraceFixtureDirectory { path }")
        );
        assert!(runtime_rs.contains("RuntimeCommand::FidelityScenarioList"));
        assert!(runtime_rs.contains("RuntimeCommand::FidelityScenarioInputWriter"));
        assert!(runtime_rs.contains("pub(crate) fn help_text()"));
        assert!(runtime_rs.contains("pub(crate) fn run_rom_report"));
        assert!(runtime_rs.contains("pub(crate) fn run_verify_roms"));
        assert!(runtime_rs.contains("pub(crate) fn run_fidelity_trace"));
        assert!(runtime_rs.contains("pub(crate) fn run_fidelity_trace_inputs"));
        assert!(runtime_rs.contains("pub(crate) fn run_fidelity_trace_inputs_file"));
        assert!(runtime_rs.contains("pub(crate) fn run_fidelity_trace_check"));
        assert!(runtime_rs.contains("pub(crate) fn run_fidelity_trace_check_dir"));
        assert!(runtime_rs.contains("pub(crate) fn run_fidelity_reference_trace_check_dir"));
        assert!(runtime_rs.contains("pub(crate) fn run_fidelity_scenario_list"));
        assert!(runtime_rs.contains("pub(crate) fn run_fidelity_scenario_input_writer"));
        assert!(runtime_rs.contains("crate::rom_report::run(path.as_deref())"));
        assert!(runtime_rs.contains("crate::rom_report::run_verify(&path)"));
        assert!(runtime_rs.contains("crate::fidelity_traces::run_trace(frame_count)"));
        assert!(runtime_rs.contains("crate::fidelity_traces::run_trace_inputs(&script)"));
        assert!(runtime_rs.contains("crate::fidelity_traces::run_trace_inputs_file(&path)"));
        assert!(
            runtime_rs
                .contains("crate::fidelity_traces::run_check_trace(&inputs_path, &expected_path)")
        );
        assert!(runtime_rs.contains("crate::fidelity_traces::run_check_trace_dir(&path)"));
        assert!(
            runtime_rs.contains("crate::fidelity_traces::run_check_reference_trace_dir(&path)")
        );
        assert!(runtime_rs.contains("crate::fidelity_scenarios::run_list()"));
        assert!(runtime_rs.contains("crate::fidelity_scenarios::run_write_inputs(&path)"));
        assert!(!runtime_rs.contains("RuntimeCommand::AcceptedCli"));
        assert!(!runtime_rs.contains("pub(crate) fn run_cli"));
        assert!(!runtime_rs.contains("crate::accepted_behavior::run_runtime()"));
        assert!(!runtime_rs.contains("crate::rom::"));
        assert!(!runtime_rs.contains(&accepted_runtime_call));
        assert!(!runtime_rs.contains(&app_runtime_call));

        let lib_rs = include_str!("lib.rs");
        let public_rom_report_module = format!("pub mod {};", "rom_report");
        let public_fidelity_scenarios_module = format!("pub mod {};", "fidelity_scenarios");
        let public_fidelity_traces_module = format!("pub mod {};", "fidelity_traces");
        assert!(lib_rs.contains("mod rom_report;"));
        assert!(!lib_rs.contains(&public_rom_report_module));
        assert!(lib_rs.contains("mod fidelity_scenarios;"));
        assert!(!lib_rs.contains(&public_fidelity_scenarios_module));
        assert!(lib_rs.contains("mod fidelity_traces;"));
        assert!(!lib_rs.contains(&public_fidelity_traces_module));

        let rom_report_rs = include_str!("rom_report.rs");
        assert!(rom_report_rs.contains("pub(crate) fn run("));
        assert!(rom_report_rs.contains("pub(crate) fn run_verify("));
        assert!(rom_report_rs.contains("fn listing_text()"));
        assert!(rom_report_rs.contains("fn verification_text("));
        assert!(rom_report_rs.contains("fn report_text(report: &crate::rom::RomReport)"));
        assert!(rom_report_rs.contains("crate::rom::expected_roms()"));
        assert!(rom_report_rs.contains("crate::rom::scan_dir(path)"));
        assert!(rom_report_rs.contains("crate::rom::load_verified_dir(path)"));
        assert!(rom_report_rs.contains("crate::rom::RedLabelRomImages::from_verified_rom_set"));

        let fidelity_scenarios_rs = include_str!("fidelity_scenarios.rs");
        assert!(fidelity_scenarios_rs.contains("pub(crate) fn run_list("));
        assert!(fidelity_scenarios_rs.contains("pub(crate) fn run_write_inputs("));
        assert!(fidelity_scenarios_rs.contains("fn listing_text()"));
        assert!(fidelity_scenarios_rs.contains("fn write_inputs_text(path: &Path)"));
        assert!(fidelity_scenarios_rs.contains("crate::legacy_fidelity::trace_scenarios()"));
        assert!(
            fidelity_scenarios_rs.contains("crate::legacy_fidelity::expanded_trace_input_text")
        );

        let fidelity_traces_rs = include_str!("fidelity_traces.rs");
        assert!(fidelity_traces_rs.contains("pub(crate) fn run_trace("));
        assert!(fidelity_traces_rs.contains("pub(crate) fn run_trace_inputs("));
        assert!(fidelity_traces_rs.contains("pub(crate) fn run_trace_inputs_file("));
        assert!(fidelity_traces_rs.contains("pub(crate) fn run_check_trace("));
        assert!(fidelity_traces_rs.contains("pub(crate) fn run_check_trace_dir("));
        assert!(fidelity_traces_rs.contains("pub(crate) fn run_check_reference_trace_dir("));
        assert!(fidelity_traces_rs.contains("fn trace_text("));
        assert!(fidelity_traces_rs.contains("fn trace_input_text("));
        assert!(fidelity_traces_rs.contains("fn trace_input_file_text("));
        assert!(fidelity_traces_rs.contains("fn check_trace_text("));
        assert!(fidelity_traces_rs.contains("fn check_trace_dir_text("));
        assert!(fidelity_traces_rs.contains("fn check_reference_trace_dir_text("));
        assert!(fidelity_traces_rs.contains("fn check_reference_trace_required_cells("));
        assert!(fidelity_traces_rs.contains("fn check_reference_trace_evidence("));
        assert!(fidelity_traces_rs.contains("fn parse_trace_requirements("));
        assert!(fidelity_traces_rs.contains("fn trace_fixture_pairs("));
        assert!(fidelity_traces_rs.contains("fn check_trace_fixtures("));
        assert!(fidelity_traces_rs.contains("struct TraceFixture"));
        assert!(fidelity_traces_rs.contains("struct TraceRequirement"));
        assert!(fidelity_traces_rs.contains("fs::read_to_string(path)"));
        assert!(fidelity_traces_rs.contains("fs::read_to_string(expected_path)"));
        assert!(
            fidelity_traces_rs
                .contains("include_str!(\"../assets/red-label/trace-requirements.tsv\")")
        );
        assert!(fidelity_traces_rs.contains("crate::legacy_fidelity::expanded_trace_input_text"));
        assert!(fidelity_traces_rs.contains("crate::legacy_fidelity::parse_trace_input_script"));
        assert!(fidelity_traces_rs.contains("crate::legacy_fidelity::trace_text_for_inputs"));
        assert!(fidelity_traces_rs.contains("crate::legacy_fidelity::compare_trace_text"));
        assert!(fidelity_traces_rs.contains("crate::legacy_fidelity::trace_scenarios()"));
        assert!(fidelity_traces_rs.contains("crate::legacy_fidelity::trace_header()"));

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
        assert!(!accepted_rs.contains("run_runtime"));
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

        let fidelity_rs = include_str!("fidelity.rs");
        for forbidden in [
            "crate::accepted::",
            "AcceptedFrame",
            "AcceptedGameplayMachine",
            "adapt_accepted_",
            "crate::input::",
            "crate::machine::",
            "crate::machine_state::",
            "crate::red_label::",
            "crate::video::",
        ] {
            assert!(
                !fidelity_rs.contains(forbidden),
                "clean fidelity contracts must avoid legacy module import {forbidden}"
            );
        }

        let audio_rs = include_str!("audio.rs");
        for forbidden in ["crate::machine_state::", "FrameOutput"] {
            assert!(
                !audio_rs.contains(forbidden),
                "clean audio runtime must consume GameFrame/SoundEvent contracts instead of {forbidden}"
            );
        }

        let game_rs = include_str!("game.rs");
        for forbidden in [
            "crate::accepted::",
            "crate::machine::",
            "crate::machine_state::",
            "crate::red_label::",
            "FrameOutput",
            "from_accepted_command",
            "accepted_command",
            "UnmappedAcceptedCommand",
            "red_label",
            "RED_LABEL",
            "source routine",
            "assembler",
            "memory",
        ] {
            assert!(
                !game_rs.contains(forbidden),
                "clean gameplay contracts must not expose legacy implementation terminology {forbidden}"
            );
        }
    }

    #[test]
    fn clean_module_sources_keep_legacy_access_quarantined() {
        let clean_sources = [
            ("src/accepted.rs", include_str!("accepted.rs")),
            ("src/audio.rs", include_str!("audio.rs")),
            ("src/fidelity.rs", include_str!("fidelity.rs")),
            ("src/game.rs", include_str!("game.rs")),
            (
                "src/fidelity_scenarios.rs",
                include_str!("fidelity_scenarios.rs"),
            ),
            ("src/fidelity_traces.rs", include_str!("fidelity_traces.rs")),
            ("src/main.rs", include_str!("main.rs")),
            ("src/oracle.rs", include_str!("oracle.rs")),
            ("src/platform.rs", include_str!("platform.rs")),
            ("src/renderer.rs", include_str!("renderer.rs")),
            ("src/rom_report.rs", include_str!("rom_report.rs")),
            ("src/runtime.rs", include_str!("runtime.rs")),
            ("src/systems.rs", include_str!("systems.rs")),
        ];
        let low_level_legacy_imports = [
            "crate::app::",
            "crate::assets::",
            "crate::board::",
            "crate::cmos_storage::",
            "crate::input::",
            "crate::legacy_fidelity::",
            "crate::live::",
            "crate::machine::",
            "crate::machine_process::",
            "crate::machine_state::",
            "crate::pia::",
            "crate::red_label::",
            "crate::red_label_memory::",
            "crate::red_label_message::",
            "crate::red_label_wave::",
            "crate::rom::",
            "crate::sound::",
            "crate::video::",
            "crate::wgpu_presenter::",
        ];

        for (path, source) in clean_sources {
            for forbidden in low_level_legacy_imports {
                if path == "src/runtime.rs"
                    && matches!(forbidden, "crate::input::" | "crate::wgpu_presenter::")
                {
                    continue;
                }
                if path == "src/rom_report.rs" && forbidden == "crate::rom::" {
                    continue;
                }
                if matches!(path, "src/fidelity_scenarios.rs" | "src/fidelity_traces.rs")
                    && forbidden == "crate::legacy_fidelity::"
                {
                    continue;
                }

                assert!(
                    !source.contains(forbidden),
                    "{path} must not import legacy root module {forbidden}"
                );
            }

            if path != "src/accepted.rs" {
                assert!(
                    !source.contains("crate::accepted_behavior::"),
                    "{path} must not bypass the accepted runtime adapter"
                );
            }

            if !matches!(path, "src/accepted.rs" | "src/oracle.rs") {
                assert!(
                    !source.contains("crate::accepted::"),
                    "{path} must not depend on the temporary accepted facade"
                );
            }

            for forbidden in [
                "red_label",
                "RED_LABEL",
                "source routine",
                "assembler",
                "memory",
                "FrameOutput",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "{path} must not expose legacy implementation terminology {forbidden}"
                );
            }
        }
    }

    #[test]
    fn legacy_modules_are_crate_private_at_root() {
        let lib_rs = include_str!("lib.rs");
        let legacy_modules = [
            ("accepted_behavior", "accepted_behavior"),
            ("app", "app"),
            ("assets", "assets"),
            ("board", "board"),
            ("cmos_storage", "cmos_storage"),
            ("legacy_fidelity", "fidelity"),
            ("input", "input"),
            ("live", "live"),
            ("machine", "machine"),
            ("machine_process", "machine_process"),
            ("machine_state", "machine_state"),
            ("pia", "pia"),
            ("red_label", "red_label"),
            ("red_label_memory", "red_label_memory"),
            ("red_label_message", "red_label_message"),
            ("red_label_wave", "red_label_wave"),
            ("rom", "rom"),
            ("sound", "sound"),
            ("video", "video"),
            ("wgpu_presenter", "wgpu_presenter"),
        ];

        for (module, path) in legacy_modules {
            let marker = format!("#[path = \"../src_legacy/{path}.rs\"]");
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
