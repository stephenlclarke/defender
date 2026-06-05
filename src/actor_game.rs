//! Actor-oriented Defender rewrite prototype.
//!
//! This module is intentionally independent from the current MAME-shaped
//! `Game` implementation. It models the game as driver-owned actor threads:
//! the driver prompts every asset once per step, gathers commands, resolves
//! world rules in a stable order, and publishes a step description.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

const PLAYER_SPEED: i16 = 2;
const INITIAL_SMART_BOMBS: u8 = 3;
const PLAYER_LASER_COOLDOWN_STEPS: u8 = 8;
const PLAYER_HYPERSPACE_HIDDEN_STEPS: u8 = 33;
const PLAYER_HYPERSPACE_REMATERIALIZE_X: i16 = 128;
const PLAYER_HYPERSPACE_REMATERIALIZE_Y: i16 = 120;
const PLAYER_HYPERSPACE_DEATH_DELAY_STEPS: u8 = 39;
const PLAYER_HYPERSPACE_DEATH_LSEED: u8 = 0x0C;
const SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD: u8 = 0xC0;
const SOURCE_PLAYFIELD_Y_MIN: u8 = 42;
const PLAYER_BOUNDS: Rect = Rect::new(0, 18, 255, 220);
const LASER_SPEED: i16 = 8;
const LASER_LIFETIME: u16 = 34;
const LANDER_FIRE_PERIOD: u64 = 96;
const LANDER_SHOT_SPEED: i16 = 3;
const LANDER_SHOT_LIFETIME: u16 = 90;
const EXPLOSION_LIFETIME: u16 = 20;
const SCORE_POPUP_LIFETIME: u16 = 50;
const WILLIAMS_REVEAL_STEPS: u16 = 36;
const WILLIAMS_COLOR_PERIOD: u16 = 8;
const DEFENDER_WORDMARK_START_STEP: u64 = 72;
const DEFENDER_WORDMARK_SLOTS: u16 = 15;
const DEFENDER_WORDMARK_ROW_PAIRS: u16 = 6;
const HUMAN_GROUND_Y: i16 = 214;
const HUMAN_FALL_ACCELERATION: i16 = 1;
const HUMAN_MAX_FALL_SPEED: i16 = 8;
const HUMAN_SAFE_LANDING_SPEED: i16 = 3;
const HUMAN_CARRIED_OFFSET_Y: i16 = 8;
const SOURCE_HUMAN_WALK_SLEEP_TICKS: u8 = 2;
const SOURCE_HUMAN_LEFT_X_VELOCITY: u16 = 0xFFE0;
const SOURCE_HUMAN_RIGHT_X_VELOCITY: u16 = 0x0020;
const SOURCE_INITIAL_POD_X_SPEED: u8 = 0x20;
const SOURCE_POD_SWARMER_REQUEST_LIMIT: usize = 6;
const SOURCE_ACTIVE_SWARMER_LIMIT: usize = 20;
const SOURCE_ACTIVE_BAITER_LIMIT: usize = 12;
const SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS: u8 = 3;
const SOURCE_BAITER_INITIAL_SHOT_TIMER: u8 = 8;
const SOURCE_BAITER_LOOP_SLEEP_TICKS: u8 = 6;
const SOURCE_BAITER_X_SEEK_SPEED: u8 = 0x40;
const SOURCE_BAITER_Y_SEEK_BYTE: u8 = 0x01;
const SOURCE_BAITER_X_SEEK_WINDOW_HALF_PIXELS: i16 = 20;
const SOURCE_BAITER_Y_SEEK_WINDOW_HALF_PIXELS: i16 = 10;
const SOURCE_BAITER_PICTURE_FRAME_COUNT: u8 = 3;
const ACTOR_BAITER_TIMER_PACING_STEPS: u8 = 15;
const SOURCE_BOMBER_LOOP_SLEEP_TICKS: u8 = 1;
const SOURCE_BOMBER_PICTURE_FRAME_COUNT: u8 = 4;
const SOURCE_BOMBER_CRUISE_ALTITUDE: i16 = 0x50;
const SOURCE_ACTIVE_BOMBER_BOMB_LIMIT: usize = 10;
const SOURCE_MAX_ACTIVE_WAVE_ENEMIES: usize = 5;
const STATUS_SCORE_POSITION: Point = Point::new(8, 6);
const STATUS_HIGH_SCORE_POSITION: Point = Point::new(94, 6);
const STATUS_WAVE_POSITION: Point = Point::new(8, 18);
const STATUS_LIVES_POSITION: Point = Point::new(86, 18);
const STATUS_SMART_BOMBS_POSITION: Point = Point::new(140, 18);
const STATUS_CREDITS_POSITION: Point = Point::new(176, 226);
const STATUS_FINAL_SCORE_POSITION: Point = Point::new(56, 92);
const STATUS_HIGH_SCORE_TABLE_TITLE_POSITION: Point = Point::new(78, 112);
const STATUS_HIGH_SCORE_TABLE_START_Y: i16 = 128;
const STATUS_HIGH_SCORE_TABLE_ROW_HEIGHT: i16 = 12;
const LANDER_SEEK_SPEED: i16 = 1;
const LANDER_DRIFT_SPEED: i16 = 1;
const LANDER_CARRY_SPEED: i16 = 2;
const LANDER_PICKUP_RADIUS_X: i16 = 6;
const LANDER_PICKUP_RADIUS_Y: i16 = 8;
const LANDER_CONVERSION_Y: i16 = 24;
const MUTANT_SEEK_SPEED: i16 = 1;
const BOMBER_DRIFT_SPEED: i16 = 1;
const BOMBER_BOMB_PERIOD: u64 = 64;
const POD_DRIFT_SPEED: i16 = 1;
const SWARMER_SEEK_SPEED: i16 = 2;
const SWARMER_FIRE_PERIOD: u64 = 58;
const SWARMER_SHOT_SPEED: i16 = 3;
const BAITER_SEEK_SPEED: i16 = 3;
const BAITER_FIRE_PERIOD: u64 = 42;
const BAITER_SHOT_SPEED: i16 = 4;
const BOMB_LIFETIME: u16 = 96;
const LANDER_SCORE: u32 = 150;
const MUTANT_SCORE: u32 = 150;
const BOMBER_SCORE: u32 = 250;
const POD_SCORE: u32 = 1000;
const SWARMER_SCORE: u32 = 150;
const BAITER_SCORE: u32 = 200;
const HUMAN_RESCUE_SCORE: u32 = 500;
const HUMAN_SAFE_LANDING_SCORE: u32 = 250;
const ACTOR_SOURCE_WAVE_TABLE_TSV: &str = include_str!("../assets/red-label/wave-table.tsv");
const ACTOR_SOURCE_WAVE_TABLE_HEADER: &str =
    "key\tceiling\tfloor\tintra_delta\tinter_delta\twave1\twave2\twave3\twave4";
