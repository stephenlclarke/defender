//! Actor runtime smoke runner.
//!
//! This command path exercises the actor driver through `ActorRuntimeAdapter`
//! and the native draw planner while actor frames also expose a clean
//! `GameFrame` handoff for runtime preflights.

use std::collections::BTreeSet;

use anyhow::bail;

use crate::{
    actor_game::{
        ActorFrame, ActorKind, ActorRuntimeAdapter, AttractScript, GameInput, Phase, Point,
        SpriteKey, VisualEffect,
    },
    game::{GameEvent, GameOverSnapshot, SoundEvent},
    renderer::{
        NativeRenderPipeline, NativeSceneRenderer, RenderLayer, SceneDrawPlan, SpriteId,
        source_message_text,
    },
};

const ACTOR_SMOKE_FRAMES: u32 = 192;
const ACTOR_SMOKE_COIN_FRAME: u32 = 1;
const ACTOR_SMOKE_START_FRAME: u32 = 3;
const ACTOR_SMOKE_FIRE_FRAME: u32 = 143;
const ACTOR_SMOKE_THRUST_FRAME: u32 = 145;
const ACTOR_SMOKE_REVERSE_FRAME: u32 = 147;
const ACTOR_SMOKE_SMART_BOMB_FRAME: u32 = 149;
const ACTOR_SMOKE_HYPERSPACE_FRAME: u32 = 151;
const ACTOR_SMOKE_ALTITUDE_DOWN_FRAME: u32 = 153;
const POST_GAME_PLAYER_COLLISIONS: u8 = 3;
const POST_GAME_HALL_STALL_STEPS: u8 = 60;
const POST_GAME_PLAYER_RESPAWN_SEARCH_STEPS: u16 = 160;
const POST_GAME_ATTRACT_RETURN_SEARCH_STEPS: u8 = 96;
const REQUIRED_INPUTS: [&str; 8] = [
    "coin",
    "start_one",
    "fire",
    "thrust",
    "reverse",
    "smart_bomb",
    "hyperspace",
    "altitude_down",
];
const REQUIRED_SPRITES: [&str; 6] = [
    "attract_williams_logo_pixel",
    "player_ship",
    "enemy_lander",
    "human",
    "player_projectile",
    "score_digit_0",
];
const REQUIRED_PIPELINES: [&str; 3] = ["sprites", "projectiles", "hud_text"];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorSmokeReport {
    pub(crate) frames: u32,
    pub(crate) first_frame_size: Option<(u32, u32)>,
    pub(crate) distinct_scene_signatures: usize,
    pub(crate) saw_attract: bool,
    pub(crate) saw_credit: bool,
    pub(crate) saw_playing: bool,
    pub(crate) attract_frames: u32,
    pub(crate) credited_frames: u32,
    pub(crate) playing_frames: u32,
    pub(crate) actor_event_frames: u32,
    pub(crate) actor_sound_frames: u32,
    pub(crate) actor_sound_events: usize,
    pub(crate) sprite_frames: u32,
    pub(crate) sprite_instances: usize,
    pub(crate) sprite_draw_commands: usize,
    pub(crate) object_sprites: usize,
    pub(crate) projectile_sprites: usize,
    pub(crate) hud_sprites: usize,
    pub(crate) overlay_sprites: usize,
    pub(crate) covered_sprites: Vec<String>,
    pub(crate) object_draw_commands: usize,
    pub(crate) projectile_draw_commands: usize,
    pub(crate) hud_draw_commands: usize,
    pub(crate) overlay_draw_commands: usize,
    pub(crate) covered_pipelines: Vec<String>,
    pub(crate) wgpu_frame_commands: usize,
    pub(crate) temporary_raster_commands: usize,
    pub(crate) missing_sprite_regions: usize,
    pub(crate) injected_inputs: Vec<String>,
    pub(crate) clean_exit: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorAttractCycleSmokeReport {
    pub(crate) frames: u64,
    pub(crate) cycle_steps: u64,
    pub(crate) distinct_scene_signatures: usize,
    pub(crate) attract_frames: u64,
    pub(crate) playing_frames: u64,
    pub(crate) game_over_frames: u64,
    pub(crate) high_score_entry_frames: u64,
    pub(crate) actor_event_frames: u64,
    pub(crate) actor_sound_frames: u64,
    pub(crate) actor_sound_events: usize,
    pub(crate) sprite_instances: usize,
    pub(crate) sprite_draw_commands: usize,
    pub(crate) wgpu_frame_commands: usize,
    pub(crate) missing_sprite_regions: usize,
    pub(crate) saw_williams_reveal: bool,
    pub(crate) saw_defender_coalescence: bool,
    pub(crate) saw_hall_of_fame: bool,
    pub(crate) saw_scoring_surface: bool,
    pub(crate) saw_final_scoring_label: bool,
    pub(crate) saw_cycle_return: bool,
    pub(crate) clean_exit: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ActorPostGameSmokeReport {
    pub(crate) frames: u32,
    pub(crate) distinct_scene_signatures: usize,
    pub(crate) playing_frames: u32,
    pub(crate) high_score_entry_frames: u32,
    pub(crate) game_over_frames: u32,
    pub(crate) attract_frames: u32,
    pub(crate) forced_player_collisions: u8,
    pub(crate) final_score: u32,
    pub(crate) final_lives: u8,
    pub(crate) player_destroyed_events: usize,
    pub(crate) game_over_events: usize,
    pub(crate) high_score_entry_events: usize,
    pub(crate) high_score_initial_accept_events: usize,
    pub(crate) high_score_submit_events: usize,
    pub(crate) actor_sound_frames: u32,
    pub(crate) actor_sound_events: usize,
    pub(crate) game_over_sound_events: usize,
    pub(crate) saw_game_over_hall_stall: bool,
    pub(crate) hall_stall_frames: u32,
    pub(crate) saw_attract_return: bool,
    pub(crate) saw_return_williams_reveal: bool,
    pub(crate) saw_player_sprite: bool,
    pub(crate) saw_pod_sprite: bool,
    pub(crate) saw_explosion_pixels: bool,
    pub(crate) saw_hall_of_fame: bool,
    pub(crate) sprite_instances: usize,
    pub(crate) sprite_draw_commands: usize,
    pub(crate) wgpu_frame_commands: usize,
    pub(crate) missing_sprite_regions: usize,
    pub(crate) clean_exit: bool,
}

impl ActorSmokeReport {
    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        if self.frames == 0 {
            bail!("actor smoke did not advance any frames");
        }
        if self.first_frame_size.is_none() {
            bail!("actor smoke did not record a frame size");
        }
        if self.distinct_scene_signatures < 3 {
            bail!("actor smoke did not produce dynamic scene signatures");
        }
        if !self.saw_attract || self.attract_frames == 0 {
            bail!("actor smoke did not observe attract frames");
        }
        if !self.saw_credit || self.credited_frames == 0 {
            bail!("actor smoke did not observe a credited attract frame");
        }
        if !self.saw_playing || self.playing_frames == 0 {
            bail!("actor smoke did not observe playing frames");
        }
        if self.actor_event_frames == 0 {
            bail!("actor smoke did not produce clean gameplay events");
        }
        if self.actor_sound_frames == 0 || self.actor_sound_events == 0 {
            bail!("actor smoke did not produce clean sound events");
        }
        if self.sprite_frames != self.frames {
            bail!("actor smoke did not produce sprites for every frame");
        }
        if self.sprite_instances == 0 || self.sprite_draw_commands == 0 {
            bail!("actor smoke did not produce sprite draw plans");
        }
        if self.object_sprites == 0 {
            bail!("actor smoke did not produce object sprites");
        }
        if self.projectile_sprites == 0 {
            bail!("actor smoke did not produce projectile sprites");
        }
        if self.hud_sprites == 0 {
            bail!("actor smoke did not produce hud sprites");
        }
        if self.overlay_sprites == 0 {
            bail!("actor smoke did not produce overlay sprites");
        }
        for required in REQUIRED_SPRITES {
            if !self.covered_sprites.iter().any(|sprite| sprite == required) {
                bail!("actor smoke did not cover {required} sprite");
            }
        }
        if self.object_draw_commands == 0 {
            bail!("actor smoke did not produce object draw commands");
        }
        if self.projectile_draw_commands == 0 {
            bail!("actor smoke did not produce projectile draw commands");
        }
        if self.hud_draw_commands == 0 {
            bail!("actor smoke did not produce hud draw commands");
        }
        if self.overlay_draw_commands == 0 {
            bail!("actor smoke did not produce overlay draw commands");
        }
        for required in REQUIRED_PIPELINES {
            if !self
                .covered_pipelines
                .iter()
                .any(|pipeline| pipeline == required)
            {
                bail!("actor smoke did not cover {required} pipeline");
            }
        }
        if self.wgpu_frame_commands == 0 {
            bail!("actor smoke did not produce wgpu frame commands");
        }
        if self.temporary_raster_commands != 0 {
            bail!("actor smoke unexpectedly produced temporary raster frame commands");
        }
        if self.missing_sprite_regions != 0 {
            bail!("actor smoke had missing sprite atlas regions");
        }
        for required in REQUIRED_INPUTS {
            if !self.injected_inputs.iter().any(|input| input == required) {
                bail!("actor smoke did not inject {required}");
            }
        }
        if !self.clean_exit {
            bail!("actor smoke did not exit cleanly");
        }
        Ok(())
    }

    pub(crate) fn to_text(&self) -> String {
        let frame_size = self
            .first_frame_size
            .map(|(width, height)| format!("{width}x{height}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        format!(
            "actor smoke passed\n  frames: {}\n  first_frame_size: {}\n  distinct_scene_signatures: {}\n  saw_attract: {} (frames: {})\n  saw_credit: {} (frames: {})\n  saw_playing: {} (frames: {})\n  actor_event_frames: {}\n  actor_sound_frames: {}\n  actor_sound_events: {}\n  sprite_frames: {}\n  sprite_instances: {}\n  sprite_draw_commands: {}\n  object_sprites: {}\n  projectile_sprites: {}\n  hud_sprites: {}\n  overlay_sprites: {}\n  covered_sprites: {}\n  object_draw_commands: {}\n  projectile_draw_commands: {}\n  hud_draw_commands: {}\n  overlay_draw_commands: {}\n  covered_pipelines: {}\n  wgpu_frame_commands: {}\n  temporary_raster_commands: {}\n  missing_sprite_regions: {}\n  injected_inputs: {}\n  clean_exit: {}\n",
            self.frames,
            frame_size,
            self.distinct_scene_signatures,
            self.saw_attract,
            self.attract_frames,
            self.saw_credit,
            self.credited_frames,
            self.saw_playing,
            self.playing_frames,
            self.actor_event_frames,
            self.actor_sound_frames,
            self.actor_sound_events,
            self.sprite_frames,
            self.sprite_instances,
            self.sprite_draw_commands,
            self.object_sprites,
            self.projectile_sprites,
            self.hud_sprites,
            self.overlay_sprites,
            self.covered_sprites.join(","),
            self.object_draw_commands,
            self.projectile_draw_commands,
            self.hud_draw_commands,
            self.overlay_draw_commands,
            self.covered_pipelines.join(","),
            self.wgpu_frame_commands,
            self.temporary_raster_commands,
            self.missing_sprite_regions,
            self.injected_inputs.join(","),
            self.clean_exit
        )
    }
}

impl ActorAttractCycleSmokeReport {
    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        if self.frames == 0 {
            bail!("actor attract smoke did not advance any steps");
        }
        if self.cycle_steps == 0 {
            bail!("actor attract smoke did not find an attract cycle length");
        }
        if self.frames < self.cycle_steps {
            bail!(
                "actor attract smoke only advanced {} step(s), expected at least {}",
                self.frames,
                self.cycle_steps
            );
        }
        if self.attract_frames != self.frames {
            bail!("actor attract smoke left attract mode");
        }
        if self.playing_frames != 0
            || self.game_over_frames != 0
            || self.high_score_entry_frames != 0
        {
            bail!("actor attract smoke observed non-attract phases");
        }
        if self.distinct_scene_signatures < 8 {
            bail!("actor attract smoke did not produce dynamic attract scene signatures");
        }
        if self.actor_event_frames != 0 {
            bail!("actor attract smoke unexpectedly produced gameplay events");
        }
        if self.actor_sound_frames != 0 || self.actor_sound_events != 0 {
            bail!("actor attract smoke unexpectedly produced sound events");
        }
        if self.sprite_instances == 0 || self.sprite_draw_commands == 0 {
            bail!("actor attract smoke did not produce sprite draw plans");
        }
        if self.wgpu_frame_commands == 0 {
            bail!("actor attract smoke did not produce wgpu frame commands");
        }
        if self.missing_sprite_regions != 0 {
            bail!("actor attract smoke had missing sprite atlas regions");
        }
        if !self.saw_williams_reveal {
            bail!("actor attract smoke did not cover Williams reveal pixels");
        }
        if !self.saw_defender_coalescence {
            bail!("actor attract smoke did not cover Defender coalescence");
        }
        if !self.saw_hall_of_fame {
            bail!("actor attract smoke did not cover Hall of Fame attract page");
        }
        if !self.saw_scoring_surface {
            bail!("actor attract smoke did not cover scoring surface");
        }
        if !self.saw_final_scoring_label {
            bail!("actor attract smoke did not cover final scoring label");
        }
        if !self.saw_cycle_return {
            bail!("actor attract smoke did not return to Williams after cycle boundary");
        }
        if !self.clean_exit {
            bail!("actor attract smoke did not exit cleanly");
        }
        Ok(())
    }

    pub(crate) fn to_text(&self) -> String {
        format!(
            "actor attract smoke passed\n  frames: {}\n  cycle_steps: {}\n  distinct_scene_signatures: {}\n  attract_frames: {}\n  non_attract_frames: {}\n  actor_event_frames: {}\n  actor_sound_frames: {}\n  actor_sound_events: {}\n  sprite_instances: {}\n  sprite_draw_commands: {}\n  wgpu_frame_commands: {}\n  missing_sprite_regions: {}\n  saw_williams_reveal: {}\n  saw_defender_coalescence: {}\n  saw_hall_of_fame: {}\n  saw_scoring_surface: {}\n  saw_final_scoring_label: {}\n  saw_cycle_return: {}\n  clean_exit: {}\n",
            self.frames,
            self.cycle_steps,
            self.distinct_scene_signatures,
            self.attract_frames,
            self.playing_frames
                .saturating_add(self.game_over_frames)
                .saturating_add(self.high_score_entry_frames),
            self.actor_event_frames,
            self.actor_sound_frames,
            self.actor_sound_events,
            self.sprite_instances,
            self.sprite_draw_commands,
            self.wgpu_frame_commands,
            self.missing_sprite_regions,
            self.saw_williams_reveal,
            self.saw_defender_coalescence,
            self.saw_hall_of_fame,
            self.saw_scoring_surface,
            self.saw_final_scoring_label,
            self.saw_cycle_return,
            self.clean_exit
        )
    }
}

impl ActorPostGameSmokeReport {
    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        if self.frames == 0 {
            bail!("actor post-game smoke did not advance any frames");
        }
        if self.distinct_scene_signatures < 6 {
            bail!("actor post-game smoke did not produce dynamic scene signatures");
        }
        if self.playing_frames == 0 {
            bail!("actor post-game smoke did not observe playing frames");
        }
        if self.high_score_entry_frames == 0 {
            bail!("actor post-game smoke did not observe high-score-entry frames");
        }
        if self.game_over_frames == 0 {
            bail!("actor post-game smoke did not observe game-over frames");
        }
        if self.attract_frames == 0 || !self.saw_attract_return {
            bail!("actor post-game smoke did not return to attract");
        }
        if self.forced_player_collisions != POST_GAME_PLAYER_COLLISIONS {
            bail!(
                "actor post-game smoke forced {} player collision(s), expected {}",
                self.forced_player_collisions,
                POST_GAME_PLAYER_COLLISIONS
            );
        }
        if self.final_score < 3_000 {
            bail!("actor post-game smoke did not build a qualifying final score");
        }
        if self.final_lives != 0 {
            bail!("actor post-game smoke did not end with zero lives");
        }
        if self.player_destroyed_events < usize::from(POST_GAME_PLAYER_COLLISIONS) {
            bail!("actor post-game smoke did not emit player-destroyed events");
        }
        if self.game_over_events == 0 {
            bail!("actor post-game smoke did not emit game-over events");
        }
        if self.high_score_entry_events == 0 {
            bail!("actor post-game smoke did not emit high-score-entry events");
        }
        if self.high_score_initial_accept_events != 3 {
            bail!("actor post-game smoke did not accept three high-score initials");
        }
        if self.high_score_submit_events != 1 {
            bail!("actor post-game smoke did not submit one high-score entry");
        }
        if self.actor_sound_frames == 0 || self.actor_sound_events == 0 {
            bail!("actor post-game smoke did not produce clean sound events");
        }
        if self.game_over_sound_events == 0 {
            bail!("actor post-game smoke did not bridge the game-over sound command");
        }
        if !self.saw_game_over_hall_stall
            || self.hall_stall_frames != u32::from(POST_GAME_HALL_STALL_STEPS)
        {
            bail!("actor post-game smoke did not observe the 60-step Hall-of-Fame stall");
        }
        if !self.saw_return_williams_reveal {
            bail!("actor post-game smoke did not restart Williams reveal after post-game return");
        }
        if !self.saw_player_sprite {
            bail!("actor post-game smoke did not render player sprites");
        }
        if !self.saw_pod_sprite {
            bail!("actor post-game smoke did not render pod sprites");
        }
        if !self.saw_explosion_pixels {
            bail!("actor post-game smoke did not render source explosion pixels");
        }
        if !self.saw_hall_of_fame {
            bail!("actor post-game smoke did not render Hall of Fame after submission");
        }
        if self.sprite_instances == 0 || self.sprite_draw_commands == 0 {
            bail!("actor post-game smoke did not produce sprite draw plans");
        }
        if self.wgpu_frame_commands == 0 {
            bail!("actor post-game smoke did not produce wgpu frame commands");
        }
        if self.missing_sprite_regions != 0 {
            bail!("actor post-game smoke had missing sprite atlas regions");
        }
        if !self.clean_exit {
            bail!("actor post-game smoke did not exit cleanly");
        }
        Ok(())
    }

    pub(crate) fn to_text(&self) -> String {
        format!(
            "actor post-game smoke passed\n  frames: {}\n  distinct_scene_signatures: {}\n  playing_frames: {}\n  high_score_entry_frames: {}\n  game_over_frames: {}\n  attract_frames: {}\n  forced_player_collisions: {}\n  final_score: {}\n  final_lives: {}\n  player_destroyed_events: {}\n  game_over_events: {}\n  high_score_entry_events: {}\n  high_score_initial_accept_events: {}\n  high_score_submit_events: {}\n  actor_sound_frames: {}\n  actor_sound_events: {}\n  game_over_sound_events: {}\n  hall_stall_frames: {}\n  saw_attract_return: {}\n  saw_return_williams_reveal: {}\n  saw_player_sprite: {}\n  saw_pod_sprite: {}\n  saw_explosion_pixels: {}\n  saw_hall_of_fame: {}\n  sprite_instances: {}\n  sprite_draw_commands: {}\n  wgpu_frame_commands: {}\n  missing_sprite_regions: {}\n  clean_exit: {}\n",
            self.frames,
            self.distinct_scene_signatures,
            self.playing_frames,
            self.high_score_entry_frames,
            self.game_over_frames,
            self.attract_frames,
            self.forced_player_collisions,
            self.final_score,
            self.final_lives,
            self.player_destroyed_events,
            self.game_over_events,
            self.high_score_entry_events,
            self.high_score_initial_accept_events,
            self.high_score_submit_events,
            self.actor_sound_frames,
            self.actor_sound_events,
            self.game_over_sound_events,
            self.hall_stall_frames,
            self.saw_attract_return,
            self.saw_return_williams_reveal,
            self.saw_player_sprite,
            self.saw_pod_sprite,
            self.saw_explosion_pixels,
            self.saw_hall_of_fame,
            self.sprite_instances,
            self.sprite_draw_commands,
            self.wgpu_frame_commands,
            self.missing_sprite_regions,
            self.clean_exit
        )
    }
}

