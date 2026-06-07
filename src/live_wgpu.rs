//! Runtime-facing WGPU live launch facade.

#[cfg(all(not(test), not(coverage)))]
use std::{
    collections::BTreeSet,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use std::{fs, path::Path};

use anyhow::Context;
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
        ActorDriverScripts, ActorFrame, ActorId, ActorKind, ActorRuntimeAdapter, GameCommand,
        GameInput as ActorGameInput, HostileMovementMode, LanderBehaviorMode, Phase, SpawnRequest,
        SpriteKey, VisualEffect, XyzzyController, XyzzyMode,
    },
    actor_smoke::ActorSmokeReport,
    audio::LiveAudioMode,
    game_smoke::GameSmokeReport,
    renderer::{SpriteId, source_message_text},
};
#[cfg(all(not(test), not(coverage)))]
use crate::{
    audio::LiveAudioRuntime,
    game::{Game, GameFrame},
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
#[cfg(all(not(test), not(coverage)))]
const EXPECTED_OFFSCREEN_FIRST_FRAME_SIGNATURE: u64 = 0x8DAE_D38B_41A6_92A9;
#[cfg(all(not(test), not(coverage)))]
const EXPECTED_OFFSCREEN_LAST_FRAME_SIGNATURE: u64 = 0xFE80_CC2B_377E_8066;
const ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT: usize = 512;
const ACTOR_SCRIPT_CHECK_ATTRACT_CYCLE_STEP_LIMIT: u64 = 4096;
const ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT: usize = 4096;
const ACTOR_SCRIPT_CHECK_RESERVE_ACTIVATION_BATCH_LIMIT: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiveInputProfile {
    Planetoid,
    Cabinet,
    Test,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LiveSmokeReport {
    pub(crate) frame_source: &'static str,
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
pub(crate) struct ActorScriptCheckPlayingSummary {
    pub(crate) wave: u16,
    pub(crate) wave_size: u8,
    pub(crate) source_landers: u8,
    pub(crate) source_bombers: u8,
    pub(crate) source_pods: u8,
    pub(crate) source_mutants: u8,
    pub(crate) source_swarmers: u8,
    pub(crate) world_enemies: usize,
    pub(crate) world_humans: usize,
    pub(crate) reserve_landers: u8,
    pub(crate) reserve_bombers: u8,
    pub(crate) reserve_pods: u8,
    pub(crate) reserve_mutants: u8,
    pub(crate) reserve_swarmers: u8,
    pub(crate) source_background_left: u16,
    pub(crate) source_rng_seed: Option<u8>,
    pub(crate) source_rng_hseed: Option<u8>,
    pub(crate) source_rng_lseed: Option<u8>,
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
    pub(crate) playing: ActorScriptCheckPlayingSummary,
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
    pub(crate) saw_final_scoring_label: bool,
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
    pub(crate) first_playing_source_landers: u8,
    pub(crate) first_playing_source_bombers: u8,
    pub(crate) first_playing_source_pods: u8,
    pub(crate) first_playing_source_mutants: u8,
    pub(crate) first_playing_source_swarmers: u8,
    pub(crate) first_playing_world_enemies: usize,
    pub(crate) first_playing_world_humans: usize,
    pub(crate) first_playing_reserve_landers: u8,
    pub(crate) first_playing_reserve_bombers: u8,
    pub(crate) first_playing_reserve_pods: u8,
    pub(crate) first_playing_reserve_mutants: u8,
    pub(crate) first_playing_reserve_swarmers: u8,
    pub(crate) first_playing_source_background_left: u16,
    pub(crate) first_playing_source_rng_seed: Option<u8>,
    pub(crate) first_playing_source_rng_hseed: Option<u8>,
    pub(crate) first_playing_source_rng_lseed: Option<u8>,
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
            "wgpu live smoke passed\n  frame_source: {}\n  legacy_presenter_used: {}\n  window_created: {}\n  rendered_frames: {}\n  first_frame_size: {}\n  distinct_frame_signatures: {}\n  saw_non_blank_frame: {}\n  saw_attract: {} (visual_frames: {}, visual_signatures: {})\n  saw_credit: {} (visual_frames: {}, visual_signatures: {})\n  saw_playing: {} (visual_frames: {}, visual_signatures: {})\n  clean_game_frames: {}\n  actor_frames: {}\n  sprite_frames: {}\n  sprite_instances: {}\n  sprite_draw_commands: {}\n  temporary_raster_frames: {}\n  temporary_raster_commands: {}\n  offscreen_wgpu_frames: {}\n  offscreen_non_blank_frames: {}\n  offscreen_distinct_frame_signatures: {}\n  offscreen_first_frame_signature: {}\n  offscreen_last_frame_signature: {}\n  injected_inputs: {}\n  clean_exit: {}\n",
            self.frame_source,
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
    fn validate_offscreen_wgpu(&self) -> anyhow::Result<()> {
        if self.offscreen_wgpu_frames != self.rendered_frames {
            anyhow::bail!(
                "wgpu live smoke rendered {} offscreen frame(s), expected {}",
                self.offscreen_wgpu_frames,
                self.rendered_frames
            );
        }
        if self.offscreen_non_blank_frames != self.rendered_frames {
            anyhow::bail!(
                "wgpu live smoke rendered {} nonblank offscreen frame(s), expected {}",
                self.offscreen_non_blank_frames,
                self.rendered_frames
            );
        }
        if self.offscreen_distinct_frame_signatures < 3 {
            anyhow::bail!("wgpu live smoke did not produce dynamic offscreen frame signatures");
        }
        let Some(first_frame_signature) = self.offscreen_first_frame_signature else {
            anyhow::bail!("wgpu live smoke did not record an offscreen frame signature");
        };
        if first_frame_signature != EXPECTED_OFFSCREEN_FIRST_FRAME_SIGNATURE {
            anyhow::bail!(
                "wgpu live smoke first offscreen frame signature {first_frame_signature:016x} did not match expected {EXPECTED_OFFSCREEN_FIRST_FRAME_SIGNATURE:016x}"
            );
        }
        let Some(last_frame_signature) = self.offscreen_last_frame_signature else {
            anyhow::bail!("wgpu live smoke did not record a final offscreen frame signature");
        };
        if last_frame_signature != EXPECTED_OFFSCREEN_LAST_FRAME_SIGNATURE {
            anyhow::bail!(
                "wgpu live smoke last offscreen frame signature {last_frame_signature:016x} did not match expected {EXPECTED_OFFSCREEN_LAST_FRAME_SIGNATURE:016x}"
            );
        }
        Ok(())
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
        let source_rng = source_rng_summary(
            self.first_playing_source_rng_seed,
            self.first_playing_source_rng_hseed,
            self.first_playing_source_rng_lseed,
        );
        let attract_cycle = self
            .attract_cycle
            .as_ref()
            .map(|summary| {
                format!(
                    "  attract_cycle_steps: {}\n  attract_cycle_sampled_steps: {}\n  attract_cycle_frames: attract={},non_attract={}\n  attract_cycle_draws: {}\n  attract_cycle_scene_sprites: {}\n  attract_cycle_milestones: williams_reveal={},defender_coalescence={},hall_of_fame={},scoring_surface={},final_scoring_label={},cycle_return={}\n",
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
                    summary.saw_final_scoring_label,
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
        let mut reserve_activation = format!(
            "  reserve_activation_batches: {}\n",
            self.reserve_activation_batches.len()
        );
        for (index, summary) in self.reserve_activation_batches.iter().enumerate() {
            let prefix = format!("reserve_activation_{}", index + 1);
            reserve_activation.push_str(&format!(
                "  {prefix}_assist_steps: {}\n  {prefix}_spawned_counts: landers={},bombers={},pods={},mutants={},swarmers={}\n{}",
                summary.assist_steps,
                summary.spawned_counts.landers,
                summary.spawned_counts.bombers,
                summary.spawned_counts.pods,
                summary.spawned_counts.mutants,
                summary.spawned_counts.swarmers,
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
            "actor script check passed\n  path: {}\n  attract_events: {}\n{}  behavior_kind_profiles: {}\n  behavior_actor_profiles: {}\n  wave_profiles: {}\n  first_frame_phase: {}\n  first_frame_draws: {}\n  first_playing_wave: {}\n  first_playing_wave_size: {}\n  first_playing_source_counts: landers={},bombers={},pods={},mutants={},swarmers={}\n  first_playing_world_counts: enemies={},humans={}\n  first_playing_reserve_counts: landers={},bombers={},pods={},mutants={},swarmers={}\n  first_playing_source_state: background_left=0x{:04x},rng={}\n  first_playing_player_behavior: takes_enemy_collision_damage={},laser_cooldown_steps={}\n  first_playing_lander_behavior: mode={},seek_speed={},drift_speed={},fire_period_steps={}\n  first_playing_hostile_modes: mutant={},bomber={},pod={},swarmer={},baiter={}\n  first_playing_hostile_fire: swarmer_period_steps={},baiter_period_steps={}\n{}{}{}{}{}{}{}  clean_exit: {}\n",
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
            self.first_playing_source_landers,
            self.first_playing_source_bombers,
            self.first_playing_source_pods,
            self.first_playing_source_mutants,
            self.first_playing_source_swarmers,
            self.first_playing_world_enemies,
            self.first_playing_world_humans,
            self.first_playing_reserve_landers,
            self.first_playing_reserve_bombers,
            self.first_playing_reserve_pods,
            self.first_playing_reserve_mutants,
            self.first_playing_reserve_swarmers,
            self.first_playing_source_background_left,
            source_rng,
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
    let source_rng = source_rng_summary(
        summary.source_rng_seed,
        summary.source_rng_hseed,
        summary.source_rng_lseed,
    );
    format!(
        "  {prefix}_wave: {}\n  {prefix}_wave_size: {}\n  {prefix}_source_counts: landers={},bombers={},pods={},mutants={},swarmers={}\n  {prefix}_world_counts: enemies={},humans={}\n  {prefix}_reserve_counts: landers={},bombers={},pods={},mutants={},swarmers={}\n  {prefix}_source_state: background_left=0x{:04x},rng={}\n  {prefix}_player_behavior: takes_enemy_collision_damage={},laser_cooldown_steps={}\n  {prefix}_lander_behavior: mode={},seek_speed={},drift_speed={},fire_period_steps={}\n  {prefix}_hostile_modes: mutant={},bomber={},pod={},swarmer={},baiter={}\n  {prefix}_hostile_fire: swarmer_period_steps={},baiter_period_steps={}\n",
        summary.wave,
        summary.wave_size,
        summary.source_landers,
        summary.source_bombers,
        summary.source_pods,
        summary.source_mutants,
        summary.source_swarmers,
        summary.world_enemies,
        summary.world_humans,
        summary.reserve_landers,
        summary.reserve_bombers,
        summary.reserve_pods,
        summary.reserve_mutants,
        summary.reserve_swarmers,
        summary.source_background_left,
        source_rng,
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

impl From<GameSmokeReport> for LiveSmokeReport {
    fn from(report: GameSmokeReport) -> Self {
        Self {
            frame_source: "clean_game",
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
            clean_game_frames: report.frames,
            actor_frames: 0,
            sprite_frames: report.sprite_frames,
            sprite_instances: report.sprite_instances,
            sprite_draw_commands: report.sprite_draw_commands,
            temporary_raster_frames: report.raster_frames,
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

impl From<ActorSmokeReport> for LiveSmokeReport {
    fn from(report: ActorSmokeReport) -> Self {
        Self {
            frame_source: "actor_game",
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

pub(crate) fn actor_runtime_from_script_path(
    actor_script_path: Option<&Path>,
) -> anyhow::Result<ActorRuntimeAdapter> {
    let Some(path) = actor_script_path else {
        return Ok(ActorRuntimeAdapter::new());
    };
    let source = fs::read_to_string(path)
        .with_context(|| format!("reading actor driver script {}", path.display()))?;
    let scripts = source
        .parse::<ActorDriverScripts>()
        .with_context(|| format!("parsing actor driver script {}", path.display()))?;
    Ok(ActorRuntimeAdapter::with_scripts(scripts))
}

pub(crate) fn run_actor_script_check(path: &Path) -> anyhow::Result<ActorScriptCheckReport> {
    let mut runtime = actor_runtime_from_script_path(Some(path))?;
    let manifest = runtime.driver().script_manifest();
    let frame = runtime.step(ActorGameInput::NONE);
    let (attract_cycle, attract_cycle_unavailable_reason) =
        actor_script_check_attract_cycle(&mut runtime, manifest.attract_script.cycle_steps, &frame);
    let playing = run_actor_script_check_to_first_playing_wave(&mut runtime)?;
    let first_playing = actor_script_check_playing_summary(&playing);
    let next_wave_progression =
        run_actor_script_check_to_next_wave_progression(&mut runtime, &playing);
    let reserve_activation = actor_script_check_reserve_activations(
        &mut runtime,
        next_wave_progression.next_playing.as_ref(),
    );
    let (next_playing_assist_steps, next_playing) = match next_wave_progression.next_playing {
        Some(next_playing_frame) => (
            Some(next_playing_frame.assist_steps),
            Some(actor_script_check_playing_summary(
                &next_playing_frame.frame,
            )),
        ),
        None => (Some(ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT as u32), None),
    };

    Ok(ActorScriptCheckReport {
        path: path.display().to_string(),
        attract_events: manifest.attract_script.events.len(),
        attract_cycle,
        attract_cycle_unavailable_reason,
        behavior_kind_profiles: manifest.behavior_script.kind_profiles.len(),
        behavior_actor_profiles: manifest.behavior_script.actor_profiles.len(),
        wave_profiles: manifest.wave_script.waves.len(),
        first_frame_phase: format!("{:?}", frame.state.phase),
        first_frame_draws: frame.report.draws.len(),
        first_playing_wave: first_playing.wave,
        first_playing_wave_size: first_playing.wave_size,
        first_playing_source_landers: first_playing.source_landers,
        first_playing_source_bombers: first_playing.source_bombers,
        first_playing_source_pods: first_playing.source_pods,
        first_playing_source_mutants: first_playing.source_mutants,
        first_playing_source_swarmers: first_playing.source_swarmers,
        first_playing_world_enemies: first_playing.world_enemies,
        first_playing_world_humans: first_playing.world_humans,
        first_playing_reserve_landers: first_playing.reserve_landers,
        first_playing_reserve_bombers: first_playing.reserve_bombers,
        first_playing_reserve_pods: first_playing.reserve_pods,
        first_playing_reserve_mutants: first_playing.reserve_mutants,
        first_playing_reserve_swarmers: first_playing.reserve_swarmers,
        first_playing_source_background_left: first_playing.source_background_left,
        first_playing_source_rng_seed: first_playing.source_rng_seed,
        first_playing_source_rng_hseed: first_playing.source_rng_hseed,
        first_playing_source_rng_lseed: first_playing.source_rng_lseed,
        first_playing_player_takes_enemy_collision_damage: first_playing
            .player_takes_enemy_collision_damage,
        first_playing_player_laser_cooldown_steps: first_playing.player_laser_cooldown_steps,
        first_playing_lander_mode: first_playing.lander_mode,
        first_playing_lander_seek_speed: first_playing.lander_seek_speed,
        first_playing_lander_drift_speed: first_playing.lander_drift_speed,
        first_playing_lander_fire_period_steps: first_playing.lander_fire_period_steps,
        first_playing_mutant_mode: first_playing.mutant_mode,
        first_playing_bomber_mode: first_playing.bomber_mode,
        first_playing_pod_mode: first_playing.pod_mode,
        first_playing_swarmer_mode: first_playing.swarmer_mode,
        first_playing_baiter_mode: first_playing.baiter_mode,
        first_playing_swarmer_fire_period_steps: first_playing.swarmer_fire_period_steps,
        first_playing_baiter_fire_period_steps: first_playing.baiter_fire_period_steps,
        wave_clear: next_wave_progression.wave_clear,
        wave_clear_unavailable_reason: next_wave_progression.wave_clear_unavailable_reason,
        wave_clear_advance_sleep: next_wave_progression.wave_clear_advance_sleep,
        wave_clear_advance_sleep_unavailable_reason: next_wave_progression
            .wave_clear_advance_sleep_unavailable_reason,
        next_playing_assist_steps,
        next_playing,
        reserve_activation_batches: reserve_activation.batches,
        reserve_activation_status: reserve_activation.status,
        post_reserve_wave_clear: reserve_activation.post_reserve_wave_clear,
        post_reserve_wave_clear_unavailable_reason: reserve_activation
            .post_reserve_wave_clear_unavailable_reason,
        post_reserve_wave_clear_advance_sleep: reserve_activation
            .post_reserve_wave_clear_advance_sleep,
        post_reserve_wave_clear_advance_sleep_unavailable_reason: reserve_activation
            .post_reserve_wave_clear_advance_sleep_unavailable_reason,
        post_reserve_next_playing_assist_steps: reserve_activation
            .post_reserve_next_playing_assist_steps,
        post_reserve_next_playing: reserve_activation.post_reserve_next_playing,
        post_reserve_next_playing_unavailable_reason: reserve_activation
            .post_reserve_next_playing_unavailable_reason,
        clean_exit: true,
    })
}

#[derive(Debug, Clone, PartialEq)]
struct ActorScriptCheckNextPlayingFrame {
    frame: ActorFrame,
    assist_steps: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ActorScriptCheckNextWaveProgression {
    wave_clear: Option<ActorScriptCheckWaveClearSummary>,
    wave_clear_unavailable_reason: Option<String>,
    wave_clear_advance_sleep: Option<ActorScriptCheckWaveClearSummary>,
    wave_clear_advance_sleep_unavailable_reason: Option<String>,
    next_playing: Option<ActorScriptCheckNextPlayingFrame>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ActorScriptCheckReserveActivationSequence {
    batches: Vec<ActorScriptCheckReserveActivationSummary>,
    status: String,
    post_reserve_wave_clear: Option<ActorScriptCheckWaveClearSummary>,
    post_reserve_wave_clear_unavailable_reason: Option<String>,
    post_reserve_wave_clear_advance_sleep: Option<ActorScriptCheckWaveClearSummary>,
    post_reserve_wave_clear_advance_sleep_unavailable_reason: Option<String>,
    post_reserve_next_playing_assist_steps: Option<u32>,
    post_reserve_next_playing: Option<ActorScriptCheckPlayingSummary>,
    post_reserve_next_playing_unavailable_reason: Option<String>,
}

fn actor_script_check_attract_cycle(
    runtime: &mut ActorRuntimeAdapter,
    cycle_steps: Option<u64>,
    first_frame: &ActorFrame,
) -> (Option<ActorScriptCheckAttractCycleSummary>, Option<String>) {
    let Some(cycle_steps) = cycle_steps else {
        return (None, Some(String::from("no_attract_cycle_declared")));
    };
    if cycle_steps > ACTOR_SCRIPT_CHECK_ATTRACT_CYCLE_STEP_LIMIT {
        return (
            None,
            Some(format!(
                "attract_cycle_exceeds_check_limit_{}",
                ACTOR_SCRIPT_CHECK_ATTRACT_CYCLE_STEP_LIMIT
            )),
        );
    }

    let mut summary = ActorScriptCheckAttractCycleSummary {
        cycle_steps,
        ..ActorScriptCheckAttractCycleSummary::default()
    };
    actor_script_check_observe_attract_cycle_frame(&mut summary, first_frame);
    for _ in 1..cycle_steps {
        let frame = runtime.step(ActorGameInput::NONE);
        actor_script_check_observe_attract_cycle_frame(&mut summary, &frame);
    }
    (Some(summary), None)
}

fn actor_script_check_observe_attract_cycle_frame(
    summary: &mut ActorScriptCheckAttractCycleSummary,
    frame: &ActorFrame,
) {
    let hall_title = source_message_text("HALLD_TITLE").expect("HALLD_TITLE message is checked in");
    let final_scoring_label = source_message_text("SWARMV").expect("SWARMV message is checked in");
    let mut cycle_has_first_williams_step = false;
    let mut cycle_has_scoring_surface = false;
    let mut cycle_has_final_label = false;

    summary.sampled_steps = summary.sampled_steps.saturating_add(1);
    if frame.report.phase == Phase::Attract {
        summary.attract_frames = summary.attract_frames.saturating_add(1);
    } else {
        summary.non_attract_frames = summary.non_attract_frames.saturating_add(1);
    }
    summary.draw_commands = summary
        .draw_commands
        .saturating_add(frame.report.draws.len());
    summary.scene_sprites = summary
        .scene_sprites
        .saturating_add(frame.scene.sprites.len());

    for draw in &frame.report.draws {
        if draw.sprite == SpriteKey::WilliamsLogo
            && matches!(draw.effect, VisualEffect::WilliamsReveal { .. })
        {
            summary.saw_williams_reveal = true;
        }
        if draw.sprite == SpriteKey::DefenderCoalescence {
            summary.saw_defender_coalescence = true;
        }
        if draw.text.as_deref() == Some(hall_title) {
            summary.saw_hall_of_fame = true;
        }
        if matches!(draw.effect, VisualEffect::AttractScoringSurface { .. }) {
            summary.saw_scoring_surface = true;
            cycle_has_scoring_surface = true;
        }
        if draw.text.as_deref() == Some(final_scoring_label) {
            summary.saw_final_scoring_label = true;
            cycle_has_final_label = true;
        }
        if frame.report.step == summary.cycle_steps
            && draw.sprite == SpriteKey::WilliamsLogo
            && matches!(
                draw.effect,
                VisualEffect::WilliamsReveal { stroke_step: 1, .. }
            )
        {
            cycle_has_first_williams_step = true;
        }
    }

    for sprite in &frame.scene.sprites {
        if sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL {
            summary.saw_williams_reveal = true;
        }
        if SpriteId::attract_defender_wordmark_block(0) == Some(sprite.sprite) {
            summary.saw_defender_coalescence = true;
        }
        if sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO {
            summary.saw_hall_of_fame = true;
        }
    }

    if frame.report.step == summary.cycle_steps {
        summary.saw_cycle_return =
            cycle_has_first_williams_step && !cycle_has_scoring_surface && !cycle_has_final_label;
    }
}

fn actor_script_check_playing_summary(frame: &ActorFrame) -> ActorScriptCheckPlayingSummary {
    let profile = frame.report.source_wave;
    let reserve = frame.state.world.enemy_reserve;
    debug_assert_eq!(reserve, frame.report.enemy_reserve);
    let source_rng = frame.report.source_rng;
    let player_behavior = first_playing_behavior_for(frame, ActorKind::Player);
    let lander_behavior = first_playing_behavior_for(frame, ActorKind::Lander);
    let mutant_behavior = first_playing_behavior_for(frame, ActorKind::Mutant);
    let bomber_behavior = first_playing_behavior_for(frame, ActorKind::Bomber);
    let pod_behavior = first_playing_behavior_for(frame, ActorKind::Pod);
    let swarmer_behavior = first_playing_behavior_for(frame, ActorKind::Swarmer);
    let baiter_behavior = first_playing_behavior_for(frame, ActorKind::Baiter);

    ActorScriptCheckPlayingSummary {
        wave: frame.report.wave,
        wave_size: profile.wave_size,
        source_landers: profile.landers,
        source_bombers: profile.bombers,
        source_pods: profile.pods,
        source_mutants: profile.mutants,
        source_swarmers: profile.swarmers,
        world_enemies: frame.state.world.enemies.len(),
        world_humans: frame.state.world.humans.len(),
        reserve_landers: reserve.landers,
        reserve_bombers: reserve.bombers,
        reserve_pods: reserve.pods,
        reserve_mutants: reserve.mutants,
        reserve_swarmers: reserve.swarmers,
        source_background_left: frame.report.source_background_left,
        source_rng_seed: source_rng.map(|source_rng| source_rng.seed),
        source_rng_hseed: source_rng.map(|source_rng| source_rng.hseed),
        source_rng_lseed: source_rng.map(|source_rng| source_rng.lseed),
        player_takes_enemy_collision_damage: player_behavior.player_takes_enemy_collision_damage,
        player_laser_cooldown_steps: player_behavior.player_laser_cooldown_steps,
        lander_mode: lander_behavior_mode_label(lander_behavior.lander_mode).to_string(),
        lander_seek_speed: lander_behavior.lander_seek_speed,
        lander_drift_speed: lander_behavior.lander_drift_speed,
        lander_fire_period_steps: lander_behavior.lander_fire_period_steps,
        mutant_mode: hostile_movement_mode_label(mutant_behavior.mutant_mode).to_string(),
        bomber_mode: hostile_movement_mode_label(bomber_behavior.bomber_mode).to_string(),
        pod_mode: hostile_movement_mode_label(pod_behavior.pod_mode).to_string(),
        swarmer_mode: hostile_movement_mode_label(swarmer_behavior.swarmer_mode).to_string(),
        baiter_mode: hostile_movement_mode_label(baiter_behavior.baiter_mode).to_string(),
        swarmer_fire_period_steps: swarmer_behavior.swarmer_fire_period_steps,
        baiter_fire_period_steps: baiter_behavior.baiter_fire_period_steps,
    }
}

fn source_rng_summary(seed: Option<u8>, hseed: Option<u8>, lseed: Option<u8>) -> String {
    match (seed, hseed, lseed) {
        (Some(seed), Some(hseed), Some(lseed)) => {
            format!("seed=0x{seed:02x},hseed=0x{hseed:02x},lseed=0x{lseed:02x}")
        }
        _ => String::from("unavailable"),
    }
}

fn first_playing_behavior_for(
    frame: &ActorFrame,
    kind: ActorKind,
) -> crate::actor_game::ActorBehaviorProfile {
    let actor = frame
        .report
        .snapshots
        .iter()
        .find(|snapshot| snapshot.kind == kind && snapshot.alive)
        .map(|snapshot| snapshot.id)
        .unwrap_or_else(|| ActorId::new(0));
    frame.report.behavior_script.behavior_for(actor, kind)
}

fn lander_behavior_mode_label(mode: LanderBehaviorMode) -> &'static str {
    match mode {
        LanderBehaviorMode::SeekNearestHuman => "seek_nearest_human",
        LanderBehaviorMode::ChasePlayer => "chase_player",
        LanderBehaviorMode::Drift => "drift",
    }
}

fn hostile_movement_mode_label(mode: HostileMovementMode) -> &'static str {
    match mode {
        HostileMovementMode::Drift => "drift",
        HostileMovementMode::ChasePlayer => "chase_player",
    }
}

fn run_actor_script_check_to_first_playing_wave(
    runtime: &mut ActorRuntimeAdapter,
) -> anyhow::Result<ActorFrame> {
    runtime.step(ActorGameInput {
        coin: true,
        ..ActorGameInput::NONE
    });
    runtime.step(ActorGameInput {
        start_one: true,
        ..ActorGameInput::NONE
    });

    for _ in 0..ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT {
        let frame = runtime.step(ActorGameInput::NONE);
        if frame.report.phase == Phase::Playing && frame.report.player_start.is_none() {
            return Ok(frame);
        }
    }

    anyhow::bail!(
        "actor script check did not reach the first playable wave within {ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT} actor steps"
    );
}

fn run_actor_script_check_to_next_wave_progression(
    runtime: &mut ActorRuntimeAdapter,
    first_playing: &ActorFrame,
) -> ActorScriptCheckNextWaveProgression {
    let mut frame = first_playing.clone();
    let mut wave_clear = None;
    let mut wave_clear_advance_sleep = None;
    let first_wave = frame.report.wave;
    for step in 1..=ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT {
        let input = actor_script_check_next_wave_input(&frame);
        frame = runtime.step(input);
        if wave_clear.is_none() {
            wave_clear = actor_script_check_wave_clear_summary(&frame, step as u32);
        }
        if wave_clear_advance_sleep.is_none() {
            wave_clear_advance_sleep =
                actor_script_check_wave_clear_advance_sleep_summary(&frame, step as u32);
        }
        if frame.report.phase == Phase::Playing
            && frame.report.player_start.is_none()
            && frame.report.wave > first_wave
        {
            return ActorScriptCheckNextWaveProgression {
                wave_clear_unavailable_reason: wave_clear
                    .is_none()
                    .then(|| String::from("wave_clear_not_observed")),
                wave_clear_advance_sleep_unavailable_reason: wave_clear_advance_sleep
                    .is_none()
                    .then(|| String::from("wave_clear_advance_sleep_not_observed")),
                wave_clear_advance_sleep,
                wave_clear,
                next_playing: Some(ActorScriptCheckNextPlayingFrame {
                    frame,
                    assist_steps: step as u32,
                }),
            };
        }
    }

    ActorScriptCheckNextWaveProgression {
        wave_clear_unavailable_reason: wave_clear
            .is_none()
            .then(|| String::from("wave_clear_not_observed")),
        wave_clear_advance_sleep_unavailable_reason: wave_clear_advance_sleep
            .is_none()
            .then(|| String::from("wave_clear_advance_sleep_not_observed")),
        wave_clear_advance_sleep,
        wave_clear,
        next_playing: None,
    }
}

fn actor_script_check_wave_clear_summary(
    frame: &ActorFrame,
    assist_steps: u32,
) -> Option<ActorScriptCheckWaveClearSummary> {
    let next_wave = frame
        .report
        .commands
        .iter()
        .find_map(|command| match command {
            GameCommand::WaveCleared { next_wave } => Some(*next_wave),
            _ => None,
        })?;
    let survivor_bonus = frame.report.survivor_bonus;
    Some(ActorScriptCheckWaveClearSummary {
        assist_steps,
        next_wave,
        score: frame.report.score,
        world_enemies: frame.state.world.enemies.len(),
        world_humans: frame.state.world.humans.len(),
        total_survivors: survivor_bonus.map(|bonus| bonus.total_survivors),
        visible_icons: survivor_bonus.map(|bonus| bonus.visible_icons),
        remaining_awards: survivor_bonus.map(|bonus| bonus.remaining_awards),
        awarded_points: survivor_bonus.and_then(|bonus| bonus.awarded_points),
        astronaut_sleep_steps_remaining: survivor_bonus
            .map(|bonus| bonus.astronaut_sleep_steps_remaining),
        wave_advance_sleep_steps_remaining: survivor_bonus
            .and_then(|bonus| bonus.wave_advance_sleep_steps_remaining),
    })
}

fn actor_script_check_wave_clear_advance_sleep_summary(
    frame: &ActorFrame,
    assist_steps: u32,
) -> Option<ActorScriptCheckWaveClearSummary> {
    let survivor_bonus = frame.report.survivor_bonus?;
    let wave_advance_sleep_steps_remaining = survivor_bonus.wave_advance_sleep_steps_remaining?;
    Some(ActorScriptCheckWaveClearSummary {
        assist_steps,
        next_wave: survivor_bonus.next_wave,
        score: frame.report.score,
        world_enemies: frame.state.world.enemies.len(),
        world_humans: frame.state.world.humans.len(),
        total_survivors: Some(survivor_bonus.total_survivors),
        visible_icons: Some(survivor_bonus.visible_icons),
        remaining_awards: Some(survivor_bonus.remaining_awards),
        awarded_points: survivor_bonus.awarded_points,
        astronaut_sleep_steps_remaining: Some(survivor_bonus.astronaut_sleep_steps_remaining),
        wave_advance_sleep_steps_remaining: Some(wave_advance_sleep_steps_remaining),
    })
}

fn actor_script_check_reserve_activations(
    runtime: &mut ActorRuntimeAdapter,
    next_playing: Option<&ActorScriptCheckNextPlayingFrame>,
) -> ActorScriptCheckReserveActivationSequence {
    let Some(next_playing) = next_playing else {
        return ActorScriptCheckReserveActivationSequence::unavailable("next_playing_unavailable");
    };
    if actor_script_check_reserve_total(next_playing.frame.report.enemy_reserve) == 0 {
        return ActorScriptCheckReserveActivationSequence::unavailable(
            "next_playing_has_no_reserve",
        );
    }

    let mut frame = next_playing.frame.clone();
    let mut batches = Vec::new();
    let wave = frame.report.wave;
    for step in 1..=ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT {
        let previous_reserve = frame.report.enemy_reserve;
        let input = actor_script_check_assist_input(&frame);
        frame = runtime.step(input);
        let spawned_counts = actor_script_check_spawned_counts(&frame);
        if frame.report.phase == Phase::Playing
            && frame.report.wave == wave
            && actor_script_check_reserve_total(previous_reserve) > 0
            && !spawned_counts.is_empty()
        {
            batches.push(ActorScriptCheckReserveActivationSummary {
                assist_steps: step as u32,
                spawned_counts,
                playing: actor_script_check_playing_summary(&frame),
            });
            if actor_script_check_reserve_total(frame.report.enemy_reserve) == 0 {
                return actor_script_check_to_post_reserve_wave_clear(
                    runtime,
                    frame,
                    step as u32,
                    batches,
                );
            }
            if batches.len() >= ACTOR_SCRIPT_CHECK_RESERVE_ACTIVATION_BATCH_LIMIT {
                return ActorScriptCheckReserveActivationSequence::new(
                    batches,
                    "batch_limit_reached",
                );
            }
        }
        if frame.report.wave > wave {
            let status = if batches.is_empty() {
                "wave_advanced_before_reserve_activation"
            } else {
                "wave_advanced_before_reserve_empty"
            };
            return ActorScriptCheckReserveActivationSequence::new(batches, status);
        }
    }

    let status = if batches.is_empty() {
        "reserve_activation_not_reached"
    } else {
        "step_limit_reached"
    };
    ActorScriptCheckReserveActivationSequence::new(batches, status)
}

impl ActorScriptCheckReserveActivationSequence {
    fn new(batches: Vec<ActorScriptCheckReserveActivationSummary>, status: &str) -> Self {
        Self {
            batches,
            status: status.to_string(),
            post_reserve_wave_clear: None,
            post_reserve_wave_clear_unavailable_reason: Some(status.to_string()),
            post_reserve_wave_clear_advance_sleep: None,
            post_reserve_wave_clear_advance_sleep_unavailable_reason: Some(status.to_string()),
            post_reserve_next_playing_assist_steps: None,
            post_reserve_next_playing: None,
            post_reserve_next_playing_unavailable_reason: Some(status.to_string()),
        }
    }

    fn unavailable(status: &str) -> Self {
        Self::new(Vec::new(), status)
    }

    fn with_post_reserve_wave_clear(
        batches: Vec<ActorScriptCheckReserveActivationSummary>,
        summary: ActorScriptCheckWaveClearSummary,
        advance_sleep: Option<ActorScriptCheckWaveClearSummary>,
        advance_sleep_unavailable_reason: Option<String>,
        next_playing_assist_steps: Option<u32>,
        next_playing: Option<ActorScriptCheckPlayingSummary>,
        next_playing_unavailable_reason: Option<String>,
    ) -> Self {
        Self {
            batches,
            status: String::from("reserve_empty"),
            post_reserve_wave_clear: Some(summary),
            post_reserve_wave_clear_unavailable_reason: None,
            post_reserve_wave_clear_advance_sleep: advance_sleep,
            post_reserve_wave_clear_advance_sleep_unavailable_reason:
                advance_sleep_unavailable_reason,
            post_reserve_next_playing_assist_steps: next_playing_assist_steps,
            post_reserve_next_playing: next_playing,
            post_reserve_next_playing_unavailable_reason: next_playing_unavailable_reason,
        }
    }

    fn with_post_reserve_wave_clear_unavailable(
        batches: Vec<ActorScriptCheckReserveActivationSummary>,
        reason: &str,
    ) -> Self {
        Self {
            batches,
            status: String::from("reserve_empty"),
            post_reserve_wave_clear: None,
            post_reserve_wave_clear_unavailable_reason: Some(reason.to_string()),
            post_reserve_wave_clear_advance_sleep: None,
            post_reserve_wave_clear_advance_sleep_unavailable_reason: Some(reason.to_string()),
            post_reserve_next_playing_assist_steps: None,
            post_reserve_next_playing: None,
            post_reserve_next_playing_unavailable_reason: Some(reason.to_string()),
        }
    }
}

fn actor_script_check_to_post_reserve_wave_clear(
    runtime: &mut ActorRuntimeAdapter,
    reserve_empty_frame: ActorFrame,
    reserve_empty_assist_steps: u32,
    batches: Vec<ActorScriptCheckReserveActivationSummary>,
) -> ActorScriptCheckReserveActivationSequence {
    let mut frame = reserve_empty_frame;
    let wave = frame.report.wave;
    for step in 1..=ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT {
        let input = actor_script_check_assist_input(&frame);
        frame = runtime.step(input);
        let assist_steps = reserve_empty_assist_steps.saturating_add(step as u32);
        if let Some(summary) = actor_script_check_wave_clear_summary(&frame, assist_steps) {
            return actor_script_check_to_post_reserve_wave_clear_advance_sleep(
                runtime,
                frame,
                assist_steps,
                batches,
                summary,
            );
        }
        if frame.report.wave > wave {
            return ActorScriptCheckReserveActivationSequence::with_post_reserve_wave_clear_unavailable(
                batches,
                "wave_advanced_before_post_reserve_wave_clear",
            );
        }
    }

    ActorScriptCheckReserveActivationSequence::with_post_reserve_wave_clear_unavailable(
        batches,
        "post_reserve_wave_clear_not_observed",
    )
}

fn actor_script_check_to_post_reserve_wave_clear_advance_sleep(
    runtime: &mut ActorRuntimeAdapter,
    wave_clear_frame: ActorFrame,
    wave_clear_assist_steps: u32,
    batches: Vec<ActorScriptCheckReserveActivationSummary>,
    wave_clear: ActorScriptCheckWaveClearSummary,
) -> ActorScriptCheckReserveActivationSequence {
    let mut frame = wave_clear_frame;
    let wave = frame.report.wave;
    if let Some(summary) =
        actor_script_check_wave_clear_advance_sleep_summary(&frame, wave_clear_assist_steps)
    {
        let (next_steps, next_playing, next_reason) =
            actor_script_check_to_post_reserve_next_playing(
                runtime,
                frame,
                wave_clear_assist_steps,
                wave,
            );
        return ActorScriptCheckReserveActivationSequence::with_post_reserve_wave_clear(
            batches,
            wave_clear,
            Some(summary),
            None,
            next_steps,
            next_playing,
            next_reason,
        );
    }

    for step in 1..=ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT {
        let input = actor_script_check_assist_input(&frame);
        frame = runtime.step(input);
        let assist_steps = wave_clear_assist_steps.saturating_add(step as u32);
        if let Some(summary) =
            actor_script_check_wave_clear_advance_sleep_summary(&frame, assist_steps)
        {
            let (next_steps, next_playing, next_reason) =
                actor_script_check_to_post_reserve_next_playing(runtime, frame, assist_steps, wave);
            return ActorScriptCheckReserveActivationSequence::with_post_reserve_wave_clear(
                batches,
                wave_clear,
                Some(summary),
                None,
                next_steps,
                next_playing,
                next_reason,
            );
        }
        if frame.report.wave > wave {
            return ActorScriptCheckReserveActivationSequence::with_post_reserve_wave_clear(
                batches,
                wave_clear,
                None,
                Some(String::from(
                    "wave_advanced_before_post_reserve_wave_clear_advance_sleep",
                )),
                None,
                None,
                Some(String::from(
                    "wave_advanced_before_post_reserve_wave_clear_advance_sleep",
                )),
            );
        }
    }

    ActorScriptCheckReserveActivationSequence::with_post_reserve_wave_clear(
        batches,
        wave_clear,
        None,
        Some(String::from(
            "post_reserve_wave_clear_advance_sleep_not_observed",
        )),
        None,
        None,
        Some(String::from(
            "post_reserve_wave_clear_advance_sleep_not_observed",
        )),
    )
}

fn actor_script_check_to_post_reserve_next_playing(
    runtime: &mut ActorRuntimeAdapter,
    wave_sleep_frame: ActorFrame,
    wave_sleep_assist_steps: u32,
    previous_wave: u16,
) -> (
    Option<u32>,
    Option<ActorScriptCheckPlayingSummary>,
    Option<String>,
) {
    let mut frame = wave_sleep_frame;
    for step in 1..=ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT {
        let input = actor_script_check_assist_input(&frame);
        frame = runtime.step(input);
        let assist_steps = wave_sleep_assist_steps.saturating_add(step as u32);
        if frame.report.phase == Phase::Playing
            && frame.report.player_start.is_none()
            && frame.report.wave > previous_wave
        {
            return (
                Some(assist_steps),
                Some(actor_script_check_playing_summary(&frame)),
                None,
            );
        }
    }

    (
        Some(
            wave_sleep_assist_steps.saturating_add(ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT as u32),
        ),
        None,
        Some(String::from("post_reserve_next_playing_not_observed")),
    )
}

fn actor_script_check_reserve_total(reserve: crate::game::EnemyReserveSnapshot) -> u8 {
    reserve
        .landers
        .saturating_add(reserve.bombers)
        .saturating_add(reserve.pods)
        .saturating_add(reserve.mutants)
        .saturating_add(reserve.swarmers)
}

fn actor_script_check_spawned_counts(frame: &ActorFrame) -> ActorScriptCheckSpawnedCounts {
    let mut counts = ActorScriptCheckSpawnedCounts::default();
    for command in &frame.report.commands {
        match command {
            GameCommand::Spawn(SpawnRequest::Lander { .. }) => {
                counts.landers = counts.landers.saturating_add(1);
            }
            GameCommand::Spawn(SpawnRequest::Bomber { .. }) => {
                counts.bombers = counts.bombers.saturating_add(1);
            }
            GameCommand::Spawn(SpawnRequest::Pod { .. }) => {
                counts.pods = counts.pods.saturating_add(1);
            }
            GameCommand::Spawn(SpawnRequest::Mutant { .. }) => {
                counts.mutants = counts.mutants.saturating_add(1);
            }
            GameCommand::Spawn(SpawnRequest::Swarmer { .. }) => {
                counts.swarmers = counts.swarmers.saturating_add(1);
            }
            _ => {}
        }
    }
    counts
}

impl ActorScriptCheckSpawnedCounts {
    fn is_empty(&self) -> bool {
        self.landers == 0
            && self.bombers == 0
            && self.pods == 0
            && self.mutants == 0
            && self.swarmers == 0
    }
}

fn actor_script_check_next_wave_input(frame: &ActorFrame) -> ActorGameInput {
    actor_script_check_assist_input(frame)
}

fn actor_script_check_assist_input(frame: &ActorFrame) -> ActorGameInput {
    if frame.report.phase == Phase::Playing
        && frame.report.player_start.is_none()
        && !frame.state.world.enemies.is_empty()
    {
        return ActorGameInput {
            xyzzy: XyzzyMode {
                active: true,
                auto_fire: false,
                invincible: true,
                overlay_smart_bomb: true,
            },
            ..ActorGameInput::NONE
        };
    }

    ActorGameInput::NONE
}

#[cfg(all(not(test), not(coverage)))]
pub(crate) fn run_actor_live(
    input_profile: LiveInputProfile,
    audio_mode: LiveAudioMode,
    cmos_path: Option<&Path>,
    actor_script_path: Option<&Path>,
) -> anyhow::Result<()> {
    run_actor_live_app(input_profile, audio_mode, cmos_path, actor_script_path)
}

#[cfg(any(test, coverage))]
pub(crate) fn run_actor_live(
    _input_profile: LiveInputProfile,
    _audio_mode: LiveAudioMode,
    _cmos_path: Option<&Path>,
    actor_script_path: Option<&Path>,
) -> anyhow::Result<()> {
    let _runtime = actor_runtime_from_script_path(actor_script_path)?;
    Ok(())
}

#[cfg(all(not(test), not(coverage)))]
pub(crate) fn run_smoke(
    _input_profile: LiveInputProfile,
    _cmos_path: Option<&Path>,
) -> anyhow::Result<LiveSmokeReport> {
    let game_report = crate::game_smoke::default_smoke_report()?;
    let offscreen_report = pollster::block_on(render_offscreen_smoke())?;
    let mut report = LiveSmokeReport::from(game_report);
    report.saw_non_blank_frame = offscreen_report.non_blank_frames > 0;
    report.offscreen_wgpu_frames = offscreen_report.frames;
    report.offscreen_non_blank_frames = offscreen_report.non_blank_frames;
    report.offscreen_distinct_frame_signatures = offscreen_report.distinct_frame_signatures;
    report.offscreen_first_frame_signature = offscreen_report.first_frame_signature;
    report.offscreen_last_frame_signature = offscreen_report.last_frame_signature;
    report.validate_offscreen_wgpu()?;
    Ok(report)
}

#[cfg(all(not(test), not(coverage)))]
pub(crate) fn run_actor_wgpu_smoke() -> anyhow::Result<LiveSmokeReport> {
    let actor_report = crate::actor_smoke::default_smoke_report()?;
    let offscreen_report = pollster::block_on(render_actor_offscreen_smoke())?;
    let mut report = LiveSmokeReport::from(actor_report);
    report.saw_non_blank_frame = offscreen_report.non_blank_frames > 0;
    report.offscreen_wgpu_frames = offscreen_report.frames;
    report.offscreen_non_blank_frames = offscreen_report.non_blank_frames;
    report.offscreen_distinct_frame_signatures = offscreen_report.distinct_frame_signatures;
    report.offscreen_first_frame_signature = offscreen_report.first_frame_signature;
    report.offscreen_last_frame_signature = offscreen_report.last_frame_signature;
    report.validate_actor_offscreen_wgpu()?;
    Ok(report)
}

#[cfg(any(test, coverage))]
pub(crate) fn run_actor_wgpu_smoke() -> anyhow::Result<LiveSmokeReport> {
    crate::actor_smoke::default_smoke_report().map(LiveSmokeReport::from)
}

#[cfg(any(test, coverage))]
pub(crate) fn run_smoke(
    _input_profile: LiveInputProfile,
    _cmos_path: Option<&Path>,
) -> anyhow::Result<LiveSmokeReport> {
    crate::game_smoke::default_smoke_report().map(LiveSmokeReport::from)
}

#[cfg(all(not(test), not(coverage)))]
fn run_actor_live_app(
    input_profile: LiveInputProfile,
    audio_mode: LiveAudioMode,
    _cmos_path: Option<&Path>,
    actor_script_path: Option<&Path>,
) -> anyhow::Result<()> {
    let event_loop =
        winit::event_loop::EventLoop::new().context("creating actor wgpu event loop")?;
    let runtime = actor_runtime_from_script_path(actor_script_path)?;
    let mut app = ActorLiveApp::new(
        input_profile,
        LiveAudioRuntime::for_mode(audio_mode),
        runtime,
    );

    event_loop
        .run_app(&mut app)
        .context("running actor wgpu live event loop")?;
    if let Some(error) = app.take_error() {
        return Err(error);
    }
    Ok(())
}

#[cfg(all(not(test), not(coverage)))]
struct ActorLiveApp {
    input_profile: LiveInputProfile,
    runtime: ActorRuntimeAdapter,
    audio: LiveAudioRuntime,
    input: LiveInputState,
    accumulator: FixedStepAccumulator,
    frame_duration: Duration,
    last_tick: Instant,
    next_wake_at: Instant,
    latest_frame: Option<GameFrame>,
    quit_requested: bool,
    window: Option<Arc<Window>>,
    presenter: Option<WgpuScenePresenter>,
    error: Option<anyhow::Error>,
}

#[cfg(all(not(test), not(coverage)))]
impl ActorLiveApp {
    fn new(
        input_profile: LiveInputProfile,
        audio: LiveAudioRuntime,
        runtime: ActorRuntimeAdapter,
    ) -> Self {
        let now = Instant::now();
        let frame_duration = Duration::from_micros(FrameRate::CABINET.frame_duration_micros());
        let mut app = Self {
            input_profile,
            runtime,
            audio,
            input: LiveInputState::default(),
            accumulator: FixedStepAccumulator::new(FrameRate::CABINET),
            frame_duration,
            last_tick: now,
            next_wake_at: now + frame_duration,
            latest_frame: None,
            quit_requested: false,
            window: None,
            presenter: None,
            error: None,
        };
        app.step_one_frame();
        app
    }

    fn take_error(&mut self) -> Option<anyhow::Error> {
        self.error.take()
    }

    fn initialize_window(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        if self.window.is_some() {
            return Ok(());
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Defender Actor Runtime")
                        .with_inner_size(LogicalSize::new(
                            f64::from(INITIAL_WINDOW_WIDTH),
                            f64::from(INITIAL_WINDOW_HEIGHT),
                        )),
                )
                .context("creating actor wgpu window")?,
        );
        let presenter = pollster::block_on(WgpuScenePresenter::new(window.clone()))
            .context("initializing actor wgpu presenter")?;
        self.window = Some(window);
        self.presenter = Some(presenter);
        self.last_tick = Instant::now();
        self.next_wake_at = self.last_tick + self.frame_duration;
        Ok(())
    }

    fn handle_error(&mut self, event_loop: &ActiveEventLoop, error: anyhow::Error) {
        if self.error.is_none() {
            self.error = Some(error);
        }
        event_loop.exit();
    }

    fn window_matches(&self, window_id: WindowId) -> bool {
        self.window
            .as_ref()
            .is_some_and(|window| window.id() == window_id)
    }

    fn handle_keyboard_input(&mut self, event: &KeyEvent) {
        let control = live_control_from_winit(self.input_profile, event);
        self.input.observe_key_event_for_xyzzy(event, control);
        let Some(control) = control else {
            return;
        };
        let pressed = event.state == ElementState::Pressed;
        if control == LiveControl::Quit && pressed {
            self.quit_requested = true;
            return;
        }
        self.input.apply(control, pressed);
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        let Some((width, height)) = renderable_window_size(size) else {
            return;
        };
        if let Some(presenter) = &mut self.presenter {
            presenter.resize(width, height);
        }
    }

    fn step_one_frame(&mut self) {
        let input = self.input.drain_game_input();
        let xyzzy = self.input.drain_xyzzy_mode();
        let actor_frame = self.runtime.step_clean_input(input, xyzzy);
        let frame = actor_frame.game_frame();
        self.audio.submit_game_frame(&frame);
        self.latest_frame = Some(frame);
    }

    fn step_due_frames(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        self.accumulator
            .add_elapsed_micros(elapsed.as_micros().try_into().unwrap_or(u64::MAX));
        let due_steps = self.accumulator.consume_due_steps(MAX_STEPS_PER_TICK);

        for _ in 0..due_steps {
            self.step_one_frame();
        }

        let micros_until_next = FrameRate::CABINET
            .frame_duration_micros()
            .saturating_sub(self.accumulator.accumulated_micros())
            .max(1);
        self.next_wake_at = Instant::now() + Duration::from_micros(micros_until_next);
        due_steps > 0
    }

    fn draw_frame(&mut self) -> anyhow::Result<()> {
        let Some(frame) = &self.latest_frame else {
            return Ok(());
        };
        let Some(presenter) = &mut self.presenter else {
            return Ok(());
        };
        presenter.draw_scene(&frame.scene)
    }
}

#[cfg(all(not(test), not(coverage)))]
impl ApplicationHandler for ActorLiveApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.initialize_window(event_loop) {
            self.handle_error(event_loop, error);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if !self.window_matches(window_id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(&event);
                if self.quit_requested {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.pre_present_notify();
                }
                if let Err(error) = self.draw_frame() {
                    self.handle_error(event_loop, error);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.error.is_some() || self.quit_requested {
            event_loop.exit();
            return;
        }

        if self.step_due_frames()
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_wake_at));
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.presenter = None;
        self.window = None;
    }
}

#[cfg(all(not(test), not(coverage)))]
struct WgpuScenePresenter {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: NativeSceneRenderer,
    sprite_resources: Option<SpriteGpuResources>,
}

#[cfg(all(not(test), not(coverage)))]
impl WgpuScenePresenter {
    async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let (width, height) =
            renderable_window_size(size).unwrap_or((INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT));
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .context("creating clean wgpu surface")?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("requesting clean wgpu adapter")?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("defender clean wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("requesting clean wgpu device")?;
        let mut config = surface
            .get_default_config(&adapter, width, height)
            .ok_or_else(|| {
                anyhow::anyhow!("wgpu surface is not supported by the selected adapter")
            })?;
        config.present_mode = wgpu::PresentMode::Fifo;
        config.desired_maximum_frame_latency = 2;
        surface.configure(&device, &config);
        let settings = GpuRendererSettings {
            texture_format: config.format,
            present_mode: config.present_mode,
            alpha_mode: config.alpha_mode,
        };
        let renderer = NativeSceneRenderer::with_settings(Default::default(), settings);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            renderer,
            sprite_resources: None,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn draw_scene(&mut self, scene: &crate::renderer::RenderScene) -> anyhow::Result<()> {
        let target = SurfaceSize::new(self.config.width, self.config.height);
        let plan = self.renderer.prepare_for_target(scene, target);
        self.update_sprite_resources(&plan)?;

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                let size = self.window.inner_size();
                if let Some((width, height)) = renderable_window_size(size) {
                    self.resize(width, height);
                }
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                anyhow::bail!("wgpu surface validation error while acquiring frame")
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("defender clean wgpu frame encoder"),
            });
        encode_scene_render_pass(&mut encoder, &view, &plan, self.sprite_resources.as_ref());

        self.queue.submit([encoder.finish()]);
        surface_texture.present();
        Ok(())
    }

    fn update_sprite_resources(&mut self, plan: &SceneDrawPlan) -> anyhow::Result<()> {
        if plan.sprite_render_pass_encoder.is_none() {
            return Ok(());
        }
        if self.sprite_resources.is_none() {
            self.sprite_resources = Some(SpriteGpuResources::new(&self.device, &self.queue, plan)?);
        }
        let Some(resources) = &mut self.sprite_resources else {
            return Ok(());
        };
        let Some(bindings) = &plan.sprite_resource_bindings else {
            return Ok(());
        };
        self.queue.write_buffer(
            &resources.projection_buffer,
            0,
            &bindings.projection_upload.bytes,
        );
        if let Some(uploads) = &plan.sprite_buffer_uploads {
            resources.update_instances(&self.device, &self.queue, &uploads.instances);
        }
        Ok(())
    }
}

#[cfg(all(not(test), not(coverage)))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct OffscreenWgpuSmokeReport {
    frames: u32,
    non_blank_frames: u32,
    distinct_frame_signatures: usize,
    first_frame_signature: Option<u64>,
    last_frame_signature: Option<u64>,
}

#[cfg(all(not(test), not(coverage)))]
async fn render_offscreen_smoke() -> anyhow::Result<OffscreenWgpuSmokeReport> {
    let mut renderer = WgpuOffscreenRenderer::new().await?;
    let mut game = Game::new();
    let mut signatures = BTreeSet::new();
    let mut report = OffscreenWgpuSmokeReport::default();

    for _ in 0..crate::game_smoke::smoke_visual_warmup_frames() {
        game.step(GameInput::NONE);
    }

    for frame_index in 0..crate::game_smoke::smoke_frame_count() {
        let frame = game.step(crate::game_smoke::smoke_game_input(frame_index));
        let rendered = renderer.render_scene(&frame.scene)?;
        report.frames = report.frames.saturating_add(1);
        if rendered.non_blank {
            report.non_blank_frames = report.non_blank_frames.saturating_add(1);
        }
        report
            .first_frame_signature
            .get_or_insert(rendered.signature);
        report.last_frame_signature = Some(rendered.signature);
        signatures.insert(rendered.signature);
    }

    report.distinct_frame_signatures = signatures.len();
    Ok(report)
}

#[cfg(all(not(test), not(coverage)))]
async fn render_actor_offscreen_smoke() -> anyhow::Result<OffscreenWgpuSmokeReport> {
    let mut renderer = WgpuOffscreenRenderer::new().await?;
    let mut runtime = ActorRuntimeAdapter::new();
    let mut signatures = BTreeSet::new();
    let mut report = OffscreenWgpuSmokeReport::default();

    for frame_index in 0..crate::actor_smoke::smoke_frame_count() {
        let frame = runtime.step(crate::actor_smoke::smoke_actor_input(frame_index));
        let rendered = renderer.render_scene(&frame.scene)?;
        report.frames = report.frames.saturating_add(1);
        if rendered.non_blank {
            report.non_blank_frames = report.non_blank_frames.saturating_add(1);
        }
        report
            .first_frame_signature
            .get_or_insert(rendered.signature);
        report.last_frame_signature = Some(rendered.signature);
        signatures.insert(rendered.signature);
    }

    report.distinct_frame_signatures = signatures.len();
    Ok(report)
}

#[cfg(all(not(test), not(coverage)))]
struct WgpuOffscreenRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: NativeSceneRenderer,
    sprite_resources: Option<SpriteGpuResources>,
}

#[cfg(all(not(test), not(coverage)))]
impl WgpuOffscreenRenderer {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    async fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .context("requesting clean offscreen wgpu adapter")?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("defender clean offscreen wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("requesting clean offscreen wgpu device")?;
        let renderer = NativeSceneRenderer::with_settings(
            Default::default(),
            GpuRendererSettings {
                texture_format: Self::TEXTURE_FORMAT,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
            },
        );

        Ok(Self {
            device,
            queue,
            renderer,
            sprite_resources: None,
        })
    }

    fn render_scene(
        &mut self,
        scene: &crate::renderer::RenderScene,
    ) -> anyhow::Result<RenderedOffscreenFrame> {
        if scene.surface.is_empty() {
            anyhow::bail!("cannot render empty offscreen scene");
        }

        let plan = self.renderer.prepare(scene);
        self.update_sprite_resources(&plan)?;

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("defender.offscreen.live_smoke.texture"),
            size: wgpu::Extent3d {
                width: scene.surface.width,
                height: scene.surface.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let readback = OffscreenReadbackLayout::for_surface(scene.surface)?;
        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("defender.offscreen.live_smoke.readback"),
            size: readback.buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("defender.offscreen.live_smoke.encoder"),
            });
        encode_scene_render_pass(&mut encoder, &view, &plan, self.sprite_resources.as_ref());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(readback.padded_bytes_per_row),
                    rows_per_image: Some(scene.surface.height),
                },
            },
            wgpu::Extent3d {
                width: scene.surface.width,
                height: scene.surface.height,
                depth_or_array_layers: 1,
            },
        );

        let (sender, receiver) = mpsc::channel();
        encoder.map_buffer_on_submit(
            &readback_buffer,
            wgpu::MapMode::Read,
            0..readback.buffer_size,
            move |result| {
                let _ = sender.send(result);
            },
        );
        self.queue.submit([encoder.finish()]);
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .context("polling clean offscreen wgpu readback")?;
        receiver
            .recv()
            .context("waiting for clean offscreen wgpu readback")?
            .context("mapping clean offscreen wgpu readback")?;

        let mapped = readback_buffer.slice(..).get_mapped_range();
        let pixels = readback.unpadded_pixels(&mapped);
        drop(mapped);
        readback_buffer.unmap();

        Ok(RenderedOffscreenFrame {
            surface: scene.surface,
            signature: rendered_rgba_signature(scene.surface, &pixels),
            non_blank: rendered_rgba_is_non_blank(&pixels),
        })
    }

    fn update_sprite_resources(&mut self, plan: &SceneDrawPlan) -> anyhow::Result<()> {
        if plan.sprite_render_pass_encoder.is_none() {
            return Ok(());
        }
        if self.sprite_resources.is_none() {
            self.sprite_resources = Some(SpriteGpuResources::new(&self.device, &self.queue, plan)?);
        }
        let Some(resources) = &mut self.sprite_resources else {
            return Ok(());
        };
        let Some(bindings) = &plan.sprite_resource_bindings else {
            return Ok(());
        };
        self.queue.write_buffer(
            &resources.projection_buffer,
            0,
            &bindings.projection_upload.bytes,
        );
        if let Some(uploads) = &plan.sprite_buffer_uploads {
            resources.update_instances(&self.device, &self.queue, &uploads.instances);
        }
        Ok(())
    }
}

#[cfg(all(not(test), not(coverage)))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RenderedOffscreenFrame {
    surface: SurfaceSize,
    signature: u64,
    non_blank: bool,
}

#[cfg(all(not(test), not(coverage)))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OffscreenReadbackLayout {
    unpadded_bytes_per_row: u32,
    padded_bytes_per_row: u32,
    buffer_size: wgpu::BufferAddress,
    surface: SurfaceSize,
}

#[cfg(all(not(test), not(coverage)))]
impl OffscreenReadbackLayout {
    fn for_surface(surface: SurfaceSize) -> anyhow::Result<Self> {
        let unpadded_bytes_per_row = surface
            .width
            .checked_mul(4)
            .context("clean offscreen readback row byte length overflow")?;
        let padded_bytes_per_row = align_copy_bytes_per_row(unpadded_bytes_per_row);
        let buffer_size = u64::from(padded_bytes_per_row)
            .checked_mul(u64::from(surface.height))
            .context("clean offscreen readback buffer length overflow")?;

        Ok(Self {
            unpadded_bytes_per_row,
            padded_bytes_per_row,
            buffer_size,
            surface,
        })
    }

    fn unpadded_pixels(&self, mapped: &[u8]) -> Vec<u8> {
        let unpadded_bytes_per_row = self.unpadded_bytes_per_row as usize;
        let padded_bytes_per_row = self.padded_bytes_per_row as usize;
        let mut pixels = Vec::with_capacity(self.surface.rgba_len().unwrap_or_default());

        for row in 0..self.surface.height as usize {
            let row_start = row * padded_bytes_per_row;
            let row_end = row_start + unpadded_bytes_per_row;
            pixels.extend_from_slice(&mapped[row_start..row_end]);
        }

        pixels
    }
}

#[cfg(all(not(test), not(coverage)))]
fn align_copy_bytes_per_row(bytes_per_row: u32) -> u32 {
    bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
}

#[cfg(all(not(test), not(coverage)))]
fn rendered_rgba_is_non_blank(pixels: &[u8]) -> bool {
    pixels.chunks_exact(4).any(|pixel| pixel != [0, 0, 0, 0])
}

#[cfg(all(not(test), not(coverage)))]
fn rendered_rgba_signature(surface: SurfaceSize, pixels: &[u8]) -> u64 {
    let mut signature = 0xCBF2_9CE4_8422_2325_u64;
    signature = fnv1a_mix_u64(signature, u64::from(surface.width));
    signature = fnv1a_mix_u64(signature, u64::from(surface.height));
    for byte in pixels {
        signature ^= u64::from(*byte);
        signature = signature.wrapping_mul(0x0000_0100_0000_01B3);
    }
    signature
}

#[cfg(all(not(test), not(coverage)))]
fn fnv1a_mix_u64(mut signature: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        signature ^= u64::from(byte);
        signature = signature.wrapping_mul(0x0000_0100_0000_01B3);
    }
    signature
}

#[cfg(all(not(test), not(coverage)))]
struct SpriteGpuResources {
    pipeline: wgpu::RenderPipeline,
    projection_buffer: wgpu::Buffer,
    projection_bind_group: wgpu::BindGroup,
    atlas_bind_group: wgpu::BindGroup,
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
    instance_buffer: Option<wgpu::Buffer>,
    instance_buffer_size: wgpu::BufferAddress,
}

#[cfg(all(not(test), not(coverage)))]
impl SpriteGpuResources {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        plan: &SceneDrawPlan,
    ) -> anyhow::Result<Self> {
        let bindings = plan
            .sprite_resource_bindings
            .as_ref()
            .context("sprite plan missing resource bindings")?;
        let layout = plan
            .sprite_pipeline_layout
            .as_ref()
            .context("sprite plan missing pipeline layout")?;
        let descriptor = plan
            .sprite_render_pipeline_descriptor
            .as_ref()
            .context("sprite plan missing render pipeline descriptor")?;
        let pipeline_plan = plan
            .sprite_pipeline
            .as_ref()
            .context("sprite plan missing pipeline")?;
        let uploads = plan
            .sprite_buffer_uploads
            .as_ref()
            .context("sprite plan missing buffer uploads")?;

        let projection_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(bindings.projection_layout.label),
            entries: &bindings.projection_layout.entries,
        });
        let atlas_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(bindings.atlas_layout.label),
            entries: &bindings.atlas_layout.entries,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(layout.label),
            bind_group_layouts: &[Some(&projection_layout), Some(&atlas_layout)],
            immediate_size: layout.immediate_size,
        });

        let projection_buffer = create_buffer(device, &bindings.projection_upload);
        queue.write_buffer(&projection_buffer, 0, &bindings.projection_upload.bytes);
        let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("defender.sprite.scene_projection.bind_group"),
            layout: &projection_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
        });

        let atlas_texture = device.create_texture(&bindings.atlas_upload.texture_descriptor());
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bindings.atlas_upload.bytes,
            bindings.atlas_upload.copy_layout(),
            bindings.atlas_upload.extent(),
        );
        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(bindings.atlas_sampler.label),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..wgpu::SamplerDescriptor::default()
        });
        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("defender.sprite.atlas.bind_group"),
            layout: &atlas_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(pipeline_plan.shader.shader_module_descriptor());
        let color_targets = descriptor.color_targets();
        let vertex_buffers = descriptor.vertex_buffer_layouts();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(descriptor.label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some(descriptor.vertex_entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &vertex_buffers,
            },
            primitive: descriptor.primitive,
            depth_stencil: None,
            multisample: descriptor.multisample,
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(descriptor.fragment_entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &color_targets,
            }),
            multiview_mask: None,
            cache: None,
        });

        let quad_vertex_buffer =
            create_buffer_from_sprite_upload(device, queue, &uploads.quad_vertices);
        let quad_index_buffer =
            create_buffer_from_sprite_upload(device, queue, &uploads.quad_indices);
        let mut resources = Self {
            pipeline,
            projection_buffer,
            projection_bind_group,
            atlas_bind_group,
            quad_vertex_buffer,
            quad_index_buffer,
            instance_buffer: None,
            instance_buffer_size: 0,
        };
        resources.update_instances(device, queue, &uploads.instances);
        Ok(resources)
    }

    fn update_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        upload: &SpriteBufferUpload,
    ) {
        if upload.byte_len == 0 {
            self.instance_buffer = None;
            self.instance_buffer_size = 0;
            return;
        }
        if self.instance_buffer_size < upload.byte_len {
            self.instance_buffer = Some(create_empty_buffer(
                device,
                upload.label,
                upload.usage,
                upload.byte_len,
            ));
            self.instance_buffer_size = upload.byte_len;
        }
        if let Some(buffer) = &self.instance_buffer {
            queue.write_buffer(buffer, 0, &upload.bytes);
        }
    }
}

