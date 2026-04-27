//! Cabinet actions and keyboard profiles.
//!
//! MAME's Williams driver maps Defender cabinet inputs as active-high IN0/IN1
//! and IN2 ports read by PIA0 port A, PIA0 port B, and PIA1 port A.
//! Source: <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams.cpp>.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, ModifierKeyCode};

pub const DEFENDER_IN0_FIRE: u8 = 0x01;
pub const DEFENDER_IN0_THRUST: u8 = 0x02;
pub const DEFENDER_IN0_SMART_BOMB: u8 = 0x04;
pub const DEFENDER_IN0_HYPERSPACE: u8 = 0x08;
pub const DEFENDER_IN0_START_TWO: u8 = 0x10;
pub const DEFENDER_IN0_START_ONE: u8 = 0x20;
pub const DEFENDER_IN0_REVERSE: u8 = 0x40;
pub const DEFENDER_IN0_ALTITUDE_DOWN: u8 = 0x80;
pub const DEFENDER_IN1_ALTITUDE_UP: u8 = 0x01;
pub const DEFENDER_IN2_AUTO_UP_MANUAL_DOWN: u8 = 0x01;
pub const DEFENDER_IN2_ADVANCE: u8 = 0x02;
pub const DEFENDER_IN2_COIN_THREE: u8 = 0x04;
pub const DEFENDER_IN2_HIGH_SCORE_RESET: u8 = 0x08;
pub const DEFENDER_IN2_COIN_ONE: u8 = 0x10;
pub const DEFENDER_IN2_COIN_TWO: u8 = 0x20;
pub const DEFENDER_IN2_TILT: u8 = 0x40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DefenderInputPorts {
    pub in0: u8,
    pub in1: u8,
    pub in2: u8,
}

impl DefenderInputPorts {
    pub const EMPTY: Self = Self {
        in0: 0,
        in1: 0,
        in2: 0,
    };

    pub const fn pia0_port_a(self) -> u8 {
        self.in0
    }

    pub const fn pia0_port_b(self) -> u8 {
        self.in1
    }

