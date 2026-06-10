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
const MUTANT_DIVE_ENTRY_SHOT_SCREEN: Point = Point::new(0x13, 0x46);
const MUTANT_DIVE_FIRST_PATH_SHOT_SCREEN: Point = Point::new(0x1E, 0x70);
const MUTANT_DIVE_SECOND_PATH_SHOT_SCREEN: Point = Point::new(0x21, 0x87);
const MUTANT_DIVE_FORCED_FIRST_PROJECTILE: MutantDiveExactProjectile = MutantDiveExactProjectile {
    position: Point::new(0x1E, 0x54),
    x_fraction: 0x33,
    y_fraction: 0x56,
    x_velocity: 0xFFE0,
    y_velocity: 0x0138,
};
const MUTANT_DIVE_FORCED_SECOND_PROJECTILE: MutantDiveExactProjectile =
    MutantDiveExactProjectile {
        position: Point::new(0x21, 0x7F),
        x_fraction: 0x6F,
        y_fraction: 0xE1,
        x_velocity: 0xFFF0,
        y_velocity: 0x00C0,
    };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MutantDiveExactProjectile {
    position: Point,
    x_fraction: u8,
    y_fraction: u8,
    x_velocity: u16,
    y_velocity: u16,
}

fn mutant_dive_arcade_conversion_x_correction(
    lander_runtime: LanderArcadeState,
) -> Option<u16> {
    (lander_runtime.target_human_index == Some(6) && lander_runtime.x_velocity == 0)
        .then_some(MUTANT_DIVE_CONVERSION_X_CORRECTION)
}

fn mutant_dive_has_conversion_correction(
    arcade_state: MutantArcadeState,
) -> bool {
    arcade_state.render_x_correction == MUTANT_DIVE_CONVERSION_X_CORRECTION
}

fn mutant_dive_uses_path_projection(arcade_state: MutantArcadeState) -> bool {
    mutant_dive_has_conversion_correction(arcade_state)
        && arcade_state.y_velocity == MUTANT_DIVE_PATH_Y_VELOCITY
}

fn mutant_dive_defers_first_shot(
    position: Point,
    arcade_state: MutantArcadeState,
) -> bool {
    mutant_dive_has_conversion_correction(arcade_state)
        && !arcade_state.dive_entry_shot_deferred
        && position.x <= MUTANT_DIVE_ENTRY_SHOT_MAX_X
        && position.y <= MUTANT_DIVE_ENTRY_SHOT_MAX_Y
}

fn mutant_dive_fires_visible_entry_shot(
    position: Point,
    arcade_state: MutantArcadeState,
    player_position: Point,
) -> bool {
    mutant_dive_has_conversion_correction(arcade_state)
        && !arcade_state.dive_entry_shot_deferred
        && arcade_state.shot_timer == MUTANT_DIVE_DEFERRED_SHOT_TIMER
        && arcade_state.sleep_ticks == MUTANT_LOOP_SLEEP_TICKS
        && position.x <= MUTANT_DIVE_ENTRY_SHOT_MAX_X
        && position.y <= MUTANT_DIVE_ENTRY_SHOT_MAX_Y
        && player_position.y <= FIRST_WAVE_RESCUE_AIM_PLAYER_MIN_Y
}

fn mutant_dive_suppresses_regular_shot(
    position: Point,
    arcade_state: MutantArcadeState,
) -> bool {
    if !mutant_dive_uses_path_projection(arcade_state) {
        return false;
    }

    let (_, world_y_word) =
        arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction);
    (MUTANT_DIVE_SUPPRESSED_FIRST_SHOT_WORLD_Y_MIN
        ..=MUTANT_DIVE_SUPPRESSED_FIRST_SHOT_WORLD_Y_MAX)
        .contains(&world_y_word)
        || (MUTANT_DIVE_SUPPRESSED_SECOND_SHOT_WORLD_Y_MIN
            ..=MUTANT_DIVE_SUPPRESSED_SECOND_SHOT_WORLD_Y_MAX)
            .contains(&world_y_word)
}

fn mutant_dive_fires_path_shot(
    position: Point,
    arcade_state: MutantArcadeState,
) -> bool {
    if !mutant_dive_uses_path_projection(arcade_state)
        || !arcade_state.dive_entry_shot_deferred
        || arcade_state.sleep_ticks != 0
    {
        return false;
    }

    matches!(
        arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction),
        MUTANT_DIVE_FIRST_SHOT_WORLD_WORDS | MUTANT_DIVE_SECOND_SHOT_WORLD_WORDS
    )
}

