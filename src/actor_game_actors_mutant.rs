const MUTANT_HOP_DIRECTION_SIGN_BIT: u8 = 0x80;

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

const fn arcade_hyperspace_background_left(arcade_seed: ActorHyperspaceArcadeSeed) -> u16 {
    u16::from_be_bytes([arcade_seed.seed, arcade_seed.hseed])
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
    let step = if seed & MUTANT_HOP_DIRECTION_SIGN_BIT == 0 {
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