const ACTOR_SOURCE_DEFAULT_DIFFICULTY_INITIAL: u8 = 5;
const ACTOR_SOURCE_DEFAULT_DIFFICULTY_CEILING: u8 = 15;
const ACTOR_SOURCE_BACKED_WAVES: u16 = 16;
const ACTOR_SOURCE_FIRST_WAVE_HUMAN_SPAWNS: [ActorHumanSpawn; 10] = [
    ActorHumanSpawn::source_first_wave(
        0,
        ActorSourceFirstWaveHumanStart {
            x16: 0x18C3,
            y16: 0xE000,
            picture_frame: 2,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        1,
        ActorSourceFirstWaveHumanStart {
            x16: 0x1C81,
            y16: 0xE100,
            picture_frame: 3,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        2,
        ActorSourceFirstWaveHumanStart {
            x16: 0x4E30,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        3,
        ActorSourceFirstWaveHumanStart {
            x16: 0x5718,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        4,
        ActorSourceFirstWaveHumanStart {
            x16: 0x9B8C,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        5,
        ActorSourceFirstWaveHumanStart {
            x16: 0x9DC6,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        6,
        ActorSourceFirstWaveHumanStart {
            x16: 0xCEE3,
            y16: 0xE000,
            picture_frame: 2,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        7,
        ActorSourceFirstWaveHumanStart {
            x16: 0xD771,
            y16: 0xE000,
            picture_frame: 2,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        8,
        ActorSourceFirstWaveHumanStart {
            x16: 0xD2B8,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        9,
        ActorSourceFirstWaveHumanStart {
            x16: 0xE8DC,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
];
const ACTOR_SOURCE_FIRST_WAVE_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0xFB33,
        y16: 0x2CE0,
        x_velocity: 0xFFDE,
        y_velocity: 0x0070,
        shot_timer: 0x27,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(1),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x3F4A,
        y16: 0x2CE0,
        x_velocity: 0xFFEE,
        y_velocity: 0x0070,
        shot_timer: 0x3B,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(2),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x67FF,
        y16: 0x2C70,
        x_velocity: 0x0012,
        y_velocity: 0x0070,
        shot_timer: 0x23,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(3),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x0D11,
        y16: 0x2C70,
        x_velocity: 0x0014,
        y_velocity: 0x0070,
        shot_timer: 0x3C,
        sleep_ticks: 0x04,
        picture_frame: 0,
        target_human_index: Some(4),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x41B9,
        y16: 0x2C70,
        x_velocity: 0x001A,
        y_velocity: 0x0070,
        shot_timer: 0x25,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(5),
    }),
];
const ACTOR_WAVE_ACTIVE_SPAWN_SLOTS: [Point; SOURCE_MAX_ACTIVE_WAVE_ENEMIES] = [
    Point::new(0xE4, 0x2A),
    Point::new(228, 104),
    Point::new(184, 72),
    Point::new(148, 96),
    Point::new(236, 66),
];
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActorId(u64);

impl ActorId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct XyzzyMode {
    pub active: bool,
    pub auto_fire: bool,
    pub invincible: bool,
    pub overlay_smart_bomb: bool,
}

impl XyzzyMode {
    pub const INACTIVE: Self = Self {
        active: false,
        auto_fire: false,
        invincible: false,
        overlay_smart_bomb: false,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameInput {
    pub coin: bool,
    pub coin_two: bool,
    pub coin_three: bool,
    pub start_one: bool,
    pub start_two: bool,
    pub altitude_up: bool,
    pub altitude_down: bool,
    pub thrust: bool,
    pub reverse: bool,
    pub fire: bool,
    pub smart_bomb: bool,
    pub hyperspace: bool,
    pub service_advance: bool,
    pub high_score_reset: bool,
    pub auto_up_manual_down: bool,
    pub tilt: bool,
    pub xyzzy: XyzzyMode,
}

impl GameInput {
    pub const NONE: Self = Self {
        coin: false,
        coin_two: false,
        coin_three: false,
        start_one: false,
        start_two: false,
        altitude_up: false,
        altitude_down: false,
        thrust: false,
        reverse: false,
        fire: false,
        smart_bomb: false,
        hyperspace: false,
        service_advance: false,
        high_score_reset: false,
        auto_up_manual_down: false,
        tilt: false,
        xyzzy: XyzzyMode::INACTIVE,
    };

    fn coin_insertions(self) -> u8 {
        u8::from(self.coin) + u8::from(self.coin_two) + u8::from(self.coin_three)
    }

    fn wants_fire(self) -> bool {
        self.fire || self.xyzzy.auto_fire
    }

    fn wants_stock_smart_bomb(self) -> bool {
        self.smart_bomb && !self.xyzzy.overlay_smart_bomb
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KeyboardProfile {
    #[default]
    Planetoid,
    Cabinet,
}

impl KeyboardProfile {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "planetoid" => Some(Self::Planetoid),
            "cabinet" => Some(Self::Cabinet),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Planetoid => "planetoid",
            Self::Cabinet => "cabinet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardKey {
    Character(char),
    Enter,
    Backspace,
    Escape,
    Tab,
    ArrowUp,
    ArrowDown,
    Function(u8),
    LeftShift,
    RightShift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyTransition {
    Press,
    Repeat,
    Release,
}

impl KeyTransition {
    const fn contributes_input(self) -> bool {
        matches!(self, Self::Press | Self::Repeat)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardEvent {
    pub key: KeyboardKey,
    pub transition: KeyTransition,
}

impl KeyboardEvent {
    pub const fn press(key: KeyboardKey) -> Self {
        Self {
            key,
            transition: KeyTransition::Press,
        }
    }

    pub const fn release(key: KeyboardKey) -> Self {
        Self {
            key,
            transition: KeyTransition::Release,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyboardPoll {
    pub input: GameInput,
    pub typed_chars: Vec<char>,
    pub quit_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct HeldControls {
    altitude_up: bool,
    altitude_down: bool,
    thrust: bool,
    auto_up_manual_down: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardMapper {
    profile: KeyboardProfile,
    held: HeldControls,
    xyzzy: XyzzyController,
}

impl KeyboardMapper {
    pub fn new(profile: KeyboardProfile) -> Self {
        Self {
            profile,
            held: HeldControls::default(),
            xyzzy: XyzzyController::default(),
        }
    }

    pub fn profile(&self) -> KeyboardProfile {
        self.profile
    }

    pub fn xyzzy_mode(&self) -> XyzzyMode {
        self.xyzzy.mode(false)
    }

    pub fn map_event(&mut self, event: KeyboardEvent, step: &mut KeyboardPoll) {
        let active = event.transition.contributes_input();
        if event.transition == KeyTransition::Press {
            match event.key {
                KeyboardKey::Character(character) => {
                    let typed = character.to_ascii_lowercase();
                    step.typed_chars.push(typed);
                    self.xyzzy.ingest(typed);
                }
                KeyboardKey::Backspace => step.typed_chars.push('\u{8}'),
                _ => {}
            }
        }

        match event.key {
            KeyboardKey::Escape if active => step.quit_requested = true,
            KeyboardKey::Character('q' | 'Q') if active => step.quit_requested = true,
            _ => self.map_profile_event(event, step),
        }

        if self.xyzzy.active() && active {
            match event.key {
                KeyboardKey::Character('f' | 'F') => self.xyzzy.toggle_auto_fire(),
                KeyboardKey::Character('g' | 'G') => self.xyzzy.toggle_invincible(),
                KeyboardKey::Tab => step.input.xyzzy.overlay_smart_bomb = true,
                _ => {}
            }
        }
    }

    pub fn finish_poll(&self, step: &mut KeyboardPoll) {
        step.input.altitude_up |= self.held.altitude_up;
        step.input.altitude_down |= self.held.altitude_down;
        step.input.thrust |= self.held.thrust;
        step.input.auto_up_manual_down |= self.held.auto_up_manual_down;
        let overlay_smart_bomb = step.input.xyzzy.overlay_smart_bomb;
        step.input.xyzzy = self.xyzzy.mode(overlay_smart_bomb);
        if step.input.xyzzy.auto_fire {
            step.input.fire = true;
        }
    }

    fn map_profile_event(&mut self, event: KeyboardEvent, step: &mut KeyboardPoll) {
        match self.profile {
            KeyboardProfile::Planetoid => self.map_planetoid_event(event, step),
            KeyboardProfile::Cabinet => self.map_cabinet_event(event, step),
        }
    }

    fn map_planetoid_event(&mut self, event: KeyboardEvent, step: &mut KeyboardPoll) {
        let active = event.transition.contributes_input();
        match event.key {
            KeyboardKey::Enter if active => {
                step.input.start_one = true;
                step.input.fire = true;
            }
            KeyboardKey::Character('1') if active => step.input.start_one = true,
            KeyboardKey::Character('5') if active => step.input.coin = true,
            KeyboardKey::Character('6') if active => step.input.coin_two = true,
            KeyboardKey::Character('7') if active => step.input.coin_three = true,
            KeyboardKey::Character('a' | 'A') => set_held_control(
                &mut self.held.altitude_up,
                event.transition,
                &mut step.input.altitude_up,
            ),
            KeyboardKey::Character('z' | 'Z') => set_held_control(
                &mut self.held.altitude_down,
                event.transition,
                &mut step.input.altitude_down,
            ),
            KeyboardKey::LeftShift | KeyboardKey::RightShift => set_held_control(
                &mut self.held.thrust,
                event.transition,
                &mut step.input.thrust,
            ),
            KeyboardKey::Character(' ') if active => step.input.reverse = true,
            KeyboardKey::Tab if active => step.input.smart_bomb = true,
            KeyboardKey::Character('h' | 'H') if active => step.input.hyperspace = true,
            KeyboardKey::Function(2) if active => step.input.service_advance = true,
            KeyboardKey::Function(3) if active => step.input.high_score_reset = true,
            KeyboardKey::Function(4) => set_held_control(
                &mut self.held.auto_up_manual_down,
                event.transition,
                &mut step.input.auto_up_manual_down,
            ),
            KeyboardKey::Function(5) if active => step.input.tilt = true,
            _ => {}
        }
    }

    fn map_cabinet_event(&mut self, event: KeyboardEvent, step: &mut KeyboardPoll) {
        let active = event.transition.contributes_input();
        match event.key {
            KeyboardKey::Character('5') if active => step.input.coin = true,
            KeyboardKey::Character('6') if active => step.input.coin_two = true,
            KeyboardKey::Character('7') if active => step.input.coin_three = true,
            KeyboardKey::Character('1') if active => step.input.start_one = true,
            KeyboardKey::Character('2') if active => step.input.start_two = true,
            KeyboardKey::ArrowUp => set_held_control(
                &mut self.held.altitude_up,
                event.transition,
                &mut step.input.altitude_up,
            ),
            KeyboardKey::ArrowDown => set_held_control(
                &mut self.held.altitude_down,
                event.transition,
                &mut step.input.altitude_down,
            ),
            KeyboardKey::Character('r' | 'R') if active => step.input.reverse = true,
            KeyboardKey::Character('t' | 'T') => set_held_control(
                &mut self.held.thrust,
                event.transition,
                &mut step.input.thrust,
            ),
            KeyboardKey::Character('f' | 'F') if active => step.input.fire = true,
            KeyboardKey::Character('b' | 'B') if active => step.input.smart_bomb = true,
            KeyboardKey::Character('h' | 'H') if active => step.input.hyperspace = true,
            KeyboardKey::Function(2) if active => step.input.service_advance = true,
            KeyboardKey::Function(3) if active => step.input.high_score_reset = true,
            KeyboardKey::Function(4) => set_held_control(
                &mut self.held.auto_up_manual_down,
                event.transition,
                &mut step.input.auto_up_manual_down,
            ),
            KeyboardKey::Function(5) if active => step.input.tilt = true,
            _ => {}
        }
    }
}

impl Default for KeyboardMapper {
    fn default() -> Self {
        Self::new(KeyboardProfile::default())
    }
}

fn set_held_control(held: &mut bool, transition: KeyTransition, output: &mut bool) {
    match transition {
        KeyTransition::Press | KeyTransition::Repeat => {
            *held = true;
            *output = true;
        }
        KeyTransition::Release => *held = false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XyzzyController {
    active: bool,
    sequence_index: usize,
    auto_fire: bool,
    invincible: bool,
}

impl XyzzyController {
    const CODE: [char; 5] = ['x', 'y', 'z', 'z', 'y'];

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn auto_fire(&self) -> bool {
        self.auto_fire
    }

    pub fn invincible(&self) -> bool {
        self.invincible
    }

    pub fn ingest(&mut self, character: char) {
        let character = character.to_ascii_lowercase();
        if character == Self::CODE[self.sequence_index] {
            self.sequence_index += 1;
            if self.sequence_index == Self::CODE.len() {
                self.active = !self.active;
                self.sequence_index = 0;
                if !self.active {
                    self.auto_fire = false;
                    self.invincible = false;
                }
            }
        } else {
            self.sequence_index = usize::from(character == Self::CODE[0]);
        }
    }

    fn toggle_auto_fire(&mut self) {
        if self.active {
            self.auto_fire = !self.auto_fire;
        }
    }

    fn toggle_invincible(&mut self) {
        if self.active {
            self.invincible = !self.invincible;
        }
    }

    fn mode(&self, overlay_smart_bomb: bool) -> XyzzyMode {
        XyzzyMode {
            active: self.active,
            auto_fire: self.active && self.auto_fire,
            invincible: self.active && self.invincible,
            overlay_smart_bomb: self.active && overlay_smart_bomb,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub const fn offset(self, velocity: Velocity) -> Self {
        Self {
            x: self.x + velocity.dx,
            y: self.y + velocity.dy,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Velocity {
    pub dx: i16,
    pub dy: i16,
}

impl Velocity {
    pub const fn new(dx: i16, dy: i16) -> Self {
        Self { dx, dy }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

impl Rect {
    pub const fn new(left: i16, top: i16, right: i16, bottom: i16) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn from_center(center: Point, width: i16, height: i16) -> Self {
        let half_width = width / 2;
        let half_height = height / 2;
        Self {
            left: center.x - half_width,
            top: center.y - half_height,
            right: center.x + half_width,
            bottom: center.y + half_height,
        }
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.left <= other.right
            && self.right >= other.left
            && self.top <= other.bottom
            && self.bottom >= other.top
    }

    pub const fn clamp_point(self, point: Point) -> Point {
        Point {
            x: clamp_i16(point.x, self.left, self.right),
            y: clamp_i16(point.y, self.top, self.bottom),
        }
    }
}

const fn clamp_i16(value: i16, min: i16, max: i16) -> i16 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    const fn sign(self) -> i16 {
        match self {
            Self::Left => -1,
            Self::Right => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Attract,
    Playing,
    GameOver,
    HighScoreEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActorKind {
    AttractDirector,
    AttractScript,
    StatusDisplay,
    WilliamsLogo,
    DefenderWordmark,
    Player,
    Lander,
    Mutant,
    Bomber,
    Bomb,
    Pod,
    Swarmer,
    Baiter,
    Human,
    Laser,
    EnemyLaser,
    Explosion,
    ScorePopup,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanderBehaviorMode {
    SeekNearestHuman,
    ChasePlayer,
    Drift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostileMovementMode {
    Drift,
    ChasePlayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorHyperspaceSourceSeed {
    pub seed: u8,
    pub hseed: u8,
    pub lseed: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBehaviorProfile {
    pub player_speed: i16,
    pub player_laser_cooldown_steps: u8,
    pub player_hyperspace_hidden_steps: u8,
    pub player_hyperspace_rematerialize_x: i16,
    pub player_hyperspace_rematerialize_y: i16,
    pub player_hyperspace_source_seed: Option<ActorHyperspaceSourceSeed>,
    pub player_hyperspace_death_delay_steps: u8,
    pub player_hyperspace_death_lseed: u8,
    pub player_takes_enemy_collision_damage: bool,
    pub laser_speed: i16,
    pub laser_lifetime_steps: u16,
    pub lander_seek_speed: i16,
    pub lander_drift_speed: i16,
    pub lander_carry_speed: i16,
    pub lander_pickup_radius_x: i16,
    pub lander_pickup_radius_y: i16,
    pub lander_conversion_y: i16,
    pub lander_fire_period_steps: u64,
    pub lander_shot_speed: i16,
    pub lander_shot_lifetime_steps: u16,
    pub lander_mode: LanderBehaviorMode,
    pub mutant_seek_speed: i16,
    pub mutant_mode: HostileMovementMode,
    pub bomber_drift_speed: i16,
    pub bomber_bomb_period_steps: u64,
    pub bomber_mode: HostileMovementMode,
    pub pod_drift_speed: i16,
    pub pod_mode: HostileMovementMode,
    pub swarmer_seek_speed: i16,
    pub swarmer_fire_period_steps: u64,
    pub swarmer_shot_speed: i16,
    pub swarmer_mode: HostileMovementMode,
    pub baiter_seek_speed: i16,
    pub baiter_fire_period_steps: u64,
    pub baiter_shot_speed: i16,
    pub baiter_mode: HostileMovementMode,
    pub bomb_lifetime_steps: u16,
    pub human_ground_y: i16,
    pub human_fall_acceleration: i16,
    pub human_max_fall_speed: i16,
    pub human_safe_landing_speed: i16,
    pub human_carried_offset_y: i16,
    pub explosion_lifetime_steps: u16,
    pub score_popup_lifetime_steps: u16,
}

impl ActorBehaviorProfile {
    pub const DEFAULT: Self = Self {
        player_speed: PLAYER_SPEED,
        player_laser_cooldown_steps: PLAYER_LASER_COOLDOWN_STEPS,
        player_hyperspace_hidden_steps: PLAYER_HYPERSPACE_HIDDEN_STEPS,
        player_hyperspace_rematerialize_x: PLAYER_HYPERSPACE_REMATERIALIZE_X,
        player_hyperspace_rematerialize_y: PLAYER_HYPERSPACE_REMATERIALIZE_Y,
        player_hyperspace_source_seed: None,
        player_hyperspace_death_delay_steps: PLAYER_HYPERSPACE_DEATH_DELAY_STEPS,
        player_hyperspace_death_lseed: PLAYER_HYPERSPACE_DEATH_LSEED,
        player_takes_enemy_collision_damage: true,
        laser_speed: LASER_SPEED,
        laser_lifetime_steps: LASER_LIFETIME,
        lander_seek_speed: LANDER_SEEK_SPEED,
        lander_drift_speed: LANDER_DRIFT_SPEED,
        lander_carry_speed: LANDER_CARRY_SPEED,
        lander_pickup_radius_x: LANDER_PICKUP_RADIUS_X,
        lander_pickup_radius_y: LANDER_PICKUP_RADIUS_Y,
        lander_conversion_y: LANDER_CONVERSION_Y,
        lander_fire_period_steps: LANDER_FIRE_PERIOD,
        lander_shot_speed: LANDER_SHOT_SPEED,
        lander_shot_lifetime_steps: LANDER_SHOT_LIFETIME,
        lander_mode: LanderBehaviorMode::SeekNearestHuman,
        mutant_seek_speed: MUTANT_SEEK_SPEED,
        mutant_mode: HostileMovementMode::ChasePlayer,
        bomber_drift_speed: BOMBER_DRIFT_SPEED,
        bomber_bomb_period_steps: BOMBER_BOMB_PERIOD,
        bomber_mode: HostileMovementMode::Drift,
        pod_drift_speed: POD_DRIFT_SPEED,
        pod_mode: HostileMovementMode::Drift,
        swarmer_seek_speed: SWARMER_SEEK_SPEED,
        swarmer_fire_period_steps: SWARMER_FIRE_PERIOD,
        swarmer_shot_speed: SWARMER_SHOT_SPEED,
        swarmer_mode: HostileMovementMode::ChasePlayer,
        baiter_seek_speed: BAITER_SEEK_SPEED,
        baiter_fire_period_steps: BAITER_FIRE_PERIOD,
        baiter_shot_speed: BAITER_SHOT_SPEED,
        baiter_mode: HostileMovementMode::ChasePlayer,
        bomb_lifetime_steps: BOMB_LIFETIME,
        human_ground_y: HUMAN_GROUND_Y,
        human_fall_acceleration: HUMAN_FALL_ACCELERATION,
        human_max_fall_speed: HUMAN_MAX_FALL_SPEED,
        human_safe_landing_speed: HUMAN_SAFE_LANDING_SPEED,
        human_carried_offset_y: HUMAN_CARRIED_OFFSET_Y,
        explosion_lifetime_steps: EXPLOSION_LIFETIME,
        score_popup_lifetime_steps: SCORE_POPUP_LIFETIME,
    };

    pub const fn arcade_default() -> Self {
        Self::DEFAULT
    }
}

impl Default for ActorBehaviorProfile {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorBehaviorScript {
    default_profile: ActorBehaviorProfile,
    kind_profiles: BTreeMap<ActorKind, ActorBehaviorProfile>,
    actor_profiles: BTreeMap<ActorId, ActorBehaviorProfile>,
}

impl ActorBehaviorScript {
    pub fn new(default_profile: ActorBehaviorProfile) -> Self {
        Self {
            default_profile,
            kind_profiles: BTreeMap::new(),
            actor_profiles: BTreeMap::new(),
        }
    }

    pub fn default_profile(&self) -> ActorBehaviorProfile {
        self.default_profile
    }

    pub fn set_default_profile(&mut self, profile: ActorBehaviorProfile) {
        self.default_profile = profile;
    }

    pub fn set_kind_behavior(&mut self, kind: ActorKind, profile: ActorBehaviorProfile) {
        self.kind_profiles.insert(kind, profile);
    }

    pub fn set_actor_behavior(&mut self, actor: ActorId, profile: ActorBehaviorProfile) {
        self.actor_profiles.insert(actor, profile);
    }

    pub fn remove_actor_behavior(&mut self, actor: ActorId) {
        self.actor_profiles.remove(&actor);
    }

    pub fn with_kind_behavior(mut self, kind: ActorKind, profile: ActorBehaviorProfile) -> Self {
        self.set_kind_behavior(kind, profile);
        self
    }

    pub fn with_actor_behavior(mut self, actor: ActorId, profile: ActorBehaviorProfile) -> Self {
        self.set_actor_behavior(actor, profile);
        self
    }

    pub fn behavior_for(&self, actor: ActorId, kind: ActorKind) -> ActorBehaviorProfile {
        self.actor_profiles
            .get(&actor)
            .copied()
            .or_else(|| self.kind_profiles.get(&kind).copied())
            .unwrap_or(self.default_profile)
    }

    fn with_input_overrides(
        &self,
        input: GameInput,
        snapshots: impl Iterator<Item = ActorSnapshot>,
    ) -> Self {
        let mut script = self.clone();
        if input.xyzzy.invincible {
            let mut player_kind_behavior = script.behavior_for(ActorId::new(0), ActorKind::Player);
            player_kind_behavior.player_takes_enemy_collision_damage = false;
            script.set_kind_behavior(ActorKind::Player, player_kind_behavior);

            let actor_ids = script.actor_profiles.keys().copied().collect::<Vec<_>>();
            for actor in actor_ids {
                let mut behavior = script.behavior_for(actor, ActorKind::Player);
                behavior.player_takes_enemy_collision_damage = false;
                script.set_actor_behavior(actor, behavior);
            }

            for snapshot in
                snapshots.filter(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            {
                let mut behavior = script.behavior_for(snapshot.id, ActorKind::Player);
                behavior.player_takes_enemy_collision_damage = false;
                script.set_actor_behavior(snapshot.id, behavior);
            }
        }
        script
    }
}

impl Default for ActorBehaviorScript {
    fn default() -> Self {
        Self::new(ActorBehaviorProfile::DEFAULT)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceWaveProfile {
    pub landers: u8,
    pub bombers: u8,
    pub pods: u8,
    pub wave_size: u8,
    pub lander_x_velocity: u8,
    pub bomber_x_velocity: u8,
    pub swarmer_x_velocity: u8,
    pub swarmer_shot_time: u32,
    pub swarmer_acceleration_mask: u8,
    pub baiter_delay: u32,
    pub baiter_shot_time: u32,
    pub baiter_seek_probability: u8,
    pub lander_shot_time: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceLanderMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub picture_frame: u8,
    pub target_human_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceBomberMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub picture_frame: u8,
    pub cruise_altitude: i16,
    pub sleep_ticks: u8,
    pub source_slot: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourcePodMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceSwarmerMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub acceleration: u8,
    pub sleep_ticks: u8,
    pub shot_timer: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceBaiterMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub picture_frame: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorLanderSpawn {
    pub position: Point,
    pub source: Option<ActorSourceLanderMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBomberSpawn {
    pub position: Point,
    pub source: Option<ActorSourceBomberMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorPodSpawn {
    pub position: Point,
    pub source: Option<ActorSourcePodMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSwarmerSpawn {
    pub position: Point,
    pub source: Option<ActorSourceSwarmerMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBaiterSpawn {
    pub position: Point,
    pub source: Option<ActorSourceBaiterMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceHumanMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub picture_frame: u8,
    pub target_slot_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorHumanSpawn {
    pub position: Point,
    pub mode: HumanMode,
    pub source: Option<ActorSourceHumanMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceFirstWaveLanderStart {
    x16: u16,
    y16: u16,
    x_velocity: u16,
    y_velocity: u16,
    shot_timer: u8,
    sleep_ticks: u8,
    picture_frame: u8,
    target_human_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceFirstWaveHumanStart {
    x16: u16,
    y16: u16,
    picture_frame: u8,
}

impl ActorLanderSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            source: None,
        }
    }

    const fn source_first_wave(start: ActorSourceFirstWaveLanderStart) -> Self {
        Self {
            position: Point::new((start.x16 >> 8) as i16, (start.y16 >> 8) as i16),
            source: Some(ActorSourceLanderMetadata {
                x_fraction: (start.x16 & 0x00FF) as u8,
                y_fraction: (start.y16 & 0x00FF) as u8,
                x_velocity: start.x_velocity,
                y_velocity: start.y_velocity,
                shot_timer: start.shot_timer,
                sleep_ticks: start.sleep_ticks,
                picture_frame: start.picture_frame,
                target_human_index: start.target_human_index,
            }),
        }
    }
}

impl ActorBomberSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            source: None,
        }
    }

    const fn source_initial(position: Point, source_x_velocity: u8, spawn_index: usize) -> Self {
        let velocity_low = if spawn_index < 2 {
            0u8.wrapping_sub(source_x_velocity)
        } else {
            source_x_velocity
        };
        Self {
            position,
            source: Some(ActorSourceBomberMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: actor_sign_extend_u8_to_u16(velocity_low),
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                source_slot: (spawn_index % 4) as u8,
            }),
        }
    }
}

impl ActorPodSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            source: None,
        }
    }

    const fn source_initial(position: Point, spawn_index: usize) -> Self {
        let velocity_low = if spawn_index < 2 {
            0u8.wrapping_sub(SOURCE_INITIAL_POD_X_SPEED)
        } else {
            SOURCE_INITIAL_POD_X_SPEED
        };
        Self {
            position,
            source: Some(ActorSourcePodMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: actor_sign_extend_u8_to_u16(velocity_low),
                y_velocity: 0,
            }),
        }
    }
}

impl ActorSwarmerSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            source: None,
        }
    }

    fn source_from_pod(position: Point, profile: ActorSourceWaveProfile, index: usize) -> Self {
        let x_velocity_low = if index.is_multiple_of(2) {
            profile.swarmer_x_velocity
        } else {
            0u8.wrapping_sub(profile.swarmer_x_velocity)
        };
        let y_velocity_low = swarmer_spawn_y_velocity_low(index);
        Self {
            position,
            source: Some(ActorSourceSwarmerMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: actor_sign_extend_u8_to_u16(x_velocity_low),
                y_velocity: actor_sign_extend_u8_to_u16(y_velocity_low),
                acceleration: ((index as u8).wrapping_mul(7)) & profile.swarmer_acceleration_mask,
                sleep_ticks: 0,
                shot_timer: profile.swarmer_shot_time.min(u32::from(u8::MAX)) as u8,
            }),
        }
    }
}

impl ActorBaiterSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            source: None,
        }
    }

    fn source_from_player(
        profile: ActorSourceWaveProfile,
        player_position: Point,
        active_baiters: usize,
    ) -> Self {
        let spawn_x = if (active_baiters + usize::from(player_position.x >= 128)).is_multiple_of(2)
        {
            228
        } else {
            28
        };
        let spawn_y = (player_position.y + 24
            - (i16::try_from(active_baiters % 3).unwrap_or(0) * 24))
            .clamp(PLAYER_BOUNDS.top + 8, HUMAN_GROUND_Y - 24);
        let position = Point::new(spawn_x, spawn_y);
        let mut source = ActorSourceBaiterMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: SOURCE_BAITER_INITIAL_SHOT_TIMER,
            sleep_ticks: 0,
            picture_frame: 0,
        };
        source_baiter_velocity_update(
            &mut source,
            position,
            profile,
            player_position,
            false,
            u8::MAX,
        );
        Self {
            position,
            source: Some(source),
        }
    }
}

fn swarmer_spawn_y_velocity_low(index: usize) -> u8 {
    match index % SOURCE_POD_SWARMER_REQUEST_LIMIT {
        0 => 0x20,
        1 => 0xE0,
        2 => 0x18,
        3 => 0xE8,
        4 => 0x10,
        _ => 0xF0,
    }
}

impl ActorHumanSpawn {
    pub const fn new(position: Point, mode: HumanMode) -> Self {
        Self {
            position,
            mode,
            source: None,
        }
    }

    const fn source_first_wave(
        target_slot_index: usize,
        start: ActorSourceFirstWaveHumanStart,
    ) -> Self {
        Self {
            position: Point::new((start.x16 >> 8) as i16, (start.y16 >> 8) as i16),
            mode: HumanMode::Grounded,
            source: Some(ActorSourceHumanMetadata {
                x_fraction: (start.x16 & 0x00FF) as u8,
                y_fraction: (start.y16 & 0x00FF) as u8,
                picture_frame: start.picture_frame,
                target_slot_index,
            }),
        }
    }
}

impl ActorSourceWaveProfile {
    pub fn for_wave(wave: u16) -> Self {
        let wave = u8::try_from(wave.min(u16::from(u8::MAX))).unwrap_or(u8::MAX);
        Self {
            landers: actor_source_wave_u8("landers", wave),
            bombers: actor_source_wave_u8("bombers", wave),
            pods: actor_source_wave_u8("pods", wave),
            wave_size: actor_source_wave_u8("wave_size", wave),
            lander_x_velocity: actor_source_wave_u8("lander_x_velocity", wave),
            bomber_x_velocity: actor_source_wave_u8("bomber_x_velocity", wave),
            swarmer_x_velocity: actor_source_wave_u8("swarmer_x_velocity", wave),
            swarmer_shot_time: actor_source_wave_u32("swarmer_shot_time", wave),
            swarmer_acceleration_mask: actor_source_wave_u8("swarmer_acceleration_mask", wave),
            baiter_delay: actor_source_wave_u32("baiter_time", wave),
            baiter_shot_time: actor_source_wave_u32("baiter_shot_time", wave),
            baiter_seek_probability: actor_source_wave_u8("baiter_seek_probability", wave),
            lander_shot_time: actor_source_wave_u32("lander_shot_time", wave),
        }
    }

    fn lander_behavior(self) -> ActorBehaviorProfile {
        ActorBehaviorProfile {
            lander_seek_speed: actor_lander_speed_from_source(self.lander_x_velocity),
            lander_drift_speed: actor_lander_speed_from_source(self.lander_x_velocity),
            lander_fire_period_steps: u64::from(self.lander_shot_time.max(1)),
            ..ActorBehaviorProfile::default()
        }
    }

    fn lander_spawns(self, wave: u16) -> Vec<ActorLanderSpawn> {
        let mut source_lander_index = 0;
        self.active_family_slots()
            .into_iter()
            .filter_map(|slot| {
                if slot.kind != ActorSourceEnemyKind::Lander {
                    return None;
                }
                let spawn = if wave == 1 {
                    ACTOR_SOURCE_FIRST_WAVE_LANDER_SPAWNS
                        .get(source_lander_index)
                        .copied()
                        .unwrap_or_else(|| ActorLanderSpawn::new(slot.position))
                } else {
                    ActorLanderSpawn::new(slot.position)
                };
                source_lander_index += 1;
                Some(spawn)
            })
            .collect()
    }

    fn bomber_spawns(self) -> Vec<ActorBomberSpawn> {
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == ActorSourceEnemyKind::Bomber)
            .map(|slot| {
                ActorBomberSpawn::source_initial(slot.position, self.bomber_x_velocity, slot.index)
            })
            .collect()
    }

    fn pod_spawns(self) -> Vec<ActorPodSpawn> {
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == ActorSourceEnemyKind::Pod)
            .map(|slot| ActorPodSpawn::source_initial(slot.position, slot.index))
            .collect()
    }

    fn active_family_slots(self) -> Vec<ActorSourceEnemySlot> {
        let mut counts = ActorSourceEnemyCounts {
            landers: self.landers,
            bombers: self.bombers,
            pods: self.pods,
        };
        let target = usize::from(self.wave_size)
            .min(SOURCE_MAX_ACTIVE_WAVE_ENEMIES)
            .min(usize::from(counts.total()));
        let mut kinds = Vec::with_capacity(target);

        push_actor_source_kind(
            &mut kinds,
            &mut counts,
            target,
            ActorSourceEnemyKind::Lander,
        );
        push_actor_source_kind(
            &mut kinds,
            &mut counts,
            target,
            ActorSourceEnemyKind::Bomber,
        );
        push_actor_source_kind(&mut kinds, &mut counts, target, ActorSourceEnemyKind::Pod);
        for kind in [
            ActorSourceEnemyKind::Lander,
            ActorSourceEnemyKind::Bomber,
            ActorSourceEnemyKind::Pod,
        ] {
            while kinds.len() < target && counts.take(kind) {
                kinds.push(kind);
            }
        }

        kinds
            .into_iter()
            .enumerate()
            .map(|(index, kind)| ActorSourceEnemySlot {
                kind,
                index,
                position: ACTOR_WAVE_ACTIVE_SPAWN_SLOTS[index],
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceEnemySlot {
    kind: ActorSourceEnemyKind,
    index: usize,
    position: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorSourceEnemyKind {
    Lander,
    Bomber,
    Pod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceEnemyCounts {
    landers: u8,
    bombers: u8,
    pods: u8,
}

impl ActorSourceEnemyCounts {
    const fn total(self) -> u8 {
        self.landers
            .saturating_add(self.bombers)
            .saturating_add(self.pods)
    }

    fn take(&mut self, kind: ActorSourceEnemyKind) -> bool {
        let count = match kind {
            ActorSourceEnemyKind::Lander => &mut self.landers,
            ActorSourceEnemyKind::Bomber => &mut self.bombers,
            ActorSourceEnemyKind::Pod => &mut self.pods,
        };
        if *count == 0 {
            return false;
        }
        *count = count.saturating_sub(1);
        true
    }
}

fn push_actor_source_kind(
    kinds: &mut Vec<ActorSourceEnemyKind>,
    counts: &mut ActorSourceEnemyCounts,
    target: usize,
    kind: ActorSourceEnemyKind,
) {
    if kinds.len() < target && counts.take(kind) {
        kinds.push(kind);
    }
}

fn actor_lander_speed_from_source(velocity: u8) -> i16 {
    i16::from((velocity / 16).max(1))
}

fn actor_velocity_pixels_from_source(velocity: u8) -> i16 {
    i16::from((velocity / 32).max(1))
}

const fn actor_sign_extend_u8_to_u16(value: u8) -> u16 {
    let sign = if value & 0x80 == 0 { 0x00 } else { 0xFF };
    u16::from_be_bytes([sign, value])
}

fn actor_source_wave_u8(key: &str, wave: u8) -> u8 {
    u8::try_from(actor_source_wave_value(key, wave))
        .unwrap_or_else(|_| panic!("actor source wave table {key} should fit u8"))
}

fn actor_source_wave_u32(key: &str, wave: u8) -> u32 {
    u32::try_from(actor_source_wave_value(key, wave))
        .unwrap_or_else(|_| panic!("actor source wave table {key} should be non-negative"))
}

fn actor_source_wave_value(key: &str, wave: u8) -> i32 {
    let mut lines = ACTOR_SOURCE_WAVE_TABLE_TSV.lines();
    let header = lines
        .next()
        .expect("actor source wave table should have a header");
    assert_eq!(header, ACTOR_SOURCE_WAVE_TABLE_HEADER);

    for row in lines.map(str::trim).filter(|row| !row.is_empty()) {
        let fields = row.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 9, "actor source wave table row width changed");
        if fields[0] != key {
            continue;
        }

        let ceiling = parse_actor_wave_i32(fields[1], key, "ceiling");
        let floor = parse_actor_wave_i32(fields[2], key, "floor");
        let inter_delta = parse_actor_wave_i32(fields[4], key, "inter_delta");
        let wave = wave.max(1);
        let wave_index = usize::from(wave.min(4));
        let mut value = parse_actor_wave_i32(fields[4 + wave_index], key, "wave");
        for _ in 0..actor_wave_inter_delta_iterations(wave) {
            value = apply_actor_wave_delta(value, inter_delta, floor, ceiling);
        }
        return value;
    }

    panic!("missing actor source wave table key {key}");
}

fn actor_wave_inter_delta_iterations(wave: u8) -> u16 {
    let wave_delta = wave.saturating_sub(4);
    let pre_ceiling = ACTOR_SOURCE_DEFAULT_DIFFICULTY_INITIAL.saturating_add(wave_delta);
    u16::from(pre_ceiling.min(ACTOR_SOURCE_DEFAULT_DIFFICULTY_CEILING))
}

fn parse_actor_wave_i32(value: &str, key: &str, field: &str) -> i32 {
    value
        .parse()
        .unwrap_or_else(|_| panic!("actor source wave table {key}.{field} is not an integer"))
}

fn apply_actor_wave_delta(value: i32, delta: i32, floor: i32, ceiling: i32) -> i32 {
    if delta > 0 {
        (value + delta).min(ceiling)
    } else if delta < 0 {
        (value + delta).max(floor)
    } else {
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveProfile {
    pub wave: u16,
    pub behavior_script: ActorBehaviorScript,
    pub lander_spawns: Vec<ActorLanderSpawn>,
    pub bomber_spawns: Vec<ActorBomberSpawn>,
    pub pod_spawns: Vec<ActorPodSpawn>,
    pub human_spawns: Vec<ActorHumanSpawn>,
}

impl ActorWaveProfile {
    pub fn new(wave: u16, behavior_script: ActorBehaviorScript, lander_spawns: Vec<Point>) -> Self {
        Self::with_lander_spawns(
            wave,
            behavior_script,
            lander_spawns
                .into_iter()
                .map(ActorLanderSpawn::new)
                .collect(),
        )
    }

    pub fn with_lander_spawns(
        wave: u16,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<ActorLanderSpawn>,
    ) -> Self {
        Self::with_spawns(
            wave,
            behavior_script,
            lander_spawns,
            ACTOR_SOURCE_FIRST_WAVE_HUMAN_SPAWNS.to_vec(),
        )
    }

    pub fn with_spawns(
        wave: u16,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<ActorLanderSpawn>,
        human_spawns: Vec<ActorHumanSpawn>,
    ) -> Self {
        Self::with_family_spawns(
            wave,
            behavior_script,
            lander_spawns,
            Vec::new(),
            Vec::new(),
            human_spawns,
        )
    }

    pub fn with_family_spawns(
        wave: u16,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<ActorLanderSpawn>,
        bomber_spawns: Vec<ActorBomberSpawn>,
        pod_spawns: Vec<ActorPodSpawn>,
        human_spawns: Vec<ActorHumanSpawn>,
    ) -> Self {
        Self {
            wave: wave.max(1),
            behavior_script,
            lander_spawns,
            bomber_spawns,
            pod_spawns,
            human_spawns,
        }
    }

    pub fn lander_spawn_points(&self) -> Vec<Point> {
        self.lander_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn human_spawn_points(&self) -> Vec<Point> {
        self.human_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn bomber_spawn_points(&self) -> Vec<Point> {
        self.bomber_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn pod_spawn_points(&self) -> Vec<Point> {
        self.pod_spawns.iter().map(|spawn| spawn.position).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveScript {
    name: String,
    waves: Vec<ActorWaveProfile>,
}

impl ActorWaveScript {
    pub fn new(name: impl Into<String>, mut waves: Vec<ActorWaveProfile>) -> Self {
        if waves.is_empty() {
            waves.push(Self::source_backed_profile(1));
        }
        waves.sort_by_key(|profile| profile.wave);
        Self {
            name: name.into(),
            waves,
        }
    }

    pub fn single_wave(
        name: impl Into<String>,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<Point>,
    ) -> Self {
        Self::new(
            name,
            vec![ActorWaveProfile::new(1, behavior_script, lander_spawns)],
        )
    }

    pub fn default_progression() -> Self {
        let waves = (1..=ACTOR_SOURCE_BACKED_WAVES)
            .map(Self::source_backed_profile)
            .collect::<Vec<_>>();
        Self::new("actor-source-wave-table", waves)
    }

    fn source_backed_profile(wave: u16) -> ActorWaveProfile {
        let source = ActorSourceWaveProfile::for_wave(wave);
        ActorWaveProfile::with_family_spawns(
            wave,
            ActorBehaviorScript::default()
                .with_kind_behavior(ActorKind::Lander, source.lander_behavior())
                .with_kind_behavior(
                    ActorKind::Bomber,
                    ActorBehaviorProfile {
                        bomber_drift_speed: actor_velocity_pixels_from_source(
                            source.bomber_x_velocity,
                        ),
                        ..ActorBehaviorProfile::default()
                    },
                ),
            source.lander_spawns(wave),
            source.bomber_spawns(),
            source.pod_spawns(),
            ACTOR_SOURCE_FIRST_WAVE_HUMAN_SPAWNS.to_vec(),
        )
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn profile_for_wave(&self, wave: u16) -> &ActorWaveProfile {
        self.waves
            .iter()
            .rev()
            .find(|profile| wave >= profile.wave)
            .unwrap_or(&self.waves[0])
    }
}

impl Default for ActorWaveScript {
    fn default() -> Self {
        Self::default_progression()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteKey {
    WilliamsLogo,
    DefenderCoalescence,
    DefenderWordmark,
    DefenderLogo,
    HighScoreText,
    PlayerRight,
    PlayerLeft,
    Lander,
    Mutant,
    Bomber,
    Bomb,
    Pod,
    Swarmer,
    Baiter,
    Human,
    HumanFalling,
    HumanCarried,
    Laser,
    EnemyLaser,
    Explosion,
    Score250,
    Score500,
    Text,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VisualEffect {
    #[default]
    Static,
    WilliamsReveal {
        stroke_step: u16,
        color_phase: u8,
    },
    DefenderCoalescence {
        slot: u8,
        row_pair: u8,
    },
    SourceLanderFrame {
        frame: u8,
    },
    SourceBomberFrame {
        frame: u8,
    },
    SourcePod,
    SourceBaiterFrame {
        frame: u8,
    },
    SourceHumanFrame {
        frame: u8,
    },
    ExplosionCloud {
        kind: ExplosionKind,
        age: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionKind {
    Enemy,
    Bomb,
    Player,
    Human,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCue {
    Credit,
    Start,
    Thrust,
    Laser,
    SmartBomb,
    Hyperspace,
    HyperspaceMaterialize,
    Explosion,
    LanderPickup,
    HumanPulled,
    HumanReleased,
    HumanRescued,
    HumanSafeLanding,
    HumanLost,
    MutantSpawn,
    BomberHit,
    BombHit,
    PodHit,
    SwarmerHit,
    SwarmerShot,
    BaiterHit,
    BaiterShot,
    AttractPulse,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScript {
    events: Vec<AttractScriptEvent>,
}

impl AttractScript {
    pub fn new(mut events: Vec<AttractScriptEvent>) -> Self {
        events.sort_by_key(|event| event.start_after_steps);
        Self { events }
    }

    pub fn red_label_title() -> Self {
        Self::new(vec![
            AttractScriptEvent::williams_logo(1, None, Point::new(94, 34)),
            AttractScriptEvent::defender_wordmark(
                DEFENDER_WORDMARK_START_STEP,
                None,
                Point::new(88, 78),
            ),
            AttractScriptEvent::text(1, None, Point::new(78, 176), "HIGH SCORES"),
        ])
    }

    fn draws_for(&self, actor: ActorId, step: u64) -> Vec<DrawCommand> {
        self.events
            .iter()
            .filter(|event| event.active_at(step))
            .map(|event| event.draw(actor, step))
            .collect()
    }
}

impl Default for AttractScript {
    fn default() -> Self {
        Self::red_label_title()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScriptEvent {
    pub start_after_steps: u64,
    pub duration_steps: Option<u64>,
    pub action: AttractScriptAction,
}

impl AttractScriptEvent {
    pub fn text(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
        value: impl Into<String>,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::Text {
                position,
                value: value.into(),
            },
        }
    }

    pub fn sprite(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        sprite: SpriteKey,
        position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::Sprite { sprite, position },
        }
    }

    pub fn williams_logo(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::WilliamsLogo {
                position,
                reveal_steps: WILLIAMS_REVEAL_STEPS,
                color_period: WILLIAMS_COLOR_PERIOD,
            },
        }
    }

    pub fn defender_wordmark(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::DefenderWordmark {
                position,
                slots: DEFENDER_WORDMARK_SLOTS,
                row_pairs: DEFENDER_WORDMARK_ROW_PAIRS,
            },
        }
    }

    fn active_at(&self, step: u64) -> bool {
        if step < self.start_after_steps {
            return false;
        }
        match self.duration_steps {
            Some(duration_steps) => step < self.start_after_steps.saturating_add(duration_steps),
            None => true,
        }
    }

    fn draw(&self, actor: ActorId, step: u64) -> DrawCommand {
        self.action.draw(actor, self.age(step))
    }

    fn age(&self, step: u64) -> u64 {
        step.saturating_sub(self.start_after_steps)
            .saturating_add(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttractScriptAction {
    Text {
        position: Point,
        value: String,
    },
    Sprite {
        sprite: SpriteKey,
        position: Point,
    },
    WilliamsLogo {
        position: Point,
        reveal_steps: u16,
        color_period: u16,
    },
    DefenderWordmark {
        position: Point,
        slots: u16,
        row_pairs: u16,
    },
}

impl AttractScriptAction {
    fn draw(&self, actor: ActorId, age: u64) -> DrawCommand {
        match self {
            Self::Text { position, value } => DrawCommand::text(actor, *position, value.clone()),
            Self::Sprite { sprite, position } => DrawCommand::sprite(actor, *sprite, *position),
            Self::WilliamsLogo {
                position,
                reveal_steps,
                color_period,
            } => {
                let color_period = (*color_period).max(1);
                let color_phase = ((age.saturating_sub(1) / u64::from(color_period)) % 4) as u8;
                DrawCommand::sprite_with_effect(
                    actor,
                    SpriteKey::WilliamsLogo,
                    *position,
                    VisualEffect::WilliamsReveal {
                        stroke_step: (age as u16).min(*reveal_steps),
                        color_phase,
                    },
                )
            }
            Self::DefenderWordmark {
                position,
                slots,
                row_pairs,
            } => {
                let row_pairs = (*row_pairs).max(1);
                let progress = age.saturating_sub(1) as u16;
                let total_steps = slots.saturating_mul(row_pairs);
                if progress >= total_steps {
                    DrawCommand::sprite(actor, SpriteKey::DefenderWordmark, *position)
                } else {
                    DrawCommand::sprite_with_effect(
                        actor,
                        SpriteKey::DefenderCoalescence,
                        *position,
                        VisualEffect::DefenderCoalescence {
                            slot: (progress / row_pairs) as u8,
                            row_pair: (progress % row_pairs) as u8,
                        },
                    )
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionBody {
    pub owner: ActorId,
    pub kind: ActorKind,
    pub bounds: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSnapshot {
    pub id: ActorId,
    pub kind: ActorKind,
    pub position: Point,
    pub bounds: Option<Rect>,
    pub alive: bool,
    pub source_lander: Option<ActorSourceLanderMetadata>,
    pub source_bomber: Option<ActorSourceBomberMetadata>,
    pub source_pod: Option<ActorSourcePodMetadata>,
    pub source_swarmer: Option<ActorSourceSwarmerMetadata>,
    pub source_baiter: Option<ActorSourceBaiterMetadata>,
    pub source_human: Option<ActorSourceHumanMetadata>,
}

impl ActorSnapshot {
    fn collision_body(&self) -> Option<CollisionBody> {
        Some(CollisionBody {
            owner: self.id,
            kind: self.kind,
            bounds: self.bounds?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanMode {
    Grounded,
    Falling { velocity: i16 },
    CarriedBy(ActorId),
}

impl HumanMode {
    const fn sprite(self) -> SpriteKey {
        match self {
            Self::Grounded => SpriteKey::Human,
            Self::Falling { .. } => SpriteKey::HumanFalling,
            Self::CarriedBy(_) => SpriteKey::HumanCarried,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawCommand {
    pub actor: ActorId,
    pub sprite: SpriteKey,
    pub position: Point,
    pub effect: VisualEffect,
    pub text: Option<String>,
}

impl DrawCommand {
    pub fn sprite(actor: ActorId, sprite: SpriteKey, position: Point) -> Self {
        Self::sprite_with_effect(actor, sprite, position, VisualEffect::Static)
    }

    pub fn sprite_with_effect(
        actor: ActorId,
        sprite: SpriteKey,
        position: Point,
        effect: VisualEffect,
    ) -> Self {
        Self {
            actor,
            sprite,
            position,
            effect,
            text: None,
        }
    }

    pub fn text(actor: ActorId, position: Point, value: impl Into<String>) -> Self {
        Self {
            actor,
            sprite: SpriteKey::Text,
            position,
            effect: VisualEffect::Static,
            text: Some(value.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnRequest {
    Laser {
        position: Point,
        direction: Direction,
        owner: ActorId,
    },
    EnemyLaser {
        position: Point,
        velocity: Velocity,
    },
    Lander {
        position: Point,
    },
    Mutant {
        position: Point,
    },
    Bomber {
        position: Point,
    },
    Bomb {
        position: Point,
    },
    Pod {
        position: Point,
    },
    Swarmer {
        position: Point,
        source: Option<ActorSourceSwarmerMetadata>,
    },
    Baiter {
        position: Point,
        source: Option<ActorSourceBaiterMetadata>,
    },
    Human {
        position: Point,
        mode: HumanMode,
    },
    Explosion {
        position: Point,
        kind: ExplosionKind,
    },
    ScorePopup {
        position: Point,
        points: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommand {
    Credit,
    StartOnePlayer,
    StartTwoPlayer,
    Spawn(SpawnRequest),
    Destroy(ActorId),
    AttachHuman {
        lander: ActorId,
        human: ActorId,
        position: Point,
    },
    SmartBomb {
        consume_stock: bool,
    },
    Hyperspace,
    HumanLost(ActorId),
    AddScore(u32),
    PlaySound(SoundCue),
    PlayerKilled,
    AdvanceWave {
        wave: u16,
    },
    EnterGameOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepPrompt {
    pub step: u64,
    pub phase: Phase,
    pub input: GameInput,
    pub wave: u16,
    pub score: u32,
    pub credits: u8,
    pub lives: u8,
    pub smart_bombs: u8,
    pub high_scores: [u32; 5],
    pub snapshots: Vec<ActorSnapshot>,
    pub behavior_script: ActorBehaviorScript,
}

impl StepPrompt {
    pub fn behavior_for(&self, actor: ActorId, kind: ActorKind) -> ActorBehaviorProfile {
        self.behavior_script.behavior_for(actor, kind)
    }

    pub fn player_position(&self) -> Option<Point> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.position)
    }

    fn snapshot(&self, id: ActorId) -> Option<&ActorSnapshot> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.id == id && snapshot.alive)
    }

    fn nearest_human(&self, position: Point) -> Option<&ActorSnapshot> {
        self.snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
            .min_by_key(|snapshot| manhattan_distance(position, snapshot.position))
    }

    fn source_target_human(&self, target_slot_index: usize) -> Option<&ActorSnapshot> {
        self.snapshots.iter().find(|snapshot| {
            snapshot.kind == ActorKind::Human
                && snapshot.alive
                && snapshot.bounds.is_some()
                && snapshot
                    .source_human
                    .is_some_and(|source| source.target_slot_index == target_slot_index)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorReply {
    pub id: ActorId,
    pub snapshot: ActorSnapshot,
    pub commands: Vec<GameCommand>,
    pub draws: Vec<DrawCommand>,
}

pub trait AssetActor: Send + 'static {
    fn id(&self) -> ActorId;

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply;
}

enum ActorRequest {
    Prompt(Box<StepPrompt>),
    Stop,
}

struct ThreadedAsset {
    sender: Sender<ActorRequest>,
    receiver: Receiver<ActorReply>,
    handle: Option<JoinHandle<()>>,
}

impl ThreadedAsset {
    fn spawn(actor: impl AssetActor) -> Self {
        let (request_sender, request_receiver) = mpsc::channel();
        let (reply_sender, reply_receiver) = mpsc::channel();
        let handle = thread::spawn(move || run_actor_thread(actor, request_receiver, reply_sender));
        Self {
            sender: request_sender,
            receiver: reply_receiver,
            handle: Some(handle),
        }
    }

    fn prompt(&self, prompt: StepPrompt) -> Option<ActorReply> {
        self.sender
            .send(ActorRequest::Prompt(Box::new(prompt)))
            .ok()?;
        self.receiver.recv().ok()
    }
}

impl Drop for ThreadedAsset {
    fn drop(&mut self) {
        let _ = self.sender.send(ActorRequest::Stop);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_actor_thread(
    mut actor: impl AssetActor,
    receiver: Receiver<ActorRequest>,
    sender: Sender<ActorReply>,
) {
    while let Ok(request) = receiver.recv() {
        match request {
            ActorRequest::Prompt(prompt) => {
                if sender.send(actor.update(prompt.as_ref())).is_err() {
                    break;
                }
            }
            ActorRequest::Stop => break,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepReport {
    pub step: u64,
    pub phase: Phase,
    pub wave: u16,
    pub score: u32,
    pub credits: u8,
    pub lives: u8,
    pub smart_bombs: u8,
    pub snapshots: Vec<ActorSnapshot>,
    pub draws: Vec<DrawCommand>,
    pub sounds: Vec<SoundCue>,
    pub commands: Vec<GameCommand>,
}

pub struct ActorGameDriver {
    step: u64,
    phase: Phase,
    wave: u16,
    score: u32,
    credits: u8,
    lives: u8,
    smart_bombs: u8,
    next_actor_id: u64,
    actors: BTreeMap<ActorId, ThreadedAsset>,
    snapshots: BTreeMap<ActorId, ActorSnapshot>,
    high_scores: HighScoreTable,
    behavior_script: ActorBehaviorScript,
    wave_script: ActorWaveScript,
    baiter_timer_steps: Option<u32>,
    baiter_pacing_steps_remaining: u8,
}

impl ActorGameDriver {
    pub fn new() -> Self {
        Self::with_attract_script(AttractScript::red_label_title())
    }

    pub fn with_attract_script(attract_script: AttractScript) -> Self {
        Self::with_attract_and_wave_scripts(attract_script, ActorWaveScript::default())
    }

    pub fn with_wave_script(wave_script: ActorWaveScript) -> Self {
        Self::with_attract_and_wave_scripts(AttractScript::red_label_title(), wave_script)
    }

    pub fn with_attract_and_wave_scripts(
        attract_script: AttractScript,
        wave_script: ActorWaveScript,
    ) -> Self {
        let mut driver = Self {
            step: 0,
            phase: Phase::Attract,
            wave: 0,
            score: 0,
            credits: 0,
            lives: 3,
            smart_bombs: 0,
            next_actor_id: 1,
            actors: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            high_scores: HighScoreTable::default(),
            behavior_script: ActorBehaviorScript::default(),
            wave_script,
            baiter_timer_steps: None,
            baiter_pacing_steps_remaining: ACTOR_BAITER_TIMER_PACING_STEPS,
        };
        let attract_id = driver.allocate_actor_id();
        let script_id = driver.allocate_actor_id();
        let status_id = driver.allocate_actor_id();
        driver.spawn_actor(AttractDirector::new(attract_id));
        driver.spawn_actor(ScriptedAttractProgram::new(script_id, attract_script));
        driver.spawn_actor(StatusDisplay::new(status_id));
        driver
    }

    pub fn step(&mut self, input: GameInput) -> StepReport {
        self.step = self.step.saturating_add(1);
        let was_playing = self.phase == Phase::Playing;
        let behavior_script = self
            .behavior_script
            .with_input_overrides(input, self.snapshots.values().cloned());
        let base_prompt = StepPrompt {
            step: self.step,
            phase: self.phase,
            input,
            wave: self.wave,
            score: self.score,
            credits: self.credits,
            lives: self.lives,
            smart_bombs: self.smart_bombs,
            high_scores: self.high_scores.entries(),
            snapshots: self.snapshots.values().cloned().collect(),
            behavior_script: behavior_script.clone(),
        };

        let mut replies = Vec::new();
        for (id, actor) in &self.actors {
            if let Some(reply) = actor.prompt(base_prompt.clone()) {
                replies.push((*id, reply));
            }
        }
        replies.sort_by_key(|(id, _)| *id);

        let mut draws = Vec::new();
        let mut commands = Vec::new();
        let mut dead_actor_ids = Vec::new();
        self.snapshots.clear();
        for (_, reply) in replies {
            if reply.snapshot.alive {
                self.snapshots.insert(reply.id, reply.snapshot);
            } else {
                dead_actor_ids.push(reply.id);
            }
            draws.extend(reply.draws);
            commands.extend(reply.commands);
        }

        self.resolve_collisions(&behavior_script, &mut commands);
        self.advance_baiter_timer(&mut commands);
        let sounds = self.apply_commands(&commands);
        self.remove_dead_actors(&dead_actor_ids);
        if self.advance_wave_if_cleared(was_playing, &commands) {
            commands.push(GameCommand::AdvanceWave { wave: self.wave });
        }

        StepReport {
            step: self.step,
            phase: self.phase,
            wave: self.wave,
            score: self.score,
            credits: self.credits,
            lives: self.lives,
            smart_bombs: self.smart_bombs,
            snapshots: self.snapshots.values().cloned().collect(),
            draws,
            sounds,
            commands,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn wave(&self) -> u16 {
        self.wave
    }

    pub fn actor_count(&self) -> usize {
        self.actors.len()
    }

    pub fn wave_script_name(&self) -> &str {
        self.wave_script.name()
    }

    pub fn behavior_script(&self) -> &ActorBehaviorScript {
        &self.behavior_script
    }

    pub fn behavior_script_mut(&mut self) -> &mut ActorBehaviorScript {
        &mut self.behavior_script
    }

    pub fn set_default_behavior(&mut self, profile: ActorBehaviorProfile) {
        self.behavior_script.set_default_profile(profile);
    }

    pub fn set_kind_behavior(&mut self, kind: ActorKind, profile: ActorBehaviorProfile) {
        self.behavior_script.set_kind_behavior(kind, profile);
    }

    pub fn set_actor_behavior(&mut self, actor: ActorId, profile: ActorBehaviorProfile) {
        self.behavior_script.set_actor_behavior(actor, profile);
    }

    pub fn snapshot_count(&self, kind: ActorKind) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == kind)
            .count()
    }

    pub fn spawn_lander_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_lander(position)
    }

    pub fn spawn_bomber_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_bomber(position)
    }

    pub fn spawn_bomb_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_bomb(position)
    }

    pub fn spawn_pod_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_pod(position)
    }

    pub fn spawn_swarmer_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_swarmer(position)
    }

    pub fn spawn_baiter_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_baiter(position)
    }

    pub fn set_baiter_timer_for_test(&mut self, timer_steps: u32) {
        self.baiter_timer_steps = Some(timer_steps.max(1));
        self.baiter_pacing_steps_remaining = 1;
    }

    pub fn spawn_human_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_human(position, HumanMode::Grounded)
    }

    pub fn spawn_falling_human_for_test(&mut self, position: Point, velocity: i16) -> ActorId {
        self.spawn_human(position, HumanMode::Falling { velocity })
    }

    pub fn spawn_carried_human_for_test(&mut self, position: Point, carrier: ActorId) -> ActorId {
        self.spawn_human(position, HumanMode::CarriedBy(carrier))
    }

    fn allocate_actor_id(&mut self) -> ActorId {
        let id = ActorId::new(self.next_actor_id);
        self.next_actor_id = self.next_actor_id.saturating_add(1);
        id
    }

    fn spawn_actor(&mut self, actor: impl AssetActor) {
        let id = actor.id();
        self.actors.insert(id, ThreadedAsset::spawn(actor));
    }

    fn spawn_player(&mut self) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(PlayerShip::new(id, Point::new(42, 120)));
        id
    }

    fn spawn_lander(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Lander::new(id, position));
        id
    }

    fn spawn_lander_from_spawn(&mut self, spawn: ActorLanderSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Lander::from_spawn(id, spawn));
        id
    }

    fn spawn_mutant(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Mutant::new(id, position));
        id
    }

    fn spawn_bomber(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Bomber::new(id, position));
        id
    }

    fn spawn_bomber_from_spawn(&mut self, spawn: ActorBomberSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Bomber::from_spawn(id, spawn));
        id
    }

    fn spawn_bomb(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        let lifetime = self
            .behavior_script
            .behavior_for(id, ActorKind::Bomb)
            .bomb_lifetime_steps;
        self.spawn_actor(Bomb::new(id, position, lifetime));
        id
    }

    fn spawn_pod(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Pod::new(id, position));
        id
    }

    fn spawn_pod_from_spawn(&mut self, spawn: ActorPodSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Pod::from_spawn(id, spawn));
        id
    }

    fn spawn_swarmer(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Swarmer::new(id, position));
        id
    }

    fn spawn_swarmer_from_spawn(&mut self, spawn: ActorSwarmerSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Swarmer::from_spawn(id, spawn));
        id
    }

    fn spawn_baiter(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Baiter::new(id, position));
        id
    }

    fn spawn_baiter_from_spawn(&mut self, spawn: ActorBaiterSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Baiter::from_spawn(id, spawn));
        id
    }

    fn spawn_human(&mut self, position: Point, mode: HumanMode) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Human::new(id, position, mode));
        id
    }

    fn spawn_human_from_spawn(&mut self, spawn: ActorHumanSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Human::from_spawn(id, spawn));
        id
    }

    fn spawn_laser(&mut self, position: Point, direction: Direction, owner: ActorId) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(LaserShot::new(id, position, direction, owner));
        id
    }

    fn spawn_enemy_laser(&mut self, position: Point, velocity: Velocity) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(EnemyLaserShot::new(id, position, velocity));
        id
    }

    fn spawn_explosion(&mut self, position: Point, kind: ExplosionKind) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Explosion::new(id, position, kind));
        id
    }

    fn spawn_score_popup(&mut self, position: Point, points: u32) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(ScorePopup::new(id, position, points));
        id
    }

    fn resolve_collisions(
        &self,
        behavior_script: &ActorBehaviorScript,
        commands: &mut Vec<GameCommand>,
    ) {
        if self.phase != Phase::Playing {
            return;
        }

        let bodies = self
            .snapshots
            .values()
            .filter_map(ActorSnapshot::collision_body)
            .collect::<Vec<_>>();
        let mut destroyed = BTreeSet::new();
        for laser in bodies.iter().filter(|body| body.kind == ActorKind::Laser) {
            for enemy in bodies
                .iter()
                .filter(|body| is_player_laser_target(body.kind))
            {
                if laser.bounds.intersects(enemy.bounds) {
                    destroyed.insert(laser.owner);
                    destroyed.insert(enemy.owner);
                    commands.push(GameCommand::Destroy(laser.owner));
                    commands.push(GameCommand::Destroy(enemy.owner));
                    commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                        position: center_of(enemy.bounds),
                        kind: explosion_kind_for_target(enemy.kind),
                    }));
                    commands.push(GameCommand::AddScore(score_for_hostile(enemy.kind)));
                    commands.push(GameCommand::PlaySound(hit_sound_for_hostile(enemy.kind)));
                    if enemy.kind == ActorKind::Pod {
                        commands.extend(self.pod_swarmer_spawn_commands(center_of(enemy.bounds)));
                    }
                    break;
                }
            }
        }

        let Some(player) = bodies.iter().find(|body| body.kind == ActorKind::Player) else {
            return;
        };
        let player_behavior = behavior_script.behavior_for(player.owner, ActorKind::Player);
        if !player_behavior.player_takes_enemy_collision_damage {
            return;
        }
        let hyperspace_clears_enemy_lasers = commands
            .iter()
            .any(|command| matches!(command, GameCommand::Hyperspace));
        for enemy in bodies.iter().filter(|body| is_player_hazard(body.kind)) {
            if destroyed.contains(&enemy.owner) {
                continue;
            }
            if hyperspace_clears_enemy_lasers && enemy.kind == ActorKind::EnemyLaser {
                continue;
            }
            if player.bounds.intersects(enemy.bounds) {
                commands.push(GameCommand::Destroy(player.owner));
                commands.push(GameCommand::Destroy(enemy.owner));
                commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                    position: center_of(player.bounds),
                    kind: player_hazard_explosion_kind(enemy.kind),
                }));
                commands.push(GameCommand::PlaySound(player_hazard_sound(enemy.kind)));
                commands.push(GameCommand::PlayerKilled);
                break;
            }
        }
    }

    fn apply_commands(&mut self, commands: &[GameCommand]) -> Vec<SoundCue> {
        let mut sounds = Vec::new();
        for command in commands {
            match *command {
                GameCommand::Credit => {
                    self.credits = self.credits.saturating_add(1);
                    sounds.push(SoundCue::Credit);
                }
                GameCommand::StartOnePlayer => {
                    if self.phase == Phase::Attract && self.credits > 0 {
                        self.credits = self.credits.saturating_sub(1);
                        self.start_play();
                        sounds.push(SoundCue::Start);
                    }
                }
                GameCommand::StartTwoPlayer => {
                    if self.phase == Phase::Attract && self.credits > 1 {
                        self.credits = self.credits.saturating_sub(2);
                        self.start_play();
                        sounds.push(SoundCue::Start);
                    }
                }
                GameCommand::Spawn(SpawnRequest::Laser {
                    position,
                    direction,
                    owner,
                }) => {
                    self.spawn_laser(position, direction, owner);
                }
                GameCommand::Spawn(SpawnRequest::EnemyLaser { position, velocity }) => {
                    self.spawn_enemy_laser(position, velocity);
                }
                GameCommand::Spawn(SpawnRequest::Lander { position }) => {
                    self.spawn_lander(position);
                }
                GameCommand::Spawn(SpawnRequest::Mutant { position }) => {
                    self.spawn_mutant(position);
                }
                GameCommand::Spawn(SpawnRequest::Bomber { position }) => {
                    self.spawn_bomber(position);
                }
                GameCommand::Spawn(SpawnRequest::Bomb { position }) => {
                    self.spawn_bomb(position);
                }
                GameCommand::Spawn(SpawnRequest::Pod { position }) => {
                    self.spawn_pod(position);
                }
                GameCommand::Spawn(SpawnRequest::Swarmer { position, source }) => {
                    self.spawn_swarmer_from_spawn(ActorSwarmerSpawn { position, source });
                }
                GameCommand::Spawn(SpawnRequest::Baiter { position, source }) => {
                    self.spawn_baiter_from_spawn(ActorBaiterSpawn { position, source });
                }
                GameCommand::Spawn(SpawnRequest::Human { position, mode }) => {
                    self.spawn_human(position, mode);
                }
                GameCommand::Spawn(SpawnRequest::Explosion { position, kind }) => {
                    self.spawn_explosion(position, kind);
                }
                GameCommand::Spawn(SpawnRequest::ScorePopup { position, points }) => {
                    self.spawn_score_popup(position, points);
                }
                GameCommand::Destroy(id) => {
                    self.snapshots.remove(&id);
                    self.actors.remove(&id);
                    self.behavior_script.remove_actor_behavior(id);
                }
                GameCommand::AttachHuman {
                    lander,
                    human,
                    position,
                } => {
                    let source = self
                        .snapshots
                        .get(&human)
                        .and_then(|snapshot| snapshot.source_human);
                    self.snapshots.remove(&human);
                    self.actors.remove(&human);
                    self.spawn_actor(Human::with_source(
                        human,
                        position,
                        HumanMode::CarriedBy(lander),
                        source,
                    ));
                }
                GameCommand::SmartBomb { consume_stock } => {
                    self.detonate_smart_bomb(&mut sounds, consume_stock);
                }
                GameCommand::Hyperspace => {
                    self.clear_enemy_projectiles_for_hyperspace();
                }
                GameCommand::HumanLost(id) => {
                    self.snapshots.remove(&id);
                    self.actors.remove(&id);
                    self.behavior_script.remove_actor_behavior(id);
                }
                GameCommand::AddScore(points) => {
                    self.score = self.score.saturating_add(points);
                }
                GameCommand::PlaySound(sound) => sounds.push(sound),
                GameCommand::PlayerKilled => {
                    self.lose_player_life(&mut sounds);
                }
                GameCommand::AdvanceWave { .. } => {}
                GameCommand::EnterGameOver => {
                    self.enter_game_over(&mut sounds);
                }
            }
        }
        sounds
    }

    fn lose_player_life(&mut self, sounds: &mut Vec<SoundCue>) {
        if self.lives > 1 {
            self.lives = self.lives.saturating_sub(1);
            self.spawn_player();
            return;
        }

        self.enter_game_over(sounds);
    }

    fn enter_game_over(&mut self, sounds: &mut Vec<SoundCue>) {
        self.lives = 0;
        self.smart_bombs = 0;
        self.wave = 0;
        self.high_scores.record(self.score);
        self.phase = if self.high_scores.qualifies(self.score) {
            Phase::HighScoreEntry
        } else {
            Phase::GameOver
        };
        self.baiter_timer_steps = None;
        sounds.push(SoundCue::GameOver);
    }

    fn start_play(&mut self) {
        self.phase = Phase::Playing;
        self.wave = 1;
        self.score = 0;
        self.lives = 3;
        self.smart_bombs = INITIAL_SMART_BOMBS;
        self.apply_wave_profile();
        self.spawn_player();
        self.spawn_wave_hostiles();
        self.spawn_initial_humans();
    }

    fn apply_wave_profile(&mut self) {
        self.behavior_script = self
            .wave_script
            .profile_for_wave(self.wave)
            .behavior_script
            .clone();
        if self.phase == Phase::Playing {
            self.reset_baiter_timer();
        }
    }

    fn reset_baiter_timer(&mut self) {
        let source_profile = ActorSourceWaveProfile::for_wave(self.wave.max(1));
        self.baiter_timer_steps = Some(source_profile.baiter_delay.max(1));
        self.baiter_pacing_steps_remaining = ACTOR_BAITER_TIMER_PACING_STEPS;
    }

    fn spawn_wave_hostiles(&mut self) {
        let lander_spawns = self
            .wave_script
            .profile_for_wave(self.wave)
            .lander_spawns
            .clone();
        for spawn in lander_spawns {
            self.spawn_lander_from_spawn(spawn);
        }
        let bomber_spawns = self
            .wave_script
            .profile_for_wave(self.wave)
            .bomber_spawns
            .clone();
        for spawn in bomber_spawns {
            self.spawn_bomber_from_spawn(spawn);
        }
        let pod_spawns = self
            .wave_script
            .profile_for_wave(self.wave)
            .pod_spawns
            .clone();
        for spawn in pod_spawns {
            self.spawn_pod_from_spawn(spawn);
        }
    }

    fn advance_wave_if_cleared(&mut self, was_playing: bool, commands: &[GameCommand]) -> bool {
        if !was_playing
            || self.phase != Phase::Playing
            || self.wave == 0
            || self.has_hostile_snapshots()
            || commands_spawn_hostiles(commands)
        {
            return false;
        }

        self.wave = self.wave.saturating_add(1);
        self.apply_wave_profile();
        self.spawn_wave_hostiles();
        true
    }

    fn has_hostile_snapshots(&self) -> bool {
        self.snapshots
            .values()
            .any(|snapshot| is_hostile(snapshot.kind))
    }

    fn pod_swarmer_spawn_commands(&self, position: Point) -> Vec<GameCommand> {
        let active_swarmers = self
            .snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Swarmer)
            .count();
        let spawn_count = SOURCE_POD_SWARMER_REQUEST_LIMIT
            .min(SOURCE_ACTIVE_SWARMER_LIMIT.saturating_sub(active_swarmers));
        let source_profile = ActorSourceWaveProfile::for_wave(self.wave.max(1));

        (0..spawn_count)
            .map(|index| {
                let spawn = ActorSwarmerSpawn::source_from_pod(position, source_profile, index);
                GameCommand::Spawn(SpawnRequest::Swarmer {
                    position: spawn.position,
                    source: spawn.source,
                })
            })
            .collect()
    }

    fn advance_baiter_timer(&mut self, commands: &mut Vec<GameCommand>) {
        if self.phase != Phase::Playing || self.wave == 0 {
            return;
        }
        let enemy_total = self.source_wave_enemy_total();
        if enemy_total == 0 {
            return;
        }
        let Some(timer_steps) = self.baiter_timer_steps else {
            return;
        };

        if self.baiter_pacing_steps_remaining > 1 {
            self.baiter_pacing_steps_remaining =
                self.baiter_pacing_steps_remaining.saturating_sub(1);
            return;
        }
        self.baiter_pacing_steps_remaining = ACTOR_BAITER_TIMER_PACING_STEPS;

        let profile = ActorSourceWaveProfile::for_wave(self.wave.max(1));
        let timer_steps = source_baiter_accelerated_timer_steps(timer_steps, profile, enemy_total);
        let decremented_steps = timer_steps.saturating_sub(1);
        if decremented_steps > 0 {
            self.baiter_timer_steps = Some(decremented_steps);
            return;
        }

        self.baiter_timer_steps = Some(source_baiter_reset_timer_steps(profile, enemy_total));
        let active_baiters = self.snapshot_count(ActorKind::Baiter);
        if active_baiters >= SOURCE_ACTIVE_BAITER_LIMIT {
            return;
        }
        let Some(player_position) = self
            .snapshots
            .values()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.position)
        else {
            return;
        };
        let spawn = ActorBaiterSpawn::source_from_player(profile, player_position, active_baiters);
        commands.push(GameCommand::Spawn(SpawnRequest::Baiter {
            position: spawn.position,
            source: spawn.source,
        }));
    }

    fn source_wave_enemy_total(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| is_hostile(snapshot.kind))
            .count()
    }

    fn spawn_initial_humans(&mut self) {
        let human_spawns = self
            .wave_script
            .profile_for_wave(self.wave)
            .human_spawns
            .clone();
        for spawn in human_spawns {
            self.spawn_human_from_spawn(spawn);
        }
    }

    fn detonate_smart_bomb(&mut self, sounds: &mut Vec<SoundCue>, consume_stock: bool) {
        if consume_stock {
            if self.smart_bombs == 0 {
                return;
            }
            self.smart_bombs = self.smart_bombs.saturating_sub(1);
        }

        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| is_smart_bomb_target(snapshot.kind))
            .map(|snapshot| (snapshot.id, snapshot.kind, snapshot.position))
            .collect::<Vec<_>>();
        for (id, kind, position) in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
            self.spawn_explosion(position, explosion_kind_for_target(kind));
            self.score = self.score.saturating_add(score_for_hostile(kind));
            sounds.push(SoundCue::Explosion);
        }
    }

    fn clear_enemy_projectiles_for_hyperspace(&mut self) {
        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>();
        for id in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
        }
    }

    fn remove_dead_actors(&mut self, dead_actor_ids: &[ActorId]) {
        for id in dead_actor_ids {
            self.snapshots.remove(id);
            self.actors.remove(id);
            self.behavior_script.remove_actor_behavior(*id);
        }
    }
}

impl Default for ActorGameDriver {
    fn default() -> Self {
        Self::new()
    }
}

fn center_of(bounds: Rect) -> Point {
    Point::new(
        (bounds.left + bounds.right) / 2,
        (bounds.top + bounds.bottom) / 2,
    )
}

fn manhattan_distance(left: Point, right: Point) -> i16 {
    (left.x - right.x).abs() + (left.y - right.y).abs()
}

fn is_hostile(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Pod
            | ActorKind::Swarmer
    )
}

fn is_player_laser_target(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::EnemyLaser
    )
}

fn is_player_hazard(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::EnemyLaser
    )
}

fn is_smart_bomb_target(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::EnemyLaser
    )
}

fn commands_spawn_hostiles(commands: &[GameCommand]) -> bool {
    commands.iter().any(|command| {
        matches!(
            command,
            GameCommand::Spawn(SpawnRequest::Lander { .. })
                | GameCommand::Spawn(SpawnRequest::Mutant { .. })
                | GameCommand::Spawn(SpawnRequest::Bomber { .. })
                | GameCommand::Spawn(SpawnRequest::Pod { .. })
                | GameCommand::Spawn(SpawnRequest::Swarmer { .. })
        )
    })
}

fn score_for_hostile(kind: ActorKind) -> u32 {
    match kind {
        ActorKind::Lander => LANDER_SCORE,
        ActorKind::Mutant => MUTANT_SCORE,
        ActorKind::Bomber => BOMBER_SCORE,
        ActorKind::Bomb => 0,
        ActorKind::Pod => POD_SCORE,
        ActorKind::Swarmer => SWARMER_SCORE,
        ActorKind::Baiter => BAITER_SCORE,
        _ => 0,
    }
}

fn hit_sound_for_hostile(kind: ActorKind) -> SoundCue {
    match kind {
        ActorKind::Bomber => SoundCue::BomberHit,
        ActorKind::Bomb => SoundCue::BombHit,
        ActorKind::Pod => SoundCue::PodHit,
        ActorKind::Swarmer => SoundCue::SwarmerHit,
        ActorKind::Baiter => SoundCue::BaiterHit,
        _ => SoundCue::Explosion,
    }
}

fn player_hazard_sound(kind: ActorKind) -> SoundCue {
    match kind {
        ActorKind::Bomb => SoundCue::BombHit,
        _ => SoundCue::Explosion,
    }
}

fn explosion_kind_for_target(kind: ActorKind) -> ExplosionKind {
    match kind {
        ActorKind::Bomb => ExplosionKind::Bomb,
        _ => ExplosionKind::Enemy,
    }
}

fn player_hazard_explosion_kind(kind: ActorKind) -> ExplosionKind {
    match kind {
        ActorKind::Bomb => ExplosionKind::Bomb,
        _ => ExplosionKind::Player,
    }
}

fn source_baiter_accelerated_timer_steps(
    current_steps: u32,
    profile: ActorSourceWaveProfile,
    enemy_total: usize,
) -> u32 {
    if enemy_total > 8 {
        return current_steps;
    }

    let mut target_steps = profile.baiter_delay / 2;
    if enemy_total <= 3 {
        target_steps /= 2;
    }
    target_steps = target_steps.saturating_add(1).max(1);
    current_steps.min(target_steps)
}

fn source_baiter_reset_timer_steps(profile: ActorSourceWaveProfile, enemy_total: usize) -> u32 {
    if enemy_total < 4 {
        (profile.baiter_delay / 4).max(1)
    } else {
        profile.baiter_delay.max(1)
    }
}

#[derive(Debug, Clone)]
struct HighScoreTable {
    entries: [u32; 5],
}

impl HighScoreTable {
    fn entries(&self) -> [u32; 5] {
        self.entries
    }

    fn qualifies(&self, score: u32) -> bool {
        self.entries.iter().any(|entry| score > *entry)
    }

    fn record(&mut self, score: u32) {
        if !self.qualifies(score) {
            return;
        }
        let mut entries = self.entries.to_vec();
        entries.push(score);
        entries.sort_by(|left, right| right.cmp(left));
        self.entries.copy_from_slice(&entries[..5]);
    }
}

impl Default for HighScoreTable {
    fn default() -> Self {
        Self {
            entries: [10_000, 7_500, 5_000, 2_500, 1_000],
        }
    }
}

#[derive(Debug)]
struct AttractDirector {
    id: ActorId,
}

impl AttractDirector {
    fn new(id: ActorId) -> Self {
        Self { id }
    }
}

impl AssetActor for AttractDirector {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        for _ in 0..prompt.input.coin_insertions() {
            commands.push(GameCommand::Credit);
        }
        if prompt.input.start_one {
            commands.push(GameCommand::StartOnePlayer);
        }
        if prompt.input.start_two {
            commands.push(GameCommand::StartTwoPlayer);
        }
        if prompt.phase == Phase::HighScoreEntry {
            draws.push(DrawCommand::text(
                self.id,
                Point::new(66, 120),
                "ENTER INITIALS",
            ));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::AttractDirector,
                position: Point::new(0, 0),
                bounds: None,
                alive: true,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct ScriptedAttractProgram {
    id: ActorId,
    script: AttractScript,
    elapsed_steps: u64,
}

impl ScriptedAttractProgram {
    fn new(id: ActorId, script: AttractScript) -> Self {
        Self {
            id,
            script,
            elapsed_steps: 0,
        }
    }
}

impl AssetActor for ScriptedAttractProgram {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let draws = if matches!(prompt.phase, Phase::Attract | Phase::GameOver) {
            self.elapsed_steps = self.elapsed_steps.saturating_add(1);
            self.script.draws_for(self.id, self.elapsed_steps)
        } else {
            self.elapsed_steps = 0;
            Vec::new()
        };

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::AttractScript,
                position: Point::new(0, 0),
                bounds: None,
                alive: true,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands: Vec::new(),
            draws,
        }
    }
}

#[derive(Debug)]
struct StatusDisplay {
    id: ActorId,
}

impl StatusDisplay {
    fn new(id: ActorId) -> Self {
        Self { id }
    }

    fn playing_draws(&self, prompt: &StepPrompt) -> Vec<DrawCommand> {
        vec![
            DrawCommand::text(
                self.id,
                STATUS_SCORE_POSITION,
                format!("1UP {}", format_status_score(prompt.score)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_HIGH_SCORE_POSITION,
                format!("HIGH {}", format_status_score(prompt.high_scores[0])),
            ),
            DrawCommand::text(
                self.id,
                STATUS_WAVE_POSITION,
                format!("WAVE {:02}", prompt.wave.min(99)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_LIVES_POSITION,
                format!("LIVES {:02}", prompt.lives.min(99)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_SMART_BOMBS_POSITION,
                format!("BOMBS {:02}", prompt.smart_bombs.min(99)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_CREDITS_POSITION,
                format!("CREDIT {:02}", prompt.credits.min(99)),
            ),
        ]
    }

    fn high_score_entry_draws(&self, prompt: &StepPrompt) -> Vec<DrawCommand> {
        let mut draws = vec![
            DrawCommand::text(
                self.id,
                STATUS_FINAL_SCORE_POSITION,
                format!("FINAL SCORE {}", format_status_score(prompt.score)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_HIGH_SCORE_TABLE_TITLE_POSITION,
                "HIGH SCORES",
            ),
        ];

        for (index, score) in prompt.high_scores.iter().enumerate() {
            draws.push(DrawCommand::text(
                self.id,
                Point::new(
                    82,
                    STATUS_HIGH_SCORE_TABLE_START_Y
                        + i16::try_from(index).unwrap_or(0) * STATUS_HIGH_SCORE_TABLE_ROW_HEIGHT,
                ),
                format!("{}. {}", index + 1, format_status_score(*score)),
            ));
        }
        draws
    }

    fn snapshot(&self) -> ActorSnapshot {
        ActorSnapshot {
            id: self.id,
            kind: ActorKind::StatusDisplay,
            position: Point::new(0, 0),
            bounds: None,
            alive: true,
            source_lander: None,
            source_bomber: None,
            source_pod: None,
            source_swarmer: None,
            source_baiter: None,
            source_human: None,
        }
    }
}

impl AssetActor for StatusDisplay {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let draws = match prompt.phase {
            Phase::Playing => self.playing_draws(prompt),
            Phase::HighScoreEntry => self.high_score_entry_draws(prompt),
            Phase::Attract | Phase::GameOver => Vec::new(),
        };

        ActorReply {
            id: self.id,
            snapshot: self.snapshot(),
            commands: Vec::new(),
            draws,
        }
    }
}

fn format_status_score(score: u32) -> String {
    format!("{:06}", score.min(999_999))
}

#[derive(Debug)]
struct PlayerShip {
    id: ActorId,
    position: Point,
    direction: Direction,
    laser_cooldown: u8,
    hyperspace_steps_remaining: u8,
    hyperspace_death_steps_remaining: Option<u8>,
}

impl PlayerShip {
    fn new(id: ActorId, position: Point) -> Self {
        Self {
            id,
            position,
            direction: Direction::Right,
            laser_cooldown: 0,
            hyperspace_steps_remaining: 0,
            hyperspace_death_steps_remaining: None,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 18, 10)
    }

    fn is_hidden_for_hyperspace(&self) -> bool {
        self.hyperspace_steps_remaining > 0
    }

    fn enter_hyperspace(&mut self, behavior: ActorBehaviorProfile) {
        self.hyperspace_steps_remaining = behavior.player_hyperspace_hidden_steps;
    }

    fn hyperspace_rematerialization(
        &self,
        behavior: ActorBehaviorProfile,
    ) -> (Point, Direction, u8) {
        if let Some(source) = behavior.player_hyperspace_source_seed {
            let (x, direction) = if source.hseed & 1 != 0 {
                (0x20, Direction::Right)
            } else {
                (0x70, Direction::Left)
            };
            let y = (source.hseed >> 1).wrapping_add(SOURCE_PLAYFIELD_Y_MIN);
            return (Point::new(x, i16::from(y)), direction, source.lseed);
        }

        (
            Point::new(
                behavior.player_hyperspace_rematerialize_x,
                behavior.player_hyperspace_rematerialize_y,
            ),
            self.direction,
            behavior.player_hyperspace_death_lseed,
        )
    }

    fn advance_hyperspace(&mut self, behavior: ActorBehaviorProfile) -> bool {
        if self.hyperspace_steps_remaining == 0 {
            return false;
        }

        self.hyperspace_steps_remaining = self.hyperspace_steps_remaining.saturating_sub(1);
        if self.hyperspace_steps_remaining == 0 {
            let (position, direction, death_lseed) = self.hyperspace_rematerialization(behavior);
            self.position = PLAYER_BOUNDS.clamp_point(position);
            self.direction = direction;
            if death_lseed > SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD {
                self.hyperspace_death_steps_remaining =
                    Some(behavior.player_hyperspace_death_delay_steps);
            }
            return true;
        }
        false
    }

    fn advance_hyperspace_death(&mut self, commands: &mut Vec<GameCommand>) -> bool {
        let Some(steps_remaining) = self.hyperspace_death_steps_remaining else {
            return false;
        };

        let steps_remaining = steps_remaining.saturating_sub(1);
        if steps_remaining > 0 {
            self.hyperspace_death_steps_remaining = Some(steps_remaining);
            return false;
        }

        self.hyperspace_death_steps_remaining = None;
        commands.push(GameCommand::Destroy(self.id));
        commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
            position: self.position,
            kind: ExplosionKind::Player,
        }));
        commands.push(GameCommand::PlaySound(SoundCue::Explosion));
        commands.push(GameCommand::PlayerKilled);
        true
    }
}

impl AssetActor for PlayerShip {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let mut death_due = false;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Player);
            death_due = self.advance_hyperspace_death(&mut commands);
            let was_hidden = self.is_hidden_for_hyperspace();
            if self.advance_hyperspace(behavior) {
                commands.push(GameCommand::PlaySound(SoundCue::HyperspaceMaterialize));
            }
            let input_blocked = was_hidden || self.hyperspace_death_steps_remaining.is_some();
            if !death_due && !input_blocked {
                let mut velocity = Velocity::default();
                if prompt.input.altitude_up {
                    velocity.dy -= behavior.player_speed;
                }
                if prompt.input.altitude_down {
                    velocity.dy += behavior.player_speed;
                }
                if prompt.input.thrust {
                    velocity.dx += self.direction.sign() * behavior.player_speed;
                    commands.push(GameCommand::PlaySound(SoundCue::Thrust));
                }
                if prompt.input.reverse {
                    self.direction = match self.direction {
                        Direction::Left => Direction::Right,
                        Direction::Right => Direction::Left,
                    };
                }
                self.position = PLAYER_BOUNDS.clamp_point(self.position.offset(velocity));
                self.laser_cooldown = self.laser_cooldown.saturating_sub(1);
                if prompt.input.wants_fire() && self.laser_cooldown == 0 {
                    self.laser_cooldown = behavior.player_laser_cooldown_steps;
                    let muzzle = self
                        .position
                        .offset(Velocity::new(self.direction.sign() * 12, 0));
                    commands.push(GameCommand::Spawn(SpawnRequest::Laser {
                        position: muzzle,
                        direction: self.direction,
                        owner: self.id,
                    }));
                    commands.push(GameCommand::PlaySound(SoundCue::Laser));
                }
                if prompt.input.xyzzy.overlay_smart_bomb {
                    commands.push(GameCommand::SmartBomb {
                        consume_stock: false,
                    });
                    commands.push(GameCommand::PlaySound(SoundCue::SmartBomb));
                } else if prompt.input.wants_stock_smart_bomb() && prompt.smart_bombs > 0 {
                    commands.push(GameCommand::SmartBomb {
                        consume_stock: true,
                    });
                    commands.push(GameCommand::PlaySound(SoundCue::SmartBomb));
                }
                if prompt.input.hyperspace {
                    commands.push(GameCommand::Hyperspace);
                    commands.push(GameCommand::PlaySound(SoundCue::Hyperspace));
                    self.enter_hyperspace(behavior);
                }
            }
            if !death_due && !self.is_hidden_for_hyperspace() {
                draws.push(DrawCommand::sprite(
                    self.id,
                    match self.direction {
                        Direction::Left => SpriteKey::PlayerLeft,
                        Direction::Right => SpriteKey::PlayerRight,
                    },
                    self.position,
                ));
            }
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Player,
                position: self.position,
                bounds: if prompt.phase == Phase::Playing
                    && !self.is_hidden_for_hyperspace()
                    && !death_due
                {
                    Some(self.bounds())
                } else {
                    None
                },
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct Lander {
    id: ActorId,
    position: Point,
    drift: i16,
    mode: LanderMode,
    source: Option<ActorSourceLanderMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LanderMode {
    Seeking,
    Carrying {
        human_id: ActorId,
        pull_sound_sent: bool,
    },
}

impl Lander {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorLanderSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorLanderSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .source
                .map(|source| source_lander_drift_from_velocity(source.x_velocity))
                .unwrap_or(-1),
            mode: LanderMode::Seeking,
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 14, 12)
    }
}

impl AssetActor for Lander {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Lander);
            if !self.tick_source_sleep() {
                match self.mode {
                    LanderMode::Seeking => self.update_seeking(prompt, behavior, &mut commands),
                    LanderMode::Carrying {
                        human_id,
                        pull_sound_sent,
                    } => {
                        self.update_carrying(human_id, pull_sound_sent, behavior, &mut commands);
                    }
                }
                self.tick_fire_timer(prompt, behavior, &mut commands);
            }
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Lander,
                self.position,
                self.draw_effect(),
            ));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Lander,
                position: self.position,
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: self.source,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands,
            draws,
        }
    }
}

impl Lander {
    fn update_seeking(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        match behavior.lander_mode {
            LanderBehaviorMode::SeekNearestHuman => {
                self.seek_nearest_human(prompt, behavior, commands);
            }
            LanderBehaviorMode::ChasePlayer => {
                if let Some(player) = prompt.player_position() {
                    self.position = step_toward(self.position, player, behavior.lander_seek_speed);
                } else {
                    self.drift(behavior);
                }
            }
            LanderBehaviorMode::Drift => {
                if !self.advance_source_fixed_point_motion() {
                    self.drift(behavior);
                }
            }
        }
    }

    fn seek_nearest_human(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        let target = self
            .source_target_human(prompt)
            .or_else(|| prompt.nearest_human(self.position));

        if let Some(target) = target {
            if pickup_distance(self.position, target.position, behavior) {
                self.mode = LanderMode::Carrying {
                    human_id: target.id,
                    pull_sound_sent: false,
                };
                commands.push(GameCommand::AttachHuman {
                    lander: self.id,
                    human: target.id,
                    position: target.position,
                });
                commands.push(GameCommand::PlaySound(SoundCue::LanderPickup));
                return;
            }
            if self.advance_source_fixed_point_motion() {
                return;
            }
            self.position = step_toward(self.position, target.position, behavior.lander_seek_speed);
            return;
        }

        if let Some(player) = prompt.player_position() {
            self.drift = if player.x < self.position.x { -1 } else { 1 };
        }
        if !self.advance_source_fixed_point_motion() {
            self.drift(behavior);
        }
    }

    fn source_target_human<'a>(&self, prompt: &'a StepPrompt) -> Option<&'a ActorSnapshot> {
        self.source
            .and_then(|source| source.target_human_index)
            .and_then(|target_slot_index| prompt.source_target_human(target_slot_index))
    }

    fn drift(&mut self, behavior: ActorBehaviorProfile) {
        self.position = self.position.offset(Velocity::new(
            self.drift * behavior.lander_drift_speed.max(0),
            0,
        ));
    }

    fn advance_source_fixed_point_motion(&mut self) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };
        if source.x_velocity == 0 && source.y_velocity == 0 {
            return false;
        }

        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) =
            actor_source_axis_step(self.position.y, source.y_fraction, source.y_velocity);
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        self.drift = source_lander_drift_from_velocity(source.x_velocity);
        true
    }

    fn update_carrying(
        &mut self,
        human_id: ActorId,
        pull_sound_sent: bool,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        self.position = self
            .position
            .offset(Velocity::new(0, -behavior.lander_carry_speed));
        if !pull_sound_sent {
            self.mode = LanderMode::Carrying {
                human_id,
                pull_sound_sent: true,
            };
            commands.push(GameCommand::PlaySound(SoundCue::HumanPulled));
        }
        if self.position.y <= behavior.lander_conversion_y {
            commands.push(GameCommand::Destroy(self.id));
            commands.push(GameCommand::Destroy(human_id));
            commands.push(GameCommand::Spawn(SpawnRequest::Mutant {
                position: self.position,
            }));
            commands.push(GameCommand::PlaySound(SoundCue::MutantSpawn));
        }
    }

    fn tick_source_sleep(&mut self) -> bool {
        if let Some(source) = &mut self.source
            && source.sleep_ticks > 0
        {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return true;
        }
        false
    }

    fn tick_fire_timer(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        let mut source_fired = false;
        if let Some(source) = &mut self.source {
            if source.shot_timer > 0 {
                source.shot_timer = source.shot_timer.saturating_sub(1);
            }
            if source.shot_timer == 0 {
                source.shot_timer = clamped_source_lander_shot_reset(behavior);
                source_fired = true;
            }
        }
        if source_fired {
            self.fire_lander_shot(prompt, behavior, commands);
            return;
        }
        if self.source.is_some() {
            return;
        }

        let fire_period = behavior.lander_fire_period_steps.max(1);
        if prompt.step % fire_period == self.id.value() % fire_period {
            self.fire_lander_shot(prompt, behavior, commands);
        }
    }

    fn fire_lander_shot(
        &self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
            position: self.position,
            velocity: self.lander_shot_velocity(prompt, behavior),
        }));
        commands.push(GameCommand::PlaySound(SoundCue::Laser));
    }

    fn lander_shot_velocity(
        &self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> Velocity {
        let speed = behavior.lander_shot_speed.max(1);
        if let Some(player) = prompt.player_position() {
            return Velocity::new(
                axis_step(player.x - self.position.x, speed),
                axis_step(player.y - self.position.y, speed),
            );
        }

        Velocity::new(self.drift.signum() * speed, 0)
    }

    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceLanderFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

const fn source_lander_drift_from_velocity(x_velocity: u16) -> i16 {
    actor_source_drift_from_velocity(x_velocity)
}

const fn actor_source_drift_from_velocity(x_velocity: u16) -> i16 {
    if x_velocity & 0x8000 != 0 {
        -1
    } else if x_velocity == 0 {
        0
    } else {
        1
    }
}

fn clamped_source_lander_shot_reset(behavior: ActorBehaviorProfile) -> u8 {
    let clamped = behavior
        .lander_fire_period_steps
        .max(1)
        .min(u64::from(u8::MAX));
    u8::try_from(clamped).unwrap_or(u8::MAX)
}

fn actor_source_axis_step(position: i16, fraction: u8, velocity: u16) -> (i16, u8) {
    let [position, fraction] = u16::from_be_bytes([position as u8, fraction])
        .wrapping_add(velocity)
        .to_be_bytes();
    (i16::from(position), fraction)
}

fn pickup_distance(lander: Point, human: Point, behavior: ActorBehaviorProfile) -> bool {
    (lander.x - human.x).abs() <= behavior.lander_pickup_radius_x
        && (lander.y - human.y).abs() <= behavior.lander_pickup_radius_y
}

fn step_toward(position: Point, target: Point, speed: i16) -> Point {
    Point::new(
        position.x + axis_step(target.x - position.x, speed),
        position.y + axis_step(target.y - position.y, speed),
    )
}

fn axis_step(delta: i16, speed: i16) -> i16 {
    let speed = speed.max(0);
    if delta == 0 {
        0
    } else if delta > 0 {
        delta.min(speed)
    } else {
        delta.max(-speed)
    }
}

fn move_by_hostile_mode(
    position: Point,
    mode: HostileMovementMode,
    prompt: &StepPrompt,
    speed: i16,
    drift: i16,
) -> Option<Point> {
    match mode {
        HostileMovementMode::Drift => Some(position.offset(Velocity::new(drift * speed.max(0), 0))),
        HostileMovementMode::ChasePlayer => prompt
            .player_position()
            .map(|player| step_toward(position, player, speed)),
    }
}

#[derive(Debug)]
struct Mutant {
    id: ActorId,
    position: Point,
    drift: i16,
}

impl Mutant {
    fn new(id: ActorId, position: Point) -> Self {
        Self {
            id,
            position,
            drift: -1,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 14, 12)
    }
}

impl AssetActor for Mutant {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut draws = Vec::new();
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Mutant);
            if let Some(position) = move_by_hostile_mode(
                self.position,
                behavior.mutant_mode,
                prompt,
                behavior.mutant_seek_speed,
                self.drift,
            ) {
                self.position = position;
            }
            draws.push(DrawCommand::sprite(
                self.id,
                SpriteKey::Mutant,
                self.position,
            ));
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Mutant,
                position: self.position,
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands: Vec::new(),
            draws,
        }
    }
}

#[derive(Debug)]
struct Bomber {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourceBomberMetadata>,
}

impl Bomber {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorBomberSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorBomberSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .source
                .map(|source| actor_source_drift_from_velocity(source.x_velocity))
                .unwrap_or(-1),
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 8, 8)
    }

    fn advance_source_motion(&mut self) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };
        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return true;
        }

        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) =
            actor_source_axis_step(self.position.y, source.y_fraction, source.y_velocity);
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        source.picture_frame =
            source.picture_frame.saturating_add(1) % SOURCE_BOMBER_PICTURE_FRAME_COUNT;
        source.sleep_ticks = SOURCE_BOMBER_LOOP_SLEEP_TICKS;
        self.drift = actor_source_drift_from_velocity(source.x_velocity);
        true
    }

    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceBomberFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }

    fn maybe_spawn_bomb(
        &self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        let active_bombs = prompt
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomb && snapshot.alive)
            .count();
        if active_bombs >= SOURCE_ACTIVE_BOMBER_BOMB_LIMIT {
            return;
        }

        let bomb_period = behavior.bomber_bomb_period_steps.max(1);
        let phase = self
            .source
            .map(|source| u64::from(source.source_slot))
            .unwrap_or_else(|| self.id.value());
        if prompt.step % bomb_period == phase % bomb_period {
            commands.push(GameCommand::Spawn(SpawnRequest::Bomb {
                position: self.position,
            }));
        }
    }
}

impl AssetActor for Bomber {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Bomber);
            if !self.advance_source_motion()
                && let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.bomber_mode,
                    prompt,
                    behavior.bomber_drift_speed,
                    self.drift,
                )
            {
                self.position = position;
            }
            self.maybe_spawn_bomb(prompt, behavior, &mut commands);
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Bomber,
                self.position,
                self.draw_effect(),
            ));
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Bomber,
                position: self.position,
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: self.source,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct Bomb {
    id: ActorId,
    position: Point,
    lifetime_steps: u16,
}

impl Bomb {
    fn new(id: ActorId, position: Point, lifetime_steps: u16) -> Self {
        Self {
            id,
            position,
            lifetime_steps,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 4, 6)
    }
}

impl AssetActor for Bomb {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut draws = Vec::new();
        if prompt.phase == Phase::Playing && self.lifetime_steps > 0 {
            self.lifetime_steps = self.lifetime_steps.saturating_sub(1);
            draws.push(DrawCommand::sprite(self.id, SpriteKey::Bomb, self.position));
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Bomb,
                position: self.position,
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing && self.lifetime_steps > 0,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands: Vec::new(),
            draws,
        }
    }
}

#[derive(Debug)]
struct Pod {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourcePodMetadata>,
}

impl Pod {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorPodSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorPodSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .source
                .map(|source| actor_source_drift_from_velocity(source.x_velocity))
                .unwrap_or(1),
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 8, 8)
    }

    fn advance_source_motion(&mut self) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };
        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) =
            actor_source_axis_step(self.position.y, source.y_fraction, source.y_velocity);
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        self.drift = actor_source_drift_from_velocity(source.x_velocity);
        true
    }
}

