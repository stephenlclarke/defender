#[derive(Debug)]
struct Bomber {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourceBomberMetadata>,
}

impl Bomber {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorBomberSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorBomberSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .source
                .map(|source| actor_source_drift_from_velocity(source.x_velocity))
                .unwrap_or(-1),
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 8, 8)
    }

    fn advance_source_motion(&mut self) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };

        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) = actor_source_active_object_y_step(
            self.position.y,
            source.y_fraction,
            source.y_velocity,
        );
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        self.drift = actor_source_drift_from_velocity(source.x_velocity);
        true
    }

    fn advance_source_tie_step(&mut self, prompt: &StepPrompt, arcade_rng: ActorSourceRngSnapshot) {
        let Some(source) = &mut self.source else {
            return;
        };
        if source.slot != actor_source_tie_selected_slot(arcade_rng.seed) {
            return;
        }
        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return;
        }

        source.picture_frame =
            actor_source_bomber_picture_frame(arcade_rng.seed, source.picture_frame);
        source.y_velocity =
            actor_source_bomber_random_y_velocity(source.y_velocity, arcade_rng.seed);
        if self.position.y == 0 {
            source.y_velocity = actor_source_bomber_cruise_y_velocity(
                source.y_velocity,
                &mut source.cruise_altitude,
                self.position.y,
                arcade_rng.seed,
            );
        } else if let Some(player) = prompt.player_position()
            && let Some(delta) =
                actor_source_bomber_onscreen_y_velocity_delta(self.position.y, player.y)
        {
            source.y_velocity = source.y_velocity.wrapping_add(delta);
        }

        source.sleep_ticks = BOMBER_LOOP_SLEEP_TICKS;
    }

    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceBomberFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }

    fn maybe_spawn_bomb(
        &self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        if let Some(source) = self.source {
            let Some(arcade_rng) = prompt.arcade_rng else {
                return;
            };
            if source.slot != actor_source_tie_selected_slot(arcade_rng.seed)
                || source.sleep_ticks > 0
                || self.position.y == 0
                || arcade_rng.lseed & 0x07 != 0
                || actor_source_bomb_shell_count(prompt) >= ACTIVE_BOMBER_BOMB_LIMIT
                || actor_source_shell_count(prompt) >= ENEMY_PROJECTILE_SLOT_LIMIT
                || !source_shell_spawn_in_bounds(self.position)
            {
                return;
            }

            commands.push(GameCommand::Spawn(SpawnRequest::Bomb {
                position: self.position,
                source: Some(ActorSourceEnemyProjectileMetadata {
                    x_fraction: source.x_fraction,
                    y_fraction: source.y_fraction,
                    x_velocity: 0,
                    y_velocity: 0,
                    lifetime_ticks: actor_source_bomber_bomb_lifetime_ticks(arcade_rng),
                }),
            }));
            return;
        }

        let active_bombs = prompt
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomb && snapshot.alive)
            .count();
        if active_bombs >= ACTIVE_BOMBER_BOMB_LIMIT {
            return;
        }

        let bomb_period = behavior.bomber_bomb_period_steps.max(1);
        let phase = self.id.value();
        if prompt.step % bomb_period == phase % bomb_period {
            commands.push(GameCommand::Spawn(SpawnRequest::Bomb {
                position: self.position,
                source: None,
            }));
        }
    }
}

fn actor_source_bomber_picture_frame(seed: u8, current: u8) -> u8 {
    let step = (seed & 0x3F).wrapping_sub(0x20);
    if step & 0x80 != 0 {
        current
            .saturating_add(1)
            .min(BOMBER_PICTURE_FRAME_COUNT - 1)
    } else {
        current.saturating_sub(1)
    }
}

fn actor_source_bomber_random_y_velocity(previous: u16, seed: u8) -> u16 {
    let random_delta = actor_sign_extend_u8_to_u16((seed & 0x3F).wrapping_sub(0x20));
    let mut velocity = previous.wrapping_add(random_delta);
    let damping_byte = 0u8.wrapping_sub(velocity.wrapping_shl(3).to_be_bytes()[0]);
    velocity = velocity.wrapping_add(actor_sign_extend_u8_to_u16(damping_byte));
    velocity
}