pub(crate) fn run() -> anyhow::Result<()> {
    let report = default_smoke_report()?;

    print!("{}", report.to_text());
    Ok(())
}

pub(crate) fn run_attract_cycle() -> anyhow::Result<()> {
    let report = default_attract_cycle_report()?;

    print!("{}", report.to_text());
    Ok(())
}

pub(crate) fn run_post_game() -> anyhow::Result<()> {
    let report = default_post_game_report()?;

    print!("{}", report.to_text());
    Ok(())
}

pub(crate) fn default_smoke_report() -> anyhow::Result<ActorSmokeReport> {
    smoke_report(ACTOR_SMOKE_FRAMES)
}

pub(crate) fn default_attract_cycle_report() -> anyhow::Result<ActorAttractCycleSmokeReport> {
    attract_cycle_report(default_attract_cycle_steps()?)
}

pub(crate) fn default_post_game_report() -> anyhow::Result<ActorPostGameSmokeReport> {
    post_game_report()
}

pub(crate) fn smoke_frame_count() -> u32 {
    ACTOR_SMOKE_FRAMES
}

pub(crate) fn smoke_actor_input(frame_index: u32) -> GameInput {
    smoke_input(frame_index).value
}

pub(crate) fn smoke_report(frames: u32) -> anyhow::Result<ActorSmokeReport> {
    if frames == 0 {
        bail!("actor smoke frame count must be positive");
    }

    let mut runtime = ActorRuntimeAdapter::new();
    let renderer = NativeSceneRenderer::default();
    let mut report = ActorSmokeReport {
        clean_exit: true,
        ..ActorSmokeReport::default()
    };
    let mut signatures = BTreeSet::new();

    for frame_index in 0..frames {
        let input = smoke_input(frame_index);
        if let Some(label) = input.label {
            record_unique_label(&mut report.injected_inputs, label);
        }

        let frame = runtime.step(input.value);
        let plan = renderer.prepare(&frame.scene);
        observe_frame(&mut report, &mut signatures, &frame, &plan);
    }

    report.distinct_scene_signatures = signatures.len();
    report.validate()?;
    Ok(report)
}

