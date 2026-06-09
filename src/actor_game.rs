//! Actor-oriented Defender rewrite prototype.
//!
//! This module is intentionally independent from the current MAME-shaped
//! `Game` implementation. It models the game as driver-owned actor threads:
//! the driver prompts every asset once per step, gathers commands, resolves
//! world rules in a stable order, and publishes a step description.

include!("actor_game_core.rs");
include!("actor_game_behavior_scripts.rs");
include!("actor_game_arcade_state.rs");
include!("actor_game_spawn_patterns.rs");
include!("actor_game_wave_profile.rs");
include!("actor_game_wave_scripts.rs");
include!("actor_game_attract_scripts.rs");
include!("actor_game_protocol.rs");
include!("actor_game_render_bridge.rs");
include!("actor_game_attract_scoring.rs");
include!("actor_game_runtime.rs");
include!("actor_game_driver.rs");
include!("actor_game_actor_helpers.rs");
include!("actor_game_actors_hostile_motion.rs");
include!("actor_game_actors_hostile_projectiles.rs");
include!("actor_game_actors_player.rs");
include!("actor_game_actors_lander.rs");
include!("actor_game_actors_mutant.rs");
include!("actor_game_actors_hostiles.rs");
include!("actor_game_actors_effects.rs");
include!("actor_game_tests.rs");
