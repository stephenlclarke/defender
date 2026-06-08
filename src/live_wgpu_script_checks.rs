fn actor_scripts_from_path(path: &Path) -> anyhow::Result<ActorDriverScripts> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("reading actor driver script {}", path.display()))?;
    source
        .parse::<ActorDriverScripts>()
        .with_context(|| format!("parsing actor driver script {}", path.display()))
}

pub(crate) fn run_actor_script_check(path: &Path) -> anyhow::Result<ActorScriptCheckReport> {
    let scripts = actor_scripts_from_path(path)?;
    let mut runtime = ActorRuntimeAdapter::with_scripts(scripts.clone());
    let manifest = runtime.driver().script_manifest();
    let frame = runtime.step(ActorGameInput::NONE);
    let (attract_cycle, attract_cycle_unavailable_reason) =
        actor_script_check_attract_cycle(&mut runtime, manifest.attract_script.cycle_steps, &frame);
    let playing = run_actor_script_check_to_first_playing_wave(&mut runtime)?;
    let first_playing = actor_script_check_playing_summary(&playing);
    let (first_player_laser, first_player_laser_unavailable_reason) =
        actor_script_check_first_player_laser(scripts.clone())?;
    let (first_player_laser_hit, first_player_laser_hit_unavailable_reason) =
        actor_script_check_first_player_laser_hit(scripts.clone())?;
    let hostile_laser_hit_matrix = actor_script_check_hostile_laser_hit_matrix()?;
    let hostile_projectile_matrix = actor_script_check_hostile_projectile_matrix()?;
    let (first_source_projectile, first_source_projectile_unavailable_reason) =
        actor_script_check_first_source_projectile(scripts)?;
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
        first_playing_source_background_left: first_playing.background_left,
        first_playing_source_rng_seed: first_playing.source_rng_seed,
        first_playing_source_rng_hseed: first_playing.source_rng_hseed,
        first_playing_source_rng_lseed: first_playing.source_rng_lseed,
        first_playing_source_actor_samples: first_playing.source_actor_samples,
        first_playing_source_projectile_samples: first_playing.source_projectile_samples,
        first_playing_sound_commands: first_playing.sound_commands,
        first_player_laser,
        first_player_laser_unavailable_reason,
        first_player_laser_hit,
        first_player_laser_hit_unavailable_reason,
        hostile_laser_hit_matrix,
        hostile_projectile_matrix,
        first_source_projectile,
        first_source_projectile_unavailable_reason,
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

fn actor_script_check_first_source_projectile(
    scripts: ActorDriverScripts,
) -> anyhow::Result<(
    Option<ActorScriptCheckFirstSourceProjectileSummary>,
    Option<String>,
)> {
    let mut runtime = ActorRuntimeAdapter::with_scripts(scripts);
    let mut frame = run_actor_script_check_to_first_playing_wave(&mut runtime)?;
    let mut recent_projectile_sound_commands = actor_script_check_projectile_sound_commands(&frame);

    for sample_steps in 0..=ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_STEP_LIMIT {
        let samples = actor_script_check_source_projectile_samples(&frame);
        if !samples.is_empty() {
            let sound_commands = actor_script_check_projectile_sound_commands(&frame);
            let sound_commands = if sound_commands.is_empty() {
                recent_projectile_sound_commands
            } else {
                sound_commands
            };
            return Ok((
                Some(ActorScriptCheckFirstSourceProjectileSummary {
                    sample_steps: sample_steps as u32,
                    samples,
                    sound_commands,
                }),
                None,
            ));
        }

        if sample_steps == ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_STEP_LIMIT {
            break;
        }

        frame = runtime.step(ActorGameInput::NONE);
        let sound_commands = actor_script_check_projectile_sound_commands(&frame);
        if !sound_commands.is_empty() {
            recent_projectile_sound_commands = sound_commands;
        }
    }

    Ok((
        None,
        Some(format!(
            "source_projectile_not_observed_after_{}_steps",
            ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_STEP_LIMIT
        )),
    ))
}

fn actor_script_check_hostile_projectile_matrix()
-> anyhow::Result<Vec<ActorScriptCheckHostileProjectileSample>> {
    [
        ActorKind::Lander,
        ActorKind::Mutant,
        ActorKind::Swarmer,
        ActorKind::Baiter,
    ]
    .into_iter()
    .map(actor_script_check_hostile_projectile_matrix_sample_for)
    .collect()
}

fn actor_script_check_hostile_projectile_matrix_sample_for(
    kind: ActorKind,
) -> anyhow::Result<ActorScriptCheckHostileProjectileSample> {
    let kind_label = actor_script_check_source_actor_kind_label(kind);
    let source = actor_script_check_hostile_projectile_matrix_script(kind);
    let scripts = ActorDriverScripts::parse_text(&source).with_context(|| {
        format!("parsing built-in hostile projectile matrix script `{kind_label}`")
    })?;
    let mut runtime = ActorRuntimeAdapter::with_scripts(scripts);
    let mut frame = run_actor_script_check_to_first_playing_wave(&mut runtime)
        .with_context(|| format!("starting hostile projectile matrix script `{kind_label}`"))?;

    for sample_steps in 0..=ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_STEP_LIMIT {
        let samples = actor_script_check_projectile_spawn_command_samples(&frame);
        if !samples.is_empty() {
            let sound_commands = actor_script_check_projectile_sound_commands(&frame);
            return Ok(ActorScriptCheckHostileProjectileSample {
                kind: kind_label.to_string(),
                sample_steps: sample_steps as u32,
                samples,
                sound_commands,
            });
        }

        if sample_steps == ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_STEP_LIMIT {
            break;
        }

        frame = runtime.step(actor_script_check_hostile_projectile_matrix_input(kind));
    }

    anyhow::bail!(
        "hostile projectile matrix script `{kind_label}` did not observe a projectile after {} steps",
        ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_STEP_LIMIT
    );
}

fn actor_script_check_hostile_projectile_matrix_input(kind: ActorKind) -> ActorGameInput {
    if kind == ActorKind::Swarmer {
        return ActorGameInput {
            thrust: true,
            ..ActorGameInput::NONE
        };
    }

    ActorGameInput::NONE
}

fn actor_script_check_hostile_projectile_matrix_script(kind: ActorKind) -> String {
    let arcade_wave = match kind {
        ActorKind::Lander => {
            "arcade_wave 2 wave_size 1 landers 1 bombers 0 pods 0 mutants 0 swarmers 0 lander_shot_time 1\n"
        }
        ActorKind::Mutant => {
            "arcade_wave 1 wave_size 1 landers 0 bombers 0 pods 0 mutants 1 swarmers 0 mutant_shot_time 1 mutant_x_velocity 48 mutant_random_y 2\n"
        }
        ActorKind::Swarmer => {
            concat!(
                "arcade_wave 1 wave_size 0 landers 0 bombers 0 pods 0 mutants 0 swarmers 0\n",
                "swarmer 62 120\n",
            )
        }
        ActorKind::Baiter => {
            concat!(
                "arcade_wave 1 wave_size 0 landers 0 bombers 0 pods 0 mutants 0 swarmers 0 ",
                "baiter_time 1 baiter_shot_time 1 lander_shot_time 255\n",
                "lander 220 120\n",
            )
        }
        _ => "",
    };
    format!(
        concat!(
            "[attract]\n",
            "text 1 forever 12 20 PROJECTILE MATRIX\n",
            "[behavior]\n",
            "kind player player_takes_enemy_collision_damage false\n",
            "kind player player_speed 16\n",
            "kind lander lander_mode drift\n",
            "kind lander lander_drift_speed 0\n",
            "kind lander lander_fire_period_steps 18446744073709551615\n",
            "kind swarmer swarmer_mode drift\n",
            "kind swarmer swarmer_seek_speed 0\n",
            "kind swarmer swarmer_fire_period_steps 1\n",
            "[wave]\n",
            "name hostile projectile matrix {kind_label}\n",
            "{arcade_wave}",
            "human 100 214\n",
        ),
        kind_label = actor_script_check_source_actor_kind_label(kind),
        arcade_wave = arcade_wave
    )
}

fn actor_script_check_first_player_laser(
    scripts: ActorDriverScripts,
) -> anyhow::Result<(
    Option<ActorScriptCheckFirstPlayerLaserSummary>,
    Option<String>,
)> {
    let mut runtime = ActorRuntimeAdapter::with_scripts(scripts);
    let mut frame = run_actor_script_check_to_first_playing_wave(&mut runtime)?;
    let mut recent_laser_sound_commands = Vec::new();

    for sample_steps in 0..=ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT {
        let samples = actor_script_check_player_laser_samples(&frame);
        if !samples.is_empty() {
            let sound_commands = actor_script_check_laser_sound_commands(&frame);
            let sound_commands = if sound_commands.is_empty() {
                recent_laser_sound_commands
            } else {
                sound_commands
            };
            return Ok((
                Some(ActorScriptCheckFirstPlayerLaserSummary {
                    sample_steps: sample_steps as u32,
                    samples,
                    sound_commands,
                }),
                None,
            ));
        }

        if sample_steps == ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT {
            break;
        }

        let input = if sample_steps == 0 {
            ActorGameInput {
                fire: true,
                ..ActorGameInput::NONE
            }
        } else {
            ActorGameInput::NONE
        };
        frame = runtime.step(input);
        let sound_commands = actor_script_check_laser_sound_commands(&frame);
        if !sound_commands.is_empty() {
            recent_laser_sound_commands = sound_commands;
        }
    }

    Ok((
        None,
        Some(format!(
            "player_laser_not_observed_after_{}_steps",
            ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT
        )),
    ))
}

fn actor_script_check_first_player_laser_hit(
    scripts: ActorDriverScripts,
) -> anyhow::Result<(
    Option<ActorScriptCheckFirstPlayerLaserHitSummary>,
    Option<String>,
)> {
    let mut runtime = ActorRuntimeAdapter::with_scripts(scripts);
    let mut frame = run_actor_script_check_to_first_playing_wave(&mut runtime)?;

    for sample_steps in 0..=ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT {
        let explosion_samples = actor_script_check_explosion_command_samples(&frame);
        let sound_commands = actor_script_check_hit_sound_commands(&frame);
        if !explosion_samples.is_empty() {
            return Ok((
                Some(ActorScriptCheckFirstPlayerLaserHitSummary {
                    sample_steps: sample_steps as u32,
                    score: frame.report.score,
                    explosion_samples,
                    sound_commands,
                }),
                None,
            ));
        }

        if sample_steps == ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT {
            break;
        }

        let input = if sample_steps == 0 {
            ActorGameInput {
                fire: true,
                ..ActorGameInput::NONE
            }
        } else {
            ActorGameInput::NONE
        };
        frame = runtime.step(input);
    }

    Ok((
        None,
        Some(format!(
            "player_laser_hit_not_observed_after_{}_steps",
            ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT
        )),
    ))
}

fn actor_script_check_hostile_laser_hit_matrix()
-> anyhow::Result<Vec<ActorScriptCheckHostileLaserHitSample>> {
    [
        ActorKind::Lander,
        ActorKind::Mutant,
        ActorKind::Bomber,
        ActorKind::Pod,
        ActorKind::Swarmer,
        ActorKind::Baiter,
    ]
    .into_iter()
    .map(actor_script_check_hostile_laser_hit_matrix_sample_for)
    .collect()
}

fn actor_script_check_hostile_laser_hit_matrix_sample_for(
    kind: ActorKind,
) -> anyhow::Result<ActorScriptCheckHostileLaserHitSample> {
    let kind_label = actor_script_check_source_actor_kind_label(kind);
    let source = actor_script_check_hostile_laser_hit_matrix_script(kind);
    let scripts = ActorDriverScripts::parse_text(&source).with_context(|| {
        format!("parsing built-in hostile laser-hit matrix script `{kind_label}`")
    })?;
    let mut runtime = ActorRuntimeAdapter::with_scripts(scripts);
    let mut frame = run_actor_script_check_to_first_playing_wave(&mut runtime)
        .with_context(|| format!("starting hostile laser-hit matrix script `{kind_label}`"))?;
    let initial_score = frame.report.score;

    for sample_steps in 0..=ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT {
        let explosion_samples = actor_script_check_explosion_command_samples(&frame);
        let sound_commands = actor_script_check_hit_sound_commands(&frame);
        if !explosion_samples.is_empty() {
            return Ok(ActorScriptCheckHostileLaserHitSample {
                kind: kind_label.to_string(),
                sample_steps: sample_steps as u32,
                score_delta: frame.report.score.saturating_sub(initial_score),
                score: frame.report.score,
                explosion_samples,
                sound_commands,
                spawned_counts: actor_script_check_spawned_counts(&frame),
            });
        }

        if sample_steps == ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT {
            break;
        }

        let input = if sample_steps == 0 {
            ActorGameInput {
                fire: true,
                ..ActorGameInput::NONE
            }
        } else {
            ActorGameInput::NONE
        };
        frame = runtime.step(input);
    }

    anyhow::bail!(
        "hostile laser-hit matrix script `{kind_label}` did not observe a hit after {} steps",
        ACTOR_SCRIPT_CHECK_PLAYING_STEP_LIMIT
    );
}

fn actor_script_check_hostile_laser_hit_matrix_script(kind: ActorKind) -> String {
    let kind_label = actor_script_check_source_actor_kind_label(kind);
    format!(
        concat!(
            "[attract]\n",
            "text 1 forever 12 20 HIT MATRIX\n",
            "[behavior]\n",
            "kind player player_takes_enemy_collision_damage false\n",
            "kind lander lander_mode drift\n",
            "kind lander lander_drift_speed 0\n",
            "kind lander lander_fire_period_steps 18446744073709551615\n",
            "kind mutant mutant_mode drift\n",
            "kind mutant mutant_seek_speed 0\n",
            "kind bomber bomber_mode drift\n",
            "kind bomber bomber_drift_speed 0\n",
            "kind bomber bomber_bomb_period_steps 18446744073709551615\n",
            "kind pod pod_mode drift\n",
            "kind pod pod_drift_speed 0\n",
            "kind swarmer swarmer_mode drift\n",
            "kind swarmer swarmer_seek_speed 0\n",
            "kind swarmer swarmer_fire_period_steps 18446744073709551615\n",
            "kind baiter baiter_mode drift\n",
            "kind baiter baiter_seek_speed 0\n",
            "kind baiter baiter_fire_period_steps 18446744073709551615\n",
            "[wave]\n",
            "name hostile hit matrix {kind_label}\n",
            "wave 1\n",
            "{kind_label} 62 120\n",
            "lander 220 120\n",
            "human 100 214\n",
        ),
        kind_label = kind_label
    )
}

fn actor_script_check_playing_summary(frame: &ActorFrame) -> ActorScriptCheckPlayingSummary {
    let profile = frame.report.arcade_wave;
    let reserve = frame.state.world.enemy_reserve;
    debug_assert_eq!(reserve, frame.report.enemy_reserve);
    let arcade_rng = frame.report.arcade_rng;
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
        background_left: frame.report.background_left,
        source_rng_seed: arcade_rng.map(|arcade_rng| arcade_rng.seed),
        source_rng_hseed: arcade_rng.map(|arcade_rng| arcade_rng.hseed),
        source_rng_lseed: arcade_rng.map(|arcade_rng| arcade_rng.lseed),
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
        source_actor_samples: actor_script_check_source_actor_samples(frame),
        source_projectile_samples: actor_script_check_source_projectile_samples(frame),
        sound_commands: actor_script_check_sound_commands(frame),
    }
}