pub(crate) fn attract_cycle_report(frames: u64) -> anyhow::Result<ActorAttractCycleSmokeReport> {
    if frames == 0 {
        bail!("actor attract smoke frame count must be positive");
    }

    let cycle_steps = default_attract_cycle_steps()?;
    let mut runtime = ActorRuntimeAdapter::new();
    let renderer = NativeSceneRenderer::default();
    let mut report = ActorAttractCycleSmokeReport {
        cycle_steps,
        clean_exit: true,
        ..ActorAttractCycleSmokeReport::default()
    };
    let mut signatures = BTreeSet::new();

    for _ in 0..frames {
        let frame = runtime.step(GameInput::NONE);
        let plan = renderer.prepare(&frame.scene);
        observe_attract_cycle_frame(&mut report, &mut signatures, &frame, &plan);
    }

    report.distinct_scene_signatures = signatures.len();
    report.validate()?;
    Ok(report)
}

pub(crate) fn post_game_report() -> anyhow::Result<ActorPostGameSmokeReport> {
    let mut runtime = ActorRuntimeAdapter::new();
    let renderer = NativeSceneRenderer::default();
    let mut report = ActorPostGameSmokeReport {
        clean_exit: true,
        ..ActorPostGameSmokeReport::default()
    };
    let mut signatures = BTreeSet::new();

    step_post_game(
        &mut runtime,
        &renderer,
        &mut report,
        &mut signatures,
        GameInput {
            coin: true,
            ..GameInput::NONE
        },
    );
    step_post_game(
        &mut runtime,
        &renderer,
        &mut report,
        &mut signatures,
        GameInput {
            start_one: true,
            ..GameInput::NONE
        },
    );

    for _ in 0..POST_GAME_PLAYER_COLLISIONS {
        let player_position =
            step_until_player_position(&mut runtime, &renderer, &mut report, &mut signatures)?;
        runtime.driver_mut().spawn_pod_for_test(player_position);
        let collision = step_post_game(
            &mut runtime,
            &renderer,
            &mut report,
            &mut signatures,
            GameInput::NONE,
        );
        if collision
            .events
            .gameplay()
            .contains(&GameEvent::PlayerDestroyed)
        {
            report.forced_player_collisions = report.forced_player_collisions.saturating_add(1);
        }
    }

    for initial in ['P', 'L', 'R'] {
        step_post_game(
            &mut runtime,
            &renderer,
            &mut report,
            &mut signatures,
            GameInput {
                high_score_initial: Some(initial),
                ..GameInput::NONE
            },
        );
    }

    for _ in 0..POST_GAME_ATTRACT_RETURN_SEARCH_STEPS {
        let frame = step_post_game(
            &mut runtime,
            &renderer,
            &mut report,
            &mut signatures,
            GameInput::NONE,
        );
        if frame.report.phase == Phase::Attract {
            report.saw_attract_return = true;
            break;
        }
    }

    report.distinct_scene_signatures = signatures.len();
    report.validate()?;
    Ok(report)
}

