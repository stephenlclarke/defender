//! Deterministic fixed-step system utilities.

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
    use super::{FixedStepAccumulator, FrameRate};

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
}
