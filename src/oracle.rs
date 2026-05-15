//! Temporary gameplay oracle backed by the current implementation.
//!
//! This module is the explicit boundary where the clean rewrite can compare
//! against the existing behavior without letting converted implementation names
//! leak into new production contracts.

#[cfg(test)]
use crate::accepted::AcceptedGameplayMachine;
use crate::accepted::{
    AcceptedDirection, AcceptedEvent, AcceptedFrame, AcceptedPhase, AcceptedSnapshot,
};

use crate::game::{
    Direction, GameEvent, GameEvents, GameFrame, GamePhase, GameState, PlayerSnapshot,
    ScoreSnapshot, SoundEvent, WorldSnapshot, WorldVector,
};
use crate::renderer::{Color, RenderLayer, RenderScene, SceneSprite, SpriteId, SurfaceSize};
#[cfg(test)]
use crate::{game::GameInput, systems::GameSimulation};

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct GameplayOracle {
    machine: AcceptedGameplayMachine,
}

#[cfg(test)]
impl GameplayOracle {
    pub(crate) fn new() -> Self {
        Self {
            machine: AcceptedGameplayMachine::new(),
        }
    }

    pub(crate) fn snapshot(&self) -> GameState {
        adapt_snapshot(self.machine.snapshot())
    }

    pub(crate) fn step(&mut self, input: GameInput) -> GameFrame {
        adapt_frame_output(self.machine.step(input))
    }
}

#[cfg(test)]
impl Default for GameplayOracle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl GameSimulation for GameplayOracle {
    fn state(&self) -> GameState {
        self.snapshot()
    }

    fn step(&mut self, input: GameInput) -> GameFrame {
        GameplayOracle::step(self, input)
    }
}

pub(crate) fn game_frame_from_accepted(output: AcceptedFrame) -> GameFrame {
    adapt_frame_output(output)
}

fn adapt_frame_output(output: AcceptedFrame) -> GameFrame {
    let state = adapt_snapshot(output.snapshot);
    let scene = adapt_scene(&state, output.visual_signature);

    GameFrame {
        state,
        events: GameEvents::new(
            output.events.into_iter().map(adapt_event).collect(),
            output
                .sound_commands
                .into_iter()
                .map(adapt_sound_command)
                .collect(),
        ),
        scene,
    }
}

fn adapt_snapshot(snapshot: AcceptedSnapshot) -> GameState {
    GameState {
        frame: snapshot.frame,
        phase: adapt_phase(snapshot.phase),
        credits: snapshot.credits,
        current_player: snapshot.current_player,
        wave: snapshot.wave,
        player: PlayerSnapshot {
            position: (
                WorldVector::from_subpixels(snapshot.player.x_subpixels),
                WorldVector::from_subpixels(snapshot.player.y_subpixels),
            ),
            velocity: (
                WorldVector::from_subpixels(snapshot.player.x_velocity_subpixels),
                WorldVector::from_subpixels(snapshot.player.y_velocity_subpixels),
            ),
            direction: adapt_direction(snapshot.player.direction),
            lives: snapshot.player.lives,
            smart_bombs: snapshot.player.smart_bombs,
        },
        scores: ScoreSnapshot {
            player_one: snapshot.scores.player_one,
            player_two: snapshot.scores.player_two,
            high_score: snapshot.scores.high_score,
            next_bonus: snapshot.scores.next_bonus,
        },
        world: WorldSnapshot::default(),
    }
}

fn adapt_scene(state: &GameState, visual_signature: Option<u32>) -> RenderScene {
    let (width, height) = crate::accepted::native_visible_size();
    let mut scene = RenderScene::empty(
        state.frame,
        SurfaceSize::new(u32::from(width), u32::from(height)),
    );
    scene.visual_signature = visual_signature;
    scene.push_sprite(SceneSprite {
        sprite: SpriteId::SCORE_TEXT,
        layer: RenderLayer::Hud,
        position: [0.0, 0.0],
        size: [96.0, 8.0],
        tint: Color::WHITE,
    });

    if state.phase == GamePhase::Playing {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [
                world_vector_pixels(state.player.position.0),
                world_vector_pixels(state.player.position.1),
            ],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });
    }

    scene
}

