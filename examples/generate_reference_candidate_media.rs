use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use defender::{
    Game, GameInput, SoundEvent, SpriteId, advance_one_frame,
    audio::render_sound_event_timeline_to_samples,
    game::{
        EnemyAppearanceSnapshot, EnemyProjectileSnapshot, ExpandedObjectDetailSnapshot,
        ObjectEvidenceDetailSnapshot, ReferenceCaptureSteer, SourceRandSnapshot,
    },
    readme_media::{FRAME_RATE_MILLIHZ, ReadmeMediaFrame, ReadmeMediaFrameSource},
};
use gif::{Encoder, Frame, Repeat};

const DEFAULT_OUT_DIR: &str = "target/reference-media/clean";
const DEFAULT_BASENAME: &str = "defender-clean-candidate";
const DEFAULT_SCENARIO: &str = "firing";
const DEFAULT_WIDTH: u32 = 768;
const DEFAULT_HEIGHT: u32 = 576;
const DEFAULT_SAMPLE_STEP_FRAMES: u64 = 6;
const DEFAULT_AUDIO_SAMPLE_RATE_HZ: u32 = 22_050;
const TERRAIN_BLOW_STATE_STEER_WAKE_FRAMES: u64 = 50;
const TRACE_SCENARIOS_TSV: &str = include_str!("../assets/red-label/trace-scenarios.tsv");
const PCM_CHANNELS: u16 = 1;
const PCM_BITS_PER_SAMPLE: u16 = 16;
const PCM_BYTES_PER_SAMPLE: usize = 2;

fn main() -> Result<()> {
    let args = MediaArgs::parse(std::env::args_os().skip(1))?;
    let input_program = select_input_program(&args)?;
    let input_frames = expand_input_program(&input_program)?;
    let paths = OutputPaths::new(&args);
    if args.debug_only {
        write_debug_tsv(
            &paths.debug_tsv,
            &input_frames,
            args.state_steer,
            args.state_steer_frame,
        )?;
        println!("wrote {}", paths.debug_tsv.display());
        println!("captured {} clean debug frame(s)", input_frames.len());
        return Ok(());
    }

    let sequence = capture_sequence(
        &input_frames,
        CaptureOptions {
            sample_step: args.sample_step,
            width: args.width,
            height: args.height,
            state_steer: args.state_steer,
            state_steer_frame: args.state_steer_frame,
            capture_start_frame: args.capture_start_frame,
            capture_end_frame: args.capture_end_frame,
        },
    )?;
    ensure_parent_dir(&paths.gif)?;
    write_gif(&paths.gif, &sequence.visual_frames)?;
    write_audio_wav(&paths.wav, &sequence, args.audio_sample_rate)?;
    write_events_tsv(&paths.events_tsv, &sequence)?;
    write_debug_tsv(
        &paths.debug_tsv,
        &input_frames,
        args.state_steer,
        args.state_steer_frame,
    )?;

    println!("wrote {}", paths.gif.display());
    println!("wrote {}", paths.wav.display());
    println!("wrote {}", paths.events_tsv.display());
    println!("wrote {}", paths.debug_tsv.display());
    println!(
        "captured {} clean frame(s), {} sampled visual frame(s)",
        sequence.frame_count,
        sequence.visual_frames.len()
    );
    Ok(())
}

fn select_input_program(args: &MediaArgs) -> Result<String> {
    match (&args.scenario, &args.input_program) {
        (Some(_), Some(_)) => bail!("--scenario and --input-program are mutually exclusive"),
        (None, Some(program)) => Ok(program.clone()),
        (Some(name), None) => scenario_input_program(name),
        (None, None) => scenario_input_program(DEFAULT_SCENARIO),
    }
}

fn scenario_input_program(name: &str) -> Result<String> {
    let scenarios = read_scenarios()?;
    let Some(scenario) = scenarios.iter().find(|scenario| scenario.name == name) else {
        bail!("unknown trace scenario {name:?}");
    };
    Ok(scenario.input_program.clone())
}

fn read_scenarios() -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();
    for (line_index, line) in TRACE_SCENARIOS_TSV.lines().enumerate() {
        if line_index == 0 || line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() < 3 {
            bail!(
                "trace-scenarios.tsv:{} has fewer than 3 columns",
                line_index + 1
            );
        }
        scenarios.push(Scenario {
            name: columns[0].to_string(),
            frames: columns[1].parse().with_context(|| {
                format!("parsing trace-scenarios.tsv:{} frame count", line_index + 1)
            })?,
            input_program: columns[2].to_string(),
        });
    }
    Ok(scenarios)
}

fn expand_input_program(program: &str) -> Result<Vec<GameInput>> {
    let mut frames = Vec::new();
    for segment in program.split(';') {
        let segment = segment.trim();
        if segment.is_empty() {
            bail!("input program contains an empty segment");
        }
        let (frame_text, repeat) =
            if let Some((frame_text, repeat_text)) = segment.rsplit_once('*') {
                let repeat = repeat_text.trim().parse::<usize>().with_context(|| {
                    format!("parsing repeat count in input segment {segment:?}")
                })?;
                if repeat == 0 {
                    bail!("input segment {segment:?} repeats zero frames");
                }
                (frame_text.trim(), repeat)
            } else {
                (segment, 1)
            };
        let input = parse_input_frame(frame_text)?;
        frames.extend(std::iter::repeat_n(input, repeat));
    }
    if frames.is_empty() {
        bail!("input program produced no frames");
    }
    Ok(frames)
}

