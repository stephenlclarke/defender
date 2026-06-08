#[derive(Debug)]
struct AttractDirector {
    id: ActorId,
}

impl AttractDirector {
    fn new(id: ActorId) -> Self {
        Self { id }
    }
}

impl AssetActor for AttractDirector {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        for _ in 0..prompt.input.coin_insertions() {
            commands.push(GameCommand::Credit);
        }
        if prompt.input.start_one {
            commands.push(GameCommand::StartOnePlayer);
        }
        if prompt.input.start_two {
            commands.push(GameCommand::StartTwoPlayer);
        }
        if prompt.phase == Phase::HighScoreEntry {
            draws.push(DrawCommand::text(
                self.id,
                Point::new(66, 120),
                "ENTER INITIALS",
            ));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::AttractDirector,
                position: Point::new(0, 0),
                velocity: Velocity::default(),
                direction: None,
                bounds: None,
                alive: true,
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
struct ScriptedAttractProgram {
    id: ActorId,
    script: AttractScript,
    elapsed_steps: u64,
}

impl ScriptedAttractProgram {
    fn new(id: ActorId, script: AttractScript) -> Self {
        Self {
            id,
            script,
            elapsed_steps: 0,
        }
    }
}

impl AssetActor for ScriptedAttractProgram {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let draws = match prompt.phase {
            Phase::Attract => {
                self.elapsed_steps = self.elapsed_steps.saturating_add(1);
                self.script.draws_for(
                    self.id,
                    self.elapsed_steps,
                    &prompt.high_scores,
                    prompt.credits,
                )
            }
            Phase::GameOver if prompt.game_over_hall_of_fame_stall_remaining.is_some() => {
                self.elapsed_steps = 0;
                self.script.draws_for(
                    self.id,
                    ATTRACT_HALL_OF_FAME_START_STEP,
                    &prompt.high_scores,
                    prompt.credits,
                )
            }
            Phase::GameOver if prompt.player_switch.is_some() => {
                self.elapsed_steps = 0;
                Vec::new()
            }
            Phase::GameOver if prompt.player_death_sleep_remaining.is_some() => {
                self.elapsed_steps = 0;
                Vec::new()
            }
            Phase::GameOver => {
                self.elapsed_steps = 0;
                Vec::new()
            }
            Phase::Playing | Phase::HighScoreEntry => {
                self.elapsed_steps = 0;
                Vec::new()
            }
        };

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::AttractScript,
                position: Point::new(0, 0),
                velocity: Velocity::default(),
                direction: None,
                bounds: None,
                alive: true,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands: Vec::new(),
            draws,
        }
    }
}

#[derive(Debug)]
struct StatusDisplay {
    id: ActorId,
}

impl StatusDisplay {
    fn new(id: ActorId) -> Self {
        Self { id }
    }