fn actor_script_check_source_actor_samples(
    frame: &ActorFrame,
) -> Vec<ActorScriptCheckSourceActorSample> {
    frame
        .report
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.alive)
        .filter_map(|snapshot| {
            actor_script_check_source_actor_fraction(snapshot).map(
                |(x_subpixel, y_subpixel)| ActorScriptCheckSourceActorSample {
                    kind: actor_script_check_source_actor_kind_label(snapshot.kind).to_string(),
                    x: snapshot.position.x,
                    y: snapshot.position.y,
                    x_subpixel,
                    y_subpixel,
                },
            )
        })
        .take(ACTOR_SCRIPT_CHECK_ACTOR_SAMPLE_LIMIT)
        .collect()
}

fn actor_script_check_source_actor_fraction(
    snapshot: &crate::actor_game::ActorSnapshot,
) -> Option<(u8, u8)> {
    match snapshot.kind {
        ActorKind::Lander => snapshot
            .lander_runtime
            .map(|source| (source.x_fraction, source.y_fraction)),
        ActorKind::Mutant => snapshot
            .mutant_runtime
            .map(|source| (source.x_fraction, source.y_fraction)),
        ActorKind::Bomber => snapshot
            .bomber_runtime
            .map(|source| (source.x_fraction, source.y_fraction)),
        ActorKind::Pod => snapshot
            .pod_runtime
            .map(|source| (source.x_fraction, source.y_fraction)),
        ActorKind::Swarmer => snapshot
            .swarmer_runtime
            .map(|source| (source.x_fraction, source.y_fraction)),
        ActorKind::Baiter => snapshot
            .baiter_runtime
            .map(|source| (source.x_fraction, source.y_fraction)),
        ActorKind::Human => snapshot
            .human_runtime
            .map(|source| (source.x_fraction, source.y_fraction)),
        _ => None,
    }
}

