const OBJECT_ACTIVE_LEFT_MARGIN: u16 = 100 * 32; // original: SOURCE_OBJECT_ACTIVE_LEFT_BUFFER
const OBJECT_ACTIVE_WORLD_WIDTH: u16 = 500 * 32; // original: SOURCE_OBJECT_ACTIVE_WINDOW
const OBJECT_WORLD_TO_SCREEN_SHIFT: u8 = 6; // original: SOURCE_OBJECT_SCREEN_X_SHIFT
const OBJECT_VISIBLE_SCREEN_WIDTH: u16 = 292; // original: SOURCE_OBJECT_VISIBLE_WIDTH
const OBJECT_EVIDENCE_TABLE_BASE_ADDRESS: u16 = 0xA23C; // original: SOURCE_OBJECT_TABLE_BASE
const OBJECT_EVIDENCE_SLOT_STRIDE: u16 = 0x17; // original: SOURCE_OBJECT_TABLE_STRIDE
const OBJECT_EVIDENCE_DEFAULT_TYPE: u8 = 0x00; // original: SOURCE_OBJECT_DEFAULT_TYPE
const BOMBER_PICTURE_FRAME_COUNT: u8 = 4; // original: SOURCE_BOMBER_PICTURE_FRAME_COUNT
const ENEMY_BOMB_ARCADE_ROUTINE_ADDRESS: u16 = 0xE498; // original: SOURCE_BOMB_SHELL_OUTPUT_ROUTINE_ADDRESS
const FIREBALL_ARCADE_ROUTINE_ADDRESS: u16 = 0xE4D9; // original: SOURCE_FIREBALL_OUTPUT_ROUTINE_ADDRESS
const ENEMY_BOMB_PICTURE_LABEL: &str = "BMBP1"; // original: SOURCE_BOMB_SHELL_PICTURE_LABEL
const ENEMY_BOMB_PICTURE_DESCRIPTOR_ADDRESS: u16 = 0xF95B; // original: SOURCE_BOMB_SHELL_PICTURE_ADDRESS
const ENEMY_BOMB_PRIMARY_IMAGE_ADDRESS: u16 = 0xCCB0; // original: SOURCE_BOMB_SHELL_PRIMARY_IMAGE_ADDRESS
const ENEMY_BOMB_ALTERNATE_IMAGE_ADDRESS: u16 = 0xCCB6; // original: SOURCE_BOMB_SHELL_ALTERNATE_IMAGE_ADDRESS
const ENEMY_BOMB_PICTURE_SIZE: (u8, u8) = (2, 3); // original: SOURCE_BOMB_SHELL_PICTURE_SIZE
const BAITER_PICTURE_FRAME_COUNT: u8 = 3; // original: SOURCE_BAITER_PICTURE_FRAME_COUNT
// Clean stores process sleep as frames remaining after the current update.
// Source PTIME values of 6/4/1 therefore wake after 5/3/0 clean sleeps.
const LANDER_PICTURE_FRAME_COUNT: u8 = 3; // original: SOURCE_LANDER_PICTURE_FRAME_COUNT

