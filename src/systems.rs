//! Deterministic fixed-step system utilities.

use crate::game::{GameFrame, GameInput, GameState};

pub trait GameSimulation {
    fn state(&self) -> GameState;

    fn step(&mut self, input: GameInput) -> GameFrame;
}

pub fn advance_one_frame(simulation: &mut impl GameSimulation, input: GameInput) -> GameFrame {
    simulation.step(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRate {
    millihz: u32,
}

impl FrameRate {
    pub const CABINET: Self = Self { millihz: 60_100 };

    pub const fn from_millihz(millihz: u32) -> Self {
        Self { millihz }
    }

    pub const fn millihz(self) -> u32 {
        self.millihz
    }

    pub const fn frame_duration_micros(self) -> u64 {
        let rate = self.millihz as u64;
        (1_000_000_000 + (rate / 2)) / rate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedStepAccumulator {
    frame_rate: FrameRate,
    accumulated_micros: u64,
}

impl FixedStepAccumulator {
    pub const fn new(frame_rate: FrameRate) -> Self {
        Self {
            frame_rate,
            accumulated_micros: 0,
        }
    }

    pub fn add_elapsed_micros(&mut self, elapsed_micros: u64) {
        self.accumulated_micros = self.accumulated_micros.saturating_add(elapsed_micros);
    }

    pub fn consume_due_steps(&mut self, max_steps: u32) -> u32 {
        let frame_duration = self.frame_rate.frame_duration_micros();
        let due = (self.accumulated_micros / frame_duration).min(u64::from(max_steps)) as u32;
        self.accumulated_micros -= u64::from(due) * frame_duration;
        due
    }

    pub const fn accumulated_micros(self) -> u64 {
        self.accumulated_micros
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        game::{
            Direction, GameEvents, GameFrame, GameInput, GamePhase, GameState, PlayerSnapshot,
            ScoreSnapshot, WorldVector,
        },
        renderer::{RenderScene, SurfaceSize},
    };

    use super::{FixedStepAccumulator, FrameRate, GameSimulation, advance_one_frame};

    #[test]
    fn frame_rate_uses_rounded_microsecond_duration() {
        assert_eq!(FrameRate::CABINET.millihz(), 60_100);
        assert_eq!(FrameRate::CABINET.frame_duration_micros(), 16_639);
    }

    #[test]
    fn fixed_step_accumulator_consumes_bounded_steps() {
        let mut accumulator = FixedStepAccumulator::new(FrameRate::from_millihz(1_000));
        accumulator.add_elapsed_micros(3_500_000);

        assert_eq!(accumulator.consume_due_steps(2), 2);
        assert_eq!(accumulator.accumulated_micros(), 1_500_000);
        assert_eq!(accumulator.consume_due_steps(8), 1);
        assert_eq!(accumulator.accumulated_micros(), 500_000);
    }

    #[test]
    fn simulation_trait_advances_clean_frames_without_memory_contracts() {
        let mut simulation = FakeSimulation::default();

        let frame = advance_one_frame(
            &mut simulation,
            GameInput {
                coin: true,
                ..GameInput::NONE
            },
        );

        assert_eq!(frame.state.frame, 1);
        assert_eq!(frame.state.credits, 1);
        assert_eq!(frame.scene.summary().frame, 1);
    }

    #[derive(Debug)]
    struct FakeSimulation {
        state: GameState,
    }

    impl Default for FakeSimulation {
        fn default() -> Self {
            Self {
                state: GameState {
                    frame: 0,
                    phase: GamePhase::Attract,
                    credits: 0,
                    current_player: 1,
                    wave: 0,
                    player: PlayerSnapshot {
                        position: (WorldVector::default(), WorldVector::default()),
                        velocity: (WorldVector::default(), WorldVector::default()),
                        direction: Direction::Right,
                        lives: 3,
                        smart_bombs: 3,
                    },
                    scores: ScoreSnapshot {
                        player_one: 0,
                        player_two: 0,
                        high_score: 100,
                        next_bonus: 10_000,
                    },
                },
            }
        }
    }

    impl GameSimulation for FakeSimulation {
        fn state(&self) -> GameState {
            self.state.clone()
        }

        fn step(&mut self, input: GameInput) -> GameFrame {
            self.state.frame += 1;
            if input.coin {
                self.state.credits += 1;
            }
            GameFrame {
                state: self.state.clone(),
                events: GameEvents::default(),
                scene: RenderScene::empty(self.state.frame, SurfaceSize::new(292, 240)),
            }
        }
    }
}
