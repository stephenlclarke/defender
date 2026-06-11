//! Wgpu-oriented scene contracts.

#[path = "renderer_atlas.rs"]
mod atlas;
#[path = "renderer_core.rs"]
mod core;
#[path = "renderer_resources.rs"]
mod resources;
#[cfg(test)]
#[path = "renderer_tests.rs"]
mod tests;
#[path = "renderer_wgpu_plan.rs"]
mod wgpu_plan;

pub use atlas::*;
pub use core::*;
pub use resources::*;
pub use wgpu_plan::*;