fn baiter_screen_x_velocity(x_velocity_word: u16) -> u16 {
    x_velocity_word.wrapping_shl(2)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub frame: u64,
    pub phase: GamePhase,
    pub credits: u8,
    pub current_player: u8,
    pub player_count: u8,
    pub wave: u8,
    pub wave_profile: WaveProfileSnapshot,
    pub player: PlayerSnapshot,
    pub player_stocks: [PlayerStockSnapshot; 2],
    pub scores: ScoreSnapshot,
    pub attract: AttractPresentationSnapshot,
    pub post_game_playfield: Option<PostGamePlayfieldSnapshot>,
    pub high_score_initials: HighScoreInitialsState,
    pub high_score_entry: Option<HighScoreEntrySnapshot>,
    pub high_score_submission: Option<HighScoreSubmissionSnapshot>,
    pub high_score_tables: HighScoreTablesSnapshot,
    pub game_over: GameOverSnapshot,
    pub world: WorldSnapshot,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameOverSnapshot {
    pub player_death_sleep_remaining: Option<u8>,
    pub player_switch_sleep_remaining: Option<u8>,
    pub player_switch_from: Option<u8>,
    pub player_switch_to: Option<u8>,
    pub no_entry_delay_remaining: Option<u8>,
    pub hall_of_fame_stall_remaining: Option<u8>,
}

impl GameOverSnapshot {
    pub const NONE: Self = Self {
        player_death_sleep_remaining: None,
        player_switch_sleep_remaining: None,
        player_switch_from: None,
        player_switch_to: None,
        no_entry_delay_remaining: None,
        hall_of_fame_stall_remaining: None,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreEntrySnapshot {
    pub score: u32,
    pub rank: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreSubmissionSnapshot {
    pub player: u8,
    pub score: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreTableEntrySnapshot {
    pub rank: u8,
    pub score: u32,
    pub initials: [Option<char>; 3],
}

impl HighScoreTableEntrySnapshot {
    pub const EMPTY: Self = Self {
        rank: 0,
        score: 0,
        initials: [None, None, None],
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreTablesSnapshot {
    pub all_time: [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES],
    pub todays_greatest: [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES],
}

impl HighScoreTablesSnapshot {
    pub const EMPTY: Self = Self {
        all_time: EMPTY_HIGH_SCORE_TABLE,
        todays_greatest: EMPTY_HIGH_SCORE_TABLE,
    };

    pub const DEFAULT: Self = Self {
        all_time: DEFAULT_HIGH_SCORE_TABLE,
        todays_greatest: DEFAULT_HIGH_SCORE_TABLE,
    };
}

const EMPTY_HIGH_SCORE_TABLE: [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES] = [
    HighScoreTableEntrySnapshot {
        rank: 1,
        score: 0,
        initials: [None, None, None],
    },
    HighScoreTableEntrySnapshot {
        rank: 2,
        score: 0,
        initials: [None, None, None],
    },
    HighScoreTableEntrySnapshot {
        rank: 3,
        score: 0,
        initials: [None, None, None],
    },
    HighScoreTableEntrySnapshot {
        rank: 4,
        score: 0,
        initials: [None, None, None],
    },
    HighScoreTableEntrySnapshot {
        rank: 5,
        score: 0,
        initials: [None, None, None],
    },
    HighScoreTableEntrySnapshot {
        rank: 6,
        score: 0,
        initials: [None, None, None],
    },
    HighScoreTableEntrySnapshot {
        rank: 7,
        score: 0,
        initials: [None, None, None],
    },
    HighScoreTableEntrySnapshot {
        rank: 8,
        score: 0,
        initials: [None, None, None],
    },
];

const DEFAULT_HIGH_SCORE_TABLE: [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES] = [
    HighScoreTableEntrySnapshot {
        rank: 1,
        score: 21_270,
        initials: [Some('D'), Some('R'), Some('J')],
    },
    HighScoreTableEntrySnapshot {
        rank: 2,
        score: 18_315,
        initials: [Some('S'), Some('A'), Some('M')],
    },
    HighScoreTableEntrySnapshot {
        rank: 3,
        score: 15_920,
        initials: [Some('L'), Some('E'), Some('D')],
    },
    HighScoreTableEntrySnapshot {
        rank: 4,
        score: 14_285,
        initials: [Some('P'), Some('G'), Some('O')],
    },
    HighScoreTableEntrySnapshot {
        rank: 5,
        score: 12_520,
        initials: [Some('C'), Some('R'), Some('A')],
    },
    HighScoreTableEntrySnapshot {
        rank: 6,
        score: 11_035,
        initials: [Some('M'), Some('R'), Some('S')],
    },
    HighScoreTableEntrySnapshot {
        rank: 7,
        score: 8_265,
        initials: [Some('S'), Some('S'), Some('R')],
    },
    HighScoreTableEntrySnapshot {
        rank: 8,
        score: 6_010,
        initials: [Some('T'), Some('M'), Some('H')],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    CreditAdded,
    GameStarted,
    DiagnosticsSelected,
    AuditsSelected,
    HighScoreReset,
    ReversePressed,
    FirePressed,
    SmartBombPressed,
    HyperspacePressed,
    EnemyDestroyed,
    PlayerDestroyed,
    WaveCleared,
    WaveStarted,
    BonusAwarded,
    GameOver,
    HighScoreEntryStarted,
    HighScoreInitialAccepted,
    HighScoreSubmitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEvent {
    Startup,
    CreditAdded,
    GameStarted,
    ThrustStarted,
    ThrustStopped,
    UnmappedSoundCommand {
        command: crate::SoundCommand,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEvents {
    gameplay: Vec<GameEvent>,
    sounds: Vec<SoundEvent>,
}

impl GameEvents {
    pub fn new(gameplay: Vec<GameEvent>, sounds: Vec<SoundEvent>) -> Self {
        Self { gameplay, sounds }
    }

    pub fn gameplay(&self) -> &[GameEvent] {
        &self.gameplay
    }

    pub fn sounds(&self) -> &[SoundEvent] {
        &self.sounds
    }

    pub fn is_empty(&self) -> bool {
        self.gameplay.is_empty() && self.sounds.is_empty()
    }
}

impl Default for GameEvents {
    fn default() -> Self {
        Self::new(Vec::new(), Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameFrame {
    pub state: GameState,
    pub events: GameEvents,
    pub scene: RenderScene,
}