fn actor_source_bomber_cruise_y_velocity(
    mut velocity: u16,
    cruise_altitude: &mut i16,
    object_y: i16,
    seed: u8,
) -> u16 {
    if seed <= 0x40 {
        let nudge = i16::from((seed & 0x03).wrapping_sub(2) as i8);
        *cruise_altitude = (*cruise_altitude + nudge)
            .clamp(BOMBER_MIN_CRUISE_ALTITUDE, BOMBER_MAX_CRUISE_ALTITUDE);
    }

    let distance = *cruise_altitude - object_y;
    if distance.abs() > BOMBER_CRUISE_WINDOW_HALF_PIXELS {
        let correction = if distance >= 0 { 0xFFF0 } else { 0x0010 };
        velocity = velocity.wrapping_add(correction);
    }
    velocity
}

fn actor_source_bomber_onscreen_y_velocity_delta(object_y: i16, player_y: i16) -> Option<u16> {
    let delta = object_y - player_y;
    if delta >= 0 {
        if delta >= 0x20 {
            Some(0xFFF0)
        } else if delta > 0x10 {
            None
        } else {
            Some(0x0010)
        }
    } else if delta <= -0x20 {
        Some(0x0010)
    } else if delta < -0x10 {
        None
    } else {
        Some(0xFFF0)
    }
}

impl AssetActor for Bomber {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Bomber);
            if self.source.is_some() {
                self.maybe_spawn_bomb(prompt, behavior, &mut commands);
                if let Some(arcade_rng) = prompt.arcade_rng {
                    self.advance_source_tie_step(prompt, arcade_rng);
                }
                self.advance_source_motion();
            } else if let Some(position) = move_by_hostile_mode(
                self.position,
                behavior.bomber_mode,
                prompt,
                behavior.bomber_drift_speed,
                self.drift,
            ) {
                self.position = position;
                self.maybe_spawn_bomb(prompt, behavior, &mut commands);
            }
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Bomber,
                self.position,
                self.draw_effect(),
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Bomber,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(
                    movement_velocity,
                    drift_direction(self.drift),
                )),
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                lander_runtime: None,
                bomber_runtime: self.source,
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

#[derive(Debug)]
struct Bomb {
    id: ActorId,
    position: Point,
    lifetime_steps: u16,
    source: ActorSourceEnemyProjectileMetadata,
}

impl Bomb {
    fn new(
        id: ActorId,
        position: Point,
        lifetime_steps: u16,
        source: Option<ActorSourceEnemyProjectileMetadata>,
    ) -> Self {
        let mut source = source.unwrap_or(ActorSourceEnemyProjectileMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            lifetime_ticks: 0,
        });
        let lifetime_steps = if source.lifetime_ticks == 0 {
            lifetime_steps
        } else {
            u16::from(source.lifetime_ticks)
        };
        source.lifetime_ticks = actor_source_projectile_lifetime_ticks(lifetime_steps);
        Self {
            id,
            position,
            lifetime_steps,
            source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 4, 6)
    }
}

impl AssetActor for Bomb {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut draws = Vec::new();
        if prompt.phase == Phase::Playing && self.lifetime_steps > 0 {
            if prompt.projectile_scan_tick {
                self.lifetime_steps = self.lifetime_steps.saturating_sub(1);
                self.source.lifetime_ticks =
                    actor_source_projectile_lifetime_ticks(self.lifetime_steps);
            }
            if self.lifetime_steps > 0 {
                draws.push(DrawCommand::sprite(self.id, SpriteKey::Bomb, self.position));
            }
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Bomb,
                position: self.position,
                velocity: Velocity::default(),
                direction: None,
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing && self.lifetime_steps > 0,
                lander_runtime: None,
                bomber_runtime: None,
                pod_runtime: None,
                swarmer_runtime: None,
                baiter_runtime: None,
                mutant_runtime: None,
                human_runtime: None,
                enemy_projectile_runtime: Some(self.source),
            },
            commands: Vec::new(),
            draws,
        }
    }
}