fn world_vector_pixels(vector: WorldVector) -> f32 {
    vector.subpixels() as f32 / WorldVector::SUBPIXELS_PER_PIXEL as f32
}

fn adapt_phase(phase: AcceptedPhase) -> GamePhase {
    match phase {
        AcceptedPhase::Attract => GamePhase::Attract,
        AcceptedPhase::Playing => GamePhase::Playing,
        AcceptedPhase::GameOver => GamePhase::GameOver,
        AcceptedPhase::HighScoreEntry => GamePhase::HighScoreEntry,
    }
}

fn adapt_direction(direction: AcceptedDirection) -> Direction {
    match direction {
        AcceptedDirection::Left => Direction::Left,
        AcceptedDirection::Right => Direction::Right,
    }
}

fn adapt_event(event: AcceptedEvent) -> GameEvent {
    match event {
        AcceptedEvent::CreditAdded => GameEvent::CreditAdded,
        AcceptedEvent::GameStarted => GameEvent::GameStarted,
        AcceptedEvent::DiagnosticsSelected => GameEvent::DiagnosticsSelected,
        AcceptedEvent::AuditsSelected => GameEvent::AuditsSelected,
        AcceptedEvent::HighScoreReset => GameEvent::HighScoreReset,
        AcceptedEvent::ReversePressed => GameEvent::ReversePressed,
        AcceptedEvent::FirePressed => GameEvent::FirePressed,
        AcceptedEvent::SmartBombPressed => GameEvent::SmartBombPressed,
        AcceptedEvent::HyperspacePressed => GameEvent::HyperspacePressed,
        AcceptedEvent::BonusAwarded => GameEvent::BonusAwarded,
        AcceptedEvent::HighScoreEntryStarted => GameEvent::HighScoreEntryStarted,
        AcceptedEvent::HighScoreInitialAccepted => GameEvent::HighScoreInitialAccepted,
        AcceptedEvent::HighScoreSubmitted => GameEvent::HighScoreSubmitted,
    }
}