fn step_post_game(
    runtime: &mut ActorRuntimeAdapter,
    renderer: &NativeSceneRenderer,
    report: &mut ActorPostGameSmokeReport,
    signatures: &mut BTreeSet<u64>,
    input: GameInput,
) -> ActorFrame {
    let frame = runtime.step(input);
    let plan = renderer.prepare(&frame.scene);
    observe_post_game_frame(report, signatures, &frame, &plan);
    frame
}

fn step_until_player_position(
    runtime: &mut ActorRuntimeAdapter,
    renderer: &NativeSceneRenderer,
    report: &mut ActorPostGameSmokeReport,
    signatures: &mut BTreeSet<u64>,
) -> anyhow::Result<Point> {
    for _ in 0..POST_GAME_PLAYER_RESPAWN_SEARCH_STEPS {
        let frame = step_post_game(runtime, renderer, report, signatures, GameInput::NONE);
        if let Some(position) = frame
            .report
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.position)
        {
            return Ok(position);
        }
    }

    bail!("actor post-game smoke could not find a live player for forced collision")
}

fn observe_frame(
    report: &mut ActorSmokeReport,
    signatures: &mut BTreeSet<u64>,
    frame: &ActorFrame,
    plan: &SceneDrawPlan,
) {
    report.frames = report.frames.saturating_add(1);
    report
        .first_frame_size
        .get_or_insert(surface_tuple(frame.scene.surface));
    report.saw_attract |= frame.report.phase == Phase::Attract;
    report.saw_credit |= frame.report.phase == Phase::Attract && frame.report.credits > 0;
    report.saw_playing |= frame.report.phase == Phase::Playing;

    if frame.report.phase == Phase::Attract {
        report.attract_frames = report.attract_frames.saturating_add(1);
        if frame.report.credits > 0 {
            report.credited_frames = report.credited_frames.saturating_add(1);
        }
    }
    if frame.report.phase == Phase::Playing {
        report.playing_frames = report.playing_frames.saturating_add(1);
    }
    if !frame.events.gameplay().is_empty() {
        report.actor_event_frames = report.actor_event_frames.saturating_add(1);
    }
    if !frame.events.sounds().is_empty() {
        report.actor_sound_frames = report.actor_sound_frames.saturating_add(1);
        report.actor_sound_events = report
            .actor_sound_events
            .saturating_add(frame.events.sounds().len());
    }

    let summary = frame.scene.summary();
    if summary.sprite_count > 0 {
        report.sprite_frames = report.sprite_frames.saturating_add(1);
    }
    report.object_sprites = report.object_sprites.saturating_add(summary.layers.objects);
    report.projectile_sprites = report
        .projectile_sprites
        .saturating_add(summary.layers.projectiles);
    report.hud_sprites = report.hud_sprites.saturating_add(summary.layers.hud);
    report.overlay_sprites = report
        .overlay_sprites
        .saturating_add(summary.layers.overlay);
    for sprite in &frame.scene.sprites {
        if let Some(label) = required_sprite_label(sprite.sprite) {
            record_unique_label(&mut report.covered_sprites, label);
        }
    }

    report.sprite_instances = report
        .sprite_instances
        .saturating_add(plan.sprite_instances);
    report.sprite_draw_commands = report
        .sprite_draw_commands
        .saturating_add(plan.sprite_draw_commands.len());
    for command in &plan.sprite_draw_commands {
        record_draw_command(report, command.layer);
        if let Some(label) = required_pipeline_label(command.pipeline) {
            record_unique_label(&mut report.covered_pipelines, label);
        }
    }
    report.wgpu_frame_commands = report
        .wgpu_frame_commands
        .saturating_add(plan.frame_plan.command_count());
    report.temporary_raster_commands = report
        .temporary_raster_commands
        .saturating_add(plan.frame_plan.temporary_raster_count());
    report.missing_sprite_regions = report
        .missing_sprite_regions
        .saturating_add(plan.missing_sprite_regions);
    signatures.insert(scene_signature(frame, plan));
}