    fn playing_draws(&self, prompt: &StepPrompt) -> Vec<DrawCommand> {
        vec![
            DrawCommand::text(
                self.id,
                STATUS_WAVE_POSITION,
                format!("WAVE {:02}", prompt.wave.min(99)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_CREDITS_POSITION,
                format!("CREDIT {:02}", prompt.credits.min(99)),
            ),
        ]
    }

    fn high_score_entry_draws(&self, prompt: &StepPrompt) -> Vec<DrawCommand> {
        let mut draws = vec![
            DrawCommand::text(
                self.id,
                STATUS_FINAL_SCORE_POSITION,
                format!("FINAL SCORE {}", format_status_score(prompt.score)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_HIGH_SCORE_TABLE_TITLE_POSITION,
                "HIGH SCORES",
            ),
            DrawCommand::text(
                self.id,
                Point::new(66, 104),
                format!(
                    "INITIALS {}",
                    format_high_score_initials(prompt.high_score_initials)
                ),
            ),
        ];

        for (index, score) in prompt.high_scores.iter().enumerate() {
            draws.push(DrawCommand::text(
                self.id,
                Point::new(
                    82,
                    STATUS_HIGH_SCORE_TABLE_START_Y
                        + i16::try_from(index).unwrap_or(0) * STATUS_HIGH_SCORE_TABLE_ROW_HEIGHT,
                ),
                format!("{}. {}", index + 1, format_status_score(*score)),
            ));
        }
        draws
    }

    fn player_switch_draws(&self, prompt: &StepPrompt) -> Vec<DrawCommand> {
        if prompt.player_switch.is_none() {
            return Vec::new();
        };

        vec![
            DrawCommand::text(
                self.id,
                STATUS_SCORE_POSITION,
                format!("1UP {}", format_status_score(prompt.player_scores[0])),
            ),
            DrawCommand::text(
                self.id,
                STATUS_PLAYER_TWO_SCORE_POSITION,
                format!("2UP {}", format_status_score(prompt.player_scores[1])),
            ),
            DrawCommand::text(
                self.id,
                STATUS_HIGH_SCORE_POSITION,
                format!("HIGH {}", format_status_score(prompt.high_scores[0])),
            ),
            DrawCommand::text(
                self.id,
                STATUS_CREDITS_POSITION,
                format!("CREDIT {:02}", prompt.credits.min(99)),
            ),
        ]
    }

    fn snapshot(&self) -> ActorSnapshot {
        ActorSnapshot {
            id: self.id,
            kind: ActorKind::StatusDisplay,
            position: Point::new(0, 0),
            velocity: Velocity::default(),
            direction: None,
            bounds: None,
            alive: true,
            source_lander: None,
            source_bomber: None,
            source_pod: None,
            source_swarmer: None,
            source_baiter: None,
            source_mutant: None,
            source_human: None,
            source_enemy_projectile: None,
        }
    }
}

impl AssetActor for StatusDisplay {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let draws = match prompt.phase {
            Phase::Playing => self.playing_draws(prompt),
            Phase::HighScoreEntry => self.high_score_entry_draws(prompt),
            Phase::GameOver if prompt.player_switch.is_some() => self.player_switch_draws(prompt),
            Phase::Attract | Phase::GameOver => Vec::new(),
        };

        ActorReply {
            id: self.id,
            snapshot: self.snapshot(),
            commands: Vec::new(),
            draws,
        }
    }
}

fn format_status_score(score: u32) -> String {
    format!("{:06}", score.min(999_999))
}

fn format_high_score_initials(state: HighScoreInitialsState) -> String {
    state
        .initials
        .iter()
        .map(|initial| initial.unwrap_or('_'))
        .collect()
}

fn player_source_message_label(player: u8) -> &'static str {
    if player == 2 { "PLYR2" } else { "PLYR1" }
}

#[derive(Debug)]
struct PlayerShip {
    id: ActorId,
    position: Point,
    direction: Direction,
    reverse_held: bool,
    laser_cooldown: u8,
    hyperspace_steps_remaining: u8,
    hyperspace_entry_lseed: Option<u8>,
    hyperspace_death_steps_remaining: Option<u8>,
}

impl PlayerShip {
    fn new(id: ActorId, position: Point) -> Self {
        Self {
            id,
            position,
            direction: Direction::Right,
            reverse_held: false,
            laser_cooldown: 0,
            hyperspace_steps_remaining: 0,
            hyperspace_entry_lseed: None,
            hyperspace_death_steps_remaining: None,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 18, 10)
    }

    fn is_hidden_for_hyperspace(&self) -> bool {
        self.hyperspace_steps_remaining > 0
    }

    fn enter_hyperspace(&mut self, behavior: ActorBehaviorProfile) {
        self.hyperspace_steps_remaining = behavior.player_hyperspace_hidden_steps;
        self.hyperspace_entry_lseed = Some(Self::hyperspace_death_lseed(behavior));
    }

    fn hyperspace_death_lseed(behavior: ActorBehaviorProfile) -> u8 {
        behavior
            .player_hyperspace_source_seed
            .map_or(behavior.player_hyperspace_death_lseed, |source| {
                source.lseed
            })
    }