fn actor_script_check_source_actor_kind_label(kind: ActorKind) -> &'static str {
    match kind {
        ActorKind::Lander => "lander",
        ActorKind::Mutant => "mutant",
        ActorKind::Bomber => "bomber",
        ActorKind::Pod => "pod",
        ActorKind::Swarmer => "swarmer",
        ActorKind::Baiter => "baiter",
        ActorKind::Human => "human",
        _ => "actor",
    }
}

fn actor_script_check_source_projectile_samples(
    frame: &ActorFrame,
) -> Vec<ActorScriptCheckSourceProjectileSample> {
    frame
        .report
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.alive)
        .filter_map(|snapshot| {
            let source = snapshot.enemy_projectile_runtime?;
            let kind = match snapshot.kind {
                ActorKind::EnemyLaser => "enemy_laser",
                ActorKind::Bomb => "bomb",
                _ => return None,
            };
            Some(ActorScriptCheckSourceProjectileSample {
                kind: kind.to_string(),
                x: snapshot.position.x,
                y: snapshot.position.y,
                x_subpixel: source.x_fraction,
                y_subpixel: source.y_fraction,
                x_velocity_word: source.x_velocity,
                y_velocity_word: source.y_velocity,
                lifetime_ticks: source.lifetime_ticks,
            })
        })
        .take(ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_LIMIT)
        .collect()
}