fn observe_attract_cycle_frame(
    report: &mut ActorAttractCycleSmokeReport,
    signatures: &mut BTreeSet<u64>,
    frame: &ActorFrame,
    plan: &SceneDrawPlan,
) {
    report.frames = report.frames.saturating_add(1);
    match frame.report.phase {
        Phase::Attract => report.attract_frames = report.attract_frames.saturating_add(1),
        Phase::Playing => report.playing_frames = report.playing_frames.saturating_add(1),
        Phase::GameOver => report.game_over_frames = report.game_over_frames.saturating_add(1),
        Phase::HighScoreEntry => {
            report.high_score_entry_frames = report.high_score_entry_frames.saturating_add(1);
        }
    }

    if !frame.events.gameplay().is_empty() {
        report.actor_event_frames = report.actor_event_frames.saturating_add(1);
    }
    if !frame.events.sounds().is_empty() {
        report.actor_sound_frames = report.actor_sound_frames.saturating_add(1);
        report.actor_sound_events = report
            .actor_sound_events
            .saturating_add(frame.events.sounds().len());
    }

    observe_attract_cycle_draws(report, frame);
    observe_attract_cycle_scene(report, frame);

    report.sprite_instances = report
        .sprite_instances
        .saturating_add(plan.sprite_instances);
    report.sprite_draw_commands = report
        .sprite_draw_commands
        .saturating_add(plan.sprite_draw_commands.len());
    report.wgpu_frame_commands = report
        .wgpu_frame_commands
        .saturating_add(plan.frame_plan.command_count());
    report.missing_sprite_regions = report
        .missing_sprite_regions
        .saturating_add(plan.missing_sprite_regions);
    signatures.insert(scene_signature(frame, plan));
}

fn observe_post_game_frame(
    report: &mut ActorPostGameSmokeReport,
    signatures: &mut BTreeSet<u64>,
    frame: &ActorFrame,
    plan: &SceneDrawPlan,
) {
    report.frames = report.frames.saturating_add(1);
    match frame.report.phase {
        Phase::Attract => report.attract_frames = report.attract_frames.saturating_add(1),
        Phase::Playing => report.playing_frames = report.playing_frames.saturating_add(1),
        Phase::GameOver => report.game_over_frames = report.game_over_frames.saturating_add(1),
        Phase::HighScoreEntry => {
            report.high_score_entry_frames = report.high_score_entry_frames.saturating_add(1);
        }
    }

    report.final_score = frame.report.score;
    report.final_lives = frame.report.lives;

    for event in frame.events.gameplay() {
        match event {
            GameEvent::PlayerDestroyed => {
                report.player_destroyed_events = report.player_destroyed_events.saturating_add(1);
            }
            GameEvent::GameOver => {
                report.game_over_events = report.game_over_events.saturating_add(1);
            }
            GameEvent::HighScoreEntryStarted => {
                report.high_score_entry_events = report.high_score_entry_events.saturating_add(1);
            }
            GameEvent::HighScoreInitialAccepted => {
                report.high_score_initial_accept_events =
                    report.high_score_initial_accept_events.saturating_add(1);
            }
            GameEvent::HighScoreSubmitted => {
                report.high_score_submit_events = report.high_score_submit_events.saturating_add(1);
            }
            _ => {}
        }
    }

    if !frame.events.sounds().is_empty() {
        report.actor_sound_frames = report.actor_sound_frames.saturating_add(1);
        report.actor_sound_events = report
            .actor_sound_events
            .saturating_add(frame.events.sounds().len());
    }
    report.game_over_sound_events = report.game_over_sound_events.saturating_add(
        frame
            .events
            .sounds()
            .iter()
            .filter(|event| **event == SoundEvent::UnmappedSoundCommand { command: 0xEC })
            .count(),
    );

    if let Some(remaining) = frame.report.game_over_hall_of_fame_stall_remaining {
        report.saw_game_over_hall_stall = true;
        report.hall_stall_frames = report.hall_stall_frames.saturating_add(1);
        if frame.state.game_over
            != (GameOverSnapshot {
                hall_of_fame_stall_remaining: Some(remaining),
                ..GameOverSnapshot::NONE
            })
        {
            report.clean_exit = false;
        }
    }

    observe_post_game_draws(report, frame);
    observe_post_game_scene(report, frame);

    report.sprite_instances = report
        .sprite_instances
        .saturating_add(plan.sprite_instances);
    report.sprite_draw_commands = report
        .sprite_draw_commands
        .saturating_add(plan.sprite_draw_commands.len());
    report.wgpu_frame_commands = report
        .wgpu_frame_commands
        .saturating_add(plan.frame_plan.command_count());
    report.missing_sprite_regions = report
        .missing_sprite_regions
        .saturating_add(plan.missing_sprite_regions);
    signatures.insert(scene_signature(frame, plan));
}

fn observe_attract_cycle_draws(report: &mut ActorAttractCycleSmokeReport, frame: &ActorFrame) {
    let hall_title = source_message_text("HALLD_TITLE").expect("HALLD_TITLE message is checked in");
    let final_scoring_label = source_message_text("SWARMV").expect("SWARMV message is checked in");
    let mut cycle_has_first_williams_step = false;
    let mut cycle_has_scoring_surface = false;
    let mut cycle_has_final_label = false;

    for draw in &frame.report.draws {
        if draw.sprite == SpriteKey::WilliamsLogo
            && matches!(draw.effect, VisualEffect::WilliamsReveal { .. })
        {
            report.saw_williams_reveal = true;
        }
        if draw.sprite == SpriteKey::DefenderCoalescence {
            report.saw_defender_coalescence = true;
        }
        if draw.text.as_deref() == Some(hall_title) {
            report.saw_hall_of_fame = true;
        }
        if matches!(draw.effect, VisualEffect::AttractScoringSurface { .. }) {
            report.saw_scoring_surface = true;
            cycle_has_scoring_surface = true;
        }
        if draw.text.as_deref() == Some(final_scoring_label) {
            report.saw_final_scoring_label = true;
            cycle_has_final_label = true;
        }
        if frame.report.step == report.cycle_steps
            && draw.sprite == SpriteKey::WilliamsLogo
            && matches!(
                draw.effect,
                VisualEffect::WilliamsReveal { stroke_step: 1, .. }
            )
        {
            cycle_has_first_williams_step = true;
        }
    }

    if frame.report.step == report.cycle_steps {
        report.saw_cycle_return =
            cycle_has_first_williams_step && !cycle_has_scoring_surface && !cycle_has_final_label;
    }
}

