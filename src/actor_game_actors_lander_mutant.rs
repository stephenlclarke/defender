#[derive(Debug)]
struct Lander {
    id: ActorId,
    position: Point,
    drift: i16,
    mode: LanderMode,
    arcade_state: Option<LanderArcadeState>,
    spawn_visibility: LanderSpawnVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LanderMode {
    Seeking,
    Carrying {
        human_id: ActorId,
        pull_sound_sent: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LanderSpawnVisibility {
    Normal,
    VisibleFirstWaveRefill,
    HiddenFirstWaveRefill,
}

impl Lander {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorLanderSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorLanderSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .source
                .map(|arcade_state| {
                    lander_drift_from_arcade_velocity(arcade_state.x_velocity)
                })
                .unwrap_or(-1),
            mode: LanderMode::Seeking,
            spawn_visibility: lander_spawn_visibility(spawn.source),
            arcade_state: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 14, 12)
    }
}

impl AssetActor for Lander {
    fn id(&self) -> ActorId {
        self.id
    }

    fn apply_driver_command(&mut self, command: ActorDriverCommand) {
        let ActorDriverCommand::AdjustLanderFireTimer {
            target_human_index,
            x_velocity,
            delta,
        } = command;
        if let Some(arcade_state) = &mut self.arcade_state
            && arcade_state.target_human_index == Some(target_human_index)
            && arcade_state.x_velocity == x_velocity
        {
            arcade_state.shot_timer = arcade_state.shot_timer.wrapping_add(delta);
        }
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Lander);
            if self.spawn_visibility != LanderSpawnVisibility::HiddenFirstWaveRefill
                && !self.tick_arcade_sleep()
            {
                match self.mode {
                    LanderMode::Seeking => self.update_seeking(prompt, behavior, &mut commands),
                    LanderMode::Carrying {
                        human_id,
                        pull_sound_sent,
                    } => {
                        self.update_carrying(
                            prompt,
                            human_id,
                            pull_sound_sent,
                            behavior,
                            &mut commands,
                        );
                    }
                }
                self.tick_fire_timer(prompt, behavior, &mut commands);
            }
            if self.output_visible() {
                draws.push(DrawCommand::sprite_with_effect(
                    self.id,
                    SpriteKey::Lander,
                    self.position,
                    self.draw_effect(),
                ));
            }
        }
        let movement_velocity = observed_velocity(previous_position, self.position);
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Lander,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(
                    movement_velocity,
                    drift_direction(self.drift),
                )),
                bounds: self.output_visible().then_some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                lander_runtime: self.arcade_state,
                bomber_runtime: None,
                pod_runtime: None,
                swarmer_runtime: None,
                baiter_runtime: None,
                mutant_runtime: None,
                human_runtime: None,
                enemy_projectile_runtime: None,
            },
            commands,
            draws,
        }
    }
}

impl Lander {
    fn output_visible(&self) -> bool {
        self.spawn_visibility != LanderSpawnVisibility::HiddenFirstWaveRefill
    }

    fn update_seeking(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        match behavior.lander_mode {
            LanderBehaviorMode::SeekNearestHuman => {
                self.seek_nearest_human(prompt, behavior, commands);
            }
            LanderBehaviorMode::ChasePlayer => {
                if let Some(player) = prompt.player_position() {
                    self.position = step_toward(self.position, player, behavior.lander_seek_speed);
                } else {
                    self.drift(behavior);
                }
            }
            LanderBehaviorMode::Drift => {
                if !self.advance_arcade_fixed_point_motion() {
                    self.drift(behavior);
                }
            }
        }
    }

    fn seek_nearest_human(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        let target = self
            .target_human(prompt)
            .or_else(|| prompt.nearest_human(self.position));

        if let Some(target) = target {
            if pickup_distance(self.position, target.position, behavior) {
                self.mode = LanderMode::Carrying {
                    human_id: target.id,
                    pull_sound_sent: false,
                };
                commands.push(GameCommand::AttachHuman {
                    lander: self.id,
                    human: target.id,
                    position: target.position,
                });
                commands.push(GameCommand::PlaySound(SoundCue::LanderPickup));
                return;
            }
            if self.advance_arcade_fixed_point_motion() {
                return;
            }
            self.position = step_toward(self.position, target.position, behavior.lander_seek_speed);
            return;
        }

        if let Some(player) = prompt.player_position() {
            self.drift = if player.x < self.position.x { -1 } else { 1 };
        }
        if !self.advance_arcade_fixed_point_motion() {
            self.drift(behavior);
        }
    }