    pub const fn pia1_port_a(self) -> u8 {
        self.in2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputProfile {
    #[default]
    Planetoid,
    Cabinet,
    Test,
}

impl InputProfile {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "planetoid" => Some(Self::Planetoid),
            "cabinet" => Some(Self::Cabinet),
            "test" => Some(Self::Test),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Planetoid => "planetoid",
            Self::Cabinet => "cabinet",
            Self::Test => "test",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CabinetInput {
    pub coin: bool,
    pub coin_two: bool,
    pub coin_three: bool,
    pub start_one: bool,
    pub start_two: bool,
    pub altitude_up: bool,
    pub altitude_down: bool,
    pub reverse: bool,
    pub thrust: bool,
    pub fire: bool,
    pub smart_bomb: bool,
    pub hyperspace: bool,
    pub auto_up_manual_down: bool,
    pub service_advance: bool,
    pub high_score_reset: bool,
    pub tilt: bool,
}

impl CabinetInput {
    pub const NONE: Self = Self {
        coin: false,
        coin_two: false,
        coin_three: false,
        start_one: false,
        start_two: false,
        altitude_up: false,
        altitude_down: false,
        reverse: false,
        thrust: false,
        fire: false,
        smart_bomb: false,
        hyperspace: false,
        auto_up_manual_down: false,
        service_advance: false,
        high_score_reset: false,
        tilt: false,
    };

    pub fn bits(self) -> u16 {
        u16::from(self.coin)
            | (u16::from(self.start_one) << 1)
            | (u16::from(self.start_two) << 2)
            | (u16::from(self.altitude_up) << 3)
            | (u16::from(self.altitude_down) << 4)
            | (u16::from(self.reverse) << 5)
            | (u16::from(self.thrust) << 6)
            | (u16::from(self.fire) << 7)
            | (u16::from(self.smart_bomb) << 8)
            | (u16::from(self.hyperspace) << 9)
            | (u16::from(self.coin_two) << 10)
            | (u16::from(self.coin_three) << 11)
            | (u16::from(self.auto_up_manual_down) << 12)
            | (u16::from(self.service_advance) << 13)
            | (u16::from(self.high_score_reset) << 14)
            | (u16::from(self.tilt) << 15)
    }

    pub fn defender_input_ports(self) -> DefenderInputPorts {
        let mut ports = DefenderInputPorts::EMPTY;

        if self.fire {
            ports.in0 |= DEFENDER_IN0_FIRE;
        }
        if self.thrust {
            ports.in0 |= DEFENDER_IN0_THRUST;
        }
        if self.smart_bomb {
            ports.in0 |= DEFENDER_IN0_SMART_BOMB;
        }
        if self.hyperspace {
            ports.in0 |= DEFENDER_IN0_HYPERSPACE;
        }
        if self.start_two {
            ports.in0 |= DEFENDER_IN0_START_TWO;
        }
        if self.start_one {
            ports.in0 |= DEFENDER_IN0_START_ONE;
        }
        if self.reverse {
            ports.in0 |= DEFENDER_IN0_REVERSE;
        }
        if self.altitude_down {
            ports.in0 |= DEFENDER_IN0_ALTITUDE_DOWN;
        }

        if self.altitude_up {
            ports.in1 |= DEFENDER_IN1_ALTITUDE_UP;
        }

        if self.auto_up_manual_down {
            ports.in2 |= DEFENDER_IN2_AUTO_UP_MANUAL_DOWN;
        }
        if self.service_advance {
            ports.in2 |= DEFENDER_IN2_ADVANCE;
        }
        if self.coin_three {
            ports.in2 |= DEFENDER_IN2_COIN_THREE;
        }
        if self.high_score_reset {
            ports.in2 |= DEFENDER_IN2_HIGH_SCORE_RESET;
        }
        if self.coin {
            ports.in2 |= DEFENDER_IN2_COIN_ONE;
        }
        if self.coin_two {
            ports.in2 |= DEFENDER_IN2_COIN_TWO;
        }
        if self.tilt {
            ports.in2 |= DEFENDER_IN2_TILT;
        }

        ports
    }

    pub fn merge(&mut self, other: Self) {
        self.coin |= other.coin;
        self.coin_two |= other.coin_two;
        self.coin_three |= other.coin_three;
        self.start_one |= other.start_one;
        self.start_two |= other.start_two;
        self.altitude_up |= other.altitude_up;
        self.altitude_down |= other.altitude_down;
        self.reverse |= other.reverse;
        self.thrust |= other.thrust;
        self.fire |= other.fire;
        self.smart_bomb |= other.smart_bomb;
        self.hyperspace |= other.hyperspace;
        self.auto_up_manual_down |= other.auto_up_manual_down;
        self.service_advance |= other.service_advance;
        self.high_score_reset |= other.high_score_reset;
        self.tilt |= other.tilt;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeldInput {
    pub altitude_up: bool,
    pub altitude_down: bool,
    pub thrust: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PolledInput {
    pub cabinet: CabinetInput,
    pub typed_chars: Vec<char>,
    pub quit_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputMapper {
    profile: InputProfile,
    held: HeldInput,
}

impl InputMapper {
    pub fn new(profile: InputProfile) -> Self {
        Self {
            profile,
            held: HeldInput::default(),
        }
    }

    pub fn profile(&self) -> InputProfile {
        self.profile
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent, input: &mut PolledInput) {
        let pressed = !matches!(key_event.kind, KeyEventKind::Release);
        if matches!(key_event.kind, KeyEventKind::Press)
            && let KeyCode::Char(character) = key_event.code
        {
            input.typed_chars.push(character.to_ascii_lowercase());
        }
        if matches!(key_event.kind, KeyEventKind::Press) && key_event.code == KeyCode::Backspace {
            input.typed_chars.push('\u{8}');
        }

        match key_event.code {
            KeyCode::Esc if pressed => input.quit_requested = true,
            KeyCode::Char('q') | KeyCode::Char('Q') if pressed => input.quit_requested = true,
            _ => self.map_profile_key(key_event, input),
        }
    }

    pub fn apply_held(&self, input: &mut PolledInput) {
        input.cabinet.altitude_up |= self.held.altitude_up;
        input.cabinet.altitude_down |= self.held.altitude_down;
        input.cabinet.thrust |= self.held.thrust;
    }

    fn map_profile_key(&mut self, key_event: KeyEvent, input: &mut PolledInput) {
        match self.profile {
            InputProfile::Planetoid => self.map_planetoid_key(key_event, input),
            InputProfile::Cabinet => self.map_cabinet_key(key_event, input),
            InputProfile::Test => self.map_test_key(key_event, input),
        }
    }

    fn map_planetoid_key(&mut self, key_event: KeyEvent, input: &mut PolledInput) {
        let pressed = !matches!(key_event.kind, KeyEventKind::Release);
        match key_event.code {
            KeyCode::Enter if pressed => {
                input.cabinet.start_one = true;
                input.cabinet.fire = true;
            }
            KeyCode::Char('1') if pressed => input.cabinet.start_one = true,
            KeyCode::Char('5') if pressed => input.cabinet.coin = true,
            KeyCode::Char('a') | KeyCode::Char('A') => set_held_flag(
                &mut self.held.altitude_up,
                key_event.kind,
                &mut input.cabinet.altitude_up,
            ),
            KeyCode::Char('z') | KeyCode::Char('Z') => set_held_flag(
                &mut self.held.altitude_down,
                key_event.kind,
                &mut input.cabinet.altitude_down,
            ),
            KeyCode::Modifier(ModifierKeyCode::LeftShift)
            | KeyCode::Modifier(ModifierKeyCode::RightShift) => set_held_flag(
                &mut self.held.thrust,
                key_event.kind,
                &mut input.cabinet.thrust,
            ),
            KeyCode::Char(' ') if pressed => input.cabinet.reverse = true,
            KeyCode::Tab if pressed => input.cabinet.smart_bomb = true,
            KeyCode::Char('h') | KeyCode::Char('H') if pressed => input.cabinet.hyperspace = true,
            _ => {}
        }
    }

    fn map_cabinet_key(&mut self, key_event: KeyEvent, input: &mut PolledInput) {
        let pressed = !matches!(key_event.kind, KeyEventKind::Release);
        match key_event.code {
            KeyCode::Char('5') if pressed => input.cabinet.coin = true,
            KeyCode::Char('1') if pressed => input.cabinet.start_one = true,
            KeyCode::Char('2') if pressed => input.cabinet.start_two = true,
            KeyCode::Up => set_held_flag(
                &mut self.held.altitude_up,
                key_event.kind,
                &mut input.cabinet.altitude_up,
            ),
            KeyCode::Down => set_held_flag(
                &mut self.held.altitude_down,
                key_event.kind,
                &mut input.cabinet.altitude_down,
            ),
            KeyCode::Char('r') | KeyCode::Char('R') if pressed => input.cabinet.reverse = true,
            KeyCode::Char('t') | KeyCode::Char('T') => set_held_flag(
                &mut self.held.thrust,
                key_event.kind,
                &mut input.cabinet.thrust,
            ),
            KeyCode::Char('f') | KeyCode::Char('F') if pressed => input.cabinet.fire = true,
            KeyCode::Char('b') | KeyCode::Char('B') if pressed => input.cabinet.smart_bomb = true,
            KeyCode::Char('h') | KeyCode::Char('H') if pressed => input.cabinet.hyperspace = true,
            _ => {}
        }
    }

    fn map_test_key(&mut self, key_event: KeyEvent, input: &mut PolledInput) {
        self.map_cabinet_key(key_event, input);
    }
}

fn set_held_flag(held: &mut bool, kind: KeyEventKind, output: &mut bool) {
    match kind {
        KeyEventKind::Press | KeyEventKind::Repeat => {
            *held = true;
            *output = true;
        }
        KeyEventKind::Release => *held = false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XyzzyOverlay {
    active: bool,
    sequence_index: usize,
    invincible: bool,
    auto_fire: bool,
}

impl XyzzyOverlay {
    const CODE: [char; 5] = ['x', 'y', 'z', 'z', 'y'];

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn invincible(&self) -> bool {
        self.invincible
    }

    pub fn auto_fire(&self) -> bool {
        self.auto_fire
    }

    pub fn handle_typed_chars(&mut self, chars: &[char]) {
        for &character in chars {
            let character = character.to_ascii_lowercase();
            self.update_sequence(character);
            if self.active && character == 'g' {
                self.invincible = !self.invincible;
            } else if self.active && character == 'f' {
                self.auto_fire = !self.auto_fire;
            }
        }
    }

    fn update_sequence(&mut self, character: char) {
        if character == Self::CODE[self.sequence_index] {
            self.sequence_index += 1;
            if self.sequence_index == Self::CODE.len() {
                self.active = !self.active;
                self.sequence_index = 0;
                if !self.active {
                    self.invincible = false;
                    self.auto_fire = false;
                }
            }
            return;
        }

        self.sequence_index = usize::from(character == Self::CODE[0]);
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, ModifierKeyCode};

    use crate::{
        assets::RED_LABEL_INPUT_PORTS_TSV,
        input::{
            DEFENDER_IN0_ALTITUDE_DOWN, DEFENDER_IN0_FIRE, DEFENDER_IN0_HYPERSPACE,
            DEFENDER_IN0_REVERSE, DEFENDER_IN0_SMART_BOMB, DEFENDER_IN0_START_ONE,
            DEFENDER_IN0_START_TWO, DEFENDER_IN0_THRUST, DEFENDER_IN1_ALTITUDE_UP,
            DEFENDER_IN2_ADVANCE, DEFENDER_IN2_AUTO_UP_MANUAL_DOWN, DEFENDER_IN2_COIN_ONE,
            DEFENDER_IN2_COIN_THREE, DEFENDER_IN2_COIN_TWO, DEFENDER_IN2_HIGH_SCORE_RESET,
            DEFENDER_IN2_TILT,
        },
    };

    use super::{DefenderInputPorts, InputMapper, InputProfile, PolledInput, XyzzyOverlay};

    #[test]
    fn planetoid_profile_maps_current_live_controls_to_cabinet_actions() {
        let mut mapper = InputMapper::new(InputProfile::Planetoid);
        let mut input = PolledInput::default();

        mapper.handle_key_event(
            KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT),
            &mut input,
        );
        mapper.handle_key_event(
            KeyEvent::new_with_kind(
                KeyCode::Modifier(ModifierKeyCode::LeftShift),
                KeyModifiers::NONE,
                KeyEventKind::Press,
            ),
            &mut input,
        );
        mapper.handle_key_event(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            &mut input,
        );
        mapper.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE), &mut input);
        mapper.handle_key_event(
            KeyEvent::new(KeyCode::Char('H'), KeyModifiers::SHIFT),
            &mut input,
        );
        mapper.handle_key_event(
            KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE),
            &mut input,
        );
        mapper.handle_key_event(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut input,
        );

        assert!(input.cabinet.coin);
        assert!(input.cabinet.altitude_up);
        assert!(input.cabinet.thrust);
        assert!(input.cabinet.reverse);
        assert!(input.cabinet.smart_bomb);
        assert!(input.cabinet.hyperspace);
        assert!(input.cabinet.fire);
        assert!(input.cabinet.start_one);
    }

    #[test]
    fn cabinet_profile_maps_coin_and_start_buttons() {
        let mut mapper = InputMapper::new(InputProfile::Cabinet);
        let mut input = PolledInput::default();

        mapper.handle_key_event(
            KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE),
            &mut input,
        );
        mapper.handle_key_event(
            KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE),
            &mut input,
        );

        assert!(input.cabinet.coin);
        assert!(input.cabinet.start_one);
    }

    #[test]
    fn profile_labels_and_parser_are_stable() {
        assert_eq!(
            InputProfile::parse("planetoid"),
            Some(InputProfile::Planetoid)
        );
        assert_eq!(InputProfile::parse("cabinet"), Some(InputProfile::Cabinet));
        assert_eq!(InputProfile::parse("test"), Some(InputProfile::Test));
        assert_eq!(InputProfile::parse("unknown"), None);
        assert_eq!(InputProfile::Cabinet.label(), "cabinet");
    }

    #[test]
    fn cabinet_input_bits_and_merge_are_stable() {
        let mut input = super::CabinetInput {
            coin: true,
            fire: true,
            ..super::CabinetInput::NONE
        };
        input.merge(super::CabinetInput {
            hyperspace: true,
            ..super::CabinetInput::NONE
        });

        assert_eq!(input.bits(), 0b10_1000_0001);
    }

    #[test]
    fn cabinet_input_projects_to_mame_defender_input_ports() {
        let input = super::CabinetInput {
            coin: true,
            start_one: true,
            start_two: true,
            altitude_up: true,
            altitude_down: true,
            reverse: true,
            thrust: true,
            fire: true,
            smart_bomb: true,
            hyperspace: true,
            ..super::CabinetInput::NONE
        };
        let ports = input.defender_input_ports();

        assert_eq!(
            ports,
            DefenderInputPorts {
                in0: DEFENDER_IN0_FIRE
                    | DEFENDER_IN0_THRUST
                    | DEFENDER_IN0_SMART_BOMB
                    | DEFENDER_IN0_HYPERSPACE
                    | DEFENDER_IN0_START_TWO
                    | DEFENDER_IN0_START_ONE
                    | DEFENDER_IN0_REVERSE
                    | DEFENDER_IN0_ALTITUDE_DOWN,
                in1: DEFENDER_IN1_ALTITUDE_UP,
                in2: DEFENDER_IN2_COIN_ONE,
            }
        );
        assert_eq!(ports.pia0_port_a(), ports.in0);
        assert_eq!(ports.pia0_port_b(), ports.in1);
        assert_eq!(ports.pia1_port_a(), ports.in2);
    }

    #[test]
    fn service_and_secondary_coin_lines_project_to_mame_defender_in2() {
        let input = super::CabinetInput {
            coin: true,
            coin_two: true,
            coin_three: true,
            auto_up_manual_down: true,
            service_advance: true,
            high_score_reset: true,
            tilt: true,
            ..super::CabinetInput::NONE
        };

        assert_eq!(
            input.defender_input_ports().in2,
            DEFENDER_IN2_AUTO_UP_MANUAL_DOWN
                | DEFENDER_IN2_ADVANCE
                | DEFENDER_IN2_COIN_THREE
                | DEFENDER_IN2_HIGH_SCORE_RESET
                | DEFENDER_IN2_COIN_ONE
                | DEFENDER_IN2_COIN_TWO
                | DEFENDER_IN2_TILT
        );
        assert_eq!(input.bits(), 0b1111_1100_0000_0001);
    }

    #[test]
    fn input_masks_match_embedded_mame_port_asset() {
        assert_eq!(asset_bit("IN0", "fire"), DEFENDER_IN0_FIRE);
        assert_eq!(asset_bit("IN0", "thrust"), DEFENDER_IN0_THRUST);
        assert_eq!(asset_bit("IN0", "smart_bomb"), DEFENDER_IN0_SMART_BOMB);
        assert_eq!(asset_bit("IN0", "hyperspace"), DEFENDER_IN0_HYPERSPACE);
        assert_eq!(asset_bit("IN0", "start_two"), DEFENDER_IN0_START_TWO);
        assert_eq!(asset_bit("IN0", "start_one"), DEFENDER_IN0_START_ONE);
        assert_eq!(asset_bit("IN0", "reverse"), DEFENDER_IN0_REVERSE);
        assert_eq!(
            asset_bit("IN0", "altitude_down"),
            DEFENDER_IN0_ALTITUDE_DOWN
        );
        assert_eq!(asset_bit("IN1", "altitude_up"), DEFENDER_IN1_ALTITUDE_UP);
        assert_eq!(
            asset_bit("IN2", "auto_up_manual_down"),
            DEFENDER_IN2_AUTO_UP_MANUAL_DOWN
        );
        assert_eq!(asset_bit("IN2", "advance"), DEFENDER_IN2_ADVANCE);
        assert_eq!(asset_bit("IN2", "coin_three"), DEFENDER_IN2_COIN_THREE);
        assert_eq!(
            asset_bit("IN2", "high_score_reset"),
            DEFENDER_IN2_HIGH_SCORE_RESET
        );
        assert_eq!(asset_bit("IN2", "coin_one"), DEFENDER_IN2_COIN_ONE);
        assert_eq!(asset_bit("IN2", "coin_two"), DEFENDER_IN2_COIN_TWO);
        assert_eq!(asset_bit("IN2", "tilt"), DEFENDER_IN2_TILT);
    }

    #[test]
    fn held_keys_release_cleanly() {
        let mut mapper = InputMapper::new(InputProfile::Planetoid);
        let mut input = PolledInput::default();

        mapper.handle_key_event(
            KeyEvent::new_with_kind(KeyCode::Char('a'), KeyModifiers::NONE, KeyEventKind::Press),
            &mut input,
        );
        assert!(input.cabinet.altitude_up);

        let mut release = PolledInput::default();
        mapper.handle_key_event(
            KeyEvent::new_with_kind(
                KeyCode::Char('a'),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
            &mut release,
        );
        mapper.apply_held(&mut release);
        assert!(!release.cabinet.altitude_up);
    }

    #[test]
    fn quit_keys_and_typed_chars_are_collected() {
        let mut mapper = InputMapper::new(InputProfile::Planetoid);
        let mut input = PolledInput::default();

        mapper.handle_key_event(
            KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT),
            &mut input,
        );
        mapper.handle_key_event(
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            &mut input,
        );
        mapper.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &mut input);

        assert_eq!(input.typed_chars, vec!['x', '\u{8}']);
        assert!(input.quit_requested);
    }

    #[test]
    fn xyzzy_overlay_toggles_and_resets_hidden_flags() {
        let mut overlay = XyzzyOverlay::default();

        overlay.handle_typed_chars(&['x', 'y', 'z', 'z', 'y']);
        assert!(overlay.active());

        overlay.handle_typed_chars(&['g', 'f']);
        assert!(overlay.invincible());
        assert!(overlay.auto_fire());

        overlay.handle_typed_chars(&['x', 'y', 'z', 'z', 'y']);
        assert!(!overlay.active());
        assert!(!overlay.invincible());
        assert!(!overlay.auto_fire());
    }

    fn asset_bit(port: &str, name: &str) -> u8 {
        for line in RED_LABEL_INPUT_PORTS_TSV.lines().skip(1) {
            let mut fields = line.split('\t');
            let row_port = fields.next().unwrap_or("");
            let row_bit = fields.next().unwrap_or("");
            let row_name = fields.next().unwrap_or("");
            if row_port == port && row_name == name {
                return parse_hex_byte(row_bit);
            }
        }

        panic!("missing input port asset row for {port} {name}");
    }

    fn parse_hex_byte(value: &str) -> u8 {
        u8::from_str_radix(value.strip_prefix("0x").unwrap_or(value), 16)
            .expect("input port bit should be hex byte")
    }
}