fn observe_post_game_draws(report: &mut ActorPostGameSmokeReport, frame: &ActorFrame) {
    let hall_title = source_message_text("HALLD_TITLE").expect("HALLD_TITLE message is checked in");

    for draw in &frame.report.draws {
        match draw.sprite {
            SpriteKey::PlayerRight | SpriteKey::PlayerLeft => report.saw_player_sprite = true,
            SpriteKey::Pod => report.saw_pod_sprite = true,
            SpriteKey::Explosion => report.saw_explosion_pixels = true,
            SpriteKey::WilliamsLogo
                if frame.report.phase == Phase::Attract
                    && matches!(draw.effect, VisualEffect::WilliamsReveal { .. }) =>
            {
                report.saw_return_williams_reveal = true;
            }
            _ => {}
        }
        if draw.text.as_deref() == Some(hall_title) {
            report.saw_hall_of_fame = true;
        }
    }
}

fn observe_attract_cycle_scene(report: &mut ActorAttractCycleSmokeReport, frame: &ActorFrame) {
    for sprite in &frame.scene.sprites {
        if sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL {
            report.saw_williams_reveal = true;
        }
        if SpriteId::attract_defender_wordmark_block(0) == Some(sprite.sprite) {
            report.saw_defender_coalescence = true;
        }
        if sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO {
            report.saw_hall_of_fame = true;
        }
    }
}

fn observe_post_game_scene(report: &mut ActorPostGameSmokeReport, frame: &ActorFrame) {
    for sprite in &frame.scene.sprites {
        match sprite.sprite {
            SpriteId::PLAYER_SHIP | SpriteId::PLAYER_SHIP_LEFT => report.saw_player_sprite = true,
            SpriteId::ENEMY_POD => report.saw_pod_sprite = true,
            SpriteId::PLAYER_EXPLOSION_PIXEL => report.saw_explosion_pixels = true,
            SpriteId::HALL_OF_FAME_DEFENDER_LOGO => report.saw_hall_of_fame = true,
            SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL if frame.report.phase == Phase::Attract => {
                report.saw_return_williams_reveal = true;
            }
            _ => {}
        }
    }
}

fn default_attract_cycle_steps() -> anyhow::Result<u64> {
    AttractScript::red_label_title()
        .manifest()
        .cycle_steps
        .ok_or_else(|| anyhow::anyhow!("default actor attract script does not declare a cycle"))
}

fn required_sprite_label(sprite: SpriteId) -> Option<&'static str> {
    match sprite {
        SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL => Some("attract_williams_logo_pixel"),
        SpriteId::PLAYER_SHIP | SpriteId::PLAYER_SHIP_LEFT => Some("player_ship"),
        SpriteId::ENEMY_LANDER => Some("enemy_lander"),
        SpriteId::HUMAN => Some("human"),
        SpriteId::PLAYER_PROJECTILE => Some("player_projectile"),
        SpriteId::SCORE_DIGIT_0 => Some("score_digit_0"),
        _ => None,
    }
}

fn required_pipeline_label(pipeline: NativeRenderPipeline) -> Option<&'static str> {
    match pipeline {
        NativeRenderPipeline::Sprites => Some("sprites"),
        NativeRenderPipeline::Projectiles => Some("projectiles"),
        NativeRenderPipeline::HudText => Some("hud_text"),
        _ => None,
    }
}

fn record_draw_command(report: &mut ActorSmokeReport, layer: RenderLayer) {
    match layer {
        RenderLayer::Objects => {
            report.object_draw_commands = report.object_draw_commands.saturating_add(1);
        }
        RenderLayer::Projectiles => {
            report.projectile_draw_commands = report.projectile_draw_commands.saturating_add(1);
        }
        RenderLayer::Hud => {
            report.hud_draw_commands = report.hud_draw_commands.saturating_add(1);
        }
        RenderLayer::Overlay => {
            report.overlay_draw_commands = report.overlay_draw_commands.saturating_add(1);
        }
        RenderLayer::Terrain | RenderLayer::Starfield => {}
    }
}

fn surface_tuple(surface: crate::renderer::SurfaceSize) -> (u32, u32) {
    (surface.width, surface.height)
}

fn scene_signature(frame: &ActorFrame, plan: &SceneDrawPlan) -> u64 {
    let mut signature = 0xD1B5_4A32_D192_ED03_u64;
    signature = mix_signature(signature, frame.report.step);
    signature = mix_signature(signature, phase_code(frame.report.phase));
    signature = mix_signature(signature, u64::from(frame.report.credits));
    signature = mix_signature(signature, u64::from(frame.report.wave));
    signature = mix_signature(signature, u64::from(frame.report.score));
    signature = mix_signature(signature, frame.events.gameplay().len() as u64);
    signature = mix_signature(signature, frame.events.sounds().len() as u64);
    signature = mix_signature(signature, frame.scene.summary().sprite_count as u64);
    signature = mix_signature(signature, plan.sprite_instances as u64);
    signature = mix_signature(signature, plan.sprite_draw_commands.len() as u64);
    signature
}

