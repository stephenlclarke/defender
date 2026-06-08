use crate::game::{Direction, GameInput, ScoreSnapshot, WorldVector};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VerticalControl {
    #[default]
    Neutral,
    Up,
    Down,
}

impl VerticalControl {
    pub const fn from_input(input: GameInput) -> Self {
        if input.altitude_up {
            Self::Up
        } else if input.altitude_down {
            Self::Down
        } else {
            Self::Neutral
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerControlIntent {
    pub vertical: VerticalControl,
    pub thrust: bool,
    pub reverse: bool,
    pub fire: bool,
    pub smart_bomb: bool,
    pub hyperspace: bool,
}

impl PlayerControlIntent {
    pub const fn from_input(input: GameInput) -> Self {
        Self {
            vertical: VerticalControl::from_input(input),
            thrust: input.thrust,
            reverse: input.reverse,
            fire: input.fire,
            smart_bomb: input.smart_bomb,
            hyperspace: input.hyperspace,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerActionTriggers {
    pub fire: bool,
    pub thrust: bool,
    pub smart_bomb: bool,
    pub hyperspace: bool,
    pub reverse: bool,
    pub altitude_down: bool,
}

impl PlayerActionTriggers {
    pub const NONE: Self = Self {
        fire: false,
        thrust: false,
        smart_bomb: false,
        hyperspace: false,
        reverse: false,
        altitude_down: false,
    };

    pub const fn any(self) -> bool {
        self.fire
            || self.thrust
            || self.smart_bomb
            || self.hyperspace
            || self.reverse
            || self.altitude_down
    }

    const fn from_input(input: GameInput) -> Self {
        Self {
            fire: input.fire,
            thrust: input.thrust,
            smart_bomb: input.smart_bomb,
            hyperspace: input.hyperspace,
            reverse: input.reverse,
            altitude_down: input.altitude_down,
        }
    }

    const fn newly_pressed_after_clear_samples(self, previous: Self, older: Self) -> Self {
        Self {
            fire: self.fire && !previous.fire && !older.fire,
            thrust: self.thrust && !previous.thrust && !older.thrust,
            smart_bomb: self.smart_bomb && !previous.smart_bomb && !older.smart_bomb,
            hyperspace: self.hyperspace && !previous.hyperspace && !older.hyperspace,
            reverse: self.reverse && !previous.reverse && !older.reverse,
            altitude_down: self.altitude_down && !previous.altitude_down && !older.altitude_down,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerControlFrame {
    pub intent: PlayerControlIntent,
    pub triggers: PlayerActionTriggers,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerControlSystem {
    previous: PlayerActionTriggers,
    older: PlayerActionTriggers,
}

impl PlayerControlSystem {
    pub const fn new() -> Self {
        Self {
            previous: PlayerActionTriggers::NONE,
            older: PlayerActionTriggers::NONE,
        }
    }

    pub fn step(&mut self, input: GameInput) -> PlayerControlFrame {
        let current = PlayerActionTriggers::from_input(input);
        let frame = PlayerControlFrame {
            intent: PlayerControlIntent::from_input(input),
            triggers: current.newly_pressed_after_clear_samples(self.previous, self.older),
        };
        self.older = self.previous;
        self.previous = current;
        frame
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OperatorActionTriggers {
    pub diagnostics: bool,
    pub audits: bool,
    pub high_score_reset: bool,
}

impl OperatorActionTriggers {
    pub const NONE: Self = Self {
        diagnostics: false,
        audits: false,
        high_score_reset: false,
    };

    pub const fn any(self) -> bool {
        self.diagnostics || self.audits || self.high_score_reset
    }

    const fn from_input(input: GameInput) -> Self {
        Self {
            diagnostics: input.service_auto_up,
            audits: input.service_advance,
            high_score_reset: input.high_score_reset,
        }
    }

    const fn newly_pressed(self, previous: Self) -> Self {
        Self {
            diagnostics: self.diagnostics && !previous.diagnostics,
            audits: self.audits && !previous.audits,
            high_score_reset: self.high_score_reset && !previous.high_score_reset,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OperatorControlFrame {
    pub triggers: OperatorActionTriggers,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OperatorControlSystem {
    previous: OperatorActionTriggers,
}

impl OperatorControlSystem {
    pub const fn new() -> Self {
        Self {
            previous: OperatorActionTriggers::NONE,
        }
    }

    pub fn step(&mut self, input: GameInput) -> OperatorControlFrame {
        let current = OperatorActionTriggers::from_input(input);
        let frame = OperatorControlFrame {
            triggers: current.newly_pressed(self.previous),
        };
        self.previous = current;
        frame
    }
}
