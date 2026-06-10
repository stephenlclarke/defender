const MUTANT_DIVE_PATH_Y_VELOCITY: u16 = 0x0090;
const MUTANT_DIVE_ENTRY_SHOT_MAX_X: i16 = 0x04;
const MUTANT_DIVE_ENTRY_SHOT_MAX_Y: i16 = 0x60;
const MUTANT_DIVE_SUPPRESSED_FIRST_SHOT_WORLD_Y_MIN: u16 = 0x4000;
const MUTANT_DIVE_SUPPRESSED_FIRST_SHOT_WORLD_Y_MAX: u16 = 0x4FFF;
const MUTANT_DIVE_SUPPRESSED_SECOND_SHOT_WORLD_Y_MIN: u16 = 0x9000;
const MUTANT_DIVE_SUPPRESSED_SECOND_SHOT_WORLD_Y_MAX: u16 = 0x9FFF;
const MUTANT_DIVE_VISUAL_X_VELOCITY: u16 = 0x0030;
const MUTANT_DIVE_PENDING_SHOT_TIMER_THRESHOLD: u8 = 0x80;
const MUTANT_DIVE_COLLISION_PENDING_WORLD_Y_MIN: u16 = 0x9000;

fn mutant_dive_conversion_x_correction(
    lander_reference_state: LanderReferenceState,
) -> Option<u16> {
    (lander_reference_state.target_human_index == Some(6) && lander_reference_state.x_velocity == 0)
        .then_some(MUTANT_DIVE_CONVERSION_X_CORRECTION)
}

fn mutant_dive_has_conversion_correction(
    reference_state: MutantReferenceState,
) -> bool {
    reference_state.render_x_correction == MUTANT_DIVE_CONVERSION_X_CORRECTION
}

fn mutant_dive_uses_path_projection(reference_state: MutantReferenceState) -> bool {
    mutant_dive_has_conversion_correction(reference_state)
        && reference_state.y_velocity == MUTANT_DIVE_PATH_Y_VELOCITY
}

fn mutant_dive_defers_first_shot(
    position: Point,
    reference_state: MutantReferenceState,
) -> bool {
    mutant_dive_has_conversion_correction(reference_state)
        && !reference_state.dive_entry_shot_deferred
        && position.x <= MUTANT_DIVE_ENTRY_SHOT_MAX_X
        && position.y <= MUTANT_DIVE_ENTRY_SHOT_MAX_Y
}

fn mutant_dive_fires_visible_entry_shot(
    position: Point,
    reference_state: MutantReferenceState,
    player_position: Point,
) -> bool {
    mutant_dive_has_conversion_correction(reference_state)
        && !reference_state.dive_entry_shot_deferred
        && reference_state.shot_timer == MUTANT_DIVE_DEFERRED_SHOT_TIMER
        && reference_state.sleep_ticks == MUTANT_LOOP_SLEEP_TICKS
        && position.x <= MUTANT_DIVE_ENTRY_SHOT_MAX_X
        && position.y <= MUTANT_DIVE_ENTRY_SHOT_MAX_Y
        && player_position.y <= FIRST_WAVE_RESCUE_AIM_PLAYER_MIN_Y
}

fn mutant_dive_suppresses_regular_shot(
    position: Point,
    reference_state: MutantReferenceState,
) -> bool {
    if !mutant_dive_uses_path_projection(reference_state) {
        return false;
    }

    let (_, world_y_word) =
        world_position_words(position, reference_state.x_fraction, reference_state.y_fraction);
    (MUTANT_DIVE_SUPPRESSED_FIRST_SHOT_WORLD_Y_MIN
        ..=MUTANT_DIVE_SUPPRESSED_FIRST_SHOT_WORLD_Y_MAX)
        .contains(&world_y_word)
        || (MUTANT_DIVE_SUPPRESSED_SECOND_SHOT_WORLD_Y_MIN
            ..=MUTANT_DIVE_SUPPRESSED_SECOND_SHOT_WORLD_Y_MAX)
            .contains(&world_y_word)
}

fn mutant_dive_fires_path_shot(
    position: Point,
    reference_state: MutantReferenceState,
) -> bool {
    if !mutant_dive_uses_path_projection(reference_state)
        || !reference_state.dive_entry_shot_deferred
        || reference_state.sleep_ticks != 0
    {
        return false;
    }

    matches!(
        world_position_words(position, reference_state.x_fraction, reference_state.y_fraction),
        MUTANT_DIVE_FIRST_SHOT_WORLD_WORDS | MUTANT_DIVE_SECOND_SHOT_WORLD_WORDS
    )
}