#[cfg(all(not(test), not(coverage)))]
fn create_buffer(
    device: &wgpu::Device,
    upload: &crate::renderer::SceneProjectionUniformUpload,
) -> wgpu::Buffer {
    create_empty_buffer(device, upload.label, upload.usage, upload.byte_len)
}

#[cfg(all(not(test), not(coverage)))]
fn create_buffer_from_sprite_upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    upload: &SpriteBufferUpload,
) -> wgpu::Buffer {
    let buffer = create_empty_buffer(device, upload.label, upload.usage, upload.byte_len);
    queue.write_buffer(&buffer, 0, &upload.bytes);
    buffer
}

#[cfg(all(not(test), not(coverage)))]
fn create_empty_buffer(
    device: &wgpu::Device,
    label: &'static str,
    usage: wgpu::BufferUsages,
    byte_len: wgpu::BufferAddress,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: byte_len.max(1),
        usage,
        mapped_at_creation: false,
    })
}

#[cfg(all(not(test), not(coverage)))]
fn encode_scene_render_pass(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    plan: &SceneDrawPlan,
    sprite_resources: Option<&SpriteGpuResources>,
) {
    let color_attachment = Some(wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(plan.gpu_pass.clear_color),
            store: wgpu::StoreOp::Store,
        },
    });
    let color_attachments = [color_attachment];
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("defender clean sprite render pass"),
        color_attachments: &color_attachments,
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    if let Some(viewport) = plan.gpu_pass.viewport {
        pass.set_viewport(
            viewport.x,
            viewport.y,
            viewport.width,
            viewport.height,
            viewport.min_depth,
            viewport.max_depth,
        );
    }
    if let (Some(resources), Some(encoder_plan)) =
        (sprite_resources, &plan.sprite_render_pass_encoder)
    {
        encode_sprite_commands(&mut pass, resources, encoder_plan);
    }
}

