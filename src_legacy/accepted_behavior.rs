//! Bridge from the neutral accepted-behavior contracts to the current oracle.

use crate::{
    accepted::{
        AcceptedDirection, AcceptedEvent, AcceptedFrame, AcceptedPhase, AcceptedPlayer,
        AcceptedScores, AcceptedSnapshot,
    },
    machine_state::{self, MachineEvent, MachineSnapshot},
    red_label::Facing,
    video,
};

#[cfg(test)]
use crate::{game::GameInput, input::CabinetInput, machine::ArcadeMachine};

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct AcceptedMachineAdapter {
    machine: ArcadeMachine,
}

#[cfg(test)]
impl AcceptedMachineAdapter {
    pub(crate) fn new() -> Self {
        Self {
            machine: ArcadeMachine::new(),
        }
    }

    pub(crate) fn snapshot(&self) -> AcceptedSnapshot {
        AcceptedSnapshot::from(self.machine.snapshot())
    }

    pub(crate) fn step(&mut self, input: GameInput) -> AcceptedFrame {
        AcceptedFrame::from(self.machine.step(to_cabinet_input(input)))
    }
}

impl From<machine_state::FrameOutput> for AcceptedFrame {
    fn from(output: machine_state::FrameOutput) -> Self {
        Self {
            snapshot: AcceptedSnapshot::from(output.snapshot),
            events: output.events().map(AcceptedEvent::from).collect(),
            sound_commands: output
                .sound_commands()
                .map(|command| command.raw())
                .collect(),
            visual_signature: output.video_crc32,
        }
    }
}

impl From<MachineSnapshot> for AcceptedSnapshot {
    fn from(snapshot: MachineSnapshot) -> Self {
        Self {
            frame: snapshot.frame,
            phase: AcceptedPhase::from(snapshot.phase),
            credits: snapshot.credits,
            current_player: snapshot.current_player,
            wave: snapshot.wave,
            player: AcceptedPlayer {
                x_subpixels: snapshot.player.x.0,
                y_subpixels: snapshot.player.y.0,
                x_velocity_subpixels: snapshot.player.xv.0,
                y_velocity_subpixels: snapshot.player.yv.0,
                direction: AcceptedDirection::from(snapshot.player.facing),
                lives: snapshot.player.lives,
                smart_bombs: snapshot.player.smart_bombs,
            },
            scores: AcceptedScores {
                player_one: snapshot.scores.player_one,
                player_two: snapshot.scores.player_two,
                high_score: snapshot.scores.high_score,
                next_bonus: snapshot.scores.next_bonus,
            },
        }
    }
}

impl From<machine_state::GamePhase> for AcceptedPhase {
    fn from(phase: machine_state::GamePhase) -> Self {
        match phase {
            machine_state::GamePhase::Attract => Self::Attract,
            machine_state::GamePhase::Playing => Self::Playing,
            machine_state::GamePhase::GameOver => Self::GameOver,
            machine_state::GamePhase::HighScoreEntry => Self::HighScoreEntry,
        }
    }
}

impl From<Facing> for AcceptedDirection {
    fn from(direction: Facing) -> Self {
        match direction {
            Facing::Left => Self::Left,
            Facing::Right => Self::Right,
        }
    }
}

impl From<MachineEvent> for AcceptedEvent {
    fn from(event: MachineEvent) -> Self {
        match event {
            MachineEvent::CreditAdded => Self::CreditAdded,
            MachineEvent::GameStarted => Self::GameStarted,
            MachineEvent::DiagnosticsSelected => Self::DiagnosticsSelected,
            MachineEvent::AuditsSelected => Self::AuditsSelected,
            MachineEvent::HighScoreReset => Self::HighScoreReset,
            MachineEvent::ReversePressed => Self::ReversePressed,
            MachineEvent::FirePressed => Self::FirePressed,
            MachineEvent::SmartBombPressed => Self::SmartBombPressed,
            MachineEvent::HyperspacePressed => Self::HyperspacePressed,
            MachineEvent::BonusAwarded => Self::BonusAwarded,
            MachineEvent::HighScoreEntryStarted => Self::HighScoreEntryStarted,
            MachineEvent::HighScoreInitialAccepted => Self::HighScoreInitialAccepted,
            MachineEvent::HighScoreSubmitted => Self::HighScoreSubmitted,
        }
    }
}

pub(crate) fn native_visible_size() -> (u16, u16) {
    video::native_visible_size()
}

#[cfg(test)]
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

