#[cfg(all(not(test), not(coverage)))]
fn renderable_window_size(size: PhysicalSize<u32>) -> Option<(u32, u32)> {
    if size.width == 0 || size.height == 0 {
        None
    } else {
        Some((size.width, size.height))
    }
}

#[cfg(any(test, all(not(test), not(coverage))))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiveControl {
    Coin,
    StartOne,
    StartTwo,
    AltitudeUp,
    AltitudeDown,
    Reverse,
    Thrust,
    Fire,
    SmartBomb,
    Hyperspace,
    ServiceAutoUp,
    ServiceAdvance,
    HighScoreReset,
    HighScoreBackspace,
    HighScoreInitial(char),
    Quit,
}

#[cfg(any(test, all(not(test), not(coverage))))]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct LiveInputState {
    coin: bool,
    start_one: bool,
    start_two: bool,
    altitude_up: bool,
    altitude_down: bool,
    reverse: bool,
    thrust: bool,
    fire: bool,
    smart_bomb: bool,
    hyperspace: bool,
    service_auto_up: bool,
    service_advance: bool,
    high_score_reset: bool,
    high_score_initial: Option<char>,
    high_score_backspace: bool,
    xyzzy: XyzzyController,
    overlay_smart_bomb: bool,
}

#[cfg(any(test, all(not(test), not(coverage))))]
impl LiveInputState {
    #[cfg(all(not(test), not(coverage)))]
    fn observe_key_event_for_xyzzy(&mut self, event: &KeyEvent, control: Option<LiveControl>) {
        if event.state != ElementState::Pressed {
            return;
        }
        if matches!(control, Some(LiveControl::HighScoreInitial(_))) {
            return;
        }
        let Some(character) = logical_key_character(&event.logical_key) else {
            return;
        };
        self.ingest_xyzzy_character(character);
    }

    fn ingest_xyzzy_character(&mut self, character: char) {
        self.xyzzy.ingest(character);
        if self.xyzzy.active() {
            match character.to_ascii_lowercase() {
                'f' => self.xyzzy.toggle_auto_fire(),
                'g' => self.xyzzy.toggle_invincible(),
                _ => {}
            }
        }
    }

    fn apply(&mut self, control: LiveControl, pressed: bool) {
        match control {
            LiveControl::Coin => self.coin |= pressed,
            LiveControl::StartOne => self.start_one |= pressed,
            LiveControl::StartTwo => self.start_two |= pressed,
            LiveControl::AltitudeUp => self.altitude_up = pressed,
            LiveControl::AltitudeDown => self.altitude_down = pressed,
            LiveControl::Reverse => self.reverse = pressed,
            LiveControl::Thrust => self.thrust = pressed,
            LiveControl::Fire => self.fire = pressed,
            LiveControl::SmartBomb => {
                self.smart_bomb = pressed;
                if pressed && self.xyzzy.active() {
                    self.overlay_smart_bomb = true;
                }
            }
            LiveControl::Hyperspace => self.hyperspace = pressed,
            LiveControl::ServiceAutoUp => self.service_auto_up = pressed,
            LiveControl::ServiceAdvance => self.service_advance |= pressed,
            LiveControl::HighScoreReset => self.high_score_reset |= pressed,
            LiveControl::HighScoreBackspace => self.high_score_backspace |= pressed,
            LiveControl::HighScoreInitial(value) => {
                if pressed {
                    self.high_score_initial = Some(value);
                    self.ingest_xyzzy_character(value);
                }
            }
            LiveControl::Quit => {}
        }
    }

    fn drain_game_input(&mut self) -> GameInput {
        GameInput {
            coin: take_bool(&mut self.coin),
            coin_two: false,
            coin_three: false,
            start_one: take_bool(&mut self.start_one),
            start_two: take_bool(&mut self.start_two),
            altitude_up: self.altitude_up,
            altitude_down: self.altitude_down,
            reverse: take_bool(&mut self.reverse),
            thrust: self.thrust,
            fire: self.fire,
            smart_bomb: self.smart_bomb,
            hyperspace: self.hyperspace,
            service_auto_up: self.service_auto_up,
            service_advance: take_bool(&mut self.service_advance),
            high_score_reset: take_bool(&mut self.high_score_reset),
            high_score_initial: self.high_score_initial.take(),
            high_score_backspace: take_bool(&mut self.high_score_backspace),
            tilt: false,
        }
    }

    fn drain_xyzzy_mode(&mut self) -> XyzzyMode {
        self.xyzzy.mode(take_bool(&mut self.overlay_smart_bomb))
    }
}

#[cfg(all(not(test), not(coverage)))]
fn logical_key_character(key: &Key) -> Option<char> {
    match key {
        Key::Character(text) => single_character(text),
        _ => None,
    }
}

#[cfg(any(test, all(not(test), not(coverage))))]
fn take_bool(value: &mut bool) -> bool {
    let taken = *value;
    *value = false;
    taken
}

#[cfg(all(not(test), not(coverage)))]
fn live_control_from_winit(profile: LiveInputProfile, event: &KeyEvent) -> Option<LiveControl> {
    physical_control(profile, &event.physical_key)
        .or_else(|| logical_control(profile, &event.logical_key))
}