#[cfg(all(not(test), not(coverage)))]
fn encode_sprite_commands<'pass>(
    pass: &mut wgpu::RenderPass<'pass>,
    resources: &'pass SpriteGpuResources,
    encoder_plan: &'pass crate::renderer::SpriteRenderPassEncoderPlan,
) {
    for command in &encoder_plan.commands {
        match command {
            SpriteRenderPassEncoderCommand::SetPipeline { .. } => {
                pass.set_pipeline(&resources.pipeline);
            }
            SpriteRenderPassEncoderCommand::SetBindGroup {
                role, group_index, ..
            } => {
                let bind_group = match role {
                    SpriteBindGroupRole::SceneProjection => &resources.projection_bind_group,
                    SpriteBindGroupRole::SpriteAtlas => &resources.atlas_bind_group,
                };
                pass.set_bind_group(*group_index, bind_group, &[]);
            }
            SpriteRenderPassEncoderCommand::SetVertexBuffer {
                role,
                slot,
                byte_offset,
                byte_len,
            } => match role {
                SpriteBufferRole::QuadVertices => pass.set_vertex_buffer(
                    *slot,
                    resources
                        .quad_vertex_buffer
                        .slice(*byte_offset..byte_offset.saturating_add(*byte_len)),
                ),
                SpriteBufferRole::Instances => {
                    if let Some(buffer) = &resources.instance_buffer {
                        pass.set_vertex_buffer(
                            *slot,
                            buffer.slice(*byte_offset..byte_offset.saturating_add(*byte_len)),
                        );
                    }
                }
                SpriteBufferRole::QuadIndices => {}
            },
            SpriteRenderPassEncoderCommand::SetIndexBuffer {
                index_format,
                byte_offset,
                byte_len,
                ..
            } => pass.set_index_buffer(
                resources
                    .quad_index_buffer
                    .slice(*byte_offset..byte_offset.saturating_add(*byte_len)),
                *index_format,
            ),
            SpriteRenderPassEncoderCommand::DrawIndexed { draw } => {
                pass.draw_indexed(
                    draw.indices.clone(),
                    draw.base_vertex,
                    draw.instances.clone(),
                );
            }
        }
    }
}

