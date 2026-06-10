use std::collections::BTreeSet;

use anyhow::bail;

use crate::{
    actor_game::{
        ActorFrame, ActorKind, ActorRuntimeAdapter, AttractScript, GameInput, Phase, Point,
        SpriteKey, VisualEffect,
    },
    reference_assets::{MessageId, message_text},
    game::{GameEvent, GameOverSnapshot, SoundEvent},
    renderer::{
        NativeRenderPipeline, NativeSceneRenderer, RenderLayer, SceneDrawPlan, SpriteId,
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
    pub(crate) saw_final_scoring_instruction: bool,
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
        if !self.saw_final_scoring_instruction {
            bail!("actor attract smoke did not cover final scoring instruction");
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
            "actor attract smoke passed\n  frames: {}\n  cycle_steps: {}\n  distinct_scene_signatures: {}\n  attract_frames: {}\n  non_attract_frames: {}\n  actor_event_frames: {}\n  actor_sound_frames: {}\n  actor_sound_events: {}\n  sprite_instances: {}\n  sprite_draw_commands: {}\n  wgpu_frame_commands: {}\n  missing_sprite_regions: {}\n  saw_williams_reveal: {}\n  saw_defender_coalescence: {}\n  saw_hall_of_fame: {}\n  saw_scoring_surface: {}\n  saw_final_scoring_instruction: {}\n  saw_cycle_return: {}\n  clean_exit: {}\n",
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
            self.saw_final_scoring_instruction,
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
            bail!("actor post-game smoke did not render accepted explosion pixels");
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