#[cfg(any(test, all(not(test), not(coverage))))]
fn physical_control(profile: LiveInputProfile, physical_key: &PhysicalKey) -> Option<LiveControl> {
    let PhysicalKey::Code(code) = physical_key else {
        return None;
    };

    match code {
        KeyCode::Escape => Some(LiveControl::Quit),
        KeyCode::Digit5 | KeyCode::Numpad5 => Some(LiveControl::Coin),
        KeyCode::Digit1 | KeyCode::Numpad1 => Some(LiveControl::StartOne),
        KeyCode::Digit2 | KeyCode::Numpad2 => Some(LiveControl::StartTwo),
        KeyCode::F1 => Some(LiveControl::ServiceAutoUp),
        KeyCode::F2 => Some(LiveControl::ServiceAdvance),
        KeyCode::F3 => Some(LiveControl::HighScoreReset),
        KeyCode::Backspace => Some(LiveControl::HighScoreBackspace),
        _ => gameplay_physical_control(profile, *code),
    }
}

#[cfg(any(test, all(not(test), not(coverage))))]
fn gameplay_physical_control(profile: LiveInputProfile, code: KeyCode) -> Option<LiveControl> {
    match profile {
        LiveInputProfile::Planetoid => match code {
            KeyCode::Enter | KeyCode::NumpadEnter => Some(LiveControl::Fire),
            KeyCode::ShiftLeft | KeyCode::ShiftRight => Some(LiveControl::Reverse),
            KeyCode::KeyA => Some(LiveControl::AltitudeUp),
            KeyCode::KeyZ => Some(LiveControl::AltitudeDown),
            KeyCode::Space => Some(LiveControl::Thrust),
            KeyCode::Tab => Some(LiveControl::SmartBomb),
            KeyCode::KeyH => Some(LiveControl::Hyperspace),
            _ => None,
        },
        LiveInputProfile::Cabinet | LiveInputProfile::Test => match code {
            KeyCode::KeyF => Some(LiveControl::Fire),
            KeyCode::KeyT => Some(LiveControl::Thrust),
            KeyCode::ArrowUp => Some(LiveControl::AltitudeUp),
            KeyCode::ArrowDown => Some(LiveControl::AltitudeDown),
            KeyCode::KeyR => Some(LiveControl::Reverse),
            KeyCode::KeyB => Some(LiveControl::SmartBomb),
            KeyCode::KeyH => Some(LiveControl::Hyperspace),
            _ => None,
        },
    }
}

#[cfg(any(test, all(not(test), not(coverage))))]
fn logical_control(profile: LiveInputProfile, logical_key: &Key) -> Option<LiveControl> {
    match logical_key {
        Key::Named(NamedKey::Escape) => Some(LiveControl::Quit),
        Key::Named(NamedKey::Enter) => {
            (profile == LiveInputProfile::Planetoid).then_some(LiveControl::Fire)
        }
        Key::Named(NamedKey::Tab) => {
            (profile == LiveInputProfile::Planetoid).then_some(LiveControl::SmartBomb)
        }
        Key::Named(NamedKey::Backspace) => Some(LiveControl::HighScoreBackspace),
        Key::Named(NamedKey::ArrowUp) => {
            (profile != LiveInputProfile::Planetoid).then_some(LiveControl::AltitudeUp)
        }
        Key::Named(NamedKey::ArrowDown) => {
            (profile != LiveInputProfile::Planetoid).then_some(LiveControl::AltitudeDown)
        }
        Key::Named(NamedKey::Shift) => {
            (profile == LiveInputProfile::Planetoid).then_some(LiveControl::Reverse)
        }
        Key::Named(NamedKey::F1) => Some(LiveControl::ServiceAutoUp),
        Key::Named(NamedKey::F2) => Some(LiveControl::ServiceAdvance),
        Key::Named(NamedKey::F3) => Some(LiveControl::HighScoreReset),
        Key::Character(text) => character_control(profile, text),
        _ => None,
    }
}

#[cfg(any(test, all(not(test), not(coverage))))]
fn character_control(profile: LiveInputProfile, text: &str) -> Option<LiveControl> {
    let value = single_character(text)?;
    match value.to_ascii_lowercase() {
        '1' => Some(LiveControl::StartOne),
        '2' => Some(LiveControl::StartTwo),
        '5' => Some(LiveControl::Coin),
        'a' if profile == LiveInputProfile::Planetoid => Some(LiveControl::AltitudeUp),
        'z' if profile == LiveInputProfile::Planetoid => Some(LiveControl::AltitudeDown),
        ' ' if profile == LiveInputProfile::Planetoid => Some(LiveControl::Thrust),
        'h' => Some(LiveControl::Hyperspace),
        'f' if profile != LiveInputProfile::Planetoid => Some(LiveControl::Fire),
        't' if profile != LiveInputProfile::Planetoid => Some(LiveControl::Thrust),
        'r' if profile != LiveInputProfile::Planetoid => Some(LiveControl::Reverse),
        'b' if profile != LiveInputProfile::Planetoid => Some(LiveControl::SmartBomb),
        'a'..='z' => Some(LiveControl::HighScoreInitial(value.to_ascii_uppercase())),
        _ => None,
    }
}

#[cfg(any(test, all(not(test), not(coverage))))]
fn single_character(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let value = chars.next()?;
    chars.next().is_none().then_some(value)
}