fn mutant_dive_post_shot_timer(
    arcade_state: MutantArcadeState,
    fired: bool,
) -> Option<u8> {
    (fired && mutant_dive_has_conversion_correction(arcade_state))
        .then_some(MUTANT_DIVE_POST_SHOT_TIMER)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MutantDivePathAnchor {
    world_x_word: u16,
    world_y_word: u16,
    screen: Point,
}

const MUTANT_DIVE_PATH_ANCHORS: &[MutantDivePathAnchor] = &[
    // original: ACTOR_SOURCE_TARGET6_MUTANT_DIVE_PROJECTIONS
    MutantDivePathAnchor {
        world_x_word: 0x031C,
        world_y_word: 0x3360,
        screen: Point::new(0x12, 0x43),
    },
    MutantDivePathAnchor {
        world_x_word: 0x037C,
        world_y_word: 0x3380,
        screen: Point::new(0x13, 0x46),
    },
    MutantDivePathAnchor {
        world_x_word: 0x034C,
        world_y_word: 0x33F0,
        screen: Point::new(0x12, 0x43),
    },
    MutantDivePathAnchor {
        world_x_word: 0x03AC,
        world_y_word: 0x3410,
        screen: Point::new(0x14, 0x46),
    },
    MutantDivePathAnchor {
        world_x_word: 0x037C,
        world_y_word: 0x3480,
        screen: Point::new(0x13, 0x44),
    },
    MutantDivePathAnchor {
        world_x_word: 0x085C,
        world_y_word: 0x47A0,
        screen: Point::new(0x1F, 0x5B),
    },
    MutantDivePathAnchor {
        world_x_word: 0x085C,
        world_y_word: 0x6120,
        screen: Point::new(0x1F, 0x71),
    },
    MutantDivePathAnchor {
        world_x_word: 0x088C,
        world_y_word: 0x61B0,
        screen: Point::new(0x1E, 0x71),
    },
    MutantDivePathAnchor {
        world_x_word: 0x085C,
        world_y_word: 0x6140,
        screen: Point::new(0x1F, 0x71),
    },
    MutantDivePathAnchor {
        world_x_word: 0x082C,
        world_y_word: 0x7770,
        screen: Point::new(0x20, 0x87),
    },
    MutantDivePathAnchor {
        world_x_word: 0x07FC,
        world_y_word: 0x7800,
        screen: Point::new(0x21, 0x88),
    },
    MutantDivePathAnchor {
        world_x_word: 0x082C,
        world_y_word: 0x7990,
        screen: Point::new(0x20, 0x87),
    },
    MutantDivePathAnchor {
        world_x_word: 0x082C,
        world_y_word: 0x81E0,
        screen: Point::new(0x20, 0x90),
    },
    MutantDivePathAnchor {
        world_x_word: 0x082C,
        world_y_word: 0x9730,
        screen: Point::new(0x21, 0x9F),
    },
    MutantDivePathAnchor {
        world_x_word: 0x07FC,
        world_y_word: 0x97A0,
        screen: Point::new(0x20, 0x9E),
    },
    MutantDivePathAnchor {
        world_x_word: 0x085C,
        world_y_word: 0x97C0,
        screen: Point::new(0x20, 0xA0),
    },
    MutantDivePathAnchor {
        world_x_word: 0x088C,
        world_y_word: 0x9850,
        screen: Point::new(0x1F, 0xA0),
    },
    MutantDivePathAnchor {
        world_x_word: 0x085C,
        world_y_word: 0x99E0,
        screen: Point::new(0x1E, 0xA2),
    },
    MutantDivePathAnchor {
        world_x_word: 0x082C,
        world_y_word: 0x9A70,
        screen: Point::new(0x20, 0xA3),
    },
    MutantDivePathAnchor {
        world_x_word: 0x088C,
        world_y_word: 0xA200,
        screen: Point::new(0x20, 0xA2),
    },
    MutantDivePathAnchor {
        world_x_word: 0x08EC,
        world_y_word: 0xA320,
        screen: Point::new(0x20, 0xA2),
    },
];

const MUTANT_DIVE_VISUAL_ROWS: &[(u16, i16)] = &[
    // original: ACTOR_SOURCE_TARGET6_MUTANT_VISUAL_ROWS
    (0x0004, 0x36),
    (0x0034, 0x36),
    (0x0064, 0x37),
    (0x0094, 0x37),
    (0x00C4, 0x37),
    (0x00F4, 0x37),
    (0x0124, 0x36),
    (0x0154, 0x36),
    (0x0184, 0x37),
    (0x01B4, 0x37),
    (0x01E4, 0x37),
    (0x0214, 0x37),
    (0x0244, 0x36),
    (0x0274, 0x36),
    (0x02A4, 0x36),
    (0x02D4, 0x35),
    (0x0304, 0x34),
    (0x0334, 0x34),
    (0x0364, 0x32),
    (0x0394, 0x31),
    (0x03C4, 0x30),
    (0x03F4, 0x2F),
    (0x0424, 0x2F),
    (0x0454, 0x2E),
    (0x0484, 0x2D),
    (0x04B4, 0x2C),
    (0x04E4, 0x2B),
    (0x0514, 0x2C),
    (0x0544, 0x2B),
    (0x0574, 0x2B),
    (0x05A4, 0x2B),
    (0x05D4, 0x2B),
    (0x0604, 0x2A),
    (0x0634, 0x2C),
    (0x0664, 0x2C),
    (0x0694, 0x2D),
    (0x06C4, 0x2B),
    (0x06F4, 0x2B),
    (0x0724, 0x2A),
    (0x0754, 0x2C),
];

fn mutant_dive_path_position(
    position: Point,
    arcade_state: MutantArcadeState,
) -> Option<Point> {
    if !mutant_dive_uses_path_projection(arcade_state) {
        return None;
    }

    let (world_x_word, world_y_word) =
        arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction);
    if let Some(anchor) = MUTANT_DIVE_PATH_ANCHORS
        .iter()
        .find(|anchor| anchor.world_x_word == world_x_word && anchor.world_y_word == world_y_word)
    {
        return Some(anchor.screen);
    }

    mutant_dive_interpolated_path_position(world_y_word)
}