impl AssetActor for Pod {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut draws = Vec::new();
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Pod);
            if !self.advance_source_motion()
                && let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.pod_mode,
                    prompt,
                    behavior.pod_drift_speed,
                    self.drift,
                )
            {
                self.position = position;
            }
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Pod,
                self.position,
                VisualEffect::SourcePod,
            ));
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Pod,
                position: self.position,
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: self.source,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands: Vec::new(),
            draws,
        }
    }
}

#[derive(Debug)]
struct Swarmer {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourceSwarmerMetadata>,
}

impl Swarmer {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorSwarmerSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorSwarmerSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: -1,
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 6, 4)
    }

    fn advance_source_motion(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };
        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return true;
        }

        if let Some(player) = prompt.player_position() {
            source.x_velocity =
                swarmer_seek_velocity(source.x_velocity, player.x - self.position.x);
            source.y_velocity = swarmer_accelerated_velocity(
                source.y_velocity,
                source.acceleration,
                player.y - self.position.y,
            );
        }
        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) =
            actor_source_axis_step(self.position.y, source.y_fraction, source.y_velocity);
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        source.shot_timer = source.shot_timer.saturating_sub(1);
        if source.shot_timer == 0 {
            let profile = ActorSourceWaveProfile::for_wave(prompt.wave.max(1));
            source.shot_timer = clamped_source_swarmer_shot_reset(profile);
            push_swarmer_shot(self.position, prompt, behavior, commands);
        }
        source.sleep_ticks = SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS;
        true
    }
}

