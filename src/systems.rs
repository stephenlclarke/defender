//! Deterministic fixed-step system utilities.

use crate::game::{Direction, GameFrame, GameInput, GameState, WorldVector};

pub trait GameSimulation {
    fn state(&self) -> GameState;

    fn step(&mut self, input: GameInput) -> GameFrame;
}

pub fn advance_one_frame(simulation: &mut impl GameSimulation, input: GameInput) -> GameFrame {
    simulation.step(input)
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenPosition {
    pub x: u8,
    pub y: u8,
}

impl ScreenPosition {
    pub const fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    pub const fn from_packed(value: u16) -> Self {
        let [x, y] = value.to_be_bytes();
        Self { x, y }
    }

    pub const fn packed(self) -> u16 {
        u16::from_be_bytes([self.x, self.y])
    }

    pub const fn wrapping_offset(self, x: u8, y: u8) -> Self {
        Self {
            x: self.x.wrapping_add(x),
            y: self.y.wrapping_add(y),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScreenVelocity {
    pub dx: i8,
    pub dy: i8,
}

impl ScreenVelocity {
    pub const fn new(dx: i8, dy: i8) -> Self {
        Self { dx, dy }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyMotionFrame {
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnemyMotionSystem;

impl EnemyMotionSystem {
    pub fn step(position: ScreenPosition, velocity: ScreenVelocity) -> EnemyMotionFrame {
        EnemyMotionFrame {
            position: ScreenPosition::new(
                position.x.wrapping_add_signed(velocity.dx),
                position.y.wrapping_add_signed(velocity.dy),
            ),
            velocity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerMotionState {
    pub position: (WorldVector, WorldVector),
    pub velocity: (WorldVector, WorldVector),
    pub direction: Direction,
    pub camera_left: WorldVector,
}

impl PlayerMotionState {
    pub const fn new(
        position: (WorldVector, WorldVector),
        velocity: (WorldVector, WorldVector),
        direction: Direction,
        camera_left: WorldVector,
    ) -> Self {
        Self {
            position,
            velocity,
            direction,
            camera_left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerMotionFrame {
    pub state: PlayerMotionState,
    pub camera_delta: WorldVector,
    pub world_x: WorldVector,
    pub screen_position: ScreenPosition,
    pub blocked_by_vertical_limit: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerMotionSystem;

impl PlayerMotionSystem {
    pub fn step(state: PlayerMotionState, intent: PlayerControlIntent) -> PlayerMotionFrame {
        let mut x_velocity = Fixed24::from_world_vector(state.velocity.0).damped();
        if intent.thrust {
            x_velocity = x_velocity.add_signed_word(thrust_acceleration(state.direction));
        }

        let calculated_x = x_velocity.calculated_screen_x(state.direction);
        let previous_x = unsigned_vector_word(state.position.0);
        let (screen_x, camera_delta) = scroll_adjusted_x(previous_x, calculated_x);

        x_velocity = x_velocity.with_high_word(clamp_camera_velocity_word(x_velocity.high_word()));
        let camera_left = unsigned_vector_word(state.camera_left)
            .wrapping_add(x_velocity.high_word())
            .wrapping_sub(camera_delta);
        let world_x = player_world_x(screen_x, camera_left);

        let previous_y = unsigned_vector_word(state.position.1);
        let previous_y_velocity = signed_vector_word(state.velocity.1);
        let vertical = next_vertical_velocity(
            previous_y.to_be_bytes()[0],
            previous_y_velocity,
            intent.vertical,
        );
        let (screen_y, y_velocity, blocked_by_vertical_limit) = match vertical {
            Some(y_velocity) => (previous_y.wrapping_add(y_velocity), y_velocity, false),
            None => (previous_y, previous_y_velocity, true),
        };

        let next_state = PlayerMotionState {
            position: (
                unsigned_word_vector(screen_x),
                unsigned_word_vector(screen_y),
            ),
            velocity: (x_velocity.to_world_vector(), signed_word_vector(y_velocity)),
            direction: state.direction,
            camera_left: unsigned_word_vector(camera_left),
        };

        PlayerMotionFrame {
            state: next_state,
            camera_delta: signed_word_vector(camera_delta),
            world_x: unsigned_word_vector(world_x),
            screen_position: ScreenPosition::from_packed(u16::from_be_bytes([
                screen_x.to_be_bytes()[0],
                screen_y.to_be_bytes()[0],
            ])),
            blocked_by_vertical_limit,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjectileState {
    pub active_projectiles: u8,
}

impl ProjectileState {
    pub const fn new(active_projectiles: u8) -> Self {
        Self { active_projectiles }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileLaunchOutcome {
    Started {
        state: ProjectileState,
        direction: Direction,
        spawn: ScreenPosition,
    },
    CapacityReached {
        state: ProjectileState,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjectileSystem;

impl ProjectileSystem {
    pub const MAX_ACTIVE_PROJECTILES: u8 = 4;

    pub fn try_launch(
        state: ProjectileState,
        player_position: ScreenPosition,
        direction: Direction,
    ) -> ProjectileLaunchOutcome {
        if state.active_projectiles >= Self::MAX_ACTIVE_PROJECTILES {
            return ProjectileLaunchOutcome::CapacityReached { state };
        }

        let spawn = match direction {
            Direction::Left => player_position.wrapping_offset(0, 4),
            Direction::Right => player_position.wrapping_offset(7, 4),
        };
        ProjectileLaunchOutcome::Started {
            state: ProjectileState::new(state.active_projectiles.wrapping_add(1)),
            direction,
            spawn,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectileMotionFrame {
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjectileMotionSystem;

impl ProjectileMotionSystem {
    pub const fn velocity_for_direction(direction: Direction) -> ScreenVelocity {
        match direction {
            Direction::Left => ScreenVelocity::new(-8, 0),
            Direction::Right => ScreenVelocity::new(8, 0),
        }
    }

    pub fn step(position: ScreenPosition, velocity: ScreenVelocity) -> ProjectileMotionFrame {
        let next_x = i16::from(position.x) + i16::from(velocity.dx);
        let next_y = i16::from(position.y) + i16::from(velocity.dy);
        let active = (0..=i16::from(u8::MAX)).contains(&next_x)
            && (0..=i16::from(u8::MAX)).contains(&next_y);

        ProjectileMotionFrame {
            position: if active {
                ScreenPosition::new(next_x as u8, next_y as u8)
            } else {
                position
            },
            velocity,
            active,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionBox {
    pub position: ScreenPosition,
    pub size: (u8, u8),
}

impl CollisionBox {
    pub const fn new(position: ScreenPosition, size: (u8, u8)) -> Self {
        Self { position, size }
    }

    pub fn overlaps(self, other: Self) -> bool {
        let left = i16::from(self.position.x);
        let right = left + i16::from(self.size.0);
        let top = i16::from(self.position.y);
        let bottom = top + i16::from(self.size.1);
        let other_left = i16::from(other.position.x);
        let other_right = other_left + i16::from(other.size.0);
        let other_top = i16::from(other.position.y);
        let other_bottom = other_top + i16::from(other.size.1);

        left < other_right && right > other_left && top < other_bottom && bottom > other_top
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectileEnemyHit {
    pub projectile_index: usize,
    pub enemy_index: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CollisionSystem;

impl CollisionSystem {
    pub fn first_projectile_enemy_hit(
        projectiles: &[CollisionBox],
        enemies: &[CollisionBox],
    ) -> Option<ProjectileEnemyHit> {
        for (projectile_index, projectile) in projectiles.iter().copied().enumerate() {
            for (enemy_index, enemy) in enemies.iter().copied().enumerate() {
                if projectile.overlaps(enemy) {
                    return Some(ProjectileEnemyHit {
                        projectile_index,
                        enemy_index,
                    });
                }
            }
        }
        None
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

#[cfg(test)]
mod tests {
    use crate::{
        game::{
            Direction, GameEvents, GameFrame, GameInput, GamePhase, GameState, PlayerSnapshot,
            ScoreSnapshot, WorldSnapshot, WorldVector,
        },
        renderer::{RenderScene, SurfaceSize},
    };

    use super::{
        CollisionBox, CollisionSystem, EnemyMotionSystem, Fixed24, FixedStepAccumulator, FrameRate,
        GameSimulation, PlayerActionTriggers, PlayerControlIntent, PlayerControlSystem,
        PlayerMotionState, PlayerMotionSystem, ProjectileLaunchOutcome, ProjectileMotionSystem,
        ProjectileState, ProjectileSystem, ScreenPosition, ScreenVelocity, VerticalControl,
        advance_one_frame, clamp_camera_velocity_word, next_vertical_velocity, scroll_adjusted_x,
        thrust_acceleration,
    };

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
    fn simulation_trait_advances_clean_frames_without_legacy_state_contracts() {
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

    #[test]
    fn player_control_intent_keeps_held_controls_separate_from_edges() {
        let intent = PlayerControlIntent::from_input(GameInput {
            altitude_up: true,
            altitude_down: true,
            reverse: true,
            thrust: true,
            fire: true,
            smart_bomb: true,
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(intent.vertical, VerticalControl::Up);
        assert!(intent.reverse);
        assert!(intent.thrust);
        assert!(intent.fire);
        assert!(intent.smart_bomb);
        assert!(intent.hyperspace);
    }

    #[test]
    fn player_control_system_requires_two_clear_samples_for_new_triggers() {
        let mut controls = PlayerControlSystem::new();
        let fire = GameInput {
            fire: true,
            ..GameInput::NONE
        };

        assert!(controls.step(fire).triggers.fire);
        assert_eq!(controls.step(fire).triggers, PlayerActionTriggers::NONE);
        assert_eq!(
            controls.step(GameInput::NONE).triggers,
            PlayerActionTriggers::NONE
        );
        assert_eq!(controls.step(fire).triggers, PlayerActionTriggers::NONE);
        assert_eq!(
            controls.step(GameInput::NONE).triggers,
            PlayerActionTriggers::NONE
        );
        assert_eq!(
            controls.step(GameInput::NONE).triggers,
            PlayerActionTriggers::NONE
        );
        assert!(controls.step(fire).triggers.fire);
    }

    #[test]
    fn player_control_system_reports_all_playing_control_triggers() {
        let mut controls = PlayerControlSystem::new();
        let frame = controls.step(GameInput {
            altitude_down: true,
            reverse: true,
            thrust: true,
            fire: true,
            smart_bomb: true,
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(
            frame.triggers,
            PlayerActionTriggers {
                fire: true,
                thrust: true,
                smart_bomb: true,
                hyperspace: true,
                reverse: true,
                altitude_down: true,
            }
        );
        assert!(frame.triggers.any());
    }

    #[test]
    fn player_motion_applies_thrust_damping_scroll_and_world_position() {
        let frame = PlayerMotionSystem::step(
            player_motion_state(0x2000, 0x8000, 0, 0, Direction::Right, 0),
            PlayerControlIntent {
                thrust: true,
                ..PlayerControlIntent::default()
            },
        );

        assert_eq!(word(frame.state.position.0), 0x2000);
        assert_eq!(word(frame.state.velocity.0), 0x0300);
        assert_eq!(word(frame.state.camera_left), 0x0003);
        assert_eq!(word(frame.world_x), 0x0803);
        assert_eq!(frame.screen_position, ScreenPosition::new(0x20, 0x80));
        assert!(!frame.blocked_by_vertical_limit);
    }

    #[test]
    fn player_motion_applies_vertical_priority_acceleration_and_limits() {
        let upward = PlayerMotionSystem::step(
            player_motion_state(0x2000, 0x8000, 0, 0, Direction::Right, 0),
            PlayerControlIntent {
                vertical: VerticalControl::Up,
                ..PlayerControlIntent::default()
            },
        );

        assert_eq!(word(upward.state.velocity.1), 0xFF00);
        assert_eq!(word(upward.state.position.1), 0x7F00);
        assert_eq!(upward.screen_position, ScreenPosition::new(0x20, 0x7F));

        let blocked = PlayerMotionSystem::step(
            player_motion_state(0x2000, 0xEE00, 0, 0, Direction::Right, 0),
            PlayerControlIntent {
                vertical: VerticalControl::Down,
                ..PlayerControlIntent::default()
            },
        );

        assert!(blocked.blocked_by_vertical_limit);
        assert_eq!(word(blocked.state.position.1), 0xEE00);
        assert_eq!(word(blocked.state.velocity.1), 0);
    }

    #[test]
    fn player_motion_helpers_cover_direction_scroll_and_velocity_limits() {
        assert_eq!(Fixed24::new(0x0080_0000).to_bytes(), [0x80, 0x00, 0x00]);
        assert_eq!(
            Fixed24::from_bytes([0x01, 0x00, 0x00]).damped().to_bytes(),
            [0x00, 0xFC, 0x00]
        );
        assert_eq!(
            Fixed24::new(-0x0300).calculated_screen_x(Direction::Left),
            0x6F80
        );
        assert_eq!(
            Fixed24::new(0x0300).calculated_screen_x(Direction::Left),
            0x7000
        );
        assert_eq!(thrust_acceleration(Direction::Left), -0x0300);

        assert_eq!(scroll_adjusted_x(0x2000, 0x2080), (0x2080, 0));
        assert_eq!(scroll_adjusted_x(0x2000, 0x2200), (0x2100, 0x0040));
        assert_eq!(scroll_adjusted_x(0x2000, 0x1F80), (0x1F80, 0));
        assert_eq!(scroll_adjusted_x(0x2000, 0x1E00), (0x1F00, 0xFFC0));

        assert_eq!(clamp_camera_velocity_word(0x0200), 0x0100);
        assert_eq!(clamp_camera_velocity_word(0xFE00), 0xFF00);
        assert_eq!(clamp_camera_velocity_word(0x0080), 0x0080);

        assert_eq!(next_vertical_velocity(43, 0, VerticalControl::Up), None);
        assert_eq!(
            next_vertical_velocity(0x80, 0xFF00, VerticalControl::Up),
            Some(0xFEF8)
        );
        assert_eq!(
            next_vertical_velocity(0x80, 0xFE00, VerticalControl::Up),
            Some(0xFE00)
        );
        assert_eq!(
            next_vertical_velocity(0x80, 0, VerticalControl::Down),
            Some(0x0100)
        );
        assert_eq!(
            next_vertical_velocity(0x80, 0x0100, VerticalControl::Down),
            Some(0x0108)
        );
        assert_eq!(
            next_vertical_velocity(0x80, 0x0200, VerticalControl::Down),
            Some(0x0200)
        );
    }

    #[test]
    fn projectile_launch_uses_player_edge_and_caps_active_projectiles() {
        let started = ProjectileSystem::try_launch(
            ProjectileState::new(0),
            ScreenPosition::new(0x40, 0x50),
            Direction::Right,
        );

        assert_eq!(
            started,
            ProjectileLaunchOutcome::Started {
                state: ProjectileState::new(1),
                direction: Direction::Right,
                spawn: ScreenPosition::new(0x47, 0x54),
            }
        );
        assert_eq!(
            ProjectileSystem::try_launch(
                ProjectileState::new(3),
                ScreenPosition::new(0x40, 0x50),
                Direction::Left,
            ),
            ProjectileLaunchOutcome::Started {
                state: ProjectileState::new(4),
                direction: Direction::Left,
                spawn: ScreenPosition::new(0x40, 0x54),
            }
        );
        assert_eq!(
            ProjectileSystem::try_launch(
                ProjectileState::new(ProjectileSystem::MAX_ACTIVE_PROJECTILES),
                ScreenPosition::new(0x40, 0x50),
                Direction::Right,
            ),
            ProjectileLaunchOutcome::CapacityReached {
                state: ProjectileState::new(4),
            }
        );
    }

    #[test]
    fn projectile_motion_system_advances_directional_velocity_and_culls_screen_exit() {
        assert_eq!(
            ProjectileMotionSystem::velocity_for_direction(Direction::Right),
            ScreenVelocity::new(8, 0)
        );
        assert_eq!(
            ProjectileMotionSystem::velocity_for_direction(Direction::Left),
            ScreenVelocity::new(-8, 0)
        );

        let moved = ProjectileMotionSystem::step(
            ScreenPosition::new(0x40, 0x50),
            ScreenVelocity::new(8, 0),
        );

        assert_eq!(moved.position, ScreenPosition::new(0x48, 0x50));
        assert_eq!(moved.velocity, ScreenVelocity::new(8, 0));
        assert!(moved.active);

        let off_right =
            ProjectileMotionSystem::step(ScreenPosition::new(252, 0x50), ScreenVelocity::new(8, 0));
        let off_left =
            ProjectileMotionSystem::step(ScreenPosition::new(3, 0x50), ScreenVelocity::new(-8, 0));

        assert!(!off_right.active);
        assert!(!off_left.active);
    }

    #[test]
    fn enemy_motion_system_advances_and_wraps_screen_positions() {
        let moved =
            EnemyMotionSystem::step(ScreenPosition::new(204, 84), ScreenVelocity::new(-1, 2));

        assert_eq!(moved.position, ScreenPosition::new(203, 86));
        assert_eq!(moved.velocity, ScreenVelocity::new(-1, 2));

        let wrapped =
            EnemyMotionSystem::step(ScreenPosition::new(0, 255), ScreenVelocity::new(-2, 1));

        assert_eq!(wrapped.position, ScreenPosition::new(254, 0));
    }

    #[test]
    fn collision_boxes_detect_overlap_without_touching_edges() {
        let projectile = CollisionBox::new(ScreenPosition::new(40, 50), (8, 2));

        assert!(projectile.overlaps(CollisionBox::new(ScreenPosition::new(47, 51), (12, 8))));
        assert!(!projectile.overlaps(CollisionBox::new(ScreenPosition::new(48, 51), (12, 8))));
        assert!(!projectile.overlaps(CollisionBox::new(ScreenPosition::new(47, 52), (12, 8))));
    }

    #[test]
    fn collision_system_reports_first_projectile_enemy_hit() {
        let projectiles = [
            CollisionBox::new(ScreenPosition::new(10, 10), (8, 2)),
            CollisionBox::new(ScreenPosition::new(40, 50), (8, 2)),
        ];
        let enemies = [
            CollisionBox::new(ScreenPosition::new(80, 80), (12, 8)),
            CollisionBox::new(ScreenPosition::new(47, 51), (12, 8)),
        ];

        assert_eq!(
            CollisionSystem::first_projectile_enemy_hit(&projectiles, &enemies),
            Some(super::ProjectileEnemyHit {
                projectile_index: 1,
                enemy_index: 1,
            })
        );
        assert_eq!(
            CollisionSystem::first_projectile_enemy_hit(&projectiles[..1], &enemies[..1]),
            None
        );
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
                    world: WorldSnapshot::default(),
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

    fn player_motion_state(
        x: u16,
        y: u16,
        x_velocity: i32,
        y_velocity: i16,
        direction: Direction,
        camera_left: u16,
    ) -> PlayerMotionState {
        PlayerMotionState::new(
            (unsigned_vector(x), unsigned_vector(y)),
            (
                WorldVector::from_subpixels(x_velocity << 8),
                WorldVector::from_subpixels(i32::from(y_velocity) << 8),
            ),
            direction,
            unsigned_vector(camera_left),
        )
    }

    fn unsigned_vector(word: u16) -> WorldVector {
        WorldVector::from_subpixels(i32::from(word) << 8)
    }

    fn word(vector: WorldVector) -> u16 {
        (vector.subpixels() >> 8) as u16
    }
}
