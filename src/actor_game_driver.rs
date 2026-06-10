pub struct ActorGameDriver {
    step: u64,
    phase: Phase,
    wave: u16,
    current_player: u8,
    player_count: u8,
    score: u32,
    player_two_score: u32,
    credits: u8,
    lives: u8,
    smart_bombs: u8,
    player_two_lives: u8,
    player_two_smart_bombs: u8,
    next_bonus: u32,
    next_actor_id: u64,
    actors: BTreeMap<ActorId, ThreadedAsset>,
    snapshots: BTreeMap<ActorId, ActorSnapshot>,
    high_scores: HighScoreTable,
    high_score_initials: HighScoreInitialsState,
    attract_script: AttractScript,
    behavior_script: ActorBehaviorScript,
    wave_script: ActorWaveScript,
    wave_spawn_allocations: BTreeMap<ActorKind, usize>,
    enemy_reserve: EnemyReserveSnapshot,
    target_human_cursor: Option<usize>,
    human_walk_cursor: Option<usize>,
    human_walk_sleep_ticks: u8,
    reserve_activation_ready: bool,
    reserve_activation_cooldown_steps: u16,
    first_wave_early_reserve_steps_remaining: Option<u16>,
    first_wave_lander_refill_steps_remaining: Option<u8>,
    background_left: u16,
    baiter_timer_steps: Option<u32>,
    baiter_pacing_steps_remaining: u8,
    actor_rng: ActorRng,
    projectile_scan_steps_remaining: u8,
    pending_smart_bomb_detonation_steps: Option<u8>,
    smart_bomb_flash_steps_remaining: u8,
    pending_sound_commands: Vec<PendingActorSoundCommand>,
    terrain_blow: Option<TerrainBlowSnapshot>,
    free_play_admission: bool,
    player_death_sleep_remaining: Option<u8>,
    game_over_hall_of_fame_stall_remaining: Option<u8>,
    pending_survivor_bonus: Option<PendingSurvivorBonus>,
    pending_player_switch: Option<PendingPlayerSwitch>,
    pending_player_start: Option<PendingPlayerStart>,
    pending_start_sound_steps: Option<u8>,
}

impl ActorGameDriver {
    pub fn new() -> Self {
        Self::with_scripts(ActorDriverScripts::default())
    }

    pub fn with_attract_script(attract_script: AttractScript) -> Self {
        Self::with_attract_and_wave_scripts(attract_script, ActorWaveScript::default())
    }

    pub fn with_wave_script(wave_script: ActorWaveScript) -> Self {
        Self::with_attract_and_wave_scripts(AttractScript::default_title(), wave_script)
    }

    pub fn with_attract_and_wave_scripts(
        attract_script: AttractScript,
        wave_script: ActorWaveScript,
    ) -> Self {
        Self::with_attract_behavior_and_wave_scripts(
            attract_script,
            ActorBehaviorScript::default(),
            wave_script,
        )
    }

    pub fn with_scripts(scripts: ActorDriverScripts) -> Self {
        Self::with_attract_behavior_and_wave_scripts(
            scripts.attract_script,
            scripts.behavior_script,
            scripts.wave_script,
        )
    }

    pub fn with_attract_behavior_and_wave_scripts(
        attract_script: AttractScript,
        behavior_script: ActorBehaviorScript,
        wave_script: ActorWaveScript,
    ) -> Self {
        let mut driver = Self {
            step: 0,
            phase: Phase::Attract,
            wave: 0,
            current_player: 1,
            player_count: 1,
            score: 0,
            player_two_score: 0,
            credits: 0,
            lives: INITIAL_PLAYER_LIVES,
            smart_bombs: 0,
            player_two_lives: INITIAL_PLAYER_LIVES,
            player_two_smart_bombs: 0,
            next_bonus: REPLAY_BONUS_SCORE,
            next_actor_id: 1,
            actors: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            high_scores: HighScoreTable::default(),
            high_score_initials: HighScoreInitialsState::EMPTY,
            attract_script: attract_script.clone(),
            behavior_script,
            wave_script,
            wave_spawn_allocations: BTreeMap::new(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            target_human_cursor: None,
            human_walk_cursor: Some(0),
            human_walk_sleep_ticks: 0,
            reserve_activation_ready: false,
            reserve_activation_cooldown_steps: 0,
            first_wave_early_reserve_steps_remaining: None,
            first_wave_lander_refill_steps_remaining: None,
            background_left: 0,
            baiter_timer_steps: None,
            baiter_pacing_steps_remaining: ACTOR_BAITER_TIMER_PACING_STEPS,
            actor_rng: PLAYFIELD_START_RNG,
            projectile_scan_steps_remaining: ENEMY_PROJECTILE_SCAN_INITIAL_DELAY_STEPS,
            pending_smart_bomb_detonation_steps: None,
            smart_bomb_flash_steps_remaining: 0,
            pending_sound_commands: Vec::new(),
            terrain_blow: None,
            free_play_admission: false,
            player_death_sleep_remaining: None,
            game_over_hall_of_fame_stall_remaining: None,
            pending_survivor_bonus: None,
            pending_player_switch: None,
            pending_player_start: None,
            pending_start_sound_steps: None,
        };
        let attract_id = driver.allocate_actor_id();
        let script_id = driver.allocate_actor_id();
        let status_id = driver.allocate_actor_id();
        driver.spawn_actor(AttractDirector::new(attract_id));
        driver.spawn_actor(ScriptedAttractProgram::new(script_id, attract_script));
        driver.spawn_actor(StatusDisplay::new(status_id));
        driver
    }

    pub fn with_free_play_admission(mut self, enabled: bool) -> Self {
        self.free_play_admission = enabled;
        self
    }

    pub fn set_free_play_admission(&mut self, enabled: bool) {
        self.free_play_admission = enabled;
    }

    pub fn free_play_admission(&self) -> bool {
        self.free_play_admission
    }

    pub fn step(&mut self, input: GameInput) -> StepReport {
        self.step = self.step.saturating_add(1);
        let mut step_commands = Vec::new();
        let mut delayed_sounds = self.advance_pending_start_sound();
        delayed_sounds.extend(self.advance_pending_sound_commands());
        self.advance_smart_bomb_flash();
        delayed_sounds.extend(self.advance_terrain_blow(&mut step_commands));
        let smart_bomb_replay_awarded =
            self.advance_pending_smart_bomb_detonation(&mut step_commands);
        let mut survivor_bonus_awarded_points = None;
        let mut survivor_bonus_replay_awarded = smart_bomb_replay_awarded;
        if let PlayerStartStep::StartPlayfield = self.advance_pending_player_start() {
            step_commands.push(GameCommand::AdvanceWave { wave: self.wave });
            delayed_sounds.push(SoundCue::PlayerAppear);
        }
        self.advance_pending_player_switch();
        self.advance_reserve_activation_cooldown();
        delayed_sounds.extend(self.activate_enemy_reserve_if_ready(&mut step_commands));
        self.advance_first_wave_lander_refill_if_ready(&mut step_commands);
        match self.advance_pending_survivor_bonus() {
            SurvivorBonusStep::StartNextWave => {
                self.start_pending_wave();
                step_commands.push(GameCommand::AdvanceWave { wave: self.wave });
            }
            SurvivorBonusStep::Award(points) => {
                survivor_bonus_awarded_points = Some(points);
                if self.award_points(points) {
                    survivor_bonus_replay_awarded = true;
                }
            }
            SurvivorBonusStep::Waiting => {}
        }
        let pre_applied_command_count = step_commands.len();
        let survivor_bonus_interstitial = self.pending_survivor_bonus.is_some();
        let prompt_player_switch = self.pending_player_switch.map(PendingPlayerSwitch::report);
        let player_switch_interstitial = prompt_player_switch.is_some();
        let prompt_player_start = self.pending_player_start.map(PendingPlayerStart::report);
        let player_start_interstitial = prompt_player_start.is_some();
        let was_playing = self.phase == Phase::Playing;
        let effective_input = if survivor_bonus_interstitial
            || player_switch_interstitial
            || player_start_interstitial
        {
            GameInput::NONE
        } else {
            input
        };
        let high_score_entry_step = self.apply_high_score_entry_input(effective_input);
        let mut behavior_script = self
            .behavior_script
            .with_input_overrides(effective_input, self.snapshots.values().cloned());
        let actor_rng = if self.phase == Phase::Playing
            && !survivor_bonus_interstitial
            && !player_switch_interstitial
            && !player_start_interstitial
        {
            Some(self.actor_rng.advance().snapshot())
        } else {
            None
        };
        if let Some(actor_rng) = actor_rng {
            behavior_script =
                behavior_script.with_hyperspace_seed(actor_rng.hyperspace_seed());
        }
        let human_walk_target_slot = self.advance_human_walk_process(actor_rng);
        let projectile_scan_tick = if self.phase == Phase::Playing
            && !survivor_bonus_interstitial
            && !player_switch_interstitial
            && !player_start_interstitial
        {
            self.advance_projectile_scan_tick()
        } else {
            false
        };
        let prompt_credits = self
            .credits
            .saturating_add(effective_input.coin_insertions());
        let base_prompt = StepPrompt {
            step: self.step,
            phase: self.phase,
            input: effective_input,
            wave: self.wave,
            wave_tuning: self.current_wave_tuning_profile(),
            current_player: self.current_player,
            player_count: self.player_count,
            score: self.active_score(),
            player_scores: self.player_scores(),
            credits: prompt_credits,
            lives: self.active_stock().lives,
            smart_bombs: self.active_stock().smart_bombs,
            smart_bomb_pending: self.pending_smart_bomb_detonation_steps.is_some(),
            player_stocks: self.player_stocks(),
            player_death_sleep_remaining: self.player_death_sleep_remaining,
            game_over_hall_of_fame_stall_remaining: self.game_over_hall_of_fame_stall_remaining,
            player_switch: prompt_player_switch,
            player_start: prompt_player_start,
            high_scores: self.high_scores.entries(),
            high_score_initials: self.high_score_initials,
            snapshots: self.snapshots.values().cloned().collect(),
            behavior_script: behavior_script.clone(),
            background_left: self.background_left,
            actor_rng,
            human_walk_target_slot,
            projectile_scan_tick,
        };

        let mut replies = Vec::new();
        for (id, actor) in &self.actors {
            if let Some(reply) = actor.prompt(base_prompt.clone()) {
                replies.push((*id, reply));
            }
        }
        replies.sort_by_key(|(id, _)| *id);

        let mut draws = Vec::new();
        let mut commands = step_commands;
        let mut dead_actor_ids = Vec::new();
        self.snapshots.clear();
        for (_, reply) in replies {
            if reply.snapshot.alive {
                self.snapshots.insert(reply.id, reply.snapshot);
            } else {
                dead_actor_ids.push(reply.id);
            }
            draws.extend(reply.draws);
            commands.extend(reply.commands);
        }

        self.resolve_collisions(&behavior_script, &mut commands);
        self.advance_baiter_timer(&mut commands);
        let applied_commands = self.apply_commands(&commands[pre_applied_command_count..]);
        self.remove_dead_actors(&dead_actor_ids);
        draws.extend(applied_commands.draws);
        delayed_sounds.extend(applied_commands.sounds);
        survivor_bonus_replay_awarded |= applied_commands.bonus_awarded;
        if self.start_survivor_bonus_if_wave_cleared(was_playing, &commands) {
            commands.push(GameCommand::WaveCleared {
                next_wave: self.wave.saturating_add(1),
            });
            if let SurvivorBonusStep::Award(points) = self.award_initial_survivor_bonus() {
                survivor_bonus_awarded_points = Some(points);
                if self.award_points(points) {
                    survivor_bonus_replay_awarded = true;
                }
            }
        }
        self.schedule_first_wave_lander_refill_if_needed();
        let survivor_bonus = self
            .pending_survivor_bonus
            .map(|bonus| bonus.report(survivor_bonus_awarded_points));
        let player_switch = self.pending_player_switch.map(PendingPlayerSwitch::report);
        let player_start = self.pending_player_start.map(PendingPlayerStart::report);
        if self.phase == Phase::Playing
            && !survivor_bonus_interstitial
            && !player_switch_interstitial
            && !player_start_interstitial
        {
            self.reserve_activation_ready = true;
        }

        let report = StepReport {
            step: self.step,
            phase: self.phase,
            wave: self.wave,
            current_player: self.current_player,
            player_count: self.player_count,
            score: self.active_score(),
            player_scores: self.player_scores(),
            credits: self.credits,
            lives: self.active_stock().lives,
            smart_bombs: self.active_stock().smart_bombs,
            smart_bomb_flash_steps_remaining: self.smart_bomb_flash_steps_remaining,
            player_stocks: self.player_stocks(),
            next_bonus: self.next_bonus,
            player_death_sleep_remaining: self.player_death_sleep_remaining,
            game_over_hall_of_fame_stall_remaining: self.game_over_hall_of_fame_stall_remaining,
            player_switch,
            player_start,
            high_scores: self.high_scores.entries(),
            wave_tuning: self.current_wave_tuning_profile(),
            high_score_initials: self.high_score_initials,
            high_score_initial_accepted: high_score_entry_step.accepted,
            high_score_submitted: high_score_entry_step.submitted,
            bonus_awarded: survivor_bonus_replay_awarded,
            survivor_bonus,
            behavior_script: behavior_script.manifest(),
            enemy_reserve: self.enemy_reserve,
            background_left: self.background_left,
            actor_rng,
            terrain_blow: self.terrain_blow,
            snapshots: self.snapshots.values().cloned().collect(),
            draws,
            sounds: delayed_sounds,
            commands,
        };
        self.advance_game_over_return();
        report
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn wave(&self) -> u16 {
        self.wave
    }

    pub fn actor_count(&self) -> usize {
        self.actors.len()
    }

    pub fn wave_script_name(&self) -> &str {
        self.wave_script.name()
    }

    pub fn behavior_script(&self) -> &ActorBehaviorScript {
        &self.behavior_script
    }

    pub fn behavior_script_mut(&mut self) -> &mut ActorBehaviorScript {
        &mut self.behavior_script
    }

    pub fn script_manifest(&self) -> ActorDriverScriptManifest {
        let current_wave = self.wave.max(1);
        ActorDriverScriptManifest {
            step: self.step,
            phase: self.phase,
            wave: self.wave,
            attract_script: self.attract_script.manifest(),
            behavior_script: self.behavior_script.manifest(),
            wave_script: self.wave_script.manifest(),
            current_wave_profile: self.wave_script.profile_for_wave(current_wave).manifest(),
        }
    }

    pub fn set_default_behavior(&mut self, profile: ActorBehaviorProfile) {
        self.behavior_script.set_default_profile(profile);
    }

    pub fn set_kind_behavior(&mut self, kind: ActorKind, profile: ActorBehaviorProfile) {
        self.behavior_script.set_kind_behavior(kind, profile);
    }

    pub fn set_actor_behavior(&mut self, actor: ActorId, profile: ActorBehaviorProfile) {
        self.behavior_script.set_actor_behavior(actor, profile);
    }

    pub fn snapshot_count(&self, kind: ActorKind) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == kind)
            .count()
    }

