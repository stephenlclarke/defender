//! Clean-slate Defender red-label reimplementation.
//!
//! The old prototype implementation lives under `oldsrc/`. New code is
//! organized around a deterministic arcade core that can be checked against
//! the original red-label ROM behavior, with presentation and compatibility
//! features layered around it.

pub mod app;
pub mod assets;
pub mod board;
pub mod cmos_storage;
pub mod fidelity;
pub mod input;
pub mod kitty;
pub mod live;
pub mod machine;
pub(crate) mod machine_state;
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
