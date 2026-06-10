const GAME_OVER_OBSERVER_SOUND_COMMAND: crate::SoundCommand = crate::SoundCommand::new(0xEC);
const SCENE_SIGNATURE_INITIAL: u64 = 0xD1B5_4A32_D192_ED03;
const SCENE_SIGNATURE_GOLDEN_RATIO_MIX: u64 = 0x9E37_79B9_7F4A_7C15;

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
            .filter(|event| {
                **event == SoundEvent::UnmappedSoundCommand {
                    command: GAME_OVER_OBSERVER_SOUND_COMMAND,
                }
            })
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
    let hall_title = message_text(MessageId::HallTitle);
    let final_scoring_instruction = message_text(MessageId::SwarmerInstruction);
    let mut cycle_has_first_williams_step = false;
    let mut cycle_has_scoring_surface = false;
    let mut cycle_has_final_instruction = false;

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
        if draw.text.as_deref() == Some(final_scoring_instruction) {
            report.saw_final_scoring_instruction = true;
            cycle_has_final_instruction = true;
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
            cycle_has_first_williams_step && !cycle_has_scoring_surface && !cycle_has_final_instruction;
    }
}

fn observe_post_game_draws(report: &mut ActorPostGameSmokeReport, frame: &ActorFrame) {
    let hall_title = message_text(MessageId::HallTitle);

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
    AttractScript::default_title()
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
    let mut signature = SCENE_SIGNATURE_INITIAL;
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
            .wrapping_add(SCENE_SIGNATURE_GOLDEN_RATIO_MIX)
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
