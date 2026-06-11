use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::actor_game) struct CollisionBody {
    pub(in crate::actor_game) owner: ActorId,
    pub(in crate::actor_game) kind: ActorKind,
    pub(in crate::actor_game) position: Point,
    pub(in crate::actor_game) bounds: Rect,
    pub(in crate::actor_game) actor_state: ActorInternalState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSnapshot {
    pub id: ActorId,
    pub kind: ActorKind,
    pub position: Point,
    pub velocity: Velocity,
    pub direction: Option<Direction>,
    pub bounds: Option<Rect>,
    pub alive: bool,
    pub(in crate::actor_game) actor_state: ActorInternalState,
}

impl ActorSnapshot {
    pub(crate) fn actor_subpixels(&self) -> Option<ActorSubpixels> {
        self.actor_state.subpixels()
    }

    pub(crate) fn actor_x_fraction(&self) -> Option<u8> {
        self.actor_subpixels().map(ActorSubpixels::x)
    }

    pub(crate) fn enemy_projectile_actor_state(&self) -> Option<EnemyProjectileActorState> {
        self.actor_state.as_enemy_projectile()
    }

    pub(in crate::actor_game) fn collision_body(&self) -> Option<CollisionBody> {
        Some(CollisionBody {
            owner: self.id,
            kind: self.kind,
            position: self.position,
            bounds: self.bounds?,
            actor_state: self.actor_state,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanMode {
    Grounded,
    Falling { velocity: i16 },
    CarriedBy(ActorId),
}

impl HumanMode {
    pub(in crate::actor_game) const fn sprite(self) -> SpriteKey {
        match self {
            Self::Grounded => SpriteKey::Human,
            Self::Falling { .. } => SpriteKey::HumanFalling,
            Self::CarriedBy(_) => SpriteKey::HumanCarried,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawCommand {
    pub actor: ActorId,
    pub sprite: SpriteKey,
    pub position: Point,
    pub effect: VisualEffect,
    pub text: Option<String>,
}

impl DrawCommand {
    pub fn sprite(actor: ActorId, sprite: SpriteKey, position: Point) -> Self {
        Self::sprite_with_effect(actor, sprite, position, VisualEffect::Static)
    }

    pub fn sprite_with_effect(
        actor: ActorId,
        sprite: SpriteKey,
        position: Point,
        effect: VisualEffect,
    ) -> Self {
        Self {
            actor,
            sprite,
            position,
            effect,
            text: None,
        }
    }

    pub fn text(actor: ActorId, position: Point, value: impl Into<String>) -> Self {
        Self {
            actor,
            sprite: SpriteKey::Text,
            position,
            effect: VisualEffect::Static,
            text: Some(value.into()),
        }
    }

    pub fn message_text(
        actor: ActorId,
        value: impl Into<String>,
        screen_cell: ScreenAddress,
    ) -> Self {
        Self::message_text_with_offset(actor, value, screen_cell, Point::new(0, 0))
    }

    pub fn message_text_with_offset(
        actor: ActorId,
        value: impl Into<String>,
        screen_cell: ScreenAddress,
        visual_offset: Point,
    ) -> Self {
        Self {
            actor,
            sprite: SpriteKey::Text,
            position: Point::new(0, 0),
            effect: VisualEffect::MessageText {
                screen_cell,
                visual_offset,
            },
            text: Some(value.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SpawnRequest {
    Laser {
        position: Point,
        direction: Direction,
        owner: ActorId,
    },
    EnemyLaser {
        position: Point,
        velocity: Velocity,
        actor_state: Option<EnemyProjectileActorState>,
    },
    Lander {
        position: Point,
    },
    Mutant {
        position: Point,
        actor_state: Option<MutantActorState>,
    },
    Bomber {
        position: Point,
    },
    Bomb {
        position: Point,
        actor_state: Option<EnemyProjectileActorState>,
    },
    Pod {
        position: Point,
    },
    Swarmer {
        position: Point,
        actor_state: Option<SwarmerActorState>,
    },
    Baiter {
        position: Point,
        actor_state: Option<BaiterActorState>,
    },
    Explosion {
        position: Point,
        kind: ExplosionKind,
        explosion_anchor: Option<Point>,
    },
    ScorePopup {
        position: Point,
        points: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GameCommand {
    Credit,
    StartOnePlayer,
    StartTwoPlayer,
    Spawn(SpawnRequest),
    Destroy(ActorId),
    SetWorldScrollLeft(u16),
    AttachHuman {
        lander: ActorId,
        human: ActorId,
        position: Point,
    },
    SmartBomb {
        consume_stock: bool,
    },
    Hyperspace,
    HumanLost(ActorId),
    AddScore(u32),
    PlaySound(SoundCue),
    PlayerKilled,
    WaveCleared {
        next_wave: u16,
    },
    AdvanceWave {
        wave: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepPrompt {
    pub step: u64,
    pub phase: Phase,
    pub input: GameInput,
    pub wave: u16,
    pub wave_tuning: ActorWaveTuning,
    pub current_player: u8,
    pub player_count: u8,
    pub score: u32,
    pub player_scores: [u32; 2],
    pub credits: u8,
    pub lives: u8,
    pub smart_bombs: u8,
    pub smart_bomb_pending: bool,
    pub player_stocks: [PlayerStockSnapshot; 2],
    pub player_death_sleep_remaining: Option<u8>,
    pub game_over_hall_of_fame_stall_remaining: Option<u8>,
    pub player_switch: Option<PlayerSwitchReport>,
    pub player_start: Option<PlayerStartReport>,
    pub high_scores: [u32; 5],
    pub high_score_initials: HighScoreInitialsState,
    pub snapshots: Vec<ActorSnapshot>,
    pub behavior_script: ActorBehaviorScript,
    pub background_left: u16,
    pub actor_rng: Option<ActorRngSnapshot>,
    pub human_walk_target_slot: Option<usize>,
    pub projectile_scan_tick: bool,
}

impl StepPrompt {
    pub fn behavior_for(&self, actor: ActorId, kind: ActorKind) -> ActorBehaviorProfile {
        self.behavior_script.behavior_for(actor, kind)
    }

    pub fn player_position(&self) -> Option<Point> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.position)
    }

    pub fn player_velocity(&self) -> Option<Velocity> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.velocity)
    }

    pub(in crate::actor_game) fn snapshot(&self, id: ActorId) -> Option<&ActorSnapshot> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.id == id && snapshot.alive)
    }

    pub(in crate::actor_game) fn nearest_human(&self, position: Point) -> Option<&ActorSnapshot> {
        self.snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
            .min_by_key(|snapshot| manhattan_distance(position, snapshot.position))
    }

    pub(in crate::actor_game) fn target_human(
        &self,
        target_slot_index: usize,
    ) -> Option<&ActorSnapshot> {
        self.snapshots.iter().find(|snapshot| {
            snapshot.kind == ActorKind::Human
                && snapshot.alive
                && snapshot.bounds.is_some()
                && snapshot
                    .actor_state
                    .as_human()
                    .is_some_and(|actor_state| actor_state.target_slot_index == target_slot_index)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::actor_game) struct ActorReply {
    pub(in crate::actor_game) id: ActorId,
    pub(in crate::actor_game) snapshot: ActorSnapshot,
    pub(in crate::actor_game) commands: Vec<GameCommand>,
    pub(in crate::actor_game) draws: Vec<DrawCommand>,
}

pub(in crate::actor_game) trait AssetActor: Send + 'static {
    fn id(&self) -> ActorId;

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply;

    fn apply_driver_command(&mut self, _command: ActorDriverCommand) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::actor_game) enum ActorDriverCommand {
    AdjustLanderFireTimer {
        target_human_index: usize,
        x_velocity: u16,
        delta: u8,
    },
}

pub(in crate::actor_game) enum ActorRequest {
    Prompt(Box<StepPrompt>),
    DriverCommand(ActorDriverCommand),
    Stop,
}

pub(in crate::actor_game) struct ThreadedAsset {
    pub(in crate::actor_game) sender: Sender<ActorRequest>,
    pub(in crate::actor_game) receiver: Receiver<ActorReply>,
    pub(in crate::actor_game) handle: Option<JoinHandle<()>>,
}

impl ThreadedAsset {
    pub(in crate::actor_game) fn spawn(actor: impl AssetActor) -> Self {
        let (request_sender, request_receiver) = mpsc::channel();
        let (reply_sender, reply_receiver) = mpsc::channel();
        let handle = thread::spawn(move || run_actor_thread(actor, request_receiver, reply_sender));
        Self {
            sender: request_sender,
            receiver: reply_receiver,
            handle: Some(handle),
        }
    }

    pub(in crate::actor_game) fn prompt(&self, prompt: StepPrompt) -> Option<ActorReply> {
        self.sender
            .send(ActorRequest::Prompt(Box::new(prompt)))
            .ok()?;
        self.receiver.recv().ok()
    }

    pub(in crate::actor_game) fn apply_driver_command(&self, command: ActorDriverCommand) {
        let _ = self.sender.send(ActorRequest::DriverCommand(command));
    }
}

impl Drop for ThreadedAsset {
    fn drop(&mut self) {
        let _ = self.sender.send(ActorRequest::Stop);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

pub(in crate::actor_game) fn run_actor_thread(
    mut actor: impl AssetActor,
    receiver: Receiver<ActorRequest>,
    sender: Sender<ActorReply>,
) {
    while let Ok(request) = receiver.recv() {
        match request {
            ActorRequest::Prompt(prompt) => {
                if sender.send(actor.update(prompt.as_ref())).is_err() {
                    break;
                }
            }
            ActorRequest::DriverCommand(command) => {
                actor.apply_driver_command(command);
            }
            ActorRequest::Stop => break,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepReport {
    pub step: u64,
    pub phase: Phase,
    pub wave: u16,
    pub current_player: u8,
    pub player_count: u8,
    pub score: u32,
    pub player_scores: [u32; 2],
    pub credits: u8,
    pub lives: u8,
    pub smart_bombs: u8,
    pub smart_bomb_flash_steps_remaining: u8,
    pub player_stocks: [PlayerStockSnapshot; 2],
    pub next_bonus: u32,
    pub player_death_sleep_remaining: Option<u8>,
    pub game_over_hall_of_fame_stall_remaining: Option<u8>,
    pub player_switch: Option<PlayerSwitchReport>,
    pub player_start: Option<PlayerStartReport>,
    pub high_scores: [u32; 5],
    pub wave_tuning: ActorWaveTuning,
    pub high_score_initials: HighScoreInitialsState,
    pub high_score_initial_accepted: bool,
    pub high_score_submitted: bool,
    pub bonus_awarded: bool,
    pub survivor_bonus: Option<SurvivorBonusReport>,
    pub behavior_script: ActorBehaviorScriptManifest,
    pub enemy_reserve: EnemyReserveSnapshot,
    pub background_left: u16,
    pub actor_rng: Option<ActorRngSnapshot>,
    pub terrain_blow: Option<TerrainBlowSnapshot>,
    pub snapshots: Vec<ActorSnapshot>,
    pub draws: Vec<DrawCommand>,
    pub sounds: Vec<SoundCue>,
    pub(crate) commands: Vec<GameCommand>,
}

impl StepReport {
    pub fn sound_events(&self, bridge: &mut ActorSoundEventBridge) -> Vec<SoundEvent> {
        bridge.sound_events_for_report(self)
    }

    pub fn render_scene(&self) -> RenderScene {
        ActorRenderSceneBridge::new().render_scene_for_report(self)
    }

    pub fn render_scene_with(&self, bridge: &ActorRenderSceneBridge) -> RenderScene {
        bridge.render_scene_for_report(self)
    }

    pub fn game_state(&self) -> GameState {
        ActorStateBridge::new().state_for_report(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurvivorBonusReport {
    pub next_wave: u16,
    pub multiplier: u8,
    pub total_survivors: u8,
    pub visible_icons: u8,
    pub remaining_awards: u8,
    pub awarded_points: Option<u32>,
    pub astronaut_sleep_steps_remaining: u8,
    pub wave_advance_sleep_steps_remaining: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSwitchReport {
    pub sleep_steps_remaining: u8,
    pub from_player: u8,
    pub to_player: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStartReport {
    pub delay_steps_remaining: u8,
    pub player: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActorStateBridge;

impl ActorStateBridge {
    pub const fn new() -> Self {
        Self
    }

    pub fn state_for_report(&self, report: &StepReport) -> GameState {
        let phase = clean_phase(report.phase);
        let wave = clean_wave(report.wave);
        let high_score_tables = high_score_tables_for_report(report);
        GameState {
            step: report.step,
            phase,
            credits: report.credits,
            current_player: report.current_player,
            player_count: report.player_count,
            wave,
            wave_profile: actor_wave_profile_for_report(report),
            player: player_snapshot_for_report(report),
            player_stocks: report.player_stocks,
            scores: ScoreSnapshot {
                player_one: report.player_scores[0],
                player_two: report.player_scores[1],
                high_score: report.high_scores[0]
                    .max(report.player_scores[0])
                    .max(report.player_scores[1]),
                next_bonus: report.next_bonus,
            },
            attract: attract_snapshot_for_report(report),
            post_game_playfield: None,
            high_score_initials: report.high_score_initials,
            high_score_entry: high_score_entry_for_report(report),
            high_score_submission: None,
            high_score_tables,
            game_over: game_over_snapshot_for_report(report),
            world: world_snapshot_for_report(report),
        }
    }
}

pub(in crate::actor_game) fn clean_phase(phase: Phase) -> GamePhase {
    match phase {
        Phase::Attract => GamePhase::Attract,
        Phase::Playing => GamePhase::Playing,
        Phase::GameOver => GamePhase::GameOver,
        Phase::HighScoreEntry => GamePhase::HighScoreEntry,
    }
}

pub(in crate::actor_game) fn clean_wave(wave: u16) -> u8 {
    u8::try_from(wave.max(1)).unwrap_or(u8::MAX)
}

pub(in crate::actor_game) fn actor_wave_profile_for_report(
    report: &StepReport,
) -> WaveProfileSnapshot {
    let mut profile = WaveProfileSnapshot::for_wave(clean_wave(report.wave));
    let wave_tuning = report.wave_tuning;
    profile.landers = wave_tuning.landers;
    profile.bombers = wave_tuning.bombers;
    profile.pods = wave_tuning.pods;
    profile.mutants = wave_tuning.mutants;
    profile.swarmers = wave_tuning.swarmers;
    profile.lander_x_velocity = wave_tuning.lander_x_velocity;
    profile.lander_y_velocity = wave_tuning.lander_y_velocity;
    profile.mutant_random_y = wave_tuning.mutant_random_y;
    profile.mutant_y_velocity = wave_tuning.mutant_y_velocity;
    profile.mutant_x_velocity = wave_tuning.mutant_x_velocity;
    profile.swarmer_x_velocity = wave_tuning.swarmer_x_velocity;
    profile.wave_size = wave_tuning.wave_size;
    profile.lander_shot_time = wave_tuning.lander_shot_time;
    profile.bomber_x_velocity = wave_tuning.bomber_x_velocity;
    profile.mutant_shot_time = wave_tuning.mutant_shot_time;
    profile.swarmer_shot_time = wave_tuning.swarmer_shot_time;
    profile.swarmer_acceleration_mask = wave_tuning.swarmer_acceleration_mask;
    profile.baiter_delay = wave_tuning.baiter_delay;
    profile.baiter_shot_time = wave_tuning.baiter_shot_time;
    profile.baiter_seek_probability = wave_tuning.baiter_seek_probability;
    profile
}

pub(in crate::actor_game) fn attract_snapshot_for_report(
    report: &StepReport,
) -> AttractPresentationSnapshot {
    if report.phase == Phase::Attract {
        AttractPresentationSnapshot::for_page_step(u16::try_from(report.step).unwrap_or(u16::MAX))
    } else {
        AttractPresentationSnapshot::INACTIVE
    }
}

pub(in crate::actor_game) fn player_snapshot_for_report(report: &StepReport) -> PlayerSnapshot {
    let snapshot = report
        .snapshots
        .iter()
        .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive);
    let position = snapshot
        .map(|snapshot| snapshot.position)
        .unwrap_or_default();
    let velocity = snapshot
        .map(|snapshot| snapshot.velocity)
        .unwrap_or_default();
    PlayerSnapshot {
        position: (world_vector(position.x), world_vector(position.y)),
        velocity: (world_vector(velocity.dx), world_vector(velocity.dy)),
        direction: snapshot
            .and_then(|snapshot| snapshot.direction)
            .map(clean_direction)
            .unwrap_or_else(|| player_direction_for_report(report)),
        lives: report.lives,
        smart_bombs: report.smart_bombs,
    }
}

pub(in crate::actor_game) fn clean_direction(direction: Direction) -> CleanDirection {
    match direction {
        Direction::Left => CleanDirection::Left,
        Direction::Right => CleanDirection::Right,
    }
}

pub(in crate::actor_game) fn player_direction_for_report(report: &StepReport) -> CleanDirection {
    report
        .draws
        .iter()
        .rev()
        .find_map(|draw| match draw.sprite {
            SpriteKey::PlayerLeft => Some(CleanDirection::Left),
            SpriteKey::PlayerRight => Some(CleanDirection::Right),
            _ => None,
        })
        .unwrap_or(CleanDirection::Right)
}

pub(in crate::actor_game) fn high_score_tables_for_report(
    report: &StepReport,
) -> HighScoreTablesSnapshot {
    let entries = hall_score_entries(&report.high_scores);
    HighScoreTablesSnapshot {
        all_time: entries,
        todays_greatest: entries,
    }
}

pub(in crate::actor_game) fn high_score_entry_for_report(
    report: &StepReport,
) -> Option<HighScoreEntrySnapshot> {
    if report.phase != Phase::HighScoreEntry {
        return None;
    }

    report
        .high_scores
        .iter()
        .position(|score| *score == report.score)
        .map(|index| HighScoreEntrySnapshot {
            score: report.score,
            rank: u8::try_from(index + 1).expect("actor high-score rank should fit u8"),
        })
}

pub(in crate::actor_game) fn game_over_snapshot_for_report(
    report: &StepReport,
) -> GameOverSnapshot {
    if let Some(remaining) = report.player_death_sleep_remaining {
        return GameOverSnapshot {
            player_death_sleep_remaining: Some(remaining),
            ..GameOverSnapshot::NONE
        };
    }

    if let Some(player_switch) = report.player_switch {
        return GameOverSnapshot {
            player_switch_sleep_remaining: Some(player_switch.sleep_steps_remaining),
            player_switch_from: Some(player_switch.from_player),
            player_switch_to: Some(player_switch.to_player),
            ..GameOverSnapshot::NONE
        };
    }

    let Some(remaining) = report.game_over_hall_of_fame_stall_remaining else {
        return GameOverSnapshot::NONE;
    };

    GameOverSnapshot {
        hall_of_fame_stall_remaining: Some(remaining),
        ..GameOverSnapshot::NONE
    }
}

pub(in crate::actor_game) fn world_snapshot_for_report(report: &StepReport) -> WorldSnapshot {
    if report.player_start.is_some() {
        return WorldSnapshot::default();
    }

    let phase = clean_phase(report.phase);
    let player = player_snapshot_for_report(report);
    let mut world = WorldSnapshot {
        terrain: actor_playfield_terrain_segments(report),
        terrain_blow: report.terrain_blow,
        enemies: actor_enemies_for_report(report),
        humans: actor_humans_for_report(report),
        projectiles: actor_projectiles_for_report(report),
        enemy_projectiles: actor_enemy_projectiles_for_report(report),
        explosions: actor_explosions_for_report(report),
        score_popups: actor_score_popups_for_report(report),
        enemy_reserve: report.enemy_reserve,
        actor_rng: report.actor_rng.map(clean_actor_rng).unwrap_or_default(),
        ..WorldSnapshot::default()
    };
    world.sync_actor_presentation(
        phase,
        report.step,
        actor_world_word(report.background_left),
        player.position,
    );
    world
}

pub(in crate::actor_game) fn actor_playfield_terrain_segments(
    report: &StepReport,
) -> Vec<TerrainSegment> {
    if report.phase != Phase::Playing || report.terrain_blow.is_some() {
        return Vec::new();
    }

    playfield_terrain_segments()
}

pub(in crate::actor_game) fn playfield_terrain_segments() -> Vec<TerrainSegment> {
    PLAYFIELD_TERRAIN_SEGMENTS.to_vec()
}

pub(in crate::actor_game) fn actor_human_walk_target_y(position_x: i16, offset: u8) -> Option<i16> {
    actor_playfield_terrain_altitude_at_x(position_x)
        .map(|altitude| i16::from(altitude.wrapping_add(offset).min(HUMAN_MAX_TARGET_Y)))
}

pub(in crate::actor_game) fn actor_playfield_terrain_altitude_at_x(position_x: i16) -> Option<u8> {
    let object_x = u16::from(u8::try_from(position_x).ok()?);
    playfield_terrain_segments()
        .into_iter()
        .find(|segment| {
            let start = u16::from(segment.position.x);
            let end = start.saturating_add(u16::from(segment.size.0));
            object_x >= start && object_x < end
        })
        .map(|segment| segment.position.y)
}

pub(in crate::actor_game) fn actor_step_human_toward_walk_target_y(
    position_y: i16,
    target_y: i16,
) -> i16 {
    match position_y.cmp(&target_y) {
        Ordering::Less => position_y + 1,
        Ordering::Equal => position_y,
        Ordering::Greater => position_y - 1,
    }
}

pub(in crate::actor_game) fn actor_enemies_for_report(
    report: &StepReport,
) -> Vec<CleanEnemySnapshot> {
    report
        .snapshots
        .iter()
        .filter_map(clean_enemy_snapshot)
        .collect()
}

pub(in crate::actor_game) fn clean_enemy_snapshot(
    snapshot: &ActorSnapshot,
) -> Option<CleanEnemySnapshot> {
    if snapshot.kind == ActorKind::Lander && snapshot.bounds.is_none() {
        return None;
    }

    let kind = match snapshot.kind {
        ActorKind::Lander => CleanEnemyKind::Lander,
        ActorKind::Mutant => CleanEnemyKind::Mutant,
        ActorKind::Bomber => CleanEnemyKind::Bomber,
        ActorKind::Pod => CleanEnemyKind::Pod,
        ActorKind::Swarmer => CleanEnemyKind::Swarmer,
        ActorKind::Baiter => CleanEnemyKind::Baiter,
        _ => return None,
    };
    let mut enemy = CleanEnemySnapshot::new(
        kind,
        screen_position(snapshot.position),
        screen_velocity(snapshot.velocity),
    );
    enemy.lander_actor_state = snapshot
        .actor_state
        .as_lander()
        .map(clean_lander_actor_state);
    enemy.bomber_actor_state = snapshot
        .actor_state
        .as_bomber()
        .map(clean_bomber_actor_state);
    enemy.pod_actor_state = snapshot.actor_state.as_pod().map(clean_pod_actor_state);
    enemy.swarmer_actor_state = snapshot
        .actor_state
        .as_swarmer()
        .map(clean_swarmer_actor_state);
    enemy.baiter_actor_state = snapshot
        .actor_state
        .as_baiter()
        .map(clean_baiter_actor_state);
    enemy.mutant_actor_state = snapshot
        .actor_state
        .as_mutant()
        .map(clean_mutant_actor_state);
    Some(enemy)
}

pub(in crate::actor_game) fn clean_lander_actor_state(
    actor_state: LanderActorState,
) -> LanderDebugStateSnapshot {
    LanderDebugStateSnapshot {
        motion: ActorDebugMotion::new(
            actor_state.x_fraction(),
            actor_state.y_fraction(),
            actor_state.x_velocity(),
            actor_state.y_velocity(),
        ),
        shot_timer: actor_state.shot_timer,
        sleep_ticks: actor_state.sleep_ticks,
        animation_frame: actor_state.animation_frame.index(),
        target_human_index: actor_state.target_human_index,
    }
}

pub(in crate::actor_game) fn clean_bomber_actor_state(
    actor_state: BomberActorState,
) -> BomberDebugStateSnapshot {
    BomberDebugStateSnapshot {
        motion: ActorDebugMotion::new(
            actor_state.x_fraction(),
            actor_state.y_fraction(),
            actor_state.x_velocity(),
            actor_state.y_velocity(),
        ),
        animation_frame: actor_state.animation_frame.index(),
        cruise_altitude: screen_coordinate(actor_state.cruise_altitude),
        sleep_ticks: actor_state.sleep_ticks,
        slot: actor_state.slot,
    }
}

pub(in crate::actor_game) fn clean_pod_actor_state(
    actor_state: PodActorState,
) -> PodDebugStateSnapshot {
    PodDebugStateSnapshot {
        motion: ActorDebugMotion::new(
            actor_state.x_fraction(),
            actor_state.y_fraction(),
            actor_state.x_velocity(),
            actor_state.y_velocity(),
        ),
    }
}

pub(in crate::actor_game) fn clean_swarmer_actor_state(
    actor_state: SwarmerActorState,
) -> SwarmerDebugStateSnapshot {
    SwarmerDebugStateSnapshot {
        motion: ActorDebugMotion::new(
            actor_state.x_fraction(),
            actor_state.y_fraction(),
            actor_state.x_velocity(),
            actor_state.y_velocity(),
        ),
        acceleration: actor_state.acceleration,
        shot_timer: actor_state.shot_timer,
        sleep_ticks: actor_state.sleep_ticks,
        horizontal_seek_pending: actor_state.horizontal_seek_pending,
    }
}

pub(in crate::actor_game) fn clean_baiter_actor_state(
    actor_state: BaiterActorState,
) -> BaiterDebugStateSnapshot {
    BaiterDebugStateSnapshot {
        motion: ActorDebugMotion::new(
            actor_state.x_fraction(),
            actor_state.y_fraction(),
            actor_state.x_velocity(),
            actor_state.y_velocity(),
        ),
        shot_timer: actor_state.shot_timer,
        sleep_ticks: actor_state.sleep_ticks,
        animation_frame: actor_state.animation_frame.index(),
    }
}

pub(in crate::actor_game) fn clean_mutant_actor_state(
    actor_state: MutantActorState,
) -> MutantDebugStateSnapshot {
    MutantDebugStateSnapshot {
        motion: ActorDebugMotion::new(
            actor_state.x_fraction(),
            actor_state.y_fraction(),
            actor_state.x_velocity(),
            actor_state.y_velocity(),
        ),
        shot_timer: actor_state.shot_timer,
        sleep_ticks: actor_state.sleep_ticks,
        hop_rng: clean_actor_rng(actor_state.hop_rng),
        render_x_correction: actor_state.render_x_correction,
        dive_entry_shot_deferred: actor_state.dive_entry_shot_deferred,
    }
}

pub(in crate::actor_game) const fn clean_actor_rng(actor_rng: ActorRngSnapshot) -> GameRngSnapshot {
    GameRngSnapshot {
        seed: actor_rng.seed,
        hseed: actor_rng.hseed,
        lseed: actor_rng.lseed,
    }
}

pub(in crate::actor_game) fn actor_humans_for_report(
    report: &StepReport,
) -> Vec<CleanHumanSnapshot> {
    report
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
        .map(|snapshot| {
            let mut human = CleanHumanSnapshot::new(screen_position(snapshot.position));
            human.carried = report.draws.iter().any(|draw| {
                draw.actor == snapshot.id && matches!(draw.sprite, SpriteKey::HumanCarried)
            });
            if let Some(actor_state) = snapshot.actor_state.as_human() {
                human.actor_x_fraction = actor_state.x_fraction();
                human.animation_frame = actor_state.animation_frame.index();
            }
            human
        })
        .collect()
}

pub(in crate::actor_game) fn actor_projectiles_for_report(
    report: &StepReport,
) -> Vec<CleanProjectileSnapshot> {
    report
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.kind == ActorKind::Laser && snapshot.alive)
        .map(|snapshot| CleanProjectileSnapshot {
            position: screen_position(snapshot.position),
            tail_position: screen_position(Point::new(
                snapshot.position.x.saturating_sub(16),
                snapshot.position.y,
            )),
            velocity: screen_velocity(snapshot.velocity),
        })
        .collect()
}

pub(in crate::actor_game) fn actor_enemy_projectiles_for_report(
    report: &StepReport,
) -> Vec<CleanEnemyProjectileSnapshot> {
    report
        .snapshots
        .iter()
        .filter(|snapshot| {
            matches!(snapshot.kind, ActorKind::EnemyLaser | ActorKind::Bomb) && snapshot.alive
        })
        .map(|snapshot| CleanEnemyProjectileSnapshot {
            position: screen_position(snapshot.position),
            velocity: screen_velocity(snapshot.velocity),
            kind: if snapshot.kind == ActorKind::Bomb {
                EnemyProjectileKind::BomberBombShell
            } else {
                EnemyProjectileKind::Fireball
            },
            actor_x_fraction: snapshot
                .actor_state
                .as_enemy_projectile()
                .map_or(0, |actor_state| actor_state.x_fraction()),
            actor_y_fraction: snapshot
                .actor_state
                .as_enemy_projectile()
                .map_or(0, |actor_state| actor_state.y_fraction()),
            actor_x_velocity: snapshot
                .actor_state
                .as_enemy_projectile()
                .map_or(0, |actor_state| actor_state.x_velocity()),
            actor_y_velocity: snapshot
                .actor_state
                .as_enemy_projectile()
                .map_or(0, |actor_state| actor_state.y_velocity()),
            lifetime_ticks: snapshot
                .actor_state
                .as_enemy_projectile()
                .map_or(0, |actor_state| actor_state.lifetime_ticks),
        })
        .collect()
}

pub(in crate::actor_game) fn actor_explosions_for_report(
    report: &StepReport,
) -> Vec<CleanExplosionSnapshot> {
    report
        .draws
        .iter()
        .filter_map(|draw| match draw.effect {
            VisualEffect::ExplosionCloud {
                kind,
                age,
                explosion_anchor,
            } => {
                let mut explosion = CleanExplosionSnapshot::spawn(
                    clean_explosion_kind(kind),
                    screen_position(draw.position),
                );
                explosion.explosion_anchor = explosion_anchor.map(screen_position);
                explosion.growth_size = actor_explosion_growth_size_for_kind(kind, age);
                Some(explosion)
            }
            _ => None,
        })
        .collect()
}

pub(in crate::actor_game) fn clean_explosion_kind(kind: ExplosionKind) -> CleanExplosionKind {
    match kind {
        ExplosionKind::Lander => CleanExplosionKind::Lander,
        ExplosionKind::Mutant => CleanExplosionKind::Mutant,
        ExplosionKind::Bomber => CleanExplosionKind::Bomber,
        ExplosionKind::Pod => CleanExplosionKind::Pod,
        ExplosionKind::Swarmer => CleanExplosionKind::Swarmer,
        ExplosionKind::Baiter => CleanExplosionKind::Baiter,
        ExplosionKind::Bomb => CleanExplosionKind::Bomb,
        ExplosionKind::Player => CleanExplosionKind::PlayerShip,
        ExplosionKind::Human => CleanExplosionKind::Astronaut,
        ExplosionKind::Terrain => CleanExplosionKind::Terrain,
    }
}

pub(in crate::actor_game) fn actor_score_popups_for_report(
    report: &StepReport,
) -> Vec<CleanScorePopupSnapshot> {
    report
        .draws
        .iter()
        .filter_map(|draw| {
            let kind = match draw.sprite {
                SpriteKey::Score250 => CleanScorePopupKind::Points250,
                SpriteKey::Score500 => CleanScorePopupKind::Points500,
                _ => return None,
            };
            Some(CleanScorePopupSnapshot::spawn(
                kind,
                screen_position(draw.position),
            ))
        })
        .collect()
}

pub(in crate::actor_game) fn world_vector(value: i16) -> WorldVector {
    WorldVector::from_subpixels(i32::from(value) * WorldVector::SUBPIXELS_PER_PIXEL)
}

pub(in crate::actor_game) fn actor_world_word(value: u16) -> WorldVector {
    WorldVector::from_subpixels(i32::from(value) << 8)
}

pub(in crate::actor_game) fn scroll_background_right(background_left: u16, pixels: i16) -> u16 {
    background_left.wrapping_add(background_pixel_delta(pixels))
}

pub(in crate::actor_game) fn scroll_background_left(background_left: u16, pixels: i16) -> u16 {
    background_left.wrapping_sub(background_pixel_delta(pixels))
}

pub(in crate::actor_game) fn background_pixel_delta(pixels: i16) -> u16 {
    u16::try_from(pixels.max(0))
        .unwrap_or(u16::MAX)
        .wrapping_mul(BACKGROUND_WORD_PER_PIXEL)
}

pub(in crate::actor_game) fn screen_position(point: Point) -> ScreenPosition {
    ScreenPosition::new(screen_coordinate(point.x), screen_coordinate(point.y))
}

pub(in crate::actor_game) fn try_screen_position(point: Point) -> Option<ScreenPosition> {
    Some(ScreenPosition::new(
        u8::try_from(point.x).ok()?,
        u8::try_from(point.y).ok()?,
    ))
}

pub(in crate::actor_game) fn screen_velocity(velocity: Velocity) -> ScreenVelocity {
    ScreenVelocity::new(
        screen_velocity_component(velocity.dx),
        screen_velocity_component(velocity.dy),
    )
}

pub(in crate::actor_game) fn screen_velocity_component(value: i16) -> i8 {
    i8::try_from(value.clamp(i16::from(i8::MIN), i16::from(i8::MAX)))
        .expect("screen velocity should be clamped to i8")
}

pub(in crate::actor_game) fn screen_coordinate(value: i16) -> u8 {
    u8::try_from(value.clamp(SCREEN_MIN_COORDINATE, SCREEN_MAX_COORDINATE))
        .expect("screen coordinate should be clamped to u8")
}

pub(in crate::actor_game) fn projectile_velocity_word(value: i16) -> u16 {
    let clamped = value.clamp(i16::from(i8::MIN), i16::from(i8::MAX)) as i8;
    ((i16::from(clamped)) << 8) as u16
}

pub(in crate::actor_game) fn projectile_lifetime_ticks(remaining_steps: u16) -> u8 {
    remaining_steps.min(u16::from(u8::MAX)) as u8
}

pub(in crate::actor_game) fn enemy_projectile_actor_state(
    x_fraction: u8,
    y_fraction: u8,
    velocity: Velocity,
    lifetime_steps: u16,
) -> EnemyProjectileActorState {
    EnemyProjectileActorState::from_velocity(
        x_fraction,
        y_fraction,
        velocity,
        projectile_lifetime_ticks(lifetime_steps),
    )
}

pub(in crate::actor_game) fn step_projectile_axis(
    position: i16,
    fraction: u8,
    velocity: u16,
) -> (i16, u8) {
    let fraction_scale = i32::from(MOTION_WORD_FRACTION_SCALE);
    let next_axis_word =
        i32::from(position) * fraction_scale + i32::from(fraction) + i32::from(velocity as i16);
    let next_position = next_axis_word.div_euclid(fraction_scale);
    let next_fraction = next_axis_word.rem_euclid(fraction_scale);
    (
        next_position.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16,
        next_fraction as u8,
    )
}
