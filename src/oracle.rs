//! Temporary gameplay oracle backed by the current implementation.
//!
//! This module is the explicit boundary where the clean rewrite can compare
//! against the existing behavior without letting converted implementation names
//! leak into new production contracts.

use crate::{
    input::CabinetInput,
    machine::ArcadeMachine,
    machine_state::{FrameOutput, MachineEvent, MachineSnapshot},
    red_label::Facing,
};

use super::game::{
    Direction, GameEvent, GameFrame, GameInput, GamePhase, GameSnapshot, PlayerSnapshot,
    ScoreSnapshot, SoundEvent, WorldVector,
};

#[derive(Debug)]
pub struct GameplayOracle {
    machine: ArcadeMachine,
}

impl GameplayOracle {
    pub fn new() -> Self {
        Self {
            machine: ArcadeMachine::new(),
        }
    }

    pub fn snapshot(&self) -> GameSnapshot {
        adapt_snapshot(self.machine.snapshot())
    }

    pub fn step(&mut self, input: GameInput) -> GameFrame {
        adapt_frame_output(self.machine.step(to_cabinet_input(input)))
    }
}

impl Default for GameplayOracle {
    fn default() -> Self {
        Self::new()
    }
}

fn to_cabinet_input(input: GameInput) -> CabinetInput {
    CabinetInput {
        coin: input.coin,
        coin_two: input.coin_two,
        coin_three: input.coin_three,
        start_one: input.start_one,
        start_two: input.start_two,
        altitude_up: input.altitude_up,
        altitude_down: input.altitude_down,
        reverse: input.reverse,
        thrust: input.thrust,
        fire: input.fire,
        smart_bomb: input.smart_bomb,
        hyperspace: input.hyperspace,
        auto_up_manual_down: input.service_auto_up,
        service_advance: input.service_advance,
        high_score_reset: input.high_score_reset,
        tilt: input.tilt,
    }
}

fn adapt_frame_output(output: FrameOutput) -> GameFrame {
    GameFrame {
        snapshot: adapt_snapshot(output.snapshot),
        events: output.events().map(adapt_event).collect(),
        sounds: output
            .sound_commands()
            .map(|command| SoundEvent {
                command: command.raw(),
            })
            .collect(),
    }
}

fn adapt_snapshot(snapshot: MachineSnapshot) -> GameSnapshot {
    GameSnapshot {
        frame: snapshot.frame,
        phase: adapt_phase(snapshot.phase),
        credits: snapshot.credits,
        current_player: snapshot.current_player,
        wave: snapshot.wave,
        player: PlayerSnapshot {
            position: (
                WorldVector::from_subpixels(snapshot.player.x.0),
                WorldVector::from_subpixels(snapshot.player.y.0),
            ),
            velocity: (
                WorldVector::from_subpixels(snapshot.player.xv.0),
                WorldVector::from_subpixels(snapshot.player.yv.0),
            ),
            direction: adapt_direction(snapshot.player.facing),
            lives: snapshot.player.lives,
            smart_bombs: snapshot.player.smart_bombs,
        },
        scores: ScoreSnapshot {
            player_one: snapshot.scores.player_one,
            player_two: snapshot.scores.player_two,
            high_score: snapshot.scores.high_score,
            next_bonus: snapshot.scores.next_bonus,
        },
    }
}

fn adapt_phase(phase: crate::machine_state::GamePhase) -> GamePhase {
    match phase {
        crate::machine_state::GamePhase::Attract => GamePhase::Attract,
        crate::machine_state::GamePhase::Playing => GamePhase::Playing,
        crate::machine_state::GamePhase::GameOver => GamePhase::GameOver,
        crate::machine_state::GamePhase::HighScoreEntry => GamePhase::HighScoreEntry,
    }
}

fn adapt_direction(direction: Facing) -> Direction {
    match direction {
        Facing::Left => Direction::Left,
        Facing::Right => Direction::Right,
    }
}

fn adapt_event(event: MachineEvent) -> GameEvent {
    match event {
        MachineEvent::CreditAdded => GameEvent::CreditAdded,
        MachineEvent::GameStarted => GameEvent::GameStarted,
        MachineEvent::DiagnosticsSelected => GameEvent::DiagnosticsSelected,
        MachineEvent::AuditsSelected => GameEvent::AuditsSelected,
        MachineEvent::HighScoreReset => GameEvent::HighScoreReset,
        MachineEvent::ReversePressed => GameEvent::ReversePressed,
        MachineEvent::FirePressed => GameEvent::FirePressed,
        MachineEvent::SmartBombPressed => GameEvent::SmartBombPressed,
        MachineEvent::HyperspacePressed => GameEvent::HyperspacePressed,
        MachineEvent::BonusAwarded => GameEvent::BonusAwarded,
        MachineEvent::HighScoreEntryStarted => GameEvent::HighScoreEntryStarted,
        MachineEvent::HighScoreInitialAccepted => GameEvent::HighScoreInitialAccepted,
        MachineEvent::HighScoreSubmitted => GameEvent::HighScoreSubmitted,
    }
}

#[cfg(test)]
mod tests {
    use crate::input::CabinetInput;

    use super::{GameInput, GamePhase, GameplayOracle, to_cabinet_input};

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

        assert_eq!(frame.snapshot.frame, 1);
        assert_eq!(frame.snapshot.phase, GamePhase::Attract);
    }

    #[test]
    fn oracle_maps_clean_input_to_cabinet_input() {
        let input = GameInput {
            coin: true,
            start_one: true,
            fire: true,
            service_auto_up: true,
            ..GameInput::NONE
        };

        let cabinet = to_cabinet_input(input);

        assert_eq!(
            cabinet.bits(),
            CabinetInput {
                coin: true,
                start_one: true,
                fire: true,
                auto_up_manual_down: true,
                ..CabinetInput::NONE
            }
            .bits()
        );
    }
}