#[cfg(all(not(test), not(coverage)))]
fn renderable_window_size(size: PhysicalSize<u32>) -> Option<(u32, u32)> {
    if size.width == 0 || size.height == 0 {
        None
    } else {
        Some((size.width, size.height))
    }
}

#[cfg(any(test, all(not(test), not(coverage))))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiveControl {
    Coin,
    StartOne,
    StartTwo,
    AltitudeUp,
    AltitudeDown,
    Reverse,
    Thrust,
    Fire,
    SmartBomb,
    Hyperspace,
    ServiceAutoUp,
    ServiceAdvance,
    HighScoreReset,
    HighScoreBackspace,
    HighScoreInitial(char),
    Quit,
}

#[cfg(any(test, all(not(test), not(coverage))))]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct LiveInputState {
    coin: bool,
    start_one: bool,
    start_two: bool,
    altitude_up: bool,
    altitude_down: bool,
    reverse: bool,
    thrust: bool,
    fire: bool,
    smart_bomb: bool,
    hyperspace: bool,
    service_auto_up: bool,
    service_advance: bool,
    high_score_reset: bool,
    high_score_initial: Option<char>,
    high_score_backspace: bool,
    xyzzy: XyzzyController,
    overlay_smart_bomb: bool,
}

#[cfg(any(test, all(not(test), not(coverage))))]
impl LiveInputState {
    #[cfg(all(not(test), not(coverage)))]
    fn observe_key_event_for_xyzzy(&mut self, event: &KeyEvent, control: Option<LiveControl>) {
        if event.state != ElementState::Pressed {
            return;
        }
        if matches!(control, Some(LiveControl::HighScoreInitial(_))) {
            return;
        }
        let Some(character) = logical_key_character(&event.logical_key) else {
            return;
        };
        self.ingest_xyzzy_character(character);
    }