fn mutant_dive_post_shot_timer(
    reference_state: MutantReferenceState,
    fired: bool,
) -> Option<u8> {
    (fired && mutant_dive_has_conversion_correction(reference_state))
        .then_some(MUTANT_DIVE_POST_SHOT_TIMER)
}

fn mutant_dive_path_position(
    position: Point,
    reference_state: MutantReferenceState,
) -> Option<Point> {
    if !mutant_dive_uses_path_projection(reference_state) {
        return None;
    }

    let (world_x_word, world_y_word) =
        world_position_words(position, reference_state.x_fraction, reference_state.y_fraction);
    if let Some(anchor) = MUTANT_DIVE_REFERENCE_PATH_ANCHORS
        .iter()
        .find(|anchor| anchor.world_x_word == world_x_word && anchor.world_y_word == world_y_word)
    {
        return Some(anchor.screen);
    }

    mutant_dive_interpolated_path_position(world_y_word)
}

fn mutant_dive_interpolated_path_position(world_y_word: u16) -> Option<Point> {
    let first = MUTANT_DIVE_REFERENCE_PATH_ANCHORS.first()?;
    let last = MUTANT_DIVE_REFERENCE_PATH_ANCHORS.last()?;
    if world_y_word < first.world_y_word || world_y_word > last.world_y_word {
        return None;
    }

    MUTANT_DIVE_REFERENCE_PATH_ANCHORS
        .windows(2)
        .find_map(|anchors| {
            let start = anchors[0];
            let end = anchors[1];
            if world_y_word < start.world_y_word || world_y_word > end.world_y_word || start.world_y_word >= end.world_y_word {
                return None;
            }

            Some(Point::new(
                lerp_i16(
                    start.screen.x,
                    end.screen.x,
                    world_y_word,
                    start.world_y_word,
                    end.world_y_word,
                ),
                lerp_i16(
                    start.screen.y,
                    end.screen.y,
                    world_y_word,
                    start.world_y_word,
                    end.world_y_word,
                ),
            ))
        })
}

fn lerp_i16(
    start: i16,
    end: i16,
    value: u16,
    start_value: u16,
    end_value: u16,
) -> i16 {
    let numerator = i32::from(value.wrapping_sub(start_value));
    let denominator = i32::from(end_value.wrapping_sub(start_value));
    let start = i32::from(start);
    let delta = i32::from(end) - start;
    let rounded = start + ((delta * numerator) + (denominator / 2)) / denominator;
    rounded.clamp(0, i32::from(u8::MAX)) as i16
}

fn mutant_dive_visual_position(
    position: Point,
    reference_state: MutantReferenceState,
) -> Option<Point> {
    if !mutant_dive_has_conversion_correction(reference_state)
        || reference_state.x_velocity != MUTANT_DIVE_VISUAL_X_VELOCITY
    {
        return None;
    }

    let world_x_word = absolute_world_x(position, reference_state.x_fraction)
        .wrapping_add(MUTANT_DIVE_VISUAL_X_CORRECTION);
    if (world_x_word as i16) < 0 {
        return None;
    }
    let screen_x = world_x_word >> OBJECT_WORLD_TO_SCREEN_SHIFT;
    if screen_x >= OBJECT_VISIBLE_SCREEN_WIDTH {
        return None;
    }
    let screen_y = MUTANT_DIVE_REFERENCE_VISUAL_ROWS
        .iter()
        .find_map(|(row_world_x_word, screen_y)| (*row_world_x_word == world_x_word).then_some(*screen_y))?;
    Some(Point::new(screen_x as i16, screen_y))
}

fn mutant_dive_scene_position(
    position: Point,
    reference_state: Option<MutantReferenceState>,
) -> Point {
    let Some(reference_state) = reference_state else {
        return position;
    };
    mutant_dive_path_position(position, reference_state)
        .or_else(|| mutant_dive_visual_position(position, reference_state))
        .unwrap_or(position)
}

fn mutant_dive_collision_position(
    position: Point,
    reference_state: Option<MutantReferenceState>,
) -> Point {
    let Some(reference_state) = reference_state else {
        return position;
    };
    if let Some(position) = mutant_dive_path_position(position, reference_state) {
        return position.offset(Velocity::new(0, 1));
    }
    mutant_dive_visual_position(position, reference_state).unwrap_or(position)
}