fn adapt_sound_command(command: u8) -> SoundEvent {
    match command {
        0xC0 => SoundEvent::Startup,
        0xE6 => SoundEvent::CreditAdded,
        0xF5 => SoundEvent::GameStarted,
        0xE9 => SoundEvent::ThrustStarted,
        0xF0 => SoundEvent::ThrustStopped,
        command => SoundEvent::UnmappedSoundCommand { command },
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::{GameEvent, GameFrame, GameInput, GameState, RenderScene, SoundEvent};
    use crate::accepted::{AcceptedEvent, AcceptedGameplayMachine, AcceptedSnapshot};

    #[derive(Debug)]
    pub(crate) struct ReferenceFrameProbe {
        machine: AcceptedGameplayMachine,
    }

    impl ReferenceFrameProbe {
        pub(crate) fn new() -> Self {
            Self {
                machine: AcceptedGameplayMachine::new(),
            }
        }

        pub(crate) fn step(&mut self, input: GameInput) -> GameFrame {
            super::adapt_frame_output(self.machine.step(input))
        }
    }

    pub(crate) fn adapt_accepted_snapshot(snapshot: AcceptedSnapshot) -> GameState {
        super::adapt_snapshot(snapshot)
    }

    pub(crate) fn adapt_accepted_event(event: AcceptedEvent) -> GameEvent {
        super::adapt_event(event)
    }

    pub(crate) fn adapt_accepted_scene(
        state: &GameState,
        visual_signature: Option<u32>,
    ) -> RenderScene {
        super::adapt_scene(state, visual_signature)
    }

    pub(crate) fn adapt_accepted_sound_command(command: u8) -> SoundEvent {
        super::adapt_sound_command(command)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        accepted::{AcceptedDirection, AcceptedEvent, AcceptedPhase},
        systems::{GameSimulation, PlayerActionTriggers, advance_one_frame},
    };

    use super::{
        Direction, GameEvent, GameInput, GamePhase, GameplayOracle, SoundEvent, adapt_direction,
        adapt_event, adapt_phase, adapt_sound_command,
    };

    #[test]
    fn oracle_starts_from_clean_attract_snapshot() {
        let oracle = GameplayOracle::new();
        let snapshot = oracle.snapshot();

        assert_eq!(snapshot.frame, 0);
        assert_eq!(snapshot.phase, GamePhase::Attract);
        assert_eq!(snapshot.current_player, 1);
    }

    #[test]
    fn oracle_steps_through_clean_frame_contract() {
        let mut oracle = GameplayOracle::new();
        let frame = oracle.step(GameInput::NONE);

        assert_eq!(frame.state.frame, 1);
        assert_eq!(frame.state.phase, GamePhase::Attract);
        assert_eq!(frame.scene.summary().frame, 1);
    }

    #[test]
    fn oracle_maps_all_accepted_phase_contracts() {
        assert_eq!(adapt_phase(AcceptedPhase::Attract), GamePhase::Attract);
        assert_eq!(adapt_phase(AcceptedPhase::Playing), GamePhase::Playing);
        assert_eq!(adapt_phase(AcceptedPhase::GameOver), GamePhase::GameOver);
        assert_eq!(
            adapt_phase(AcceptedPhase::HighScoreEntry),
            GamePhase::HighScoreEntry
        );
    }

    #[test]
    fn oracle_maps_all_accepted_direction_contracts() {
        assert_eq!(adapt_direction(AcceptedDirection::Left), Direction::Left);
        assert_eq!(adapt_direction(AcceptedDirection::Right), Direction::Right);
    }

    #[test]
    fn oracle_maps_all_accepted_event_contracts() {
        let pairs = [
            (AcceptedEvent::CreditAdded, GameEvent::CreditAdded),
            (AcceptedEvent::GameStarted, GameEvent::GameStarted),
            (
                AcceptedEvent::DiagnosticsSelected,
                GameEvent::DiagnosticsSelected,
            ),
            (AcceptedEvent::AuditsSelected, GameEvent::AuditsSelected),
            (AcceptedEvent::HighScoreReset, GameEvent::HighScoreReset),
            (AcceptedEvent::ReversePressed, GameEvent::ReversePressed),
            (AcceptedEvent::FirePressed, GameEvent::FirePressed),
            (AcceptedEvent::SmartBombPressed, GameEvent::SmartBombPressed),
            (
                AcceptedEvent::HyperspacePressed,
                GameEvent::HyperspacePressed,
            ),
            (AcceptedEvent::BonusAwarded, GameEvent::BonusAwarded),
            (
                AcceptedEvent::HighScoreEntryStarted,
                GameEvent::HighScoreEntryStarted,
            ),
            (
                AcceptedEvent::HighScoreInitialAccepted,
                GameEvent::HighScoreInitialAccepted,
            ),
            (
                AcceptedEvent::HighScoreSubmitted,
                GameEvent::HighScoreSubmitted,
            ),
        ];

        for (accepted, clean) in pairs {
            assert_eq!(adapt_event(accepted), clean);
        }
    }

    #[test]
    fn oracle_maps_accepted_sound_commands_to_clean_events() {
        assert_eq!(adapt_sound_command(0xC0), SoundEvent::Startup);
        assert_eq!(adapt_sound_command(0xE6), SoundEvent::CreditAdded);
        assert_eq!(adapt_sound_command(0xF5), SoundEvent::GameStarted);
        assert_eq!(adapt_sound_command(0xE9), SoundEvent::ThrustStarted);
        assert_eq!(adapt_sound_command(0xF0), SoundEvent::ThrustStopped);
        assert_eq!(
            adapt_sound_command(0x3E),
            SoundEvent::UnmappedSoundCommand { command: 0x3E }
        );
    }

    #[test]
    fn oracle_implements_clean_simulation_trait() {
        let mut oracle = GameplayOracle::new();

        let frame = advance_one_frame(&mut oracle, GameInput::NONE);

        assert_eq!(frame.state.frame, 1);
        assert_eq!(GameSimulation::state(&oracle).frame, 1);
    }

    #[test]
    fn player_action_triggers_none_is_empty() {
        assert!(!PlayerActionTriggers::NONE.any());
    }
}
