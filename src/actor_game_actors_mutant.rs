#[derive(Debug)]
struct Mutant {
    id: ActorId,
    position: Point,
    drift: i16,
    arcade_state: Option<MutantArcadeState>,
}

impl Mutant {
    fn from_spawn(id: ActorId, spawn: ActorMutantSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .arcade_state
                .map(|arcade_state| arcade_drift_from_velocity(arcade_state.x_velocity))
                .unwrap_or(-1),
            arcade_state: spawn.arcade_state,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.collision_position(), 14, 12)
    }

    fn scene_position(&self) -> Point {
        mutant_dive_scene_position(self.position, self.arcade_state)
    }

    fn collision_position(&self) -> Point {
        mutant_dive_collision_position(self.position, self.arcade_state)
    }

    fn advance_arcade_motion(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(arcade_state) = &mut self.arcade_state else {
            return false;
        };
        if arcade_state.sleep_ticks > 0 {
            if let Some((position, velocity, projectile_arcade_state)) =
                mutant_dive_forced_shot(
                    self.position,
                    *arcade_state,
                    prompt,
                    behavior,
                )
            {
                arcade_state.dive_entry_shot_deferred = true;
                arcade_state.shot_timer = MUTANT_DIVE_COLLISION_PENDING_SHOT_TIMER;
                push_arcade_enemy_projectile_command(
                    position,
                    velocity,
                    projectile_arcade_state,
                    SoundCue::MutantShot,
                    commands,
                );
            }
            if let Some(player_position) = prompt.player_position()
                && mutant_dive_fires_visible_entry_shot(
                    self.position,
                    *arcade_state,
                    player_position,
                )
            {
                arcade_state.dive_entry_shot_deferred = true;
                let shot_rng = mutant_arcade_shot_rng(prompt, self.id, self.position);
                let shot_position =
                    mutant_dive_shot_position(self.position, *arcade_state);
                push_mutant_arcade_shot(
                    shot_position,
                    prompt,
                    behavior,
                    *arcade_state,
                    shot_rng,
                    commands,
                );
            }
            arcade_state.sleep_ticks = arcade_state.sleep_ticks.saturating_sub(1);
            return true;
        }

        let Some(player_position) = prompt.player_position() else {
            return false;
        };
        let profile = prompt.arcade_wave;
        let player_absolute_x = arcade_absolute_x(player_position, 0);
        let object_absolute_x = arcade_absolute_x(self.position, arcade_state.x_fraction);
        arcade_state.x_velocity = mutant_arcade_x_velocity(
            profile.mutant_x_velocity,
            player_absolute_x,
            object_absolute_x,
        );
        arcade_state.y_velocity = mutant_arcade_y_velocity(
            profile,
            player_position.y,
            player_absolute_x,
            object_absolute_x,
            self.position,
        );

        let mut next_sleep_ticks = MUTANT_LOOP_SLEEP_TICKS;
        if mutant_arcade_should_hop_and_shoot(
            player_absolute_x,
            object_absolute_x,
            self.position,
        ) {
            let forced_dive_shot =
                mutant_dive_fires_path_shot(self.position, *arcade_state);
            let forced_dive_shot_position = self.position;
            let mut hop_rng = arcade_rng_from_snapshot(arcade_state.hop_rng);
            let hop_state = hop_rng.advance();
            arcade_state.hop_rng = hop_state.snapshot();
            self.position.y =
                mutant_arcade_hop_y(self.position.y, profile.mutant_random_y, hop_state.seed);

            if forced_dive_shot {
                let shot_rng = mutant_arcade_shot_rng(prompt, self.id, self.position);
                let shot_position = mutant_dive_shot_position(
                    forced_dive_shot_position,
                    *arcade_state,
                );
                push_mutant_arcade_shot(
                    shot_position,
                    prompt,
                    behavior,
                    *arcade_state,
                    shot_rng,
                    commands,
                );
                arcade_state.shot_timer = MUTANT_DIVE_POST_SHOT_TIMER;
            } else {
                arcade_state.shot_timer = arcade_state.shot_timer.wrapping_sub(1);
                if arcade_state.shot_timer == 0 {
                    if mutant_dive_suppresses_regular_shot(
                        self.position,
                        *arcade_state,
                    ) {
                        arcade_state.shot_timer = MUTANT_DIVE_COLLISION_PENDING_SHOT_TIMER;
                    } else if mutant_dive_defers_first_shot(self.position, *arcade_state)
                    {
                        arcade_state.dive_entry_shot_deferred = true;
                        arcade_state.shot_timer = MUTANT_DIVE_DEFERRED_SHOT_TIMER;
                        next_sleep_ticks = 0;
                    } else {
                        let shot_rng = mutant_arcade_shot_rng(prompt, self.id, self.position);
                        let default_reset = mutant_arcade_shot_reset(profile, shot_rng.seed);
                        let shot_position =
                            mutant_dive_shot_position(self.position, *arcade_state);
                        let fired = push_mutant_arcade_shot(
                            shot_position,
                            prompt,
                            behavior,
                            *arcade_state,
                            shot_rng,
                            commands,
                        );
                        arcade_state.shot_timer =
                            mutant_dive_post_shot_timer(*arcade_state, fired)
                                .unwrap_or(default_reset);
                    }
                }
            }
        }

        let (x, x_fraction) = arcade_axis_step(
            self.position.x,
            arcade_state.x_fraction,
            arcade_state.x_velocity,
        );
        let (y, y_fraction) = arcade_active_object_y_step(
            self.position.y,
            arcade_state.y_fraction,
            arcade_state.y_velocity,
        );
        self.position = Point::new(x, y);
        arcade_state.x_fraction = x_fraction;
        arcade_state.y_fraction = y_fraction;
        arcade_state.sleep_ticks = next_sleep_ticks;
        self.drift = arcade_drift_from_velocity(arcade_state.x_velocity);
        true
    }
}

