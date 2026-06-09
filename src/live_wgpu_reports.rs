#[cfg(all(not(test), not(coverage)))]
use std::{
    collections::BTreeSet,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use std::{fs, path::Path};

use anyhow::Context;
#[cfg(test)]
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
#[cfg(all(not(test), not(coverage)))]
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    window::{Window, WindowId},
};

#[cfg(any(test, all(not(test), not(coverage))))]
use crate::game::GameInput;
use crate::{
    actor_game::{
        ActorDriverScripts, ActorFrame, ActorId, ActorKind, ActorRuntimeAdapter, ExplosionKind,
        GameCommand, GameInput as ActorGameInput, HostileMovementMode, LanderBehaviorMode, Phase,
        SoundCue, SpawnRequest, SpriteKey, VisualEffect, XyzzyController, XyzzyMode,
    },
    actor_smoke::ActorSmokeReport,
    arcade_assets::{MessageId, message_text},
    audio::LiveAudioMode,
    renderer::SpriteId,
};
#[cfg(all(not(test), not(coverage)))]
use crate::{
    audio::LiveAudioRuntime,
    game::GameFrame,
    renderer::{
        GpuRendererSettings, NativeSceneRenderer, SceneDrawPlan, SpriteBindGroupRole,
        SpriteBufferRole, SpriteBufferUpload, SpriteRenderPassEncoderCommand, SurfaceSize,
    },
    systems::{FixedStepAccumulator, FrameRate},
};