#[derive(Debug)]
struct Pod {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourcePodMetadata>,
}

impl Pod {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorPodSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorPodSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .source
                .map(|source| actor_source_drift_from_velocity(source.x_velocity))
                .unwrap_or(1),
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 8, 8)
    }

    fn advance_source_motion(&mut self) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };
        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) = actor_source_active_object_y_step(
            self.position.y,
            source.y_fraction,
            source.y_velocity,
        );
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        self.drift = actor_source_drift_from_velocity(source.x_velocity);
        true
    }
}

impl AssetActor for Pod {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Pod);
            if !self.advance_source_motion()
                && let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.pod_mode,
                    prompt,
                    behavior.pod_drift_speed,
                    self.drift,
                )
            {
                self.position = position;
            }
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Pod,
                self.position,
                VisualEffect::SourcePod,
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Pod,
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
                pod_runtime: self.source,
                swarmer_runtime: None,
                baiter_runtime: None,
                mutant_runtime: None,
                human_runtime: None,
                enemy_projectile_runtime: None,
            },
            commands: Vec::new(),
            draws,
        }
    }
}

#[derive(Debug)]
struct Swarmer {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourceSwarmerMetadata>,
}

impl Swarmer {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorSwarmerSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorSwarmerSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: -1,
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 6, 4)
    }

    fn advance_source_motion(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };
        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return true;
        }

        let Some(player) = prompt.player_position() else {
            return false;
        };
        let profile = prompt.source_wave;
        let mut horizontal_seek_only = false;
        if source.horizontal_seek_pending {
            source.x_velocity = source_mini_swarmer_seek_velocity(
                profile.swarmer_x_velocity,
                player.x,
                self.position.x,
            );
            source.horizontal_seek_pending = false;
            source.sleep_ticks = MINI_SWARMER_LOOP_SLEEP_TICKS;
            horizontal_seek_only = true;
        }

        let in_shot_window = if horizontal_seek_only {
            false
        } else {
            source.y_velocity = source_mini_swarmer_y_velocity(
                source.y_velocity,
                source.acceleration,
                player.y,
                self.position.y,
                prompt.arcade_rng.map(|rng| rng.seed).unwrap_or(0),
            );
            let player_absolute_x = actor_source_absolute_x(player, 0);
            let object_absolute_x = actor_source_absolute_x(self.position, source.x_fraction);
            let past_window = player_absolute_x
                .wrapping_sub(object_absolute_x)
                .wrapping_add(MINI_SWARMER_TURN_WINDOW_HALF);
            let in_shot_window = past_window <= MINI_SWARMER_TURN_WINDOW;
            if !in_shot_window {
                source.x_velocity = source_mini_swarmer_seek_velocity(
                    profile.swarmer_x_velocity,
                    player.x,
                    self.position.x,
                );
            }
            in_shot_window
        };

        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) = actor_source_active_object_y_step(
            self.position.y,
            source.y_fraction,
            source.y_velocity,
        );
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        if in_shot_window {
            source.shot_timer = source.shot_timer.wrapping_sub(1);
            if source.shot_timer == 0 {
                source.shot_timer = prompt
                    .arcade_rng
                    .map(|rng| source_rmax(clamped_source_swarmer_shot_reset(profile), rng.seed))
                    .unwrap_or_else(|| clamped_source_swarmer_shot_reset(profile));
                push_swarmer_shot(self.position, prompt, behavior, Some(*source), commands);
            }
        }
        source.sleep_ticks = MINI_SWARMER_LOOP_SLEEP_TICKS;
        true
    }
}

