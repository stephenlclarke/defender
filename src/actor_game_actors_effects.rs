#[derive(Debug)]
struct Human {
    id: ActorId,
    position: Point,
    mode: HumanMode,
    safe_landing_awarded: bool,
    source: Option<ActorSourceHumanMetadata>,
}

impl Human {
    fn new(id: ActorId, position: Point, mode: HumanMode) -> Self {
        Self::with_source(id, position, mode, None)
    }

    fn from_spawn(id: ActorId, spawn: ActorHumanSpawn) -> Self {
        Self::with_source(id, spawn.position, spawn.mode, spawn.source)
    }

    fn with_source(
        id: ActorId,
        position: Point,
        mode: HumanMode,
        source: Option<ActorSourceHumanMetadata>,
    ) -> Self {
        Self {
            id,
            position,
            mode,
            safe_landing_awarded: false,
            source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 4, 8)
    }

    fn screen_bounds(&self, background_left: u16) -> Option<Rect> {
        let bounds = self.bounds();
        let Some(source) = self.source else {
            return Some(bounds);
        };
        let position = actor_source_project_screen_position(
            self.position,
            source.x_fraction,
            background_left,
        )?;
        let delta = Velocity::new(position.x - self.position.x, position.y - self.position.y);
        Some(translate_rect(bounds, delta))
    }

    fn update_grounded(
        &mut self,
        source_rng: Option<ActorSourceRngSnapshot>,
        source_walk_target_slot: Option<usize>,
    ) {
        let Some(source) = self.source else {
            return;
        };
        if source_walk_target_slot != Some(source.target_slot_index) {
            return;
        }

        if let Some(source_rng) = source_rng {
            self.advance_source_walk(source_rng.seed);
        }
    }

    fn advance_source_walk(&mut self, source_seed: u8) {
        if let Some(source) = &mut self.source {
            let frame = source.picture_frame % 4;
            let (next_frame, target_y, velocity) = if frame <= 1 {
                if source_seed <= HUMAN_TURN_SEED_MAX {
                    (2, None, HUMAN_RIGHT_X_VELOCITY)
                } else {
                    (
                        1 - frame,
                        actor_source_human_target_y(self.position.x, HUMAN_LEFT_TARGET_Y_OFFSET),
                        HUMAN_LEFT_X_VELOCITY,
                    )
                }
            } else if source_seed <= HUMAN_TURN_SEED_MAX {
                (0, None, HUMAN_LEFT_X_VELOCITY)
            } else {
                (
                    if frame == 2 { 3 } else { 2 },
                    actor_source_human_target_y(self.position.x, HUMAN_RIGHT_TARGET_Y_OFFSET),
                    HUMAN_RIGHT_X_VELOCITY,
                )
            };
            if let Some(target_y) = target_y {
                self.position.y = actor_source_step_human_y(self.position.y, target_y);
            }
            let (x, x_fraction) =
                actor_source_axis_step(self.position.x, source.x_fraction, velocity);
            self.position.x = x;
            source.x_fraction = x_fraction;
            source.picture_frame = next_frame;
        }
    }

    fn update_falling(
        &mut self,
        velocity: i16,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> Vec<GameCommand> {
        let mut commands = Vec::new();
        let next_velocity =
            (velocity + behavior.human_fall_acceleration).min(behavior.human_max_fall_speed);
        self.position = self.position.offset(Velocity::new(0, next_velocity));

        if self
            .screen_bounds(prompt.source_background_left)
            .is_some_and(|bounds| {
                prompt.snapshots.iter().any(|snapshot| {
                    snapshot.kind == ActorKind::Player && intersects_snapshot(snapshot, bounds)
                })
            })
        {
            commands.push(GameCommand::Destroy(self.id));
            commands.push(GameCommand::AddScore(HUMAN_RESCUE_SCORE));
            commands.push(GameCommand::Spawn(SpawnRequest::ScorePopup {
                position: self.position,
                points: HUMAN_RESCUE_SCORE,
            }));
            commands.push(GameCommand::PlaySound(SoundCue::HumanRescued));
            return commands;
        }

        if self.position.y >= behavior.human_ground_y {
            self.position.y = behavior.human_ground_y;
            if next_velocity <= behavior.human_safe_landing_speed {
                self.mode = HumanMode::Grounded;
                if !self.safe_landing_awarded {
                    self.safe_landing_awarded = true;
                    commands.push(GameCommand::AddScore(HUMAN_SAFE_LANDING_SCORE));
                    commands.push(GameCommand::Spawn(SpawnRequest::ScorePopup {
                        position: self.position,
                        points: HUMAN_SAFE_LANDING_SCORE,
                    }));
                    commands.push(GameCommand::PlaySound(SoundCue::HumanSafeLanding));
                }
            } else {
                commands.push(GameCommand::Destroy(self.id));
                commands.push(GameCommand::HumanLost(self.id));
                commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                    position: self.position,
                    kind: ExplosionKind::Human,
                    source_center: None,
                }));
                commands.push(GameCommand::PlaySound(SoundCue::HumanLost));
            }
        } else {
            self.mode = HumanMode::Falling {
                velocity: next_velocity,
            };
        }

        commands
    }

    fn update_carried(
        &mut self,
        carrier: ActorId,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> Vec<GameCommand> {
        let mut commands = Vec::new();
        if let Some(carrier_snapshot) = prompt.snapshot(carrier) {
            self.position = carrier_snapshot
                .position
                .offset(Velocity::new(0, behavior.human_carried_offset_y));
        } else {
            self.mode = HumanMode::Falling { velocity: 0 };
            commands.push(GameCommand::PlaySound(SoundCue::HumanReleased));
        }
        commands
    }
}