    fn ingest_xyzzy_character(&mut self, character: char) {
        self.xyzzy.ingest(character);
        if self.xyzzy.active() {
            match character.to_ascii_lowercase() {
                'f' => self.xyzzy.toggle_auto_fire(),
                'g' => self.xyzzy.toggle_invincible(),
                _ => {}
            }
        }
    }

    fn apply(&mut self, control: LiveControl, pressed: bool) {
        match control {
            LiveControl::Coin => self.coin |= pressed,
            LiveControl::StartOne => self.start_one |= pressed,
            LiveControl::StartTwo => self.start_two |= pressed,
            LiveControl::AltitudeUp => self.altitude_up = pressed,
            LiveControl::AltitudeDown => self.altitude_down = pressed,
            LiveControl::Reverse => self.reverse = pressed,
            LiveControl::Thrust => self.thrust = pressed,
            LiveControl::Fire => self.fire = pressed,
            LiveControl::SmartBomb => {
                self.smart_bomb = pressed;
                if pressed && self.xyzzy.active() {
                    self.overlay_smart_bomb = true;
                }
            }
            LiveControl::Hyperspace => self.hyperspace = pressed,
            LiveControl::ServiceAutoUp => self.service_auto_up = pressed,
            LiveControl::ServiceAdvance => self.service_advance |= pressed,
            LiveControl::HighScoreReset => self.high_score_reset |= pressed,
            LiveControl::HighScoreBackspace => self.high_score_backspace |= pressed,
            LiveControl::HighScoreInitial(value) => {
                if pressed {
                    self.high_score_initial = Some(value);
                    self.ingest_xyzzy_character(value);
                }
            }
            LiveControl::Quit => {}
        }
    }

    fn drain_game_input(&mut self) -> GameInput {
        GameInput {
            coin: take_bool(&mut self.coin),
            coin_two: false,
            coin_three: false,
            start_one: take_bool(&mut self.start_one),
            start_two: take_bool(&mut self.start_two),
            altitude_up: self.altitude_up,
            altitude_down: self.altitude_down,
            reverse: self.reverse,
            thrust: self.thrust,
            fire: self.fire,
            smart_bomb: self.smart_bomb,
            hyperspace: self.hyperspace,
            service_auto_up: self.service_auto_up,
            service_advance: take_bool(&mut self.service_advance),
            high_score_reset: take_bool(&mut self.high_score_reset),
            high_score_initial: self.high_score_initial.take(),
            high_score_backspace: take_bool(&mut self.high_score_backspace),
            tilt: false,
        }
    }

    fn drain_xyzzy_mode(&mut self) -> XyzzyMode {
        self.xyzzy.mode(take_bool(&mut self.overlay_smart_bomb))
    }
}

#[cfg(all(not(test), not(coverage)))]
fn logical_key_character(key: &Key) -> Option<char> {
    match key {
        Key::Character(text) => single_character(text),
        _ => None,
    }
}

#[cfg(any(test, all(not(test), not(coverage))))]
fn take_bool(value: &mut bool) -> bool {
    let taken = *value;
    *value = false;
    taken
}

#[cfg(all(not(test), not(coverage)))]
fn live_control_from_winit(profile: LiveInputProfile, event: &KeyEvent) -> Option<LiveControl> {
    physical_control(profile, &event.physical_key)
        .or_else(|| logical_control(profile, &event.logical_key))
}

#[cfg(all(not(test), not(coverage)))]
fn physical_control(profile: LiveInputProfile, physical_key: &PhysicalKey) -> Option<LiveControl> {
    let PhysicalKey::Code(code) = physical_key else {
        return None;
    };

    match code {
        KeyCode::Escape => Some(LiveControl::Quit),
        KeyCode::Digit5 | KeyCode::Numpad5 => Some(LiveControl::Coin),
        KeyCode::Digit1 | KeyCode::Numpad1 => Some(LiveControl::StartOne),
        KeyCode::Digit2 | KeyCode::Numpad2 => Some(LiveControl::StartTwo),
        KeyCode::F1 => Some(LiveControl::ServiceAutoUp),
        KeyCode::F2 => Some(LiveControl::ServiceAdvance),
        KeyCode::F3 => Some(LiveControl::HighScoreReset),
        KeyCode::Backspace => Some(LiveControl::HighScoreBackspace),
        _ => gameplay_physical_control(profile, *code),
    }
}