impl AssetActor for Swarmer {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Swarmer);
            if !self.advance_source_motion(prompt, behavior, &mut commands) {
                if let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.swarmer_mode,
                    prompt,
                    behavior.swarmer_seek_speed,
                    self.drift,
                ) {
                    self.position = position;
                }
                let fire_period = behavior.swarmer_fire_period_steps.max(1);
                let can_fire = behavior.swarmer_mode == HostileMovementMode::Drift
                    || prompt.player_position().is_some();
                if can_fire && prompt.step % fire_period == self.id.value() % fire_period {
                    push_swarmer_shot(self.position, prompt, behavior, &mut commands);
                }
            }
            draws.push(DrawCommand::sprite(
                self.id,
                SpriteKey::Swarmer,
                self.position,
            ));
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Swarmer,
                position: self.position,
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: self.source,
                source_baiter: None,
                source_human: None,
            },
            commands,
            draws,
        }
    }
}

fn push_swarmer_shot(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    commands: &mut Vec<GameCommand>,
) {
    commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
        position,
        velocity: hostile_shot_velocity(position, prompt, behavior.swarmer_shot_speed),
    }));
    commands.push(GameCommand::PlaySound(SoundCue::SwarmerShot));
}

fn clamped_source_swarmer_shot_reset(profile: ActorSourceWaveProfile) -> u8 {
    profile.swarmer_shot_time.max(1).min(u32::from(u8::MAX)) as u8
}

