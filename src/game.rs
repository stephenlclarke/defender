//! Domain-facing gameplay contracts.

#[path = "game_core.rs"]
mod core;
#[path = "game_input.rs"]
mod input;
#[path = "game_render_projection.rs"]
mod render_projection;
#[path = "game_state.rs"]
mod state;
#[path = "game_visual_evidence.rs"]
mod visual_evidence;
#[path = "game_world_snapshots.rs"]
mod world_snapshots;

pub use core::*;
pub use input::*;
pub(crate) use render_projection::*;
pub use state::*;
pub use visual_evidence::*;
pub use world_snapshots::*;