impl AssetActor for Human {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Human);
            commands.extend(match self.mode {
                HumanMode::Grounded => {
                    self.update_grounded(prompt.source_rng, prompt.source_human_walk_target_slot);
                    Vec::new()
                }
                HumanMode::Falling { velocity } => self.update_falling(velocity, prompt, behavior),
                HumanMode::CarriedBy(carrier) => self.update_carried(carrier, prompt, behavior),
            });
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                self.mode.sprite(),
                self.position,
                self.draw_effect(),
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Human,
                position: self.position,
                velocity: movement_velocity,
                direction: None,
                bounds: human_collision_bounds(self.mode, self.position),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: self.source,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

impl Human {
    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceHumanFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

fn intersects_snapshot(snapshot: &ActorSnapshot, bounds: Rect) -> bool {
    snapshot
        .bounds
        .is_some_and(|snapshot_bounds| snapshot_bounds.intersects(bounds))
}

fn human_collision_bounds(mode: HumanMode, position: Point) -> Option<Rect> {
    match mode {
        HumanMode::CarriedBy(_) => None,
        HumanMode::Grounded | HumanMode::Falling { .. } => Some(Rect::from_center(position, 4, 8)),
    }
}

fn actor_source_astronaut_walk_targetable(human_count: usize, snapshot: &ActorSnapshot) -> bool {
    snapshot.kind == ActorKind::Human
        && snapshot.alive
        && snapshot.bounds.is_some()
        && snapshot.source_human.is_some_and(|source| {
            human_count != usize::from(START_HUMAN_COUNT) || source.target_slot_index < 2
        })
}

#[derive(Debug)]
struct ScorePopup {
    id: ActorId,
    position: Point,
    points: u32,
    age: u16,
}

impl ScorePopup {
    fn new(id: ActorId, position: Point, points: u32) -> Self {
        Self {
            id,
            position,
            points,
            age: 0,
        }
    }
}

impl AssetActor for ScorePopup {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let behavior = prompt.behavior_for(self.id, ActorKind::ScorePopup);
        if self.age < behavior.score_popup_lifetime_steps {
            let sprite = match self.points {
                HUMAN_RESCUE_SCORE => SpriteKey::Score500,
                HUMAN_SAFE_LANDING_SCORE => SpriteKey::Score250,
                _ => SpriteKey::Text,
            };
            draws.push(DrawCommand::sprite(self.id, sprite, self.position));
            self.age = self.age.saturating_add(1);
        }
        if self.age >= behavior.score_popup_lifetime_steps {
            commands.push(GameCommand::Destroy(self.id));
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::ScorePopup,
                position: self.position,
                velocity: Velocity::default(),
                direction: None,
                bounds: None,
                alive: self.age < behavior.score_popup_lifetime_steps,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct LaserShot {
    id: ActorId,
    position: Point,
    direction: Direction,
    age: u16,
}

impl LaserShot {
    fn new(id: ActorId, position: Point, direction: Direction, _owner: ActorId) -> Self {
        Self {
            id,
            position,
            direction,
            age: 0,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 10, 2)
    }
}

impl AssetActor for LaserShot {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let mut movement_velocity = Velocity::default();
        let behavior = prompt.behavior_for(self.id, ActorKind::Laser);
        if prompt.phase == Phase::Playing && self.age < behavior.laser_lifetime_steps {
            movement_velocity = Velocity::new(self.direction.sign() * behavior.laser_speed, 0);
            self.position = self.position.offset(movement_velocity);
            self.age = self.age.saturating_add(1);
            draws.push(DrawCommand::sprite(
                self.id,
                SpriteKey::Laser,
                self.position,
            ));
        }
        if self.age >= behavior.laser_lifetime_steps || self.position.x < 0 || self.position.x > 255
        {
            commands.push(GameCommand::Destroy(self.id));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Laser,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(self.direction),
                bounds: Some(self.bounds()),
                alive: self.age < behavior.laser_lifetime_steps,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct EnemyLaserShot {
    id: ActorId,
    position: Point,
    source: ActorSourceEnemyProjectileMetadata,
    lifetime_steps: Option<u16>,
}

impl EnemyLaserShot {
    fn new(
        id: ActorId,
        position: Point,
        velocity: Velocity,
        source: Option<ActorSourceEnemyProjectileMetadata>,
    ) -> Self {
        let source = source.unwrap_or(ActorSourceEnemyProjectileMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: actor_source_projectile_velocity_component(velocity.dx),
            y_velocity: actor_source_projectile_velocity_component(velocity.dy),
            lifetime_ticks: 0,
        });
        let lifetime_steps = if source.lifetime_ticks == 0 {
            None
        } else {
            Some(u16::from(source.lifetime_ticks))
        };
        Self {
            id,
            position,
            source,
            lifetime_steps,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 4, 4)
    }

    fn in_playfield(&self) -> bool {
        self.position.x >= 0
            && self.position.x <= 255
            && self.position.y >= 0
            && self.position.y <= 255
    }

    fn initialize_lifetime(&mut self, behavior: ActorBehaviorProfile) {
        let lifetime_steps = self
            .lifetime_steps
            .get_or_insert(behavior.lander_shot_lifetime_steps);
        self.source.lifetime_ticks = actor_source_projectile_lifetime_ticks(*lifetime_steps);
    }

    fn advance_source_projectile(&mut self) -> Velocity {
        let previous_position = self.position;
        let (x, x_fraction) = actor_source_projectile_axis_step(
            self.position.x,
            self.source.x_fraction,
            self.source.x_velocity,
        );
        let (y, y_fraction) = actor_source_projectile_axis_step(
            self.position.y,
            self.source.y_fraction,
            self.source.y_velocity,
        );
        self.position = Point::new(x, y);
        self.source.x_fraction = x_fraction;
        self.source.y_fraction = y_fraction;
        observed_velocity(previous_position, self.position)
    }
}

impl AssetActor for EnemyLaserShot {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let mut movement_velocity = Velocity::default();
        let behavior = prompt.behavior_for(self.id, ActorKind::EnemyLaser);
        self.initialize_lifetime(behavior);
        if prompt.phase == Phase::Playing && self.lifetime_steps.is_some_and(|steps| steps > 0) {
            if prompt.source_shell_scan_tick
                && let Some(lifetime_steps) = &mut self.lifetime_steps
            {
                *lifetime_steps = lifetime_steps.saturating_sub(1);
                self.source.lifetime_ticks =
                    actor_source_projectile_lifetime_ticks(*lifetime_steps);
            }
            if self.lifetime_steps.is_some_and(|steps| steps > 0) {
                movement_velocity = self.advance_source_projectile();
                draws.push(DrawCommand::sprite(
                    self.id,
                    SpriteKey::EnemyLaser,
                    self.position,
                ));
            }
        }
        let expired_or_out_of_bounds = self.lifetime_steps == Some(0) || !self.in_playfield();
        if expired_or_out_of_bounds {
            commands.push(GameCommand::Destroy(self.id));
        }
        let alive = prompt.phase == Phase::Playing && !expired_or_out_of_bounds;
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::EnemyLaser,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(movement_velocity, Direction::Right)),
                bounds: Some(self.bounds()),
                alive,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: Some(self.source),
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct Explosion {
    id: ActorId,
    position: Point,
    kind: ExplosionKind,
    source_center: Option<Point>,
    age: u16,
}

impl Explosion {
    fn new(
        id: ActorId,
        position: Point,
        kind: ExplosionKind,
        source_center: Option<Point>,
    ) -> Self {
        Self {
            id,
            position,
            kind,
            source_center,
            age: 0,
        }
    }
}

impl AssetActor for Explosion {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let behavior = prompt.behavior_for(self.id, ActorKind::Explosion);
        let lifetime_steps = explosion_lifetime_steps(self.kind, behavior);
        if self.age < lifetime_steps {
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Explosion,
                self.position,
                VisualEffect::ExplosionCloud {
                    kind: self.kind,
                    age: self.age,
                    source_center: self.source_center,
                },
            ));
            self.age = self.age.saturating_add(1);
        }
        if self.age >= lifetime_steps {
            commands.push(GameCommand::Destroy(self.id));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Explosion,
                position: self.position,
                velocity: Velocity::default(),
                direction: None,
                bounds: None,
                alive: self.age < lifetime_steps,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

fn explosion_lifetime_steps(kind: ExplosionKind, behavior: ActorBehaviorProfile) -> u16 {
    if kind == ExplosionKind::Terrain {
        return u16::from(TERRAIN_EXPLOSION_LIFETIME_FRAMES);
    }

    behavior.explosion_lifetime_steps
}