impl AssetActor for Swarmer {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Swarmer);
            if !self.advance_source_motion(prompt, behavior, &mut commands) {
                if let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.swarmer_mode,
                    prompt,
                    behavior.swarmer_seek_speed,
                    self.drift,
                ) {
                    self.position = position;
                }
                let fire_period = behavior.swarmer_fire_period_steps.max(1);
                let can_fire = behavior.swarmer_mode == HostileMovementMode::Drift
                    || prompt.player_position().is_some();
                if can_fire && prompt.step % fire_period == self.id.value() % fire_period {
                    push_swarmer_shot(self.position, prompt, behavior, None, &mut commands);
                }
            }
            draws.push(DrawCommand::sprite(
                self.id,
                SpriteKey::Swarmer,
                self.position,
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Swarmer,
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
                swarmer_runtime: self.source,
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

fn push_swarmer_shot(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    source: Option<ActorSourceSwarmerMetadata>,
    commands: &mut Vec<GameCommand>,
) {
    if let Some(source) = source {
        if let Some((velocity, projectile_source)) =
            actor_source_mini_swarmer_fireball(position, prompt, source)
        {
            push_source_enemy_projectile_command(
                position,
                velocity,
                projectile_source,
                SoundCue::SwarmerShot,
                commands,
            );
        }
        return;
    }

    let velocity = hostile_shot_velocity(position, prompt, behavior.swarmer_shot_speed);
    commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
        position,
        velocity,
        source: None,
    }));
    commands.push(GameCommand::PlaySound(SoundCue::SwarmerShot));
}

fn actor_source_mini_swarmer_fireball(
    position: Point,
    prompt: &StepPrompt,
    source: ActorSourceSwarmerMetadata,
) -> Option<(Velocity, ActorSourceEnemyProjectileMetadata)> {
    let player = prompt.player_position()?;
    let player_delta = actor_source_absolute_x(player, 0)
        .wrapping_sub(actor_source_absolute_x(position, source.x_fraction));
    if (player_delta.to_be_bytes()[0] ^ source.x_velocity.to_be_bytes()[0]) & 0x80 != 0
        || actor_source_shell_count(prompt) >= ENEMY_PROJECTILE_SLOT_LIMIT
    {
        return None;
    }

    let x_velocity = source.x_velocity.wrapping_shl(3);
    let y_velocity = actor_arithmetic_shift_right_word(
        u16::from_be_bytes([(player.y as u8).wrapping_sub(position.y as u8), 0]),
        5,
    );
    let velocity = actor_source_screen_velocity(x_velocity, y_velocity);
    Some((
        velocity,
        ActorSourceEnemyProjectileMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity,
            y_velocity,
            lifetime_ticks: ENEMY_PROJECTILE_LIFETIME_TICKS,
        },
    ))
}

fn clamped_source_swarmer_shot_reset(profile: ActorSourceWaveProfile) -> u8 {
    profile.swarmer_shot_time.max(1).min(u32::from(u8::MAX)) as u8
}

#[derive(Debug)]
struct Baiter {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourceBaiterMetadata>,
}

impl Baiter {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorBaiterSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorBaiterSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: -1,
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 12, 4)
    }

    fn advance_source_motion(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };

        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
        } else {
            source.shot_timer = source.shot_timer.wrapping_sub(1);
            if source.shot_timer == 0 {
                let profile = prompt.source_wave;
                let shot_rng = actor_source_baiter_shot_rng(prompt, self.id, self.position);
                source.shot_timer = actor_source_baiter_shot_reset(profile, shot_rng.seed);
                push_baiter_shot(
                    self.id,
                    self.position,
                    prompt,
                    behavior,
                    Some(*source),
                    Some(shot_rng),
                    commands,
                );
            }

            source.picture_frame = (source.picture_frame + 1) % BAITER_PICTURE_FRAME_COUNT;
            if source.picture_frame == 0
                && let Some(player) = prompt.player_position()
            {
                let profile = prompt.source_wave;
                let seed = prompt
                    .arcade_rng
                    .map(|arcade_rng| arcade_rng.seed)
                    .unwrap_or_else(|| actor_source_motion_seed(prompt.step, self.id));
                source_baiter_velocity_update(
                    source,
                    self.position,
                    profile,
                    player,
                    prompt.player_velocity().unwrap_or_default(),
                    true,
                    seed,
                );
            }
            source.sleep_ticks = BAITER_LOOP_SLEEP_TICKS;
        }

        let (x, x_fraction) = actor_source_axis_step(
            self.position.x,
            source.x_fraction,
            actor_source_baiter_screen_x_velocity(source.x_velocity),
        );
        let (y, y_fraction) = actor_source_active_object_y_step(
            self.position.y,
            source.y_fraction,
            source.y_velocity,
        );
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        true
    }

    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceBaiterFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