fn mutant_dive_interpolated_path_position(world_y_word: u16) -> Option<Point> {
    let first = MUTANT_DIVE_PATH_ANCHORS.first()?;
    let last = MUTANT_DIVE_PATH_ANCHORS.last()?;
    if world_y_word < first.world_y_word || world_y_word > last.world_y_word {
        return None;
    }

    MUTANT_DIVE_PATH_ANCHORS
        .windows(2)
        .find_map(|anchors| {
            let start = anchors[0];
            let end = anchors[1];
            if world_y_word < start.world_y_word || world_y_word > end.world_y_word || start.world_y_word >= end.world_y_word {
                return None;
            }

            Some(Point::new(
                arcade_lerp_i16(
                    start.screen.x,
                    end.screen.x,
                    world_y_word,
                    start.world_y_word,
                    end.world_y_word,
                ),
                arcade_lerp_i16(
                    start.screen.y,
                    end.screen.y,
                    world_y_word,
                    start.world_y_word,
                    end.world_y_word,
                ),
            ))
        })
}

fn arcade_lerp_i16(
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
    arcade_state: MutantArcadeState,
) -> Option<Point> {
    if !mutant_dive_has_conversion_correction(arcade_state)
        || arcade_state.x_velocity != MUTANT_DIVE_VISUAL_X_VELOCITY
    {
        return None;
    }

    let world_x_word = arcade_absolute_x(position, arcade_state.x_fraction)
        .wrapping_add(MUTANT_DIVE_VISUAL_X_CORRECTION);
    if (world_x_word as i16) < 0 {
        return None;
    }
    let screen_x = world_x_word >> OBJECT_WORLD_TO_SCREEN_SHIFT;
    if screen_x >= OBJECT_VISIBLE_SCREEN_WIDTH {
        return None;
    }
    let screen_y = MUTANT_DIVE_VISUAL_ROWS
        .iter()
        .find_map(|(row_world_x_word, screen_y)| (*row_world_x_word == world_x_word).then_some(*screen_y))?;
    Some(Point::new(screen_x as i16, screen_y))
}

fn mutant_dive_scene_position(
    position: Point,
    arcade_state: Option<MutantArcadeState>,
) -> Point {
    let Some(arcade_state) = arcade_state else {
        return position;
    };
    mutant_dive_path_position(position, arcade_state)
        .or_else(|| mutant_dive_visual_position(position, arcade_state))
        .unwrap_or(position)
}

