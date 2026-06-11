const GAME_OVER_OBSERVER_SOUND_COMMAND: crate::SoundCommand = crate::SoundCommand::new(0xEC);
const SCENE_SIGNATURE_INITIAL: u64 = 0xD1B5_4A32_D192_ED03;
const SCENE_SIGNATURE_GOLDEN_RATIO_MIX: u64 = 0x9E37_79B9_7F4A_7C15;

fn step_post_game(
    runtime: &mut ActorRuntimeAdapter,
    renderer: &NativeSceneRenderer,
    report: &mut ActorPostGameSmokeReport,
    signatures: &mut BTreeSet<u64>,
    input: GameInput,
) -> ActorStepSnapshot {
    let step = runtime.step(input);
    let plan = renderer.prepare(&step.scene);
    observe_post_game_step(report, signatures, &step, &plan);
    step
}

fn step_until_player_position(
    runtime: &mut ActorRuntimeAdapter,
    renderer: &NativeSceneRenderer,
    report: &mut ActorPostGameSmokeReport,
    signatures: &mut BTreeSet<u64>,
) -> anyhow::Result<Point> {
    for _ in 0..POST_GAME_PLAYER_RESPAWN_SEARCH_STEPS {
        let step = step_post_game(runtime, renderer, report, signatures, GameInput::NONE);
        if let Some(position) = step
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

fn observe_step(
    report: &mut ActorSmokeReport,
    signatures: &mut BTreeSet<u64>,
    step: &ActorStepSnapshot,
    plan: &SceneDrawPlan,
) {
    report.steps = report.steps.saturating_add(1);
    report
        .initial_surface_size
        .get_or_insert(surface_tuple(step.scene.surface));
    report.saw_attract |= step.report.phase == Phase::Attract;
    report.saw_credit |= step.report.phase == Phase::Attract && step.report.credits > 0;
    report.saw_playing |= step.report.phase == Phase::Playing;

    if step.report.phase == Phase::Attract {
        report.attract_steps = report.attract_steps.saturating_add(1);
        if step.report.credits > 0 {
            report.credited_steps = report.credited_steps.saturating_add(1);
        }
    }
    if step.report.phase == Phase::Playing {
        report.playing_steps = report.playing_steps.saturating_add(1);
    }
    if !step.events.gameplay().is_empty() {
        report.actor_event_steps = report.actor_event_steps.saturating_add(1);
    }
    if !step.events.sounds().is_empty() {
        report.actor_sound_steps = report.actor_sound_steps.saturating_add(1);
        report.actor_sound_events = report
            .actor_sound_events
            .saturating_add(step.events.sounds().len());
    }

    let summary = step.scene.summary();
    if summary.sprite_count > 0 {
        report.sprite_steps = report.sprite_steps.saturating_add(1);
    }
    report.object_sprites = report.object_sprites.saturating_add(summary.layers.objects);
    report.projectile_sprites = report
        .projectile_sprites
        .saturating_add(summary.layers.projectiles);
    report.hud_sprites = report.hud_sprites.saturating_add(summary.layers.hud);
    report.overlay_sprites = report
        .overlay_sprites
        .saturating_add(summary.layers.overlay);
    for sprite in &step.scene.sprites {
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
    report.wgpu_render_commands = report
        .wgpu_render_commands
        .saturating_add(plan.frame_plan.command_count());
    report.temporary_raster_commands = report
        .temporary_raster_commands
        .saturating_add(plan.frame_plan.temporary_raster_count());
    report.missing_sprite_regions = report
        .missing_sprite_regions
        .saturating_add(plan.missing_sprite_regions);
    signatures.insert(scene_signature(step, plan));
}

fn observe_attract_cycle_step(
    report: &mut ActorAttractCycleSmokeReport,
    signatures: &mut BTreeSet<u64>,
    step: &ActorStepSnapshot,
    plan: &SceneDrawPlan,
) {
    report.steps = report.steps.saturating_add(1);
    match step.report.phase {
        Phase::Attract => report.attract_steps = report.attract_steps.saturating_add(1),
        Phase::Playing => report.playing_steps = report.playing_steps.saturating_add(1),
        Phase::GameOver => report.game_over_steps = report.game_over_steps.saturating_add(1),
        Phase::HighScoreEntry => {
            report.high_score_entry_steps = report.high_score_entry_steps.saturating_add(1);
        }
    }

    if !step.events.gameplay().is_empty() {
        report.actor_event_steps = report.actor_event_steps.saturating_add(1);
    }
    if !step.events.sounds().is_empty() {
        report.actor_sound_steps = report.actor_sound_steps.saturating_add(1);
        report.actor_sound_events = report
            .actor_sound_events
            .saturating_add(step.events.sounds().len());
    }

    observe_attract_cycle_draws(report, step);
    observe_attract_cycle_scene(report, step);

    report.sprite_instances = report
        .sprite_instances
        .saturating_add(plan.sprite_instances);
    report.sprite_draw_commands = report
        .sprite_draw_commands
        .saturating_add(plan.sprite_draw_commands.len());
    report.wgpu_render_commands = report
        .wgpu_render_commands
        .saturating_add(plan.frame_plan.command_count());
    report.missing_sprite_regions = report
        .missing_sprite_regions
        .saturating_add(plan.missing_sprite_regions);
    signatures.insert(scene_signature(step, plan));
}

fn observe_post_game_step(
    report: &mut ActorPostGameSmokeReport,
    signatures: &mut BTreeSet<u64>,
    step: &ActorStepSnapshot,
    plan: &SceneDrawPlan,
) {
    report.steps = report.steps.saturating_add(1);
    match step.report.phase {
        Phase::Attract => report.attract_steps = report.attract_steps.saturating_add(1),
        Phase::Playing => report.playing_steps = report.playing_steps.saturating_add(1),
        Phase::GameOver => report.game_over_steps = report.game_over_steps.saturating_add(1),
        Phase::HighScoreEntry => {
            report.high_score_entry_steps = report.high_score_entry_steps.saturating_add(1);
        }
    }

    report.final_score = step.report.score;
    report.final_lives = step.report.lives;

    for event in step.events.gameplay() {
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

    if !step.events.sounds().is_empty() {
        report.actor_sound_steps = report.actor_sound_steps.saturating_add(1);
        report.actor_sound_events = report
            .actor_sound_events
            .saturating_add(step.events.sounds().len());
    }
    report.game_over_sound_events = report.game_over_sound_events.saturating_add(
        step
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

    if let Some(remaining) = step.report.game_over_hall_of_fame_stall_remaining {
        report.saw_game_over_hall_stall = true;
        report.hall_stall_steps = report.hall_stall_steps.saturating_add(1);
        if step.state.game_over
            != (GameOverSnapshot {
                hall_of_fame_stall_remaining: Some(remaining),
                ..GameOverSnapshot::NONE
            })
        {
            report.clean_exit = false;
        }
    }

    observe_post_game_draws(report, step);
    observe_post_game_scene(report, step);

    report.sprite_instances = report
        .sprite_instances
        .saturating_add(plan.sprite_instances);
    report.sprite_draw_commands = report
        .sprite_draw_commands
        .saturating_add(plan.sprite_draw_commands.len());
    report.wgpu_render_commands = report
        .wgpu_render_commands
        .saturating_add(plan.frame_plan.command_count());
    report.missing_sprite_regions = report
        .missing_sprite_regions
        .saturating_add(plan.missing_sprite_regions);
    signatures.insert(scene_signature(step, plan));
}

fn observe_attract_cycle_draws(report: &mut ActorAttractCycleSmokeReport, step: &ActorStepSnapshot) {
    let hall_title = message_text(MessageId::HallTitle);
    let final_scoring_instruction = message_text(MessageId::SwarmerInstruction);
    let mut cycle_has_first_williams_step = false;
    let mut cycle_has_scoring_surface = false;
    let mut cycle_has_final_instruction = false;

    for draw in &step.report.draws {
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
        if step.report.step == report.cycle_steps
            && draw.sprite == SpriteKey::WilliamsLogo
            && matches!(
                draw.effect,
                VisualEffect::WilliamsReveal { stroke_step: 1, .. }
            )
        {
            cycle_has_first_williams_step = true;
        }
    }

    if step.report.step == report.cycle_steps {
        report.saw_cycle_return =
            cycle_has_first_williams_step && !cycle_has_scoring_surface && !cycle_has_final_instruction;
    }
}

fn observe_post_game_draws(report: &mut ActorPostGameSmokeReport, step: &ActorStepSnapshot) {
    let hall_title = message_text(MessageId::HallTitle);

    for draw in &step.report.draws {
        match draw.sprite {
            SpriteKey::PlayerRight | SpriteKey::PlayerLeft => report.saw_player_sprite = true,
            SpriteKey::Pod => report.saw_pod_sprite = true,
            SpriteKey::Explosion => report.saw_explosion_pixels = true,
            SpriteKey::WilliamsLogo
                if step.report.phase == Phase::Attract
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

fn observe_attract_cycle_scene(report: &mut ActorAttractCycleSmokeReport, step: &ActorStepSnapshot) {
    for sprite in &step.scene.sprites {
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

fn observe_post_game_scene(report: &mut ActorPostGameSmokeReport, step: &ActorStepSnapshot) {
    for sprite in &step.scene.sprites {
        match sprite.sprite {
            SpriteId::PLAYER_SHIP | SpriteId::PLAYER_SHIP_LEFT => report.saw_player_sprite = true,
            SpriteId::ENEMY_POD => report.saw_pod_sprite = true,
            SpriteId::PLAYER_EXPLOSION_PIXEL => report.saw_explosion_pixels = true,
            SpriteId::HALL_OF_FAME_DEFENDER_LOGO => report.saw_hall_of_fame = true,
            SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL if step.report.phase == Phase::Attract => {
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

fn scene_signature(step: &ActorStepSnapshot, plan: &SceneDrawPlan) -> u64 {
    let mut signature = SCENE_SIGNATURE_INITIAL;
    signature = mix_signature(signature, step.report.step);
    signature = mix_signature(signature, phase_code(step.report.phase));
    signature = mix_signature(signature, u64::from(step.report.credits));
    signature = mix_signature(signature, u64::from(step.report.wave));
    signature = mix_signature(signature, u64::from(step.report.score));
    signature = mix_signature(signature, step.events.gameplay().len() as u64);
    signature = mix_signature(signature, step.events.sounds().len() as u64);
    signature = mix_signature(signature, step.scene.summary().sprite_count as u64);
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

fn smoke_input(step_index: u32) -> ScriptedInput {
    let (value, label) = match step_index {
        ACTOR_SMOKE_COIN_STEP => (
            GameInput {
                coin: true,
                ..GameInput::NONE
            },
            Some("coin"),
        ),
        ACTOR_SMOKE_START_STEP => (
            GameInput {
                start_one: true,
                ..GameInput::NONE
            },
            Some("start_one"),
        ),
        ACTOR_SMOKE_FIRE_STEP => (
            GameInput {
                fire: true,
                ..GameInput::NONE
            },
            Some("fire"),
        ),
        ACTOR_SMOKE_THRUST_STEP => (
            GameInput {
                thrust: true,
                ..GameInput::NONE
            },
            Some("thrust"),
        ),
        ACTOR_SMOKE_REVERSE_STEP => (
            GameInput {
                reverse: true,
                ..GameInput::NONE
            },
            Some("reverse"),
        ),
        ACTOR_SMOKE_SMART_BOMB_STEP => (
            GameInput {
                smart_bomb: true,
                ..GameInput::NONE
            },
            Some("smart_bomb"),
        ),
        ACTOR_SMOKE_HYPERSPACE_STEP => (
            GameInput {
                hyperspace: true,
                ..GameInput::NONE
            },
            Some("hyperspace"),
        ),
        ACTOR_SMOKE_ALTITUDE_DOWN_STEP => (
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