fn actor_script_check_projectile_spawn_command_samples(
    frame: &ActorFrame,
) -> Vec<ActorScriptCheckProjectileSpawnSample> {
    frame
        .report
        .commands
        .iter()
        .filter_map(|command| match command {
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position,
                velocity,
                source,
            }) => Some(("enemy_laser", *position, *velocity, *source)),
            GameCommand::Spawn(SpawnRequest::Bomb { position, source }) => Some((
                "bomb",
                *position,
                crate::actor_game::Velocity::default(),
                *source,
            )),
            _ => None,
        })
        .take(ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_LIMIT)
        .map(
            |(kind, position, velocity, source)| ActorScriptCheckProjectileSpawnSample {
                kind: kind.to_string(),
                x: position.x,
                y: position.y,
                velocity_dx: velocity.dx,
                velocity_dy: velocity.dy,
                x_subpixel: source.map(|source| source.x_fraction),
                y_subpixel: source.map(|source| source.y_fraction),
                x_velocity_word: source.map(|source| source.x_velocity),
                y_velocity_word: source.map(|source| source.y_velocity),
                lifetime_ticks: source.map(|source| source.lifetime_ticks),
            },
        )
        .collect()
}

fn actor_script_check_player_laser_samples(
    frame: &ActorFrame,
) -> Vec<ActorScriptCheckPlayerLaserSample> {
    frame
        .report
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.kind == ActorKind::Laser && snapshot.alive)
        .take(ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_LIMIT)
        .map(|snapshot| ActorScriptCheckPlayerLaserSample {
            x: snapshot.position.x,
            y: snapshot.position.y,
            velocity_dx: snapshot.velocity.dx,
            velocity_dy: snapshot.velocity.dy,
            direction: snapshot
                .direction
                .map(direction_label)
                .unwrap_or("none")
                .to_string(),
        })
        .collect()
}