fn parse_input_frame(frame_text: &str) -> Result<GameInput> {
    if frame_text == "-" || frame_text == "none" {
        return Ok(GameInput::NONE);
    }

    let mut input = GameInput::NONE;
    for action in frame_text.split(',') {
        match action.trim() {
            "fire" => input.fire = true,
            "thrust" => input.thrust = true,
            "smart_bomb" | "smartbomb" => input.smart_bomb = true,
            "hyperspace" => input.hyperspace = true,
            "start_one" | "start1" => input.start_one = true,
            "start_two" | "start2" => input.start_two = true,
            "reverse" => input.reverse = true,
            "altitude_down" | "down" => input.altitude_down = true,
            "altitude_up" | "up" => input.altitude_up = true,
            "auto_up_manual_down" => input.service_auto_up = true,
            "service_advance" | "advance" => input.service_advance = true,
            "coin" | "coin_one" | "coin1" => input.coin = true,
            "coin_two" | "coin2" => input.coin_two = true,
            "coin_three" | "coin3" => input.coin_three = true,
            "high_score_reset" => input.high_score_reset = true,
            "tilt" => input.tilt = true,
            "" => bail!("input frame {frame_text:?} contains an empty action"),
            unknown => bail!("unknown input action {unknown:?}"),
        }
    }
    Ok(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CaptureOptions {
    sample_step: u64,
    width: u32,
    height: u32,
    state_steer: Option<ReferenceCaptureSteer>,
    state_steer_frame: u64,
    capture_start_frame: u64,
    capture_end_frame: Option<u64>,
}

fn capture_sequence(
    input_frames: &[GameInput],
    options: CaptureOptions,
) -> Result<CandidateMediaSequence> {
    if input_frames.is_empty() {
        bail!("candidate capture requires at least one input frame");
    }
    if options.sample_step == 0 {
        bail!("--sample-step must be positive");
    }
    if options.capture_start_frame == 0 {
        bail!("--capture-start-frame must be positive");
    }

    let input_frame_count = input_frames.len() as u64;
    let capture_end_frame = options.capture_end_frame.unwrap_or(input_frame_count);
    if options.capture_start_frame > capture_end_frame {
        bail!("--capture-start-frame must not exceed --capture-end-frame");
    }
    if capture_end_frame > input_frame_count {
        bail!("--capture-end-frame exceeds the expanded input frame count");
    }

    let mut source = ReadmeMediaFrameSource::new_with_first_input(
        options.width,
        options.height,
        input_frames[0],
    );
    let mut delay = DelayAccumulator::new();
    let mut visual_frames = Vec::new();
    let mut audio_events = Vec::new();
    let mut captured_audio_events = Vec::new();
    let state_steer_activation_frame =
        clean_state_steer_activation_frame(options.state_steer, options.state_steer_frame);

    for frame_index in 0..input_frames.len() {
        let frame_index_u64 = frame_index as u64;
        let frame_number = frame_index_u64 + 1;
        if let Some(steer) = options.state_steer
            && frame_number == state_steer_activation_frame
        {
            source.seed_reference_capture_window(steer);
        }

        let sounds = source.sound_events().to_vec();
        if !sounds.is_empty() {
            audio_events.push((frame_number.saturating_sub(1), sounds.clone()));
        }

        if (options.capture_start_frame..=capture_end_frame).contains(&frame_number) {
            let capture_frame_index = frame_number - options.capture_start_frame;
            if capture_frame_index.is_multiple_of(options.sample_step) {
                let remaining = capture_end_frame - frame_number + 1;
                let delay_frames = options.sample_step.min(remaining);
                visual_frames.push((
                    source
                        .render_frame()
                        .context("rendering clean reference candidate frame")?
                        .into(),
                    delay.centiseconds_for_frames(delay_frames),
                ));
            }

            if !sounds.is_empty() {
                captured_audio_events.push((capture_frame_index, sounds.clone()));
            }
        }

        if let Some(next_input) = input_frames.get(frame_index + 1) {
            source.step_with_input(*next_input);
        }
    }

    Ok(CandidateMediaSequence {
        visual_frames,
        audio_events,
        captured_audio_events,
        capture_start_frame: options.capture_start_frame,
        capture_end_frame,
        frame_count: capture_end_frame - options.capture_start_frame + 1,
    })
}

fn clean_state_steer_activation_frame(
    state_steer: Option<ReferenceCaptureSteer>,
    state_steer_frame: u64,
) -> u64 {
    match state_steer {
        Some(ReferenceCaptureSteer::TerrainBlow) => {
            state_steer_frame.saturating_add(TERRAIN_BLOW_STATE_STEER_WAKE_FRAMES)
        }
        _ => state_steer_frame,
    }
}

fn write_gif(path: &Path, frames: &[(RgbaImage, u16)]) -> Result<()> {
    let Some((first, _)) = frames.first() else {
        bail!("candidate sequence did not produce any visual frame");
    };
    let file = File::create(path).with_context(|| format!("creating GIF {}", path.display()))?;
    let mut encoder = Encoder::new(file, first.width as u16, first.height as u16, &[])
        .with_context(|| format!("creating GIF encoder for {}", path.display()))?;
    encoder
        .set_repeat(Repeat::Infinite)
        .context("setting GIF repeat mode")?;

    for (image, delay) in frames {
        let mut pixels = image.pixels.clone();
        let mut frame =
            Frame::from_rgba_speed(image.width as u16, image.height as u16, &mut pixels, 30);
        frame.delay = *delay;
        encoder.write_frame(&frame).context("writing GIF frame")?;
    }

    Ok(())
}

fn write_audio_wav(
    path: &Path,
    sequence: &CandidateMediaSequence,
    sample_rate_hz: u32,
) -> Result<()> {
    ensure_parent_dir(path)?;
    let samples = render_windowed_sound_event_timeline_to_samples(
        &sequence.audio_events,
        sequence.capture_end_frame,
        sequence.capture_start_frame,
        sequence.capture_end_frame,
        sample_rate_hz,
    );
    let mut file =
        File::create(path).with_context(|| format!("creating WAV {}", path.display()))?;
    write_wav_header(&mut file, samples.len(), sample_rate_hz)?;
    for sample in samples {
        let scaled = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        file.write_all(&scaled.to_le_bytes())
            .with_context(|| format!("writing WAV samples to {}", path.display()))?;
    }
    Ok(())
}

fn render_windowed_sound_event_timeline_to_samples(
    audio_events: &[(u64, Vec<SoundEvent>)],
    total_frames: u64,
    capture_start_frame: u64,
    capture_end_frame: u64,
    sample_rate_hz: u32,
) -> Vec<f32> {
    let samples = render_sound_event_timeline_to_samples(
        audio_events,
        total_frames,
        FRAME_RATE_MILLIHZ,
        sample_rate_hz,
    );
    let start_sample = sample_count_for_frame(
        capture_start_frame.saturating_sub(1),
        FRAME_RATE_MILLIHZ,
        sample_rate_hz,
    );
    let end_sample = sample_count_for_frame(capture_end_frame, FRAME_RATE_MILLIHZ, sample_rate_hz)
        .min(samples.len());
    samples[start_sample.min(end_sample)..end_sample].to_vec()
}

fn sample_count_for_frame(frame: u64, frame_rate_millihz: u32, sample_rate_hz: u32) -> usize {
    let numerator = u128::from(frame) * u128::from(sample_rate_hz) * 1_000;
    let denominator = u128::from(frame_rate_millihz.max(1));
    usize::try_from(numerator.div_ceil(denominator)).unwrap_or(usize::MAX)
}

fn write_events_tsv(path: &Path, sequence: &CandidateMediaSequence) -> Result<()> {
    ensure_parent_dir(path)?;
    let mut rows = Vec::with_capacity(sequence.captured_audio_events.len() + 1);
    rows.push("frame\tsound_events".to_string());
    for (frame, events) in &sequence.captured_audio_events {
        let sounds = events
            .iter()
            .map(|event| format!("{event:?}"))
            .collect::<Vec<_>>()
            .join(",");
        rows.push(format!("{frame}\t{sounds}"));
    }
    std::fs::write(path, rows.join("\n") + "\n")
        .with_context(|| format!("writing event TSV {}", path.display()))
}

fn write_debug_tsv(
    path: &Path,
    input_frames: &[GameInput],
    state_steer: Option<ReferenceCaptureSteer>,
    state_steer_frame: u64,
) -> Result<()> {
    ensure_parent_dir(path)?;
    let mut game = Game::new();
    let mut rows = Vec::with_capacity(input_frames.len() + 1);
    let state_steer_activation_frame =
        clean_state_steer_activation_frame(state_steer, state_steer_frame);
    rows.push(
        [
            "input_frame",
            "state_frame",
            "phase",
            "p1_score",
            "lives",
            "direction",
            "player_world_x",
            "player_world_y",
            "source_rng",
            "enemies",
            "humans",
            "terrain_blow",
            "projectiles",
            "enemy_projectiles",
            "explosions",
            "appearances",
            "object_evidence",
            "expanded_objects",
            "sprites",
            "sound_events",
            "gameplay_events",
        ]
        .join("\t"),
    );

    for (input_frame, input) in input_frames.iter().copied().enumerate() {
        let mut frame = advance_one_frame(&mut game, input);
        if let Some(steer) = state_steer
            && frame.state.frame == state_steer_activation_frame
        {
            frame = game.seed_reference_capture_window(steer);
        }
        rows.push(
            [
                input_frame.to_string(),
                frame.state.frame.to_string(),
                format!("{:?}", frame.state.phase),
                frame.state.scores.player_one.to_string(),
                frame.state.player.lives.to_string(),
                format!("{:?}", frame.state.player.direction),
                frame.state.player.position.0.subpixels().to_string(),
                frame.state.player.position.1.subpixels().to_string(),
                format_source_rng(frame.state.world.source_rng),
                format_clean_enemies(&frame.state.world.enemies),
                format_clean_humans(&frame.state.world.humans),
                format_clean_terrain_blow(frame.state.world.terrain_blow),
                format_clean_projectiles(&frame.state.world.projectiles),
                format_clean_enemy_projectiles(&frame.state.world.enemy_projectiles),
                format_clean_explosions(&frame.state.world.explosions),
                format_clean_appearances(&frame.state.world.enemy_appearances),
                format_object_evidence(&frame.state.world.object_evidence.details),
                format_expanded_objects(&frame.state.world.expanded_objects.details),
                format_reference_sprites(&frame.scene.sprites),
                format_debug_values(frame.events.sounds()),
                format_debug_values(frame.events.gameplay()),
            ]
            .join("\t"),
        );
    }

    std::fs::write(path, rows.join("\n") + "\n")
        .with_context(|| format!("writing debug TSV {}", path.display()))
}

fn format_clean_enemies(enemies: &[defender::EnemySnapshot]) -> String {
    if enemies.is_empty() {
        return "-".to_string();
    }
    enemies
        .iter()
        .map(|enemy| {
            let base = format!("{:?}@{},{}", enemy.kind, enemy.position.x, enemy.position.y);
            if let Some(source_lander) = enemy.source_lander {
                format!(
                    "{}:xf0x{:02X}:yf0x{:02X}:xv0x{:04X}:yv0x{:04X}:shot{}:sleep{}:pic{}:target{}",
                    base,
                    source_lander.x_fraction,
                    source_lander.y_fraction,
                    source_lander.x_velocity,
                    source_lander.y_velocity,
                    source_lander.shot_timer,
                    source_lander.sleep_ticks,
                    source_lander.picture_frame,
                    format_optional_usize(source_lander.target_human_index),
                )
            } else if let Some(source_mutant) = enemy.source_mutant {
                format!(
                    "{}:xf0x{:02X}:yf0x{:02X}:xv0x{:04X}:yv0x{:04X}:shot{}:sleep{}:rx0x{:04X}",
                    base,
                    source_mutant.x_fraction,
                    source_mutant.y_fraction,
                    source_mutant.x_velocity,
                    source_mutant.y_velocity,
                    source_mutant.shot_timer,
                    source_mutant.sleep_ticks,
                    source_mutant.render_x_correction,
                )
            } else {
                base
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_clean_humans(humans: &[defender::HumanSnapshot]) -> String {
    if humans.is_empty() {
        return "-".to_string();
    }
    humans
        .iter()
        .map(|human| {
            format!(
                "{},{}:carried{}:player{}",
                human.position.x, human.position.y, human.carried, human.carried_by_player
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_clean_terrain_blow(terrain_blow: Option<defender::game::TerrainBlowSnapshot>) -> String {
    let Some(terrain_blow) = terrain_blow else {
        return "-".to_string();
    };
    format!(
        "{:?}:elapsed{}:iter{}:sleep{}:color0x{:02X}:ov{}:terrain{}:scanner{}",
        terrain_blow.stage,
        terrain_blow.source_elapsed_frames,
        terrain_blow.source_iteration,
        format_optional_u8(terrain_blow.source_sleep_remaining),
        terrain_blow.source_pseudo_color,
        terrain_blow.source_overload_counter,
        terrain_blow.terrain_words_remaining,
        terrain_blow.scanner_terrain_words_remaining,
    )
}

fn format_clean_projectiles(projectiles: &[defender::ProjectileSnapshot]) -> String {
    if projectiles.is_empty() {
        return "-".to_string();
    }
    projectiles
        .iter()
        .map(|projectile| {
            format!(
                "{},{}->tail{},{}:v{},{}",
                projectile.position.x,
                projectile.position.y,
                projectile.source_tail_position.x,
                projectile.source_tail_position.y,
                projectile.velocity.dx,
                projectile.velocity.dy
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_clean_enemy_projectiles(projectiles: &[EnemyProjectileSnapshot]) -> String {
    if projectiles.is_empty() {
        return "-".to_string();
    }
    projectiles
        .iter()
        .map(|projectile| {
            format!(
                "{:?}@{},{}:v{},{}:life{}",
                projectile.source_kind,
                projectile.position.x,
                projectile.position.y,
                projectile.velocity.dx,
                projectile.velocity.dy,
                projectile.source_lifetime_ticks
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_clean_explosions(explosions: &[defender::game::ExplosionSnapshot]) -> String {
    if explosions.is_empty() {
        return "-".to_string();
    }
    explosions
        .iter()
        .map(|explosion| {
            format!(
                "{:?}@{},{}:size{}:frames{}",
                explosion.kind,
                explosion.position.x,
                explosion.position.y,
                explosion.source_size,
                explosion.frames_remaining
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_clean_appearances(appearances: &[EnemyAppearanceSnapshot]) -> String {
    if appearances.is_empty() {
        return "-".to_string();
    }
    appearances
        .iter()
        .map(|appearance| {
            format!(
                "{}@{},{}:size0x{:04X}:sprite{}",
                appearance.picture_label,
                appearance.position.x,
                appearance.position.y,
                appearance.source_size,
                appearance.mapped_sprite.0
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_source_rng(rng: SourceRandSnapshot) -> String {
    format!(
        "seed=0x{:02X}:hseed=0x{:02X}:lseed=0x{:02X}",
        rng.seed, rng.hseed, rng.lseed
    )
}

fn format_object_evidence(details: &[ObjectEvidenceDetailSnapshot]) -> String {
    let values = details
        .iter()
        .filter(|detail| detail.address.is_some() || detail.screen_position.is_some())
        .map(|detail| {
            let position = detail
                .screen_position
                .map(|position| format!("{},{}", position.x, position.y))
                .unwrap_or_else(|| "-".to_string());
            let world = detail
                .world_position
                .map(|(x, y)| format!("0x{x:04X},0x{y:04X}"))
                .unwrap_or_else(|| "-".to_string());
            let velocity = detail
                .velocity
                .map(|(x, y)| format!("0x{x:04X},0x{y:04X}"))
                .unwrap_or_else(|| "-".to_string());
            let picture = detail.picture_label.unwrap_or("-");
            let mapped = detail
                .mapped_sprite
                .map(|sprite| sprite.0.to_string())
                .unwrap_or_else(|| "-".to_string());
            format!(
                "{:?}:addr{}:slot{}:pic{}:sprite{}:screen{}:world{}:vel{}",
                detail.object_category,
                format_optional_hex(detail.address),
                format_optional_hex(detail.slot),
                picture,
                mapped,
                position,
                world,
                velocity
            )
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(",")
    }
}

fn format_expanded_objects(details: &[ExpandedObjectDetailSnapshot]) -> String {
    let values = details
        .iter()
        .filter(|detail| {
            detail.mapped_sprite.is_some() || detail.center.is_some() || detail.top_left.is_some()
        })
        .map(|detail| {
            let center = detail
                .center
                .map(|position| format!("{},{}", position.x, position.y))
                .unwrap_or_else(|| "-".to_string());
            let top_left = detail
                .top_left
                .map(|position| format!("{},{}", position.x, position.y))
                .unwrap_or_else(|| "-".to_string());
            let mapped = detail
                .mapped_sprite
                .map(|sprite| sprite.0.to_string())
                .unwrap_or_else(|| "-".to_string());
            format!(
                "{:?}:slot{}:size0x{:04X}:pic{}:sprite{}:center{}:top{}:obj{}:frame{}:life{}",
                detail.kind,
                format_optional_hex(detail.slot_address),
                detail.size,
                detail.picture_label.unwrap_or("-"),
                mapped,
                center,
                top_left,
                format_optional_hex(detail.object_address),
                format_optional_u8(detail.explosion_frame),
                format_optional_u8(detail.explosion_lifetime_frames)
            )
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(",")
    }
}

fn format_reference_sprites(sprites: &[defender::SceneSprite]) -> String {
    let values = sprites
        .iter()
        .filter(|sprite| {
            matches!(
                sprite.sprite,
                SpriteId::PLAYER_SHIP
                    | SpriteId::PLAYER_SHIP_LEFT
                    | SpriteId::PLAYER_PROJECTILE
                    | SpriteId::ENEMY_BOMB
                    | SpriteId::ENEMY_LANDER
                    | SpriteId::ENEMY_MUTANT
                    | SpriteId::ENEMY_BAITER
                    | SpriteId::ENEMY_BOMBER
                    | SpriteId::ENEMY_POD
                    | SpriteId::ENEMY_SWARMER
                    | SpriteId::HUMAN
                    | SpriteId::PLAYER_EXPLOSION_PIXEL
                    | SpriteId::BOMB_EXPLOSION
                    | SpriteId::SWARMER_EXPLOSION
            )
        })
        .map(|sprite| {
            format!(
                "{}@{:.0},{:.0}:{:.0}x{:.0}:{:?}",
                sprite.sprite.0,
                sprite.position[0],
                sprite.position[1],
                sprite.size[0],
                sprite.size[1],
                sprite.layer
            )
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(",")
    }
}

fn format_optional_hex(value: Option<u16>) -> String {
    value
        .map(|value| format!("0x{value:04X}"))
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_u8(value: Option<u8>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn format_debug_values<T: std::fmt::Debug>(values: &[T]) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn write_wav_header(file: &mut File, sample_count: usize, sample_rate_hz: u32) -> Result<()> {
    let data_size = sample_count
        .checked_mul(PCM_BYTES_PER_SAMPLE)
        .context("candidate audio sample data is too large")?;
    let data_size = u32::try_from(data_size).context("candidate audio WAV data is too large")?;
    let byte_rate = sample_rate_hz * u32::from(PCM_CHANNELS) * u32::from(PCM_BITS_PER_SAMPLE) / 8;
    let block_align = PCM_CHANNELS * PCM_BITS_PER_SAMPLE / 8;

    file.write_all(b"RIFF").context("writing WAV RIFF header")?;
    file.write_all(&(36 + data_size).to_le_bytes())
        .context("writing WAV file size")?;
    file.write_all(b"WAVEfmt ")
        .context("writing WAV format header")?;
    file.write_all(&16_u32.to_le_bytes())
        .context("writing WAV fmt chunk size")?;
    file.write_all(&1_u16.to_le_bytes())
        .context("writing WAV PCM format")?;
    file.write_all(&PCM_CHANNELS.to_le_bytes())
        .context("writing WAV channel count")?;
    file.write_all(&sample_rate_hz.to_le_bytes())
        .context("writing WAV sample rate")?;
    file.write_all(&byte_rate.to_le_bytes())
        .context("writing WAV byte rate")?;
    file.write_all(&block_align.to_le_bytes())
        .context("writing WAV block alignment")?;
    file.write_all(&PCM_BITS_PER_SAMPLE.to_le_bytes())
        .context("writing WAV bit depth")?;
    file.write_all(b"data").context("writing WAV data chunk")?;
    file.write_all(&data_size.to_le_bytes())
        .context("writing WAV data size")?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating directory {}", parent.display()))?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MediaArgs {
    out_dir: PathBuf,
    basename: String,
    scenario: Option<String>,
    input_program: Option<String>,
    state_steer: Option<ReferenceCaptureSteer>,
    state_steer_frame: u64,
    capture_start_frame: u64,
    capture_end_frame: Option<u64>,
    width: u32,
    height: u32,
    sample_step: u64,
    audio_sample_rate: u32,
    debug_only: bool,
}

impl MediaArgs {
    fn parse(mut args: impl Iterator<Item = OsString>) -> Result<Self> {
        let mut parsed = Self {
            out_dir: PathBuf::from(DEFAULT_OUT_DIR),
            basename: DEFAULT_BASENAME.to_string(),
            scenario: None,
            input_program: None,
            state_steer: None,
            state_steer_frame: 1400,
            capture_start_frame: 1,
            capture_end_frame: None,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            sample_step: DEFAULT_SAMPLE_STEP_FRAMES,
            audio_sample_rate: DEFAULT_AUDIO_SAMPLE_RATE_HZ,
            debug_only: false,
        };

        while let Some(arg) = args.next() {
            match arg.as_os_str() {
                value if value == OsStr::new("--out-dir") => {
                    parsed.out_dir = next_path_arg(&mut args, "--out-dir")?;
                }
                value if value == OsStr::new("--basename") => {
                    parsed.basename = next_string_arg(&mut args, "--basename")?;
                }
                value if value == OsStr::new("--scenario") => {
                    parsed.scenario = Some(next_string_arg(&mut args, "--scenario")?);
                }
                value if value == OsStr::new("--input-program") => {
                    parsed.input_program = Some(next_string_arg(&mut args, "--input-program")?);
                }
                value if value == OsStr::new("--state-steer") => {
                    let value = next_string_arg(&mut args, "--state-steer")?;
                    parsed.state_steer = Some(parse_state_steer(&value)?);
                }
                value if value == OsStr::new("--state-steer-frame") => {
                    parsed.state_steer_frame = next_string_arg(&mut args, "--state-steer-frame")?
                        .parse()
                        .context("parsing --state-steer-frame")?;
                }
                value if value == OsStr::new("--capture-start-frame") => {
                    parsed.capture_start_frame =
                        next_string_arg(&mut args, "--capture-start-frame")?
                            .parse()
                            .context("parsing --capture-start-frame")?;
                }
                value if value == OsStr::new("--capture-end-frame") => {
                    parsed.capture_end_frame = Some(
                        next_string_arg(&mut args, "--capture-end-frame")?
                            .parse()
                            .context("parsing --capture-end-frame")?,
                    );
                }
                value if value == OsStr::new("--width") => {
                    parsed.width = next_string_arg(&mut args, "--width")?
                        .parse()
                        .context("parsing --width")?;
                }
                value if value == OsStr::new("--height") => {
                    parsed.height = next_string_arg(&mut args, "--height")?
                        .parse()
                        .context("parsing --height")?;
                }
                value if value == OsStr::new("--sample-step") => {
                    parsed.sample_step = next_string_arg(&mut args, "--sample-step")?
                        .parse()
                        .context("parsing --sample-step")?;
                }
                value if value == OsStr::new("--audio-sample-rate") => {
                    parsed.audio_sample_rate = next_string_arg(&mut args, "--audio-sample-rate")?
                        .parse()
                        .context("parsing --audio-sample-rate")?;
                }
                value if value == OsStr::new("--debug-only") => {
                    parsed.debug_only = true;
                }
                unknown => bail!("unknown argument {:?}", unknown),
            }
        }

        if parsed.basename.is_empty() {
            bail!("--basename must not be empty");
        }
        if parsed.state_steer.is_some() && parsed.state_steer_frame == 0 {
            bail!("--state-steer-frame must be positive");
        }
        if parsed.capture_start_frame == 0 {
            bail!("--capture-start-frame must be positive");
        }
        if parsed
            .capture_end_frame
            .is_some_and(|end_frame| end_frame < parsed.capture_start_frame)
        {
            bail!("--capture-end-frame must be greater than or equal to --capture-start-frame");
        }
        Ok(parsed)
    }
}

fn parse_state_steer(value: &str) -> Result<ReferenceCaptureSteer> {
    match value {
        "afall_fall" | "falling_human_fall" => Ok(ReferenceCaptureSteer::FallingHumanFall),
        "afall_safe_landing" | "falling_human_safe_landing" => {
            Ok(ReferenceCaptureSteer::FallingHumanSafeLanding)
        }
        "afall_player_catch" | "falling_human_catch" => {
            Ok(ReferenceCaptureSteer::FallingHumanCatch)
        }
        "enemy_explosion_matrix" => Ok(ReferenceCaptureSteer::EnemyExplosionMatrix),
        "enemy_materialize_matrix" | "enemy_coalescence_matrix" => {
            Ok(ReferenceCaptureSteer::EnemyMaterializeMatrix)
        }
        "sound_command_matrix" | "sound_commands" => Ok(ReferenceCaptureSteer::SoundCommandMatrix),
        "sound_bomber_hit" | "sound_tie_hit" | "sound_command_fe" => {
            Ok(ReferenceCaptureSteer::SoundBomberHit)
        }
        "sound_pod_hit" | "sound_command_fa" => Ok(ReferenceCaptureSteer::SoundPodHit),
        "sound_swarmer_hit" | "sound_baiter_hit" | "sound_command_f8" => {
            Ok(ReferenceCaptureSteer::SoundSwarmerHit)
        }
        "sound_swarmer_shot" | "sound_command_f3" => Ok(ReferenceCaptureSteer::SoundSwarmerShot),
        "terrain_blow" => Ok(ReferenceCaptureSteer::TerrainBlow),
        unknown => bail!("unknown --state-steer value {unknown:?}"),
    }
}

fn next_path_arg(args: &mut impl Iterator<Item = OsString>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(next_string_arg(args, name)?))
}

fn next_string_arg(args: &mut impl Iterator<Item = OsString>, name: &str) -> Result<String> {
    let Some(value) = args.next() else {
        bail!("{name} requires a value");
    };
    value
        .into_string()
        .map_err(|_| anyhow::anyhow!("{name} must be valid UTF-8"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OutputPaths {
    gif: PathBuf,
    wav: PathBuf,
    events_tsv: PathBuf,
    debug_tsv: PathBuf,
}

impl OutputPaths {
    fn new(args: &MediaArgs) -> Self {
        Self {
            gif: args.out_dir.join(format!("{}.gif", args.basename)),
            wav: args.out_dir.join(format!("{}.wav", args.basename)),
            events_tsv: args.out_dir.join(format!("{}.events.tsv", args.basename)),
            debug_tsv: args.out_dir.join(format!("{}.debug.tsv", args.basename)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scenario {
    name: String,
    frames: u64,
    input_program: String,
}

#[derive(Debug, Clone, PartialEq)]
struct CandidateMediaSequence {
    visual_frames: Vec<(RgbaImage, u16)>,
    audio_events: Vec<(u64, Vec<SoundEvent>)>,
    captured_audio_events: Vec<(u64, Vec<SoundEvent>)>,
    capture_start_frame: u64,
    capture_end_frame: u64,
    frame_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RgbaImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl From<ReadmeMediaFrame> for RgbaImage {
    fn from(image: ReadmeMediaFrame) -> Self {
        Self {
            width: image.width,
            height: image.height,
            pixels: image.pixels,
        }
    }
}

struct DelayAccumulator {
    remainder: u64,
}

impl DelayAccumulator {
    const fn new() -> Self {
        Self { remainder: 0 }
    }

    fn centiseconds_for_frames(&mut self, frames: u64) -> u16 {
        let total = frames * 100_000 + self.remainder;
        let rate = u64::from(FRAME_RATE_MILLIHZ);
        let centiseconds = total / rate;
        self.remainder = total % rate;
        u16::try_from(centiseconds.max(1)).unwrap_or(u16::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_BASENAME, DEFAULT_HEIGHT, DEFAULT_OUT_DIR, DEFAULT_SAMPLE_STEP_FRAMES,
        DEFAULT_SCENARIO, DEFAULT_WIDTH, MediaArgs, OutputPaths,
        clean_state_steer_activation_frame, expand_input_program, format_clean_enemies,
        parse_state_steer, read_scenarios, render_windowed_sound_event_timeline_to_samples,
        scenario_input_program, select_input_program,
    };
    use defender::{
        EnemyKind, EnemySnapshot, ScreenPosition, ScreenVelocity, SoundEvent,
        game::{SourceMutantSnapshot, SourceRandSnapshot},
    };
    use std::{ffi::OsString, path::Path};

    #[test]
    fn default_args_select_scripted_firing_scenario() {
        let args = MediaArgs::parse(std::iter::empty()).expect("default args");

        assert_eq!(args.out_dir, Path::new(DEFAULT_OUT_DIR));
        assert_eq!(args.basename, DEFAULT_BASENAME);
        assert_eq!(args.scenario, None);
        assert_eq!(args.state_steer, None);
        assert_eq!(args.state_steer_frame, 1400);
        assert_eq!(args.capture_start_frame, 1);
        assert_eq!(args.capture_end_frame, None);
        assert_eq!(args.width, DEFAULT_WIDTH);
        assert_eq!(args.height, DEFAULT_HEIGHT);
        assert_eq!(args.sample_step, DEFAULT_SAMPLE_STEP_FRAMES);
        assert!(!args.debug_only);
        assert!(
            select_input_program(&args)
                .expect("default program")
                .contains("fire")
        );
        assert_eq!(DEFAULT_SCENARIO, "firing");
    }

    #[test]
    fn arguments_select_paths_and_explicit_program() {
        let args = vec![
            OsString::from("--out-dir"),
            OsString::from("target/sample"),
            OsString::from("--basename"),
            OsString::from("clip"),
            OsString::from("--input-program"),
            OsString::from("none*2;fire"),
            OsString::from("--state-steer"),
            OsString::from("enemy_explosion_matrix"),
            OsString::from("--state-steer-frame"),
            OsString::from("42"),
            OsString::from("--capture-start-frame"),
            OsString::from("40"),
            OsString::from("--capture-end-frame"),
            OsString::from("80"),
            OsString::from("--width"),
            OsString::from("320"),
            OsString::from("--height"),
            OsString::from("240"),
            OsString::from("--sample-step"),
            OsString::from("3"),
            OsString::from("--audio-sample-rate"),
            OsString::from("48000"),
            OsString::from("--debug-only"),
        ];

        let parsed = MediaArgs::parse(args.into_iter()).expect("parsed args");
        let paths = OutputPaths::new(&parsed);

        assert_eq!(paths.gif, Path::new("target/sample/clip.gif"));
        assert_eq!(paths.wav, Path::new("target/sample/clip.wav"));
        assert_eq!(paths.events_tsv, Path::new("target/sample/clip.events.tsv"));
        assert_eq!(paths.debug_tsv, Path::new("target/sample/clip.debug.tsv"));
        assert_eq!(parsed.width, 320);
        assert_eq!(parsed.height, 240);
        assert_eq!(parsed.sample_step, 3);
        assert_eq!(parsed.audio_sample_rate, 48_000);
        assert_eq!(
            parsed.state_steer,
            Some(defender::game::ReferenceCaptureSteer::EnemyExplosionMatrix)
        );
        assert_eq!(parsed.state_steer_frame, 42);
        assert_eq!(parsed.capture_start_frame, 40);
        assert_eq!(parsed.capture_end_frame, Some(80));
        assert!(parsed.debug_only);
        assert_eq!(
            select_input_program(&parsed).expect("program"),
            "none*2;fire"
        );
    }

    #[test]
    fn terrain_blow_state_steer_activates_at_mame_wake_frame() {
        assert_eq!(
            clean_state_steer_activation_frame(
                Some(defender::game::ReferenceCaptureSteer::TerrainBlow),
                1400,
            ),
            1450
        );
        assert_eq!(
            clean_state_steer_activation_frame(
                Some(defender::game::ReferenceCaptureSteer::FallingHumanCatch),
                1400,
            ),
            1400
        );
    }

    #[test]
    fn sound_command_matrix_state_steer_parses() {
        assert_eq!(
            parse_state_steer("sound_command_matrix").expect("steer"),
            defender::game::ReferenceCaptureSteer::SoundCommandMatrix
        );
        assert_eq!(
            parse_state_steer("enemy_materialize_matrix").expect("materialize matrix"),
            defender::game::ReferenceCaptureSteer::EnemyMaterializeMatrix
        );
        assert_eq!(
            parse_state_steer("enemy_coalescence_matrix").expect("coalescence alias"),
            defender::game::ReferenceCaptureSteer::EnemyMaterializeMatrix
        );
        assert_eq!(
            parse_state_steer("sound_commands").expect("alias"),
            defender::game::ReferenceCaptureSteer::SoundCommandMatrix
        );
        assert_eq!(
            parse_state_steer("sound_command_fe").expect("single command"),
            defender::game::ReferenceCaptureSteer::SoundBomberHit
        );
        assert_eq!(
            parse_state_steer("sound_command_fa").expect("single command"),
            defender::game::ReferenceCaptureSteer::SoundPodHit
        );
        assert_eq!(
            parse_state_steer("sound_command_f8").expect("single command"),
            defender::game::ReferenceCaptureSteer::SoundSwarmerHit
        );
        assert_eq!(
            parse_state_steer("sound_command_f3").expect("single command"),
            defender::game::ReferenceCaptureSteer::SoundSwarmerShot
        );
    }

    #[test]
    fn scenario_table_exposes_trace_programs() {
        let scenarios = read_scenarios().expect("trace scenarios");

        assert!(scenarios.iter().any(|scenario| {
            scenario.name == "thrust_reverse" && scenario.input_program.contains("reverse,thrust")
        }));
        assert!(
            scenario_input_program("firing")
                .expect("firing scenario")
                .contains("fire")
        );
    }

    #[test]
    fn input_program_repeats_and_maps_mame_action_names() {
        let frames = expand_input_program(
            "none*2;coin;start_one;reverse,thrust;smartbomb,hyperspace;\
             down,up,coin2,coin3,advance,high_score_reset,tilt,fire",
        )
        .expect("expanded program");

        assert_eq!(frames.len(), 7);
        assert_eq!(frames[0], defender::GameInput::NONE);
        assert_eq!(frames[1], defender::GameInput::NONE);
        assert!(frames[2].coin);
        assert!(frames[3].start_one);
        assert!(frames[4].reverse);
        assert!(frames[4].thrust);
        assert!(frames[5].smart_bomb);
        assert!(frames[5].hyperspace);
        assert!(frames[6].altitude_down);
        assert!(frames[6].altitude_up);
        assert!(frames[6].coin_two);
        assert!(frames[6].coin_three);
        assert!(frames[6].service_advance);
        assert!(frames[6].high_score_reset);
        assert!(frames[6].tilt);
        assert!(frames[6].fire);
    }

    #[test]
    fn windowed_audio_keeps_pre_window_sound_tails() {
        let samples = render_windowed_sound_event_timeline_to_samples(
            &[(0, vec![SoundEvent::UnmappedSoundCommand { command: 0xEA }])],
            3,
            2,
            3,
            22_050,
        );

        assert!(!samples.is_empty());
        assert!(samples.iter().any(|sample| sample.abs() > f32::EPSILON));
    }

    #[test]
    fn debug_enemy_format_includes_mutant_source_state() {
        let enemy = EnemySnapshot {
            kind: EnemyKind::Mutant,
            position: ScreenPosition::new(4, 45),
            velocity: ScreenVelocity::new(1, -1),
            source_lander: None,
            source_mutant: Some(SourceMutantSnapshot {
                x_fraction: 0x84,
                y_fraction: 0x16,
                x_velocity: 0x0030,
                y_velocity: 0xFF6F,
                shot_timer: 5,
                sleep_ticks: 2,
                hop_rng: SourceRandSnapshot::default(),
                render_x_correction: 0x0120,
                target6_first_shot_deferred: false,
            }),
            source_bomber: None,
            source_swarmer: None,
            source_baiter: None,
            source_pod: None,
        };

        assert_eq!(
            format_clean_enemies(&[enemy]),
            "Mutant@4,45:xf0x84:yf0x16:xv0x0030:yv0xFF6F:shot5:sleep2:rx0x0120"
        );
    }
}