#[derive(Debug)]
struct Baiter {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourceBaiterMetadata>,
}

impl Baiter {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorBaiterSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorBaiterSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: -1,
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 12, 4)
    }

    fn advance_source_motion(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };

        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
        } else {
            source.shot_timer = source.shot_timer.wrapping_sub(1);
            if source.shot_timer == 0 {
                let profile = ActorSourceWaveProfile::for_wave(prompt.wave.max(1));
                source.shot_timer = clamped_source_baiter_shot_reset(profile);
                push_baiter_shot(self.position, prompt, behavior, commands);
            }

            source.picture_frame = (source.picture_frame + 1) % SOURCE_BAITER_PICTURE_FRAME_COUNT;
            if source.picture_frame == 0
                && let Some(player) = prompt.player_position()
            {
                let profile = ActorSourceWaveProfile::for_wave(prompt.wave.max(1));
                source_baiter_velocity_update(
                    source,
                    self.position,
                    profile,
                    player,
                    true,
                    actor_source_motion_seed(prompt.step, self.id),
                );
            }
            source.sleep_ticks = SOURCE_BAITER_LOOP_SLEEP_TICKS;
        }

        let (x, x_fraction) = actor_source_axis_step(
            self.position.x,
            source.x_fraction,
            actor_source_baiter_screen_x_velocity(source.x_velocity),
        );
        let (y, y_fraction) =
            actor_source_axis_step(self.position.y, source.y_fraction, source.y_velocity);
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        true
    }

    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceBaiterFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