    pub fn spawn_lander_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_lander(position)
    }

    pub fn spawn_mutant_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_mutant(position)
    }

    pub fn spawn_bomber_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_bomber(position)
    }

    pub fn spawn_bomb_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_bomb(position, None)
    }

    pub fn spawn_pod_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_pod(position)
    }

    pub fn spawn_swarmer_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_swarmer(position)
    }

    pub fn spawn_baiter_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_baiter(position)
    }

    pub fn set_baiter_timer_for_test(&mut self, timer_steps: u32) {
        self.baiter_timer_steps = Some(timer_steps.max(1));
        self.baiter_pacing_steps_remaining = 1;
    }

    pub fn spawn_human_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_human(position, HumanMode::Grounded)
    }

    pub fn spawn_falling_human_for_test(&mut self, position: Point, velocity: i16) -> ActorId {
        self.spawn_human(position, HumanMode::Falling { velocity })
    }

    pub fn spawn_carried_human_for_test(&mut self, position: Point, carrier: ActorId) -> ActorId {
        self.spawn_human(position, HumanMode::CarriedBy(carrier))
    }

    fn allocate_actor_id(&mut self) -> ActorId {
        let id = ActorId::new(self.next_actor_id);
        self.next_actor_id = self.next_actor_id.saturating_add(1);
        id
    }

    fn spawn_actor(&mut self, actor: impl AssetActor) {
        let id = actor.id();
        self.actors.insert(id, ThreadedAsset::spawn(actor));
    }

    fn spawn_player(&mut self) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(PlayerShip::new(id, Point::new(42, 120)));
        id
    }

    fn spawn_lander(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Lander::new(id, position));
        id
    }

    fn spawn_lander_from_spawn(&mut self, spawn: ActorLanderSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Lander::from_spawn(id, spawn));
        id
    }

    fn spawn_mutant(&mut self, position: Point) -> ActorId {
        self.spawn_mutant_from_spawn(ActorMutantSpawn::new(position))
    }

    fn spawn_mutant_from_spawn(&mut self, spawn: ActorMutantSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Mutant::from_spawn(id, spawn));
        id
    }

    fn spawn_bomber(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Bomber::new(id, position));
        id
    }

    fn spawn_bomber_from_spawn(&mut self, spawn: ActorBomberSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Bomber::from_spawn(id, spawn));
        id
    }

    fn spawn_bomb(
        &mut self,
        position: Point,
        runtime_state: Option<EnemyProjectileRuntimeState>,
    ) -> ActorId {
        let id = self.allocate_actor_id();
        let lifetime = self
            .behavior_script
            .behavior_for(id, ActorKind::Bomb)
            .bomb_lifetime_steps;
        self.spawn_actor(Bomb::new(id, position, lifetime, runtime_state));
        id
    }

    fn spawn_pod(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Pod::new(id, position));
        id
    }

    fn spawn_pod_from_spawn(&mut self, spawn: ActorPodSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Pod::from_spawn(id, spawn));
        id
    }

    fn spawn_swarmer(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Swarmer::new(id, position));
        id
    }

    fn spawn_swarmer_from_spawn(&mut self, spawn: ActorSwarmerSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Swarmer::from_spawn(id, spawn));
        id
    }

    fn spawn_baiter(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Baiter::new(id, position));
        id
    }

    fn spawn_baiter_from_spawn(&mut self, spawn: ActorBaiterSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Baiter::from_spawn(id, spawn));
        id
    }

    fn spawn_human(&mut self, position: Point, mode: HumanMode) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Human::new(id, position, mode));
        id
    }

    fn spawn_human_from_spawn(&mut self, spawn: ActorHumanSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Human::from_spawn(id, spawn));
        id
    }

    fn spawn_laser(&mut self, position: Point, direction: Direction, owner: ActorId) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(LaserShot::new(id, position, direction, owner));
        id
    }

    fn spawn_enemy_laser_from_spawn(
        &mut self,
        position: Point,
        velocity: Velocity,
        runtime_state: Option<EnemyProjectileRuntimeState>,
    ) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(EnemyLaserShot::new(id, position, velocity, runtime_state));
        id
    }

    fn spawn_explosion(&mut self, position: Point, kind: ExplosionKind) -> ActorId {
        self.spawn_explosion_with_anchor(position, kind, None)
    }

    fn spawn_explosion_with_anchor(
        &mut self,
        position: Point,
        kind: ExplosionKind,
        explosion_anchor: Option<Point>,
    ) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Explosion::new(id, position, kind, explosion_anchor));
        id
    }

    fn spawn_score_popup(&mut self, position: Point, points: u32) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(ScorePopup::new(id, position, points));
        id
    }

    fn resolve_collisions(
        &mut self,
        behavior_script: &ActorBehaviorScript,
        commands: &mut Vec<GameCommand>,
    ) {
        if self.phase != Phase::Playing {
            return;
        }

        let bodies = self
            .snapshots
            .values()
            .filter_map(|snapshot| {
                actor_collision_body_for_snapshot(snapshot, self.background_left)
            })
            .collect::<Vec<_>>();
        let mut destroyed = BTreeSet::new();
        for laser in bodies.iter().filter(|body| body.kind == ActorKind::Laser) {
            for enemy in bodies
                .iter()
                .filter(|body| is_player_laser_target(body.kind))
            {
                if laser.bounds.intersects(enemy.bounds) {
                    destroyed.insert(laser.owner);
                    destroyed.insert(enemy.owner);
                    commands.push(GameCommand::Destroy(laser.owner));
                    commands.push(GameCommand::Destroy(enemy.owner));
                    if let Some(kind) = explosion_kind_for_target(enemy.kind) {
                        commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                            position: center_of(enemy.bounds),
                            kind,
                            explosion_anchor: None,
                        }));
                    }
                    commands.push(GameCommand::AddScore(score_for_hostile(enemy.kind)));
                    commands.push(GameCommand::PlaySound(hit_sound_for_hostile(enemy.kind)));
                    if enemy.kind == ActorKind::Pod {
                        commands.extend(self.pod_swarmer_spawn_commands(center_of(enemy.bounds)));
                    }
                    break;
                }
            }
        }

        let Some(player) = bodies.iter().find(|body| body.kind == ActorKind::Player) else {
            return;
        };
        let player_behavior = behavior_script.behavior_for(player.owner, ActorKind::Player);
        if !player_behavior.player_takes_enemy_collision_damage {
            return;
        }
        let hyperspace_clears_enemy_lasers = commands
            .iter()
            .any(|command| matches!(command, GameCommand::Hyperspace));
        for enemy in bodies.iter().filter(|body| is_player_hazard(body.kind)) {
            if destroyed.contains(&enemy.owner) {
                continue;
            }
            if hyperspace_clears_enemy_lasers && enemy.kind == ActorKind::EnemyLaser {
                continue;
            }
            if mutant_dive_collision_window_pending(
                enemy.position,
                enemy.runtime.as_mutant(),
            ) {
                continue;
            }
            if player.bounds.intersects(enemy.bounds) {
                commands.push(GameCommand::Destroy(player.owner));
                commands.push(GameCommand::Destroy(enemy.owner));
                if is_player_enemy_collision_target(enemy.kind) {
                    if let Some(kind) = explosion_kind_for_target(enemy.kind) {
                        let placement = actor_player_enemy_collision_explosion_placement(enemy);
                        commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                            position: placement.position,
                            kind,
                            explosion_anchor: placement.explosion_anchor,
                        }));
                    }
                    commands.push(GameCommand::AddScore(score_for_hostile(enemy.kind)));
                    commands.push(GameCommand::PlaySound(hit_sound_for_hostile(enemy.kind)));
                }
                commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                    position: center_of(player.bounds),
                    kind: player_hazard_explosion_kind(enemy.kind),
                    explosion_anchor: None,
                }));
                commands.push(GameCommand::PlaySound(player_hazard_sound(enemy.kind)));
                commands.push(GameCommand::PlayerKilled);
                break;
            }
        }
    }

    fn apply_commands(&mut self, commands: &[GameCommand]) -> AppliedCommands {
        let mut applied = AppliedCommands::default();
        let mut active_enemy_projectiles = self.active_enemy_projectile_count();
        let mut active_bomb_projectiles = self.active_bomb_projectile_count();
        let mut removed_enemy_projectiles = BTreeSet::new();
        let mut removed_bomb_projectiles = BTreeSet::new();
        let mut terrain_blow_started_this_batch = false;
        for command in commands {
            match *command {
                GameCommand::Credit => {
                    self.credits = self.credits.saturating_add(1);
                    applied.sounds.push(SoundCue::Credit);
                }
                GameCommand::StartOnePlayer => {
                    if self.start_admitted(1) {
                        self.consume_start_credits(1);
                        self.start_play(1);
                        self.pending_start_sound_steps = Some(PLAYER_START_SOUND_DELAY_STEPS);
                    }
                }
                GameCommand::StartTwoPlayer => {
                    if self.start_admitted(2) {
                        self.consume_start_credits(2);
                        self.start_play(2);
                        self.pending_start_sound_steps = Some(PLAYER_START_SOUND_DELAY_STEPS);
                    }
                }
                GameCommand::Spawn(SpawnRequest::Laser {
                    position,
                    direction,
                    owner,
                }) => {
                    self.spawn_laser(position, direction, owner);
                }
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position,
                    velocity,
                    runtime_state,
                }) => {
                    if enemy_projectile_spawn_in_bounds(position)
                        && reserve_enemy_projectile_slot(&mut active_enemy_projectiles)
                    {
                        self.spawn_enemy_laser_from_spawn(position, velocity, runtime_state);
                    }
                }
                GameCommand::Spawn(SpawnRequest::Lander { position }) => {
                    let actor = self.spawn_lander(position);
                    self.apply_next_wave_spawn_behavior(ActorKind::Lander, actor);
                }
                GameCommand::Spawn(SpawnRequest::Mutant {
                    position,
                    runtime_state,
                }) => {
                    let actor = self.spawn_mutant_from_spawn(ActorMutantSpawn {
                        position,
                        runtime_state,
                    });
                    self.apply_next_wave_spawn_behavior(ActorKind::Mutant, actor);
                }
                GameCommand::Spawn(SpawnRequest::Bomber { position }) => {
                    let actor = self.spawn_bomber(position);
                    self.apply_next_wave_spawn_behavior(ActorKind::Bomber, actor);
                }
                GameCommand::Spawn(SpawnRequest::Bomb {
                    position,
                    runtime_state,
                }) => {
                    if bomb_projectile_spawn_in_world_bounds(position, runtime_state)
                        && enemy_projectile_slot_available(active_enemy_projectiles)
                        && bomb_projectile_slot_available(active_bomb_projectiles)
                    {
                        active_enemy_projectiles += 1;
                        active_bomb_projectiles += 1;
                        self.spawn_bomb(position, runtime_state);
                    }
                }
                GameCommand::Spawn(SpawnRequest::Pod { position }) => {
                    let actor = self.spawn_pod(position);
                    self.apply_next_wave_spawn_behavior(ActorKind::Pod, actor);
                }
                GameCommand::Spawn(SpawnRequest::Swarmer {
                    position,
                    runtime_state,
                }) => {
                    let actor = self.spawn_swarmer_from_spawn(ActorSwarmerSpawn {
                        position,
                        runtime_state,
                    });
                    self.apply_next_wave_spawn_behavior(ActorKind::Swarmer, actor);
                }
                GameCommand::Spawn(SpawnRequest::Baiter {
                    position,
                    runtime_state,
                }) => {
                    let actor = self.spawn_baiter_from_spawn(ActorBaiterSpawn {
                        position,
                        runtime_state,
                    });
                    self.apply_next_wave_spawn_behavior(ActorKind::Baiter, actor);
                }
                GameCommand::Spawn(SpawnRequest::Human { position, mode }) => {
                    let actor = self.spawn_human(position, mode);
                    self.apply_next_wave_spawn_behavior(ActorKind::Human, actor);
                }
                GameCommand::Spawn(SpawnRequest::Explosion {
                    position,
                    kind,
                    explosion_anchor,
                }) => {
                    self.spawn_explosion_with_anchor(position, kind, explosion_anchor);
                }
                GameCommand::Spawn(SpawnRequest::ScorePopup { position, points }) => {
                    self.spawn_score_popup(position, points);
                }
                GameCommand::Destroy(id) => {
                    let removed_kind = self.snapshots.get(&id).map(|snapshot| snapshot.kind);
                    if let Some(snapshot) = self.snapshots.get(&id) {
                        if removed_enemy_projectiles.insert(id) && is_enemy_projectile_kind(snapshot.kind) {
                            active_enemy_projectiles = active_enemy_projectiles.saturating_sub(1);
                        }
                        if removed_bomb_projectiles.insert(id) && snapshot.kind == ActorKind::Bomb {
                            active_bomb_projectiles = active_bomb_projectiles.saturating_sub(1);
                        }
                    }
                    self.snapshots.remove(&id);
                    self.actors.remove(&id);
                    self.behavior_script.remove_actor_behavior(id);
                    if removed_kind == Some(ActorKind::Human) {
                        let draws = self.start_terrain_blow_if_no_humans();
                        terrain_blow_started_this_batch |= !draws.is_empty();
                        applied.draws.extend(draws);
                    }
                }
                GameCommand::SetWorldScrollLeft(background_left) => {
                    self.background_left = background_left;
                }
                GameCommand::AttachHuman {
                    lander,
                    human,
                    position,
                } => {
                    let runtime_state = self
                        .snapshots
                        .get(&human)
                        .and_then(|snapshot| snapshot.runtime.as_human());
                    self.snapshots.remove(&human);
                    self.actors.remove(&human);
                    self.spawn_actor(Human::with_runtime_state(
                        human,
                        position,
                        HumanMode::CarriedBy(lander),
                        runtime_state,
                    ));
                }
                GameCommand::SmartBomb { consume_stock } => {
                    self.start_smart_bomb(consume_stock);
                }
                GameCommand::Hyperspace => {
                    self.clear_enemy_projectiles_for_hyperspace();
                }
                GameCommand::HumanLost(id) => {
                    self.snapshots.remove(&id);
                    self.actors.remove(&id);
                    self.behavior_script.remove_actor_behavior(id);
                    let draws = self.start_terrain_blow_if_no_humans();
                    terrain_blow_started_this_batch |= !draws.is_empty();
                    applied.draws.extend(draws);
                }
                GameCommand::AddScore(points) => {
                    if self.award_points(points) {
                        applied.bonus_awarded = true;
                    }
                }
                GameCommand::PlaySound(sound) => {
                    if !(terrain_blow_started_this_batch && sound == SoundCue::HumanLost) {
                        applied.sounds.push(sound);
                        if sound == SoundCue::HumanRescued {
                            self.queue_astronaut_rescue_sound_tail();
                        }
                    }
                }
                GameCommand::PlayerKilled => {
                    self.lose_player_life(&mut applied.sounds);
                }
                GameCommand::WaveCleared { .. } => {}
                GameCommand::AdvanceWave { .. } => {}
                GameCommand::EnterGameOver => {
                    self.enter_game_over(&mut applied.sounds);
                }
            }
        }
        applied
    }

    fn start_admitted(&self, player_count: u8) -> bool {
        self.phase == Phase::Attract
            && (self.free_play_admission || self.credits >= player_count.clamp(1, 2))
    }

    fn consume_start_credits(&mut self, player_count: u8) {
        if self.free_play_admission {
            return;
        }

        self.credits = self.credits.saturating_sub(player_count.clamp(1, 2));
    }

    fn active_enemy_projectile_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| is_enemy_projectile_kind(snapshot.kind) && snapshot.alive)
            .count()
    }

    fn active_bomb_projectile_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomb && snapshot.alive)
            .count()
    }

    fn active_score(&self) -> u32 {
        if self.current_player == 2 {
            self.player_two_score
        } else {
            self.score
        }
    }

    fn player_scores(&self) -> [u32; 2] {
        [self.score, self.player_two_score]
    }

    fn active_stock(&self) -> PlayerStock {
        self.stock_for_player(self.current_player)
    }

    fn stock_for_player(&self, player: u8) -> PlayerStock {
        if player == 2 {
            PlayerStock::new(self.player_two_lives, self.player_two_smart_bombs)
        } else {
            PlayerStock::new(self.lives, self.smart_bombs)
        }
    }

    fn set_active_stock(&mut self, stock: PlayerStock) {
        self.set_stock_for_player(self.current_player, stock);
    }

    fn set_stock_for_player(&mut self, player: u8, stock: PlayerStock) {
        if player == 2 {
            self.player_two_lives = stock.lives;
            self.player_two_smart_bombs = stock.smart_bombs;
        } else {
            self.lives = stock.lives;
            self.smart_bombs = stock.smart_bombs;
        }
    }

    fn player_stocks(&self) -> [PlayerStockSnapshot; 2] {
        [
            PlayerStockSnapshot::new(self.lives, self.smart_bombs),
            PlayerStockSnapshot::new(self.player_two_lives, self.player_two_smart_bombs),
        ]
    }

    fn highest_visible_score(&self) -> u32 {
        self.high_scores.entries()[0]
            .max(self.score)
            .max(self.player_two_score)
    }

    fn lose_player_life(&mut self, sounds: &mut Vec<SoundCue>) {
        let from_player = self.current_player;
        let mut stock = self.active_stock();
        stock.lives = stock.lives.saturating_sub(1);
        self.set_active_stock(stock);

        if let Some(to_player) = self.next_other_stocked_player(from_player) {
            self.begin_player_switch(from_player, to_player);
            return;
        }

        if stock.lives > 0 {
            self.spawn_player();
            return;
        }

        self.enter_game_over(sounds);
    }

    fn next_other_stocked_player(&self, from_player: u8) -> Option<u8> {
        let player_count = self.player_count.clamp(1, 2);
        for offset in 1..=player_count {
            let candidate = ((from_player.saturating_sub(1) + offset) % player_count) + 1;
            if candidate != from_player && self.stock_for_player(candidate).lives > 0 {
                return Some(candidate);
            }
        }
        None
    }

    fn begin_player_switch(&mut self, from_player: u8, to_player: u8) {
        self.phase = Phase::GameOver;
        self.player_death_sleep_remaining = None;
        self.game_over_hall_of_fame_stall_remaining = None;
        self.pending_survivor_bonus = None;
        self.pending_player_switch = Some(PendingPlayerSwitch::new(from_player, to_player));
        self.pending_player_start = None;
        self.pending_start_sound_steps = None;
        self.clear_pending_smart_bomb();
        self.clear_terrain_blow();
        self.clear_pending_astronaut_rescue();
        self.clear_first_wave_lander_refill();
        self.enemy_reserve = EnemyReserveSnapshot::default();
        self.target_human_cursor = None;
        self.reserve_activation_ready = false;
        self.reserve_activation_cooldown_steps = 0;
        self.first_wave_early_reserve_steps_remaining = None;
        self.background_left = 0;
        self.baiter_timer_steps = None;
        self.clear_turn_playfield_actors();
    }

    fn advance_pending_player_switch(&mut self) {
        let Some(mut pending) = self.pending_player_switch else {
            return;
        };

        pending.sleep_steps_remaining = pending.sleep_steps_remaining.saturating_sub(1);
        if pending.sleep_steps_remaining > 0 {
            self.pending_player_switch = Some(pending);
            return;
        }

        self.pending_player_switch = None;
        self.start_next_player_turn(pending.to_player);
    }

    fn start_next_player_turn(&mut self, player: u8) {
        let player = player.clamp(1, self.player_count.clamp(1, 2));
        self.phase = Phase::Playing;
        self.current_player = player;
        self.wave = 1;
        self.high_score_initials = HighScoreInitialsState::EMPTY;
        self.player_death_sleep_remaining = None;
        self.game_over_hall_of_fame_stall_remaining = None;
        self.pending_survivor_bonus = None;
        self.pending_player_switch = None;
        self.pending_player_start = Some(PendingPlayerStart::new(player));
        self.pending_start_sound_steps = None;
        self.clear_pending_smart_bomb();
        self.clear_terrain_blow();
        self.clear_pending_astronaut_rescue();
        self.clear_first_wave_lander_refill();
        self.actor_rng = PLAYFIELD_START_RNG;
        self.background_left = 0;
        self.reserve_activation_cooldown_steps = 0;
        self.first_wave_early_reserve_steps_remaining = None;
        self.reset_enemy_projectile_scan();
        self.clear_turn_playfield_actors();
        self.apply_wave_profile();
    }

    fn advance_pending_start_sound(&mut self) -> Vec<SoundCue> {
        let Some(mut remaining) = self.pending_start_sound_steps else {
            return Vec::new();
        };

        remaining = remaining.saturating_sub(1);
        if remaining > 0 {
            self.pending_start_sound_steps = Some(remaining);
            Vec::new()
        } else {
            self.pending_start_sound_steps = None;
            vec![SoundCue::Start]
        }
    }

    fn advance_pending_sound_commands(&mut self) -> Vec<SoundCue> {
        let mut sounds = Vec::new();
        let mut pending = Vec::new();
        for mut command in self.pending_sound_commands.drain(..) {
            command.steps_remaining = command.steps_remaining.saturating_sub(1);
            if command.steps_remaining == 0 {
                sounds.push(SoundCue::SoundBoardCommand(command.command));
            } else {
                pending.push(command);
            }
        }
        self.pending_sound_commands = pending;
        sounds
    }

    fn queue_smart_bomb_sound_sequence(&mut self) {
        self.pending_sound_commands
            .extend(
                SMART_BOMB_SOUND_SEQUENCE
                    .iter()
                    .copied()
                    .map(|(steps_remaining, command)| PendingActorSoundCommand {
                        steps_remaining,
                        command,
                        trigger: PendingActorSoundTrigger::SmartBomb,
                    }),
            );
    }

    fn queue_terrain_blow_sound_tail(&mut self) {
        self.pending_sound_commands
            .extend(TERRAIN_BLOW_SOUND_TAIL_SEQUENCE.iter().copied().map(
                |(steps_remaining, command)| PendingActorSoundCommand {
                    steps_remaining,
                    command,
                    trigger: PendingActorSoundTrigger::TerrainBlow,
                },
            ));
    }

    fn queue_astronaut_rescue_sound_tail(&mut self) {
        self.pending_sound_commands.extend(
            ASTRONAUT_CATCH_SOUND_TAIL_SEQUENCE.iter().copied().map(
                |(steps_remaining, command)| PendingActorSoundCommand {
                    steps_remaining,
                    command,
                    trigger: PendingActorSoundTrigger::AstronautRescue,
                },
            ),
        );
    }

    fn queue_first_wave_lander_refill_appearance_sound(&mut self) {
        self.pending_sound_commands.push(PendingActorSoundCommand {
            steps_remaining: FIRST_WAVE_LANDER_REFILL_APPEAR_SOUND_DELAY_STEPS,
            command: APPEARANCE_SOUND_COMMAND,
            trigger: PendingActorSoundTrigger::FirstWaveLanderRefill,
        });
    }

    fn advance_smart_bomb_flash(&mut self) {
        self.smart_bomb_flash_steps_remaining =
            self.smart_bomb_flash_steps_remaining.saturating_sub(1);
    }

    fn start_terrain_blow_if_no_humans(&mut self) -> Vec<DrawCommand> {
        if self.terrain_blow.is_some()
            || self
                .snapshots
                .values()
                .any(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
        {
            return Vec::new();
        }

        self.terrain_blow = Some(TerrainBlowSnapshot::started());
        self.spawn_terrain_blow_explosion_births(0)
    }

    fn advance_terrain_blow(&mut self, commands: &mut Vec<GameCommand>) -> Vec<SoundCue> {
        let Some(terrain_blow) = self.terrain_blow else {
            return Vec::new();
        };
        if terrain_blow.stage == TerrainBlowStage::Completed {
            return Vec::new();
        }

        let elapsed = terrain_blow.elapsed_ticks.saturating_add(1);
        for draw in self.spawn_terrain_blow_explosion_births(elapsed) {
            commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                position: draw.position,
                kind: ExplosionKind::Terrain,
                explosion_anchor: None,
            }));
        }

        let mut sounds = Vec::new();
        if elapsed >= TERRAIN_BLOW_COMPLETE_STEP {
            if let Some(terrain_blow) = self.terrain_blow.as_mut() {
                terrain_blow.stage = TerrainBlowStage::Completed;
                terrain_blow.elapsed_ticks = elapsed;
                terrain_blow.explosion_pass = terrain_blow.explosion_pass_limit;
                terrain_blow.sleep_ticks_remaining = None;
                terrain_blow.flash_color_byte = 0;
            }
            sounds.push(SoundCue::SoundBoardCommand(TERRAIN_BLOW_SOUND_COMMAND));
            self.queue_terrain_blow_sound_tail();
            return sounds;
        }

        let start_sound_index = TERRAIN_BLOW_START_SOUND_STEPS
            .iter()
            .position(|step| *step == elapsed);
        let next_step = TERRAIN_BLOW_START_SOUND_STEPS
            .iter()
            .copied()
            .find(|step| *step > elapsed)
            .unwrap_or(TERRAIN_BLOW_COMPLETE_STEP);
        if let Some(terrain_blow) = self.terrain_blow.as_mut() {
            terrain_blow.elapsed_ticks = elapsed;
            terrain_blow.sleep_ticks_remaining = u8::try_from(next_step - elapsed).ok();
            terrain_blow.overload_counter = TERRAIN_BLOW_OVERLOAD_COUNTER;
            if let Some(start_sound_index) = start_sound_index {
                terrain_blow.stage = TerrainBlowStage::ExplosionPassSleeping;
                terrain_blow.explosion_pass = terrain_blow.explosion_pass.saturating_add(1);
                terrain_blow.flash_color_byte =
                    TERRAIN_BLOW_FLASH_COLOR_BYTES[start_sound_index];
            } else {
                terrain_blow.stage = TerrainBlowStage::FlashClearedSleeping;
                terrain_blow.flash_color_byte = 0;
            }
        }
        if start_sound_index.is_some() {
            sounds.push(SoundCue::SoundBoardCommand(SMART_BOMB_SOUND_COMMAND));
        }
        sounds
    }

    fn spawn_terrain_blow_explosion_births(&mut self, elapsed: u16) -> Vec<DrawCommand> {
        TERRAIN_BLOW_EXPLOSION_BIRTHS
            .iter()
            .copied()
            .filter(|(birth_step, _)| *birth_step == elapsed)
            .map(|(_, screen)| {
                let position = Point::new(i16::from(screen.x), i16::from(screen.y));
                let id = self.spawn_explosion(position, ExplosionKind::Terrain);
                DrawCommand::sprite_with_effect(
                    id,
                    SpriteKey::Explosion,
                    position,
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Terrain,
                        age: 0,
                        explosion_anchor: None,
                    },
                )
            })
            .collect()
    }

    fn clear_pending_smart_bomb(&mut self) {
        self.pending_smart_bomb_detonation_steps = None;
        self.smart_bomb_flash_steps_remaining = 0;
        self.pending_sound_commands
            .retain(|command| command.trigger != PendingActorSoundTrigger::SmartBomb);
    }

    fn clear_terrain_blow(&mut self) {
        self.terrain_blow = None;
        self.pending_sound_commands
            .retain(|command| command.trigger != PendingActorSoundTrigger::TerrainBlow);
    }

    fn clear_pending_astronaut_rescue(&mut self) {
        self.pending_sound_commands
            .retain(|command| command.trigger != PendingActorSoundTrigger::AstronautRescue);
    }

    fn clear_first_wave_lander_refill(&mut self) {
        self.first_wave_lander_refill_steps_remaining = None;
        self.pending_sound_commands
            .retain(|command| command.trigger != PendingActorSoundTrigger::FirstWaveLanderRefill);
    }

    fn start_smart_bomb(&mut self, consume_stock: bool) -> bool {
        if self.pending_smart_bomb_detonation_steps.is_some() {
            return false;
        }

        if consume_stock {
            let mut stock = self.active_stock();
            if stock.smart_bombs == 0 {
                return false;
            }
            stock.smart_bombs = stock.smart_bombs.saturating_sub(1);
            self.set_active_stock(stock);
        }

        self.pending_smart_bomb_detonation_steps = Some(SMART_BOMB_DETONATION_DELAY_STEPS);
        self.reserve_activation_cooldown_steps = SMART_BOMB_RESERVE_DELAY_STEPS;
        self.first_wave_early_reserve_steps_remaining = None;
        self.clear_first_wave_lander_refill();
        self.queue_smart_bomb_sound_sequence();
        true
    }

    fn advance_pending_smart_bomb_detonation(&mut self, commands: &mut Vec<GameCommand>) -> bool {
        let Some(mut remaining) = self.pending_smart_bomb_detonation_steps else {
            return false;
        };

        remaining = remaining.saturating_sub(1);
        if remaining > 0 {
            self.pending_smart_bomb_detonation_steps = Some(remaining);
            return false;
        }

        self.pending_smart_bomb_detonation_steps = None;
        self.smart_bomb_flash_steps_remaining = SMART_BOMB_FLASH_STEPS;
        self.detonate_smart_bomb_targets(commands)
    }

    fn advance_pending_player_start(&mut self) -> PlayerStartStep {
        let Some(mut pending) = self.pending_player_start else {
            return PlayerStartStep::Waiting;
        };

        pending.delay_steps_remaining = pending.delay_steps_remaining.saturating_sub(1);
        if pending.delay_steps_remaining > 0 {
            self.pending_player_start = Some(pending);
            return PlayerStartStep::Waiting;
        }

        self.pending_player_start = None;
        self.start_delayed_playfield();
        PlayerStartStep::StartPlayfield
    }

    fn start_delayed_playfield(&mut self) {
        self.clear_turn_playfield_actors();
        self.apply_wave_profile();
        self.spawn_player();
        self.spawn_wave_hostiles();
        self.spawn_initial_humans();
        self.arm_first_wave_early_lander_reserve_delay();
    }

    fn enter_game_over(&mut self, sounds: &mut Vec<SoundCue>) {
        let active_score = self.active_score();
        self.set_active_stock(PlayerStock::new(0, 0));
        self.wave = 0;
        self.actor_rng = PLAYFIELD_START_RNG;
        self.background_left = 0;
        self.reset_enemy_projectile_scan();
        self.high_score_initials = HighScoreInitialsState::EMPTY;
        self.player_death_sleep_remaining = None;
        self.game_over_hall_of_fame_stall_remaining = None;
        self.pending_survivor_bonus = None;
        self.pending_player_switch = None;
        self.pending_player_start = None;
        self.pending_start_sound_steps = None;
        self.clear_pending_smart_bomb();
        self.clear_terrain_blow();
        self.clear_pending_astronaut_rescue();
        self.clear_first_wave_lander_refill();
        self.enemy_reserve = EnemyReserveSnapshot::default();
        self.target_human_cursor = None;
        self.reserve_activation_ready = false;
        self.reserve_activation_cooldown_steps = 0;
        self.first_wave_early_reserve_steps_remaining = None;
        self.high_scores.record(active_score);
        self.phase = if self.high_scores.qualifies(active_score) {
            Phase::HighScoreEntry
        } else {
            self.player_death_sleep_remaining = Some(FINAL_GAME_OVER_DELAY_STEPS);
            Phase::GameOver
        };
        self.baiter_timer_steps = None;
        sounds.push(SoundCue::GameOver);
    }

    fn apply_high_score_entry_input(&mut self, input: GameInput) -> HighScoreEntryStep {
        if self.phase != Phase::HighScoreEntry {
            return HighScoreEntryStep::default();
        }

        let entry_step = HighScoreEntrySystem::enter_initial(
            self.high_score_initials,
            input.high_score_initial,
            input.high_score_backspace,
        );
        self.high_score_initials = entry_step.state;
        if entry_step.submitted {
            self.phase = Phase::GameOver;
            self.player_death_sleep_remaining = None;
            self.game_over_hall_of_fame_stall_remaining = Some(HIGH_SCORE_HALL_STALL_STEPS);
        }
        HighScoreEntryStep {
            accepted: entry_step.accepted,
            submitted: entry_step.submitted,
        }
    }

    fn start_play(&mut self, player_count: u8) {
        let player_count = player_count.clamp(1, 2);
        self.phase = Phase::Playing;
        self.current_player = 1;
        self.player_count = player_count;
        self.wave = 1;
        self.score = 0;
        self.player_two_score = 0;
        self.lives = INITIAL_PLAYER_LIVES;
        self.smart_bombs = INITIAL_SMART_BOMBS;
        self.player_two_lives = INITIAL_PLAYER_LIVES;
        self.player_two_smart_bombs = INITIAL_SMART_BOMBS;
        self.next_bonus = REPLAY_BONUS_SCORE;
        self.high_score_initials = HighScoreInitialsState::EMPTY;
        self.player_death_sleep_remaining = None;
        self.game_over_hall_of_fame_stall_remaining = None;
        self.pending_survivor_bonus = None;
        self.pending_player_switch = None;
        self.pending_player_start = None;
        self.pending_start_sound_steps = None;
        self.clear_pending_smart_bomb();
        self.clear_terrain_blow();
        self.clear_pending_astronaut_rescue();
        self.clear_first_wave_lander_refill();
        self.enemy_reserve = EnemyReserveSnapshot::default();
        self.target_human_cursor = None;
        self.reserve_activation_ready = false;
        self.reserve_activation_cooldown_steps = 0;
        self.first_wave_early_reserve_steps_remaining = None;
        self.actor_rng = PLAYFIELD_START_RNG;
        self.background_left = 0;
        self.reset_enemy_projectile_scan();
        self.clear_turn_playfield_actors();
        self.apply_wave_profile();
        self.pending_player_start = Some(PendingPlayerStart::new(self.current_player));
    }

    fn start_pending_wave(&mut self) {
        self.wave = self.wave.saturating_add(1);
        self.player_death_sleep_remaining = None;
        self.pending_survivor_bonus = None;
        self.pending_player_switch = None;
        self.pending_player_start = None;
        self.pending_start_sound_steps = None;
        self.clear_pending_smart_bomb();
        self.clear_terrain_blow();
        self.clear_pending_astronaut_rescue();
        self.clear_first_wave_lander_refill();
        self.clear_wave_playfield_actors();
        self.apply_wave_profile();
        self.reserve_activation_cooldown_steps = 0;
        self.spawn_wave_hostiles();
        self.spawn_initial_humans();
        self.arm_first_wave_early_lander_reserve_delay();
    }

    fn advance_pending_survivor_bonus(&mut self) -> SurvivorBonusStep {
        let Some(mut bonus) = self.pending_survivor_bonus else {
            return SurvivorBonusStep::Waiting;
        };

        if let Some(wave_sleep) = bonus.wave_advance_sleep_steps_remaining {
            let next_sleep = wave_sleep.saturating_sub(1);
            if next_sleep == 0 {
                self.pending_survivor_bonus = None;
                return SurvivorBonusStep::StartNextWave;
            }
            bonus.wave_advance_sleep_steps_remaining = Some(next_sleep);
            self.pending_survivor_bonus = Some(bonus);
            return SurvivorBonusStep::Waiting;
        }

        if bonus.astronaut_sleep_steps_remaining > 0 {
            bonus.astronaut_sleep_steps_remaining =
                bonus.astronaut_sleep_steps_remaining.saturating_sub(1);
            if bonus.astronaut_sleep_steps_remaining > 0 {
                self.pending_survivor_bonus = Some(bonus);
                return SurvivorBonusStep::Waiting;
            }
        }

        if let Some(points) = bonus.award_next_survivor() {
            self.pending_survivor_bonus = Some(bonus);
            return SurvivorBonusStep::Award(points);
        }

        bonus.wave_advance_sleep_steps_remaining = Some(SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS);
        self.pending_survivor_bonus = Some(bonus);
        SurvivorBonusStep::Waiting
    }

    fn award_initial_survivor_bonus(&mut self) -> SurvivorBonusStep {
        let Some(mut bonus) = self.pending_survivor_bonus else {
            return SurvivorBonusStep::Waiting;
        };

        if let Some(points) = bonus.award_next_survivor() {
            self.pending_survivor_bonus = Some(bonus);
            return SurvivorBonusStep::Award(points);
        }

        bonus.wave_advance_sleep_steps_remaining = Some(SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS);
        self.pending_survivor_bonus = Some(bonus);
        SurvivorBonusStep::Waiting
    }

    fn reset_enemy_projectile_scan(&mut self) {
        self.projectile_scan_steps_remaining = ENEMY_PROJECTILE_SCAN_INITIAL_DELAY_STEPS;
    }

    fn reset_human_walk_process(&mut self) {
        self.human_walk_cursor = Some(0);
        self.human_walk_sleep_ticks = 0;
    }

    fn advance_human_walk_process(
        &mut self,
        actor_rng: Option<ActorRngSnapshot>,
    ) -> Option<usize> {
        if actor_rng.is_none() || !self.has_targetable_human_snapshots() {
            return None;
        }
        if self.human_walk_sleep_ticks > 0 {
            self.human_walk_sleep_ticks = self.human_walk_sleep_ticks.saturating_sub(1);
            return None;
        }

        let current_cursor = self
            .human_walk_cursor
            .filter(|slot| *slot < ASTRONAUT_TARGET_CURSOR_ENTRY_COUNT)
            .unwrap_or(0);
        let next_cursor = next_astronaut_target_slot_index(current_cursor);
        self.human_walk_cursor = Some(next_cursor);
        self.human_walk_sleep_ticks = ASTRONAUT_PROCESS_SLEEP_TICKS;

        let human_count = self.human_snapshot_count();
        self.snapshots.values().find_map(|snapshot| {
            let runtime_state = snapshot.runtime.as_human()?;
            (runtime_state.target_slot_index == next_cursor
                && actor_human_walk_targetable(human_count, snapshot))
            .then_some(next_cursor)
        })
    }

    fn advance_reserve_activation_cooldown(&mut self) {
        self.reserve_activation_cooldown_steps = self
            .reserve_activation_cooldown_steps
            .saturating_sub(1);
    }

    fn advance_game_over_return(&mut self) {
        if self.phase != Phase::GameOver {
            return;
        }
        if let Some(remaining) = self.player_death_sleep_remaining {
            let next = remaining.saturating_sub(1);
            if next > 0 {
                self.player_death_sleep_remaining = Some(next);
                return;
            }

            self.player_death_sleep_remaining = None;
            self.phase = Phase::Attract;
            return;
        }

        let Some(remaining) = self.game_over_hall_of_fame_stall_remaining else {
            return;
        };

        let next = remaining.saturating_sub(1);
        if next > 0 {
            self.game_over_hall_of_fame_stall_remaining = Some(next);
            return;
        }

        self.game_over_hall_of_fame_stall_remaining = None;
        self.phase = Phase::Attract;
    }

    fn advance_projectile_scan_tick(&mut self) -> bool {
        if self.projectile_scan_steps_remaining > 0 {
            self.projectile_scan_steps_remaining =
                self.projectile_scan_steps_remaining.saturating_sub(1);
            return false;
        }

        self.projectile_scan_steps_remaining = ENEMY_PROJECTILE_SCAN_CADENCE_STEPS - 1;
        true
    }

    fn apply_wave_profile(&mut self) {
        let wave_profile = self.wave_script.profile_for_wave(self.wave);
        self.behavior_script = wave_profile.behavior_script.clone();
        self.wave_spawn_allocations.clear();
        self.enemy_reserve = wave_profile.enemy_reserve;
        self.target_human_cursor = Some(0);
        self.reset_human_walk_process();
        self.reserve_activation_ready = false;
        self.reserve_activation_cooldown_steps = 0;
        self.first_wave_early_reserve_steps_remaining = None;
        self.clear_first_wave_lander_refill();
        if self.phase == Phase::Playing {
            self.reset_baiter_timer();
        }
    }

    fn current_wave_tuning_profile(&self) -> ActorWaveTuning {
        self.wave_script
            .profile_for_wave(self.wave)
            .wave_tuning
            .unwrap_or_else(|| ActorWaveTuning::for_wave(self.wave.max(1)))
    }

    fn reset_baiter_timer(&mut self) {
        let wave_tuning_profile = self.current_wave_tuning_profile();
        self.baiter_timer_steps = Some(wave_tuning_profile.baiter_delay.max(1));
        self.baiter_pacing_steps_remaining = ACTOR_BAITER_TIMER_PACING_STEPS;
    }

    fn activate_enemy_reserve_if_ready(
        &mut self,
        commands: &mut Vec<GameCommand>,
    ) -> Vec<SoundCue> {
        if self.phase != Phase::Playing
            || self.wave == 0
            || self.pending_survivor_bonus.is_some()
            || self.pending_player_switch.is_some()
            || self.pending_player_start.is_some()
            || !self.reserve_activation_ready
            || self.reserve_activation_cooldown_steps > 0
            || actor_enemy_reserve_is_empty(self.enemy_reserve)
        {
            return Vec::new();
        }

        if self.has_hostile_snapshots() {
            if self.activate_first_wave_early_lander_reserve_if_ready(commands) {
                return vec![SoundCue::HyperspaceMaterialize];
            }
            return Vec::new();
        }

        self.first_wave_early_reserve_steps_remaining = None;
        self.clear_first_wave_lander_refill();
        let wave_tuning_profile = self.current_wave_tuning_profile();
        let reserve_kinds =
            reserve_wave_enemy_kinds(&mut self.enemy_reserve, wave_tuning_profile);
        let mut index = 0;
        while index < reserve_kinds.len() {
            match reserve_kinds[index] {
                WaveEnemyKind::Lander => {
                    let target_index = self.select_next_lander_target_human_slot();
                    if let Some(target_index) = target_index {
                        let spawn = ActorLanderSpawn::from_wave_restore(
                            &mut self.actor_rng,
                            wave_tuning_profile,
                            Some(target_index),
                        );
                        commands.push(GameCommand::Spawn(SpawnRequest::Lander {
                            position: spawn.position,
                        }));
                        let actor = self.spawn_lander_from_spawn(spawn);
                        self.apply_next_wave_spawn_behavior(ActorKind::Lander, actor);
                        index += 1;
                    } else {
                        let lander_count = reserve_kinds[index..]
                            .iter()
                            .take_while(|&&kind| kind == WaveEnemyKind::Lander)
                            .count();
                        for _ in 0..lander_count {
                            let spawn = ActorMutantSpawn::from_wave_restore(
                                &mut self.actor_rng,
                                wave_tuning_profile,
                                self.background_left,
                            );
                            commands.push(GameCommand::Spawn(SpawnRequest::Mutant {
                                position: spawn.position,
                                runtime_state: spawn.runtime_state,
                            }));
                            let actor = self.spawn_mutant_from_spawn(spawn);
                            self.apply_next_wave_spawn_behavior(ActorKind::Mutant, actor);
                        }
                        index += lander_count;
                    }
                }
                WaveEnemyKind::Bomber => {
                    let bomber_count = reserve_kinds[index..]
                        .iter()
                        .take_while(|&&kind| kind == WaveEnemyKind::Bomber)
                        .count();
                    let player_absolute_x = self
                        .active_player_position()
                        .map_or(0, |position| absolute_world_x(position, 0));
                    for spawn in ActorBomberSpawn::wave_restore_batch(
                        wave_tuning_profile,
                        player_absolute_x,
                        bomber_count,
                    ) {
                        commands.push(GameCommand::Spawn(SpawnRequest::Bomber {
                            position: spawn.position,
                        }));
                        let actor = self.spawn_bomber_from_spawn(spawn);
                        self.apply_next_wave_spawn_behavior(ActorKind::Bomber, actor);
                    }
                    index += bomber_count;
                }
                WaveEnemyKind::Pod => {
                    let spawn = ActorPodSpawn::from_wave_restore(&mut self.actor_rng);
                    commands.push(GameCommand::Spawn(SpawnRequest::Pod {
                        position: spawn.position,
                    }));
                    let actor = self.spawn_pod_from_spawn(spawn);
                    self.apply_next_wave_spawn_behavior(ActorKind::Pod, actor);
                    index += 1;
                }
                WaveEnemyKind::Mutant => {
                    let spawn = ActorMutantSpawn::from_wave_restore(
                        &mut self.actor_rng,
                        wave_tuning_profile,
                        self.background_left,
                    );
                    commands.push(GameCommand::Spawn(SpawnRequest::Mutant {
                        position: spawn.position,
                        runtime_state: spawn.runtime_state,
                    }));
                    let actor = self.spawn_mutant_from_spawn(spawn);
                    self.apply_next_wave_spawn_behavior(ActorKind::Mutant, actor);
                    index += 1;
                }
                WaveEnemyKind::Swarmer => {
                    let swarmer_count = reserve_kinds[index..]
                        .iter()
                        .take_while(|&&kind| kind == WaveEnemyKind::Swarmer)
                        .count();
                    for spawn in ActorSwarmerSpawn::wave_restore_batch(
                        &mut self.actor_rng,
                        wave_tuning_profile,
                        swarmer_count,
                    ) {
                        commands.push(GameCommand::Spawn(SpawnRequest::Swarmer {
                            position: spawn.position,
                            runtime_state: spawn.runtime_state,
                        }));
                        let actor = self.spawn_swarmer_from_spawn(spawn);
                        self.apply_next_wave_spawn_behavior(ActorKind::Swarmer, actor);
                    }
                    index += swarmer_count;
                }
            }
        }
        Vec::new()
    }

    fn advance_first_wave_lander_refill_if_ready(&mut self, commands: &mut Vec<GameCommand>) {
        let Some(remaining) = self.first_wave_lander_refill_steps_remaining else {
            return;
        };
        if self.phase != Phase::Playing
            || self.wave != 1
            || self.pending_survivor_bonus.is_some()
            || self.pending_player_switch.is_some()
            || self.pending_player_start.is_some()
        {
            self.clear_first_wave_lander_refill();
            return;
        }

        let remaining = remaining.saturating_sub(1);
        if remaining > 0 {
            self.first_wave_lander_refill_steps_remaining = Some(remaining);
            return;
        }

        self.first_wave_lander_refill_steps_remaining = None;
        self.activate_first_wave_lander_refill(commands);
    }

    fn arm_first_wave_early_lander_reserve_delay(&mut self) {
        self.first_wave_early_reserve_steps_remaining = (self.wave == 1
            && self.enemy_reserve.landers > 0)
            .then_some(FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS);
    }

    fn schedule_first_wave_lander_refill_if_needed(&mut self) {
        if self
            .first_wave_lander_refill_steps_remaining
            .is_some()
            || self.phase != Phase::Playing
            || self.wave != 1
            || self.pending_survivor_bonus.is_some()
            || self.pending_player_switch.is_some()
            || self.pending_player_start.is_some()
            || self.pending_smart_bomb_detonation_steps.is_some()
            || self.reserve_activation_cooldown_steps > 0
        {
            return;
        }

        let active_landers = self.wave_lander_snapshot_count();
        if active_landers == 0
            || active_landers >= FIRST_WAVE_LANDER_REFILL_ACTIVE_THRESHOLD
            || self.enemy_reserve.landers == 0
            || usize::from(self.enemy_reserve.landers) > MAX_ACTIVE_WAVE_ENEMIES
        {
            return;
        }

        self.first_wave_lander_refill_steps_remaining =
            Some(FIRST_WAVE_LANDER_REFILL_DELAY_STEPS);
    }

    fn activate_first_wave_early_lander_reserve_if_ready(
        &mut self,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(remaining) = self.first_wave_early_reserve_steps_remaining else {
            return false;
        };
        if self.wave != 1 || self.enemy_reserve.landers == 0 {
            self.first_wave_early_reserve_steps_remaining = None;
            return false;
        }

        let remaining = remaining.saturating_sub(1);
        if remaining > 0 {
            self.first_wave_early_reserve_steps_remaining = Some(remaining);
            return false;
        }

        self.first_wave_early_reserve_steps_remaining = None;
        if self.wave_hostile_snapshot_count() >= FIRST_WAVE_EARLY_RESERVE_ACTIVE_LIMIT {
            return false;
        }

        let reserve_count = ACTOR_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS
            .len()
            .min(usize::from(self.enemy_reserve.landers));
        if reserve_count == 0 {
            return false;
        }

        self.apply_first_wave_early_reserve_shot_phase_delay();
        for spawn in ACTOR_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS
            .iter()
            .copied()
            .take(reserve_count)
        {
            commands.push(GameCommand::Spawn(SpawnRequest::Lander {
                position: spawn.position,
            }));
            let actor = self.spawn_lander_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Lander, actor);
        }
        self.enemy_reserve.landers = self
            .enemy_reserve
            .landers
            .saturating_sub(u8::try_from(reserve_count).expect("early reserve count fits u8"));
        self.actor_rng = FIRST_WAVE_EARLY_RESERVE_RNG;
        self.target_human_cursor = Some(FIRST_WAVE_EARLY_RESERVE_TARGET_CURSOR_SLOT);
        true
    }

    fn activate_first_wave_lander_refill(&mut self, commands: &mut Vec<GameCommand>) -> bool {
        if self.enemy_reserve.landers == 0 {
            return false;
        }

        let reserve_count = ACTOR_FIRST_WAVE_REFILL_LANDER_SPAWNS
            .len()
            .min(usize::from(self.enemy_reserve.landers));
        if reserve_count == 0 {
            return false;
        }

        let mut materialized = false;
        for spawn in ACTOR_FIRST_WAVE_REFILL_LANDER_SPAWNS
            .iter()
            .copied()
            .take(reserve_count)
        {
            materialized |= spawn.runtime_state.is_some_and(lander_spawn_is_visible);
            commands.push(GameCommand::Spawn(SpawnRequest::Lander {
                position: spawn.position,
            }));
            let actor = self.spawn_lander_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Lander, actor);
        }
        self.enemy_reserve.landers = self
            .enemy_reserve
            .landers
            .saturating_sub(u8::try_from(reserve_count).expect("refill count fits u8"));
        if materialized {
            self.queue_first_wave_lander_refill_appearance_sound();
        }
        true
    }

    fn apply_first_wave_early_reserve_shot_phase_delay(&self) {
        for actor in self.actors.values() {
            actor.apply_driver_command(ActorDriverCommand::AdjustLanderFireTimer {
                target_human_index: 2,
                x_velocity: FIRST_WAVE_EARLY_RESERVE_TARGET2_X_VELOCITY,
                delta: FIRST_WAVE_EARLY_RESERVE_TARGET2_SHOT_PHASE_DELAY,
            });
        }
    }

    fn spawn_wave_hostiles(&mut self) {
        let wave_profile = self.wave_script.profile_for_wave(self.wave).clone();
        for spawn in wave_profile.lander_spawns.iter().copied() {
            let actor = self.spawn_lander_from_spawn(spawn);
            if let Some(target_index) = spawn
                .runtime_state
                .and_then(|runtime_state| runtime_state.target_human_index)
            {
                self.target_human_cursor = Some(target_index);
            }
            self.apply_next_wave_spawn_behavior(ActorKind::Lander, actor);
        }
        for spawn in wave_profile.bomber_spawns.iter().copied() {
            let actor = self.spawn_bomber_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Bomber, actor);
        }
        for spawn in wave_profile.pod_spawns.iter().copied() {
            let actor = self.spawn_pod_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Pod, actor);
        }
        for spawn in wave_profile.mutant_spawns.iter().copied() {
            let actor = self.spawn_mutant_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Mutant, actor);
        }
        for spawn in wave_profile.swarmer_spawns.iter().copied() {
            let actor = self.spawn_swarmer_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Swarmer, actor);
        }
        for spawn in wave_profile.baiter_spawns.iter().copied() {
            let actor = self.spawn_baiter_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Baiter, actor);
        }
    }

    fn active_player_position(&self) -> Option<Point> {
        self.snapshots
            .values()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.position)
    }

    fn has_targetable_human_snapshots(&self) -> bool {
        self.snapshots.values().any(|snapshot| {
            snapshot.kind == ActorKind::Human && snapshot.alive && snapshot.runtime.as_human().is_some()
        })
    }

    fn human_snapshot_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
            .count()
    }

    fn wave_hostile_snapshot_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| is_hostile(snapshot.kind))
            .count()
    }

    fn wave_lander_snapshot_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .count()
    }

    fn select_next_lander_target_human_slot(&mut self) -> Option<usize> {
        if !self.has_targetable_human_snapshots() {
            return None;
        }

        let original_cursor = self
            .target_human_cursor
            .filter(|slot| *slot < TARGET_LIST_ENTRY_COUNT)
            .unwrap_or(0);
        let mut probe = original_cursor;
        for _ in 0..TARGET_LIST_ENTRY_COUNT {
            probe = next_target_list_slot_index(probe);
            if self.snapshots.values().any(|snapshot| {
                snapshot.kind == ActorKind::Human
                    && snapshot.alive
                    && snapshot
                        .runtime.as_human()
                        .is_some_and(|runtime_state| runtime_state.target_slot_index == probe)
            }) {
                self.target_human_cursor = Some(probe);
                return Some(probe);
            }
            if probe == original_cursor {
                break;
            }
        }

        None
    }

    fn apply_next_wave_spawn_behavior(&mut self, kind: ActorKind, actor: ActorId) {
        let spawn_index = self.next_wave_spawn_index(kind);
        if let Some(profile) = self
            .wave_script
            .profile_for_wave(self.wave)
            .spawn_behavior_profile(kind, spawn_index)
        {
            self.behavior_script.set_actor_behavior(actor, profile);
        }
    }

    fn next_wave_spawn_index(&mut self, kind: ActorKind) -> usize {
        let next = self.wave_spawn_allocations.entry(kind).or_insert(0);
        let spawn_index = *next;
        *next = next.saturating_add(1);
        spawn_index
    }

    fn start_survivor_bonus_if_wave_cleared(
        &mut self,
        was_playing: bool,
        commands: &[GameCommand],
    ) -> bool {
        if !was_playing
            || self.phase != Phase::Playing
            || self.wave == 0
            || self.pending_survivor_bonus.is_some()
            || self.pending_player_start.is_some()
            || !actor_enemy_reserve_is_empty(self.enemy_reserve)
            || self.has_wave_clear_hostile_snapshots()
            || commands_spawn_hostiles(commands)
        {
            return false;
        }

        self.clear_nonblocking_wave_hostiles();
        self.pending_survivor_bonus = Some(PendingSurvivorBonus::new(
            self.wave,
            self.wave.saturating_add(1),
            self.surviving_human_count(),
        ));
        true
    }

    fn surviving_human_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
            .count()
    }

    fn has_hostile_snapshots(&self) -> bool {
        self.snapshots
            .values()
            .any(|snapshot| is_hostile(snapshot.kind))
    }

    fn has_wave_clear_hostile_snapshots(&self) -> bool {
        self.snapshots.values().any(snapshot_blocks_wave_clear)
    }

    fn clear_nonblocking_wave_hostiles(&mut self) {
        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| is_hostile(snapshot.kind) && !snapshot_blocks_wave_clear(snapshot))
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>();
        for id in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
        }
    }

    fn pod_swarmer_spawn_commands(&mut self, position: Point) -> Vec<GameCommand> {
        let active_swarmers = self
            .snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Swarmer)
            .count();
        let spawn_count =
            POD_SWARMER_REQUEST_LIMIT.min(ACTIVE_SWARMER_LIMIT.saturating_sub(active_swarmers));
        let wave_tuning_profile = self.current_wave_tuning_profile();

        (0..spawn_count)
            .map(|_| {
                let spawn = ActorSwarmerSpawn::from_pod_release(
                    &mut self.actor_rng,
                    wave_tuning_profile,
                    position,
                );
                GameCommand::Spawn(SpawnRequest::Swarmer {
                    position: spawn.position,
                    runtime_state: spawn.runtime_state,
                })
            })
            .collect()
    }

    fn advance_baiter_timer(&mut self, commands: &mut Vec<GameCommand>) {
        if self.phase != Phase::Playing || self.wave == 0 {
            return;
        }
        let enemy_total = self.wave_tuning_enemy_total();
        if enemy_total == 0 {
            return;
        }
        let Some(timer_steps) = self.baiter_timer_steps else {
            return;
        };

        if self.baiter_pacing_steps_remaining > 1 {
            self.baiter_pacing_steps_remaining =
                self.baiter_pacing_steps_remaining.saturating_sub(1);
            return;
        }
        self.baiter_pacing_steps_remaining = ACTOR_BAITER_TIMER_PACING_STEPS;

        let profile = self.current_wave_tuning_profile();
        let timer_steps = accelerated_baiter_timer_steps(timer_steps, profile, enemy_total);
        let decremented_steps = timer_steps.saturating_sub(1);
        if decremented_steps > 0 {
            self.baiter_timer_steps = Some(decremented_steps);
            return;
        }

        self.baiter_timer_steps = Some(baiter_timer_reset_steps(profile, enemy_total));
        let active_baiters = self.snapshot_count(ActorKind::Baiter);
        if active_baiters >= ACTIVE_BAITER_LIMIT {
            return;
        }
        let Some(player_position) = self
            .snapshots
            .values()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.position)
        else {
            return;
        };
        let spawn = ActorBaiterSpawn::from_player_position(profile, player_position, active_baiters);
        commands.push(GameCommand::Spawn(SpawnRequest::Baiter {
            position: spawn.position,
            runtime_state: spawn.runtime_state,
        }));
    }

    fn wave_tuning_enemy_total(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot_blocks_wave_clear(snapshot))
            .count()
    }

    fn spawn_initial_humans(&mut self) {
        let wave_profile = self.wave_script.profile_for_wave(self.wave).clone();
        for spawn in wave_profile.human_spawns.iter().copied() {
            let actor = self.spawn_human_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Human, actor);
        }
    }

    fn detonate_smart_bomb_targets(&mut self, commands: &mut Vec<GameCommand>) -> bool {
        let mut bonus_awarded = false;
        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| is_smart_bomb_target(snapshot.kind))
            .map(|snapshot| (snapshot.id, snapshot.kind, snapshot.position))
            .collect::<Vec<_>>();
        for (id, kind, position) in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
            commands.push(GameCommand::Destroy(id));
            if let Some(explosion_kind) = explosion_kind_for_target(kind) {
                self.spawn_explosion(position, explosion_kind);
                commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                    position,
                    kind: explosion_kind,
                    explosion_anchor: None,
                }));
            }
            let points = score_for_hostile(kind);
            commands.push(GameCommand::AddScore(points));
            if self.award_points(points) {
                bonus_awarded = true;
            }
        }
        bonus_awarded
    }

    fn award_points(&mut self, points: u32) -> bool {
        let score_step = ScoreSystem::award_points(
            ScoreSnapshot {
                player_one: self.score,
                player_two: self.player_two_score,
                high_score: self.highest_visible_score(),
                next_bonus: self.next_bonus,
            },
            self.active_stock(),
            self.current_player,
            points,
        );
        self.score = score_step.scores.player_one;
        self.player_two_score = score_step.scores.player_two;
        self.set_active_stock(score_step.stock);
        self.next_bonus = score_step.scores.next_bonus;
        score_step.bonus_awards > 0
    }

    fn clear_enemy_projectiles_for_hyperspace(&mut self) {
        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| is_enemy_projectile_kind(snapshot.kind))
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>();
        for id in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
        }
    }

    fn remove_dead_actors(&mut self, dead_actor_ids: &[ActorId]) {
        for id in dead_actor_ids {
            self.snapshots.remove(id);
            self.actors.remove(id);
            self.behavior_script.remove_actor_behavior(*id);
        }
    }

    fn clear_wave_playfield_actors(&mut self) {
        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| clears_for_next_wave(snapshot.kind))
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>();
        for id in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
        }
    }

    fn clear_turn_playfield_actors(&mut self) {
        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| clears_for_next_turn(snapshot.kind))
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>();
        for id in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
        }
    }
}

impl Default for ActorGameDriver {
    fn default() -> Self {
        Self::new()
    }
}
