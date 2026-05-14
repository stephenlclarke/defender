//! Domain-facing gameplay contracts.

use crate::{
    renderer::{Color, RenderLayer, RenderScene, SceneSprite, SpriteId, SurfaceSize},
    systems::{
        GameSimulation, PlayerControlSystem, PlayerMotionState, PlayerMotionSystem,
        ProjectileLaunchOutcome, ProjectileState, ProjectileSystem, ScreenPosition,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub frame: u64,
    pub phase: GamePhase,
    pub credits: u8,
    pub current_player: u8,
    pub wave: u8,
    pub player: PlayerSnapshot,
    pub scores: ScoreSnapshot,
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
    projectiles: Vec<ScreenPosition>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: initial_state(),
            controls: PlayerControlSystem::new(),
            camera_left: WorldVector::default(),
            projectiles: Vec::new(),
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
        self.camera_left = WorldVector::default();
        self.controls = PlayerControlSystem::new();
        self.projectiles.clear();
    }

    fn step_playing(
        &mut self,
        input: GameInput,
        gameplay_events: &mut Vec<GameEvent>,
        sound_events: &mut Vec<SoundEvent>,
    ) {
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

        if controls.triggers.fire {
            gameplay_events.push(GameEvent::FirePressed);
            if let ProjectileLaunchOutcome::Started { spawn, .. } = ProjectileSystem::try_launch(
                ProjectileState::new(self.projectiles.len() as u8),
                motion.screen_position,
                self.state.player.direction,
            ) {
                self.projectiles.push(spawn);
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
    }

    fn scene(&self) -> RenderScene {
        let mut scene = RenderScene::empty(self.state.frame, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::SCORE_TEXT,
            layer: RenderLayer::Hud,
            position: [0.0, 0.0],
            size: [96.0, 8.0],
            tint: Color::WHITE,
        });

        if self.state.phase == GamePhase::Playing {
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

            for projectile in &self.projectiles {
                scene.push_sprite(SceneSprite {
                    sprite: SpriteId::PLAYER_PROJECTILE,
                    layer: RenderLayer::Projectiles,
                    position: [f32::from(projectile.x), f32::from(projectile.y)],
                    size: [8.0, 2.0],
                    tint: Color::WHITE,
                });
            }
        }

        scene
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
        renderer::{RenderLayerCounts, RenderScene, SurfaceSize},
        systems::{GameSimulation, advance_one_frame},
    };

    use super::{
        Direction, Game, GameEvent, GameEvents, GameFrame, GameInput, GamePhase, SoundEvent,
        WorldVector,
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
        assert_eq!(started.events.gameplay(), &[GameEvent::GameStarted]);
        assert_eq!(started.events.sounds(), &[SoundEvent::GameStarted]);
        assert_eq!(
            started.scene.summary().layers,
            RenderLayerCounts {
                objects: 1,
                hud: 1,
                ..RenderLayerCounts::default()
            }
        );
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
                objects: 1,
                projectiles: 1,
                hud: 1,
                ..RenderLayerCounts::default()
            }
        );
        assert_eq!(frame.scene.summary().raster_count, 0);
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
