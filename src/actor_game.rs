//! Actor-oriented Defender rewrite prototype.
//!
//! This module is intentionally independent from the current MAME-shaped
//! `Game` implementation. It models the game as driver-owned actor threads:
//! the driver prompts every asset once per step, gathers commands, resolves
//! world rules in a stable order, and publishes a step description.

include!("actor_game_core.rs");
include!("actor_game_behavior_sources.rs");
include!("actor_game_wave_scripts.rs");
include!("actor_game_attract_scripts.rs");
include!("actor_game_protocol.rs");
include!("actor_game_render_bridge.rs");
include!("actor_game_runtime.rs");
include!("actor_game_driver.rs");
include!("actor_game_actor_helpers.rs");
include!("actor_game_actors_player.rs");
include!("actor_game_actors_lander_mutant.rs");
include!("actor_game_actors_hostiles.rs");
include!("actor_game_actors_effects.rs");
include!("actor_game_tests.rs");
