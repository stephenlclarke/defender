#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameInput {
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
    pub service_auto_up: bool,
    pub service_advance: bool,
    pub high_score_reset: bool,
    pub high_score_initial: Option<char>,
    pub high_score_backspace: bool,
    pub tilt: bool,
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
        reverse: false,
        thrust: false,
        fire: false,
        smart_bomb: false,
        hyperspace: false,
        service_auto_up: false,
        service_advance: false,
        high_score_reset: false,
        high_score_initial: None,
        high_score_backspace: false,
        tilt: false,
    };
}
