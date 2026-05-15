//! Domain-facing gameplay contracts.

use crate::{
    renderer::{Color, RenderLayer, RenderScene, SceneSprite, SpriteId, SurfaceSize},
    systems::{
        CollisionBox, CollisionSystem, EnemyMotionSystem, GameSimulation, PlayerControlSystem,
        PlayerMotionState, PlayerMotionSystem, PlayerStock, ProjectileLaunchOutcome,
        ProjectileMotionSystem, ProjectileState, ProjectileSystem, ScoreSystem, ScreenPosition,
        ScreenVelocity, WaveState, WaveStatus, WaveSystem,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Attract,
    Playing,
    GameOver,
    HighScoreEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorldVector {
    subpixels: i32,
}

impl WorldVector {
    pub const SUBPIXELS_PER_PIXEL: i32 = 256;

    pub const fn from_subpixels(subpixels: i32) -> Self {
        Self { subpixels }
    }

    pub const fn subpixels(self) -> i32 {
        self.subpixels
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSnapshot {
    pub position: (WorldVector, WorldVector),
    pub velocity: (WorldVector, WorldVector),
    pub direction: Direction,
    pub lives: u8,
    pub smart_bombs: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreSnapshot {
    pub player_one: u32,
    pub player_two: u32,
    pub high_score: u32,
    pub next_bonus: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Lander,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemySnapshot {
    pub kind: EnemyKind,
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HumanSnapshot {
    pub position: ScreenPosition,
    pub carried: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectileSnapshot {
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainSegment {
    pub position: ScreenPosition,
    pub size: (u8, u8),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldSnapshot {
    pub terrain: Vec<TerrainSegment>,
    pub stars: Vec<ScreenPosition>,
    pub enemies: Vec<EnemySnapshot>,
    pub humans: Vec<HumanSnapshot>,
    pub projectiles: Vec<ProjectileSnapshot>,
}

impl WorldSnapshot {
    fn first_wave() -> Self {
        Self::for_wave(1)
    }

    fn for_wave(wave: u8) -> Self {
        let enemy_count = usize::from(wave.clamp(1, MAX_CLEAN_WAVE_ENEMIES));
        Self {
            terrain: vec![
                TerrainSegment {
                    position: ScreenPosition::new(0, 224),
                    size: (64, 8),
                },
                TerrainSegment {
                    position: ScreenPosition::new(64, 222),
                    size: (64, 8),
                },
                TerrainSegment {
                    position: ScreenPosition::new(128, 226),
                    size: (64, 8),
                },
                TerrainSegment {
                    position: ScreenPosition::new(192, 220),
                    size: (56, 8),
                },
                TerrainSegment {
                    position: ScreenPosition::new(248, 224),
                    size: (44, 8),
                },
            ],
            stars: vec![
                ScreenPosition::new(24, 32),
                ScreenPosition::new(112, 56),
                ScreenPosition::new(236, 24),
            ],
            enemies: CLEAN_WAVE_LANDER_SPAWNS[..enemy_count].to_vec(),
            humans: vec![
                HumanSnapshot {
                    position: ScreenPosition::new(72, 216),
                    carried: false,
                },
                HumanSnapshot {
                    position: ScreenPosition::new(180, 218),
                    carried: false,
                },
            ],
            projectiles: Vec::new(),
        }
    }
}

const MAX_CLEAN_WAVE_ENEMIES: u8 = 4;
const CLEAN_WAVE_LANDER_SPAWNS: [EnemySnapshot; MAX_CLEAN_WAVE_ENEMIES as usize] = [
    EnemySnapshot {
        kind: EnemyKind::Lander,
        position: ScreenPosition::new(204, 84),
        velocity: ScreenVelocity::new(-1, 0),
    },
    EnemySnapshot {
        kind: EnemyKind::Lander,
        position: ScreenPosition::new(228, 104),
        velocity: ScreenVelocity::new(-1, 0),
    },
    EnemySnapshot {
        kind: EnemyKind::Lander,
        position: ScreenPosition::new(184, 72),
        velocity: ScreenVelocity::new(1, 0),
    },
    EnemySnapshot {
        kind: EnemyKind::Lander,
        position: ScreenPosition::new(148, 96),
        velocity: ScreenVelocity::new(1, 0),
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub frame: u64,
    pub phase: GamePhase,
    pub credits: u8,
    pub current_player: u8,
    pub wave: u8,
    pub player: PlayerSnapshot,
    pub scores: ScoreSnapshot,
    pub world: WorldSnapshot,
}

pub type GameSnapshot = GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    CreditAdded,
    GameStarted,
    DiagnosticsSelected,
    AuditsSelected,
    HighScoreReset,
    ReversePressed,
    FirePressed,
    SmartBombPressed,
    HyperspacePressed,
    EnemyDestroyed,
    WaveCleared,
    WaveStarted,
    BonusAwarded,
    HighScoreEntryStarted,
    HighScoreInitialAccepted,
    HighScoreSubmitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEvent {
    Startup,
    CreditAdded,
    GameStarted,
    ThrustStarted,
    ThrustStopped,
    UnmappedSoundCommand { command: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEvents {
    gameplay: Vec<GameEvent>,
    sounds: Vec<SoundEvent>,
}

impl GameEvents {
    pub fn new(gameplay: Vec<GameEvent>, sounds: Vec<SoundEvent>) -> Self {
        Self { gameplay, sounds }
    }

    pub fn gameplay(&self) -> &[GameEvent] {
        &self.gameplay
    }

    pub fn sounds(&self) -> &[SoundEvent] {
        &self.sounds
    }

    pub fn is_empty(&self) -> bool {
        self.gameplay.is_empty() && self.sounds.is_empty()
    }
}

impl Default for GameEvents {
    fn default() -> Self {
        Self::new(Vec::new(), Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameFrame {
    pub state: GameState,
    pub events: GameEvents,
    pub scene: RenderScene,
}

#[derive(Debug, Clone)]
pub struct Game {
    state: GameState,
    controls: PlayerControlSystem,
    camera_left: WorldVector,
    pending_wave_start: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: initial_state(),
            controls: PlayerControlSystem::new(),
            camera_left: WorldVector::default(),
            pending_wave_start: false,
        }
    }

    pub fn state(&self) -> GameState {
        self.state.clone()
    }

    pub fn step(&mut self, input: GameInput) -> GameFrame {
        self.state.frame = self.state.frame.saturating_add(1);
        let mut gameplay_events = Vec::new();
        let mut sound_events = Vec::new();

        if input.coin {
            self.state.credits = self.state.credits.saturating_add(1);
            gameplay_events.push(GameEvent::CreditAdded);
            sound_events.push(SoundEvent::CreditAdded);
        }

        if self.state.phase == GamePhase::Attract && input.start_one && self.state.credits > 0 {
            self.start_one_player_game();
            gameplay_events.push(GameEvent::GameStarted);
            sound_events.push(SoundEvent::GameStarted);
        }

        if self.state.phase == GamePhase::Playing {
            self.step_playing(input, &mut gameplay_events, &mut sound_events);
        }

        GameFrame {
            state: self.state.clone(),
            events: GameEvents::new(gameplay_events, sound_events),
            scene: self.scene(),
        }
    }

    fn start_one_player_game(&mut self) {
        self.state.credits = self.state.credits.saturating_sub(1);
        self.state.phase = GamePhase::Playing;
        self.state.current_player = 1;
        self.state.wave = 1;
        self.state.player = PlayerSnapshot {
            position: (world_word(0x2000), world_word(0x8000)),
            velocity: (WorldVector::default(), WorldVector::default()),
            direction: Direction::Right,
            lives: 3,
            smart_bombs: 3,
        };
        self.state.world = WorldSnapshot::first_wave();
        self.camera_left = WorldVector::default();
        self.controls = PlayerControlSystem::new();
        self.pending_wave_start = false;
    }

    fn step_playing(
        &mut self,
        input: GameInput,
        gameplay_events: &mut Vec<GameEvent>,
        sound_events: &mut Vec<SoundEvent>,
    ) {
        if self.pending_wave_start {
            self.start_pending_wave(gameplay_events);
        }

        let controls = self.controls.step(input);

        if controls.triggers.reverse {
            self.state.player.direction = match self.state.player.direction {
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
            };
            gameplay_events.push(GameEvent::ReversePressed);
        }

        let motion = PlayerMotionSystem::step(
            PlayerMotionState::new(
                self.state.player.position,
                self.state.player.velocity,
                self.state.player.direction,
                self.camera_left,
            ),
            controls.intent,
        );
        self.state.player.position = motion.state.position;
        self.state.player.velocity = motion.state.velocity;
        self.camera_left = motion.state.camera_left;

        self.advance_projectiles();

        if controls.triggers.fire {
            gameplay_events.push(GameEvent::FirePressed);
            if let ProjectileLaunchOutcome::Started {
                direction, spawn, ..
            } = ProjectileSystem::try_launch(
                ProjectileState::new(self.state.world.projectiles.len() as u8),
                motion.screen_position,
                self.state.player.direction,
            ) {
                self.state.world.projectiles.push(ProjectileSnapshot {
                    position: spawn,
                    velocity: ProjectileMotionSystem::velocity_for_direction(direction),
                });
            }
        }

        if controls.triggers.smart_bomb && self.state.player.smart_bombs > 0 {
            self.state.player.smart_bombs -= 1;
            gameplay_events.push(GameEvent::SmartBombPressed);
        }

        if controls.triggers.hyperspace {
            gameplay_events.push(GameEvent::HyperspacePressed);
        }

        if controls.triggers.thrust {
            sound_events.push(SoundEvent::ThrustStarted);
        }

        for enemy in &mut self.state.world.enemies {
            let motion = EnemyMotionSystem::step(enemy.position, enemy.velocity);
            enemy.position = motion.position;
            enemy.velocity = motion.velocity;
        }

        self.resolve_projectile_enemy_collisions(gameplay_events);
        self.queue_wave_clear_if_needed(gameplay_events);
    }

    fn advance_projectiles(&mut self) {
        self.state.world.projectiles.retain_mut(|projectile| {
            let motion = ProjectileMotionSystem::step(projectile.position, projectile.velocity);
            if motion.active {
                projectile.position = motion.position;
                projectile.velocity = motion.velocity;
            }
            motion.active
        });
    }

    fn resolve_projectile_enemy_collisions(&mut self, gameplay_events: &mut Vec<GameEvent>) {
        let projectile_boxes = self
            .state
            .world
            .projectiles
            .iter()
            .map(|projectile| CollisionBox::new(projectile.position, PROJECTILE_SPRITE_SIZE))
            .collect::<Vec<_>>();
        let enemy_boxes = self
            .state
            .world
            .enemies
            .iter()
            .map(|enemy| CollisionBox::new(enemy.position, enemy_sprite_size(enemy.kind)))
            .collect::<Vec<_>>();

        let Some(hit) =
            CollisionSystem::first_projectile_enemy_hit(&projectile_boxes, &enemy_boxes)
        else {
            return;
        };

        let enemy = self.state.world.enemies.remove(hit.enemy_index);
        self.state.world.projectiles.remove(hit.projectile_index);
        gameplay_events.push(GameEvent::EnemyDestroyed);
        self.award_enemy_score(enemy.kind, gameplay_events);
    }

    fn queue_wave_clear_if_needed(&mut self, gameplay_events: &mut Vec<GameEvent>) {
        if matches!(
            WaveSystem::evaluate(WaveState::new(
                self.state.wave,
                self.state.world.enemies.len()
            )),
            WaveStatus::Cleared { .. }
        ) {
            self.pending_wave_start = true;
            gameplay_events.push(GameEvent::WaveCleared);
        }
    }

    fn start_pending_wave(&mut self, gameplay_events: &mut Vec<GameEvent>) {
        let next_wave = WaveSystem::next_wave(self.state.wave);

        self.state.wave = next_wave;
        self.state.world = WorldSnapshot::for_wave(next_wave);
        self.pending_wave_start = false;
        gameplay_events.push(GameEvent::WaveStarted);
    }

    fn award_enemy_score(&mut self, kind: EnemyKind, gameplay_events: &mut Vec<GameEvent>) {
        let frame = ScoreSystem::award_points(
            self.state.scores,
            PlayerStock::new(self.state.player.lives, self.state.player.smart_bombs),
            self.state.current_player,
            enemy_score(kind),
        );
        self.state.scores = frame.scores;
        self.state.player.lives = frame.stock.lives;
        self.state.player.smart_bombs = frame.stock.smart_bombs;

        if frame.bonus_awards > 0 {
            gameplay_events.push(GameEvent::BonusAwarded);
        }
    }

    fn scene(&self) -> RenderScene {
        let mut scene = RenderScene::empty(self.state.frame, SurfaceSize::new(292, 240));
        for star in &self.state.world.stars {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::STAR,
                layer: RenderLayer::Starfield,
                position: [f32::from(star.x), f32::from(star.y)],
                size: [1.0, 1.0],
                tint: Color::WHITE,
            });
        }
        for terrain in &self.state.world.terrain {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::TERRAIN_TILE,
                layer: RenderLayer::Terrain,
                position: [f32::from(terrain.position.x), f32::from(terrain.position.y)],
                size: [f32::from(terrain.size.0), f32::from(terrain.size.1)],
                tint: Color::from_rgba(0x26, 0xAE, 0x00, 0xFF),
            });
        }
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::SCORE_TEXT,
            layer: RenderLayer::Hud,
            position: [0.0, 0.0],
            size: [96.0, 8.0],
            tint: Color::WHITE,
        });

        if self.state.phase == GamePhase::Playing {
            for enemy in &self.state.world.enemies {
                let size = enemy_sprite_size(enemy.kind);
                scene.push_sprite(SceneSprite {
                    sprite: enemy_sprite(enemy.kind),
                    layer: RenderLayer::Objects,
                    position: [f32::from(enemy.position.x), f32::from(enemy.position.y)],
                    size: [f32::from(size.0), f32::from(size.1)],
                    tint: Color::from_rgba(0xF4, 0x5B, 0x5B, 0xFF),
                });
            }
            for human in &self.state.world.humans {
                scene.push_sprite(SceneSprite {
                    sprite: SpriteId::HUMAN,
                    layer: RenderLayer::Objects,
                    position: [f32::from(human.position.x), f32::from(human.position.y)],
                    size: [6.0, 8.0],
                    tint: if human.carried {
                        Color::from_rgba(0xFF, 0xF8, 0x80, 0xFF)
                    } else {
                        Color::from_rgba(0x7C, 0xD7, 0xFF, 0xFF)
                    },
                });
            }
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::PLAYER_SHIP,
                layer: RenderLayer::Objects,
                position: [
                    world_vector_pixels(self.state.player.position.0),
                    world_vector_pixels(self.state.player.position.1),
                ],
                size: [16.0, 8.0],
                tint: Color::WHITE,
            });

            for projectile in &self.state.world.projectiles {
                scene.push_sprite(SceneSprite {
                    sprite: SpriteId::PLAYER_PROJECTILE,
                    layer: RenderLayer::Projectiles,
                    position: [
                        f32::from(projectile.position.x),
                        f32::from(projectile.position.y),
                    ],
                    size: [
                        f32::from(PROJECTILE_SPRITE_SIZE.0),
                        f32::from(PROJECTILE_SPRITE_SIZE.1),
                    ],
                    tint: Color::WHITE,
                });
            }
        }

        scene
    }
}

fn enemy_sprite(kind: EnemyKind) -> SpriteId {
    match kind {
        EnemyKind::Lander => SpriteId::ENEMY_LANDER,
    }
}

const PROJECTILE_SPRITE_SIZE: (u8, u8) = (8, 2);

fn enemy_sprite_size(kind: EnemyKind) -> (u8, u8) {
    match kind {
        EnemyKind::Lander => (12, 8),
    }
}

fn enemy_score(kind: EnemyKind) -> u32 {
    match kind {
        EnemyKind::Lander => 150,
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl GameSimulation for Game {
    fn state(&self) -> GameState {
        self.state()
    }

    fn step(&mut self, input: GameInput) -> GameFrame {
        Game::step(self, input)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameInput {
    pub coin: bool,
    pub coin_two: bool,
    pub coin_three: bool,
    pub start_one: bool,
    pub start_two: bool,
    pub altitude_up: bool,
    pub altitude_down: bool,
    pub reverse: bool,
    pub thrust: bool,
    pub fire: bool,
    pub smart_bomb: bool,
    pub hyperspace: bool,
    pub service_auto_up: bool,
    pub service_advance: bool,
    pub high_score_reset: bool,
    pub tilt: bool,
}

impl GameInput {
    pub const NONE: Self = Self {
        coin: false,
        coin_two: false,
        coin_three: false,
        start_one: false,
        start_two: false,
        altitude_up: false,
        altitude_down: false,
        reverse: false,
        thrust: false,
        fire: false,
        smart_bomb: false,
        hyperspace: false,
        service_auto_up: false,
        service_advance: false,
        high_score_reset: false,
        tilt: false,
    };
}

fn initial_state() -> GameState {
    GameState {
        frame: 0,
        phase: GamePhase::Attract,
        credits: 0,
        current_player: 1,
        wave: 0,
        player: PlayerSnapshot {
            position: (world_word(0), world_word(0)),
            velocity: (WorldVector::default(), WorldVector::default()),
            direction: Direction::Right,
            lives: 0,
            smart_bombs: 0,
        },
        scores: ScoreSnapshot {
            player_one: 0,
            player_two: 0,
            high_score: 0,
            next_bonus: 10_000,
        },
        world: WorldSnapshot::default(),
    }
}

fn world_word(word: u16) -> WorldVector {
    WorldVector::from_subpixels(i32::from(word) << 8)
}

fn world_vector_pixels(vector: WorldVector) -> f32 {
    vector.subpixels() as f32 / WorldVector::SUBPIXELS_PER_PIXEL as f32
}

#[cfg(test)]
mod tests {
    use crate::{
        renderer::{Color, RenderLayerCounts, RenderScene, SpriteId, SurfaceSize, TextureAtlas},
        systems::{GameSimulation, ScreenPosition, ScreenVelocity, advance_one_frame},
    };

    use super::{
        Direction, EnemyKind, Game, GameEvent, GameEvents, GameFrame, GameInput, GamePhase,
        ProjectileSnapshot, SoundEvent, WorldSnapshot, WorldVector,
    };

    #[test]
    fn world_vectors_preserve_subpixel_units() {
        let vector = WorldVector::from_subpixels(512);

        assert_eq!(vector.subpixels(), 512);
        assert_eq!(WorldVector::SUBPIXELS_PER_PIXEL, 256);
    }

    #[test]
    fn game_input_none_is_empty() {
        assert_eq!(GameInput::NONE, GameInput::default());
    }

    #[test]
    fn game_events_keep_gameplay_and_sound_surfaces_separate() {
        let events = GameEvents::new(vec![GameEvent::CreditAdded], vec![SoundEvent::CreditAdded]);

        assert_eq!(events.gameplay(), &[GameEvent::CreditAdded]);
        assert_eq!(events.sounds(), &[SoundEvent::CreditAdded]);
        assert!(!events.is_empty());
        assert!(GameEvents::default().is_empty());
    }

    #[test]
    fn game_frame_uses_state_events_and_scene_contracts() {
        let frame = GameFrame {
            state: super::GameState {
                frame: 9,
                phase: GamePhase::Attract,
                credits: 1,
                current_player: 1,
                wave: 0,
                player: super::PlayerSnapshot {
                    position: (WorldVector::default(), WorldVector::default()),
                    velocity: (WorldVector::default(), WorldVector::default()),
                    direction: super::Direction::Right,
                    lives: 3,
                    smart_bombs: 3,
                },
                scores: super::ScoreSnapshot {
                    player_one: 0,
                    player_two: 0,
                    high_score: 100,
                    next_bonus: 10_000,
                },
                world: WorldSnapshot::default(),
            },
            events: GameEvents::default(),
            scene: RenderScene::empty(9, SurfaceSize::new(292, 240)),
        };

        assert_eq!(frame.state.frame, frame.scene.frame);
        assert!(frame.events.is_empty());
    }

    #[test]
    fn clean_game_starts_from_domain_state() {
        let game = Game::new();
        let state = game.state();

        assert_eq!(state.frame, 0);
        assert_eq!(state.phase, GamePhase::Attract);
        assert_eq!(state.credits, 0);
        assert_eq!(state.current_player, 1);
        assert_eq!(state.player.direction, Direction::Right);
        assert_eq!(state.player.lives, 0);
        assert_eq!(state.world, WorldSnapshot::default());
        assert_eq!(Game::default().state(), state);
    }

    #[test]
    fn clean_game_credits_starts_and_emits_sprite_frame() {
        let mut game = Game::new();

        let credited = game.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.state.frame, 1);
        assert_eq!(credited.state.credits, 1);
        assert_eq!(credited.events.gameplay(), &[GameEvent::CreditAdded]);
        assert_eq!(credited.events.sounds(), &[SoundEvent::CreditAdded]);
        assert_eq!(credited.scene.summary().layers.hud, 1);
        assert_eq!(credited.scene.summary().raster_count, 0);

        let started = game.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.state.phase, GamePhase::Playing);
        assert_eq!(started.state.credits, 0);
        assert_eq!(started.state.wave, 1);
        assert_eq!(started.state.player.lives, 3);
        assert_eq!(started.state.player.smart_bombs, 3);
        assert_eq!(started.state.world.terrain.len(), 5);
        assert_eq!(started.state.world.stars.len(), 3);
        assert_eq!(started.state.world.enemies.len(), 1);
        assert_eq!(started.state.world.enemies[0].kind, EnemyKind::Lander);
        assert_eq!(
            started.state.world.enemies[0].velocity,
            ScreenVelocity::new(-1, 0)
        );
        assert_eq!(started.state.world.humans.len(), 2);
        assert!(started.state.world.projectiles.is_empty());
        assert_eq!(started.events.gameplay(), &[GameEvent::GameStarted]);
        assert_eq!(started.events.sounds(), &[SoundEvent::GameStarted]);
        assert_eq!(
            started.scene.summary().layers,
            RenderLayerCounts {
                terrain: 5,
                starfield: 3,
                objects: 4,
                hud: 1,
                ..RenderLayerCounts::default()
            }
        );
        assert_eq!(started.scene.summary().sprite_count, 13);
    }

    #[test]
    fn clean_game_applies_playing_controls_through_systems() {
        let mut game = credited_started_game();

        let frame = game.step(GameInput {
            altitude_up: true,
            reverse: true,
            thrust: true,
            fire: true,
            smart_bomb: true,
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(frame.state.player.direction, Direction::Left);
        assert_eq!(frame.state.player.smart_bombs, 2);
        assert!(frame.state.player.velocity.1.subpixels() < 0);
        assert_eq!(frame.state.world.projectiles.len(), 1);
        assert_eq!(
            frame.state.world.projectiles[0].velocity,
            ScreenVelocity::new(-8, 0)
        );
        assert_eq!(
            frame.events.gameplay(),
            &[
                GameEvent::ReversePressed,
                GameEvent::FirePressed,
                GameEvent::SmartBombPressed,
                GameEvent::HyperspacePressed,
            ]
        );
        assert_eq!(frame.events.sounds(), &[SoundEvent::ThrustStarted]);
        assert_eq!(
            frame.scene.summary().layers,
            RenderLayerCounts {
                terrain: 5,
                starfield: 3,
                objects: 4,
                projectiles: 1,
                hud: 1,
                ..RenderLayerCounts::default()
            }
        );
        assert_eq!(frame.scene.summary().raster_count, 0);
    }

    #[test]
    fn clean_game_advances_projectiles_through_world_snapshots() {
        let mut game = credited_started_game();

        let fired = game.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let projectile = fired.state.world.projectiles[0];

        assert_eq!(projectile.velocity, ScreenVelocity::new(8, 0));
        assert!(fired.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.position
                    == [
                        f32::from(projectile.position.x),
                        f32::from(projectile.position.y),
                    ]
        }));

        let moved = game.step(GameInput::NONE);
        let moved_projectile = moved.state.world.projectiles[0];

        assert_eq!(
            moved_projectile.position.x,
            projectile.position.x.wrapping_add(8)
        );
        assert_eq!(moved_projectile.position.y, projectile.position.y);
        assert_eq!(moved_projectile.velocity, projectile.velocity);
    }

    #[test]
    fn clean_game_culls_projectiles_that_leave_the_screen() {
        let mut game = credited_started_game();
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(252, 80),
            velocity: ScreenVelocity::new(8, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.projectiles.is_empty());
        assert!(
            !frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::PLAYER_PROJECTILE)
        );
    }

    #[test]
    fn clean_game_resolves_projectile_enemy_collision_and_scores() {
        let mut game = credited_started_game();
        game.state.world.enemies[0].position = ScreenPosition::new(100, 80);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.enemies.is_empty());
        assert!(frame.state.world.projectiles.is_empty());
        assert_eq!(frame.state.wave, 1);
        assert_eq!(frame.state.scores.player_one, 150);
        assert_eq!(
            frame.events.gameplay(),
            &[GameEvent::EnemyDestroyed, GameEvent::WaveCleared]
        );
        assert!(
            !frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER)
        );
        assert!(
            !frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::PLAYER_PROJECTILE)
        );
    }

    #[test]
    fn clean_game_scores_current_second_player_on_collision() {
        let mut game = credited_started_game();
        game.state.current_player = 2;
        game.state.world.enemies[0].position = ScreenPosition::new(100, 80);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.scores.player_one, 0);
        assert_eq!(frame.state.scores.player_two, 150);
        assert_eq!(
            frame.events.gameplay(),
            &[GameEvent::EnemyDestroyed, GameEvent::WaveCleared]
        );
    }

    #[test]
    fn clean_game_bonus_award_updates_stock_threshold_and_events() {
        let mut game = credited_started_game();
        game.state.scores.player_one = 9_900;
        game.state.scores.high_score = 9_900;
        game.state.scores.next_bonus = 10_000;
        game.state.player.lives = 3;
        game.state.player.smart_bombs = 1;
        game.state.world.enemies[0].position = ScreenPosition::new(100, 80);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.scores.player_one, 10_050);
        assert_eq!(frame.state.scores.high_score, 10_050);
        assert_eq!(frame.state.scores.next_bonus, 20_000);
        assert_eq!(frame.state.player.lives, 4);
        assert_eq!(frame.state.player.smart_bombs, 2);
        assert_eq!(
            frame.events.gameplay(),
            &[
                GameEvent::EnemyDestroyed,
                GameEvent::BonusAwarded,
                GameEvent::WaveCleared,
            ]
        );
    }

    #[test]
    fn clean_game_wave_clear_delays_next_wave_spawn_until_following_frame() {
        let mut game = credited_started_game();
        game.state.world.enemies[0].position = ScreenPosition::new(100, 80);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let cleared = game.step(GameInput::NONE);

        assert_eq!(cleared.state.wave, 1);
        assert!(cleared.state.world.enemies.is_empty());
        assert_eq!(
            cleared.events.gameplay(),
            &[GameEvent::EnemyDestroyed, GameEvent::WaveCleared]
        );
        assert!(
            !cleared
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER)
        );

        let next_wave = game.step(GameInput::NONE);

        assert_eq!(next_wave.state.wave, 2);
        assert_eq!(next_wave.state.world.enemies.len(), 2);
        assert!(next_wave.state.world.projectiles.is_empty());
        assert_eq!(next_wave.state.world.terrain.len(), 5);
        assert_eq!(next_wave.state.world.humans.len(), 2);
        assert_eq!(next_wave.events.gameplay(), &[GameEvent::WaveStarted]);
        assert_eq!(
            next_wave
                .scene
                .sprites
                .iter()
                .filter(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER)
                .count(),
            2
        );
    }

    #[test]
    fn clean_game_advances_enemy_positions_through_systems() {
        let mut game = credited_started_game();
        let before = game.state.world.enemies[0].position;

        let frame = game.step(GameInput::NONE);
        let enemy = frame.state.world.enemies[0];

        assert_eq!(enemy.position.x, before.x.wrapping_sub(1));
        assert_eq!(enemy.position.y, before.y);
        assert_eq!(enemy.velocity, ScreenVelocity::new(-1, 0));
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER
                && sprite.position == [f32::from(enemy.position.x), f32::from(enemy.position.y)]
        }));
    }

    #[test]
    fn clean_game_world_sprites_are_atlas_backed() {
        let mut game = credited_started_game();

        let frame = game.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let atlas = TextureAtlas::default_sprites();
        for sprite in &frame.scene.sprites {
            assert!(atlas.contains(sprite.sprite), "{:?}", sprite.sprite);
        }
        assert!(
            frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER)
        );
        assert!(
            frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::HUMAN)
        );
        assert!(
            frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::TERRAIN_TILE)
        );
        assert!(
            frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::STAR)
        );
    }

    #[test]
    fn clean_game_highlights_carried_humans() {
        let mut game = credited_started_game();
        game.state.world.humans[0].carried = true;

        let frame = game.step(GameInput::NONE);

        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.tint == Color::from_rgba(0xFF, 0xF8, 0x80, 0xFF)
        }));
    }

    #[test]
    fn clean_game_reverses_left_to_right() {
        let mut game = credited_started_game();
        game.state.player.direction = Direction::Left;

        let frame = game.step(GameInput {
            reverse: true,
            ..GameInput::NONE
        });

        assert_eq!(frame.state.player.direction, Direction::Right);
        assert_eq!(frame.events.gameplay(), &[GameEvent::ReversePressed]);
    }

    #[test]
    fn clean_game_implements_simulation_trait() {
        let mut game = Game::new();

        let frame = advance_one_frame(
            &mut game,
            GameInput {
                coin: true,
                ..GameInput::NONE
            },
        );

        assert_eq!(GameSimulation::state(&game).frame, 1);
        assert_eq!(frame.state.credits, 1);
    }

    fn credited_started_game() -> Game {
        let mut game = Game::new();
        game.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        game.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        game
    }
}
