//! Domain-facing gameplay contracts.

use crate::renderer::RenderScene;

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
    UnmappedAcceptedCommand { command: u8 },
}

impl SoundEvent {
    pub const fn from_accepted_command(command: u8) -> Self {
        match command {
            0xC0 => Self::Startup,
            0xE6 => Self::CreditAdded,
            0xF5 => Self::GameStarted,
            0xE9 => Self::ThrustStarted,
            0xF0 => Self::ThrustStopped,
            command => Self::UnmappedAcceptedCommand { command },
        }
    }

    pub const fn accepted_command(self) -> u8 {
        match self {
            Self::Startup => 0xC0,
            Self::CreditAdded => 0xE6,
            Self::GameStarted => 0xF5,
            Self::ThrustStarted => 0xE9,
            Self::ThrustStopped => 0xF0,
            Self::UnmappedAcceptedCommand { command } => command,
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::renderer::{RenderScene, SurfaceSize};

    use super::{GameEvent, GameEvents, GameFrame, GameInput, GamePhase, SoundEvent, WorldVector};

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
    fn sound_events_map_accepted_command_bytes_at_oracle_boundary() {
        assert_eq!(SoundEvent::from_accepted_command(0xC0), SoundEvent::Startup);
        assert_eq!(
            SoundEvent::from_accepted_command(0xE6),
            SoundEvent::CreditAdded
        );
        assert_eq!(
            SoundEvent::from_accepted_command(0xF5),
            SoundEvent::GameStarted
        );
        assert_eq!(
            SoundEvent::from_accepted_command(0xE9),
            SoundEvent::ThrustStarted
        );
        assert_eq!(
            SoundEvent::from_accepted_command(0xF0),
            SoundEvent::ThrustStopped
        );
        assert_eq!(
            SoundEvent::from_accepted_command(0x3E),
            SoundEvent::UnmappedAcceptedCommand { command: 0x3E }
        );
        assert_eq!(SoundEvent::Startup.accepted_command(), 0xC0);
        assert_eq!(SoundEvent::CreditAdded.accepted_command(), 0xE6);
        assert_eq!(SoundEvent::GameStarted.accepted_command(), 0xF5);
        assert_eq!(SoundEvent::ThrustStarted.accepted_command(), 0xE9);
        assert_eq!(SoundEvent::ThrustStopped.accepted_command(), 0xF0);
        assert_eq!(
            SoundEvent::UnmappedAcceptedCommand { command: 0x3E }.accepted_command(),
            0x3E
        );
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
}
