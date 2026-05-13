//! Clean-slate Defender red-label reimplementation.
//!
//! The old prototype implementation lives under `oldsrc/`. New code is
//! organized around a deterministic arcade core that can be checked against
//! the original red-label ROM behavior, with presentation and compatibility
//! features layered around it.

pub mod app;
pub mod assets;
pub mod audio;
pub mod board;
pub mod cmos_storage;
pub mod fidelity;
pub mod input;
pub mod kitty;
pub mod live;
pub mod machine;
pub mod machine_process;
pub mod machine_state;
pub mod pia;
pub mod presentation;
pub mod red_label;
pub mod red_label_memory;
pub mod red_label_message;
pub(crate) mod red_label_trace_samples;
pub mod red_label_wave;
pub mod rom;
pub mod sound;
pub mod terminal;
pub mod video;
pub mod wgpu_presenter;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod public_api_tests {
    #[test]
    fn machine_state_contracts_have_direct_and_compatibility_paths() {
        let direct_phase = crate::machine_state::GamePhase::Attract;
        let compatibility_phase: crate::machine::GamePhase = direct_phase;
        let direct_phase_again: crate::machine_state::GamePhase = compatibility_phase;
        assert_eq!(direct_phase_again, crate::machine_state::GamePhase::Attract);

        let direct = crate::machine_state::CompatibilityState::default();
        let compatibility: crate::machine::CompatibilityState = direct;
        assert!(!compatibility.xyzzy_active);
    }

    #[test]
    fn machine_process_contracts_have_direct_and_compatibility_paths() {
        let direct =
            crate::machine_process::RedLabelScheduledProcess::from_source_disp(0xA05F, 0xC123);
        let compatibility: crate::machine::RedLabelScheduledProcess = direct;
        let direct_again: crate::machine_process::RedLabelScheduledProcess = compatibility;

        assert_eq!(direct_again.process_address, 0xA05F);
        assert_eq!(direct_again.routine_address, 0xC123);
    }
}
