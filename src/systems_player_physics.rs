#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmartBombFrame {
    pub destroyed_enemies: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SmartBombSystem;

impl SmartBombSystem {
    pub const fn detonate(active_enemies: usize) -> SmartBombFrame {
        SmartBombFrame {
            destroyed_enemies: active_enemies,
        }
    }
}

const PLAYER_MIN_SCREEN_Y: u8 = 42;
const PLAYER_DOWN_LIMIT_SCREEN_Y: u8 = 238;
const PLAYER_RIGHT_ANCHOR_X: u8 = 0x20;
const PLAYER_LEFT_ANCHOR_X: u8 = 0x70;
const PLAYER_ACCELERATION: i16 = 0x0300;
const HORIZONTAL_VELOCITY_LIMIT: u16 = 0x0100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fixed24 {
    value: i32,
}

impl Fixed24 {
    const MASK: i32 = 0x00FF_FFFF;
    const SIGN: i32 = 0x0080_0000;

    fn from_world_vector(vector: WorldVector) -> Self {
        Self::new(vector.subpixels() >> 8)
    }

    fn new(value: i32) -> Self {
        let raw = value & Self::MASK;
        if raw & Self::SIGN == 0 {
            Self { value: raw }
        } else {
            Self {
                value: raw | !Self::MASK,
            }
        }
    }

    fn from_bytes(bytes: [u8; 3]) -> Self {
        let raw = i32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
        Self::new(raw)
    }

    fn to_bytes(self) -> [u8; 3] {
        let raw = (self.value & Self::MASK) as u32;
        [
            ((raw >> 16) & 0xFF) as u8,
            ((raw >> 8) & 0xFF) as u8,
            (raw & 0xFF) as u8,
        ]
    }

    fn to_world_vector(self) -> WorldVector {
        WorldVector::from_subpixels(self.value << 8)
    }

    fn damped(self) -> Self {
        let [high, middle, low] = self.to_bytes();
        let negated_high_word = (!u16::from_be_bytes([high, middle])).wrapping_add(1);
        let sign_extension: u8 = if negated_high_word & 0x8000 == 0 {
            0x00
        } else {
            0xFF
        };
        let shifted = negated_high_word.wrapping_shl(2);
        let (middle_low, carry) = u16::from_be_bytes([middle, low]).overflowing_add(shifted);
        let next_high = sign_extension
            .wrapping_add(high)
            .wrapping_add(u8::from(carry));
        let [next_middle, next_low] = middle_low.to_be_bytes();
        Self::from_bytes([next_high, next_middle, next_low])
    }

    fn add_signed_word(self, delta: i16) -> Self {
        Self::new(self.value.wrapping_add(i32::from(delta)))
    }

    fn high_word(self) -> u16 {
        let [high, middle, _] = self.to_bytes();
        u16::from_be_bytes([high, middle])
    }

    fn with_high_word(self, high_word: u16) -> Self {
        let [_, _, low] = self.to_bytes();
        let [high, middle] = high_word.to_be_bytes();
        Self::from_bytes([high, middle, low])
    }

    fn calculated_screen_x(self, direction: Direction) -> u16 {
        let [mut high, mut middle, _] = self.to_bytes();
        for _ in 0..2 {
            let carry = high & 1;
            high = (high >> 1) | (high & 0x80);
            middle = (middle >> 1) | (carry << 7);
        }

        let carry = middle & 1;
        middle = (middle >> 1) | (middle & 0x80);
        let mut offset_high = middle;
        let mut offset_low = carry << 7;
        let anchor = match direction {
            Direction::Left => PLAYER_LEFT_ANCHOR_X,
            Direction::Right => PLAYER_RIGHT_ANCHOR_X,
        };
        let moving_with_direction = match direction {
            Direction::Left => offset_high & 0x80 != 0,
            Direction::Right => offset_high & 0x80 == 0,
        };
        if !moving_with_direction {
            offset_high = 0;
            offset_low = 0;
        }

        u16::from_be_bytes([anchor.wrapping_add(offset_high), offset_low])
    }
}

fn thrust_acceleration(direction: Direction) -> i16 {
    match direction {
        Direction::Left => -PLAYER_ACCELERATION,
        Direction::Right => PLAYER_ACCELERATION,
    }
}

fn unsigned_vector_word(vector: WorldVector) -> u16 {
    (vector.subpixels() >> 8) as u16
}

fn signed_vector_word(vector: WorldVector) -> u16 {
    (vector.subpixels() >> 8) as i16 as u16
}

fn unsigned_word_vector(word: u16) -> WorldVector {
    WorldVector::from_subpixels(i32::from(word) << 8)
}

fn signed_word_vector(word: u16) -> WorldVector {
    WorldVector::from_subpixels(i32::from(word as i16) << 8)
}

fn scroll_adjusted_x(previous_x: u16, calculated_x: u16) -> (u16, u16) {
    let delta = calculated_x.wrapping_sub(previous_x);
    if delta == 0 {
        return (calculated_x, 0);
    }

    if calculated_x >= previous_x {
        if delta <= 0x0100 {
            (calculated_x, 0)
        } else {
            (previous_x.wrapping_add(0x0100), 0x0040)
        }
    } else if signed_word_greater_than(delta, 0xFF00) {
        (calculated_x, 0)
    } else {
        (previous_x.wrapping_sub(0x0100), 0xFFC0)
    }
}

fn clamp_camera_velocity_word(value: u16) -> u16 {
    if signed_word_greater_or_equal(value, HORIZONTAL_VELOCITY_LIMIT) {
        HORIZONTAL_VELOCITY_LIMIT
    } else if signed_word_less_or_equal(value, (!HORIZONTAL_VELOCITY_LIMIT).wrapping_add(1)) {
        (!HORIZONTAL_VELOCITY_LIMIT).wrapping_add(1)
    } else {
        value
    }
}

fn player_world_x(screen_x: u16, camera_left: u16) -> u16 {
    let mut shifted = screen_x >> 2;
    shifted &= 0xFFE0;
    shifted.wrapping_add(camera_left)
}

fn next_vertical_velocity(
    screen_y: u8,
    current_velocity: u16,
    control: VerticalControl,
) -> Option<u16> {
    match control {
        VerticalControl::Neutral => Some(0),
        VerticalControl::Up => {
            if screen_y <= PLAYER_MIN_SCREEN_Y + 1 {
                return None;
            }
            if current_velocity & 0x8000 == 0 {
                Some(0xFF00)
            } else {
                let candidate = current_velocity.wrapping_sub(8);
                if signed_word_greater_or_equal(candidate, 0xFE00) {
                    Some(candidate)
                } else {
                    Some(0xFE00)
                }
            }
        }
        VerticalControl::Down => {
            if screen_y >= PLAYER_DOWN_LIMIT_SCREEN_Y {
                return None;
            }
            if signed_word_less_or_equal(current_velocity, 0) {
                Some(0x0100)
            } else {
                let candidate = current_velocity.wrapping_add(8);
                if candidate <= 0x0200 {
                    Some(candidate)
                } else {
                    Some(0x0200)
                }
            }
        }
    }
}

fn signed_word_greater_than(left: u16, right: u16) -> bool {
    (left as i16) > (right as i16)
}

fn signed_word_greater_or_equal(left: u16, right: u16) -> bool {
    (left as i16) >= (right as i16)
}

fn signed_word_less_or_equal(left: u16, right: u16) -> bool {
    (left as i16) <= (right as i16)
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
