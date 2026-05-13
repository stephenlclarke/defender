//! Neutral facade over the temporary accepted-behavior implementation.
//!
//! The clean rewrite should depend on the contracts in this module, not on
//! legacy module names. This keeps the current machine available as an oracle
//! while making the remaining retirement work explicit and localized.

use crate::{
    compatibility::{
        app,
        input::CabinetInput,
        machine::ArcadeMachine,
        machine_state::{self, MachineEvent, MachineSnapshot},
        red_label::Facing,
        video,
    },
    game::GameInput,
};

#[derive(Debug)]
pub(crate) struct AcceptedGameplayMachine {
    machine: ArcadeMachine,
}

impl AcceptedGameplayMachine {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AcceptedFrame {
    pub(crate) snapshot: AcceptedSnapshot,
    pub(crate) events: Vec<AcceptedEvent>,
    pub(crate) sound_commands: Vec<u8>,
    pub(crate) visual_hash: Option<u32>,
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
            visual_hash: output.video_crc32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedSnapshot {
    pub(crate) frame: u64,
    pub(crate) phase: AcceptedPhase,
    pub(crate) credits: u8,
    pub(crate) current_player: u8,
    pub(crate) wave: u8,
    pub(crate) player: AcceptedPlayer,
    pub(crate) scores: AcceptedScores,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedPlayer {
    pub(crate) x_subpixels: i32,
    pub(crate) y_subpixels: i32,
    pub(crate) x_velocity_subpixels: i32,
    pub(crate) y_velocity_subpixels: i32,
    pub(crate) direction: AcceptedDirection,
    pub(crate) lives: u8,
    pub(crate) smart_bombs: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedScores {
    pub(crate) player_one: u32,
    pub(crate) player_two: u32,
    pub(crate) high_score: u32,
    pub(crate) next_bonus: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcceptedPhase {
    Attract,
    Playing,
    GameOver,
    HighScoreEntry,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcceptedDirection {
    Left,
    Right,
}

impl From<Facing> for AcceptedDirection {
    fn from(direction: Facing) -> Self {
        match direction {
            Facing::Left => Self::Left,
            Facing::Right => Self::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcceptedEvent {
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

pub(crate) fn run_runtime() -> anyhow::Result<()> {
    app::run()
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

#[cfg(test)]
pub(crate) fn cabinet_input_for_test(input: GameInput) -> CabinetInput {
    to_cabinet_input(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        accepted::{
            AcceptedDirection, AcceptedEvent, AcceptedFrame, AcceptedGameplayMachine,
            AcceptedPhase, AcceptedSnapshot, to_cabinet_input,
        },
        compatibility::{
            input::CabinetInput,
            machine_state::{self, MachineEvent},
            red_label::Facing,
        },
        game::GameInput,
    };

    #[test]
    fn accepted_machine_starts_from_attract_snapshot() {
        let machine = AcceptedGameplayMachine::new();
        let snapshot = machine.snapshot();

        assert_eq!(snapshot.frame, 0);
        assert_eq!(snapshot.phase, AcceptedPhase::Attract);
        assert_eq!(snapshot.current_player, 1);
    }

    #[test]
    fn accepted_machine_steps_clean_input_contract() {
        let mut machine = AcceptedGameplayMachine::new();

        let frame = machine.step(GameInput {
            coin: true,
            start_one: true,
            fire: true,
            service_auto_up: true,
            ..GameInput::NONE
        });

        assert_eq!(frame.snapshot.frame, 1);
        assert_eq!(frame.snapshot.phase, AcceptedPhase::Attract);
        assert!(frame.visual_hash.is_some());
    }

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
    fn accepted_frame_owns_snapshot_sounds_and_visual_hash() {
        let mut machine = crate::compatibility::machine::ArcadeMachine::new();
        let output = machine.step(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });

        let frame = AcceptedFrame::from(output);

        assert_eq!(frame.snapshot.frame, 1);
        assert!(frame.sound_commands.is_empty());
        assert!(frame.visual_hash.is_some());
    }

    #[test]
    fn accepted_snapshot_carries_clean_direction_and_score_fields() {
        let mut snapshot = crate::compatibility::machine::ArcadeMachine::new().snapshot();
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
    fn accepted_facade_exposes_native_visible_size() {
        assert_eq!(crate::accepted::native_visible_size(), (292, 240));
    }
}