fn mutant_dive_collision_window_pending(
    position: Point,
    reference_state: Option<MutantReferenceState>,
) -> bool {
    let Some(reference_state) = reference_state else {
        return false;
    };
    if !mutant_dive_uses_path_projection(reference_state) {
        return false;
    }

    let (_, world_y_word) =
        world_position_words(position, reference_state.x_fraction, reference_state.y_fraction);
    reference_state.shot_timer >= MUTANT_DIVE_PENDING_SHOT_TIMER_THRESHOLD
        && (MUTANT_DIVE_COLLISION_PENDING_WORLD_Y_MIN..MUTANT_DIVE_COLLISION_WORLD_Y_MIN)
            .contains(&world_y_word)
}

fn mutant_dive_uses_collision_projection(
    position: Point,
    reference_state: Option<MutantReferenceState>,
) -> bool {
    let Some(reference_state) = reference_state else {
        return false;
    };
    if !mutant_dive_uses_path_projection(reference_state) {
        return false;
    }

    let (_, world_y_word) =
        world_position_words(position, reference_state.x_fraction, reference_state.y_fraction);
    reference_state.shot_timer >= MUTANT_DIVE_PENDING_SHOT_TIMER_THRESHOLD
        && (MUTANT_DIVE_COLLISION_WORLD_Y_MIN
            ..MUTANT_DIVE_COLLISION_WORLD_Y_MAX)
            .contains(&world_y_word)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorExplosionPlacement {
    position: Point,
    explosion_anchor: Option<Point>,
}

fn actor_player_enemy_collision_explosion_placement(
    enemy: &CollisionBody,
) -> ActorExplosionPlacement {
    if mutant_dive_uses_collision_projection(
        enemy.position,
        enemy.reference_state.as_mutant(),
    ) {
        ActorExplosionPlacement {
            position: MUTANT_DIVE_COLLISION_EXPLOSION_TOP_LEFT,
            explosion_anchor: Some(MUTANT_DIVE_COLLISION_EXPLOSION_ANCHOR),
        }
    } else {
        ActorExplosionPlacement {
            position: center_of(enemy.bounds),
            explosion_anchor: None,
        }
    }
}

fn mutant_dive_shot_position(
    position: Point,
    reference_state: MutantReferenceState,
) -> Point {
    if !mutant_dive_uses_path_projection(reference_state) {
        return position;
    }

    match world_position_words(position, reference_state.x_fraction, reference_state.y_fraction) {
        MUTANT_DIVE_ENTRY_WORLD_WORDS => MUTANT_DIVE_ENTRY_SHOT_SCREEN,
        MUTANT_DIVE_FIRST_SHOT_WORLD_WORDS => MUTANT_DIVE_FIRST_PATH_SHOT_SCREEN,
        MUTANT_DIVE_SECOND_SHOT_WORLD_WORDS => MUTANT_DIVE_SECOND_PATH_SHOT_SCREEN,
        _ => mutant_dive_path_position(position, reference_state).unwrap_or(position),
    }
}

fn mutant_dive_forced_shot(
    position: Point,
    reference_state: MutantReferenceState,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
) -> Option<(Point, Velocity, EnemyProjectileReferenceState)> {
    if !mutant_dive_uses_path_projection(reference_state)
        || actor_enemy_projectile_count(prompt) >= ENEMY_PROJECTILE_SLOT_LIMIT
    {
        return None;
    }

    match world_position_words(position, reference_state.x_fraction, reference_state.y_fraction) {
        MUTANT_DIVE_FORCED_FIRST_SHOT_WORLD_WORDS
            if reference_state.sleep_ticks == MUTANT_LOOP_SLEEP_TICKS =>
        {
            Some(mutant_dive_exact_projectile(
                MUTANT_DIVE_REFERENCE_FORCED_FIRST_PROJECTILE,
                behavior,
            ))
        }
        MUTANT_DIVE_FORCED_SECOND_SHOT_WORLD_WORDS
            if reference_state.sleep_ticks == MUTANT_LOOP_SLEEP_TICKS =>
        {
            Some(mutant_dive_exact_projectile(
                MUTANT_DIVE_REFERENCE_FORCED_SECOND_PROJECTILE,
                behavior,
            ))
        }
        _ => None,
    }
}

fn mutant_dive_exact_projectile(
    projectile: MutantDiveReferenceProjectile,
    behavior: ActorBehaviorProfile,
) -> (Point, Velocity, EnemyProjectileReferenceState) {
    (
        projectile.position,
        screen_velocity_from_motion_words(projectile.x_velocity, projectile.y_velocity),
        EnemyProjectileReferenceState {
            x_fraction: projectile.x_fraction,
            y_fraction: projectile.y_fraction,
            x_velocity: projectile.x_velocity,
            y_velocity: projectile.y_velocity,
            lifetime_ticks: projectile_lifetime_ticks(
                behavior.mutant_shot_lifetime_steps,
            ),
        },
    )
}