    fn step_horizontal_motion(position_x: i16, dx: i16, background_left: u16) -> (i16, u16) {
        if dx == 0 {
            return (position_x, background_left);
        }

        if dx > 0 {
            return Self::step_right_motion(position_x, dx, background_left);
        }

        Self::step_left_motion(position_x, dx.saturating_abs(), background_left)
    }

    fn step_right_motion(position_x: i16, dx: i16, background_left: u16) -> (i16, u16) {
        let next_x = position_x.saturating_add(dx);
        if position_x >= PLAYER_SCROLL_CENTER_X {
            return (
                PLAYER_SCROLL_CENTER_X,
                scroll_background_right(background_left, dx),
            );
        }
        if next_x <= PLAYER_SCROLL_CENTER_X {
            return (next_x, background_left);
        }

        let scroll_pixels = next_x.saturating_sub(PLAYER_SCROLL_CENTER_X);
        (
            PLAYER_SCROLL_CENTER_X,
            scroll_background_right(background_left, scroll_pixels),
        )
    }

    fn step_left_motion(position_x: i16, dx: i16, background_left: u16) -> (i16, u16) {
        let next_x = position_x.saturating_sub(dx);
        if position_x <= PLAYER_SCROLL_CENTER_X {
            return (
                PLAYER_SCROLL_CENTER_X,
                scroll_background_left(background_left, dx),
            );
        }
        if next_x >= PLAYER_SCROLL_CENTER_X {
            return (next_x, background_left);
        }

        let scroll_pixels = PLAYER_SCROLL_CENTER_X.saturating_sub(next_x);
        (
            PLAYER_SCROLL_CENTER_X,
            scroll_background_left(background_left, scroll_pixels),
        )
    }

    fn hyperspace_rematerialization(
        &self,
        behavior: ActorBehaviorProfile,
    ) -> (Point, Direction, Option<u16>) {
        if let Some(source) = behavior.player_hyperspace_source_seed {
            let (x, direction) = if source.hseed & 1 != 0 {
                (0x20, Direction::Right)
            } else {
                (0x70, Direction::Left)
            };
            let y = (source.hseed >> 1).wrapping_add(PLAYFIELD_TOP_EDGE_Y);
            return (
                Point::new(x, i16::from(y)),
                direction,
                Some(actor_source_hyperspace_background_left(source)),
            );
        }

        (
            Point::new(
                behavior.player_hyperspace_rematerialize_x,
                behavior.player_hyperspace_rematerialize_y,
            ),
            self.direction,
            None,
        )
    }

    fn advance_hyperspace(
        &mut self,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        if self.hyperspace_steps_remaining == 0 {
            return false;
        }

        self.hyperspace_steps_remaining = self.hyperspace_steps_remaining.saturating_sub(1);
        if self.hyperspace_steps_remaining == 0 {
            let (position, direction, source_background_left) =
                self.hyperspace_rematerialization(behavior);
            self.position = PLAYER_BOUNDS.clamp_point(position);
            self.direction = direction;
            if let Some(source_background_left) = source_background_left {
                commands.push(GameCommand::SetSourceBackgroundLeft(source_background_left));
            }
            let death_lseed = self
                .hyperspace_entry_lseed
                .take()
                .unwrap_or_else(|| Self::hyperspace_death_lseed(behavior));
            if death_lseed > HYPERSPACE_DEATH_LOW_SEED_THRESHOLD {
                self.hyperspace_death_steps_remaining =
                    Some(behavior.player_hyperspace_death_delay_steps);
            }
            return true;
        }
        false
    }

    fn advance_hyperspace_death(&mut self, commands: &mut Vec<GameCommand>) -> bool {
        let Some(steps_remaining) = self.hyperspace_death_steps_remaining else {
            return false;
        };

        let steps_remaining = steps_remaining.saturating_sub(1);
        if steps_remaining > 0 {
            self.hyperspace_death_steps_remaining = Some(steps_remaining);
            return false;
        }

        self.hyperspace_death_steps_remaining = None;
        commands.push(GameCommand::Destroy(self.id));
        commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
            position: self.position,
            kind: ExplosionKind::Player,
            source_center: None,
        }));
        commands.push(GameCommand::PlaySound(SoundCue::Explosion));
        commands.push(GameCommand::PlayerKilled);
        true
    }
}