#[cfg(test)]
pub(crate) fn cabinet_input_for_test(input: GameInput) -> CabinetInput {
    to_cabinet_input(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        accepted::{
            AcceptedDirection, AcceptedEvent, AcceptedFrame, AcceptedGameplayMachine,
            AcceptedPhase, AcceptedSnapshot,
        },
        input::CabinetInput,
        machine_state::{self, MachineEvent},
        red_label::Facing,
    };

    use super::to_cabinet_input;
    use crate::game::GameInput;

    #[test]
    fn accepted_input_maps_every_clean_control() {
        let cabinet = to_cabinet_input(GameInput {
            coin: true,
            coin_two: true,
            coin_three: true,
            start_one: true,
            start_two: true,
            altitude_up: true,
            altitude_down: true,
            reverse: true,
            thrust: true,
            fire: true,
            smart_bomb: true,
            hyperspace: true,
            service_auto_up: true,
            service_advance: true,
            high_score_reset: true,
            tilt: true,
        });

        assert_eq!(
            cabinet,
            CabinetInput {
                coin: true,
                coin_two: true,
                coin_three: true,
                start_one: true,
                start_two: true,
                altitude_up: true,
                altitude_down: true,
                reverse: true,
                thrust: true,
                fire: true,
                smart_bomb: true,
                hyperspace: true,
                auto_up_manual_down: true,
                service_advance: true,
                high_score_reset: true,
                tilt: true,
            }
        );
    }

    #[test]
    fn accepted_phase_maps_all_current_phase_variants() {
        assert_eq!(
            AcceptedPhase::from(machine_state::GamePhase::Attract),
            AcceptedPhase::Attract
        );
        assert_eq!(
            AcceptedPhase::from(machine_state::GamePhase::Playing),
            AcceptedPhase::Playing
        );
        assert_eq!(
            AcceptedPhase::from(machine_state::GamePhase::GameOver),
            AcceptedPhase::GameOver
        );
        assert_eq!(
            AcceptedPhase::from(machine_state::GamePhase::HighScoreEntry),
            AcceptedPhase::HighScoreEntry
        );
    }

    #[test]
    fn accepted_direction_maps_both_current_direction_variants() {
        assert_eq!(
            AcceptedDirection::from(Facing::Left),
            AcceptedDirection::Left
        );
        assert_eq!(
            AcceptedDirection::from(Facing::Right),
            AcceptedDirection::Right
        );
    }

    #[test]
    fn accepted_event_maps_all_current_event_variants() {
        let pairs = [
            (MachineEvent::CreditAdded, AcceptedEvent::CreditAdded),
            (MachineEvent::GameStarted, AcceptedEvent::GameStarted),
            (
                MachineEvent::DiagnosticsSelected,
                AcceptedEvent::DiagnosticsSelected,
            ),
            (MachineEvent::AuditsSelected, AcceptedEvent::AuditsSelected),
            (MachineEvent::HighScoreReset, AcceptedEvent::HighScoreReset),
            (MachineEvent::ReversePressed, AcceptedEvent::ReversePressed),
            (MachineEvent::FirePressed, AcceptedEvent::FirePressed),
            (
                MachineEvent::SmartBombPressed,
                AcceptedEvent::SmartBombPressed,
            ),
            (
                MachineEvent::HyperspacePressed,
                AcceptedEvent::HyperspacePressed,
            ),
            (MachineEvent::BonusAwarded, AcceptedEvent::BonusAwarded),
            (
                MachineEvent::HighScoreEntryStarted,
                AcceptedEvent::HighScoreEntryStarted,
            ),
            (
                MachineEvent::HighScoreInitialAccepted,
                AcceptedEvent::HighScoreInitialAccepted,
            ),
            (
                MachineEvent::HighScoreSubmitted,
                AcceptedEvent::HighScoreSubmitted,
            ),
        ];

        for (legacy, accepted) in pairs {
            assert_eq!(AcceptedEvent::from(legacy), accepted);
        }
    }

    #[test]
    fn accepted_frame_owns_snapshot_sounds_and_visual_signature() {
        let mut machine = crate::machine::ArcadeMachine::new();
        let output = machine.step(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });

        let frame = AcceptedFrame::from(output);

        assert_eq!(frame.snapshot.frame, 1);
        assert!(frame.sound_commands.is_empty());
        assert!(frame.visual_signature.is_some());
    }

    #[test]
    fn accepted_snapshot_carries_clean_direction_and_score_fields() {
        let mut snapshot = crate::machine::ArcadeMachine::new().snapshot();
        snapshot.phase = machine_state::GamePhase::Playing;
        snapshot.player.facing = Facing::Left;
        snapshot.player.x.0 = 0x1234;
        snapshot.player.y.0 = 0x5678;
        snapshot.player.xv.0 = 0x0100;
        snapshot.player.yv.0 = -0x0200;
        snapshot.scores.player_one = 100;

        let accepted = AcceptedSnapshot::from(snapshot);

        assert_eq!(accepted.phase, AcceptedPhase::Playing);
        assert_eq!(accepted.player.direction, AcceptedDirection::Left);
        assert_eq!(accepted.player.x_subpixels, 0x1234);
        assert_eq!(accepted.player.y_subpixels, 0x5678);
        assert_eq!(accepted.player.x_velocity_subpixels, 0x0100);
        assert_eq!(accepted.player.y_velocity_subpixels, -0x0200);
        assert_eq!(accepted.scores.player_one, 100);
    }

    #[test]
    fn accepted_adapter_reaches_current_machine() {
        let mut machine = AcceptedGameplayMachine::new();
        let frame = machine.step(GameInput::NONE);

        assert_eq!(frame.snapshot.frame, 1);
        assert!(frame.visual_signature.is_some());
    }
}