#[cfg(all(not(test), not(coverage)))]
fn gameplay_physical_control(profile: LiveInputProfile, code: KeyCode) -> Option<LiveControl> {
    match profile {
        LiveInputProfile::Planetoid => match code {
            KeyCode::Enter | KeyCode::NumpadEnter => Some(LiveControl::Fire),
            KeyCode::ShiftLeft | KeyCode::ShiftRight => Some(LiveControl::Thrust),
            KeyCode::KeyA => Some(LiveControl::AltitudeUp),
            KeyCode::KeyZ => Some(LiveControl::AltitudeDown),
            KeyCode::Space => Some(LiveControl::Reverse),
            KeyCode::Tab => Some(LiveControl::SmartBomb),
            KeyCode::KeyH => Some(LiveControl::Hyperspace),
            _ => None,
        },
        LiveInputProfile::Cabinet | LiveInputProfile::Test => match code {
            KeyCode::KeyF => Some(LiveControl::Fire),
            KeyCode::KeyT => Some(LiveControl::Thrust),
            KeyCode::ArrowUp => Some(LiveControl::AltitudeUp),
            KeyCode::ArrowDown => Some(LiveControl::AltitudeDown),
            KeyCode::KeyR => Some(LiveControl::Reverse),
            KeyCode::KeyB => Some(LiveControl::SmartBomb),
            KeyCode::KeyH => Some(LiveControl::Hyperspace),
            _ => None,
        },
    }
}

#[cfg(all(not(test), not(coverage)))]
fn logical_control(profile: LiveInputProfile, logical_key: &Key) -> Option<LiveControl> {
    match logical_key {
        Key::Named(NamedKey::Escape) => Some(LiveControl::Quit),
        Key::Named(NamedKey::Enter) => {
            (profile == LiveInputProfile::Planetoid).then_some(LiveControl::Fire)
        }
        Key::Named(NamedKey::Tab) => {
            (profile == LiveInputProfile::Planetoid).then_some(LiveControl::SmartBomb)
        }
        Key::Named(NamedKey::Backspace) => Some(LiveControl::HighScoreBackspace),
        Key::Named(NamedKey::ArrowUp) => {
            (profile != LiveInputProfile::Planetoid).then_some(LiveControl::AltitudeUp)
        }
        Key::Named(NamedKey::ArrowDown) => {
            (profile != LiveInputProfile::Planetoid).then_some(LiveControl::AltitudeDown)
        }
        Key::Named(NamedKey::Shift) => {
            (profile == LiveInputProfile::Planetoid).then_some(LiveControl::Thrust)
        }
        Key::Named(NamedKey::F1) => Some(LiveControl::ServiceAutoUp),
        Key::Named(NamedKey::F2) => Some(LiveControl::ServiceAdvance),
        Key::Named(NamedKey::F3) => Some(LiveControl::HighScoreReset),
        Key::Character(text) => character_control(profile, text),
        _ => None,
    }
}

#[cfg(all(not(test), not(coverage)))]
fn character_control(profile: LiveInputProfile, text: &str) -> Option<LiveControl> {
    let value = single_character(text)?;
    match value.to_ascii_lowercase() {
        '1' => Some(LiveControl::StartOne),
        '2' => Some(LiveControl::StartTwo),
        '5' => Some(LiveControl::Coin),
        'a' if profile == LiveInputProfile::Planetoid => Some(LiveControl::AltitudeUp),
        'z' if profile == LiveInputProfile::Planetoid => Some(LiveControl::AltitudeDown),
        ' ' if profile == LiveInputProfile::Planetoid => Some(LiveControl::Reverse),
        'h' => Some(LiveControl::Hyperspace),
        'f' if profile != LiveInputProfile::Planetoid => Some(LiveControl::Fire),
        't' if profile != LiveInputProfile::Planetoid => Some(LiveControl::Thrust),
        'r' if profile != LiveInputProfile::Planetoid => Some(LiveControl::Reverse),
        'b' if profile != LiveInputProfile::Planetoid => Some(LiveControl::SmartBomb),
        'a'..='z' => Some(LiveControl::HighScoreInitial(value.to_ascii_uppercase())),
        _ => None,
    }
}