fn mix_signature(current: u64, value: u64) -> u64 {
    current
        ^ value
            .wrapping_add(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(current << 6)
            .wrapping_add(current >> 2)
}

fn phase_code(phase: Phase) -> u64 {
    match phase {
        Phase::Attract => 1,
        Phase::Playing => 2,
        Phase::GameOver => 3,
        Phase::HighScoreEntry => 4,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScriptedInput {
    value: GameInput,
    label: Option<&'static str>,
}

fn smoke_input(frame_index: u32) -> ScriptedInput {
    let (value, label) = match frame_index {
        ACTOR_SMOKE_COIN_FRAME => (
            GameInput {
                coin: true,
                ..GameInput::NONE
            },
            Some("coin"),
        ),
        ACTOR_SMOKE_START_FRAME => (
            GameInput {
                start_one: true,
                ..GameInput::NONE
            },
            Some("start_one"),
        ),
        ACTOR_SMOKE_FIRE_FRAME => (
            GameInput {
                fire: true,
                ..GameInput::NONE
            },
            Some("fire"),
        ),
        ACTOR_SMOKE_THRUST_FRAME => (
            GameInput {
                thrust: true,
                ..GameInput::NONE
            },
            Some("thrust"),
        ),
        ACTOR_SMOKE_REVERSE_FRAME => (
            GameInput {
                reverse: true,
                ..GameInput::NONE
            },
            Some("reverse"),
        ),
        ACTOR_SMOKE_SMART_BOMB_FRAME => (
            GameInput {
                smart_bomb: true,
                ..GameInput::NONE
            },
            Some("smart_bomb"),
        ),
        ACTOR_SMOKE_HYPERSPACE_FRAME => (
            GameInput {
                hyperspace: true,
                ..GameInput::NONE
            },
            Some("hyperspace"),
        ),
        ACTOR_SMOKE_ALTITUDE_DOWN_FRAME => (
            GameInput {
                altitude_down: true,
                ..GameInput::NONE
            },
            Some("altitude_down"),
        ),
        _ => (GameInput::NONE, None),
    };

    ScriptedInput { value, label }
}

fn record_unique_label(labels: &mut Vec<String>, label: &str) {
    if !labels.iter().any(|existing| existing == label) {
        labels.push(label.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ACTOR_SMOKE_COIN_FRAME, ACTOR_SMOKE_FIRE_FRAME, ACTOR_SMOKE_FRAMES,
        ACTOR_SMOKE_START_FRAME, ACTOR_SMOKE_THRUST_FRAME, ActorAttractCycleSmokeReport,
        ActorPostGameSmokeReport, ActorSmokeReport, attract_cycle_report,
        default_attract_cycle_report, default_post_game_report, smoke_actor_input,
        smoke_frame_count, smoke_input, smoke_report,
    };

    #[test]
    fn smoke_report_exercises_actor_runtime_and_native_draw_plans() {
        let report = smoke_report(ACTOR_SMOKE_FRAMES).expect("actor smoke report");

        assert_eq!(report.frames, ACTOR_SMOKE_FRAMES);
        assert!(report.saw_attract);
        assert!(report.saw_credit);
        assert!(report.saw_playing);
        assert!(report.actor_event_frames > 0);
        assert!(report.actor_sound_events > 0);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.wgpu_frame_commands > 0);
        assert_eq!(report.temporary_raster_commands, 0);
        assert_eq!(report.missing_sprite_regions, 0);
        assert!(report.clean_exit);
    }

    #[test]
    fn smoke_report_rejects_zero_frames() {
        let error = smoke_report(0).expect_err("zero-frame smoke should fail");

        assert_eq!(
            error.to_string(),
            "actor smoke frame count must be positive"
        );
    }

    #[test]
    fn attract_cycle_report_exercises_default_actor_attract_loop() {
        let report = default_attract_cycle_report().expect("actor attract cycle smoke report");

        assert_eq!(report.frames, 3367);
        assert_eq!(report.cycle_steps, 3367);
        assert_eq!(report.attract_frames, report.frames);
        assert_eq!(report.playing_frames, 0);
        assert_eq!(report.game_over_frames, 0);
        assert_eq!(report.high_score_entry_frames, 0);
        assert_eq!(report.actor_event_frames, 0);
        assert_eq!(report.actor_sound_frames, 0);
        assert_eq!(report.actor_sound_events, 0);
        assert!(report.distinct_scene_signatures >= 8);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.wgpu_frame_commands > 0);
        assert_eq!(report.missing_sprite_regions, 0);
        assert!(report.saw_williams_reveal);
        assert!(report.saw_defender_coalescence);
        assert!(report.saw_hall_of_fame);
        assert!(report.saw_scoring_surface);
        assert!(report.saw_final_scoring_label);
        assert!(report.saw_cycle_return);
        assert!(report.clean_exit);
    }

    #[test]
    fn attract_cycle_report_rejects_zero_frames() {
        let error = attract_cycle_report(0).expect_err("zero-frame attract smoke should fail");

        assert_eq!(
            error.to_string(),
            "actor attract smoke frame count must be positive"
        );
    }

    #[test]
    fn post_game_report_exercises_high_score_hall_return() {
        let report = default_post_game_report().expect("actor post-game smoke report");

        assert!(report.frames >= 60);
        assert!(report.playing_frames > 0);
        assert!(report.high_score_entry_frames > 0);
        assert_eq!(report.game_over_frames, 60);
        assert_eq!(report.hall_stall_frames, 60);
        assert!(report.attract_frames > 0);
        assert_eq!(report.forced_player_collisions, 3);
        assert!(report.final_score >= 3_000);
        assert_eq!(report.final_lives, 0);
        assert!(report.player_destroyed_events >= 3);
        assert!(report.game_over_events > 0);
        assert!(report.high_score_entry_events > 0);
        assert_eq!(report.high_score_initial_accept_events, 3);
        assert_eq!(report.high_score_submit_events, 1);
        assert!(report.game_over_sound_events > 0);
        assert!(report.saw_game_over_hall_stall);
        assert!(report.saw_attract_return);
        assert!(report.saw_return_williams_reveal);
        assert!(report.saw_player_sprite);
        assert!(report.saw_pod_sprite);
        assert!(report.saw_explosion_pixels);
        assert!(report.saw_hall_of_fame);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.wgpu_frame_commands > 0);
        assert_eq!(report.missing_sprite_regions, 0);
        assert!(report.clean_exit);
    }

    #[test]
    fn attract_cycle_report_validates_required_default_milestones() {
        let mut report = ActorAttractCycleSmokeReport {
            frames: 3367,
            cycle_steps: 3367,
            distinct_scene_signatures: 8,
            attract_frames: 3367,
            sprite_instances: 1,
            sprite_draw_commands: 1,
            wgpu_frame_commands: 1,
            saw_williams_reveal: true,
            saw_defender_coalescence: true,
            saw_hall_of_fame: true,
            saw_scoring_surface: true,
            saw_final_scoring_label: true,
            saw_cycle_return: true,
            clean_exit: true,
            ..ActorAttractCycleSmokeReport::default()
        };

        report.saw_cycle_return = false;
        let error = report
            .validate()
            .expect_err("missing cycle return should fail");
        assert_eq!(
            error.to_string(),
            "actor attract smoke did not return to Williams after cycle boundary"
        );

        report.saw_cycle_return = true;
        report.actor_sound_frames = 1;
        let error = report
            .validate()
            .expect_err("attract sound event should fail");
        assert_eq!(
            error.to_string(),
            "actor attract smoke unexpectedly produced sound events"
        );
    }

    #[test]
    fn post_game_report_validates_high_score_return_contract() {
        let mut report = ActorPostGameSmokeReport {
            frames: 72,
            distinct_scene_signatures: 6,
            playing_frames: 10,
            high_score_entry_frames: 3,
            game_over_frames: 60,
            attract_frames: 1,
            forced_player_collisions: 3,
            final_score: 3_000,
            player_destroyed_events: 3,
            game_over_events: 1,
            high_score_entry_events: 1,
            high_score_initial_accept_events: 3,
            high_score_submit_events: 1,
            actor_sound_frames: 4,
            actor_sound_events: 6,
            game_over_sound_events: 1,
            saw_game_over_hall_stall: true,
            hall_stall_frames: 60,
            saw_attract_return: true,
            saw_return_williams_reveal: true,
            saw_player_sprite: true,
            saw_pod_sprite: true,
            saw_explosion_pixels: true,
            saw_hall_of_fame: true,
            sprite_instances: 1,
            sprite_draw_commands: 1,
            wgpu_frame_commands: 1,
            clean_exit: true,
            ..ActorPostGameSmokeReport::default()
        };

        report.hall_stall_frames = 59;
        let error = report.validate().expect_err("short Hall stall should fail");
        assert_eq!(
            error.to_string(),
            "actor post-game smoke did not observe the 60-step Hall-of-Fame stall"
        );

        report.hall_stall_frames = 60;
        report.saw_attract_return = false;
        let error = report
            .validate()
            .expect_err("missing attract return should fail");
        assert_eq!(
            error.to_string(),
            "actor post-game smoke did not return to attract"
        );
    }

    #[test]
    fn smoke_report_validates_required_actor_play_states() {
        let mut report = ActorSmokeReport {
            frames: 1,
            first_frame_size: Some((292, 240)),
            distinct_scene_signatures: 3,
            sprite_frames: 1,
            sprite_instances: 1,
            sprite_draw_commands: 1,
            object_sprites: 1,
            projectile_sprites: 1,
            hud_sprites: 1,
            overlay_sprites: 1,
            actor_event_frames: 1,
            actor_sound_frames: 1,
            actor_sound_events: 1,
            covered_sprites: super::REQUIRED_SPRITES
                .iter()
                .map(|label| (*label).to_owned())
                .collect(),
            object_draw_commands: 1,
            projectile_draw_commands: 1,
            hud_draw_commands: 1,
            overlay_draw_commands: 1,
            covered_pipelines: super::REQUIRED_PIPELINES
                .iter()
                .map(|label| (*label).to_owned())
                .collect(),
            wgpu_frame_commands: 1,
            injected_inputs: super::REQUIRED_INPUTS
                .iter()
                .map(|label| (*label).to_owned())
                .collect(),
            clean_exit: true,
            ..ActorSmokeReport::default()
        };

        let error = report
            .validate()
            .expect_err("missing attract should fail validation");
        assert_eq!(
            error.to_string(),
            "actor smoke did not observe attract frames"
        );

        report.saw_attract = true;
        report.attract_frames = 1;
        let error = report
            .validate()
            .expect_err("missing credited attract should fail validation");
        assert_eq!(
            error.to_string(),
            "actor smoke did not observe a credited attract frame"
        );

        report.saw_credit = true;
        report.credited_frames = 1;
        let error = report
            .validate()
            .expect_err("missing playing should fail validation");
        assert_eq!(
            error.to_string(),
            "actor smoke did not observe playing frames"
        );
    }

    #[test]
    fn smoke_script_uses_release_frames_between_edge_inputs() {
        assert!(smoke_input(ACTOR_SMOKE_COIN_FRAME).value.coin);
        assert_eq!(
            smoke_input(ACTOR_SMOKE_COIN_FRAME + 1).value,
            crate::actor_game::GameInput::NONE
        );
        assert!(smoke_input(ACTOR_SMOKE_START_FRAME).value.start_one);
        assert_eq!(
            smoke_input(ACTOR_SMOKE_START_FRAME + 1).value,
            crate::actor_game::GameInput::NONE
        );
        assert!(smoke_input(ACTOR_SMOKE_FIRE_FRAME).value.fire);
        assert_eq!(
            smoke_input(ACTOR_SMOKE_FIRE_FRAME + 1).value,
            crate::actor_game::GameInput::NONE
        );
        assert!(smoke_input(ACTOR_SMOKE_THRUST_FRAME).value.thrust);
        assert_eq!(
            smoke_input(ACTOR_SMOKE_THRUST_FRAME + 1).value,
            crate::actor_game::GameInput::NONE
        );
    }

    #[test]
    fn smoke_script_helpers_match_current_actor_smoke_contract() {
        assert_eq!(smoke_frame_count(), ACTOR_SMOKE_FRAMES);
        assert!(smoke_actor_input(ACTOR_SMOKE_COIN_FRAME).coin);
        assert!(smoke_actor_input(ACTOR_SMOKE_START_FRAME).start_one);
        assert!(smoke_actor_input(ACTOR_SMOKE_FIRE_FRAME).fire);
        assert_eq!(
            smoke_actor_input(ACTOR_SMOKE_FIRE_FRAME + 1),
            crate::actor_game::GameInput::NONE
        );
    }

    #[test]
    fn smoke_report_formats_current_cli_output() {
        let report = ActorSmokeReport {
            frames: 3,
            first_frame_size: Some((292, 240)),
            distinct_scene_signatures: 2,
            saw_attract: true,
            attract_frames: 1,
            saw_credit: true,
            credited_frames: 1,
            saw_playing: true,
            playing_frames: 2,
            actor_event_frames: 2,
            actor_sound_frames: 2,
            actor_sound_events: 3,
            sprite_frames: 3,
            sprite_instances: 12,
            sprite_draw_commands: 4,
            object_sprites: 2,
            projectile_sprites: 1,
            hud_sprites: 3,
            overlay_sprites: 6,
            covered_sprites: vec!["player_ship".to_owned(), "enemy_lander".to_owned()],
            object_draw_commands: 1,
            projectile_draw_commands: 1,
            hud_draw_commands: 1,
            overlay_draw_commands: 1,
            covered_pipelines: vec!["sprites".to_owned(), "hud_text".to_owned()],
            wgpu_frame_commands: 9,
            injected_inputs: vec!["coin".to_owned(), "start_one".to_owned()],
            clean_exit: true,
            ..ActorSmokeReport::default()
        };

        let text = report.to_text();

        assert!(text.starts_with("actor smoke passed\n"));
        assert!(text.contains("first_frame_size: 292x240"));
        assert!(text.contains("covered_sprites: player_ship,enemy_lander"));
        assert!(text.contains("injected_inputs: coin,start_one"));
    }

    #[test]
    fn attract_cycle_report_formats_current_cli_output() {
        let report = ActorAttractCycleSmokeReport {
            frames: 3367,
            cycle_steps: 3367,
            distinct_scene_signatures: 42,
            attract_frames: 3367,
            sprite_instances: 1200,
            sprite_draw_commands: 340,
            wgpu_frame_commands: 680,
            saw_williams_reveal: true,
            saw_defender_coalescence: true,
            saw_hall_of_fame: true,
            saw_scoring_surface: true,
            saw_final_scoring_label: true,
            saw_cycle_return: true,
            clean_exit: true,
            ..ActorAttractCycleSmokeReport::default()
        };

        let text = report.to_text();

        assert!(text.starts_with("actor attract smoke passed\n"));
        assert!(text.contains("cycle_steps: 3367"));
        assert!(text.contains("saw_scoring_surface: true"));
        assert!(text.contains("saw_cycle_return: true"));
    }

    #[test]
    fn post_game_report_formats_current_cli_output() {
        let report = ActorPostGameSmokeReport {
            frames: 72,
            distinct_scene_signatures: 9,
            playing_frames: 8,
            high_score_entry_frames: 3,
            game_over_frames: 60,
            attract_frames: 1,
            forced_player_collisions: 3,
            final_score: 3_000,
            player_destroyed_events: 3,
            game_over_events: 1,
            high_score_entry_events: 3,
            high_score_initial_accept_events: 3,
            high_score_submit_events: 1,
            actor_sound_frames: 4,
            actor_sound_events: 6,
            game_over_sound_events: 1,
            hall_stall_frames: 60,
            saw_attract_return: true,
            saw_return_williams_reveal: true,
            saw_player_sprite: true,
            saw_pod_sprite: true,
            saw_explosion_pixels: true,
            saw_hall_of_fame: true,
            sprite_instances: 44,
            sprite_draw_commands: 22,
            wgpu_frame_commands: 66,
            clean_exit: true,
            ..ActorPostGameSmokeReport::default()
        };

        let text = report.to_text();

        assert!(text.starts_with("actor post-game smoke passed\n"));
        assert!(text.contains("forced_player_collisions: 3"));
        assert!(text.contains("hall_stall_frames: 60"));
        assert!(text.contains("saw_attract_return: true"));
    }
}
