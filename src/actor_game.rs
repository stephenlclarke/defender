//! Actor-oriented Defender rewrite prototype.
//!
//! This module models the game as driver-owned actor threads: the driver
//! prompts every asset once per step, gathers commands, resolves world rules in
//! a stable order, and publishes a step description.

use crate::{
    MessageId, ScreenAddress, SoundCommand, SpriteFrameIndex, TimelineStep,
    game::{
        ATTRACT_SCORING_SEQUENCE_START_STEP as GAME_ATTRACT_SCORING_SEQUENCE_START_STEP,
        ActorDebugMotion, AttractPresentationSnapshot, BaiterDebugStateSnapshot,
        BomberDebugStateSnapshot, Direction as CleanDirection, EnemyKind as CleanEnemyKind,
        EnemyProjectileKind, EnemyProjectileSnapshot as CleanEnemyProjectileSnapshot,
        EnemyReserveSnapshot, EnemySnapshot as CleanEnemySnapshot,
        ExplosionKind as CleanExplosionKind, ExplosionSnapshot as CleanExplosionSnapshot,
        GameEvent, GameEvents, GameInput as CleanGameInput, GameOverSnapshot, GamePhase,
        GameRngSnapshot, GameState, GameStepSnapshot, HIGH_SCORE_TABLE_ENTRIES,
        HighScoreEntrySnapshot, HighScoreTableEntrySnapshot, HighScoreTablesSnapshot,
        HumanSnapshot as CleanHumanSnapshot, LanderDebugStateSnapshot, MutantDebugStateSnapshot,
        PlayerSnapshot, PlayerStockSnapshot, PodDebugStateSnapshot,
        ProjectileSnapshot as CleanProjectileSnapshot, ScorePopupKind as CleanScorePopupKind,
        ScorePopupSnapshot as CleanScorePopupSnapshot, ScoreSnapshot, SoundEvent,
        SpriteAssetImageSpec, SwarmerDebugStateSnapshot, TERRAIN_BLOW_COMPLETE_STEP,
        TERRAIN_BLOW_EXPLOSION_BIRTHS, TERRAIN_BLOW_FLASH_COLOR_BYTES,
        TERRAIN_BLOW_OVERLOAD_COUNTER, TERRAIN_BLOW_START_SOUND_STEPS,
        TERRAIN_EXPLOSION_LIFETIME_STEPS, TerrainBlowSnapshot, TerrainBlowStage, TerrainSegment,
        VISUAL_STATE, WaveProfileSnapshot, WorldSnapshot, WorldVector,
        appearance_growth_size_for_age, explosion_growth_size_for_age, explosion_render_scale,
        push_appearance_cloud_pixels, push_background_terrain_sprites, push_explosion_cloud_pixels,
        push_scanner_radar_sprites, terrain_blow_flash_tint, terrain_explosion_growth_size_for_age,
        wave_tuning_landscape_tint,
    },
    renderer::{
        Color, RenderLayer, RenderScene, SceneSprite, SpriteId, SurfaceSize,
        attract_defender_appearance_pixels, attract_williams_logo_operation_pixel_counts,
        attract_williams_logo_pixel_path, push_controlled_message_sprites,
        push_message_text_bytes_sprites, screen_position_from_cell,
        screen_position_from_cell_with_offset,
    },
    systems::{
        HighScoreEntrySystem, HighScoreInitialsState, PlayerStock, ScoreSystem, ScreenPosition,
        ScreenVelocity,
    },
};

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt,
    str::FromStr,
    sync::{
        OnceLock,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

#[path = "actor_game_actor_helpers.rs"]
mod actor_helpers;
#[path = "actor_game_actor_state.rs"]
mod actor_state;
#[path = "actor_game_actors_effects.rs"]
mod actors_effects;
#[path = "actor_game_actors_hostile_motion.rs"]
mod actors_hostile_motion;
#[path = "actor_game_actors_hostile_projectiles.rs"]
mod actors_hostile_projectiles;
#[path = "actor_game_actors_hostiles.rs"]
mod actors_hostiles;
#[path = "actor_game_actors_lander.rs"]
mod actors_lander;
#[path = "actor_game_actors_mutant.rs"]
mod actors_mutant;
#[path = "actor_game_actors_mutant_dive.rs"]
mod actors_mutant_dive;
#[path = "actor_game_actors_player.rs"]
mod actors_player;
#[path = "actor_game_attract_scoring.rs"]
mod attract_scoring;
#[path = "actor_game_attract_scripts.rs"]
mod attract_scripts;
#[path = "actor_game_behavior_scripts.rs"]
mod behavior_scripts;
#[path = "actor_game_core.rs"]
mod core;
#[path = "actor_game_driver.rs"]
mod driver;
#[path = "actor_game_fixed_point_motion.rs"]
mod fixed_point_motion;
#[path = "actor_game_mutant_dive_patterns.rs"]
mod mutant_dive_patterns;
#[path = "actor_game_protocol.rs"]
mod protocol;
#[path = "actor_game_render_bridge.rs"]
mod render_bridge;
#[path = "actor_game_runtime.rs"]
mod runtime;
#[path = "actor_game_spawn_blueprints.rs"]
mod spawn_blueprints;
#[path = "actor_game_spawn_patterns.rs"]
mod spawn_patterns;
#[cfg(test)]
#[path = "actor_game_tests.rs"]
mod tests;
#[path = "actor_game_wave_profile.rs"]
mod wave_profile;
#[path = "actor_game_wave_scripts.rs"]
mod wave_scripts;

#[allow(unused_imports)]
use self::{
    actor_helpers::*, actor_state::*, actors_effects::*, actors_hostile_motion::*,
    actors_hostile_projectiles::*, actors_hostiles::*, actors_lander::*, actors_mutant::*,
    actors_mutant_dive::*, actors_player::*, attract_scoring::*, behavior_scripts::*,
    fixed_point_motion::*, mutant_dive_patterns::*, protocol::*, spawn_blueprints::*,
    spawn_patterns::*, wave_profile::*,
};

#[allow(unused_imports)]
pub(crate) use actor_helpers::*;
pub use actor_state::*;
#[allow(unused_imports)]
pub(crate) use actors_effects::*;
#[allow(unused_imports)]
pub(crate) use actors_hostile_motion::*;
#[allow(unused_imports)]
pub(crate) use actors_hostile_projectiles::*;
#[allow(unused_imports)]
pub(crate) use actors_hostiles::*;
#[allow(unused_imports)]
pub(crate) use actors_lander::*;
#[allow(unused_imports)]
pub(crate) use actors_mutant::*;
#[allow(unused_imports)]
pub(crate) use actors_mutant_dive::*;
#[allow(unused_imports)]
pub(crate) use actors_player::*;
#[allow(unused_imports)]
pub(crate) use attract_scoring::*;
pub use attract_scripts::*;
pub use core::*;
pub use driver::*;
#[allow(unused_imports)]
pub(crate) use fixed_point_motion::*;
#[allow(unused_imports)]
pub(crate) use mutant_dive_patterns::*;
pub use protocol::*;
pub use render_bridge::*;
pub use runtime::*;
#[allow(unused_imports)]
pub(crate) use spawn_blueprints::*;
#[allow(unused_imports)]
pub(crate) use spawn_patterns::*;
pub use wave_scripts::*;