fn mutant_dive_collision_position(
    position: Point,
    arcade_state: Option<MutantArcadeState>,
) -> Point {
    let Some(arcade_state) = arcade_state else {
        return position;
    };
    if let Some(position) = mutant_dive_path_position(position, arcade_state) {
        return position.offset(Velocity::new(0, 1));
    }
    mutant_dive_visual_position(position, arcade_state).unwrap_or(position)
}

fn mutant_dive_collision_window_pending(
    position: Point,
    arcade_state: Option<MutantArcadeState>,
) -> bool {
    let Some(arcade_state) = arcade_state else {
        return false;
    };
    if !mutant_dive_uses_path_projection(arcade_state) {
        return false;
    }

    let (_, world_y_word) =
        arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction);
    arcade_state.shot_timer >= MUTANT_DIVE_PENDING_SHOT_TIMER_THRESHOLD
        && (MUTANT_DIVE_COLLISION_PENDING_WORLD_Y_MIN..MUTANT_DIVE_COLLISION_WORLD_Y_MIN)
            .contains(&world_y_word)
}

fn mutant_dive_uses_collision_projection(
    position: Point,
    arcade_state: Option<MutantArcadeState>,
) -> bool {
    let Some(arcade_state) = arcade_state else {
        return false;
    };
    if !mutant_dive_uses_path_projection(arcade_state) {
        return false;
    }

    let (_, world_y_word) =
        arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction);
    arcade_state.shot_timer >= MUTANT_DIVE_PENDING_SHOT_TIMER_THRESHOLD
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
        enemy.mutant_runtime,
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
    arcade_state: MutantArcadeState,
) -> Point {
    if !mutant_dive_uses_path_projection(arcade_state) {
        return position;
    }

    match arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction) {
        MUTANT_DIVE_ENTRY_WORLD_WORDS => MUTANT_DIVE_ENTRY_SHOT_SCREEN,
        MUTANT_DIVE_FIRST_SHOT_WORLD_WORDS => MUTANT_DIVE_FIRST_PATH_SHOT_SCREEN,
        MUTANT_DIVE_SECOND_SHOT_WORLD_WORDS => MUTANT_DIVE_SECOND_PATH_SHOT_SCREEN,
        _ => mutant_dive_path_position(position, arcade_state).unwrap_or(position),
    }
}

fn mutant_dive_forced_shot(
    position: Point,
    arcade_state: MutantArcadeState,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
) -> Option<(Point, Velocity, EnemyProjectileArcadeState)> {
    if !mutant_dive_uses_path_projection(arcade_state)
        || actor_enemy_projectile_count(prompt) >= ENEMY_PROJECTILE_SLOT_LIMIT
    {
        return None;
    }

    match arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction) {
        MUTANT_DIVE_FORCED_FIRST_SHOT_WORLD_WORDS
            if arcade_state.sleep_ticks == MUTANT_LOOP_SLEEP_TICKS =>
        {
            Some(mutant_dive_exact_projectile(
                MUTANT_DIVE_FORCED_FIRST_PROJECTILE,
                behavior,
            ))
        }
        MUTANT_DIVE_FORCED_SECOND_SHOT_WORLD_WORDS
            if arcade_state.sleep_ticks == MUTANT_LOOP_SLEEP_TICKS =>
        {
            Some(mutant_dive_exact_projectile(
                MUTANT_DIVE_FORCED_SECOND_PROJECTILE,
                behavior,
            ))
        }
        _ => None,
    }
}

fn mutant_dive_exact_projectile(
    projectile: MutantDiveExactProjectile,
    behavior: ActorBehaviorProfile,
) -> (Point, Velocity, EnemyProjectileArcadeState) {
    (
        projectile.position,
        arcade_screen_velocity(projectile.x_velocity, projectile.y_velocity),
        EnemyProjectileArcadeState {
            x_fraction: projectile.x_fraction,
            y_fraction: projectile.y_fraction,
            x_velocity: projectile.x_velocity,
            y_velocity: projectile.y_velocity,
            lifetime_ticks: arcade_projectile_lifetime_ticks(
                behavior.mutant_shot_lifetime_steps,
            ),
        },
    )
}