#[cfg(all(not(test), not(coverage)))]
fn single_character(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let value = chars.next()?;
    chars.next().is_none().then_some(value)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::GameInput;

    use super::{
        LiveInputState, LiveSmokeReport, actor_runtime_from_script_path, run_actor_live,
        run_actor_script_check, run_actor_wgpu_smoke, run_smoke,
    };

    #[test]
    fn live_smoke_report_formats_current_cli_output() {
        let report = LiveSmokeReport {
            frame_source: "clean_game",
            legacy_presenter_used: false,
            window_created: false,
            rendered_frames: 3,
            first_frame_size: Some((640, 480)),
            distinct_frame_signatures: 2,
            saw_non_blank_frame: true,
            saw_attract: true,
            saw_credit: true,
            saw_playing: true,
            attract_visual_frames: 1,
            credit_visual_frames: 1,
            playing_visual_frames: 1,
            attract_distinct_frame_signatures: 1,
            credit_distinct_frame_signatures: 1,
            playing_distinct_frame_signatures: 1,
            clean_game_frames: 3,
            actor_frames: 0,
            sprite_frames: 3,
            sprite_instances: 12,
            sprite_draw_commands: 4,
            temporary_raster_frames: 0,
            temporary_raster_commands: 0,
            offscreen_wgpu_frames: 3,
            offscreen_non_blank_frames: 3,
            offscreen_distinct_frame_signatures: 2,
            offscreen_first_frame_signature: Some(0x1234_ABCD),
            offscreen_last_frame_signature: Some(0xABCD_1234),
            injected_inputs: vec![String::from("coin"), String::from("start_one")],
            clean_exit: true,
        };

        assert_eq!(
            report.to_text(),
            concat!(
                "wgpu live smoke passed\n",
                "  frame_source: clean_game\n",
                "  legacy_presenter_used: false\n",
                "  window_created: false\n",
                "  rendered_frames: 3\n",
                "  first_frame_size: 640x480\n",
                "  distinct_frame_signatures: 2\n",
                "  saw_non_blank_frame: true\n",
                "  saw_attract: true (visual_frames: 1, visual_signatures: 1)\n",
                "  saw_credit: true (visual_frames: 1, visual_signatures: 1)\n",
                "  saw_playing: true (visual_frames: 1, visual_signatures: 1)\n",
                "  clean_game_frames: 3\n",
                "  actor_frames: 0\n",
                "  sprite_frames: 3\n",
                "  sprite_instances: 12\n",
                "  sprite_draw_commands: 4\n",
                "  temporary_raster_frames: 0\n",
                "  temporary_raster_commands: 0\n",
                "  offscreen_wgpu_frames: 3\n",
                "  offscreen_non_blank_frames: 3\n",
                "  offscreen_distinct_frame_signatures: 2\n",
                "  offscreen_first_frame_signature: 000000001234abcd\n",
                "  offscreen_last_frame_signature: 00000000abcd1234\n",
                "  injected_inputs: coin,start_one\n",
                "  clean_exit: true\n",
            )
        );
    }

    #[test]
    fn live_smoke_uses_clean_game_frame_source() {
        let report = run_smoke(super::LiveInputProfile::Test, None).expect("clean live smoke");

        assert_eq!(report.frame_source, "clean_game");
        assert!(!report.legacy_presenter_used);
        assert!(!report.window_created);
        assert_eq!(report.clean_game_frames, report.rendered_frames);
        assert_eq!(report.actor_frames, 0);
        assert_eq!(report.temporary_raster_frames, 0);
        assert_eq!(report.temporary_raster_commands, 0);
        assert!(report.sprite_frames > 0);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.saw_attract);
        assert!(report.saw_credit);
        assert!(report.saw_playing);
    }

    #[test]
    fn actor_wgpu_smoke_uses_actor_frame_source() {
        let report = run_actor_wgpu_smoke().expect("actor wgpu smoke");

        assert_eq!(report.frame_source, "actor_game");
        assert!(!report.legacy_presenter_used);
        assert!(!report.window_created);
        assert_eq!(report.clean_game_frames, 0);
        assert_eq!(report.actor_frames, report.rendered_frames);
        assert_eq!(report.temporary_raster_frames, 0);
        assert_eq!(report.temporary_raster_commands, 0);
        assert!(report.sprite_frames > 0);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.saw_attract);
        assert!(report.saw_credit);
        assert!(report.saw_playing);
    }

    #[test]
    fn actor_live_entrypoint_is_available_under_tests() {
        run_actor_live(
            super::LiveInputProfile::Test,
            crate::audio::LiveAudioMode::Null,
            None,
            None,
        )
        .expect("actor live entrypoint should be wired");
    }

    #[test]
    fn actor_live_entrypoint_loads_sectioned_script_file_under_tests() {
        let path = write_actor_script_file(
            "live-entrypoint",
            "\
            [attract]\n\
            text 1 forever 12 20 LIVE SCRIPT\n\
            [behavior]\n\
            kind lander lander_mode drift\n\
            [wave]\n\
            name live script waves\n\
            wave 1\n\
            lander 80 214\n\
            human 100 214\n",
        );

        run_actor_live(
            super::LiveInputProfile::Test,
            crate::audio::LiveAudioMode::Null,
            None,
            Some(&path),
        )
        .expect("actor live entrypoint should parse script file under tests");

        let mut runtime = actor_runtime_from_script_path(Some(&path))
            .expect("actor script path should build a runtime");
        let frame = runtime.step(crate::actor_game::GameInput::NONE);

        assert_eq!(
            runtime.driver().script_manifest().wave_script.name,
            "live script waves"
        );
        assert!(frame.report.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("LIVE SCRIPT")
                && draw.position == crate::actor_game::Point::new(12, 20)
        }));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn actor_script_file_loader_reports_read_and_parse_context() {
        let missing = unique_actor_script_path("missing-script");
        let read_error = match actor_runtime_from_script_path(Some(&missing)) {
            Ok(_) => panic!("missing script should fail"),
            Err(error) => error,
        };
        assert!(
            read_error
                .to_string()
                .contains("reading actor driver script")
        );
        assert!(
            read_error
                .to_string()
                .contains(&missing.display().to_string())
        );

        let malformed = write_actor_script_file(
            "malformed-script",
            "\
            [attract]\n\
            text 1 forever 12 20 BAD SCRIPT\n\
            [behavior]\n\
            kind lander lander_mode drift\n\
            [wave]\n\
            lander 80 214\n",
        );
        let parse_error = match actor_runtime_from_script_path(Some(&malformed)) {
            Ok(_) => panic!("malformed script should fail"),
            Err(error) => error,
        };
        assert!(
            parse_error
                .to_string()
                .contains("parsing actor driver script")
        );
        assert!(format!("{parse_error:#}").contains("actor driver wave script line 6"));

        let _ = fs::remove_file(malformed);
    }

    #[test]
    fn actor_script_check_report_summarizes_example_driver_script() {
        let path = std::path::Path::new("examples/actor-custom-attract.script");
        let report = run_actor_script_check(path).expect("example actor script should check");

        assert_eq!(report.path, path.display().to_string());
        assert_eq!(report.attract_events, 8);
        let attract_cycle = report
            .attract_cycle
            .as_ref()
            .expect("example script should declare a checked attract cycle");
        assert_eq!(attract_cycle.cycle_steps, 96);
        assert_eq!(attract_cycle.sampled_steps, 96);
        assert_eq!(attract_cycle.attract_frames, 96);
        assert_eq!(attract_cycle.non_attract_frames, 0);
        assert_eq!(attract_cycle.draw_commands, 193);
        assert_eq!(attract_cycle.scene_sprites, 19009);
        assert!(attract_cycle.saw_williams_reveal);
        assert!(attract_cycle.saw_defender_coalescence);
        assert!(attract_cycle.saw_hall_of_fame);
        assert!(attract_cycle.saw_scoring_surface);
        assert!(attract_cycle.saw_final_scoring_label);
        assert!(attract_cycle.saw_cycle_return);
        assert!(report.attract_cycle_unavailable_reason.is_none());
        assert_eq!(report.behavior_kind_profiles, 2);
        assert_eq!(report.behavior_actor_profiles, 0);
        assert_eq!(report.wave_profiles, 1);
        assert_eq!(report.first_frame_phase, "Attract");
        assert_eq!(report.first_frame_draws, 1);
        assert_eq!(report.first_playing_wave, 1);
        assert_eq!(report.first_playing_wave_size, 5);
        assert_eq!(report.first_playing_source_landers, 15);
        assert_eq!(report.first_playing_source_bombers, 0);
        assert_eq!(report.first_playing_source_pods, 0);
        assert_eq!(report.first_playing_source_mutants, 0);
        assert_eq!(report.first_playing_source_swarmers, 0);
        assert_eq!(report.first_playing_world_enemies, 2);
        assert_eq!(report.first_playing_world_humans, 2);
        assert_eq!(report.first_playing_reserve_landers, 0);
        assert_eq!(report.first_playing_reserve_bombers, 0);
        assert_eq!(report.first_playing_reserve_pods, 0);
        assert_eq!(report.first_playing_reserve_mutants, 0);
        assert_eq!(report.first_playing_reserve_swarmers, 0);
        assert_eq!(report.first_playing_source_background_left, 0);
        assert_eq!(report.first_playing_source_rng_seed, Some(0xbe));
        assert_eq!(report.first_playing_source_rng_hseed, Some(0xb1));
        assert_eq!(report.first_playing_source_rng_lseed, Some(0x06));
        assert!(report.first_playing_player_takes_enemy_collision_damage);
        assert_eq!(report.first_playing_player_laser_cooldown_steps, 6);
        assert_eq!(report.first_playing_lander_mode, "drift");
        assert_eq!(report.first_playing_lander_seek_speed, 1);
        assert_eq!(report.first_playing_lander_drift_speed, 3);
        assert_eq!(report.first_playing_lander_fire_period_steps, 96);
        assert_eq!(report.first_playing_mutant_mode, "chase_player");
        assert_eq!(report.first_playing_bomber_mode, "drift");
        assert_eq!(report.first_playing_pod_mode, "drift");
        assert_eq!(report.first_playing_swarmer_mode, "chase_player");
        assert_eq!(report.first_playing_baiter_mode, "chase_player");
        assert_eq!(report.first_playing_swarmer_fire_period_steps, 58);
        assert_eq!(report.first_playing_baiter_fire_period_steps, 42);
        assert_eq!(report.next_playing_assist_steps, Some(140));
        let next_playing = report
            .next_playing
            .as_ref()
            .expect("example script should reach the second wave");
        assert_eq!(next_playing.wave, 2);
        assert_eq!(next_playing.wave_size, 5);
        assert_eq!(next_playing.source_landers, 20);
        assert_eq!(next_playing.source_bombers, 3);
        assert_eq!(next_playing.source_pods, 1);
        assert_eq!(next_playing.source_mutants, 0);
        assert_eq!(next_playing.source_swarmers, 0);
        assert_eq!(next_playing.world_enemies, 2);
        assert_eq!(next_playing.world_humans, 2);
        assert_eq!(next_playing.lander_mode, "drift");
        let wave_clear = report
            .wave_clear
            .as_ref()
            .expect("example script should report wave clear interstitial");
        assert_eq!(wave_clear.assist_steps, 4);
        assert_eq!(wave_clear.next_wave, 2);
        assert_eq!(wave_clear.score, 400);
        assert_eq!(wave_clear.world_enemies, 0);
        assert_eq!(wave_clear.world_humans, 2);
        assert_eq!(wave_clear.total_survivors, Some(2));
        assert_eq!(wave_clear.visible_icons, Some(1));
        assert_eq!(wave_clear.remaining_awards, Some(1));
        assert_eq!(wave_clear.awarded_points, Some(100));
        assert_eq!(wave_clear.astronaut_sleep_steps_remaining, Some(4));
        assert_eq!(wave_clear.wave_advance_sleep_steps_remaining, None);
        let wave_sleep = report
            .wave_clear_advance_sleep
            .as_ref()
            .expect("example script should report wave advance sleep");
        assert_eq!(wave_sleep.assist_steps, 12);
        assert_eq!(wave_sleep.next_wave, 2);
        assert_eq!(wave_sleep.score, 500);
        assert_eq!(wave_sleep.world_enemies, 0);
        assert_eq!(wave_sleep.world_humans, 2);
        assert_eq!(wave_sleep.total_survivors, Some(2));
        assert_eq!(wave_sleep.visible_icons, Some(2));
        assert_eq!(wave_sleep.remaining_awards, Some(0));
        assert_eq!(wave_sleep.awarded_points, None);
        assert_eq!(wave_sleep.astronaut_sleep_steps_remaining, Some(0));
        assert_eq!(wave_sleep.wave_advance_sleep_steps_remaining, Some(128));
        assert!(report.wave_clear_unavailable_reason.is_none());
        assert!(report.wave_clear_advance_sleep_unavailable_reason.is_none());
        assert!(report.reserve_activation_batches.is_empty());
        assert_eq!(
            report.reserve_activation_status,
            "next_playing_has_no_reserve"
        );
        assert!(report.post_reserve_wave_clear.is_none());
        assert_eq!(
            report.post_reserve_wave_clear_unavailable_reason.as_deref(),
            Some("next_playing_has_no_reserve")
        );
        assert!(report.post_reserve_wave_clear_advance_sleep.is_none());
        assert_eq!(
            report
                .post_reserve_wave_clear_advance_sleep_unavailable_reason
                .as_deref(),
            Some("next_playing_has_no_reserve")
        );
        assert_eq!(report.post_reserve_next_playing_assist_steps, None);
        assert!(report.post_reserve_next_playing.is_none());
        assert_eq!(
            report
                .post_reserve_next_playing_unavailable_reason
                .as_deref(),
            Some("next_playing_has_no_reserve")
        );
        assert!(report.clean_exit);
        assert_eq!(
            report.to_text(),
            concat!(
                "actor script check passed\n",
                "  path: examples/actor-custom-attract.script\n",
                "  attract_events: 8\n",
                "  attract_cycle_steps: 96\n",
                "  attract_cycle_sampled_steps: 96\n",
                "  attract_cycle_frames: attract=96,non_attract=0\n",
                "  attract_cycle_draws: 193\n",
                "  attract_cycle_scene_sprites: 19009\n",
                "  attract_cycle_milestones: williams_reveal=true,defender_coalescence=true,hall_of_fame=true,scoring_surface=true,final_scoring_label=true,cycle_return=true\n",
                "  behavior_kind_profiles: 2\n",
                "  behavior_actor_profiles: 0\n",
                "  wave_profiles: 1\n",
                "  first_frame_phase: Attract\n",
                "  first_frame_draws: 1\n",
                "  first_playing_wave: 1\n",
                "  first_playing_wave_size: 5\n",
                "  first_playing_source_counts: landers=15,bombers=0,pods=0,mutants=0,swarmers=0\n",
                "  first_playing_world_counts: enemies=2,humans=2\n",
                "  first_playing_reserve_counts: landers=0,bombers=0,pods=0,mutants=0,swarmers=0\n",
                "  first_playing_source_state: background_left=0x0000,rng=seed=0xbe,hseed=0xb1,lseed=0x06\n",
                "  first_playing_player_behavior: takes_enemy_collision_damage=true,laser_cooldown_steps=6\n",
                "  first_playing_lander_behavior: mode=drift,seek_speed=1,drift_speed=3,fire_period_steps=96\n",
                "  first_playing_hostile_modes: mutant=chase_player,bomber=drift,pod=drift,swarmer=chase_player,baiter=chase_player\n",
                "  first_playing_hostile_fire: swarmer_period_steps=58,baiter_period_steps=42\n",
                "  wave_clear_assist_steps: 4\n",
                "  wave_clear_next_wave: 2\n",
                "  wave_clear_score: 400\n",
                "  wave_clear_world_counts: enemies=0,humans=2\n",
                "  wave_clear_survivor_bonus: total=2,visible_icons=1,remaining_awards=1,awarded_points=100\n",
                "  wave_clear_sleep: astronaut_steps=4,wave_advance_steps=none\n",
                "  wave_clear_advance_sleep_assist_steps: 12\n",
                "  wave_clear_advance_sleep_next_wave: 2\n",
                "  wave_clear_advance_sleep_score: 500\n",
                "  wave_clear_advance_sleep_world_counts: enemies=0,humans=2\n",
                "  wave_clear_advance_sleep_survivor_bonus: total=2,visible_icons=2,remaining_awards=0,awarded_points=none\n",
                "  wave_clear_advance_sleep_sleep: astronaut_steps=0,wave_advance_steps=128\n",
                "  next_playing_assist_steps: 140\n",
                "  next_playing_wave: 2\n",
                "  next_playing_wave_size: 5\n",
                "  next_playing_source_counts: landers=20,bombers=3,pods=1,mutants=0,swarmers=0\n",
                "  next_playing_world_counts: enemies=2,humans=2\n",
                "  next_playing_reserve_counts: landers=0,bombers=0,pods=0,mutants=0,swarmers=0\n",
                "  next_playing_source_state: background_left=0x0000,rng=seed=0x82,hseed=0x35,lseed=0x88\n",
                "  next_playing_player_behavior: takes_enemy_collision_damage=true,laser_cooldown_steps=6\n",
                "  next_playing_lander_behavior: mode=drift,seek_speed=1,drift_speed=3,fire_period_steps=96\n",
                "  next_playing_hostile_modes: mutant=chase_player,bomber=drift,pod=drift,swarmer=chase_player,baiter=chase_player\n",
                "  next_playing_hostile_fire: swarmer_period_steps=58,baiter_period_steps=42\n",
                "  reserve_activation_batches: 0\n",
                "  reserve_activation_status: next_playing_has_no_reserve\n",
                "  post_reserve_wave_clear: unavailable,reason=next_playing_has_no_reserve\n",
                "  post_reserve_wave_clear_advance_sleep: unavailable,reason=next_playing_has_no_reserve\n",
                "  post_reserve_next_playing: unavailable,reason=next_playing_has_no_reserve\n",
                "  clean_exit: true\n",
            )
        );
    }

    #[test]
    fn actor_script_check_reports_custom_attract_cycle_milestones() {
        let path = write_actor_script_file(
            "actor-script-attract-cycle-check",
            concat!(
                "[attract]\n",
                "cycle 12\n",
                "williams_logo 1 forever 12 20\n",
                "defender_wordmark 2 4 48 36\n",
                "message 3 3 HALLD_TITLE 0x3854\n",
                "scoring_surface 4 6\n",
                "message 5 4 SWARMV 0x5CA8\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "[wave]\n",
                "name attract cycle check waves\n",
                "wave 1\n",
                "lander 80 214\n",
                "human 100 214\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("attract cycle script should check");
        let summary = report
            .attract_cycle
            .as_ref()
            .expect("declared cycle should be sampled");

        assert_eq!(summary.cycle_steps, 12);
        assert_eq!(summary.sampled_steps, 12);
        assert_eq!(summary.attract_frames, 12);
        assert_eq!(summary.non_attract_frames, 0);
        assert!(summary.draw_commands > 0);
        assert!(summary.scene_sprites > 0);
        assert!(summary.saw_williams_reveal);
        assert!(summary.saw_defender_coalescence);
        assert!(summary.saw_hall_of_fame);
        assert!(summary.saw_scoring_surface);
        assert!(summary.saw_final_scoring_label);
        assert!(summary.saw_cycle_return);
        assert!(report.attract_cycle_unavailable_reason.is_none());
        assert!(report.to_text().contains("attract_cycle_steps: 12"));
        assert!(report.to_text().contains(
            "attract_cycle_milestones: williams_reveal=true,defender_coalescence=true,hall_of_fame=true,scoring_surface=true,final_scoring_label=true,cycle_return=true"
        ));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn actor_script_check_reports_source_wave_overrides_at_play_start() {
        let path = write_actor_script_file(
            "actor-script-source-wave-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 SOURCE CHECK\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "[wave]\n",
                "name source check waves\n",
                "source_wave 1 wave_size 5 landers 1 bombers 1 pods 1 mutants 1 swarmers 1 ",
                "swarmer_x_velocity 64 swarmer_shot_time 11 baiter_time 24 ",
                "mutant_x_velocity 48 mutant_random_y 2 mutant_shot_time 12\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("source wave script should check");

        assert_eq!(report.first_playing_wave, 1);
        assert_eq!(report.first_playing_wave_size, 5);
        assert_eq!(report.first_playing_source_landers, 1);
        assert_eq!(report.first_playing_source_bombers, 1);
        assert_eq!(report.first_playing_source_pods, 1);
        assert_eq!(report.first_playing_source_mutants, 1);
        assert_eq!(report.first_playing_source_swarmers, 1);
        assert_eq!(report.first_playing_world_enemies, 5);
        assert_eq!(report.first_playing_world_humans, 10);
        assert_eq!(report.first_playing_reserve_landers, 0);
        assert_eq!(report.first_playing_reserve_bombers, 0);
        assert_eq!(report.first_playing_reserve_pods, 0);
        assert_eq!(report.first_playing_reserve_mutants, 0);
        assert_eq!(report.first_playing_reserve_swarmers, 0);
        assert!(report.to_text().contains(
            "first_playing_source_counts: landers=1,bombers=1,pods=1,mutants=1,swarmers=1"
        ));
        assert!(
            report
                .to_text()
                .contains("first_playing_world_counts: enemies=5,humans=10")
        );
    }

    #[test]
    fn actor_script_check_reports_reserve_and_source_state_at_play_start() {
        let path = write_actor_script_file(
            "actor-script-reserve-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 RESERVE CHECK\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "[wave]\n",
                "name reserve check waves\n",
                "source_wave 1 wave_size 2 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n",
                "reserve_full 3 2 1 1 1\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("reserve script should check");

        assert_eq!(report.first_playing_world_enemies, 2);
        assert_eq!(report.first_playing_world_humans, 10);
        assert_eq!(report.first_playing_reserve_landers, 3);
        assert_eq!(report.first_playing_reserve_bombers, 2);
        assert_eq!(report.first_playing_reserve_pods, 1);
        assert_eq!(report.first_playing_reserve_mutants, 1);
        assert_eq!(report.first_playing_reserve_swarmers, 1);
        assert_eq!(report.first_playing_source_background_left, 0);
        assert!(report.first_playing_source_rng_seed.is_some());
        assert!(report.to_text().contains(
            "first_playing_reserve_counts: landers=3,bombers=2,pods=1,mutants=1,swarmers=1"
        ));
        assert!(
            report
                .to_text()
                .contains("first_playing_source_state: background_left=0x0000,rng=seed=")
        );
    }

    #[test]
    fn actor_script_check_reports_next_wave_progression_after_assisted_clear() {
        let path = write_actor_script_file(
            "actor-script-next-wave-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 NEXT WAVE CHECK\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "[wave]\n",
                "name next wave check waves\n",
                "source_wave 1 wave_size 1 landers 1 bombers 0 pods 0 mutants 0 swarmers 0\n",
                "behavior kind lander lander_mode drift\n",
                "behavior kind lander lander_drift_speed 2\n",
                "source_wave 2 wave_size 3 landers 1 bombers 1 pods 1 mutants 0 swarmers 0\n",
                "reserve_full 2 1 1 1 1\n",
                "behavior kind lander lander_mode chase_player\n",
                "behavior kind lander lander_seek_speed 7\n",
                "behavior kind swarmer swarmer_fire_period_steps 23\n",
                "behavior kind baiter baiter_fire_period_steps 31\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("next wave script should check");
        let next_playing = report
            .next_playing
            .as_ref()
            .expect("checker should reach wave two with assist");
        let wave_clear = report
            .wave_clear
            .as_ref()
            .expect("checker should report the assisted wave-clear interstitial");
        let wave_sleep = report
            .wave_clear_advance_sleep
            .as_ref()
            .expect("checker should report the source wave advance sleep");
        let post_reserve_wave_clear = report
            .post_reserve_wave_clear
            .as_ref()
            .expect("checker should report wave clear after reserve exhaustion");
        let post_reserve_wave_sleep = report
            .post_reserve_wave_clear_advance_sleep
            .as_ref()
            .expect("checker should report post-reserve wave advance sleep");
        let post_reserve_next_playing = report
            .post_reserve_next_playing
            .as_ref()
            .expect("checker should report playable wave after post-reserve sleep");
        assert_eq!(report.reserve_activation_batches.len(), 3);
        let first_activation = &report.reserve_activation_batches[0];
        let second_activation = &report.reserve_activation_batches[1];
        let third_activation = &report.reserve_activation_batches[2];

        assert_eq!(report.first_playing_wave, 1);
        assert_eq!(report.first_playing_world_enemies, 1);
        assert_eq!(wave_clear.assist_steps, 4);
        assert_eq!(wave_clear.next_wave, 2);
        assert_eq!(wave_clear.score, 250);
        assert_eq!(wave_clear.world_enemies, 0);
        assert_eq!(wave_clear.world_humans, 10);
        assert_eq!(wave_clear.total_survivors, Some(10));
        assert_eq!(wave_clear.visible_icons, Some(1));
        assert_eq!(wave_clear.remaining_awards, Some(9));
        assert_eq!(wave_clear.awarded_points, Some(100));
        assert_eq!(wave_clear.astronaut_sleep_steps_remaining, Some(4));
        assert_eq!(wave_clear.wave_advance_sleep_steps_remaining, None);
        assert!(report.wave_clear_unavailable_reason.is_none());
        assert_eq!(wave_sleep.assist_steps, 44);
        assert_eq!(wave_sleep.next_wave, 2);
        assert_eq!(wave_sleep.score, 1150);
        assert_eq!(wave_sleep.world_enemies, 0);
        assert_eq!(wave_sleep.world_humans, 10);
        assert_eq!(wave_sleep.total_survivors, Some(10));
        assert_eq!(wave_sleep.visible_icons, Some(10));
        assert_eq!(wave_sleep.remaining_awards, Some(0));
        assert_eq!(wave_sleep.awarded_points, None);
        assert_eq!(wave_sleep.astronaut_sleep_steps_remaining, Some(0));
        assert_eq!(wave_sleep.wave_advance_sleep_steps_remaining, Some(128));
        assert!(report.wave_clear_advance_sleep_unavailable_reason.is_none());
        assert_eq!(next_playing.wave, 2);
        assert_eq!(next_playing.wave_size, 3);
        assert_eq!(next_playing.source_landers, 1);
        assert_eq!(next_playing.source_bombers, 1);
        assert_eq!(next_playing.source_pods, 1);
        assert_eq!(next_playing.source_mutants, 0);
        assert_eq!(next_playing.source_swarmers, 0);
        assert_eq!(next_playing.world_enemies, 3);
        assert_eq!(next_playing.world_humans, 10);
        assert_eq!(next_playing.reserve_landers, 2);
        assert_eq!(next_playing.reserve_bombers, 1);
        assert_eq!(next_playing.reserve_pods, 1);
        assert_eq!(next_playing.reserve_mutants, 1);
        assert_eq!(next_playing.reserve_swarmers, 1);
        assert_eq!(next_playing.lander_mode, "chase_player");
        assert_eq!(next_playing.lander_seek_speed, 7);
        assert_eq!(next_playing.swarmer_fire_period_steps, 23);
        assert_eq!(next_playing.baiter_fire_period_steps, 31);
        assert!(report.next_playing_assist_steps.is_some_and(
            |steps| steps > 0 && steps < super::ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT as u32
        ));
        assert_eq!(first_activation.assist_steps, 244);
        assert_eq!(first_activation.spawned_counts.landers, 2);
        assert_eq!(first_activation.spawned_counts.bombers, 0);
        assert_eq!(first_activation.spawned_counts.pods, 0);
        assert_eq!(first_activation.spawned_counts.mutants, 0);
        assert_eq!(first_activation.spawned_counts.swarmers, 0);
        assert_eq!(first_activation.playing.wave, 2);
        assert_eq!(first_activation.playing.world_enemies, 2);
        assert_eq!(first_activation.playing.world_humans, 10);
        assert_eq!(first_activation.playing.reserve_landers, 0);
        assert_eq!(first_activation.playing.reserve_bombers, 1);
        assert_eq!(first_activation.playing.reserve_pods, 1);
        assert_eq!(first_activation.playing.reserve_mutants, 1);
        assert_eq!(first_activation.playing.reserve_swarmers, 1);
        assert_eq!(first_activation.playing.lander_mode, "chase_player");
        assert_eq!(first_activation.playing.lander_seek_speed, 7);

        assert!(second_activation.assist_steps > first_activation.assist_steps);
        assert_eq!(second_activation.spawned_counts.landers, 0);
        assert_eq!(second_activation.spawned_counts.bombers, 1);
        assert_eq!(second_activation.spawned_counts.pods, 1);
        assert_eq!(second_activation.spawned_counts.mutants, 1);
        assert_eq!(second_activation.spawned_counts.swarmers, 0);
        assert_eq!(second_activation.playing.wave, 2);
        assert_eq!(second_activation.playing.world_enemies, 3);
        assert_eq!(second_activation.playing.reserve_landers, 0);
        assert_eq!(second_activation.playing.reserve_bombers, 0);
        assert_eq!(second_activation.playing.reserve_pods, 0);
        assert_eq!(second_activation.playing.reserve_mutants, 0);
        assert_eq!(second_activation.playing.reserve_swarmers, 1);

        assert!(third_activation.assist_steps > second_activation.assist_steps);
        assert_eq!(third_activation.spawned_counts.landers, 0);
        assert_eq!(third_activation.spawned_counts.bombers, 0);
        assert_eq!(third_activation.spawned_counts.pods, 0);
        assert_eq!(third_activation.spawned_counts.mutants, 0);
        assert_eq!(third_activation.spawned_counts.swarmers, 1);
        assert_eq!(third_activation.playing.wave, 2);
        assert_eq!(third_activation.playing.world_enemies, 1);
        assert_eq!(third_activation.playing.reserve_landers, 0);
        assert_eq!(third_activation.playing.reserve_bombers, 0);
        assert_eq!(third_activation.playing.reserve_pods, 0);
        assert_eq!(third_activation.playing.reserve_mutants, 0);
        assert_eq!(third_activation.playing.reserve_swarmers, 0);
        assert_eq!(report.reserve_activation_status, "reserve_empty");
        assert_eq!(post_reserve_wave_clear.assist_steps, 736);
        assert_eq!(post_reserve_wave_clear.next_wave, 3);
        assert_eq!(post_reserve_wave_clear.score, 4600);
        assert_eq!(post_reserve_wave_clear.world_enemies, 0);
        assert_eq!(post_reserve_wave_clear.world_humans, 10);
        assert_eq!(post_reserve_wave_clear.total_survivors, Some(10));
        assert_eq!(post_reserve_wave_clear.visible_icons, Some(1));
        assert_eq!(post_reserve_wave_clear.remaining_awards, Some(9));
        assert_eq!(post_reserve_wave_clear.awarded_points, Some(200));
        assert_eq!(
            post_reserve_wave_clear.astronaut_sleep_steps_remaining,
            Some(4)
        );
        assert_eq!(
            post_reserve_wave_clear.wave_advance_sleep_steps_remaining,
            None
        );
        assert!(report.post_reserve_wave_clear_unavailable_reason.is_none());
        assert_eq!(post_reserve_wave_sleep.assist_steps, 776);
        assert_eq!(post_reserve_wave_sleep.next_wave, 3);
        assert_eq!(post_reserve_wave_sleep.score, 6400);
        assert_eq!(post_reserve_wave_sleep.world_enemies, 0);
        assert_eq!(post_reserve_wave_sleep.world_humans, 10);
        assert_eq!(post_reserve_wave_sleep.total_survivors, Some(10));
        assert_eq!(post_reserve_wave_sleep.visible_icons, Some(10));
        assert_eq!(post_reserve_wave_sleep.remaining_awards, Some(0));
        assert_eq!(post_reserve_wave_sleep.awarded_points, None);
        assert_eq!(
            post_reserve_wave_sleep.astronaut_sleep_steps_remaining,
            Some(0)
        );
        assert_eq!(
            post_reserve_wave_sleep.wave_advance_sleep_steps_remaining,
            Some(128)
        );
        assert!(
            report
                .post_reserve_wave_clear_advance_sleep_unavailable_reason
                .is_none()
        );
        assert_eq!(report.post_reserve_next_playing_assist_steps, Some(904));
        assert_eq!(post_reserve_next_playing.wave, 3);
        assert_eq!(post_reserve_next_playing.wave_size, 3);
        assert_eq!(post_reserve_next_playing.source_landers, 1);
        assert_eq!(post_reserve_next_playing.source_bombers, 1);
        assert_eq!(post_reserve_next_playing.source_pods, 1);
        assert_eq!(post_reserve_next_playing.source_mutants, 0);
        assert_eq!(post_reserve_next_playing.source_swarmers, 0);
        assert_eq!(post_reserve_next_playing.world_enemies, 3);
        assert_eq!(post_reserve_next_playing.world_humans, 10);
        assert_eq!(post_reserve_next_playing.reserve_landers, 2);
        assert_eq!(post_reserve_next_playing.reserve_bombers, 1);
        assert_eq!(post_reserve_next_playing.reserve_pods, 1);
        assert_eq!(post_reserve_next_playing.reserve_mutants, 1);
        assert_eq!(post_reserve_next_playing.reserve_swarmers, 1);
        assert_eq!(post_reserve_next_playing.lander_mode, "chase_player");
        assert_eq!(post_reserve_next_playing.lander_seek_speed, 7);
        assert_eq!(post_reserve_next_playing.swarmer_fire_period_steps, 23);
        assert_eq!(post_reserve_next_playing.baiter_fire_period_steps, 31);
        assert!(
            report
                .post_reserve_next_playing_unavailable_reason
                .is_none()
        );
        assert!(report.to_text().contains(
            "next_playing_source_counts: landers=1,bombers=1,pods=1,mutants=0,swarmers=0"
        ));
        assert!(report.to_text().contains(
            "wave_clear_survivor_bonus: total=10,visible_icons=1,remaining_awards=9,awarded_points=100"
        ));
        assert!(report.to_text().contains(
            "wave_clear_advance_sleep_survivor_bonus: total=10,visible_icons=10,remaining_awards=0,awarded_points=none"
        ));
        assert!(
            report.to_text().contains(
                "wave_clear_advance_sleep_sleep: astronaut_steps=0,wave_advance_steps=128"
            )
        );
        assert!(report.to_text().contains(
            "next_playing_reserve_counts: landers=2,bombers=1,pods=1,mutants=1,swarmers=1"
        ));
        assert!(
            report
                .to_text()
                .contains("next_playing_lander_behavior: mode=chase_player,seek_speed=7")
        );
        assert!(
            report.to_text().contains(
                "next_playing_hostile_fire: swarmer_period_steps=23,baiter_period_steps=31"
            )
        );
        assert!(report.to_text().contains("reserve_activation_batches: 3"));
        assert!(report.to_text().contains(
            "reserve_activation_1_spawned_counts: landers=2,bombers=0,pods=0,mutants=0,swarmers=0"
        ));
        assert!(report.to_text().contains(
            "reserve_activation_2_spawned_counts: landers=0,bombers=1,pods=1,mutants=1,swarmers=0"
        ));
        assert!(report.to_text().contains(
            "reserve_activation_3_spawned_counts: landers=0,bombers=0,pods=0,mutants=0,swarmers=1"
        ));
        assert!(report.to_text().contains(
            "reserve_activation_3_reserve_counts: landers=0,bombers=0,pods=0,mutants=0,swarmers=0"
        ));
        assert!(
            report
                .to_text()
                .contains("reserve_activation_status: reserve_empty")
        );
        assert!(
            report
                .to_text()
                .contains("post_reserve_wave_clear_next_wave: 3")
        );
        assert!(report.to_text().contains(
            "post_reserve_wave_clear_survivor_bonus: total=10,visible_icons=1,remaining_awards=9,awarded_points=200"
        ));
        assert!(report.to_text().contains(
            "post_reserve_wave_clear_advance_sleep_survivor_bonus: total=10,visible_icons=10,remaining_awards=0,awarded_points=none"
        ));
        assert!(report.to_text().contains(
            "post_reserve_wave_clear_advance_sleep_sleep: astronaut_steps=0,wave_advance_steps=128"
        ));
        assert!(
            report
                .to_text()
                .contains("post_reserve_next_playing_assist_steps: 904")
        );
        assert!(report.to_text().contains(
            "post_reserve_next_playing_source_counts: landers=1,bombers=1,pods=1,mutants=0,swarmers=0"
        ));
    }

    #[test]
    fn actor_script_check_reports_effective_behavior_overrides_at_play_start() {
        let path = write_actor_script_file(
            "actor-script-behavior-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 BEHAVIOR CHECK\n",
                "[behavior]\n",
                "kind player player_takes_enemy_collision_damage false\n",
                "kind player player_laser_cooldown_steps 5\n",
                "[wave]\n",
                "name behavior check waves\n",
                "source_wave 1 wave_size 5 landers 1 bombers 1 pods 1 mutants 1 swarmers 1\n",
                "behavior kind lander lander_mode chase_player\n",
                "behavior kind lander lander_seek_speed 4\n",
                "behavior kind mutant mutant_mode drift\n",
                "behavior kind bomber bomber_mode chase_player\n",
                "behavior kind pod pod_mode chase_player\n",
                "behavior kind swarmer swarmer_mode drift\n",
                "behavior kind swarmer swarmer_fire_period_steps 17\n",
                "behavior kind baiter baiter_mode drift\n",
                "behavior kind baiter baiter_fire_period_steps 19\n",
                "spawn_behavior lander 0 lander_seek_speed 9\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("behavior script should check");

        assert!(!report.first_playing_player_takes_enemy_collision_damage);
        assert_eq!(report.first_playing_player_laser_cooldown_steps, 5);
        assert_eq!(report.first_playing_lander_mode, "chase_player");
        assert_eq!(report.first_playing_lander_seek_speed, 9);
        assert_eq!(report.first_playing_mutant_mode, "drift");
        assert_eq!(report.first_playing_bomber_mode, "chase_player");
        assert_eq!(report.first_playing_pod_mode, "chase_player");
        assert_eq!(report.first_playing_swarmer_mode, "drift");
        assert_eq!(report.first_playing_baiter_mode, "drift");
        assert_eq!(report.first_playing_swarmer_fire_period_steps, 17);
        assert_eq!(report.first_playing_baiter_fire_period_steps, 19);
        assert!(report.to_text().contains(
            "first_playing_player_behavior: takes_enemy_collision_damage=false,laser_cooldown_steps=5"
        ));
        assert!(
            report
                .to_text()
                .contains("first_playing_lander_behavior: mode=chase_player,seek_speed=9")
        );
        assert!(report.to_text().contains(
            "first_playing_hostile_modes: mutant=drift,bomber=chase_player,pod=chase_player,swarmer=drift,baiter=drift"
        ));
    }

    #[test]
    fn actor_live_uses_actor_derived_game_frame_handoff() {
        let source = include_str!("live_wgpu.rs");

        assert!(source.contains("let actor_frame = self.runtime.step_clean_input(input, xyzzy);"));
        assert!(source.contains("let frame = actor_frame.game_frame();"));
        assert!(source.contains("self.audio.submit_game_frame(&frame);"));
        let old_batch_call = [
            "LiveAudioEventBatch::new(",
            "frame.report.step",
            ", frame.events.sounds())",
        ]
        .concat();
        assert!(!source.contains(&old_batch_call));
    }

    #[test]
    fn live_input_state_carries_xyzzy_mode_for_actor_runtime() {
        let mut input = LiveInputState::default();
        for character in ['X', 'Y', 'Z', 'Z', 'Y'] {
            input.apply(super::LiveControl::HighScoreInitial(character), true);
        }
        input.apply(super::LiveControl::HighScoreInitial('F'), true);
        input.apply(super::LiveControl::HighScoreInitial('G'), true);
        input.apply(super::LiveControl::SmartBomb, true);

        let clean_input = input.drain_game_input();
        let xyzzy = input.drain_xyzzy_mode();

        assert!(clean_input.smart_bomb);
        assert!(xyzzy.active);
        assert!(xyzzy.auto_fire);
        assert!(xyzzy.invincible);
        assert!(xyzzy.overlay_smart_bomb);
        assert!(!input.drain_xyzzy_mode().overlay_smart_bomb);
    }

    #[test]
    fn live_input_state_emits_edge_pulses_and_held_gameplay_controls() {
        let mut input = LiveInputState::default();
        input.apply(super::LiveControl::Coin, true);
        input.apply(super::LiveControl::StartOne, true);
        input.apply(super::LiveControl::StartTwo, true);
        input.apply(super::LiveControl::Thrust, true);
        input.apply(super::LiveControl::AltitudeUp, true);
        input.apply(super::LiveControl::AltitudeDown, true);
        input.apply(super::LiveControl::Reverse, true);
        input.apply(super::LiveControl::Fire, true);
        input.apply(super::LiveControl::SmartBomb, true);
        input.apply(super::LiveControl::Hyperspace, true);
        input.apply(super::LiveControl::ServiceAutoUp, true);
        input.apply(super::LiveControl::ServiceAdvance, true);
        input.apply(super::LiveControl::HighScoreReset, true);
        input.apply(super::LiveControl::HighScoreInitial('A'), true);
        input.apply(super::LiveControl::HighScoreBackspace, true);
        input.apply(super::LiveControl::Quit, true);

        assert_eq!(
            input.drain_game_input(),
            GameInput {
                coin: true,
                start_one: true,
                start_two: true,
                thrust: true,
                altitude_up: true,
                altitude_down: true,
                reverse: true,
                fire: true,
                smart_bomb: true,
                hyperspace: true,
                service_auto_up: true,
                service_advance: true,
                high_score_reset: true,
                high_score_initial: Some('A'),
                high_score_backspace: true,
                ..GameInput::NONE
            }
        );
        assert_eq!(
            input.drain_game_input(),
            GameInput {
                thrust: true,
                altitude_up: true,
                altitude_down: true,
                reverse: true,
                fire: true,
                smart_bomb: true,
                hyperspace: true,
                service_auto_up: true,
                ..GameInput::NONE
            }
        );

        input.apply(super::LiveControl::Thrust, false);
        input.apply(super::LiveControl::AltitudeUp, false);
        input.apply(super::LiveControl::AltitudeDown, false);
        input.apply(super::LiveControl::Reverse, false);
        input.apply(super::LiveControl::Fire, false);
        input.apply(super::LiveControl::SmartBomb, false);
        input.apply(super::LiveControl::Hyperspace, false);
        input.apply(super::LiveControl::ServiceAutoUp, false);
        assert_eq!(input.drain_game_input(), GameInput::NONE);
    }

    fn write_actor_script_file(label: &str, source: &str) -> std::path::PathBuf {
        let path = unique_actor_script_path(label);
        fs::write(&path, source).expect("write actor script");
        path
    }

    fn unique_actor_script_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "defender-{label}-{}-{nanos}.script",
            std::process::id()
        ))
    }
}