fn push_baiter_shot(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    commands: &mut Vec<GameCommand>,
) {
    commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
        position,
        velocity: baiter_shot_velocity(position, prompt, behavior),
    }));
    commands.push(GameCommand::PlaySound(SoundCue::BaiterShot));
}

fn baiter_shot_velocity(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
) -> Velocity {
    hostile_shot_velocity(position, prompt, behavior.baiter_shot_speed)
}

fn hostile_shot_velocity(position: Point, prompt: &StepPrompt, speed: i16) -> Velocity {
    let speed = speed.max(1);
    if let Some(player) = prompt.player_position() {
        return Velocity::new(
            axis_step(player.x - position.x, speed),
            axis_step(player.y - position.y, speed),
        );
    }

    Velocity::new(speed, 0)
}

impl AssetActor for Baiter {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Baiter);
            if !self.advance_source_motion(prompt, behavior, &mut commands) {
                if let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.baiter_mode,
                    prompt,
                    behavior.baiter_seek_speed,
                    self.drift,
                ) {
                    self.position = position;
                }
                let fire_period = behavior.baiter_fire_period_steps.max(1);
                let can_fire = behavior.baiter_mode == HostileMovementMode::Drift
                    || prompt.player_position().is_some();
                if can_fire && prompt.step % fire_period == self.id.value() % fire_period {
                    push_baiter_shot(self.position, prompt, behavior, &mut commands);
                }
            }
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Baiter,
                self.position,
                self.draw_effect(),
            ));
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Baiter,
                position: self.position,
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: self.source,
                source_human: None,
            },
            commands,
            draws,
        }
    }
}

fn clamped_source_baiter_shot_reset(profile: ActorSourceWaveProfile) -> u8 {
    profile.baiter_shot_time.max(1).min(u32::from(u8::MAX)) as u8
}

fn actor_source_baiter_screen_x_velocity(source_x_velocity: u16) -> u16 {
    source_x_velocity.wrapping_shl(2)
}

fn source_baiter_velocity_update(
    source: &mut ActorSourceBaiterMetadata,
    position: Point,
    profile: ActorSourceWaveProfile,
    player_position: Point,
    honor_seek_probability: bool,
    seed: u8,
) -> bool {
    if honor_seek_probability && seed <= profile.baiter_seek_probability {
        return false;
    }

    let x_delta = position.x - player_position.x;
    if x_delta.abs() > SOURCE_BAITER_X_SEEK_WINDOW_HALF_PIXELS {
        let x_seek_byte = if x_delta > 0 {
            0u8.wrapping_sub(SOURCE_BAITER_X_SEEK_SPEED)
        } else {
            SOURCE_BAITER_X_SEEK_SPEED
        };
        source.x_velocity = actor_sign_extend_u8_to_u16(x_seek_byte);
    }

    let y_delta = position.y - player_position.y;
    if y_delta.abs() > SOURCE_BAITER_Y_SEEK_WINDOW_HALF_PIXELS {
        let y_seek_byte = if y_delta > 0 {
            0u8.wrapping_sub(SOURCE_BAITER_Y_SEEK_BYTE)
        } else {
            SOURCE_BAITER_Y_SEEK_BYTE
        };
        source.y_velocity =
            actor_arithmetic_shift_right_word(u16::from_be_bytes([y_seek_byte, 0]), 1);
    }

    true
}

fn actor_arithmetic_shift_right_word(value: u16, shift: u8) -> u16 {
    ((value as i16) >> shift.min(15)) as u16
}

fn actor_source_motion_seed(step: u64, id: ActorId) -> u8 {
    (step as u8).wrapping_mul(17).wrapping_add(id.value() as u8)
}

fn swarmer_seek_velocity(current_velocity: u16, delta: i16) -> u16 {
    if delta == 0 {
        current_velocity
    } else if delta > 0 {
        actor_sign_extend_u8_to_u16(0x20)
    } else {
        actor_sign_extend_u8_to_u16(0u8.wrapping_sub(0x20))
    }
}

fn swarmer_accelerated_velocity(current_velocity: u16, acceleration: u8, delta: i16) -> u16 {
    if delta == 0 || acceleration == 0 {
        return current_velocity;
    }
    let adjustment = if delta > 0 {
        actor_sign_extend_u8_to_u16(acceleration.max(1))
    } else {
        actor_sign_extend_u8_to_u16(0u8.wrapping_sub(acceleration.max(1)))
    };
    current_velocity.wrapping_add(adjustment)
}

#[derive(Debug)]
struct Human {
    id: ActorId,
    position: Point,
    mode: HumanMode,
    safe_landing_awarded: bool,
    source_walk_sleep_ticks: u8,
    source: Option<ActorSourceHumanMetadata>,
}

impl Human {
    fn new(id: ActorId, position: Point, mode: HumanMode) -> Self {
        Self::with_source(id, position, mode, None)
    }

    fn from_spawn(id: ActorId, spawn: ActorHumanSpawn) -> Self {
        Self::with_source(id, spawn.position, spawn.mode, spawn.source)
    }

    fn with_source(
        id: ActorId,
        position: Point,
        mode: HumanMode,
        source: Option<ActorSourceHumanMetadata>,
    ) -> Self {
        Self {
            id,
            position,
            mode,
            safe_landing_awarded: false,
            source_walk_sleep_ticks: if source.is_some() {
                SOURCE_HUMAN_WALK_SLEEP_TICKS
            } else {
                0
            },
            source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 4, 8)
    }

    fn update_grounded(&mut self) {
        if self.source.is_none() {
            return;
        }
        if self.source_walk_sleep_ticks > 0 {
            self.source_walk_sleep_ticks = self.source_walk_sleep_ticks.saturating_sub(1);
            return;
        }

        self.advance_source_walk();
        self.source_walk_sleep_ticks = SOURCE_HUMAN_WALK_SLEEP_TICKS;
    }

    fn advance_source_walk(&mut self) {
        if let Some(source) = &mut self.source {
            let frame = source.picture_frame % 4;
            let (next_frame, velocity) = if frame <= 1 {
                (1 - frame, SOURCE_HUMAN_LEFT_X_VELOCITY)
            } else if frame == 2 {
                (3, SOURCE_HUMAN_RIGHT_X_VELOCITY)
            } else {
                (2, SOURCE_HUMAN_RIGHT_X_VELOCITY)
            };
            let (x, x_fraction) =
                actor_source_axis_step(self.position.x, source.x_fraction, velocity);
            self.position.x = x;
            source.x_fraction = x_fraction;
            source.picture_frame = next_frame;
        }
    }

    fn update_falling(
        &mut self,
        velocity: i16,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> Vec<GameCommand> {
        let mut commands = Vec::new();
        let next_velocity =
            (velocity + behavior.human_fall_acceleration).min(behavior.human_max_fall_speed);
        self.position = self.position.offset(Velocity::new(0, next_velocity));

        if prompt.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Player && intersects_snapshot(snapshot, self.bounds())
        }) {
            commands.push(GameCommand::Destroy(self.id));
            commands.push(GameCommand::AddScore(HUMAN_RESCUE_SCORE));
            commands.push(GameCommand::Spawn(SpawnRequest::ScorePopup {
                position: self.position,
                points: HUMAN_RESCUE_SCORE,
            }));
            commands.push(GameCommand::PlaySound(SoundCue::HumanRescued));
            return commands;
        }

        if self.position.y >= behavior.human_ground_y {
            self.position.y = behavior.human_ground_y;
            if next_velocity <= behavior.human_safe_landing_speed {
                self.mode = HumanMode::Grounded;
                if !self.safe_landing_awarded {
                    self.safe_landing_awarded = true;
                    commands.push(GameCommand::AddScore(HUMAN_SAFE_LANDING_SCORE));
                    commands.push(GameCommand::Spawn(SpawnRequest::ScorePopup {
                        position: self.position,
                        points: HUMAN_SAFE_LANDING_SCORE,
                    }));
                    commands.push(GameCommand::PlaySound(SoundCue::HumanSafeLanding));
                }
            } else {
                commands.push(GameCommand::Destroy(self.id));
                commands.push(GameCommand::HumanLost(self.id));
                commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                    position: self.position,
                    kind: ExplosionKind::Human,
                }));
                commands.push(GameCommand::PlaySound(SoundCue::HumanLost));
            }
        } else {
            self.mode = HumanMode::Falling {
                velocity: next_velocity,
            };
        }

        commands
    }

    fn update_carried(
        &mut self,
        carrier: ActorId,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> Vec<GameCommand> {
        let mut commands = Vec::new();
        if let Some(carrier_snapshot) = prompt.snapshot(carrier) {
            self.position = carrier_snapshot
                .position
                .offset(Velocity::new(0, behavior.human_carried_offset_y));
        } else {
            self.mode = HumanMode::Falling { velocity: 0 };
            commands.push(GameCommand::PlaySound(SoundCue::HumanReleased));
        }
        commands
    }
}

impl AssetActor for Human {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Human);
            commands.extend(match self.mode {
                HumanMode::Grounded => {
                    self.update_grounded();
                    Vec::new()
                }
                HumanMode::Falling { velocity } => self.update_falling(velocity, prompt, behavior),
                HumanMode::CarriedBy(carrier) => self.update_carried(carrier, prompt, behavior),
            });
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                self.mode.sprite(),
                self.position,
                self.draw_effect(),
            ));
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Human,
                position: self.position,
                bounds: human_collision_bounds(self.mode, self.position),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: self.source,
            },
            commands,
            draws,
        }
    }
}