    fn target_human<'a>(&self, prompt: &'a StepPrompt) -> Option<&'a ActorSnapshot> {
        self.arcade_state
            .and_then(|arcade_state| arcade_state.target_human_index)
            .and_then(|target_slot_index| prompt.target_human(target_slot_index))
    }

    fn drift(&mut self, behavior: ActorBehaviorProfile) {
        self.position = self.position.offset(Velocity::new(
            self.drift * behavior.lander_drift_speed.max(0),
            0,
        ));
    }

    fn advance_arcade_fixed_point_motion(&mut self) -> bool {
        let Some(arcade_state) = &mut self.arcade_state else {
            return false;
        };
        if arcade_state.x_velocity == 0 && arcade_state.y_velocity == 0 {
            return false;
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
        self.drift = lander_drift_from_arcade_velocity(arcade_state.x_velocity);
        true
    }

    fn update_carrying(
        &mut self,
        prompt: &StepPrompt,
        human_id: ActorId,
        pull_sound_sent: bool,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        self.position = self
            .position
            .offset(Velocity::new(0, -behavior.lander_carry_speed));
        if !pull_sound_sent {
            self.mode = LanderMode::Carrying {
                human_id,
                pull_sound_sent: true,
            };
            commands.push(GameCommand::PlaySound(SoundCue::HumanPulled));
        }
        if self.position.y <= behavior.lander_conversion_y {
            commands.push(GameCommand::Destroy(self.id));
            commands.push(GameCommand::Destroy(human_id));
            commands.push(GameCommand::Spawn(SpawnRequest::Mutant {
                position: self.position,
                source: self.mutant_arcade_conversion(prompt),
            }));
            commands.push(GameCommand::PlaySound(SoundCue::MutantSpawn));
        }
    }

    fn mutant_arcade_conversion(&self, prompt: &StepPrompt) -> Option<MutantArcadeState> {
        let arcade_state = self.arcade_state?;
        let hop_rng = prompt.arcade_rng?;
        Some(MutantArcadeState::from_lander_conversion(
            arcade_state,
            prompt.arcade_wave,
            hop_rng,
        ))
    }

    fn tick_arcade_sleep(&mut self) -> bool {
        if let Some(arcade_state) = &mut self.arcade_state
            && arcade_state.sleep_ticks > 0
        {
            arcade_state.sleep_ticks = arcade_state.sleep_ticks.saturating_sub(1);
            return true;
        }
        false
    }

    fn tick_fire_timer(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        let mut arcade_shot_fired = false;
        if let Some(arcade_state) = &mut self.arcade_state {
            if arcade_state.shot_timer > 0 {
                arcade_state.shot_timer = arcade_state.shot_timer.saturating_sub(1);
            }
            if arcade_state.shot_timer == 0 {
                arcade_state.shot_timer = clamped_lander_fire_timer_reset(behavior);
                arcade_shot_fired = true;
            }
        }
        if arcade_shot_fired {
            self.fire_lander_shot(prompt, behavior, commands);
            return;
        }
        if self.arcade_state.is_some() {
            return;
        }

        let fire_period = behavior.lander_fire_period_steps.max(1);
        if prompt.step % fire_period == self.id.value() % fire_period {
            self.fire_lander_shot(prompt, behavior, commands);
        }
    }

    fn fire_lander_shot(
        &self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        let velocity = self.lander_shot_velocity(prompt, behavior);
        let projectile_arcade_state = self.arcade_state.map(|arcade_state| {
            arcade_enemy_projectile_state(
                arcade_state.x_fraction,
                arcade_state.y_fraction,
                velocity,
                behavior.lander_shot_lifetime_steps,
            )
        });
        commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
            position: self.position,
            velocity,
            source: projectile_arcade_state,
        }));
        commands.push(GameCommand::PlaySound(SoundCue::LanderShot));
    }

    fn lander_shot_velocity(
        &self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> Velocity {
        let speed = behavior.lander_shot_speed.max(1);
        if let Some(player) = prompt.player_position() {
            return Velocity::new(
                axis_step(player.x - self.position.x, speed),
                axis_step(player.y - self.position.y, speed),
            );
        }

        Velocity::new(self.drift.signum() * speed, 0)
    }

    fn draw_effect(&self) -> VisualEffect {
        self.arcade_state
            .map(|arcade_state| VisualEffect::LanderSpriteFrame {
                frame: arcade_state.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

const fn lander_drift_from_arcade_velocity(x_velocity: u16) -> i16 {
    arcade_drift_from_velocity(x_velocity)
}

fn lander_spawn_visibility(arcade_state: Option<LanderArcadeState>) -> LanderSpawnVisibility {
    let Some(arcade_state) = arcade_state else {
        return LanderSpawnVisibility::Normal;
    };
    first_wave_refill_lander_spawn_visibility(arcade_state)
        .unwrap_or(LanderSpawnVisibility::Normal)
}

fn lander_spawn_is_visible(arcade_state: LanderArcadeState) -> bool {
    first_wave_refill_lander_spawn_visibility(arcade_state)
        == Some(LanderSpawnVisibility::VisibleFirstWaveRefill)
}

fn first_wave_refill_lander_spawn_visibility(
    arcade_state: LanderArcadeState,
) -> Option<LanderSpawnVisibility> {
    ACTOR_FIRST_WAVE_REFILL_LANDER_SPAWNS
        .iter()
        .copied()
        .filter_map(|spawn| spawn.source)
        .enumerate()
        .find_map(|(index, refill_arcade_state)| {
            lander_arcade_state_matches_refill_row(arcade_state, refill_arcade_state).then_some(
                if index == 2 {
                    LanderSpawnVisibility::VisibleFirstWaveRefill
                } else {
                    LanderSpawnVisibility::HiddenFirstWaveRefill
                },
            )
        })
}

fn lander_arcade_state_matches_refill_row(
    arcade_state: LanderArcadeState,
    refill_arcade_state: LanderArcadeState,
) -> bool {
    arcade_state.x_fraction == refill_arcade_state.x_fraction
        && arcade_state.y_fraction == refill_arcade_state.y_fraction
        && arcade_state.x_velocity == refill_arcade_state.x_velocity
        && arcade_state.y_velocity == refill_arcade_state.y_velocity
        && arcade_state.shot_timer == refill_arcade_state.shot_timer
        && arcade_state.sleep_ticks == refill_arcade_state.sleep_ticks
        && arcade_state.picture_frame == refill_arcade_state.picture_frame
        && arcade_state.target_human_index == refill_arcade_state.target_human_index
}

const fn arcade_drift_from_velocity(x_velocity: u16) -> i16 {
    if x_velocity & 0x8000 != 0 {
        -1
    } else if x_velocity == 0 {
        0
    } else {
        1
    }
}

fn drift_direction(drift: i16) -> Direction {
    if drift < 0 {
        Direction::Left
    } else {
        Direction::Right
    }
}

fn clamped_lander_fire_timer_reset(behavior: ActorBehaviorProfile) -> u8 {
    let clamped = behavior
        .lander_fire_period_steps
        .max(1)
        .min(u64::from(u8::MAX));
    u8::try_from(clamped).unwrap_or(u8::MAX)
}

fn arcade_axis_step(position: i16, fraction: u8, velocity: u16) -> (i16, u8) {
    let [position, fraction] = u16::from_be_bytes([position as u8, fraction])
        .wrapping_add(velocity)
        .to_be_bytes();
    (i16::from(position), fraction)
}

fn arcade_active_object_y_step(position: i16, fraction: u8, velocity: u16) -> (i16, u8) {
    let [mut position, fraction] = u16::from_be_bytes([position as u8, fraction])
        .wrapping_add(velocity)
        .to_be_bytes();
    if position < PLAYFIELD_TOP_EDGE_Y {
        position = PLAYFIELD_BOTTOM_EDGE_Y;
    } else if position > PLAYFIELD_BOTTOM_EDGE_Y {
        position = PLAYFIELD_TOP_EDGE_Y;
    }
    (i16::from(position), fraction)
}

fn pickup_distance(lander: Point, human: Point, behavior: ActorBehaviorProfile) -> bool {
    (lander.x - human.x).abs() <= behavior.lander_pickup_radius_x
        && (lander.y - human.y).abs() <= behavior.lander_pickup_radius_y
}

fn step_toward(position: Point, target: Point, speed: i16) -> Point {
    Point::new(
        position.x + axis_step(target.x - position.x, speed),
        position.y + axis_step(target.y - position.y, speed),
    )
}

fn axis_step(delta: i16, speed: i16) -> i16 {
    let speed = speed.max(0);
    if delta == 0 {
        0
    } else if delta > 0 {
        delta.min(speed)
    } else {
        delta.max(-speed)
    }
}

fn move_by_hostile_mode(
    position: Point,
    mode: HostileMovementMode,
    prompt: &StepPrompt,
    speed: i16,
    drift: i16,
) -> Option<Point> {
    match mode {
        HostileMovementMode::Drift => Some(position.offset(Velocity::new(drift * speed.max(0), 0))),
        HostileMovementMode::ChasePlayer => prompt
            .player_position()
            .map(|player| step_toward(position, player, speed)),
    }
}

fn observed_velocity(previous: Point, current: Point) -> Velocity {
    Velocity::new(current.x - previous.x, current.y - previous.y)
}

fn direction_for_velocity(velocity: Velocity, fallback: Direction) -> Direction {
    if velocity.dx < 0 {
        Direction::Left
    } else if velocity.dx > 0 {
        Direction::Right
    } else {
        fallback
    }
}

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
                .source
                .map(|arcade_state| arcade_drift_from_velocity(arcade_state.x_velocity))
                .unwrap_or(-1),
            arcade_state: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.collision_position(), 14, 12)
    }

    fn scene_position(&self) -> Point {
        target6_mutant_arcade_scene_position(self.position, self.arcade_state)
    }

    fn collision_position(&self) -> Point {
        target6_mutant_arcade_collision_position(self.position, self.arcade_state)
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
                target6_mutant_arcade_fire2524_forced_shot(
                    self.position,
                    *arcade_state,
                    prompt,
                    behavior,
                )
            {
                arcade_state.target6_first_shot_deferred = true;
                arcade_state.shot_timer = TARGET6_MUTANT_FIRE2524_PENDING_SHOT_TIMER;
                push_arcade_enemy_projectile_command(
                    position,
                    velocity,
                    projectile_arcade_state,
                    SoundCue::MutantShot,
                    commands,
                );
            }
            if let Some(player_position) = prompt.player_position()
                && target6_mutant_arcade_fires_visible_entry_shot(
                    self.position,
                    *arcade_state,
                    player_position,
                )
            {
                arcade_state.target6_first_shot_deferred = true;
                let shot_rng = mutant_arcade_shot_rng(prompt, self.id, self.position);
                let shot_position =
                    target6_mutant_arcade_shot_position(self.position, *arcade_state);
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
            let target6_forced_dive_shot =
                target6_mutant_arcade_fires_dive_shot(self.position, *arcade_state);
            let target6_forced_dive_shot_position = self.position;
            let mut hop_rng = arcade_rng_from_snapshot(arcade_state.hop_rng);
            let hop_state = hop_rng.advance();
            arcade_state.hop_rng = hop_state.snapshot();
            self.position.y =
                mutant_arcade_hop_y(self.position.y, profile.mutant_random_y, hop_state.seed);

            if target6_forced_dive_shot {
                let shot_rng = mutant_arcade_shot_rng(prompt, self.id, self.position);
                let shot_position = target6_mutant_arcade_shot_position(
                    target6_forced_dive_shot_position,
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
                arcade_state.shot_timer = TARGET6_MUTANT_POST_SHOT_TIMER;
            } else {
                arcade_state.shot_timer = arcade_state.shot_timer.wrapping_sub(1);
                if arcade_state.shot_timer == 0 {
                    if target6_mutant_arcade_suppresses_fire2524_regular_shot(
                        self.position,
                        *arcade_state,
                    ) {
                        arcade_state.shot_timer = TARGET6_MUTANT_FIRE2524_PENDING_SHOT_TIMER;
                    } else if target6_mutant_arcade_defers_first_shot(self.position, *arcade_state)
                    {
                        arcade_state.target6_first_shot_deferred = true;
                        arcade_state.shot_timer = TARGET6_MUTANT_DEFERRED_SHOT_TIMER;
                        next_sleep_ticks = 0;
                    } else {
                        let shot_rng = mutant_arcade_shot_rng(prompt, self.id, self.position);
                        let default_reset = mutant_arcade_shot_reset(profile, shot_rng.seed);
                        let shot_position =
                            target6_mutant_arcade_shot_position(self.position, *arcade_state);
                        let fired = push_mutant_arcade_shot(
                            shot_position,
                            prompt,
                            behavior,
                            *arcade_state,
                            shot_rng,
                            commands,
                        );
                        arcade_state.shot_timer =
                            target6_mutant_arcade_post_shot_timer(*arcade_state, fired)
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

fn target6_mutant_arcade_conversion_x_correction(
    lander_runtime: LanderArcadeState,
) -> Option<u16> {
    (lander_runtime.target_human_index == Some(6) && lander_runtime.x_velocity == 0)
        .then_some(TARGET6_MUTANT_CONVERSION_X_CORRECTION)
}

fn target6_mutant_arcade_has_conversion_correction(
    arcade_state: MutantArcadeState,
) -> bool {
    arcade_state.render_x_correction == TARGET6_MUTANT_CONVERSION_X_CORRECTION
}

fn target6_mutant_arcade_uses_dive_projection(arcade_state: MutantArcadeState) -> bool {
    target6_mutant_arcade_has_conversion_correction(arcade_state)
        && arcade_state.y_velocity == 0x0090
}

fn target6_mutant_arcade_defers_first_shot(
    position: Point,
    arcade_state: MutantArcadeState,
) -> bool {
    target6_mutant_arcade_has_conversion_correction(arcade_state)
        && !arcade_state.target6_first_shot_deferred
        && position.x <= 0x04
        && position.y <= 0x60
}

fn target6_mutant_arcade_fires_visible_entry_shot(
    position: Point,
    arcade_state: MutantArcadeState,
    player_position: Point,
) -> bool {
    target6_mutant_arcade_has_conversion_correction(arcade_state)
        && !arcade_state.target6_first_shot_deferred
        && arcade_state.shot_timer == TARGET6_MUTANT_DEFERRED_SHOT_TIMER
        && arcade_state.sleep_ticks == MUTANT_LOOP_SLEEP_TICKS
        && position.x <= 0x04
        && position.y <= 0x60
        && player_position.y <= FIRST_WAVE_RESCUE_AIM_PLAYER_MIN_Y
}

fn target6_mutant_arcade_suppresses_fire2524_regular_shot(
    position: Point,
    arcade_state: MutantArcadeState,
) -> bool {
    if !target6_mutant_arcade_uses_dive_projection(arcade_state) {
        return false;
    }

    let (_, raw_y16) =
        arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction);
    (0x4000..=0x4FFF).contains(&raw_y16) || (0x9000..=0x9FFF).contains(&raw_y16)
}

fn target6_mutant_arcade_fires_dive_shot(
    position: Point,
    arcade_state: MutantArcadeState,
) -> bool {
    if !target6_mutant_arcade_uses_dive_projection(arcade_state)
        || !arcade_state.target6_first_shot_deferred
        || arcade_state.sleep_ticks != 0
    {
        return false;
    }

    matches!(
        arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction),
        TARGET6_MUTANT_DIVE_FIRST_SHOT_RAW | TARGET6_MUTANT_DIVE_SECOND_SHOT_RAW
    )
}

fn target6_mutant_arcade_post_shot_timer(
    arcade_state: MutantArcadeState,
    fired: bool,
) -> Option<u8> {
    (fired && target6_mutant_arcade_has_conversion_correction(arcade_state))
        .then_some(TARGET6_MUTANT_POST_SHOT_TIMER)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Target6MutantProjectionAnchor {
    raw_x16: u16,
    raw_y16: u16,
    screen: Point,
}

const ACTOR_TARGET6_MUTANT_DIVE_PROJECTIONS: &[Target6MutantProjectionAnchor] = &[
    // original: ACTOR_SOURCE_TARGET6_MUTANT_DIVE_PROJECTIONS
    Target6MutantProjectionAnchor {
        raw_x16: 0x031C,
        raw_y16: 0x3360,
        screen: Point::new(0x12, 0x43),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x037C,
        raw_y16: 0x3380,
        screen: Point::new(0x13, 0x46),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x034C,
        raw_y16: 0x33F0,
        screen: Point::new(0x12, 0x43),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x03AC,
        raw_y16: 0x3410,
        screen: Point::new(0x14, 0x46),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x037C,
        raw_y16: 0x3480,
        screen: Point::new(0x13, 0x44),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x085C,
        raw_y16: 0x47A0,
        screen: Point::new(0x1F, 0x5B),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x085C,
        raw_y16: 0x6120,
        screen: Point::new(0x1F, 0x71),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x088C,
        raw_y16: 0x61B0,
        screen: Point::new(0x1E, 0x71),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x085C,
        raw_y16: 0x6140,
        screen: Point::new(0x1F, 0x71),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x082C,
        raw_y16: 0x7770,
        screen: Point::new(0x20, 0x87),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x07FC,
        raw_y16: 0x7800,
        screen: Point::new(0x21, 0x88),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x082C,
        raw_y16: 0x7990,
        screen: Point::new(0x20, 0x87),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x082C,
        raw_y16: 0x81E0,
        screen: Point::new(0x20, 0x90),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x082C,
        raw_y16: 0x9730,
        screen: Point::new(0x21, 0x9F),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x07FC,
        raw_y16: 0x97A0,
        screen: Point::new(0x20, 0x9E),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x085C,
        raw_y16: 0x97C0,
        screen: Point::new(0x20, 0xA0),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x088C,
        raw_y16: 0x9850,
        screen: Point::new(0x1F, 0xA0),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x085C,
        raw_y16: 0x99E0,
        screen: Point::new(0x1E, 0xA2),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x082C,
        raw_y16: 0x9A70,
        screen: Point::new(0x20, 0xA3),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x088C,
        raw_y16: 0xA200,
        screen: Point::new(0x20, 0xA2),
    },
    Target6MutantProjectionAnchor {
        raw_x16: 0x08EC,
        raw_y16: 0xA320,
        screen: Point::new(0x20, 0xA2),
    },
];

const ACTOR_TARGET6_MUTANT_VISUAL_ROWS: &[(u16, i16)] = &[
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

fn target6_mutant_arcade_dive_position(
    position: Point,
    arcade_state: MutantArcadeState,
) -> Option<Point> {
    if !target6_mutant_arcade_uses_dive_projection(arcade_state) {
        return None;
    }

    let (raw_x16, raw_y16) =
        arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction);
    if let Some(anchor) = ACTOR_TARGET6_MUTANT_DIVE_PROJECTIONS
        .iter()
        .find(|anchor| anchor.raw_x16 == raw_x16 && anchor.raw_y16 == raw_y16)
    {
        return Some(anchor.screen);
    }

    target6_mutant_arcade_interpolated_dive_position(raw_y16)
}

fn target6_mutant_arcade_interpolated_dive_position(raw_y16: u16) -> Option<Point> {
    let first = ACTOR_TARGET6_MUTANT_DIVE_PROJECTIONS.first()?;
    let last = ACTOR_TARGET6_MUTANT_DIVE_PROJECTIONS.last()?;
    if raw_y16 < first.raw_y16 || raw_y16 > last.raw_y16 {
        return None;
    }

    ACTOR_TARGET6_MUTANT_DIVE_PROJECTIONS
        .windows(2)
        .find_map(|anchors| {
            let start = anchors[0];
            let end = anchors[1];
            if raw_y16 < start.raw_y16 || raw_y16 > end.raw_y16 || start.raw_y16 >= end.raw_y16 {
                return None;
            }

            Some(Point::new(
                arcade_lerp_i16(
                    start.screen.x,
                    end.screen.x,
                    raw_y16,
                    start.raw_y16,
                    end.raw_y16,
                ),
                arcade_lerp_i16(
                    start.screen.y,
                    end.screen.y,
                    raw_y16,
                    start.raw_y16,
                    end.raw_y16,
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

fn target6_mutant_arcade_visual_position(
    position: Point,
    arcade_state: MutantArcadeState,
) -> Option<Point> {
    if !target6_mutant_arcade_has_conversion_correction(arcade_state)
        || arcade_state.x_velocity != 0x0030
    {
        return None;
    }

    let x16 = arcade_absolute_x(position, arcade_state.x_fraction)
        .wrapping_add(TARGET6_MUTANT_VISUAL_X_CORRECTION);
    if (x16 as i16) < 0 {
        return None;
    }
    let screen_x = x16 >> OBJECT_WORLD_TO_SCREEN_SHIFT;
    if screen_x >= OBJECT_VISIBLE_SCREEN_WIDTH {
        return None;
    }
    let screen_y = ACTOR_TARGET6_MUTANT_VISUAL_ROWS
        .iter()
        .find_map(|(row_x16, screen_y)| (*row_x16 == x16).then_some(*screen_y))?;
    Some(Point::new(screen_x as i16, screen_y))
}

fn target6_mutant_arcade_scene_position(
    position: Point,
    arcade_state: Option<MutantArcadeState>,
) -> Point {
    let Some(arcade_state) = arcade_state else {
        return position;
    };
    target6_mutant_arcade_dive_position(position, arcade_state)
        .or_else(|| target6_mutant_arcade_visual_position(position, arcade_state))
        .unwrap_or(position)
}

fn target6_mutant_arcade_collision_position(
    position: Point,
    arcade_state: Option<MutantArcadeState>,
) -> Point {
    let Some(arcade_state) = arcade_state else {
        return position;
    };
    if let Some(position) = target6_mutant_arcade_dive_position(position, arcade_state) {
        return position.offset(Velocity::new(0, 1));
    }
    target6_mutant_arcade_visual_position(position, arcade_state).unwrap_or(position)
}

fn target6_mutant_arcade_waits_for_fire2524_collision(
    position: Point,
    arcade_state: Option<MutantArcadeState>,
) -> bool {
    let Some(arcade_state) = arcade_state else {
        return false;
    };
    if !target6_mutant_arcade_uses_dive_projection(arcade_state) {
        return false;
    }

    let (_, raw_y16) =
        arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction);
    arcade_state.shot_timer >= 0x80
        && (0x9000..TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MIN).contains(&raw_y16)
}

fn target6_mutant_arcade_uses_fire2524_collision_projection(
    position: Point,
    arcade_state: Option<MutantArcadeState>,
) -> bool {
    let Some(arcade_state) = arcade_state else {
        return false;
    };
    if !target6_mutant_arcade_uses_dive_projection(arcade_state) {
        return false;
    }

    let (_, raw_y16) =
        arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction);
    arcade_state.shot_timer >= 0x80
        && (TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MIN
            ..TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MAX)
            .contains(&raw_y16)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorExplosionPlacement {
    position: Point,
    explosion_anchor: Option<Point>,
}

fn actor_player_enemy_collision_explosion_placement(
    enemy: &CollisionBody,
) -> ActorExplosionPlacement {
    if target6_mutant_arcade_uses_fire2524_collision_projection(
        enemy.position,
        enemy.mutant_runtime,
    ) {
        ActorExplosionPlacement {
            position: TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_TOP_LEFT,
            explosion_anchor: Some(TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_CENTER),
        }
    } else {
        ActorExplosionPlacement {
            position: center_of(enemy.bounds),
            explosion_anchor: None,
        }
    }
}

fn target6_mutant_arcade_shot_position(
    position: Point,
    arcade_state: MutantArcadeState,
) -> Point {
    if !target6_mutant_arcade_uses_dive_projection(arcade_state) {
        return position;
    }

    match arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction) {
        TARGET6_MUTANT_DIVE_ENTRY_RAW => Point::new(0x13, 0x46),
        TARGET6_MUTANT_DIVE_FIRST_SHOT_RAW => Point::new(0x1E, 0x70),
        TARGET6_MUTANT_DIVE_SECOND_SHOT_RAW => Point::new(0x21, 0x87),
        _ => target6_mutant_arcade_dive_position(position, arcade_state).unwrap_or(position),
    }
}

fn actor_enemy_projectile_count(prompt: &StepPrompt) -> usize {
    prompt
        .snapshots
        .iter()
        .filter(|snapshot| is_enemy_projectile_kind(snapshot.kind))
        .count()
}

fn actor_bomb_projectile_count(prompt: &StepPrompt) -> usize {
    prompt
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.kind == ActorKind::Bomb)
        .count()
}

fn bomber_bomb_lifetime_ticks(arcade_rng: ActorArcadeRngSnapshot) -> u8 {
    (arcade_rng.seed & 0x1F).wrapping_add(1)
}

fn arcade_tie_selected_slot(seed: u8) -> u8 {
    (seed & 0x06) >> 1
}

fn target6_mutant_arcade_fire2524_forced_shot(
    position: Point,
    arcade_state: MutantArcadeState,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
) -> Option<(Point, Velocity, EnemyProjectileArcadeState)> {
    if !target6_mutant_arcade_uses_dive_projection(arcade_state)
        || actor_enemy_projectile_count(prompt) >= ENEMY_PROJECTILE_SLOT_LIMIT
    {
        return None;
    }

    match arcade_world_position(position, arcade_state.x_fraction, arcade_state.y_fraction) {
        TARGET6_MUTANT_FIRE2524_FIRST_SHOT_RAW
            if arcade_state.sleep_ticks == MUTANT_LOOP_SLEEP_TICKS =>
        {
            Some(target6_mutant_arcade_exact_projectile(
                Point::new(0x1E, 0x54),
                0x33,
                0x56,
                0xFFE0,
                0x0138,
                behavior,
            ))
        }
        TARGET6_MUTANT_FIRE2524_SECOND_SHOT_RAW
            if arcade_state.sleep_ticks == MUTANT_LOOP_SLEEP_TICKS =>
        {
            Some(target6_mutant_arcade_exact_projectile(
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

fn target6_mutant_arcade_exact_projectile(
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

fn push_arcade_enemy_projectile_command(
    position: Point,
    velocity: Velocity,
    projectile_arcade_state: EnemyProjectileArcadeState,
    sound: SoundCue,
    commands: &mut Vec<GameCommand>,
) {
    commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
        position,
        velocity,
        source: Some(projectile_arcade_state),
    }));
    commands.push(GameCommand::PlaySound(sound));
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

fn arcade_enemy_fireball(
    position: Point,
    x_fraction: u8,
    y_fraction: u8,
    prompt: &StepPrompt,
    shot_rng: ActorArcadeRngSnapshot,
    lifetime_ticks: u8,
) -> Option<(Velocity, EnemyProjectileArcadeState)> {
    if !enemy_projectile_spawn_in_bounds(position)
        || actor_enemy_projectile_count(prompt) >= ENEMY_PROJECTILE_SLOT_LIMIT
    {
        return None;
    }
    let player_position = prompt.player_position()?;
    let player_velocity = prompt.player_velocity().unwrap_or_default();
    let x_delta = (shot_rng.seed & 0x1F)
        .wrapping_sub(0x10)
        .wrapping_add(player_position.x as u8)
        .wrapping_sub(position.x as u8);
    let mut x_velocity = actor_sign_extend_u8_to_u16(x_delta).wrapping_shl(2);
    if shot_rng.seed > 120 {
        x_velocity =
            x_velocity.wrapping_add(arcade_velocity_word(player_velocity.dx).wrapping_shl(2));
    }
    let y_delta = (shot_rng.lseed & 0x1F)
        .wrapping_sub(0x10)
        .wrapping_add(player_position.y as u8)
        .wrapping_sub(position.y as u8);
    let y_velocity = actor_sign_extend_u8_to_u16(y_delta).wrapping_shl(2);
    let velocity = arcade_screen_velocity(x_velocity, y_velocity);
    Some((
        velocity,
        EnemyProjectileArcadeState {
            x_fraction,
            y_fraction,
            x_velocity,
            y_velocity,
            lifetime_ticks,
        },
    ))
}

fn arcade_screen_velocity(x_velocity: u16, y_velocity: u16) -> Velocity {
    Velocity::new(
        arcade_screen_velocity_component(x_velocity),
        arcade_screen_velocity_component(y_velocity),
    )
}

fn arcade_screen_velocity_component(velocity: u16) -> i16 {
    let signed = velocity as i16;
    if signed == 0 {
        return 0;
    }

    let pixels = signed / 256;
    if pixels == 0 {
        if signed > 0 { 1 } else { -1 }
    } else {
        pixels
    }
}