fn actor_script_check_explosion_command_samples(
    frame: &ActorFrame,
) -> Vec<ActorScriptCheckExplosionSample> {
    frame
        .report
        .commands
        .iter()
        .filter_map(|command| match command {
            GameCommand::Spawn(SpawnRequest::Explosion {
                position,
                kind,
                source_center,
            }) => Some(ActorScriptCheckExplosionSample {
                kind: explosion_kind_label(*kind).to_string(),
                x: position.x,
                y: position.y,
                source_center_x: source_center.map(|center| center.x),
                source_center_y: source_center.map(|center| center.y),
            }),
            _ => None,
        })
        .take(ACTOR_SCRIPT_CHECK_PROJECTILE_SAMPLE_LIMIT)
        .collect()
}

fn actor_script_check_sound_commands(frame: &ActorFrame) -> Vec<u8> {
    frame
        .report
        .sounds
        .iter()
        .filter_map(|sound| sound.source_sound_command())
        .collect()
}

fn actor_script_check_hit_sound_commands(frame: &ActorFrame) -> Vec<u8> {
    frame
        .report
        .sounds
        .iter()
        .filter_map(|sound| match sound {
            SoundCue::LanderHit
            | SoundCue::MutantHit
            | SoundCue::BomberHit
            | SoundCue::BombHit
            | SoundCue::PodHit
            | SoundCue::SwarmerHit
            | SoundCue::BaiterHit => sound.source_sound_command(),
            _ => None,
        })
        .collect()
}

