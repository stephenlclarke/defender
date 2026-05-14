//! Neutral facade over the temporary accepted-behavior implementation.
//!
//! The clean rewrite should depend on the contracts in this module, not on
//! legacy module names. This keeps the current machine available as an oracle
//! while making the remaining retirement work explicit and localized.

#[cfg(test)]
use crate::game::GameInput;

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct AcceptedGameplayMachine {
    machine: crate::accepted_behavior::AcceptedMachineAdapter,
}

#[cfg(test)]
impl AcceptedGameplayMachine {
    pub(crate) fn new() -> Self {
        Self {
            machine: crate::accepted_behavior::AcceptedMachineAdapter::new(),
        }
    }

    pub(crate) fn snapshot(&self) -> AcceptedSnapshot {
        self.machine.snapshot()
    }

    pub(crate) fn step(&mut self, input: GameInput) -> AcceptedFrame {
        self.machine.step(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AcceptedFrame {
    pub(crate) snapshot: AcceptedSnapshot,
    pub(crate) events: Vec<AcceptedEvent>,
    pub(crate) sound_commands: Vec<u8>,
    pub(crate) visual_signature: Option<u32>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcceptedDirection {
    Left,
    Right,
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

pub(crate) fn native_visible_size() -> (u16, u16) {
    crate::accepted_behavior::native_visible_size()
}

pub(crate) fn run_runtime() -> anyhow::Result<()> {
    crate::accepted_behavior::run_runtime()
}

#[cfg(test)]
mod tests {
    use crate::{
        accepted::{AcceptedGameplayMachine, AcceptedPhase},
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
        assert!(frame.visual_signature.is_some());
    }

    #[test]
    fn accepted_facade_exposes_native_visible_size() {
        assert_eq!(crate::accepted::native_visible_size(), (292, 240));
    }
}