impl AssetActor for PlayerShip {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let mut death_due = false;
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Player);
            death_due = self.advance_hyperspace_death(&mut commands);
            let was_hidden = self.is_hidden_for_hyperspace();
            if self.advance_hyperspace(behavior, &mut commands) {
                commands.push(GameCommand::PlaySound(SoundCue::HyperspaceMaterialize));
            }
            let input_blocked = was_hidden || self.hyperspace_death_steps_remaining.is_some();
            if !death_due && !input_blocked {
                let mut next_position = self.position;
                let mut source_background_left = prompt.source_background_left;
                if prompt.input.altitude_up {
                    next_position.y = next_position.y.saturating_sub(behavior.player_speed);
                }
                if prompt.input.altitude_down {
                    next_position.y = next_position.y.saturating_add(behavior.player_speed);
                }
                if prompt.input.thrust {
                    let (next_x, next_background_left) = Self::step_horizontal_motion(
                        next_position.x,
                        self.direction.sign() * behavior.player_speed,
                        source_background_left,
                    );
                    next_position.x = next_x;
                    source_background_left = next_background_left;
                    commands.push(GameCommand::PlaySound(SoundCue::Thrust));
                }
                if prompt.input.reverse && !self.reverse_held {
                    self.direction = match self.direction {
                        Direction::Left => Direction::Right,
                        Direction::Right => Direction::Left,
                    };
                }
                self.position.y =
                    clamp_i16(next_position.y, PLAYER_BOUNDS.top, PLAYER_BOUNDS.bottom);
                self.position.x =
                    clamp_i16(next_position.x, PLAYER_BOUNDS.left, PLAYER_BOUNDS.right);
                if source_background_left != prompt.source_background_left {
                    commands.push(GameCommand::SetSourceBackgroundLeft(source_background_left));
                }
                self.laser_cooldown = self.laser_cooldown.saturating_sub(1);
                if prompt.input.wants_fire() && self.laser_cooldown == 0 {
                    self.laser_cooldown = behavior.player_laser_cooldown_steps;
                    let muzzle = self
                        .position
                        .offset(Velocity::new(self.direction.sign() * 12, 0));
                    commands.push(GameCommand::Spawn(SpawnRequest::Laser {
                        position: muzzle,
                        direction: self.direction,
                        owner: self.id,
                    }));
                    commands.push(GameCommand::PlaySound(SoundCue::Laser));
                }
                if prompt.input.xyzzy.overlay_smart_bomb && !prompt.smart_bomb_pending {
                    commands.push(GameCommand::SmartBomb {
                        consume_stock: false,
                    });
                } else if prompt.input.wants_stock_smart_bomb()
                    && prompt.smart_bombs > 0
                    && !prompt.smart_bomb_pending
                {
                    commands.push(GameCommand::SmartBomb {
                        consume_stock: true,
                    });
                }
                if prompt.input.hyperspace {
                    commands.push(GameCommand::Hyperspace);
                    commands.push(GameCommand::PlaySound(SoundCue::Hyperspace));
                    self.enter_hyperspace(behavior);
                }
            }
            self.reverse_held = prompt.input.reverse;
            if !death_due && !self.is_hidden_for_hyperspace() {
                draws.push(DrawCommand::sprite(
                    self.id,
                    match self.direction {
                        Direction::Left => SpriteKey::PlayerLeft,
                        Direction::Right => SpriteKey::PlayerRight,
                    },
                    self.position,
                ));
            }
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Player,
                position: self.position,
                velocity: if prompt.phase == Phase::Playing {
                    observed_velocity(previous_position, self.position)
                } else {
                    Velocity::default()
                },
                direction: Some(self.direction),
                bounds: if prompt.phase == Phase::Playing
                    && !self.is_hidden_for_hyperspace()
                    && !death_due
                {
                    Some(self.bounds())
                } else {
                    None
                },
                alive: prompt.phase == Phase::Playing,
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