fn actor_script_check_laser_sound_commands(frame: &ActorFrame) -> Vec<u8> {
    frame
        .report
        .sounds
        .iter()
        .filter_map(|sound| match sound {
            SoundCue::Laser => sound.source_sound_command(),
            _ => None,
        })
        .collect()
}

fn actor_script_check_projectile_sound_commands(frame: &ActorFrame) -> Vec<u8> {
    frame
        .report
        .sounds
        .iter()
        .filter_map(|sound| match sound {
            SoundCue::LanderShot
            | SoundCue::MutantShot
            | SoundCue::SwarmerShot
            | SoundCue::BaiterShot => sound.source_sound_command(),
            _ => None,
        })
        .collect()
}

fn direction_label(direction: crate::actor_game::Direction) -> &'static str {
    match direction {
        crate::actor_game::Direction::Left => "left",
        crate::actor_game::Direction::Right => "right",
    }
}

fn explosion_kind_label(kind: ExplosionKind) -> &'static str {
    match kind {
        ExplosionKind::Lander => "lander",
        ExplosionKind::Mutant => "mutant",
        ExplosionKind::Bomber => "bomber",
        ExplosionKind::Pod => "pod",
        ExplosionKind::Swarmer => "swarmer",
        ExplosionKind::Baiter => "baiter",
        ExplosionKind::Bomb => "bomb",
        ExplosionKind::Player => "player",
        ExplosionKind::Human => "human",
        ExplosionKind::Terrain => "terrain",
    }
}

fn arcade_rng_summary(seed: Option<u8>, hseed: Option<u8>, lseed: Option<u8>) -> String {
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
        let spawned_samples = actor_script_check_spawned_actor_samples(&frame);
        if frame.report.phase == Phase::Playing
            && frame.report.wave == wave
            && actor_script_check_reserve_total(previous_reserve) > 0
            && !spawned_counts.is_empty()
        {
            batches.push(ActorScriptCheckReserveActivationSummary {
                assist_steps: step as u32,
                spawned_counts,
                spawned_samples,
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

fn actor_script_check_spawned_actor_samples(
    frame: &ActorFrame,
) -> Vec<ActorScriptCheckSpawnedActorSample> {
    frame
        .report
        .commands
        .iter()
        .filter_map(|command| match command {
            GameCommand::Spawn(SpawnRequest::Lander { position }) => Some(("lander", *position)),
            GameCommand::Spawn(SpawnRequest::Bomber { position }) => Some(("bomber", *position)),
            GameCommand::Spawn(SpawnRequest::Pod { position }) => Some(("pod", *position)),
            GameCommand::Spawn(SpawnRequest::Mutant { position, .. }) => {
                Some(("mutant", *position))
            }
            GameCommand::Spawn(SpawnRequest::Swarmer { position, .. }) => {
                Some(("swarmer", *position))
            }
            GameCommand::Spawn(SpawnRequest::Baiter { position, .. }) => {
                Some(("baiter", *position))
            }
            _ => None,
        })
        .take(ACTOR_SCRIPT_CHECK_ACTOR_SAMPLE_LIMIT)
        .map(|(kind, position)| ActorScriptCheckSpawnedActorSample {
            kind: kind.to_string(),
            x: position.x,
            y: position.y,
        })
        .collect()
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
    let _runtime = actor_live_runtime_from_script_path(actor_script_path)?;
    Ok(())
}

#[cfg(all(not(test), not(coverage)))]
pub(crate) fn run_smoke(
    _input_profile: LiveInputProfile,
    _cmos_path: Option<&Path>,
) -> anyhow::Result<LiveSmokeReport> {
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
    crate::actor_smoke::default_smoke_report().map(LiveSmokeReport::from)
}
