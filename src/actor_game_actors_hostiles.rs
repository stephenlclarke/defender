const BOMBER_BOMB_RANDOM_DROP_MASK: u8 = 0x07;
const BOMBER_RANDOM_STEP_MASK: u8 = 0x3F;
const BOMBER_RANDOM_STEP_CENTER: u8 = 0x20;
const MOTION_BYTE_SIGN_BIT: u8 = 0x80;
const BOMBER_CRUISE_RANDOM_NUDGE_SEED_MAX: u8 = 0x40;
const BOMBER_CRUISE_NUDGE_MASK: u8 = 0x03;
const BOMBER_CRUISE_NUDGE_CENTER: u8 = 2;
const BOMBER_Y_VELOCITY_UP_CORRECTION: u16 = 0xFFF0;
const BOMBER_Y_VELOCITY_DOWN_CORRECTION: u16 = 0x0010;
const BOMBER_PLAYER_TRACKING_FAR_PIXELS: i16 = 0x20;
const BOMBER_PLAYER_TRACKING_NEAR_PIXELS: i16 = 0x10;
const MINI_SWARMER_RANDOM_STEP_MASK: u8 = 0x1F;
const MINI_SWARMER_RANDOM_STEP_CENTER: u8 = 0x10;
const BAITER_COLLISION_WIDTH: i16 = 12;
const BAITER_COLLISION_HEIGHT: i16 = 4;
const MOTION_WORD_SHIFT_LIMIT: u8 = 15;
const MOTION_SEED_STEP_MULTIPLIER: u8 = 17;
const MINI_SWARMER_DAMPING_SHIFT_STEPS: u8 = 2;