fn push_baiter_shot(
    actor: ActorId,
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    source: Option<ActorSourceBaiterMetadata>,
    shot_rng: Option<ActorSourceRngSnapshot>,
    commands: &mut Vec<GameCommand>,
) {
    if let Some(source) = source {
        let shot_rng =
            shot_rng.unwrap_or_else(|| actor_source_baiter_shot_rng(prompt, actor, position));
        if let Some((velocity, projectile_source)) =
            actor_source_baiter_fireball(position, prompt, source, shot_rng)
        {
            push_source_enemy_projectile_command(
                position,
                velocity,
                projectile_source,
                SoundCue::BaiterShot,
                commands,
            );
        }
        return;
    }

    let velocity = baiter_shot_velocity(position, prompt, behavior);
    commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
        position,
        velocity,
        source: None,
    }));
    commands.push(GameCommand::PlaySound(SoundCue::BaiterShot));
}

fn actor_source_baiter_shot_rng(
    prompt: &StepPrompt,
    actor: ActorId,
    position: Point,
) -> ActorSourceRngSnapshot {
    prompt.arcade_rng.unwrap_or(ActorSourceRngSnapshot {
        seed: actor_source_motion_seed(prompt.step, actor),
        hseed: position.x as u8,
        lseed: position.y as u8,
    })
}

fn actor_source_baiter_shot_reset(profile: ActorSourceWaveProfile, seed: u8) -> u8 {
    source_rmax(clamped_source_baiter_shot_reset(profile), seed)
}

fn actor_source_baiter_fireball(
    position: Point,
    prompt: &StepPrompt,
    source: ActorSourceBaiterMetadata,
    shot_rng: ActorSourceRngSnapshot,
) -> Option<(Velocity, ActorSourceEnemyProjectileMetadata)> {
    actor_source_enemy_fireball(
        position,
        source.x_fraction,
        source.y_fraction,
        prompt,
        shot_rng,
        ENEMY_PROJECTILE_LIFETIME_TICKS,
    )
}

fn baiter_shot_velocity(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
) -> Velocity {
    hostile_shot_velocity(position, prompt, behavior.baiter_shot_speed)
}

fn hostile_shot_velocity(position: Point, prompt: &StepPrompt, speed: i16) -> Velocity {
    let speed = speed.max(1);
    if let Some(player) = prompt.player_position() {
        return Velocity::new(
            axis_step(player.x - position.x, speed),
            axis_step(player.y - position.y, speed),
        );
    }

    Velocity::new(speed, 0)
}

impl AssetActor for Baiter {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Baiter);
            if !self.advance_source_motion(prompt, behavior, &mut commands) {
                if let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.baiter_mode,
                    prompt,
                    behavior.baiter_seek_speed,
                    self.drift,
                ) {
                    self.position = position;
                }
                let fire_period = behavior.baiter_fire_period_steps.max(1);
                let can_fire = behavior.baiter_mode == HostileMovementMode::Drift
                    || prompt.player_position().is_some();
                if can_fire && prompt.step % fire_period == self.id.value() % fire_period {
                    push_baiter_shot(
                        self.id,
                        self.position,
                        prompt,
                        behavior,
                        None,
                        None,
                        &mut commands,
                    );
                }
            }
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Baiter,
                self.position,
                self.draw_effect(),
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Baiter,
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
                baiter_runtime: self.source,
                mutant_runtime: None,
                human_runtime: None,
                enemy_projectile_runtime: None,
            },
            commands,
            draws,
        }
    }
}

fn clamped_source_baiter_shot_reset(profile: ActorSourceWaveProfile) -> u8 {
    profile.baiter_shot_time.max(1).min(u32::from(u8::MAX)) as u8
}

