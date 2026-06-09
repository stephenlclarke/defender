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
                .arcade_state
                .map(|arcade_state| {
                    lander_drift_from_arcade_velocity(arcade_state.x_velocity)
                })
                .unwrap_or(-1),
            mode: LanderMode::Seeking,
            spawn_visibility: lander_spawn_visibility(spawn.arcade_state),
            arcade_state: spawn.arcade_state,
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
                arcade_state: self.mutant_arcade_conversion(prompt),
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
            arcade_state: projectile_arcade_state,
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
                animation_frame: arcade_state.animation_frame,
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
        .filter_map(|spawn| spawn.arcade_state)
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
        && arcade_state.animation_frame == refill_arcade_state.animation_frame
        && arcade_state.target_human_index == refill_arcade_state.target_human_index
}

fn clamped_lander_fire_timer_reset(behavior: ActorBehaviorProfile) -> u8 {
    let clamped = behavior
        .lander_fire_period_steps
        .max(1)
        .min(u64::from(u8::MAX));
    u8::try_from(clamped).unwrap_or(u8::MAX)
}
fn pickup_distance(lander: Point, human: Point, behavior: ActorBehaviorProfile) -> bool {
    (lander.x - human.x).abs() <= behavior.lander_pickup_radius_x
        && (lander.y - human.y).abs() <= behavior.lander_pickup_radius_y
}
