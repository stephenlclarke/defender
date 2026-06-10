#[derive(Debug)]
struct Lander {
    id: ActorId,
    position: Point,
    drift: i16,
    mode: LanderMode,
    runtime_state: Option<LanderRuntimeState>,
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
                .runtime_state
                .map(|runtime_state| {
                    lander_drift_from_motion_word(runtime_state.x_velocity)
                })
                .unwrap_or(-1),
            mode: LanderMode::Seeking,
            spawn_visibility: lander_spawn_visibility(spawn.runtime_state),
            runtime_state: spawn.runtime_state,
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
        if let Some(runtime_state) = &mut self.runtime_state
            && runtime_state.target_human_index == Some(target_human_index)
            && runtime_state.x_velocity == x_velocity
        {
            runtime_state.shot_timer = runtime_state.shot_timer.wrapping_add(delta);
        }
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Lander);
            if self.spawn_visibility != LanderSpawnVisibility::HiddenFirstWaveRefill
                && !self.tick_runtime_sleep()
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
                runtime: ActorRuntimeState::lander(self.runtime_state),
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
                if !self.advance_fixed_point_motion() {
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
            if self.advance_fixed_point_motion() {
                return;
            }
            self.position = step_toward(self.position, target.position, behavior.lander_seek_speed);
            return;
        }

        if let Some(player) = prompt.player_position() {
            self.drift = if player.x < self.position.x { -1 } else { 1 };
        }
        if !self.advance_fixed_point_motion() {
            self.drift(behavior);
        }
    }

    fn target_human<'a>(&self, prompt: &'a StepPrompt) -> Option<&'a ActorSnapshot> {
        self.runtime_state
            .and_then(|runtime_state| runtime_state.target_human_index)
            .and_then(|target_slot_index| prompt.target_human(target_slot_index))
    }

    fn drift(&mut self, behavior: ActorBehaviorProfile) {
        self.position = self.position.offset(Velocity::new(
            self.drift * behavior.lander_drift_speed.max(0),
            0,
        ));
    }

    fn advance_fixed_point_motion(&mut self) -> bool {
        let Some(runtime_state) = &mut self.runtime_state else {
            return false;
        };
        if runtime_state.x_velocity == 0 && runtime_state.y_velocity == 0 {
            return false;
        }

        let (x, x_fraction) = step_motion_axis(
            self.position.x,
            runtime_state.x_fraction,
            runtime_state.x_velocity,
        );
        let (y, y_fraction) = step_wrapping_motion_y(
            self.position.y,
            runtime_state.y_fraction,
            runtime_state.y_velocity,
        );
        self.position = Point::new(x, y);
        runtime_state.x_fraction = x_fraction;
        runtime_state.y_fraction = y_fraction;
        self.drift = lander_drift_from_motion_word(runtime_state.x_velocity);
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
                runtime_state: self.mutant_runtime_conversion(prompt),
            }));
            commands.push(GameCommand::PlaySound(SoundCue::MutantSpawn));
        }
    }

    fn mutant_runtime_conversion(&self, prompt: &StepPrompt) -> Option<MutantRuntimeState> {
        let runtime_state = self.runtime_state?;
        let hop_rng = prompt.actor_rng?;
        Some(MutantRuntimeState::from_lander_conversion(
            runtime_state,
            prompt.wave_tuning,
            hop_rng,
        ))
    }

    fn tick_runtime_sleep(&mut self) -> bool {
        if let Some(runtime_state) = &mut self.runtime_state
            && runtime_state.sleep_ticks > 0
        {
            runtime_state.sleep_ticks = runtime_state.sleep_ticks.saturating_sub(1);
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
        let mut runtime_shot_fired = false;
        if let Some(runtime_state) = &mut self.runtime_state {
            if runtime_state.shot_timer > 0 {
                runtime_state.shot_timer = runtime_state.shot_timer.saturating_sub(1);
            }
            if runtime_state.shot_timer == 0 {
                runtime_state.shot_timer = clamped_lander_fire_timer_reset(behavior);
                runtime_shot_fired = true;
            }
        }
        if runtime_shot_fired {
            self.fire_lander_shot(prompt, behavior, commands);
            return;
        }
        if self.runtime_state.is_some() {
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
        let projectile_runtime_state = self.runtime_state.map(|runtime_state| {
            enemy_projectile_runtime_state(
                runtime_state.x_fraction,
                runtime_state.y_fraction,
                velocity,
                behavior.lander_shot_lifetime_steps,
            )
        });
        commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
            position: self.position,
            velocity,
            runtime_state: projectile_runtime_state,
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
        self.runtime_state
            .map(|runtime_state| VisualEffect::LanderSpriteFrame {
                animation_frame: runtime_state.animation_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

const fn lander_drift_from_motion_word(x_velocity: u16) -> i16 {
    drift_from_motion_word(x_velocity)
}

fn lander_spawn_visibility(runtime_state: Option<LanderRuntimeState>) -> LanderSpawnVisibility {
    let Some(runtime_state) = runtime_state else {
        return LanderSpawnVisibility::Normal;
    };
    first_wave_refill_lander_spawn_visibility(runtime_state)
        .unwrap_or(LanderSpawnVisibility::Normal)
}

fn lander_spawn_is_visible(runtime_state: LanderRuntimeState) -> bool {
    first_wave_refill_lander_spawn_visibility(runtime_state)
        == Some(LanderSpawnVisibility::VisibleFirstWaveRefill)
}

fn first_wave_refill_lander_spawn_visibility(
    runtime_state: LanderRuntimeState,
) -> Option<LanderSpawnVisibility> {
    ACTOR_FIRST_WAVE_REFILL_LANDER_SPAWNS
        .iter()
        .copied()
        .filter_map(|spawn| spawn.runtime_state)
        .enumerate()
        .find_map(|(index, refill_runtime_state)| {
            lander_runtime_state_matches_refill_row(runtime_state, refill_runtime_state).then_some(
                if index == 2 {
                    LanderSpawnVisibility::VisibleFirstWaveRefill
                } else {
                    LanderSpawnVisibility::HiddenFirstWaveRefill
                },
            )
        })
}

fn lander_runtime_state_matches_refill_row(
    runtime_state: LanderRuntimeState,
    refill_runtime_state: LanderRuntimeState,
) -> bool {
    runtime_state.x_fraction == refill_runtime_state.x_fraction
        && runtime_state.y_fraction == refill_runtime_state.y_fraction
        && runtime_state.x_velocity == refill_runtime_state.x_velocity
        && runtime_state.y_velocity == refill_runtime_state.y_velocity
        && runtime_state.shot_timer == refill_runtime_state.shot_timer
        && runtime_state.sleep_ticks == refill_runtime_state.sleep_ticks
        && runtime_state.animation_frame == refill_runtime_state.animation_frame
        && runtime_state.target_human_index == refill_runtime_state.target_human_index
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