impl AssetActor for Mutant {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Mutant);
            if !self.advance_arcade_motion(prompt, behavior, &mut commands)
                && let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.mutant_mode,
                    prompt,
                    behavior.mutant_seek_speed,
                    self.drift,
                )
            {
                self.position = position;
            }
            draws.push(DrawCommand::sprite(
                self.id,
                SpriteKey::Mutant,
                self.scene_position(),
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Mutant,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(
                    movement_velocity,
                    drift_direction(self.drift),
                )),
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                lander_runtime: None,
                bomber_runtime: None,
                pod_runtime: None,
                swarmer_runtime: None,
                baiter_runtime: None,
                mutant_runtime: self.arcade_state,
                human_runtime: None,
                enemy_projectile_runtime: None,
            },
            commands,
            draws,
        }
    }
}

fn arcade_absolute_x(position: Point, x_fraction: u8) -> u16 {
    u16::from_be_bytes([position.x as u8, x_fraction])
}

const fn arcade_hyperspace_background_left(arcade_seed: ActorHyperspaceArcadeSeed) -> u16 {
    u16::from_be_bytes([arcade_seed.seed, arcade_seed.hseed])
}

fn arcade_world_position(position: Point, x_fraction: u8, y_fraction: u8) -> (u16, u16) {
    (
        u16::from_be_bytes([position.x as u8, x_fraction]),
        u16::from_be_bytes([position.y as u8, y_fraction]),
    )
}

const fn arcade_rng_from_snapshot(snapshot: ActorArcadeRngSnapshot) -> ActorArcadeRng {
    ActorArcadeRng {
        seed: snapshot.seed,
        hseed: snapshot.hseed,
        lseed: snapshot.lseed,
    }
}

fn mutant_arcade_x_velocity(
    x_velocity_word: u8,
    player_absolute_x: u16,
    object_absolute_x: u16,
) -> u16 {
    let x_velocity_low = if (player_absolute_x as i16) >= (object_absolute_x as i16) {
        x_velocity_word
    } else {
        0u8.wrapping_sub(x_velocity_word)
    };
    actor_sign_extend_u8_to_u16(x_velocity_low)
}

fn mutant_arcade_y_velocity(
    profile: ArcadeWaveProfile,
    player_y: i16,
    player_absolute_x: u16,
    object_absolute_x: u16,
    position: Point,
) -> u16 {
    let base_y_velocity =
        u16::from_be_bytes([profile.mutant_y_velocity_msb, profile.mutant_y_velocity_lsb]);
    let player_y = player_y as u8;
    let position_y = position.y as u8;
    let x_distance = player_absolute_x
        .wrapping_sub(object_absolute_x)
        .wrapping_add(MUTANT_X_DISTANCE_OFFSET);
    if x_distance <= MUTANT_CLOSE_X_WINDOW {
        if player_y >= position_y {
            base_y_velocity
        } else {
            !base_y_velocity
        }
    } else {
        let delta = player_y.wrapping_sub(position_y);
        if player_y > position_y {
            if delta > MUTANT_VERTICAL_WINDOW {
                0
            } else {
                !base_y_velocity
            }
        } else if (delta as i8) > -(MUTANT_VERTICAL_WINDOW as i8) {
            base_y_velocity
        } else {
            0
        }
    }
}

fn mutant_arcade_should_hop_and_shoot(
    player_absolute_x: u16,
    object_absolute_x: u16,
    position: Point,
) -> bool {
    let x_distance = player_absolute_x
        .wrapping_sub(object_absolute_x)
        .wrapping_add(MUTANT_X_DISTANCE_OFFSET);
    x_distance > MUTANT_CLOSE_X_WINDOW
        || (position.y > i16::from(PLAYFIELD_TOP_EDGE_Y)
            && position.y <= i16::from(PLAYFIELD_BOTTOM_EDGE_Y))
}