#[cfg(all(not(test), not(coverage)))]
const INITIAL_WINDOW_WIDTH: u32 = 1_024;
#[cfg(all(not(test), not(coverage)))]
const INITIAL_WINDOW_HEIGHT: u32 = 768;
#[cfg(all(not(test), not(coverage)))]
const MAX_STEPS_PER_TICK: u32 = 5;
const ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT: usize = 512;
const ACTOR_SCRIPT_CHECK_ATTRACT_CYCLE_STEP_LIMIT: u64 = 4096;
const ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT: usize = 4096;
const ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_STEP_LIMIT: usize = 512; // original: ACTOR_SCRIPT_CHECK_SOURCE_PROJECTILE_STEP_LIMIT
const ACTOR_SCRIPT_CHECK_RESERVE_ACTIVATION_BATCH_LIMIT: usize = 8;
const ACTOR_SCRIPT_CHECK_ACTOR_SAMPLE_LIMIT: usize = 8; // original: ACTOR_SCRIPT_CHECK_SOURCE_ACTOR_SAMPLE_LIMIT
const ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_LIMIT: usize = 8; // original: ACTOR_SCRIPT_CHECK_SOURCE_PROJECTILE_SAMPLE_LIMIT

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiveInputProfile {
    Planetoid,
    Cabinet,
    Test,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LiveSmokeReport {
    pub(crate) render_path: &'static str,
    pub(crate) legacy_presenter_used: bool,
    pub(crate) window_created: bool,
    pub(crate) rendered_frames: u32,
    pub(crate) first_frame_size: Option<(u32, u32)>,
    pub(crate) distinct_frame_signatures: usize,
    pub(crate) saw_non_blank_frame: bool,
    pub(crate) saw_attract: bool,
    pub(crate) saw_credit: bool,
    pub(crate) saw_playing: bool,
    pub(crate) attract_visual_frames: u32,
    pub(crate) credit_visual_frames: u32,
    pub(crate) playing_visual_frames: u32,
    pub(crate) attract_distinct_frame_signatures: usize,
    pub(crate) credit_distinct_frame_signatures: usize,
    pub(crate) playing_distinct_frame_signatures: usize,
    pub(crate) clean_game_frames: u32,
    pub(crate) actor_frames: u32,
    pub(crate) sprite_frames: u32,
    pub(crate) sprite_instances: usize,
    pub(crate) sprite_draw_commands: usize,
    pub(crate) temporary_raster_frames: u32,
    pub(crate) temporary_raster_commands: usize,
    pub(crate) offscreen_wgpu_frames: u32,
    pub(crate) offscreen_non_blank_frames: u32,
    pub(crate) offscreen_distinct_frame_signatures: usize,
    pub(crate) offscreen_first_frame_signature: Option<u64>,
    pub(crate) offscreen_last_frame_signature: Option<u64>,
    pub(crate) injected_inputs: Vec<String>,
    pub(crate) clean_exit: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckActorSample {
    pub(crate) kind: String,
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) x_subpixel: u8,
    pub(crate) y_subpixel: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckEnemyProjectileSample {
    pub(crate) kind: String,
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) x_subpixel: u8,
    pub(crate) y_subpixel: u8,
    pub(crate) x_velocity_word: u16,
    pub(crate) y_velocity_word: u16,
    pub(crate) lifetime_ticks: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckProjectileSpawnSample {
    pub(crate) kind: String,
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) velocity_dx: i16,
    pub(crate) velocity_dy: i16,
    pub(crate) x_subpixel: Option<u8>,
    pub(crate) y_subpixel: Option<u8>,
    pub(crate) x_velocity_word: Option<u16>,
    pub(crate) y_velocity_word: Option<u16>,
    pub(crate) lifetime_ticks: Option<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckPlayerLaserSample {
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) velocity_dx: i16,
    pub(crate) velocity_dy: i16,
    pub(crate) direction: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckExplosionSample {
    pub(crate) kind: String,
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) explosion_anchor_x: Option<i16>,
    pub(crate) explosion_anchor_y: Option<i16>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckPlayingSummary {
    pub(crate) wave: u16,
    pub(crate) wave_size: u8,
    pub(crate) enemy_landers: u8,
    pub(crate) enemy_bombers: u8,
    pub(crate) enemy_pods: u8,
    pub(crate) enemy_mutants: u8,
    pub(crate) enemy_swarmers: u8,
    pub(crate) world_enemies: usize,
    pub(crate) world_humans: usize,
    pub(crate) reserve_landers: u8,
    pub(crate) reserve_bombers: u8,
    pub(crate) reserve_pods: u8,
    pub(crate) reserve_mutants: u8,
    pub(crate) reserve_swarmers: u8,
    pub(crate) world_scroll_left: u16,
    pub(crate) arcade_rng_seed: Option<u8>,
    pub(crate) arcade_rng_hseed: Option<u8>,
    pub(crate) arcade_rng_lseed: Option<u8>,
    pub(crate) player_takes_enemy_collision_damage: bool,
    pub(crate) player_laser_cooldown_steps: u8,
    pub(crate) lander_mode: String,
    pub(crate) lander_seek_speed: i16,
    pub(crate) lander_drift_speed: i16,
    pub(crate) lander_fire_period_steps: u64,
    pub(crate) mutant_mode: String,
    pub(crate) bomber_mode: String,
    pub(crate) pod_mode: String,
    pub(crate) swarmer_mode: String,
    pub(crate) baiter_mode: String,
    pub(crate) swarmer_fire_period_steps: u64,
    pub(crate) baiter_fire_period_steps: u64,
    pub(crate) actor_samples: Vec<ActorScriptCheckActorSample>,
    pub(crate) enemy_projectile_samples: Vec<ActorScriptCheckEnemyProjectileSample>,
    pub(crate) sound_commands: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckSpawnedActorSample {
    pub(crate) kind: String,
    pub(crate) x: i16,
    pub(crate) y: i16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckSpawnedCounts {
    pub(crate) landers: usize,
    pub(crate) bombers: usize,
    pub(crate) pods: usize,
    pub(crate) mutants: usize,
    pub(crate) swarmers: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckReserveActivationSummary {
    pub(crate) assist_steps: u32,
    pub(crate) spawned_counts: ActorScriptCheckSpawnedCounts,
    pub(crate) spawned_samples: Vec<ActorScriptCheckSpawnedActorSample>,
    pub(crate) playing: ActorScriptCheckPlayingSummary,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckFirstEnemyProjectileSummary {
    pub(crate) sample_steps: u32,
    pub(crate) samples: Vec<ActorScriptCheckEnemyProjectileSample>,
    pub(crate) sound_commands: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckHostileProjectileSample {
    pub(crate) kind: String,
    pub(crate) sample_steps: u32,
    pub(crate) samples: Vec<ActorScriptCheckProjectileSpawnSample>,
    pub(crate) sound_commands: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckFirstPlayerLaserSummary {
    pub(crate) sample_steps: u32,
    pub(crate) samples: Vec<ActorScriptCheckPlayerLaserSample>,
    pub(crate) sound_commands: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckFirstPlayerLaserHitSummary {
    pub(crate) sample_steps: u32,
    pub(crate) score: u32,
    pub(crate) explosion_samples: Vec<ActorScriptCheckExplosionSample>,
    pub(crate) sound_commands: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckHostileLaserHitSample {
    pub(crate) kind: String,
    pub(crate) sample_steps: u32,
    pub(crate) score_delta: u32,
    pub(crate) score: u32,
    pub(crate) explosion_samples: Vec<ActorScriptCheckExplosionSample>,
    pub(crate) sound_commands: Vec<u8>,
    pub(crate) spawned_counts: ActorScriptCheckSpawnedCounts,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckWaveClearSummary {
    pub(crate) assist_steps: u32,
    pub(crate) next_wave: u16,
    pub(crate) score: u32,
    pub(crate) world_enemies: usize,
    pub(crate) world_humans: usize,
    pub(crate) total_survivors: Option<u8>,
    pub(crate) visible_icons: Option<u8>,
    pub(crate) remaining_awards: Option<u8>,
    pub(crate) awarded_points: Option<u32>,
    pub(crate) astronaut_sleep_steps_remaining: Option<u8>,
    pub(crate) wave_advance_sleep_steps_remaining: Option<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckAttractCycleSummary {
    pub(crate) cycle_steps: u64,
    pub(crate) sampled_steps: u64,
    pub(crate) attract_frames: u64,
    pub(crate) non_attract_frames: u64,
    pub(crate) draw_commands: usize,
    pub(crate) scene_sprites: usize,
    pub(crate) saw_williams_reveal: bool,
    pub(crate) saw_defender_coalescence: bool,
    pub(crate) saw_hall_of_fame: bool,
    pub(crate) saw_scoring_surface: bool,
    pub(crate) saw_final_scoring_instruction: bool,
    pub(crate) saw_cycle_return: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorScriptCheckReport {
    pub(crate) path: String,
    pub(crate) attract_events: usize,
    pub(crate) attract_cycle: Option<ActorScriptCheckAttractCycleSummary>,
    pub(crate) attract_cycle_unavailable_reason: Option<String>,
    pub(crate) behavior_kind_profiles: usize,
    pub(crate) behavior_actor_profiles: usize,
    pub(crate) wave_profiles: usize,
    pub(crate) first_frame_phase: String,
    pub(crate) first_frame_draws: usize,
    pub(crate) first_playing_wave: u16,
    pub(crate) first_playing_wave_size: u8,
    pub(crate) first_playing_enemy_landers: u8,
    pub(crate) first_playing_enemy_bombers: u8,
    pub(crate) first_playing_enemy_pods: u8,
    pub(crate) first_playing_enemy_mutants: u8,
    pub(crate) first_playing_enemy_swarmers: u8,
    pub(crate) first_playing_world_enemies: usize,
    pub(crate) first_playing_world_humans: usize,
    pub(crate) first_playing_reserve_landers: u8,
    pub(crate) first_playing_reserve_bombers: u8,
    pub(crate) first_playing_reserve_pods: u8,
    pub(crate) first_playing_reserve_mutants: u8,
    pub(crate) first_playing_reserve_swarmers: u8,
    pub(crate) first_playing_world_scroll_left: u16,
    pub(crate) first_playing_arcade_rng_seed: Option<u8>,
    pub(crate) first_playing_arcade_rng_hseed: Option<u8>,
    pub(crate) first_playing_arcade_rng_lseed: Option<u8>,
    pub(crate) first_playing_actor_samples: Vec<ActorScriptCheckActorSample>,
    pub(crate) first_playing_enemy_projectile_samples: Vec<ActorScriptCheckEnemyProjectileSample>,
    pub(crate) first_playing_sound_commands: Vec<u8>,
    pub(crate) first_player_laser: Option<ActorScriptCheckFirstPlayerLaserSummary>,
    pub(crate) first_player_laser_unavailable_reason: Option<String>,
    pub(crate) first_player_laser_hit: Option<ActorScriptCheckFirstPlayerLaserHitSummary>,
    pub(crate) first_player_laser_hit_unavailable_reason: Option<String>,
    pub(crate) hostile_laser_hit_matrix: Vec<ActorScriptCheckHostileLaserHitSample>,
    pub(crate) hostile_projectile_matrix: Vec<ActorScriptCheckHostileProjectileSample>,
    pub(crate) first_enemy_projectile: Option<ActorScriptCheckFirstEnemyProjectileSummary>,
    pub(crate) first_enemy_projectile_unavailable_reason: Option<String>,
    pub(crate) first_playing_player_takes_enemy_collision_damage: bool,
    pub(crate) first_playing_player_laser_cooldown_steps: u8,
    pub(crate) first_playing_lander_mode: String,
    pub(crate) first_playing_lander_seek_speed: i16,
    pub(crate) first_playing_lander_drift_speed: i16,
    pub(crate) first_playing_lander_fire_period_steps: u64,
    pub(crate) first_playing_mutant_mode: String,
    pub(crate) first_playing_bomber_mode: String,
    pub(crate) first_playing_pod_mode: String,
    pub(crate) first_playing_swarmer_mode: String,
    pub(crate) first_playing_baiter_mode: String,
    pub(crate) first_playing_swarmer_fire_period_steps: u64,
    pub(crate) first_playing_baiter_fire_period_steps: u64,
    pub(crate) wave_clear: Option<ActorScriptCheckWaveClearSummary>,
    pub(crate) wave_clear_unavailable_reason: Option<String>,
    pub(crate) wave_clear_advance_sleep: Option<ActorScriptCheckWaveClearSummary>,
    pub(crate) wave_clear_advance_sleep_unavailable_reason: Option<String>,
    pub(crate) next_playing_assist_steps: Option<u32>,
    pub(crate) next_playing: Option<ActorScriptCheckPlayingSummary>,
    pub(crate) reserve_activation_batches: Vec<ActorScriptCheckReserveActivationSummary>,
    pub(crate) reserve_activation_status: String,
    pub(crate) post_reserve_wave_clear: Option<ActorScriptCheckWaveClearSummary>,
    pub(crate) post_reserve_wave_clear_unavailable_reason: Option<String>,
    pub(crate) post_reserve_wave_clear_advance_sleep: Option<ActorScriptCheckWaveClearSummary>,
    pub(crate) post_reserve_wave_clear_advance_sleep_unavailable_reason: Option<String>,
    pub(crate) post_reserve_next_playing_assist_steps: Option<u32>,
    pub(crate) post_reserve_next_playing: Option<ActorScriptCheckPlayingSummary>,
    pub(crate) post_reserve_next_playing_unavailable_reason: Option<String>,
    pub(crate) clean_exit: bool,
}

impl LiveSmokeReport {
    pub(crate) fn to_text(&self) -> String {
        let frame_size = self
            .first_frame_size
            .map(|(width, height)| format!("{width}x{height}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        let offscreen_first_frame_signature = self
            .offscreen_first_frame_signature
            .map(|signature| format!("{signature:016x}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        let offscreen_last_frame_signature = self
            .offscreen_last_frame_signature
            .map(|signature| format!("{signature:016x}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        format!(
            "wgpu live smoke passed\n  render_path: {}\n  legacy_presenter_used: {}\n  window_created: {}\n  rendered_frames: {}\n  first_frame_size: {}\n  distinct_frame_signatures: {}\n  saw_non_blank_frame: {}\n  saw_attract: {} (visual_frames: {}, visual_signatures: {})\n  saw_credit: {} (visual_frames: {}, visual_signatures: {})\n  saw_playing: {} (visual_frames: {}, visual_signatures: {})\n  clean_game_frames: {}\n  actor_frames: {}\n  sprite_frames: {}\n  sprite_instances: {}\n  sprite_draw_commands: {}\n  temporary_raster_frames: {}\n  temporary_raster_commands: {}\n  offscreen_wgpu_frames: {}\n  offscreen_non_blank_frames: {}\n  offscreen_distinct_frame_signatures: {}\n  offscreen_first_frame_signature: {}\n  offscreen_last_frame_signature: {}\n  injected_inputs: {}\n  clean_exit: {}\n",
            self.render_path,
            self.legacy_presenter_used,
            self.window_created,
            self.rendered_frames,
            frame_size,
            self.distinct_frame_signatures,
            self.saw_non_blank_frame,
            self.saw_attract,
            self.attract_visual_frames,
            self.attract_distinct_frame_signatures,
            self.saw_credit,
            self.credit_visual_frames,
            self.credit_distinct_frame_signatures,
            self.saw_playing,
            self.playing_visual_frames,
            self.playing_distinct_frame_signatures,
            self.clean_game_frames,
            self.actor_frames,
            self.sprite_frames,
            self.sprite_instances,
            self.sprite_draw_commands,
            self.temporary_raster_frames,
            self.temporary_raster_commands,
            self.offscreen_wgpu_frames,
            self.offscreen_non_blank_frames,
            self.offscreen_distinct_frame_signatures,
            offscreen_first_frame_signature,
            offscreen_last_frame_signature,
            self.injected_inputs.join(","),
            self.clean_exit
        )
    }

    #[cfg(all(not(test), not(coverage)))]
    fn validate_actor_offscreen_wgpu(&self) -> anyhow::Result<()> {
        if self.offscreen_wgpu_frames != self.rendered_frames {
            anyhow::bail!(
                "actor wgpu smoke rendered {} offscreen frame(s), expected {}",
                self.offscreen_wgpu_frames,
                self.rendered_frames
            );
        }
        if self.offscreen_non_blank_frames != self.rendered_frames {
            anyhow::bail!(
                "actor wgpu smoke rendered {} nonblank offscreen frame(s), expected {}",
                self.offscreen_non_blank_frames,
                self.rendered_frames
            );
        }
        if self.offscreen_distinct_frame_signatures < 3 {
            anyhow::bail!("actor wgpu smoke did not produce dynamic offscreen frame signatures");
        }
        if self.offscreen_first_frame_signature.is_none() {
            anyhow::bail!("actor wgpu smoke did not record an offscreen frame signature");
        }
        if self.offscreen_last_frame_signature.is_none() {
            anyhow::bail!("actor wgpu smoke did not record a final offscreen frame signature");
        }
        Ok(())
    }
}

impl ActorScriptCheckReport {
    pub(crate) fn to_text(&self) -> String {
        let arcade_rng = arcade_rng_summary(
            self.first_playing_arcade_rng_seed,
            self.first_playing_arcade_rng_hseed,
            self.first_playing_arcade_rng_lseed,
        );
        let attract_cycle = self
            .attract_cycle
            .as_ref()
            .map(|summary| {
                format!(
                    "  attract_cycle_steps: {}\n  attract_cycle_sampled_steps: {}\n  attract_cycle_frames: attract={},non_attract={}\n  attract_cycle_draws: {}\n  attract_cycle_scene_sprites: {}\n  attract_cycle_milestones: williams_reveal={},defender_coalescence={},hall_of_fame={},scoring_surface={},final_scoring_instruction={},cycle_return={}\n",
                    summary.cycle_steps,
                    summary.sampled_steps,
                    summary.attract_frames,
                    summary.non_attract_frames,
                    summary.draw_commands,
                    summary.scene_sprites,
                    summary.saw_williams_reveal,
                    summary.saw_defender_coalescence,
                    summary.saw_hall_of_fame,
                    summary.saw_scoring_surface,
                    summary.saw_final_scoring_instruction,
                    summary.saw_cycle_return,
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "  attract_cycle: unavailable,reason={}\n",
                    self.attract_cycle_unavailable_reason
                        .as_deref()
                        .unwrap_or("not_sampled")
                )
            });
        let wave_clear = self
            .wave_clear
            .as_ref()
            .map(|summary| wave_clear_summary_to_text("wave_clear", summary))
            .unwrap_or_else(|| {
                format!(
                    "  wave_clear: unavailable,reason={}\n",
                    self.wave_clear_unavailable_reason
                        .as_deref()
                        .unwrap_or("not_sampled")
                )
            });
        let wave_clear_advance_sleep = self
            .wave_clear_advance_sleep
            .as_ref()
            .map(|summary| wave_clear_summary_to_text("wave_clear_advance_sleep", summary))
            .unwrap_or_else(|| {
                format!(
                    "  wave_clear_advance_sleep: unavailable,reason={}\n",
                    self.wave_clear_advance_sleep_unavailable_reason
                        .as_deref()
                        .unwrap_or("not_sampled")
                )
            });
        let next_playing = self
            .next_playing
            .as_ref()
            .map(|summary| {
                format!(
                    "  next_playing_assist_steps: {}\n{}",
                    self.next_playing_assist_steps.unwrap_or_default(),
                    playing_summary_to_text("next_playing", summary),
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "  next_playing_wave: unavailable_after_assist_steps={}\n",
                    self.next_playing_assist_steps
                        .unwrap_or(ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT as u32)
                )
            });
        let first_player_laser = self
            .first_player_laser
            .as_ref()
            .map(|summary| {
                format!(
                    "  first_player_laser_sample_steps: {}\n  first_player_laser_samples: {}\n  first_player_laser_sound_commands: {}\n",
                    summary.sample_steps,
                    player_laser_samples_summary(&summary.samples),
                    sound_commands_summary(&summary.sound_commands),
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "  first_player_laser: unavailable,reason={}\n",
                    self.first_player_laser_unavailable_reason
                        .as_deref()
                        .unwrap_or("not_sampled")
                )
            });
        let first_player_laser_hit = self
            .first_player_laser_hit
            .as_ref()
            .map(|summary| {
                format!(
                    "  first_player_laser_hit_sample_steps: {}\n  first_player_laser_hit_score: {}\n  first_player_laser_hit_explosions: {}\n  first_player_laser_hit_sound_commands: {}\n",
                    summary.sample_steps,
                    summary.score,
                    explosion_samples_summary(&summary.explosion_samples),
                    sound_commands_summary(&summary.sound_commands),
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "  first_player_laser_hit: unavailable,reason={}\n",
                    self.first_player_laser_hit_unavailable_reason
                        .as_deref()
                        .unwrap_or("not_sampled")
                )
            });
        let hostile_laser_hit_matrix = format!(
            "  hostile_laser_hit_matrix: {}\n",
            hostile_laser_hit_matrix_summary(&self.hostile_laser_hit_matrix)
        );
        let hostile_projectile_matrix = format!(
            "  hostile_projectile_matrix: {}\n",
            hostile_projectile_matrix_summary(&self.hostile_projectile_matrix)
        );
        let first_enemy_projectile = self
            .first_enemy_projectile
            .as_ref()
            .map(|summary| {
                format!(
                    "  first_enemy_projectile_sample_steps: {}\n  first_enemy_projectile_samples: {}\n  first_enemy_projectile_sound_commands: {}\n",
                    summary.sample_steps,
                    enemy_projectile_samples_summary(&summary.samples),
                    sound_commands_summary(&summary.sound_commands),
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "  first_enemy_projectile: unavailable,reason={}\n",
                    self.first_enemy_projectile_unavailable_reason
                        .as_deref()
                        .unwrap_or("not_sampled")
                )
            });
        let mut reserve_activation = format!(
            "  reserve_activation_batches: {}\n",
            self.reserve_activation_batches.len()
        );
        for (index, summary) in self.reserve_activation_batches.iter().enumerate() {
            let prefix = format!("reserve_activation_{}", index + 1);
            reserve_activation.push_str(&format!(
                "  {prefix}_assist_steps: {}\n  {prefix}_spawned_counts: landers={},bombers={},pods={},mutants={},swarmers={}\n  {prefix}_spawned_samples: {}\n{}",
                summary.assist_steps,
                summary.spawned_counts.landers,
                summary.spawned_counts.bombers,
                summary.spawned_counts.pods,
                summary.spawned_counts.mutants,
                summary.spawned_counts.swarmers,
                spawned_actor_samples_summary(&summary.spawned_samples),
                playing_summary_to_text(&prefix, &summary.playing),
            ));
        }
        reserve_activation.push_str(&format!(
            "  reserve_activation_status: {}\n",
            self.reserve_activation_status
        ));
        let post_reserve_wave_clear = self
            .post_reserve_wave_clear
            .as_ref()
            .map(|summary| wave_clear_summary_to_text("post_reserve_wave_clear", summary))
            .unwrap_or_else(|| {
                format!(
                    "  post_reserve_wave_clear: unavailable,reason={}\n",
                    self.post_reserve_wave_clear_unavailable_reason
                        .as_deref()
                        .unwrap_or("not_sampled")
                )
            });
        let post_reserve_wave_clear_advance_sleep = self
            .post_reserve_wave_clear_advance_sleep
            .as_ref()
            .map(|summary| {
                wave_clear_summary_to_text("post_reserve_wave_clear_advance_sleep", summary)
            })
            .unwrap_or_else(|| {
                format!(
                    "  post_reserve_wave_clear_advance_sleep: unavailable,reason={}\n",
                    self.post_reserve_wave_clear_advance_sleep_unavailable_reason
                        .as_deref()
                        .unwrap_or("not_sampled")
                )
            });
        let post_reserve_next_playing = self
            .post_reserve_next_playing
            .as_ref()
            .map(|summary| {
                format!(
                    "  post_reserve_next_playing_assist_steps: {}\n{}",
                    self.post_reserve_next_playing_assist_steps
                        .unwrap_or_default(),
                    playing_summary_to_text("post_reserve_next_playing", summary),
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "  post_reserve_next_playing: unavailable,reason={}\n",
                    self.post_reserve_next_playing_unavailable_reason
                        .as_deref()
                        .unwrap_or("not_sampled")
                )
            });
        format!(
            "actor script check passed\n  path: {}\n  attract_events: {}\n{}  behavior_kind_profiles: {}\n  behavior_actor_profiles: {}\n  wave_profiles: {}\n  first_frame_phase: {}\n  first_frame_draws: {}\n  first_playing_wave: {}\n  first_playing_wave_size: {}\n  first_playing_enemy_counts: landers={},bombers={},pods={},mutants={},swarmers={}\n  first_playing_world_counts: enemies={},humans={}\n  first_playing_reserve_counts: landers={},bombers={},pods={},mutants={},swarmers={}\n  first_playing_arcade_state: world_scroll_left=0x{:04x},rng={}\n  first_playing_actor_samples: {}\n  first_playing_enemy_projectile_samples: {}\n  first_playing_sound_commands: {}\n  first_playing_player_behavior: takes_enemy_collision_damage={},laser_cooldown_steps={}\n  first_playing_lander_behavior: mode={},seek_speed={},drift_speed={},fire_period_steps={}\n  first_playing_hostile_modes: mutant={},bomber={},pod={},swarmer={},baiter={}\n  first_playing_hostile_fire: swarmer_period_steps={},baiter_period_steps={}\n{}{}{}{}{}{}{}{}{}{}{}{}  clean_exit: {}\n",
            self.path,
            self.attract_events,
            attract_cycle,
            self.behavior_kind_profiles,
            self.behavior_actor_profiles,
            self.wave_profiles,
            self.first_frame_phase,
            self.first_frame_draws,
            self.first_playing_wave,
            self.first_playing_wave_size,
            self.first_playing_enemy_landers,
            self.first_playing_enemy_bombers,
            self.first_playing_enemy_pods,
            self.first_playing_enemy_mutants,
            self.first_playing_enemy_swarmers,
            self.first_playing_world_enemies,
            self.first_playing_world_humans,
            self.first_playing_reserve_landers,
            self.first_playing_reserve_bombers,
            self.first_playing_reserve_pods,
            self.first_playing_reserve_mutants,
            self.first_playing_reserve_swarmers,
            self.first_playing_world_scroll_left,
            arcade_rng,
            actor_samples_summary(&self.first_playing_actor_samples),
            enemy_projectile_samples_summary(&self.first_playing_enemy_projectile_samples),
            sound_commands_summary(&self.first_playing_sound_commands),
            self.first_playing_player_takes_enemy_collision_damage,
            self.first_playing_player_laser_cooldown_steps,
            self.first_playing_lander_mode,
            self.first_playing_lander_seek_speed,
            self.first_playing_lander_drift_speed,
            self.first_playing_lander_fire_period_steps,
            self.first_playing_mutant_mode,
            self.first_playing_bomber_mode,
            self.first_playing_pod_mode,
            self.first_playing_swarmer_mode,
            self.first_playing_baiter_mode,
            self.first_playing_swarmer_fire_period_steps,
            self.first_playing_baiter_fire_period_steps,
            first_player_laser,
            first_player_laser_hit,
            hostile_laser_hit_matrix,
            hostile_projectile_matrix,
            first_enemy_projectile,
            wave_clear,
            wave_clear_advance_sleep,
            next_playing,
            reserve_activation,
            post_reserve_wave_clear,
            post_reserve_wave_clear_advance_sleep,
            post_reserve_next_playing,
            self.clean_exit
        )
    }
}

fn wave_clear_summary_to_text(prefix: &str, summary: &ActorScriptCheckWaveClearSummary) -> String {
    let awarded_points = summary
        .awarded_points
        .map(|points| points.to_string())
        .unwrap_or_else(|| String::from("none"));
    let wave_sleep = summary
        .wave_advance_sleep_steps_remaining
        .map(|steps| steps.to_string())
        .unwrap_or_else(|| String::from("none"));
    format!(
        "  {prefix}_assist_steps: {}\n  {prefix}_next_wave: {}\n  {prefix}_score: {}\n  {prefix}_world_counts: enemies={},humans={}\n  {prefix}_survivor_bonus: total={},visible_icons={},remaining_awards={},awarded_points={}\n  {prefix}_sleep: astronaut_steps={},wave_advance_steps={}\n",
        summary.assist_steps,
        summary.next_wave,
        summary.score,
        summary.world_enemies,
        summary.world_humans,
        optional_u8_summary(summary.total_survivors),
        optional_u8_summary(summary.visible_icons),
        optional_u8_summary(summary.remaining_awards),
        awarded_points,
        optional_u8_summary(summary.astronaut_sleep_steps_remaining),
        wave_sleep,
    )
}

fn optional_u8_summary(value: Option<u8>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("unavailable"))
}

fn playing_summary_to_text(prefix: &str, summary: &ActorScriptCheckPlayingSummary) -> String {
    let arcade_rng = arcade_rng_summary(
        summary.arcade_rng_seed,
        summary.arcade_rng_hseed,
        summary.arcade_rng_lseed,
    );
    format!(
        "  {prefix}_wave: {}\n  {prefix}_wave_size: {}\n  {prefix}_enemy_counts: landers={},bombers={},pods={},mutants={},swarmers={}\n  {prefix}_world_counts: enemies={},humans={}\n  {prefix}_reserve_counts: landers={},bombers={},pods={},mutants={},swarmers={}\n  {prefix}_arcade_state: world_scroll_left=0x{:04x},rng={}\n  {prefix}_actor_samples: {}\n  {prefix}_enemy_projectile_samples: {}\n  {prefix}_sound_commands: {}\n  {prefix}_player_behavior: takes_enemy_collision_damage={},laser_cooldown_steps={}\n  {prefix}_lander_behavior: mode={},seek_speed={},drift_speed={},fire_period_steps={}\n  {prefix}_hostile_modes: mutant={},bomber={},pod={},swarmer={},baiter={}\n  {prefix}_hostile_fire: swarmer_period_steps={},baiter_period_steps={}\n",
        summary.wave,
        summary.wave_size,
        summary.enemy_landers,
        summary.enemy_bombers,
        summary.enemy_pods,
        summary.enemy_mutants,
        summary.enemy_swarmers,
        summary.world_enemies,
        summary.world_humans,
        summary.reserve_landers,
        summary.reserve_bombers,
        summary.reserve_pods,
        summary.reserve_mutants,
        summary.reserve_swarmers,
        summary.world_scroll_left,
        arcade_rng,
        actor_samples_summary(&summary.actor_samples),
        enemy_projectile_samples_summary(&summary.enemy_projectile_samples),
        sound_commands_summary(&summary.sound_commands),
        summary.player_takes_enemy_collision_damage,
        summary.player_laser_cooldown_steps,
        summary.lander_mode,
        summary.lander_seek_speed,
        summary.lander_drift_speed,
        summary.lander_fire_period_steps,
        summary.mutant_mode,
        summary.bomber_mode,
        summary.pod_mode,
        summary.swarmer_mode,
        summary.baiter_mode,
        summary.swarmer_fire_period_steps,
        summary.baiter_fire_period_steps,
    )
}

fn actor_samples_summary(samples: &[ActorScriptCheckActorSample]) -> String {
    if samples.is_empty() {
        return String::from("none");
    }

    samples
        .iter()
        .map(|sample| {
            format!(
                "{}@{},{}[frac=0x{:02x}/0x{:02x}]",
                sample.kind, sample.x, sample.y, sample.x_subpixel, sample.y_subpixel
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn enemy_projectile_samples_summary(samples: &[ActorScriptCheckEnemyProjectileSample]) -> String {
    if samples.is_empty() {
        return String::from("none");
    }

    samples
        .iter()
        .map(|sample| {
            format!(
                "{}@{},{}[frac=0x{:02x}/0x{:02x},vel=0x{:04x}/0x{:04x},life={}]",
                sample.kind,
                sample.x,
                sample.y,
                sample.x_subpixel,
                sample.y_subpixel,
                sample.x_velocity_word,
                sample.y_velocity_word,
                sample.lifetime_ticks,
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn projectile_spawn_samples_summary(samples: &[ActorScriptCheckProjectileSpawnSample]) -> String {
    if samples.is_empty() {
        return String::from("none");
    }

    samples
        .iter()
        .map(|sample| {
            let arcade_state = match (
                sample.x_subpixel,
                sample.y_subpixel,
                sample.x_velocity_word,
                sample.y_velocity_word,
                sample.lifetime_ticks,
            ) {
                (Some(x_fraction), Some(y_fraction), Some(x_velocity), Some(y_velocity), Some(lifetime_ticks)) => format!(
                    "arcade_state=frac=0x{x_fraction:02x}/0x{y_fraction:02x},vel=0x{x_velocity:04x}/0x{y_velocity:04x},life={lifetime_ticks}"
                ),
                _ => String::from("arcade_state=none"),
            };
            format!(
                "{}@{},{}[velocity={}/{},{}]",
                sample.kind,
                sample.x,
                sample.y,
                sample.velocity_dx,
                sample.velocity_dy,
                arcade_state,
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn player_laser_samples_summary(samples: &[ActorScriptCheckPlayerLaserSample]) -> String {
    if samples.is_empty() {
        return String::from("none");
    }

    samples
        .iter()
        .map(|sample| {
            format!(
                "laser@{},{}[velocity={}/{},direction={}]",
                sample.x, sample.y, sample.velocity_dx, sample.velocity_dy, sample.direction,
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn explosion_samples_summary(samples: &[ActorScriptCheckExplosionSample]) -> String {
    if samples.is_empty() {
        return String::from("none");
    }

    samples
        .iter()
        .map(|sample| {
            let explosion_anchor = match (sample.explosion_anchor_x, sample.explosion_anchor_y) {
                (Some(x), Some(y)) => format!("{x},{y}"),
                _ => String::from("none"),
            };
            format!(
                "{}@{},{}[explosion_anchor={}]",
                sample.kind, sample.x, sample.y, explosion_anchor
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn hostile_laser_hit_matrix_summary(samples: &[ActorScriptCheckHostileLaserHitSample]) -> String {
    if samples.is_empty() {
        return String::from("none");
    }

    samples
        .iter()
        .map(|sample| {
            format!(
                "{}@{}[score_delta={},score={},explosions={},sounds={},spawns={}]",
                sample.kind,
                sample.sample_steps,
                sample.score_delta,
                sample.score,
                explosion_samples_summary(&sample.explosion_samples),
                sound_commands_summary(&sample.sound_commands),
                spawned_counts_summary(&sample.spawned_counts),
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn hostile_projectile_matrix_summary(
    samples: &[ActorScriptCheckHostileProjectileSample],
) -> String {
    if samples.is_empty() {
        return String::from("none");
    }

    samples
        .iter()
        .map(|sample| {
            format!(
                "{}@{}[samples={},sounds={}]",
                sample.kind,
                sample.sample_steps,
                projectile_spawn_samples_summary(&sample.samples),
                sound_commands_summary(&sample.sound_commands),
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn sound_commands_summary(commands: &[u8]) -> String {
    if commands.is_empty() {
        return String::from("none");
    }

    commands
        .iter()
        .map(|command| format!("0x{command:02x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn spawned_counts_summary(counts: &ActorScriptCheckSpawnedCounts) -> String {
    if counts.is_empty() {
        return String::from("none");
    }

    format!(
        "landers={},bombers={},pods={},mutants={},swarmers={}",
        counts.landers, counts.bombers, counts.pods, counts.mutants, counts.swarmers
    )
}

fn spawned_actor_samples_summary(samples: &[ActorScriptCheckSpawnedActorSample]) -> String {
    if samples.is_empty() {
        return String::from("none");
    }

    samples
        .iter()
        .map(|sample| format!("{}@{},{}", sample.kind, sample.x, sample.y))
        .collect::<Vec<_>>()
        .join(";")
}

impl From<ActorSmokeReport> for LiveSmokeReport {
    fn from(report: ActorSmokeReport) -> Self {
        Self {
            render_path: "actor_game",
            legacy_presenter_used: false,
            window_created: false,
            rendered_frames: report.frames,
            first_frame_size: report.first_frame_size,
            distinct_frame_signatures: report.distinct_scene_signatures,
            saw_non_blank_frame: report.sprite_frames > 0,
            saw_attract: report.saw_attract,
            saw_credit: report.saw_credit,
            saw_playing: report.saw_playing,
            attract_visual_frames: report.attract_frames,
            credit_visual_frames: report.credited_frames,
            playing_visual_frames: report.playing_frames,
            attract_distinct_frame_signatures: usize::from(report.saw_attract),
            credit_distinct_frame_signatures: usize::from(report.saw_credit),
            playing_distinct_frame_signatures: usize::from(report.saw_playing),
            clean_game_frames: 0,
            actor_frames: report.frames,
            sprite_frames: report.sprite_frames,
            sprite_instances: report.sprite_instances,
            sprite_draw_commands: report.sprite_draw_commands,
            temporary_raster_frames: 0,
            temporary_raster_commands: report.temporary_raster_commands,
            offscreen_wgpu_frames: 0,
            offscreen_non_blank_frames: 0,
            offscreen_distinct_frame_signatures: 0,
            offscreen_first_frame_signature: None,
            offscreen_last_frame_signature: None,
            injected_inputs: report.injected_inputs,
            clean_exit: report.clean_exit,
        }
    }
}

#[cfg(any(test, coverage))]
pub(crate) fn actor_runtime_from_script_path(
    actor_script_path: Option<&Path>,
) -> anyhow::Result<ActorRuntimeAdapter> {
    let Some(path) = actor_script_path else {
        return Ok(ActorRuntimeAdapter::new());
    };
    Ok(ActorRuntimeAdapter::with_scripts(actor_scripts_from_path(
        path,
    )?))
}

pub(crate) fn actor_live_runtime_from_script_path(
    actor_script_path: Option<&Path>,
) -> anyhow::Result<ActorRuntimeAdapter> {
    let Some(path) = actor_script_path else {
        return Ok(ActorRuntimeAdapter::new_with_free_play_admission());
    };
    Ok(ActorRuntimeAdapter::with_scripts_and_free_play_admission(
        actor_scripts_from_path(path)?,
    ))
}