impl Human {
    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceHumanFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

fn intersects_snapshot(snapshot: &ActorSnapshot, bounds: Rect) -> bool {
    snapshot
        .bounds
        .is_some_and(|snapshot_bounds| snapshot_bounds.intersects(bounds))
}

fn human_collision_bounds(mode: HumanMode, position: Point) -> Option<Rect> {
    match mode {
        HumanMode::CarriedBy(_) => None,
        HumanMode::Grounded | HumanMode::Falling { .. } => Some(Rect::from_center(position, 4, 8)),
    }
}

#[derive(Debug)]
struct ScorePopup {
    id: ActorId,
    position: Point,
    points: u32,
    age: u16,
}

impl ScorePopup {
    fn new(id: ActorId, position: Point, points: u32) -> Self {
        Self {
            id,
            position,
            points,
            age: 0,
        }
    }
}

impl AssetActor for ScorePopup {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let behavior = prompt.behavior_for(self.id, ActorKind::ScorePopup);
        if self.age < behavior.score_popup_lifetime_steps {
            let sprite = match self.points {
                HUMAN_RESCUE_SCORE => SpriteKey::Score500,
                HUMAN_SAFE_LANDING_SCORE => SpriteKey::Score250,
                _ => SpriteKey::Text,
            };
            draws.push(DrawCommand::sprite(self.id, sprite, self.position));
            self.age = self.age.saturating_add(1);
        }
        if self.age >= behavior.score_popup_lifetime_steps {
            commands.push(GameCommand::Destroy(self.id));
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::ScorePopup,
                position: self.position,
                bounds: None,
                alive: self.age < behavior.score_popup_lifetime_steps,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct LaserShot {
    id: ActorId,
    position: Point,
    direction: Direction,
    age: u16,
}

impl LaserShot {
    fn new(id: ActorId, position: Point, direction: Direction, _owner: ActorId) -> Self {
        Self {
            id,
            position,
            direction,
            age: 0,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 10, 2)
    }
}

impl AssetActor for LaserShot {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let behavior = prompt.behavior_for(self.id, ActorKind::Laser);
        if prompt.phase == Phase::Playing && self.age < behavior.laser_lifetime_steps {
            self.position = self.position.offset(Velocity::new(
                self.direction.sign() * behavior.laser_speed,
                0,
            ));
            self.age = self.age.saturating_add(1);
            draws.push(DrawCommand::sprite(
                self.id,
                SpriteKey::Laser,
                self.position,
            ));
        }
        if self.age >= behavior.laser_lifetime_steps || self.position.x < 0 || self.position.x > 255
        {
            commands.push(GameCommand::Destroy(self.id));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Laser,
                position: self.position,
                bounds: Some(self.bounds()),
                alive: self.age < behavior.laser_lifetime_steps,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct EnemyLaserShot {
    id: ActorId,
    position: Point,
    velocity: Velocity,
    age: u16,
}

impl EnemyLaserShot {
    fn new(id: ActorId, position: Point, velocity: Velocity) -> Self {
        Self {
            id,
            position,
            velocity,
            age: 0,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 4, 4)
    }
}

impl AssetActor for EnemyLaserShot {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let behavior = prompt.behavior_for(self.id, ActorKind::EnemyLaser);
        if prompt.phase == Phase::Playing && self.age < behavior.lander_shot_lifetime_steps {
            self.position = self.position.offset(self.velocity);
            self.age = self.age.saturating_add(1);
            draws.push(DrawCommand::sprite(
                self.id,
                SpriteKey::EnemyLaser,
                self.position,
            ));
        }
        if self.age >= behavior.lander_shot_lifetime_steps
            || self.position.x < 0
            || self.position.x > 255
            || self.position.y < 0
            || self.position.y > 255
        {
            commands.push(GameCommand::Destroy(self.id));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::EnemyLaser,
                position: self.position,
                bounds: Some(self.bounds()),
                alive: self.age < behavior.lander_shot_lifetime_steps,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct Explosion {
    id: ActorId,
    position: Point,
    kind: ExplosionKind,
    age: u16,
}

impl Explosion {
    fn new(id: ActorId, position: Point, kind: ExplosionKind) -> Self {
        Self {
            id,
            position,
            kind,
            age: 0,
        }
    }
}

impl AssetActor for Explosion {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let behavior = prompt.behavior_for(self.id, ActorKind::Explosion);
        if self.age < behavior.explosion_lifetime_steps {
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Explosion,
                self.position,
                VisualEffect::ExplosionCloud {
                    kind: self.kind,
                    age: self.age,
                },
            ));
            self.age = self.age.saturating_add(1);
        }
        if self.age >= behavior.explosion_lifetime_steps {
            commands.push(GameCommand::Destroy(self.id));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Explosion,
                position: self.position,
                bounds: None,
                alive: self.age < behavior.explosion_lifetime_steps,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_human: None,
            },
            commands,
            draws,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attract_actor_accepts_credit_and_start_commands() {
        let mut driver = ActorGameDriver::new();

        let credited = driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.phase, Phase::Attract);
        assert_eq!(credited.credits, 1);
        assert!(credited.sounds.contains(&SoundCue::Credit));
        assert!(
            credited
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::WilliamsLogo
                    && matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
        );

        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.phase, Phase::Playing);
        assert_eq!(started.credits, 0);
        assert!(started.sounds.contains(&SoundCue::Start));
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        let settled = driver.step(GameInput::NONE);
        assert_eq!(settled.phase, Phase::Playing);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert_eq!(driver.snapshot_count(ActorKind::Human), 10);
    }

    #[test]
    fn attract_title_uses_williams_animation_and_defender_coalescence() {
        let mut driver = ActorGameDriver::new();

        let williams = driver.step(GameInput::NONE);
        assert!(williams.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::WilliamsLogo
                && matches!(
                    draw.effect,
                    VisualEffect::WilliamsReveal {
                        stroke_step: 1,
                        color_phase: 0,
                    }
                )
        }));

        let mut coalescing = None;
        for _ in 0..DEFENDER_WORDMARK_START_STEP {
            let step = driver.step(GameInput::NONE);
            if step
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderCoalescence)
            {
                coalescing = Some(step);
                break;
            }
        }
        let coalescing = coalescing.expect("wordmark should enter coalescence");
        assert!(coalescing.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::DefenderCoalescence
                && matches!(
                    draw.effect,
                    VisualEffect::DefenderCoalescence {
                        slot: 0,
                        row_pair: 0,
                    }
                )
        }));

        let mut settled = None;
        for _ in 0..(DEFENDER_WORDMARK_SLOTS * DEFENDER_WORDMARK_ROW_PAIRS + 1) {
            let step = driver.step(GameInput::NONE);
            if step
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderWordmark)
            {
                settled = Some(step);
                break;
            }
        }
        assert!(settled.is_some());
    }

    #[test]
    fn custom_driver_can_script_its_own_attract_screen() {
        let script = AttractScript::new(vec![
            AttractScriptEvent::text(2, Some(5), Point::new(12, 20), "CUSTOM ATTRACT"),
            AttractScriptEvent::sprite(3, None, SpriteKey::DefenderLogo, Point::new(40, 44)),
            AttractScriptEvent::defender_wordmark(4, None, Point::new(70, 80)),
        ]);
        let mut driver = ActorGameDriver::with_attract_script(script);

        let first = driver.step(GameInput::NONE);
        assert!(!first.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("CUSTOM ATTRACT")
                || draw.sprite == SpriteKey::DefenderLogo
                || draw.sprite == SpriteKey::DefenderCoalescence
        }));

        let second = driver.step(GameInput::NONE);
        assert!(
            second
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("CUSTOM ATTRACT"))
        );
        assert!(
            !second
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderLogo)
        );

        let third = driver.step(GameInput::NONE);
        assert!(
            third
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderLogo)
        );

        let fourth = driver.step(GameInput::NONE);
        assert!(fourth.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::DefenderCoalescence
                && matches!(
                    draw.effect,
                    VisualEffect::DefenderCoalescence {
                        slot: 0,
                        row_pair: 0
                    }
                )
        }));
    }

    #[test]
    fn custom_attract_script_keeps_coin_and_start_controls() {
        let script = AttractScript::new(vec![AttractScriptEvent::text(
            1,
            None,
            Point::new(10, 10),
            "PRESS START",
        )]);
        let mut driver = ActorGameDriver::with_attract_script(script);

        let credited = driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.credits, 1);

        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.phase, Phase::Playing);
        assert!(started.sounds.contains(&SoundCue::Start));
    }

    #[test]
    fn status_display_actor_draws_play_state_from_prompt() {
        let mut driver = started_driver();
        driver.score = 9_875;
        driver.wave = 7;
        driver.lives = 2;
        driver.smart_bombs = 1;
        driver.credits = 3;

        let report = driver.step(GameInput::NONE);

        assert!(
            report
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::StatusDisplay)
        );
        assert_text(&report, "1UP 009875");
        assert_text(&report, "HIGH 010000");
        assert_text(&report, "WAVE 07");
        assert_text(&report, "LIVES 02");
        assert_text(&report, "BOMBS 01");
        assert_text(&report, "CREDIT 03");
    }

    #[test]
    fn status_display_actor_draws_high_score_entry_state() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.score = 12_000;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));

        let game_over = driver.step(GameInput::NONE);
        assert_eq!(game_over.phase, Phase::HighScoreEntry);

        let entry = driver.step(GameInput::NONE);
        assert_text(&entry, "FINAL SCORE 012000");
        assert_text(&entry, "HIGH SCORES");
        assert_text(&entry, "1. 012000");
        assert_text(&entry, "2. 010000");
        assert_text(&entry, "ENTER INITIALS");
    }

    #[test]
    fn planetoid_mapper_matches_current_live_key_contract() {
        let mut mapper = KeyboardMapper::new(KeyboardProfile::Planetoid);
        let mut step = KeyboardPoll::default();

        for key in [
            KeyboardKey::Character('5'),
            KeyboardKey::Character('6'),
            KeyboardKey::Character('7'),
            KeyboardKey::Enter,
            KeyboardKey::Character('A'),
            KeyboardKey::Character('Z'),
            KeyboardKey::LeftShift,
            KeyboardKey::Character(' '),
            KeyboardKey::Tab,
            KeyboardKey::Character('H'),
            KeyboardKey::Function(2),
            KeyboardKey::Function(3),
            KeyboardKey::Function(4),
            KeyboardKey::Function(5),
        ] {
            mapper.map_event(KeyboardEvent::press(key), &mut step);
        }
        mapper.finish_poll(&mut step);

        assert!(step.input.coin);
        assert!(step.input.coin_two);
        assert!(step.input.coin_three);
        assert!(step.input.start_one);
        assert!(step.input.fire);
        assert!(step.input.altitude_up);
        assert!(step.input.altitude_down);
        assert!(step.input.thrust);
        assert!(step.input.reverse);
        assert!(step.input.smart_bomb);
        assert!(step.input.hyperspace);
        assert!(step.input.service_advance);
        assert!(step.input.high_score_reset);
        assert!(step.input.auto_up_manual_down);
        assert!(step.input.tilt);
    }

    #[test]
    fn cabinet_mapper_keeps_enter_out_of_the_start_binding() {
        let mut mapper = KeyboardMapper::new(KeyboardProfile::Cabinet);
        let mut enter_poll = KeyboardPoll::default();

        mapper.map_event(KeyboardEvent::press(KeyboardKey::Enter), &mut enter_poll);
        mapper.finish_poll(&mut enter_poll);
        assert!(!enter_poll.input.start_one);
        assert!(!enter_poll.input.fire);

        let mut step = KeyboardPoll::default();
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('1')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('2')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::ArrowUp), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::ArrowDown), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('R')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('T')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('F')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('B')), &mut step);
        mapper.finish_poll(&mut step);

        assert!(step.input.start_one);
        assert!(step.input.start_two);
        assert!(step.input.altitude_up);
        assert!(step.input.altitude_down);
        assert!(step.input.reverse);
        assert!(step.input.thrust);
        assert!(step.input.fire);
        assert!(step.input.smart_bomb);
    }

    #[test]
    fn xyzzy_overlay_toggles_auto_fire_invincibility_and_overlay_bomb() {
        let mut mapper = KeyboardMapper::default();
        let mut step = KeyboardPoll::default();

        for character in ['x', 'y', 'z', 'z', 'y', 'f', 'g'] {
            mapper.map_event(
                KeyboardEvent::press(KeyboardKey::Character(character)),
                &mut step,
            );
        }
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Tab), &mut step);
        mapper.finish_poll(&mut step);

        assert!(step.input.xyzzy.active);
        assert!(step.input.xyzzy.auto_fire);
        assert!(step.input.xyzzy.invincible);
        assert!(step.input.xyzzy.overlay_smart_bomb);
        assert!(step.input.fire);

        for character in ['x', 'y', 'z', 'z', 'y'] {
            mapper.map_event(
                KeyboardEvent::press(KeyboardKey::Character(character)),
                &mut KeyboardPoll::default(),
            );
        }
        assert_eq!(mapper.xyzzy_mode(), XyzzyMode::INACTIVE);
    }

    #[test]
    fn player_actor_emits_spawn_and_sound_when_prompted_to_fire() {
        let mut driver = started_driver();

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });

        assert!(fired.sounds.contains(&SoundCue::Laser));
        assert!(fired.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Laser {
                    direction: Direction::Right,
                    ..
                })
            )
        }));

        let next = driver.step(GameInput::NONE);
        assert!(
            next.snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::Laser)
        );
    }

    #[test]
    fn smart_bomb_clears_hostiles_with_explosions_and_score() {
        let mut driver = started_driver();

        let report = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(report.score, LANDER_SCORE * 5);
        assert_eq!(report.smart_bombs, INITIAL_SMART_BOMBS - 1);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Human), 10);
        assert!(report.sounds.contains(&SoundCue::SmartBomb));
        assert!(report.sounds.contains(&SoundCue::Explosion));
        assert!(report.commands.contains(&GameCommand::SmartBomb {
            consume_stock: true,
        }));
    }

    #[test]
    fn smart_bomb_input_without_stock_does_not_clear_hostiles() {
        let mut driver = started_driver();
        driver.smart_bombs = 0;

        let report = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(report.score, 0);
        assert_eq!(report.smart_bombs, 0);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert!(!report.sounds.contains(&SoundCue::SmartBomb));
        assert!(!report.sounds.contains(&SoundCue::Explosion));
        assert!(
            !report
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::SmartBomb { .. }))
        );
    }

    #[test]
    fn xyzzy_overlay_smart_bomb_does_not_consume_stock() {
        let mut driver = started_driver();
        driver.smart_bombs = 0;

        let report = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                overlay_smart_bomb: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });

        assert_eq!(report.score, LANDER_SCORE * 5);
        assert_eq!(report.smart_bombs, 0);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);
        assert!(report.sounds.contains(&SoundCue::SmartBomb));
        assert!(report.commands.contains(&GameCommand::SmartBomb {
            consume_stock: false,
        }));
    }

    #[test]
    fn default_wave_script_uses_source_wave_table_values() {
        let script = ActorWaveScript::default_progression();
        assert_eq!(script.name(), "actor-source-wave-table");
        let first_source = ActorSourceWaveProfile::for_wave(1);
        assert_eq!(first_source.baiter_delay, 192);
        assert_eq!(first_source.baiter_shot_time, 10);
        assert_eq!(first_source.baiter_seek_probability, 200);

        let first = script.profile_for_wave(1);
        let first_lander = first
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Lander);
        assert_eq!(first.lander_spawns.len(), 5);
        assert_eq!(
            first.lander_spawn_points(),
            vec![
                Point::new(0xFB, 0x2C),
                Point::new(0x3F, 0x2C),
                Point::new(0x67, 0x2C),
                Point::new(0x0D, 0x2C),
                Point::new(0x41, 0x2C),
            ]
        );
        assert_eq!(first.human_spawns.len(), 10);
        assert!(first.bomber_spawns.is_empty());
        assert!(first.pod_spawns.is_empty());
        assert_eq!(
            first.human_spawn_points(),
            vec![
                Point::new(0x18, 0xE0),
                Point::new(0x1C, 0xE1),
                Point::new(0x4E, 0xE0),
                Point::new(0x57, 0xE0),
                Point::new(0x9B, 0xE0),
                Point::new(0x9D, 0xE0),
                Point::new(0xCE, 0xE0),
                Point::new(0xD7, 0xE0),
                Point::new(0xD2, 0xE0),
                Point::new(0xE8, 0xE0),
            ]
        );
        assert_eq!(
            first.human_spawns[1].source,
            Some(ActorSourceHumanMetadata {
                x_fraction: 0x81,
                y_fraction: 0x00,
                picture_frame: 3,
                target_slot_index: 1,
            })
        );
        assert_eq!(
            first.lander_spawns[0].source,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0x33,
                y_fraction: 0xE0,
                x_velocity: 0xFFDE,
                y_velocity: 0x0070,
                shot_timer: 0x27,
                sleep_ticks: 0x04,
                picture_frame: 1,
                target_human_index: Some(1),
            })
        );
        assert_eq!(
            first.lander_spawns[3].source,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0x11,
                y_fraction: 0x70,
                x_velocity: 0x0014,
                y_velocity: 0x0070,
                shot_timer: 0x3C,
                sleep_ticks: 0x04,
                picture_frame: 0,
                target_human_index: Some(4),
            })
        );
        assert_eq!(first_lander.lander_seek_speed, 2);
        assert_eq!(first_lander.lander_fire_period_steps, 64);

        let second = script.profile_for_wave(2);
        let second_lander = second
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Lander);
        let second_bomber = second
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Bomber);
        assert_eq!(second.lander_spawns.len(), 3);
        assert!(
            second
                .lander_spawns
                .iter()
                .all(|spawn| spawn.source.is_none())
        );
        assert_eq!(
            second.lander_spawn_points(),
            vec![
                Point::new(0xE4, 0x2A),
                Point::new(148, 96),
                Point::new(236, 66),
            ]
        );
        assert_eq!(second.bomber_spawns.len(), 1);
        assert_eq!(second.bomber_spawn_points(), vec![Point::new(228, 104)]);
        assert_eq!(
            second.bomber_spawns[0].source,
            Some(ActorSourceBomberMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0xFFD8,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                source_slot: 1,
            })
        );
        assert_eq!(second.pod_spawns.len(), 1);
        assert_eq!(second.pod_spawn_points(), vec![Point::new(184, 72)]);
        assert_eq!(
            second.pod_spawns[0].source,
            Some(ActorSourcePodMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0020,
                y_velocity: 0,
            })
        );
        assert_eq!(second_lander.lander_seek_speed, 2);
        assert_eq!(second_lander.lander_fire_period_steps, 48);
        assert_eq!(second_bomber.bomber_drift_speed, 1);

        let fifth = script.profile_for_wave(5);
        let fifth_lander = fifth
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Lander);
        assert_eq!(fifth_lander.lander_seek_speed, 3);
        assert_eq!(fifth_lander.lander_fire_period_steps, 30);
    }

    #[test]
    fn second_source_wave_spawns_bomber_and_pod_actor_families() {
        let mut driver = started_driver();

        let cleared = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });
        assert_eq!(cleared.wave, 2);
        assert!(
            cleared
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 2 })
        );

        let live = driver.step(GameInput::NONE);

        assert_eq!(driver.snapshot_count(ActorKind::Lander), 3);
        assert_eq!(driver.snapshot_count(ActorKind::Bomber), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 1);
        assert!(live.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Bomber
                && snapshot.source_bomber
                    == Some(ActorSourceBomberMetadata {
                        x_fraction: 0xD8,
                        y_fraction: 0,
                        x_velocity: 0xFFD8,
                        y_velocity: 0,
                        picture_frame: 1,
                        cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                        sleep_ticks: SOURCE_BOMBER_LOOP_SLEEP_TICKS,
                        source_slot: 1,
                    })
        }));
        assert!(live.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Pod
                && snapshot.source_pod
                    == Some(ActorSourcePodMetadata {
                        x_fraction: 0x20,
                        y_fraction: 0,
                        x_velocity: 0x0020,
                        y_velocity: 0,
                    })
        }));
        assert!(
            live.draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::Bomber
                    && matches!(draw.effect, VisualEffect::SourceBomberFrame { frame: 1 }))
        );
        assert!(
            live.draws.iter().any(|draw| draw.sprite == SpriteKey::Pod
                && matches!(draw.effect, VisualEffect::SourcePod))
        );
    }

    #[test]
    fn first_wave_landers_publish_source_metadata_and_picture_frames() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let live = driver.step(GameInput::NONE);
        let lander = live
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.kind == ActorKind::Lander && snapshot.position == Point::new(0xFB, 0x2C)
            })
            .expect("source first-wave lander should publish its restore position");

        assert_eq!(
            lander.source_lander,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0x33,
                y_fraction: 0xE0,
                x_velocity: 0xFFDE,
                y_velocity: 0x0070,
                shot_timer: 0x27,
                sleep_ticks: 0x03,
                picture_frame: 1,
                target_human_index: Some(1),
            })
        );
        assert!(live.draws.iter().any(|draw| {
            draw.actor == lander.id
                && draw.sprite == SpriteKey::Lander
                && matches!(draw.effect, VisualEffect::SourceLanderFrame { frame: 1 })
        }));
    }

    #[test]
    fn source_lander_sleep_ticks_delay_first_wave_motion() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let initial = Point::new(0xFB, 0x2C);
        let mut lander_id = None;
        for expected_sleep in [3, 2, 1, 0] {
            let sleeping = driver.step(GameInput::NONE);
            let lander = sleeping
                .snapshots
                .iter()
                .find(|snapshot| {
                    let matches_known_lander = match lander_id {
                        Some(id) => snapshot.id == id,
                        None => snapshot.position == initial,
                    };
                    snapshot.kind == ActorKind::Lander && matches_known_lander
                })
                .expect("sleeping source lander should stay visible");
            lander_id = Some(lander.id);
            assert_eq!(lander.position, initial);
            assert_eq!(
                lander.source_lander.map(|source| source.sleep_ticks),
                Some(expected_sleep)
            );
        }

        let awake = driver.step(GameInput::NONE);
        let lander = snapshot_for(&awake, lander_id.expect("source lander id should be known"));
        assert_eq!(lander.position, Point::new(0xFB, 0x2D));
        assert_eq!(
            lander
                .source_lander
                .map(|source| (source.x_fraction, source.y_fraction)),
            Some((0x11, 0x50))
        );
        assert_eq!(
            lander.source_lander.map(|source| source.sleep_ticks),
            Some(0)
        );
    }

    #[test]
    fn source_lander_shot_timer_controls_first_wave_laser_sound() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let mut first_laser_step = None;
        for live_step in 1..=50 {
            let report = driver.step(GameInput {
                xyzzy: XyzzyMode {
                    active: true,
                    invincible: true,
                    ..XyzzyMode::INACTIVE
                },
                ..GameInput::NONE
            });
            if report.sounds.contains(&SoundCue::Laser) {
                first_laser_step = Some(live_step);
                break;
            }
        }

        assert_eq!(first_laser_step, Some(39));
    }

    #[test]
    fn source_lander_shot_timer_spawns_hostile_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let mut shot_report = None;
        for _ in 1..=50 {
            let report = driver.step(GameInput {
                xyzzy: XyzzyMode {
                    active: true,
                    invincible: true,
                    ..XyzzyMode::INACTIVE
                },
                ..GameInput::NONE
            });
            if report.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::EnemyLaser { .. }))
            }) {
                shot_report = Some(report);
                break;
            }
        }
        let shot_report = shot_report.expect("source lander should spawn a hostile shot");

        assert!(shot_report.sounds.contains(&SoundCue::Laser));
        let settled = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                invincible: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });
        assert!(
            settled
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
        );
        assert!(
            settled
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::EnemyLaser)
        );
    }

    #[test]
    fn enemy_laser_collision_consumes_life_and_respawns_player() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_enemy_laser(Point::new(42, 120), Velocity::new(0, 0));

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::Playing);
        assert_eq!(report.lives, 2);
        assert!(report.sounds.contains(&SoundCue::Explosion));
        assert!(!report.sounds.contains(&SoundCue::GameOver));
        assert!(report.commands.contains(&GameCommand::PlayerKilled));
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        let respawned = driver.step(GameInput::NONE);
        assert_eq!(respawned.phase, Phase::Playing);
        assert_eq!(respawned.lives, 2);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn enemy_laser_collision_on_final_life_enters_game_over() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.spawn_player();
        driver.spawn_enemy_laser(Point::new(42, 120), Velocity::new(0, 0));

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::GameOver);
        assert_eq!(report.lives, 0);
        assert!(report.sounds.contains(&SoundCue::Explosion));
        assert!(report.sounds.contains(&SoundCue::GameOver));
    }

    #[test]
    fn hyperspace_clears_enemy_lasers_without_spending_stock_or_life() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 3;
        driver.smart_bombs = INITIAL_SMART_BOMBS;
        driver.spawn_player();
        driver.spawn_enemy_laser(Point::new(42, 120), Velocity::new(0, 0));

        let report = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert_eq!(report.lives, 3);
        assert_eq!(report.smart_bombs, INITIAL_SMART_BOMBS);
        assert!(report.commands.contains(&GameCommand::Hyperspace));
        assert!(!report.commands.contains(&GameCommand::PlayerKilled));
        assert!(report.sounds.contains(&SoundCue::Hyperspace));
        assert!(!report.sounds.contains(&SoundCue::GameOver));
        assert_eq!(driver.snapshot_count(ActorKind::EnemyLaser), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn hyperspace_leaves_hostiles_and_player_lasers_active() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 3;
        driver.smart_bombs = INITIAL_SMART_BOMBS;
        let player = driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(90, 120));
        driver.spawn_laser(Point::new(10, 40), Direction::Right, player);
        driver.spawn_enemy_laser(Point::new(70, 120), Velocity::new(0, 0));

        let report = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert!(report.commands.contains(&GameCommand::Hyperspace));
        let emitted_smart_bomb = report
            .commands
            .iter()
            .any(|command| matches!(command, GameCommand::SmartBomb { .. }));
        assert!(!emitted_smart_bomb);
        assert_eq!(driver.snapshot_count(ActorKind::EnemyLaser), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Laser), 1);
        assert_eq!(report.score, 0);
        assert_eq!(report.smart_bombs, INITIAL_SMART_BOMBS);
    }

    #[test]
    fn hyperspace_hides_player_until_scripted_rematerialization() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 2,
                player_hyperspace_rematerialize_x: 150,
                player_hyperspace_rematerialize_y: 92,
                ..ActorBehaviorProfile::default()
            },
        );

        let entered = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        let hidden_player = snapshot_for(&entered, player);
        assert_eq!(hidden_player.bounds, None);
        assert!(!entered.draws.iter().any(|draw| draw.actor == player));
        assert!(entered.sounds.contains(&SoundCue::Hyperspace));
        assert!(!entered.sounds.contains(&SoundCue::HyperspaceMaterialize));

        let still_hidden = driver.step(GameInput {
            thrust: true,
            fire: true,
            ..GameInput::NONE
        });

        let hidden_player = snapshot_for(&still_hidden, player);
        assert_eq!(hidden_player.bounds, None);
        assert!(!still_hidden.draws.iter().any(|draw| draw.actor == player));
        assert!(!still_hidden.sounds.contains(&SoundCue::Thrust));
        assert!(
            !still_hidden.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::Laser { .. }))
            })
        );

        let rematerialized = driver.step(GameInput::NONE);
        let player_snapshot = snapshot_for(&rematerialized, player);
        assert_eq!(player_snapshot.position, Point::new(150, 92));
        assert!(player_snapshot.bounds.is_some());
        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert!(rematerialized.draws.iter().any(|draw| {
            draw.actor == player
                && draw.position == Point::new(150, 92)
                && matches!(draw.sprite, SpriteKey::PlayerRight)
        }));
    }

    #[test]
    fn hyperspace_lseed_high_enters_delayed_player_death_path() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_death_delay_steps: 2,
                player_hyperspace_death_lseed: SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD + 1,
                ..ActorBehaviorProfile::default()
            },
        );

        let entered = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        assert!(entered.commands.contains(&GameCommand::Hyperspace));
        assert!(!entered.commands.contains(&GameCommand::PlayerKilled));

        let rematerialized = driver.step(GameInput::NONE);
        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert!(!rematerialized.commands.contains(&GameCommand::PlayerKilled));
        assert_eq!(rematerialized.lives, 3);

        let pending_death = driver.step(GameInput {
            thrust: true,
            fire: true,
            ..GameInput::NONE
        });
        assert!(!pending_death.commands.contains(&GameCommand::PlayerKilled));
        assert!(!pending_death.sounds.contains(&SoundCue::Thrust));
        assert!(
            !pending_death.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::Laser { .. }))
            })
        );

        let destroyed = driver.step(GameInput::NONE);

        assert_eq!(destroyed.phase, Phase::Playing);
        assert_eq!(destroyed.lives, 2);
        assert!(destroyed.commands.contains(&GameCommand::Destroy(player)));
        assert!(destroyed.commands.contains(&GameCommand::PlayerKilled));
        assert!(destroyed.sounds.contains(&SoundCue::Explosion));
        assert!(destroyed.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Explosion {
                    kind: ExplosionKind::Player,
                    ..
                })
            )
        }));
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        driver.step(GameInput::NONE);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn hyperspace_lseed_at_source_threshold_survives() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_death_delay_steps: 1,
                player_hyperspace_death_lseed: SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD,
                ..ActorBehaviorProfile::default()
            },
        );

        driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        let rematerialized = driver.step(GameInput::NONE);
        let settled = driver.step(GameInput::NONE);

        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert!(!rematerialized.commands.contains(&GameCommand::PlayerKilled));
        assert!(!settled.commands.contains(&GameCommand::PlayerKilled));
        assert_eq!(settled.lives, 3);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn hyperspace_source_seed_controls_rematerialization_position_and_direction() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_rematerialize_x: 150,
                player_hyperspace_rematerialize_y: 92,
                player_hyperspace_source_seed: Some(ActorHyperspaceSourceSeed {
                    seed: 0x12,
                    hseed: 0x34,
                    lseed: 0,
                }),
                ..ActorBehaviorProfile::default()
            },
        );

        driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        let rematerialized = driver.step(GameInput::NONE);

        let player_snapshot = snapshot_for(&rematerialized, player);
        assert_eq!(
            player_snapshot.position,
            Point::new(0x70, i16::from((0x34_u8 >> 1) + SOURCE_PLAYFIELD_Y_MIN))
        );
        assert!(rematerialized.draws.iter().any(|draw| {
            draw.actor == player
                && draw.position == player_snapshot.position
                && matches!(draw.sprite, SpriteKey::PlayerLeft)
        }));
        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert!(!rematerialized.commands.contains(&GameCommand::PlayerKilled));
    }

    #[test]
    fn xyzzy_invincibility_keeps_player_alive_on_enemy_laser_contact() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_enemy_laser(Point::new(42, 120), Velocity::new(0, 0));

        let report = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                invincible: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert_ne!(report.lives, 0);
    }

    #[test]
    fn first_wave_humans_publish_source_metadata_and_picture_frames() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let live = driver.step(GameInput::NONE);
        let human = live
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.kind == ActorKind::Human && snapshot.position == Point::new(0x1C, 0xE1)
            })
            .expect("source first-wave human should publish its restore position");

        assert_eq!(
            human.source_human,
            Some(ActorSourceHumanMetadata {
                x_fraction: 0x81,
                y_fraction: 0x00,
                picture_frame: 3,
                target_slot_index: 1,
            })
        );
        assert!(live.draws.iter().any(|draw| {
            draw.actor == human.id
                && draw.sprite == SpriteKey::Human
                && matches!(draw.effect, VisualEffect::SourceHumanFrame { frame: 3 })
        }));
    }

    #[test]
    fn source_human_walk_cadence_advances_fraction_and_picture_frame() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let first_live = driver.step(GameInput::NONE);
        let human = first_live
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot
                    .source_human
                    .is_some_and(|source| source.target_slot_index == 1)
            })
            .expect("source target-slot human should be visible");
        let human_id = human.id;
        assert_eq!(human.position, Point::new(0x1C, 0xE1));
        assert_eq!(
            human
                .source_human
                .map(|source| (source.x_fraction, source.picture_frame)),
            Some((0x81, 3))
        );

        driver.step(GameInput::NONE);
        let walked = driver.step(GameInput::NONE);
        let human = snapshot_for(&walked, human_id);

        assert_eq!(human.position, Point::new(0x1C, 0xE1));
        assert_eq!(
            human
                .source_human
                .map(|source| (source.x_fraction, source.picture_frame)),
            Some((0xA1, 2))
        );
    }

    #[test]
    fn source_lander_prefers_configured_target_human_slot() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Lander,
            ActorBehaviorProfile {
                lander_seek_speed: 4,
                lander_fire_period_steps: u64::MAX,
                ..ActorBehaviorProfile::default()
            },
        );
        let lander_id = driver.spawn_lander_from_spawn(ActorLanderSpawn {
            position: Point::new(100, 100),
            source: Some(ActorSourceLanderMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: u8::MAX,
                sleep_ticks: 0,
                picture_frame: 0,
                target_human_index: Some(7),
            }),
        });
        driver.spawn_human_for_test(Point::new(90, 100));
        driver.spawn_human_from_spawn(ActorHumanSpawn {
            position: Point::new(160, 100),
            mode: HumanMode::Grounded,
            source: Some(ActorSourceHumanMetadata {
                x_fraction: 0,
                y_fraction: 0,
                picture_frame: 0,
                target_slot_index: 7,
            }),
        });

        driver.step(GameInput::NONE);
        let targeted = driver.step(GameInput::NONE);

        assert_eq!(
            snapshot_for(&targeted, lander_id).position,
            Point::new(104, 100)
        );
    }

    #[test]
    fn behavior_script_can_define_level_wide_actor_motion() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 4,
            lander_fire_period_steps: u64::MAX,
            ..ActorBehaviorProfile::default()
        };
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(ActorKind::Lander, lander_behavior);
        let lander_id = driver.spawn_lander_for_test(Point::new(80, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));

        let first = driver.step(GameInput::NONE);
        assert_eq!(snapshot_for(&first, lander_id).position.x, 79);

        let seeking = driver.step(GameInput::NONE);
        assert_eq!(snapshot_for(&seeking, lander_id).position.x, 83);
    }

    #[test]
    fn behavior_script_can_define_bomber_and_pod_motion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 3,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Pod,
            ActorBehaviorProfile {
                pod_drift_speed: 4,
                ..ActorBehaviorProfile::default()
            },
        );
        let bomber = driver.spawn_bomber_for_test(Point::new(100, 80));
        let pod = driver.spawn_pod_for_test(Point::new(100, 88));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, bomber).position, Point::new(97, 80));
        assert_eq!(snapshot_for(&report, pod).position, Point::new(104, 88));
    }

    #[test]
    fn behavior_script_can_choose_mutant_drift_mode() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Mutant,
            ActorBehaviorProfile {
                mutant_seek_speed: 4,
                mutant_mode: HostileMovementMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        let mutant = driver.spawn_mutant(Point::new(100, 100));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, mutant).position, Point::new(96, 100));
    }

    #[test]
    fn behavior_script_can_choose_bomber_and_pod_targeting_modes() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 5,
                bomber_bomb_period_steps: u64::MAX,
                bomber_mode: HostileMovementMode::ChasePlayer,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Pod,
            ActorBehaviorProfile {
                pod_drift_speed: 6,
                pod_mode: HostileMovementMode::ChasePlayer,
                ..ActorBehaviorProfile::default()
            },
        );
        let bomber = driver.spawn_bomber_for_test(Point::new(70, 80));
        let pod = driver.spawn_pod_for_test(Point::new(70, 88));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, bomber).position, Point::new(65, 85));
        assert_eq!(snapshot_for(&report, pod).position, Point::new(64, 94));
    }

    #[test]
    fn bomber_actor_lays_scripted_bomb_actor() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 0,
                bomber_bomb_period_steps: 1,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.spawn_bomber_for_test(Point::new(100, 80));

        let report = driver.step(GameInput::NONE);

        assert!(report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Bomb {
                    position: Point { x: 100, y: 80 },
                })
            )
        }));

        let live = driver.step(GameInput::NONE);
        assert_eq!(driver.snapshot_count(ActorKind::Bomb), 1);
        assert!(live.draws.iter().any(|draw| draw.sprite == SpriteKey::Bomb));
    }

    #[test]
    fn behavior_script_can_define_swarmer_motion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.set_kind_behavior(
            ActorKind::Swarmer,
            ActorBehaviorProfile {
                swarmer_seek_speed: 5,
                ..ActorBehaviorProfile::default()
            },
        );
        let swarmer = driver.spawn_swarmer_for_test(Point::new(70, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, swarmer).position, Point::new(65, 120));
    }

    #[test]
    fn behavior_script_can_define_baiter_motion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.set_kind_behavior(
            ActorKind::Baiter,
            ActorBehaviorProfile {
                baiter_seek_speed: 6,
                baiter_fire_period_steps: u64::MAX,
                ..ActorBehaviorProfile::default()
            },
        );
        let baiter = driver.spawn_baiter_for_test(Point::new(70, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, baiter).position, Point::new(64, 120));
    }

    #[test]
    fn behavior_script_can_choose_swarmer_and_baiter_drift_modes() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Swarmer,
            ActorBehaviorProfile {
                swarmer_seek_speed: 4,
                swarmer_fire_period_steps: u64::MAX,
                swarmer_mode: HostileMovementMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Baiter,
            ActorBehaviorProfile {
                baiter_seek_speed: 5,
                baiter_fire_period_steps: u64::MAX,
                baiter_mode: HostileMovementMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        let swarmer = driver.spawn_swarmer_for_test(Point::new(70, 120));
        let baiter = driver.spawn_baiter_for_test(Point::new(80, 124));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, swarmer).position, Point::new(66, 120));
        assert_eq!(snapshot_for(&report, baiter).position, Point::new(75, 124));
    }

    #[test]
    fn baiter_timer_spawns_source_baiter_from_wave_profile() {
        let mut driver = started_driver();

        driver.set_baiter_timer_for_test(1);
        let report = driver.step(GameInput::NONE);

        let baiter_spawn = report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::Baiter { position, source }) => {
                    Some((*position, *source))
                }
                _ => None,
            })
            .expect("expired baiter timer should spawn a baiter");
        assert_eq!(
            baiter_spawn,
            (
                Point::new(228, 144),
                Some(ActorSourceBaiterMetadata {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0xFFC0,
                    y_velocity: 0xFF80,
                    shot_timer: SOURCE_BAITER_INITIAL_SHOT_TIMER,
                    sleep_ticks: 0,
                    picture_frame: 0,
                })
            )
        );

        let live = driver.step(GameInput::NONE);
        assert_eq!(driver.snapshot_count(ActorKind::Baiter), 1);
        assert!(live.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Baiter
                && snapshot.position == Point::new(227, 143)
                && snapshot.source_baiter
                    == Some(ActorSourceBaiterMetadata {
                        x_fraction: 0,
                        y_fraction: 0x80,
                        x_velocity: 0xFFC0,
                        y_velocity: 0xFF80,
                        shot_timer: 7,
                        sleep_ticks: SOURCE_BAITER_LOOP_SLEEP_TICKS,
                        picture_frame: 1,
                    })
        }));
        assert!(live.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::Baiter
                && matches!(draw.effect, VisualEffect::SourceBaiterFrame { frame: 1 })
        }));
    }

    #[test]
    fn source_swarmer_shot_timer_spawns_hostile_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        let swarmer = driver.spawn_swarmer_from_spawn(ActorSwarmerSpawn {
            position: Point::new(70, 120),
            source: Some(ActorSourceSwarmerMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                acceleration: 0,
                sleep_ticks: 0,
                shot_timer: 1,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert!(report.sounds.contains(&SoundCue::SwarmerShot));
        assert!(report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position: Point { x: 69, y: 120 },
                    velocity: Velocity { dx: -3, dy: 0 },
                })
            )
        }));
        assert_eq!(
            snapshot_for(&report, swarmer).source_swarmer,
            Some(ActorSourceSwarmerMetadata {
                x_fraction: 0xE0,
                y_fraction: 0,
                x_velocity: 0xFFE0,
                y_velocity: 0,
                acceleration: 0,
                sleep_ticks: SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS,
                shot_timer: 20,
            })
        );
    }

    #[test]
    fn source_baiter_shot_timer_spawns_hostile_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
            position: Point::new(70, 120),
            source: Some(ActorSourceBaiterMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: 0,
                picture_frame: 0,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert!(report.sounds.contains(&SoundCue::BaiterShot));
        assert!(report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position: Point { x: 70, y: 120 },
                    velocity: Velocity { dx: -4, dy: 0 },
                })
            )
        }));
        assert_eq!(
            snapshot_for(&report, baiter).source_baiter,
            Some(ActorSourceBaiterMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 10,
                sleep_ticks: SOURCE_BAITER_LOOP_SLEEP_TICKS,
                picture_frame: 1,
            })
        );
    }

    #[test]
    fn baiter_does_not_block_wave_completion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.reset_baiter_timer();
        let baiter = driver.spawn_baiter_for_test(Point::new(100, 100));

        let cleared = driver.step(GameInput::NONE);

        assert!(snapshot_for(&cleared, baiter).alive);
        assert_eq!(cleared.wave, 2);
        assert!(
            cleared
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 2 })
        );
    }

    #[test]
    fn wave_script_applies_behavior_when_play_starts() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 5,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::ChasePlayer,
            ..ActorBehaviorProfile::default()
        };
        let wave_script = ActorWaveScript::single_wave(
            "opening-chasers",
            ActorBehaviorScript::default().with_kind_behavior(ActorKind::Lander, lander_behavior),
            vec![Point::new(80, HUMAN_GROUND_Y)],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.wave, 1);
        assert_eq!(driver.wave(), 1);
        assert_eq!(driver.wave_script_name(), "opening-chasers");

        driver.step(GameInput::NONE);
        let chasing = driver.step(GameInput::NONE);
        let lander = chasing
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Lander)
            .expect("wave script should spawn a lander");
        assert_eq!(lander.position, Point::new(74, HUMAN_GROUND_Y - 5));
    }

    #[test]
    fn wave_script_can_configure_initial_human_spawns() {
        let wave_script = ActorWaveScript::new(
            "single-human-opening",
            vec![ActorWaveProfile::with_spawns(
                1,
                ActorBehaviorScript::default(),
                vec![ActorLanderSpawn::new(Point::new(220, 80))],
                vec![ActorHumanSpawn::new(
                    Point::new(32, HUMAN_GROUND_Y),
                    HumanMode::Grounded,
                )],
            )],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let settled = driver.step(GameInput::NONE);

        assert_eq!(driver.snapshot_count(ActorKind::Human), 1);
        assert!(settled.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Human
                && snapshot.position == Point::new(32, HUMAN_GROUND_Y)
                && snapshot.source_human.is_none()
        }));
    }

    #[test]
    fn cleared_wave_advances_to_next_behavior_script() {
        let wave_two_lander = ActorBehaviorProfile {
            lander_drift_speed: 5,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::Drift,
            ..ActorBehaviorProfile::default()
        };
        let wave_script = ActorWaveScript::new(
            "two-wave-test",
            vec![
                ActorWaveProfile::new(1, ActorBehaviorScript::default(), vec![Point::new(180, 88)]),
                ActorWaveProfile::new(
                    2,
                    ActorBehaviorScript::default()
                        .with_kind_behavior(ActorKind::Lander, wave_two_lander),
                    vec![Point::new(100, 100)],
                ),
            ],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        driver.step(GameInput::NONE);

        let cleared = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });
        assert_eq!(cleared.wave, 2);
        assert!(
            cleared
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 2 })
        );

        let next_wave = driver.step(GameInput::NONE);
        let lander = next_wave
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Lander)
            .expect("next wave should spawn a scripted lander");
        assert_eq!(lander.position, Point::new(95, 100));
    }

    #[test]
    fn behavior_script_can_choose_lander_targeting_mode() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 4,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::ChasePlayer,
            ..ActorBehaviorProfile::default()
        };
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(ActorKind::Lander, lander_behavior);
        driver.spawn_player();
        let lander_id = driver.spawn_lander_for_test(Point::new(80, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);

        let chasing = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&chasing, lander_id).position.x, 75);
    }

    #[test]
    fn actor_specific_behavior_override_changes_one_actor() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let fast_lander = driver.spawn_lander_for_test(Point::new(80, 100));
        let normal_lander = driver.spawn_lander_for_test(Point::new(100, 100));
        let fast_behavior = ActorBehaviorProfile {
            lander_drift_speed: 4,
            lander_fire_period_steps: u64::MAX,
            ..ActorBehaviorProfile::default()
        };
        driver.set_actor_behavior(fast_lander, fast_behavior);

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, fast_lander).position.x, 76);
        assert_eq!(snapshot_for(&report, normal_lander).position.x, 99);
    }

    #[test]
    fn script_can_tune_player_and_laser_behavior() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        let player_behavior = ActorBehaviorProfile {
            player_speed: 5,
            player_laser_cooldown_steps: 1,
            ..ActorBehaviorProfile::default()
        };
        driver.set_actor_behavior(player, player_behavior);

        let moved = driver.step(GameInput {
            thrust: true,
            fire: true,
            ..GameInput::NONE
        });
        assert_eq!(snapshot_for(&moved, player).position.x, 47);

        let laser_behavior = ActorBehaviorProfile {
            laser_speed: 2,
            laser_lifetime_steps: 4,
            ..ActorBehaviorProfile::default()
        };
        driver.set_kind_behavior(ActorKind::Laser, laser_behavior);

        let laser_step = driver.step(GameInput::NONE);
        let laser = laser_step
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Laser)
            .expect("configured laser should stay alive");
        assert_eq!(laser.position.x, 61);
    }

    #[test]
    fn scripted_player_damage_override_matches_xyzzy_invincibility() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));
        let player_behavior = ActorBehaviorProfile {
            player_takes_enemy_collision_damage: false,
            ..ActorBehaviorProfile::default()
        };
        driver.set_kind_behavior(ActorKind::Player, player_behavior);

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::Playing);
        assert_ne!(report.lives, 0);
    }

    #[test]
    fn lander_picks_up_and_carries_a_grounded_human() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_lander_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);

        let pickup = driver.step(GameInput::NONE);
        assert!(pickup.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::AttachHuman {
                    lander: _,
                    human: _,
                    position: Point {
                        x: 100,
                        y: HUMAN_GROUND_Y
                    },
                }
            )
        }));
        assert!(pickup.sounds.contains(&SoundCue::LanderPickup));

        let carried = driver.step(GameInput::NONE);
        assert!(carried.sounds.contains(&SoundCue::HumanPulled));
        assert!(
            carried
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::HumanCarried)
        );
    }

    #[test]
    fn carried_human_falls_when_carrier_disappears() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_carried_human_for_test(Point::new(100, 120), ActorId::new(99));

        let released = driver.step(GameInput::NONE);

        assert!(released.sounds.contains(&SoundCue::HumanReleased));
        assert!(
            released
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::HumanFalling)
        );
    }

    #[test]
    fn falling_human_rescue_awards_500_points_and_score_popup() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_falling_human_for_test(Point::new(42, 120), 0);
        driver.step(GameInput::NONE);

        let rescued = driver.step(GameInput::NONE);

        assert_eq!(rescued.score, HUMAN_RESCUE_SCORE);
        assert!(rescued.sounds.contains(&SoundCue::HumanRescued));
        assert!(
            rescued
                .commands
                .contains(&GameCommand::AddScore(HUMAN_RESCUE_SCORE))
        );
        assert!(rescued.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::ScorePopup {
                    points: HUMAN_RESCUE_SCORE,
                    ..
                })
            )
        }));
    }

    #[test]
    fn slow_falling_human_lands_safely_for_250_points() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_falling_human_for_test(Point::new(100, HUMAN_GROUND_Y - 1), 1);

        let landed = driver.step(GameInput::NONE);

        assert_eq!(landed.score, HUMAN_SAFE_LANDING_SCORE);
        assert!(landed.sounds.contains(&SoundCue::HumanSafeLanding));
        assert!(
            landed
                .commands
                .contains(&GameCommand::AddScore(HUMAN_SAFE_LANDING_SCORE))
        );
        assert!(
            landed
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::Human)
        );
    }

    #[test]
    fn completed_abduction_consumes_human_and_spawns_mutant() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_lander_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);
        driver.step(GameInput::NONE);

        let mut converted = None;
        for _ in 0..120 {
            let step = driver.step(GameInput::NONE);
            if step.sounds.contains(&SoundCue::MutantSpawn) {
                converted = Some(step);
                break;
            }
        }
        let converted = converted.expect("carried human should convert into a mutant");

        assert_eq!(driver.snapshot_count(ActorKind::Human), 0);
        assert!(
            converted
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::Spawn(SpawnRequest::Mutant { .. })))
        );
        let settled = driver.step(GameInput::NONE);
        assert!(
            settled
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::Mutant)
        );
    }

    #[test]
    fn driver_resolves_laser_lander_collision_with_score_sound_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(62, 120));

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        assert!(fired.sounds.contains(&SoundCue::Laser));

        let collision = driver.step(GameInput::NONE);
        assert_eq!(collision.score, 150);
        assert!(collision.sounds.contains(&SoundCue::Explosion));
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);
        assert!(collision.commands.contains(&GameCommand::AddScore(150)));
        assert!(collision.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Explosion {
                    kind: ExplosionKind::Enemy,
                    ..
                })
            )
        }));
    }

    #[test]
    fn driver_resolves_laser_bomber_collision_with_source_score_sound_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_bomber_for_test(Point::new(62, 120));

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        assert!(fired.sounds.contains(&SoundCue::Laser));

        let collision = driver.step(GameInput::NONE);
        assert_eq!(collision.score, BOMBER_SCORE);
        assert!(collision.sounds.contains(&SoundCue::BomberHit));
        assert_eq!(driver.snapshot_count(ActorKind::Bomber), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(BOMBER_SCORE))
        );
    }

    #[test]
    fn driver_resolves_laser_pod_collision_with_source_score_sound_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        driver.spawn_player();
        driver.spawn_pod_for_test(Point::new(62, 120));

        driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let collision = driver.step(GameInput::NONE);

        assert_eq!(collision.score, POD_SCORE);
        assert!(collision.sounds.contains(&SoundCue::PodHit));
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(POD_SCORE))
        );
        let swarmer_spawns = collision
            .commands
            .iter()
            .filter_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::Swarmer { position, source }) => {
                    Some((*position, *source))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(swarmer_spawns.len(), SOURCE_POD_SWARMER_REQUEST_LIMIT);
        assert_eq!(
            swarmer_spawns[0],
            (
                Point::new(64, 120),
                Some(ActorSourceSwarmerMetadata {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0x0028,
                    y_velocity: 0x0020,
                    acceleration: 0,
                    sleep_ticks: 0,
                    shot_timer: 20,
                })
            )
        );

        let live = driver.step(GameInput::NONE);
        assert_eq!(
            driver.snapshot_count(ActorKind::Swarmer),
            SOURCE_POD_SWARMER_REQUEST_LIMIT
        );
        assert!(
            live.draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::Swarmer)
        );
    }

    #[test]
    fn driver_resolves_laser_baiter_collision_with_source_score_sound_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_baiter_for_test(Point::new(62, 120));

        driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let collision = driver.step(GameInput::NONE);

        assert_eq!(collision.score, BAITER_SCORE);
        assert!(collision.sounds.contains(&SoundCue::BaiterHit));
        assert_eq!(driver.snapshot_count(ActorKind::Baiter), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(BAITER_SCORE))
        );
    }

    #[test]
    fn bomb_collision_enters_game_over_with_source_bomb_sound() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.spawn_player();
        driver.spawn_bomb_for_test(Point::new(42, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::GameOver);
        assert!(report.sounds.contains(&SoundCue::BombHit));
        assert!(report.sounds.contains(&SoundCue::GameOver));
        assert!(report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Explosion {
                    kind: ExplosionKind::Bomb,
                    ..
                })
            )
        }));
    }

    #[test]
    fn explosion_actor_draws_variant_metadata() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let explosion = driver.spawn_explosion(Point::new(90, 80), ExplosionKind::Bomb);

        let report = driver.step(GameInput::NONE);

        assert!(report.draws.iter().any(|draw| {
            draw.actor == explosion
                && draw.sprite == SpriteKey::Explosion
                && matches!(
                    draw.effect,
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Bomb,
                        age: 0,
                    }
                )
        }));
    }

    #[test]
    fn smart_bomb_pod_score_does_not_spawn_swarmers() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        driver.smart_bombs = INITIAL_SMART_BOMBS;
        driver.spawn_player();
        driver.spawn_pod_for_test(Point::new(120, 120));

        let report = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(report.score, POD_SCORE);
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 0);
        assert!(!report.commands.iter().any(|command| {
            matches!(command, GameCommand::Spawn(SpawnRequest::Swarmer { .. }))
        }));
    }

    #[test]
    fn high_score_entry_is_a_phase_not_a_legacy_timeline_script() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.score = 12_000;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::HighScoreEntry);
        assert_eq!(report.lives, 0);
        assert!(report.sounds.contains(&SoundCue::GameOver));
    }

    #[test]
    fn xyzzy_invincibility_keeps_player_alive_on_contact() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));

        let report = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                invincible: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert_ne!(report.lives, 0);
    }

    #[test]
    fn threaded_asset_is_prompted_once_per_driver_step() {
        let mut driver = ActorGameDriver::new();
        let first = driver.step(GameInput::NONE);
        let second = driver.step(GameInput::NONE);

        assert_eq!(first.step, 1);
        assert_eq!(second.step, 2);
        assert!(
            second
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::AttractDirector)
        );
    }

    fn started_driver() -> ActorGameDriver {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        driver.step(GameInput::NONE);
        driver
    }

    fn snapshot_for(report: &StepReport, id: ActorId) -> &ActorSnapshot {
        report
            .snapshots
            .iter()
            .find(|snapshot| snapshot.id == id)
            .expect("actor snapshot should be present")
    }

    fn assert_text(report: &StepReport, value: &str) {
        assert!(
            report
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(value)),
            "expected draw text {value:?}"
        );
    }
}