fn mutant_arcade_hop_y(position_y: i16, random_y: u8, seed: u8) -> i16 {
    let step = if seed & 0x80 == 0 {
        0u8.wrapping_sub(random_y)
    } else {
        random_y
    };
    let mut y = (position_y as u8).wrapping_add(step);
    if y < PLAYFIELD_TOP_EDGE_Y {
        y = PLAYFIELD_BOTTOM_EDGE_Y;
    }
    i16::from(y)
}

fn mutant_arcade_shot_rng(
    prompt: &StepPrompt,
    actor: ActorId,
    position: Point,
) -> ActorArcadeRngSnapshot {
    let mut arcade_rng = prompt
        .arcade_rng
        .map(arcade_rng_from_snapshot)
        .unwrap_or(ActorArcadeRng {
            seed: arcade_motion_seed(prompt.step, actor),
            hseed: position.x as u8,
            lseed: position.y as u8,
        });
    arcade_rng.advance().snapshot()
}

fn mutant_arcade_shot_reset(profile: ArcadeWaveProfile, seed: u8) -> u8 {
    arcade_rmax(
        profile.mutant_shot_time.max(1).min(u32::from(u8::MAX)) as u8,
        seed,
    )
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
        && arcade_state.y_velocity == 0x0090
}

fn mutant_dive_defers_first_shot(
    position: Point,
    arcade_state: MutantArcadeState,
) -> bool {
    mutant_dive_has_conversion_correction(arcade_state)
        && !arcade_state.dive_entry_shot_deferred
        && position.x <= 0x04
        && position.y <= 0x60
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
        && position.x <= 0x04
        && position.y <= 0x60
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
    (0x4000..=0x4FFF).contains(&world_y_word) || (0x9000..=0x9FFF).contains(&world_y_word)
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
        || arcade_state.x_velocity != 0x0030
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
    arcade_state.shot_timer >= 0x80
        && (0x9000..MUTANT_DIVE_COLLISION_WORLD_Y_MIN).contains(&world_y_word)
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
    arcade_state.shot_timer >= 0x80
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
        MUTANT_DIVE_ENTRY_WORLD_WORDS => Point::new(0x13, 0x46),
        MUTANT_DIVE_FIRST_SHOT_WORLD_WORDS => Point::new(0x1E, 0x70),
        MUTANT_DIVE_SECOND_SHOT_WORLD_WORDS => Point::new(0x21, 0x87),
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
                Point::new(0x1E, 0x54),
                0x33,
                0x56,
                0xFFE0,
                0x0138,
                behavior,
            ))
        }
        MUTANT_DIVE_FORCED_SECOND_SHOT_WORLD_WORDS
            if arcade_state.sleep_ticks == MUTANT_LOOP_SLEEP_TICKS =>
        {
            Some(mutant_dive_exact_projectile(
                Point::new(0x21, 0x7F),
                0x6F,
                0xE1,
                0xFFF0,
                0x00C0,
                behavior,
            ))
        }
        _ => None,
    }
}

fn mutant_dive_exact_projectile(
    position: Point,
    x_fraction: u8,
    y_fraction: u8,
    x_velocity: u16,
    y_velocity: u16,
    behavior: ActorBehaviorProfile,
) -> (Point, Velocity, EnemyProjectileArcadeState) {
    (
        position,
        arcade_screen_velocity(x_velocity, y_velocity),
        EnemyProjectileArcadeState {
            x_fraction,
            y_fraction,
            x_velocity,
            y_velocity,
            lifetime_ticks: arcade_projectile_lifetime_ticks(
                behavior.mutant_shot_lifetime_steps,
            ),
        },
    )
}

fn push_mutant_arcade_shot(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    arcade_state: MutantArcadeState,
    shot_rng: ActorArcadeRngSnapshot,
    commands: &mut Vec<GameCommand>,
) -> bool {
    let Some((velocity, projectile_arcade_state)) =
        mutant_arcade_fireball(position, prompt, behavior, arcade_state, shot_rng)
    else {
        return false;
    };
    push_arcade_enemy_projectile_command(
        position,
        velocity,
        projectile_arcade_state,
        SoundCue::MutantShot,
        commands,
    );
    true
}

fn mutant_arcade_fireball(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    arcade_state: MutantArcadeState,
    shot_rng: ActorArcadeRngSnapshot,
) -> Option<(Velocity, EnemyProjectileArcadeState)> {
    let lifetime_ticks =
        arcade_projectile_lifetime_ticks(behavior.mutant_shot_lifetime_steps);
    arcade_enemy_fireball(
        position,
        arcade_state.x_fraction,
        arcade_state.y_fraction,
        prompt,
        shot_rng,
        lifetime_ticks,
    )
}