#[derive(Debug)]
struct Bomber {
    id: ActorId,
    position: Point,
    drift: i16,
    actor_state: Option<BomberActorState>,
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
                .actor_state
                .map(|actor_state| drift_from_motion_word(actor_state.x_velocity()))
                .unwrap_or(-1),
            actor_state: spawn.actor_state,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 8, 8)
    }

    fn advance_actor_motion(&mut self) -> bool {
        let Some(actor_state) = &mut self.actor_state else {
            return false;
        };

        self.position = actor_state.advance_motion(self.position);
        self.drift = drift_from_motion_word(actor_state.x_velocity());
        true
    }

    fn advance_tie_step(&mut self, prompt: &StepPrompt, actor_rng: ActorRngSnapshot) {
        let Some(actor_state) = &mut self.actor_state else {
            return;
        };
        if actor_state.slot != bomber_tie_selected_slot(actor_rng.seed) {
            return;
        }
        if actor_state.sleep_ticks > 0 {
            actor_state.sleep_ticks = actor_state.sleep_ticks.saturating_sub(1);
            return;
        }

        actor_state.animation_frame = SpriteFrameIndex::new(bomber_sprite_frame_after_tie_seed(
            actor_rng.seed,
            actor_state.animation_frame.index(),
        ));
        actor_state
            .set_y_velocity(bomber_seeded_y_velocity(actor_state.y_velocity(), actor_rng.seed));
        if self.position.y == 0 {
            let y_velocity = bomber_cruise_y_velocity(
                actor_state.y_velocity(),
                &mut actor_state.cruise_altitude,
                self.position.y,
                actor_rng.seed,
            );
            actor_state.set_y_velocity(y_velocity);
        } else if let Some(player) = prompt.player_position()
            && let Some(delta) =
                bomber_player_tracking_y_velocity_delta(self.position.y, player.y)
        {
            actor_state.set_y_velocity(actor_state.y_velocity().wrapping_add(delta));
        }

        actor_state.sleep_ticks = BOMBER_LOOP_SLEEP_TICKS;
    }

    fn draw_effect(&self) -> VisualEffect {
        self.actor_state
            .map(|actor_state| VisualEffect::BomberSpriteFrame {
                animation_frame: actor_state.animation_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }

    fn maybe_spawn_bomb(
        &self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        if let Some(actor_state) = self.actor_state {
            let Some(actor_rng) = prompt.actor_rng else {
                return;
            };
            if actor_state.slot != bomber_tie_selected_slot(actor_rng.seed)
                || actor_state.sleep_ticks > 0
                || self.position.y == 0
                || actor_rng.lseed & BOMBER_BOMB_RANDOM_DROP_MASK != 0
                || actor_bomb_projectile_count(prompt) >= ACTIVE_BOMBER_BOMB_LIMIT
                || actor_enemy_projectile_count(prompt) >= ENEMY_PROJECTILE_SLOT_LIMIT
                || !enemy_projectile_spawn_in_bounds(self.position)
            {
                return;
            }

            commands.push(GameCommand::Spawn(SpawnRequest::Bomb {
                position: self.position,
                actor_state: Some(EnemyProjectileActorState {
                    motion: ActorMotion::new(actor_state.x_fraction(), actor_state.y_fraction(), 0, 0),
                    lifetime_ticks: bomber_bomb_lifetime_ticks(actor_rng),
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
                actor_state: None,
            }));
        }
    }
}

fn bomber_sprite_frame_after_tie_seed(seed: u8, current: u8) -> u8 {
    let step = (seed & BOMBER_RANDOM_STEP_MASK).wrapping_sub(BOMBER_RANDOM_STEP_CENTER);
    if step & MOTION_BYTE_SIGN_BIT != 0 {
        current
            .saturating_add(1)
            .min(BOMBER_ANIMATION_FRAME_COUNT - 1)
    } else {
        current.saturating_sub(1)
    }
}

fn bomber_seeded_y_velocity(previous: u16, seed: u8) -> u16 {
    let random_delta = actor_sign_extend_u8_to_u16(
        (seed & BOMBER_RANDOM_STEP_MASK).wrapping_sub(BOMBER_RANDOM_STEP_CENTER),
    );
    let mut velocity = previous.wrapping_add(random_delta);
    let damping_byte = 0u8.wrapping_sub(velocity.wrapping_shl(3).to_be_bytes()[0]);
    velocity = velocity.wrapping_add(actor_sign_extend_u8_to_u16(damping_byte));
    velocity
}

fn bomber_cruise_y_velocity(
    mut velocity: u16,
    cruise_altitude: &mut i16,
    object_y: i16,
    seed: u8,
) -> u16 {
    if seed <= BOMBER_CRUISE_RANDOM_NUDGE_SEED_MAX {
        let nudge = i16::from(
            (seed & BOMBER_CRUISE_NUDGE_MASK).wrapping_sub(BOMBER_CRUISE_NUDGE_CENTER) as i8,
        );
        *cruise_altitude = (*cruise_altitude + nudge)
            .clamp(BOMBER_MIN_CRUISE_ALTITUDE, BOMBER_MAX_CRUISE_ALTITUDE);
    }

    let distance = *cruise_altitude - object_y;
    if distance.abs() > BOMBER_CRUISE_WINDOW_HALF_PIXELS {
        let correction = if distance >= 0 {
            BOMBER_Y_VELOCITY_UP_CORRECTION
        } else {
            BOMBER_Y_VELOCITY_DOWN_CORRECTION
        };
        velocity = velocity.wrapping_add(correction);
    }
    velocity
}

fn bomber_player_tracking_y_velocity_delta(object_y: i16, player_y: i16) -> Option<u16> {
    let delta = object_y - player_y;
    if delta >= 0 {
        if delta >= BOMBER_PLAYER_TRACKING_FAR_PIXELS {
            Some(BOMBER_Y_VELOCITY_UP_CORRECTION)
        } else if delta > BOMBER_PLAYER_TRACKING_NEAR_PIXELS {
            None
        } else {
            Some(BOMBER_Y_VELOCITY_DOWN_CORRECTION)
        }
    } else if delta <= -BOMBER_PLAYER_TRACKING_FAR_PIXELS {
        Some(BOMBER_Y_VELOCITY_DOWN_CORRECTION)
    } else if delta < -BOMBER_PLAYER_TRACKING_NEAR_PIXELS {
        None
    } else {
        Some(BOMBER_Y_VELOCITY_UP_CORRECTION)
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
            if self.actor_state.is_some() {
                self.maybe_spawn_bomb(prompt, behavior, &mut commands);
                if let Some(actor_rng) = prompt.actor_rng {
                    self.advance_tie_step(prompt, actor_rng);
                }
                self.advance_actor_motion();
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
                actor_state: ActorInternalState::bomber(self.actor_state),
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
    actor_state: EnemyProjectileActorState,
}

impl Bomb {
    fn new(
        id: ActorId,
        position: Point,
        lifetime_steps: u16,
        actor_state: Option<EnemyProjectileActorState>,
    ) -> Self {
        let mut actor_state =
            actor_state.unwrap_or(EnemyProjectileActorState::new(ActorMotion::at_rest(), 0));
        let lifetime_steps = if actor_state.lifetime_ticks == 0 {
            lifetime_steps
        } else {
            u16::from(actor_state.lifetime_ticks)
        };
        actor_state.lifetime_ticks = projectile_lifetime_ticks(lifetime_steps);
        Self {
            id,
            position,
            lifetime_steps,
            actor_state,
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
                self.actor_state.lifetime_ticks =
                    projectile_lifetime_ticks(self.lifetime_steps);
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
                actor_state: ActorInternalState::enemy_projectile(Some(self.actor_state)),
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
    actor_state: Option<PodActorState>,
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
                .actor_state
                .map(|actor_state| drift_from_motion_word(actor_state.x_velocity()))
                .unwrap_or(1),
            actor_state: spawn.actor_state,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 8, 8)
    }

    fn advance_actor_motion(&mut self) -> bool {
        let Some(actor_state) = &mut self.actor_state else {
            return false;
        };
        self.position = actor_state.advance_motion(self.position);
        self.drift = drift_from_motion_word(actor_state.x_velocity());
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
            if !self.advance_actor_motion()
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
                VisualEffect::PodSprite,
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
                actor_state: ActorInternalState::pod(self.actor_state),
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
    actor_state: Option<SwarmerActorState>,
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
            actor_state: spawn.actor_state,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 6, 4)
    }

    fn advance_actor_motion(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(actor_state) = &mut self.actor_state else {
            return false;
        };
        if actor_state.sleep_ticks > 0 {
            actor_state.sleep_ticks = actor_state.sleep_ticks.saturating_sub(1);
            return true;
        }

        let Some(player) = prompt.player_position() else {
            return false;
        };
        let profile = prompt.wave_tuning;
        let mut horizontal_seek_only = false;
        if actor_state.horizontal_seek_pending {
            actor_state.set_x_velocity(mini_swarmer_seek_velocity(
                profile.swarmer_x_velocity,
                player.x,
                self.position.x,
            ));
            actor_state.horizontal_seek_pending = false;
            actor_state.sleep_ticks = MINI_SWARMER_LOOP_SLEEP_TICKS;
            horizontal_seek_only = true;
        }

        let in_shot_window = if horizontal_seek_only {
            false
        } else {
            actor_state.set_y_velocity(mini_swarmer_y_velocity(
                actor_state.y_velocity(),
                actor_state.acceleration,
                player.y,
                self.position.y,
                prompt.actor_rng.map(|rng| rng.seed).unwrap_or(0),
            ));
            let player_absolute_x = absolute_world_x(player, 0);
            let object_absolute_x = absolute_world_x(self.position, actor_state.x_fraction());
            let past_window = player_absolute_x
                .wrapping_sub(object_absolute_x)
                .wrapping_add(MINI_SWARMER_TURN_WINDOW_HALF);
            let in_shot_window = past_window <= MINI_SWARMER_TURN_WINDOW;
            if !in_shot_window {
                actor_state.set_x_velocity(mini_swarmer_seek_velocity(
                    profile.swarmer_x_velocity,
                    player.x,
                    self.position.x,
                ));
            }
            in_shot_window
        };

        self.position = actor_state.advance_motion(self.position);
        if in_shot_window {
            actor_state.shot_timer = actor_state.shot_timer.wrapping_sub(1);
            if actor_state.shot_timer == 0 {
                actor_state.shot_timer = prompt
                    .actor_rng
                    .map(|rng| bounded_actor_rng_value(clamped_swarmer_shot_reset(profile), rng.seed))
                    .unwrap_or_else(|| clamped_swarmer_shot_reset(profile));
                push_swarmer_shot(self.position, prompt, behavior, Some(*actor_state), commands);
            }
        }
        actor_state.sleep_ticks = MINI_SWARMER_LOOP_SLEEP_TICKS;
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
            if !self.advance_actor_motion(prompt, behavior, &mut commands) {
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
                actor_state: ActorInternalState::swarmer(self.actor_state),
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
    actor_state: Option<SwarmerActorState>,
    commands: &mut Vec<GameCommand>,
) {
    if let Some(actor_state) = actor_state {
        if let Some((velocity, projectile_actor_state)) =
            mini_swarmer_fireball(position, prompt, actor_state)
        {
            push_enemy_projectile_command(
                position,
                velocity,
                projectile_actor_state,
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
        actor_state: None,
    }));
    commands.push(GameCommand::PlaySound(SoundCue::SwarmerShot));
}

fn mini_swarmer_fireball(
    position: Point,
    prompt: &StepPrompt,
    actor_state: SwarmerActorState,
) -> Option<(Velocity, EnemyProjectileActorState)> {
    let player = prompt.player_position()?;
    let player_delta = absolute_world_x(player, 0)
        .wrapping_sub(absolute_world_x(position, actor_state.x_fraction()));
    if (player_delta.to_be_bytes()[0] ^ actor_state.x_velocity().to_be_bytes()[0])
        & MOTION_BYTE_SIGN_BIT
        != 0
            || actor_enemy_projectile_count(prompt) >= ENEMY_PROJECTILE_SLOT_LIMIT
    {
        return None;
    }

    let x_velocity = actor_state.x_velocity().wrapping_shl(3);
    let y_velocity = actor_arithmetic_shift_right_word(
        u16::from_be_bytes([(player.y as u8).wrapping_sub(position.y as u8), 0]),
        5,
    );
    let velocity = screen_velocity_from_motion_words(x_velocity, y_velocity);
    Some((
        velocity,
        EnemyProjectileActorState::new(
            ActorMotion::new(0, 0, x_velocity, y_velocity),
            ENEMY_PROJECTILE_LIFETIME_TICKS,
        ),
    ))
}

fn clamped_swarmer_shot_reset(profile: ActorWaveTuning) -> u8 {
    profile.swarmer_shot_time.max(1).min(u32::from(u8::MAX)) as u8
}

#[derive(Debug)]
struct Baiter {
    id: ActorId,
    position: Point,
    drift: i16,
    actor_state: Option<BaiterActorState>,
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
            actor_state: spawn.actor_state,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(
            self.position,
            BAITER_COLLISION_WIDTH,
            BAITER_COLLISION_HEIGHT,
        )
    }

    fn advance_actor_motion(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(actor_state) = &mut self.actor_state else {
            return false;
        };

        if actor_state.sleep_ticks > 0 {
            actor_state.sleep_ticks = actor_state.sleep_ticks.saturating_sub(1);
        } else {
            actor_state.shot_timer = actor_state.shot_timer.wrapping_sub(1);
            if actor_state.shot_timer == 0 {
                let profile = prompt.wave_tuning;
                let shot_rng = baiter_shot_actor_rng(prompt, self.id, self.position);
                actor_state.shot_timer = baiter_shot_timer_reset(profile, shot_rng.seed);
                push_baiter_shot(
                    self.id,
                    self.position,
                    prompt,
                    behavior,
                    Some(*actor_state),
                    Some(shot_rng),
                    commands,
                );
            }

            actor_state.animation_frame = SpriteFrameIndex::new(
                (actor_state.animation_frame.index() + 1) % BAITER_ANIMATION_FRAME_COUNT,
            );
            if actor_state.animation_frame.index() == 0
                && let Some(player) = prompt.player_position()
            {
                let profile = prompt.wave_tuning;
                let seed = prompt
                    .actor_rng
                    .map(|actor_rng| actor_rng.seed)
                    .unwrap_or_else(|| motion_seed(prompt.step, self.id));
                update_baiter_velocity(
                    actor_state,
                    self.position,
                    profile,
                    player,
                    prompt.player_velocity().unwrap_or_default(),
                    true,
                    seed,
                );
            }
            actor_state.sleep_ticks = BAITER_LOOP_SLEEP_TICKS;
        }

        self.position = actor_state.advance_motion_with_x_velocity(
            self.position,
            baiter_screen_x_velocity(actor_state.x_velocity()),
        );
        true
    }

    fn draw_effect(&self) -> VisualEffect {
        self.actor_state
            .map(|actor_state| VisualEffect::BaiterSpriteFrame {
                animation_frame: actor_state.animation_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

fn push_baiter_shot(
    actor: ActorId,
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    actor_state: Option<BaiterActorState>,
    shot_rng: Option<ActorRngSnapshot>,
    commands: &mut Vec<GameCommand>,
) {
    if let Some(actor_state) = actor_state {
        let shot_rng =
            shot_rng.unwrap_or_else(|| baiter_shot_actor_rng(prompt, actor, position));
        if let Some((velocity, projectile_actor_state)) =
            baiter_fireball(position, prompt, actor_state, shot_rng)
        {
            push_enemy_projectile_command(
                position,
                velocity,
                projectile_actor_state,
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
        actor_state: None,
    }));
    commands.push(GameCommand::PlaySound(SoundCue::BaiterShot));
}

fn baiter_shot_actor_rng(
    prompt: &StepPrompt,
    actor: ActorId,
    position: Point,
) -> ActorRngSnapshot {
    prompt.actor_rng.unwrap_or(ActorRngSnapshot {
        seed: motion_seed(prompt.step, actor),
        hseed: position.x as u8,
        lseed: position.y as u8,
    })
}

fn baiter_shot_timer_reset(profile: ActorWaveTuning, seed: u8) -> u8 {
    bounded_actor_rng_value(clamped_baiter_shot_timer_reset(profile), seed)
}

fn baiter_fireball(
    position: Point,
    prompt: &StepPrompt,
    actor_state: BaiterActorState,
    shot_rng: ActorRngSnapshot,
) -> Option<(Velocity, EnemyProjectileActorState)> {
    enemy_fireball_projectile(
        position,
        actor_state.x_fraction(),
        actor_state.y_fraction(),
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
            if !self.advance_actor_motion(prompt, behavior, &mut commands) {
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
                actor_state: ActorInternalState::baiter(self.actor_state),
            },
            commands,
            draws,
        }
    }
}

fn clamped_baiter_shot_timer_reset(profile: ActorWaveTuning) -> u8 {
    profile.baiter_shot_time.max(1).min(u32::from(u8::MAX)) as u8
}

fn baiter_screen_x_velocity(actor_x_velocity: u16) -> u16 {
    actor_x_velocity.wrapping_shl(2)
}

fn update_baiter_velocity(
    actor_state: &mut BaiterActorState,
    position: Point,
    profile: ActorWaveTuning,
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
            actor_arithmetic_shift_right_word(motion_velocity_word(player_velocity.dx), 2);
        actor_state.set_x_velocity(
            actor_sign_extend_u8_to_u16(x_seek_byte).wrapping_add(player_x_velocity),
        );
    }

    let y_delta = position.y - player_position.y;
    if y_delta.abs() > BAITER_Y_SEEK_WINDOW_HALF_PIXELS {
        let y_seek_byte = if y_delta > 0 {
            0u8.wrapping_sub(BAITER_Y_SEEK_BYTE)
        } else {
            BAITER_Y_SEEK_BYTE
        };
        actor_state.set_y_velocity(actor_arithmetic_shift_right_word(
            u16::from_be_bytes([y_seek_byte, 0])
                .wrapping_add(motion_velocity_word(player_velocity.dy)),
            1,
        ));
    }

    true
}

fn actor_arithmetic_shift_right_word(value: u16, shift: u8) -> u16 {
    ((value as i16) >> shift.min(MOTION_WORD_SHIFT_LIMIT)) as u16
}

fn motion_velocity_word(value: i16) -> u16 {
    value as u16
}

fn motion_seed(step: u64, id: ActorId) -> u8 {
    (step as u8)
        .wrapping_mul(MOTION_SEED_STEP_MULTIPLIER)
        .wrapping_add(id.value() as u8)
}

fn mini_swarmer_seek_velocity(x_velocity_magnitude: u8, player_x: i16, swarmer_x: i16) -> u16 {
    if player_x >= swarmer_x {
        actor_sign_extend_u8_to_u16(x_velocity_magnitude)
    } else {
        actor_sign_extend_u8_to_u16(0u8.wrapping_sub(x_velocity_magnitude))
    }
}

fn mini_swarmer_y_velocity(
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
    y_velocity = y_velocity.wrapping_add(mini_swarmer_damping_adjustment(y_velocity));
    y_velocity.wrapping_add(actor_sign_extend_u8_to_u16(
        (seed & MINI_SWARMER_RANDOM_STEP_MASK).wrapping_sub(MINI_SWARMER_RANDOM_STEP_CENTER),
    ))
}

fn mini_swarmer_damping_adjustment(value: u16) -> u16 {
    let [mut a, mut b] = value.to_be_bytes();
    a = !a;
    b = !b;
    for _ in 0..MINI_SWARMER_DAMPING_SHIFT_STEPS {
        let carry = b & MOTION_BYTE_SIGN_BIT != 0;
        b = b.wrapping_shl(1);
        a = a.wrapping_shl(1) | u8::from(carry);
    }
    actor_sign_extend_u8_to_u16(a)
}