fn actor_source_baiter_screen_x_velocity(x_velocity_word: u16) -> u16 {
    x_velocity_word.wrapping_shl(2)
}

fn source_baiter_velocity_update(
    source: &mut ActorSourceBaiterMetadata,
    position: Point,
    profile: ActorSourceWaveProfile,
    player_position: Point,
    player_velocity: Velocity,
    honor_seek_probability: bool,
    seed: u8,
) -> bool {
    if honor_seek_probability && seed <= profile.baiter_seek_probability {
        return false;
    }

    let x_delta = position.x - player_position.x;
    if x_delta.abs() > BAITER_X_SEEK_WINDOW_HALF_PIXELS {
        let x_seek_byte = if x_delta > 0 {
            0u8.wrapping_sub(BAITER_X_SEEK_SPEED)
        } else {
            BAITER_X_SEEK_SPEED
        };
        let player_x_velocity =
            actor_arithmetic_shift_right_word(actor_source_velocity_word(player_velocity.dx), 2);
        source.x_velocity =
            actor_sign_extend_u8_to_u16(x_seek_byte).wrapping_add(player_x_velocity);
    }

    let y_delta = position.y - player_position.y;
    if y_delta.abs() > BAITER_Y_SEEK_WINDOW_HALF_PIXELS {
        let y_seek_byte = if y_delta > 0 {
            0u8.wrapping_sub(BAITER_Y_SEEK_BYTE)
        } else {
            BAITER_Y_SEEK_BYTE
        };
        source.y_velocity = actor_arithmetic_shift_right_word(
            u16::from_be_bytes([y_seek_byte, 0])
                .wrapping_add(actor_source_velocity_word(player_velocity.dy)),
            1,
        );
    }

    true
}

fn actor_arithmetic_shift_right_word(value: u16, shift: u8) -> u16 {
    ((value as i16) >> shift.min(15)) as u16
}

fn actor_source_velocity_word(value: i16) -> u16 {
    value as u16
}

fn actor_source_motion_seed(step: u64, id: ActorId) -> u8 {
    (step as u8).wrapping_mul(17).wrapping_add(id.value() as u8)
}

fn source_mini_swarmer_seek_velocity(x_velocity_word: u8, player_x: i16, swarmer_x: i16) -> u16 {
    if player_x >= swarmer_x {
        actor_sign_extend_u8_to_u16(x_velocity_word)
    } else {
        actor_sign_extend_u8_to_u16(0u8.wrapping_sub(x_velocity_word))
    }
}

fn source_mini_swarmer_y_velocity(
    previous_y_velocity: u16,
    acceleration: u8,
    player_y: i16,
    swarmer_y: i16,
    seed: u8,
) -> u16 {
    let acceleration_low = if player_y > swarmer_y {
        acceleration
    } else {
        0u8.wrapping_sub(acceleration)
    };
    let mut y_velocity =
        actor_sign_extend_u8_to_u16(acceleration_low).wrapping_add(previous_y_velocity);
    if (y_velocity as i16) >= (MINI_SWARMER_MAX_Y_VELOCITY as i16) {
        y_velocity = MINI_SWARMER_MAX_Y_VELOCITY;
    }
    if (y_velocity as i16) <= (MINI_SWARMER_MIN_Y_VELOCITY as i16) {
        y_velocity = MINI_SWARMER_MIN_Y_VELOCITY;
    }
    y_velocity = y_velocity.wrapping_add(source_mini_swarmer_damping_adjustment(y_velocity));
    y_velocity.wrapping_add(actor_sign_extend_u8_to_u16(
        (seed & 0x1F).wrapping_sub(0x10),
    ))
}

fn source_mini_swarmer_damping_adjustment(value: u16) -> u16 {
    let [mut a, mut b] = value.to_be_bytes();
    a = !a;
    b = !b;
    for _ in 0..2 {
        let carry = b & 0x80 != 0;
        b = b.wrapping_shl(1);
        a = a.wrapping_shl(1) | u8::from(carry);
    }
    actor_sign_extend_u8_to_u16(a)
}
