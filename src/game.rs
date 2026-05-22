//! Domain-facing gameplay contracts.

use std::sync::OnceLock;

use crate::{
    renderer::{
        Color, RenderLayer, RenderScene, SceneSprite, SpriteId, SurfaceSize,
        push_source_controlled_message_sprites, push_source_message_sprites,
        push_source_text_bytes_sprites, source_attract_defender_appearance_pixels,
        source_attract_williams_logo_operation_pixel_counts,
        source_attract_williams_logo_pixel_path, source_message_text, source_screen_position,
        source_screen_position_with_offset,
    },
    systems::{
        CollisionBox, CollisionSystem, EnemyMotionSystem, GameSimulation, HighScoreEntrySystem,
        HighScoreInitialsState, OperatorControlSystem, PlayerControlSystem, PlayerDamageSystem,
        PlayerMotionState, PlayerMotionSystem, PlayerStock, ProjectileLaunchOutcome,
        ProjectileMotionSystem, ProjectileState, ProjectileSystem, ScoreSystem, ScreenPosition,
        ScreenVelocity, SmartBombSystem, WaveState, WaveStatus, WaveSystem,
    },
};

// Source-backed cabinet defaults from assets/red-label CMOS/high-score evidence.
const DEFAULT_CABINET_WAVE: u8 = 1;
const DEFAULT_PLAYER_LIVES: u8 = 3;
const DEFAULT_HIGH_SCORE: u32 = 21_270;
const DEFAULT_REPLAY_SCORE: u32 = 10_000;
pub const HIGH_SCORE_TABLE_ENTRIES: usize = 8;
const PLAYER_DEATH_GAME_OVER_SLEEP_FRAMES: u8 = 40;
const PLAYER_SWITCH_SLEEP_FRAMES: u8 = 0x60;
const HALL_OF_FAME_NO_ENTRY_DELAY_FRAMES: u8 = 0xFF;
const HALL_OF_FAME_STALL_FRAMES: u8 = 60;
const COIN_CREDIT_DELAY_FRAMES: u8 = 10;
const COIN_CREDIT_SOUND_DELAY_FRAMES: u8 = 1;
const START_SOUND_DELAY_FRAMES: u8 = 1;
const START_PLAYFIELD_DELAY_FRAMES: u8 = 2;
const ATTRACT_PRESENTS_START_FRAME: u16 = 236;
const ATTRACT_COPYRIGHT_START_FRAME: u16 = 488;
const ATTRACT_HALL_OF_FAME_START_FRAME: u16 = 488;
const ATTRACT_HALL_OF_FAME_STALL_TICK_FRAMES: u16 = 10;
pub(crate) const ATTRACT_SCORING_SEQUENCE_START_FRAME: u16 = ATTRACT_HALL_OF_FAME_START_FRAME
    + (HALL_OF_FAME_STALL_FRAMES as u16 * ATTRACT_HALL_OF_FAME_STALL_TICK_FRAMES);
const ATTRACT_LOGO_SLEEP_TICKS: u8 = 2;
const ATTRACT_PRESENTS_SLEEP_TICKS: u8 = 5;
const ATTRACT_DEFENDER_ENTRY_SLEEP_TICKS: u8 = 0x30;
const ATTRACT_DEFENDER_APPEAR_SLEEP_TICKS: u8 = 0x2E;
const ATTRACT_COPYRIGHT_SLEEP_TICKS: u8 = 10;
const ATTRACT_COPYRIGHT_STALL_TICKS: u8 = 60;
const ATTRACT_INSTRUCTION_ENTRY_SLEEP_TICKS: u8 = 0xE6;
const SOURCE_ATTRACT_WILLIAMS_INITIAL_BYTES_PER_SLICE: usize = 3;
pub(crate) const ATTRACT_WILLIAMS_LOGO_REVEAL_FRAMES: u16 = ATTRACT_PRESENTS_START_FRAME;
pub(crate) const ATTRACT_DEFENDER_WORDMARK_START_FRAME: u16 = 365;
const SOURCE_ATTRACT_DEFENDER_APPEARANCE_TICK_NUMERATOR: u16 = 3;
const SOURCE_ATTRACT_DEFENDER_APPEARANCE_TICK_DENOMINATOR: u16 = 2;
pub(crate) const ATTRACT_DEFENDER_WORDMARK_REVEAL_FRAMES: u16 =
    ((ATTRACT_DEFENDER_APPEAR_SLEEP_TICKS as u16
        * SOURCE_ATTRACT_DEFENDER_APPEARANCE_TICK_DENOMINATOR)
        + SOURCE_ATTRACT_DEFENDER_APPEARANCE_TICK_NUMERATOR
        - 1)
        / SOURCE_ATTRACT_DEFENDER_APPEARANCE_TICK_NUMERATOR;
const SOURCE_ATTRACT_DEFENDER_DESCRIPTOR: u16 = 0xB300;
#[cfg(test)]
const SOURCE_ATTRACT_DEFENDER_OBJECTS: u16 = 0xB304;
#[cfg(test)]
const SOURCE_ATTRACT_DEFENDER_PICTURES: u16 = 0xB3D6;
const SOURCE_ATTRACT_DEFENDER_DATA: u16 = 0xB412;
#[cfg(test)]
const SOURCE_ATTRACT_DEFENDER_OBJECT_BYTES: u16 = 14;
#[cfg(test)]
const SOURCE_ATTRACT_DEFENDER_PICTURE_BYTES: u16 = 4;
#[cfg(test)]
const SOURCE_ATTRACT_DEFENDER_PICTURE_DATA_STEP: u16 = 4 * 24;
#[cfg(test)]
const SOURCE_ATTRACT_DEFENDER_INITIAL_X16: u16 = 0x0C00;
#[cfg(test)]
const SOURCE_ATTRACT_DEFENDER_X16_STEP: u16 = 0x0100;
#[cfg(test)]
const SOURCE_ATTRACT_DEFENDER_Y16: u16 = 0x9800;
#[cfg(test)]
const SOURCE_ATTRACT_DEFENDER_APPEARANCE_SLOT: u16 = 0x9C00;
#[cfg(test)]
const SOURCE_ATTRACT_DEFENDER_APPEARANCE_SLOT_STEP: u16 = 0x40;
const ATTRACT_SCORING_RESCUE_DESCENT_TICKS: u16 = 0xE6;
const ATTRACT_SCORING_RESCUE_ASCENT_TICKS: u16 = 0xA0;
const ATTRACT_SCORING_RESCUE_LASER_TICKS: u16 = 0x15;
const ATTRACT_SCORING_RESCUE_FALL_TICKS: u16 = 0x2D * 2;
const ATTRACT_SCORING_RESCUE_SCORE_TICKS: u16 = 0x50;
const ATTRACT_SCORING_RESCUE_RETURN_TICKS: u16 = 0x60;
const ATTRACT_SCORING_LEGEND_APPROACH_TICKS: u16 = 0x5F;
const ATTRACT_SCORING_LEGEND_LASER_TICKS: u16 = 0x17;
const ATTRACT_SCORING_LEGEND_TRANSFER_TICKS: u16 = 0x20;
const ATTRACT_SCORING_LEGEND_REVEAL_TICKS: u16 = 0x20;
const ATTRACT_SCORING_LEGEND_ENTRY_TICKS: u16 = ATTRACT_SCORING_LEGEND_APPROACH_TICKS
    + ATTRACT_SCORING_LEGEND_LASER_TICKS
    + ATTRACT_SCORING_LEGEND_TRANSFER_TICKS
    + ATTRACT_SCORING_LEGEND_REVEAL_TICKS;
const ATTRACT_SCORING_LEGEND_HOLD_TICKS: u16 = 0xFF + 0xFF;
const ATTRACT_SCORING_LEGEND_ENTRY_COUNT: u16 = 6;
const ATTRACT_SCORING_RESCUE_SEQUENCE_TICKS: u16 = ATTRACT_SCORING_RESCUE_DESCENT_TICKS
    + ATTRACT_SCORING_RESCUE_ASCENT_TICKS
    + ATTRACT_SCORING_RESCUE_LASER_TICKS
    + ATTRACT_SCORING_RESCUE_FALL_TICKS
    + ATTRACT_SCORING_RESCUE_SCORE_TICKS
    + ATTRACT_SCORING_RESCUE_RETURN_TICKS;
const ATTRACT_SCORING_DEMO_TOTAL_TICKS: u16 = ATTRACT_SCORING_RESCUE_SEQUENCE_TICKS
    + (ATTRACT_SCORING_LEGEND_ENTRY_TICKS * ATTRACT_SCORING_LEGEND_ENTRY_COUNT)
    + ATTRACT_SCORING_LEGEND_HOLD_TICKS;
const ATTRACT_SCORING_PROTECTED_DEMO_TICK_OFFSET: u16 = ATTRACT_SCORING_RESCUE_DESCENT_TICKS
    + ATTRACT_SCORING_RESCUE_ASCENT_TICKS
    + ATTRACT_SCORING_RESCUE_LASER_TICKS
    + 5;
const ATTRACT_CYCLE_FRAME_COUNT: u16 =
    ATTRACT_SCORING_SEQUENCE_START_FRAME + ATTRACT_SCORING_DEMO_TOTAL_TICKS;
const ATTRACT_SCORING_PLAYER_X16: i32 = 0x0800;
const ATTRACT_SCORING_PLAYER_Y16: i32 = 0x5000;
const ATTRACT_SCORING_HUMAN_X16: i32 = 0x1E00;
const ATTRACT_SCORING_HUMAN_Y16: i32 = 0xDB00;
const ATTRACT_SCORING_LANDER_X16: i32 = 0x1DA0;
const ATTRACT_SCORING_LANDER_Y16: i32 = 0x4000;
const ATTRACT_SCORING_SCORE_500_X16: i32 = 0x1DFF;
const ATTRACT_SCORING_SCORE_500_Y16: i32 = 0x9000;
const ATTRACT_SCORING_SCORE_500_DROP_X16: i32 = 0x1C00;
const ATTRACT_SCORING_SCORE_500_DROP_Y16: i32 = 0xE000;
const ATTRACT_SCORING_CAUGHT_HUMAN_X16: i32 = 0x1E80;
const ATTRACT_SCORING_CAUGHT_HUMAN_Y16: i32 = 0xA2E0;
const ATTRACT_SCORING_GROUNDED_HUMAN_Y16: i32 = 0xDEE0;
const ATTRACT_SCORING_RESCUE_SHIP_XV16: i32 = 0x0040;
const ATTRACT_SCORING_RESCUE_SHIP_YV16: i32 = 0x00D4;
const ATTRACT_SCORING_RESCUE_HUMAN_ACCEL16: i32 = 0x0008;
const ATTRACT_SCORING_RESCUE_DROP_YV16: i32 = 0x00C0;
const ATTRACT_SCORING_RESCUE_RETURN_XV16: i32 = -0x0040;
const ATTRACT_SCORING_RESCUE_RETURN_YV16: i32 = -0x0180;
const ATTRACT_SCORING_LEGEND_SOURCE_X16: i32 = 0x1F00;
const ATTRACT_SCORING_LEGEND_SOURCE_START_Y16: i32 = 0xA000;
const SOURCE_COLTAB_COLOR_BYTES: [u8; 37] = [
    0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x37, 0x2F, 0x27, 0x1F, 0x17, 0x47, 0x47, 0x87,
    0x87, 0xC7, 0xC7, 0xC6, 0xC5, 0xCC, 0xCB, 0xCA, 0xDA, 0xE8, 0xF8, 0xF9, 0xFA, 0xFB, 0xFD, 0xFF,
    0xBF, 0x3F, 0x3E, 0x3C, 0x00,
];
const SOURCE_TIE_COLOR_BYTES: [u8; 9] = [0x81, 0x81, 0x2F, 0x81, 0x2F, 0x07, 0x2F, 0x81, 0x07];
const SOURCE_COLTAB_ACTIVE_BYTES: usize = SOURCE_COLTAB_COLOR_BYTES.len() - 1;
const ATTRACT_SCORING_REFERENCE_SAMPLE_STEP_FRAMES: u16 = 8;
const ATTRACT_SCORING_REFERENCE_TEXT_COLOR_INDICES: [u8; 226] = [
    29, 30, 30, 31, 31, 33, 33, 33, 34, 35, 0, 1, 2, 35, 5, 33, 8, 9, 11, 12, 14, 16, 16, 18, 18,
    20, 21, 22, 24, 25, 26, 28, 29, 30, 32, 33, 34, 0, 1, 3, 35, 5, 33, 8, 9, 9, 9, 9, 11, 12, 14,
    14, 16, 18, 18, 19, 21, 22, 24, 25, 26, 28, 29, 31, 32, 33, 35, 35, 35, 35, 0, 1, 3, 35, 5, 33,
    8, 9, 10, 12, 14, 14, 16, 18, 18, 20, 21, 22, 24, 25, 25, 25, 25, 27, 28, 29, 30, 32, 33, 35,
    0, 1, 2, 35, 5, 34, 8, 9, 10, 12, 14, 16, 16, 16, 16, 16, 18, 19, 20, 21, 22, 23, 25, 26, 28,
    29, 30, 32, 33, 34, 0, 1, 2, 35, 5, 34, 34, 34, 33, 8, 9, 11, 12, 14, 14, 16, 18, 18, 20, 21,
    22, 24, 25, 26, 28, 29, 31, 32, 32, 32, 32, 33, 35, 0, 1, 3, 35, 5, 34, 33, 9, 10, 11, 14, 14,
    16, 18, 18, 20, 21, 23, 24, 25, 27, 28, 29, 31, 32, 33, 35, 0, 1, 3, 35, 5, 34, 8, 9, 11, 12,
    14, 14, 16, 18, 19, 20, 22, 23, 24, 26, 27, 28, 30, 31, 32, 34, 35, 1, 2, 3, 35, 5, 33, 9, 10,
    11,
];
const SOURCE_ATTRACT_TITLE_TEXT_COLOR_FRAME_DIVISOR: u16 = 8;
const SOURCE_ATTRACT_TITLE_TEXT_COLOR_OFFSET: usize = 18;
const ATTRACT_TITLE_REFERENCE_SAMPLE_STEP_FRAMES: u16 = 8;
const ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR: u8 = u8::MAX;
const ATTRACT_TITLE_REFERENCE_LOGO_COLOR_BYTES: [u8; 59] = [
    0x00, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x2F, 0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x2F,
    0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07,
    0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x2F, 0x2F,
    0x07, 0x07, 0x07, 0x07, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x07, 0x2F,
];
const ATTRACT_TITLE_REFERENCE_TEXT_COLOR_INDICES: [u8; 59] = [
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR,
    11,
    12,
    13,
    13,
    15,
    17,
    17,
    20,
    21,
    23,
    24,
    25,
    26,
    28,
    29,
    30,
    31,
    7,
    6,
    0,
    1,
    2,
    3,
    5,
    6,
    7,
    9,
    10,
    11,
    12,
    12,
];
const SOURCE_ATTRACT_WILLIAMS_TIE_COLOR_PRIME_FRAMES: u16 = 6;
const SOURCE_ATTRACT_WILLIAMS_TIE_COLOR_SLEEP_FRAMES: u16 = 6;
const SOURCE_ATTRACT_WILLIAMS_TIE_COLOR_SLOT_OFFSET: usize = 2;
const ATTRACT_SCORING_PLAYFIELD_WIDTH: f32 = 320.0;
const ATTRACT_SCORING_PLAYFIELD_HEIGHT: f32 = 256.0;
const ATTRACT_REFERENCE_SCENE_OFFSET: [f32; 2] = [-11.0, -7.0];
const ATTRACT_HALL_OF_FAME_REFERENCE_OFFSET: [f32; 2] = [-11.0, -6.0];
const ATTRACT_SCORING_SCANNER_ORIGIN: [f32; 2] = [84.0, 0.0];
const ATTRACT_SCORING_SCANNER_SIZE: [f32; 2] = [128.0, 32.0];
const ATTRACT_SCORING_SCANNER_TERRAIN_PIXEL_SIZE: [f32; 2] = [1.0, 1.0];
const ATTRACT_SCORING_SCANNER_TERRAIN_TINT: Color = Color::from_rgba(174, 81, 0, 255);
const ATTRACT_SCORING_SCANNER_BORDER_TINT: Color = Color::from_rgba(38, 0, 160, 255);
const SOURCE_WILLIAMS_RED_GREEN_LEVELS: [u8; 8] = [0, 38, 81, 118, 137, 174, 217, 255];
const SOURCE_WILLIAMS_BLUE_LEVELS: [u8; 4] = [0, 95, 160, 255];
const SOURCE_TERRAIN_DATA_TSV: &str = include_str!("../assets/red-label/terrain-data.tsv");
const SOURCE_TERRAIN_TDATA_LABEL: &str = "TDATA";
const SOURCE_TERRAIN_TDATA_ADDRESS: u16 = 0xC350;
const SOURCE_TERRAIN_TDATA_BYTES: usize = 0x100;
const SOURCE_TERRAIN_FLAVOR_RECORDS: usize = 0x98;
const SOURCE_TERRAIN_SCREEN_WORDS: usize = 0x98;
const SOURCE_TERRAIN_WORD_7007: u16 = 0x7007;
const SOURCE_TERRAIN_WORD_0770: u16 = 0x0770;
const SOURCE_TERRAIN_WORD_SIZE: [f32; 2] = [4.0, 2.0];
const SOURCE_SCANNER_PROCESS_SLEEP_TICKS: [u8; 3] = [2, 2, 4];
const SOURCE_SCANNER_SELECTED_MAP: u8 = 1;
const SOURCE_SCANNER_OBJECT_BASE_SCREEN: u16 = 0x3008;
const SOURCE_SCANNER_SCAN_CENTER_OFFSET: u16 = 0x6D40;
const SOURCE_SCANNER_OBJECT_ERASE_START: u16 = 0xB05D;
const SOURCE_SCANNER_PLAYER_BASE_SCREEN: u16 = 0x4B07;
const SOURCE_SCANNER_PLAYER_BODY_WORD: u16 = 0x9099;
const SOURCE_SCANNER_PLAYER_TAIL_BYTE: u8 = 0x90;
const SOURCE_SCANNER_PLAYER_UPPER_BYTE: u8 = 0x09;
const SOURCE_SCANNER_LANDER_COLOR_WORD: u16 = 0x4433;
const SOURCE_SCANNER_HUMAN_COLOR_WORD: u16 = 0x6666;
pub(crate) const SOURCE_SCORE_POPUP_LIFETIME_TICKS: u8 = 50;
const SOURCE_SAFE_LANDING_SCORE_POINTS: u32 = 250;
const SOURCE_RESCUE_SCORE_POINTS: u32 = 500;
const SOURCE_RESCUE_SCORE_POPUP_Y_OFFSET: u8 = 0x18;
const SOURCE_PRHSND_SOUND_COMMAND: u8 = 0xFA;
const SOURCE_SCHSND_SOUND_COMMAND: u8 = 0xE8;
const SOURCE_UFHSND_SOUND_COMMAND: u8 = 0xF8;
const SOURCE_TIHSND_SOUND_COMMAND: u8 = 0xFE;
const SOURCE_LHSND_SOUND_COMMAND: u8 = 0xF9;
const SOURCE_SWHSND_SOUND_COMMAND: u8 = 0xF8;
const SOURCE_LSHSND_SOUND_COMMAND: u8 = 0xFC;
const SOURCE_SSHSND_SOUND_COMMAND: u8 = 0xF6;
const SOURCE_USHSND_SOUND_COMMAND: u8 = 0xFC;
const SOURCE_SWSSND_SOUND_COMMAND: u8 = 0xF3;
const SOURCE_LPKSND_SOUND_COMMAND: u8 = 0xF4;
const SOURCE_LSKSND_SOUND_COMMAND: u8 = 0xF1;
const SOURCE_LASSND_SOUND_COMMAND: u8 = 0xEB;
const SOURCE_SBSND_SOUND_COMMAND: u8 = 0xEE;
const SOURCE_PDSND_SOUND_COMMAND: u8 = 0xEE;
const SOURCE_APSND_SOUND_COMMAND: u8 = 0xEA;
const SOURCE_ACSND_SOUND_COMMAND: u8 = 0xF7;
const SOURCE_ALSND_SOUND_COMMAND: u8 = 0xE0;
const SOURCE_AHSND_SOUND_COMMAND: u8 = 0xEE;
const SOURCE_ASCSND_SOUND_COMMAND: u8 = 0xE5;
const SOURCE_TBSND_SOUND_COMMAND: u8 = 0xEB;
pub(crate) const SOURCE_EXPLOSION_INITIAL_SIZE: u16 = 0x0100;
pub(crate) const SOURCE_EXPLOSION_SIZE_DELTA: u16 = 0x00AA;
pub(crate) const SOURCE_EXPLOSION_KILL_SIZE_HIGH: u8 = 0x30;
pub(crate) const SOURCE_EXPLOSION_LIFETIME_FRAMES: u8 = 73;
pub(crate) const SOURCE_TERRAIN_BLOW_STATUS_BIT: u8 = 0x02;
pub(crate) const SOURCE_TERRAIN_BLOW_ITERATION_LIMIT: u8 = 16;
pub(crate) const SOURCE_TERRAIN_BLOW_EXPLOSIONS_PER_PASS: u8 = 2;
pub(crate) const SOURCE_TERRAIN_BLOW_SLEEP_TICKS: u8 = 2;
pub(crate) const SOURCE_TERRAIN_BLOW_OVERLOAD_COUNTER: u8 = 8;
pub(crate) const SOURCE_TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES: u16 = 0x98;
pub(crate) const SOURCE_TERRAIN_BLOW_SCANNER_ERASE_ENTRIES: u16 = 0x40;
const SOURCE_TERRAIN_BLOW_INITIAL_PSEUDO_COLOR: u8 = 0x3C;
const SOURCE_TERRAIN_BLOW_PSEUDO_COLORS: [u8; 32] = [
    0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x37, 0x2F, 0x27, 0x1F, 0x17, 0x47, 0x47, 0x87,
    0x87, 0xC7, 0xC7, 0xC6, 0xC5, 0xCC, 0xCB, 0xCA, 0xDA, 0xE8, 0xF8, 0xF9, 0xFA, 0xFB, 0xFD, 0xFF,
];
pub const PLAYER_EXPLOSION_PIECE_LIMIT: usize = 128;
const SOURCE_PLAYER_EXPLOSION_INITIAL_X_SEED: u16 = 0x0808;
const SOURCE_PLAYER_EXPLOSION_INITIAL_Y_SEED: u16 = 0x1732;
const SOURCE_PLAYER_EXPLOSION_INITIAL_COLOR_COUNTER: u8 = 56;
const SOURCE_PLAYER_EXPLOSION_COLOR_COUNTER_RELOAD: u8 = 4;
const SOURCE_PLAYER_EXPLOSION_Y_MIN: u8 = 42;
const SOURCE_PLAYER_EXPLOSION_X_MAX: u8 = 0x98;
const SOURCE_PLAYER_EXPLOSION_VELOCITY_LIMIT: u16 = 0x016A;
const SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD: u8 = 0xC0;
pub(crate) const SOURCE_PLAYER_EXPLOSION_COLORS: [u8; 15] = [
    0xFF, 0x7F, 0x3F, 0x37, 0x2F, 0x27, 0x1F, 0x17, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x00,
];

pub(crate) const SOURCE_VISUAL_STATE: SourceVisualStateSnapshot = SourceVisualStateSnapshot {
    attract_williams_status: 0xFB,
    attract_williams_logo_color_index: 0x3F,
    attract_copyright_williams_color_index: 0x0F,
    attract_williams_fast_logo_rate: 0xFF,
    attract_williams_normal_logo_rate: 10,
    attract_instruction_man_color_word: 0x6666,
    attract_instruction_ship_color_word: 0x0000,
    attract_instruction_enemy_color_word: 0x4433,
    hall_of_fame_display_letter_color_index: 0x47,
    hall_of_fame_logo_color_index: 0x3F,
    hall_of_fame_entry_letter_color_index: 0x85,
    hall_of_fame_blink_color_index: 0x85,
    hall_of_fame_blink_sleep_ticks: 15,
    hall_of_fame_underline_normal_word: 0x1111,
    hall_of_fame_underline_active_word: 0xDDDD,
    top_display_border_word: 0x5555,
    top_display_scanner_marker_word: 0x9999,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Attract,
    Playing,
    GameOver,
    HighScoreEntry,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AttractPresentationPage {
    #[default]
    Inactive,
    WilliamsLogo,
    Presents,
    DefenderWordmark,
    CopyrightWait,
    HallOfFame,
    ScoringSequence,
    Instruction,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AttractPresentationSnapshot {
    pub page_frame: u16,
    pub page: AttractPresentationPage,
    pub source_sleep_ticks: Option<u8>,
    pub source_stall_ticks: Option<u8>,
}

impl AttractPresentationSnapshot {
    pub const INACTIVE: Self = Self {
        page_frame: 0,
        page: AttractPresentationPage::Inactive,
        source_sleep_ticks: None,
        source_stall_ticks: None,
    };

    pub fn for_page_frame(page_frame: u16) -> Self {
        let page_frame = if ATTRACT_CYCLE_FRAME_COUNT == 0 {
            page_frame
        } else {
            page_frame % ATTRACT_CYCLE_FRAME_COUNT
        };
        let (page, source_sleep_ticks, source_stall_ticks) =
            if page_frame >= ATTRACT_SCORING_SEQUENCE_START_FRAME {
                (
                    AttractPresentationPage::ScoringSequence,
                    Some(ATTRACT_INSTRUCTION_ENTRY_SLEEP_TICKS),
                    None,
                )
            } else if page_frame >= ATTRACT_HALL_OF_FAME_START_FRAME {
                (
                    AttractPresentationPage::HallOfFame,
                    None,
                    Some(HALL_OF_FAME_STALL_FRAMES),
                )
            } else if page_frame >= ATTRACT_COPYRIGHT_START_FRAME {
                (
                    AttractPresentationPage::CopyrightWait,
                    Some(ATTRACT_COPYRIGHT_SLEEP_TICKS),
                    Some(ATTRACT_COPYRIGHT_STALL_TICKS),
                )
            } else if page_frame >= ATTRACT_DEFENDER_WORDMARK_START_FRAME {
                (
                    AttractPresentationPage::DefenderWordmark,
                    Some(ATTRACT_DEFENDER_ENTRY_SLEEP_TICKS),
                    None,
                )
            } else if page_frame >= ATTRACT_PRESENTS_START_FRAME {
                (
                    AttractPresentationPage::Presents,
                    Some(ATTRACT_PRESENTS_SLEEP_TICKS),
                    None,
                )
            } else {
                (
                    AttractPresentationPage::WilliamsLogo,
                    Some(ATTRACT_LOGO_SLEEP_TICKS),
                    None,
                )
            };

        Self {
            page_frame,
            page,
            source_sleep_ticks,
            source_stall_ticks,
        }
    }

    pub const fn shows_williams_logo(self) -> bool {
        matches!(
            self.page,
            AttractPresentationPage::WilliamsLogo
                | AttractPresentationPage::Presents
                | AttractPresentationPage::DefenderWordmark
                | AttractPresentationPage::CopyrightWait
        )
    }

    pub const fn shows_presents_text(self) -> bool {
        matches!(
            self.page,
            AttractPresentationPage::Presents
                | AttractPresentationPage::DefenderWordmark
                | AttractPresentationPage::CopyrightWait
        )
    }

    pub const fn shows_defender_wordmark(self) -> bool {
        matches!(
            self.page,
            AttractPresentationPage::DefenderWordmark | AttractPresentationPage::CopyrightWait
        )
    }

    pub const fn shows_copyright(self) -> bool {
        matches!(self.page, AttractPresentationPage::CopyrightWait)
    }

    pub const fn shows_hall_of_fame(self) -> bool {
        matches!(self.page, AttractPresentationPage::HallOfFame)
    }

    pub const fn shows_scoring_sequence_text(self) -> bool {
        matches!(
            self.page,
            AttractPresentationPage::ScoringSequence | AttractPresentationPage::Instruction
        )
    }

    pub const fn shows_instruction_text(self) -> bool {
        self.shows_scoring_sequence_text()
    }

    pub const fn scoring_sequence_frame(self) -> Option<u16> {
        if matches!(self.page, AttractPresentationPage::ScoringSequence) {
            Some(
                self.page_frame
                    .saturating_sub(ATTRACT_SCORING_SEQUENCE_START_FRAME),
            )
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceVisualStateSnapshot {
    pub(crate) attract_williams_status: u8,
    pub(crate) attract_williams_logo_color_index: u8,
    pub(crate) attract_copyright_williams_color_index: u16,
    pub(crate) attract_williams_fast_logo_rate: u8,
    pub(crate) attract_williams_normal_logo_rate: u8,
    pub(crate) attract_instruction_man_color_word: u16,
    pub(crate) attract_instruction_ship_color_word: u16,
    pub(crate) attract_instruction_enemy_color_word: u16,
    pub(crate) hall_of_fame_display_letter_color_index: u8,
    pub(crate) hall_of_fame_logo_color_index: u8,
    pub(crate) hall_of_fame_entry_letter_color_index: u8,
    pub(crate) hall_of_fame_blink_color_index: u8,
    pub(crate) hall_of_fame_blink_sleep_ticks: u8,
    pub(crate) hall_of_fame_underline_normal_word: u16,
    pub(crate) hall_of_fame_underline_active_word: u16,
    pub(crate) top_display_border_word: u16,
    pub(crate) top_display_scanner_marker_word: u16,
}

impl SourceVisualStateSnapshot {
    pub(crate) const fn hud_tint(self) -> Color {
        let _source_border_word = self.top_display_border_word;
        Color::WHITE
    }

    pub(crate) const fn top_display_border_tint(self) -> Color {
        let _source_word = self.top_display_border_word;
        Color::WHITE
    }

    pub(crate) const fn top_display_scanner_marker_tint(self) -> Color {
        let _source_word = self.top_display_scanner_marker_word;
        Color::WHITE
    }

    pub(crate) const fn scanner_object_blip_tint(self, source_color_word: u16) -> Color {
        let _source_words = source_color_word ^ self.top_display_scanner_marker_word;
        Color::WHITE
    }

    pub(crate) const fn scanner_player_blip_tint(self, body_word: u16) -> Color {
        let _source_words = body_word ^ self.top_display_scanner_marker_word;
        Color::WHITE
    }

    pub(crate) fn attract_title_text_tint_for_frame(self, page_frame: u16) -> Color {
        let _source_status = self.attract_williams_status;
        if let Some(color_index) = ATTRACT_TITLE_REFERENCE_TEXT_COLOR_INDICES
            .get(attract_title_reference_sample_index(page_frame))
            .copied()
            .filter(|index| *index != ATTRACT_TITLE_REFERENCE_NO_TEXT_COLOR)
        {
            return source_pseudo_color_tint(SOURCE_COLTAB_COLOR_BYTES[usize::from(color_index)]);
        }

        let color_index = (usize::from(page_frame / SOURCE_ATTRACT_TITLE_TEXT_COLOR_FRAME_DIVISOR)
            + SOURCE_ATTRACT_TITLE_TEXT_COLOR_OFFSET)
            % SOURCE_COLTAB_ACTIVE_BYTES;
        source_pseudo_color_tint(SOURCE_COLTAB_COLOR_BYTES[color_index])
    }

    pub(crate) const fn attract_instruction_text_tint(self, screen_address: u16) -> Color {
        let _source_words = screen_address
            ^ self.attract_instruction_man_color_word
            ^ self.attract_instruction_ship_color_word
            ^ self.attract_instruction_enemy_color_word;
        Color::WHITE
    }

    pub(crate) fn attract_williams_logo_tint_for_frame(self, page_frame: u16) -> Color {
        if let Some(color_byte) = ATTRACT_TITLE_REFERENCE_LOGO_COLOR_BYTES
            .get(attract_title_reference_sample_index(page_frame))
            .copied()
            .filter(|color_byte| *color_byte != 0)
        {
            return source_pseudo_color_tint(color_byte);
        }

        let source_rate_tick = page_frame
            .saturating_sub(SOURCE_ATTRACT_WILLIAMS_TIE_COLOR_PRIME_FRAMES)
            / SOURCE_ATTRACT_WILLIAMS_TIE_COLOR_SLEEP_FRAMES.max(1);
        let tie_triplet = usize::from(source_rate_tick % 3) * 3;
        source_pseudo_color_tint(
            SOURCE_TIE_COLOR_BYTES[tie_triplet + SOURCE_ATTRACT_WILLIAMS_TIE_COLOR_SLOT_OFFSET],
        )
    }

    pub(crate) const fn attract_williams_logo_should_render(self) -> bool {
        self.attract_williams_status == 0xFB
            && self.attract_williams_fast_logo_rate == 0xFF
            && self.attract_williams_normal_logo_rate == 10
    }

    pub(crate) fn attract_defender_wordmark_tint(self) -> Color {
        let _source_indices = self.attract_copyright_williams_color_index
            ^ u16::from(self.attract_williams_logo_color_index);
        Color::WHITE
    }

    pub(crate) const fn attract_copyright_tint(self) -> Color {
        let _source_index = self.attract_copyright_williams_color_index;
        Color::WHITE
    }

    pub(crate) const fn hall_of_fame_logo_tint(self) -> Color {
        let _source_index = self.hall_of_fame_logo_color_index;
        Color::WHITE
    }

    pub(crate) fn hall_of_fame_entry_text_tint(self) -> Color {
        source_pseudo_color_tint(self.hall_of_fame_entry_letter_color_index)
    }

    pub(crate) fn hall_of_fame_display_text_tint(self) -> Color {
        source_pseudo_color_tint(self.hall_of_fame_display_letter_color_index)
    }

    pub(crate) fn hall_of_fame_blink_text_tint(self) -> Color {
        let _source_sleep_ticks = self.hall_of_fame_blink_sleep_ticks;
        source_pseudo_color_tint(self.hall_of_fame_blink_color_index)
    }

    pub(crate) const fn hall_of_fame_active_underline_tint(self) -> Color {
        let _source_word = self.hall_of_fame_underline_active_word;
        Color::WHITE
    }

    pub(crate) const fn hall_of_fame_normal_underline_tint(self) -> Color {
        let _source_word = self.hall_of_fame_underline_normal_word;
        Color::from_rgba(0x66, 0x66, 0x66, 0xFF)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorldVector {
    subpixels: i32,
}

impl WorldVector {
    pub const SUBPIXELS_PER_PIXEL: i32 = 256;

    pub const fn from_subpixels(subpixels: i32) -> Self {
        Self { subpixels }
    }

    pub const fn subpixels(self) -> i32 {
        self.subpixels
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSnapshot {
    pub position: (WorldVector, WorldVector),
    pub velocity: (WorldVector, WorldVector),
    pub direction: Direction,
    pub lives: u8,
    pub smart_bombs: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerStockSnapshot {
    pub lives: u8,
    pub smart_bombs: u8,
}

impl PlayerStockSnapshot {
    pub const fn new(lives: u8, smart_bombs: u8) -> Self {
        Self { lives, smart_bombs }
    }

    const fn from_player(player: PlayerSnapshot) -> Self {
        Self {
            lives: player.lives,
            smart_bombs: player.smart_bombs,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreSnapshot {
    pub player_one: u32,
    pub player_two: u32,
    pub high_score: u32,
    pub next_bonus: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaveProfileSnapshot {
    pub landers: u8,
    pub bombers: u8,
    pub pods: u8,
    pub mutants: u8,
    pub swarmers: u8,
    pub lander_x_velocity: u8,
    pub lander_y_velocity_msb: u8,
    pub lander_y_velocity_lsb: u8,
    pub mutant_random_y: u8,
    pub mutant_y_velocity_msb: u8,
    pub mutant_y_velocity_lsb: u8,
    pub mutant_x_velocity: u8,
    pub swarmer_x_velocity: u8,
    pub wave_time: u32,
    pub wave_size: u8,
    pub lander_shot_time: u32,
    pub bomber_x_velocity: u8,
    pub mutant_shot_time: u32,
    pub swarmer_shot_time: u32,
    pub swarmer_acceleration_mask: u8,
    pub baiter_delay: u32,
    pub baiter_shot_time: u32,
    pub baiter_seek_probability: u8,
}

impl WaveProfileSnapshot {
    pub fn for_wave(wave: u8) -> Self {
        Self {
            landers: wave_table_u8("landers", wave),
            bombers: wave_table_u8("bombers", wave),
            pods: wave_table_u8("pods", wave),
            mutants: wave_table_u8("mutants", wave),
            swarmers: wave_table_u8("swarmers", wave),
            lander_x_velocity: wave_table_u8("lander_x_velocity", wave),
            lander_y_velocity_msb: wave_table_u8("lander_y_velocity_msb", wave),
            lander_y_velocity_lsb: wave_table_u8("lander_y_velocity_lsb", wave),
            mutant_random_y: wave_table_u8("mutant_random_y", wave),
            mutant_y_velocity_msb: wave_table_u8("mutant_y_velocity_msb", wave),
            mutant_y_velocity_lsb: wave_table_u8("mutant_y_velocity_lsb", wave),
            mutant_x_velocity: wave_table_u8("mutant_x_velocity", wave),
            swarmer_x_velocity: wave_table_u8("swarmer_x_velocity", wave),
            wave_time: wave_table_u32("wave_time", wave),
            wave_size: wave_table_u8("wave_size", wave),
            lander_shot_time: wave_table_u32("lander_shot_time", wave),
            bomber_x_velocity: wave_table_u8("bomber_x_velocity", wave),
            mutant_shot_time: wave_table_u32("mutant_shot_time", wave),
            swarmer_shot_time: wave_table_u32("swarmer_shot_time", wave),
            swarmer_acceleration_mask: wave_table_u8("swarmer_acceleration_mask", wave),
            baiter_delay: wave_table_u32("baiter_time", wave),
            baiter_shot_time: wave_table_u32("baiter_shot_time", wave),
            baiter_seek_probability: wave_table_u8("baiter_seek_probability", wave),
        }
    }
}

const SOURCE_WAVE_TABLE_TSV: &str = include_str!("../assets/red-label/wave-table.tsv");
const SOURCE_WAVE_TABLE_HEADER: &str =
    "key\tceiling\tfloor\tintra_delta\tinter_delta\twave1\twave2\twave3\twave4";

fn wave_table_u8(key: &str, wave: u8) -> u8 {
    u8::try_from(wave_table_value(key, wave)).expect("red-label wave profile value should fit u8")
}

fn wave_table_u32(key: &str, wave: u8) -> u32 {
    u32::try_from(wave_table_value(key, wave))
        .expect("red-label wave profile value should be non-negative")
}

fn wave_table_value(key: &str, wave: u8) -> i32 {
    let mut lines = SOURCE_WAVE_TABLE_TSV.lines();
    let header = lines
        .next()
        .expect("red-label wave table should have header");
    assert_eq!(header, SOURCE_WAVE_TABLE_HEADER);

    for line in lines.map(str::trim).filter(|line| !line.is_empty()) {
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 9, "red-label wave table row width changed");
        if fields[0] != key {
            continue;
        }

        let ceiling = parse_wave_table_i32(fields[1], key, "ceiling");
        let floor = parse_wave_table_i32(fields[2], key, "floor");
        let inter_delta = parse_wave_table_i32(fields[4], key, "inter_delta");
        let wave_index = usize::from(wave.clamp(1, 4));
        let mut value = parse_wave_table_i32(fields[4 + wave_index], key, "wave");
        for _ in 0..wave.saturating_sub(4) {
            value = apply_wave_table_delta(value, inter_delta, floor, ceiling);
        }
        return value;
    }

    panic!("missing red-label wave table key {key}");
}

fn parse_wave_table_i32(value: &str, key: &str, field: &str) -> i32 {
    value
        .parse()
        .unwrap_or_else(|_| panic!("red-label wave table {key}.{field} is not an integer"))
}

fn apply_wave_table_delta(value: i32, delta: i32, floor: i32, ceiling: i32) -> i32 {
    if delta > 0 {
        (value + delta).min(ceiling)
    } else if delta < 0 {
        (value + delta).max(floor)
    } else {
        value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Lander,
    Mutant,
    Bomber,
    Pod,
    Swarmer,
    Baiter,
}

impl EnemyKind {
    const fn object_category(self) -> ObjectEvidenceCategory {
        match self {
            Self::Lander => ObjectEvidenceCategory::Lander,
            Self::Mutant => ObjectEvidenceCategory::Mutant,
            Self::Bomber => ObjectEvidenceCategory::Bomber,
            Self::Pod => ObjectEvidenceCategory::Pod,
            Self::Swarmer => ObjectEvidenceCategory::Swarmer,
            Self::Baiter => ObjectEvidenceCategory::Baiter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemySnapshot {
    pub kind: EnemyKind,
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
    pub source_lander: Option<SourceLanderSnapshot>,
    pub source_mutant: Option<SourceMutantSnapshot>,
    pub source_bomber: Option<SourceBomberSnapshot>,
    pub source_swarmer: Option<SourceSwarmerSnapshot>,
    pub source_baiter: Option<SourceBaiterSnapshot>,
    pub source_pod: Option<SourcePodSnapshot>,
}

impl EnemySnapshot {
    pub const fn new(kind: EnemyKind, position: ScreenPosition, velocity: ScreenVelocity) -> Self {
        Self {
            kind,
            position,
            velocity,
            source_lander: None,
            source_mutant: None,
            source_bomber: None,
            source_swarmer: None,
            source_baiter: None,
            source_pod: None,
        }
    }

    const fn source_lander(
        position: ScreenPosition,
        velocity: ScreenVelocity,
        source_lander: SourceLanderSnapshot,
    ) -> Self {
        Self {
            kind: EnemyKind::Lander,
            position,
            velocity,
            source_lander: Some(source_lander),
            source_mutant: None,
            source_bomber: None,
            source_swarmer: None,
            source_baiter: None,
            source_pod: None,
        }
    }

    const fn source_mutant(
        position: ScreenPosition,
        velocity: ScreenVelocity,
        source_mutant: SourceMutantSnapshot,
    ) -> Self {
        Self {
            kind: EnemyKind::Mutant,
            position,
            velocity,
            source_lander: None,
            source_mutant: Some(source_mutant),
            source_bomber: None,
            source_swarmer: None,
            source_baiter: None,
            source_pod: None,
        }
    }

    const fn source_bomber(
        position: ScreenPosition,
        velocity: ScreenVelocity,
        source_bomber: SourceBomberSnapshot,
    ) -> Self {
        Self {
            kind: EnemyKind::Bomber,
            position,
            velocity,
            source_lander: None,
            source_mutant: None,
            source_bomber: Some(source_bomber),
            source_swarmer: None,
            source_baiter: None,
            source_pod: None,
        }
    }

    const fn source_swarmer(
        position: ScreenPosition,
        velocity: ScreenVelocity,
        source_swarmer: SourceSwarmerSnapshot,
    ) -> Self {
        Self {
            kind: EnemyKind::Swarmer,
            position,
            velocity,
            source_lander: None,
            source_mutant: None,
            source_bomber: None,
            source_swarmer: Some(source_swarmer),
            source_baiter: None,
            source_pod: None,
        }
    }

    const fn source_baiter(
        position: ScreenPosition,
        velocity: ScreenVelocity,
        source_baiter: SourceBaiterSnapshot,
    ) -> Self {
        Self {
            kind: EnemyKind::Baiter,
            position,
            velocity,
            source_lander: None,
            source_mutant: None,
            source_bomber: None,
            source_swarmer: None,
            source_baiter: Some(source_baiter),
            source_pod: None,
        }
    }

    const fn source_pod(
        position: ScreenPosition,
        velocity: ScreenVelocity,
        source_pod: SourcePodSnapshot,
    ) -> Self {
        Self {
            kind: EnemyKind::Pod,
            position,
            velocity,
            source_lander: None,
            source_mutant: None,
            source_bomber: None,
            source_swarmer: None,
            source_baiter: None,
            source_pod: Some(source_pod),
        }
    }

    fn source_picture_descriptor(self) -> SourceObjectPictureDescriptor {
        match self.kind {
            EnemyKind::Lander => source_lander_picture_descriptor(
                self.source_lander
                    .map(|source| source.picture_frame)
                    .unwrap_or_default(),
            ),
            EnemyKind::Mutant => SOURCE_MUTANT_PICTURE_DESCRIPTOR,
            EnemyKind::Bomber => source_bomber_picture_descriptor(
                self.source_bomber
                    .map(|source| source.picture_frame)
                    .unwrap_or_default(),
            ),
            EnemyKind::Pod => SOURCE_POD_PICTURE_DESCRIPTOR,
            EnemyKind::Swarmer => SOURCE_SWARMER_PICTURE_DESCRIPTOR,
            EnemyKind::Baiter => source_baiter_picture_descriptor(
                self.source_baiter
                    .map(|source| source.picture_frame)
                    .unwrap_or_default(),
            ),
        }
    }

    fn source_world_position(self) -> (u16, u16) {
        match self.kind {
            EnemyKind::Lander => self
                .source_lander
                .map(|source| {
                    source_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| source_world_position(self.position, 0, 0)),
            EnemyKind::Mutant => self
                .source_mutant
                .map(|source| {
                    source_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| source_world_position(self.position, 0, 0)),
            EnemyKind::Bomber => self
                .source_bomber
                .map(|source| {
                    source_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| source_world_position(self.position, 0, 0)),
            EnemyKind::Pod => self
                .source_pod
                .map(|source| {
                    source_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| source_world_position(self.position, 0, 0)),
            EnemyKind::Swarmer => self
                .source_swarmer
                .map(|source| {
                    source_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| source_world_position(self.position, 0, 0)),
            EnemyKind::Baiter => self
                .source_baiter
                .map(|source| {
                    source_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| source_world_position(self.position, 0, 0)),
        }
    }

    fn source_velocity_words(self) -> (u16, u16) {
        match self.kind {
            EnemyKind::Lander => self
                .source_lander
                .map(|source| (source.x_velocity, source.y_velocity))
                .unwrap_or_else(|| source_fixed_velocity_words(self.velocity)),
            EnemyKind::Mutant => self
                .source_mutant
                .map(|source| (source.x_velocity, source.y_velocity))
                .unwrap_or_else(|| source_fixed_velocity_words(self.velocity)),
            EnemyKind::Bomber => self
                .source_bomber
                .map(|source| (source.x_velocity, source.y_velocity))
                .unwrap_or_else(|| source_fixed_velocity_words(self.velocity)),
            EnemyKind::Pod => self
                .source_pod
                .map(|source| (source.x_velocity, source.y_velocity))
                .unwrap_or_else(|| source_fixed_velocity_words(self.velocity)),
            EnemyKind::Swarmer => self
                .source_swarmer
                .map(|source| (source.x_velocity, source.y_velocity))
                .unwrap_or_else(|| source_fixed_velocity_words(self.velocity)),
            EnemyKind::Baiter => self
                .source_baiter
                .map(|source| {
                    (
                        source_baiter_screen_x_velocity(source.x_velocity),
                        source.y_velocity,
                    )
                })
                .unwrap_or_else(|| source_fixed_velocity_words(self.velocity)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLanderSnapshot {
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
pub struct SourceMutantSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceBomberSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub picture_frame: u8,
    pub cruise_altitude: u8,
    pub sleep_ticks: u8,
    pub source_slot: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSwarmerSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub acceleration: u8,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub horizontal_seek_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceBaiterSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub picture_frame: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePodSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRandSnapshot {
    pub seed: u8,
    pub hseed: u8,
    pub lseed: u8,
}

impl Default for SourceRandSnapshot {
    fn default() -> Self {
        Self {
            seed: 0,
            hseed: 0xA5,
            lseed: 0x5A,
        }
    }
}

impl SourceRandSnapshot {
    fn advance(&mut self) -> Self {
        let product_low = self.seed.wrapping_mul(3).wrapping_add(17);
        let mut a = self.lseed >> 3;
        a ^= self.lseed;
        let carry_into_hseed = (a & 0x01) != 0;
        let old_hseed = self.hseed;
        self.hseed = (u8::from(carry_into_hseed) << 7) | (self.hseed >> 1);
        let carry_into_lseed = (old_hseed & 0x01) != 0;
        self.lseed = (u8::from(carry_into_lseed) << 7) | (self.lseed >> 1);
        let (with_lseed, carry) = adc8(product_low, self.lseed, false);
        let (new_seed, _) = adc8(with_lseed, self.hseed, carry);
        self.seed = new_seed;
        *self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyProjectileSourceKind {
    Fireball,
    BomberBombShell,
}

impl EnemyProjectileSourceKind {
    pub const fn output_routine_address(self) -> u16 {
        match self {
            Self::Fireball => SOURCE_FIREBALL_OUTPUT_ROUTINE_ADDRESS,
            Self::BomberBombShell => SOURCE_BOMB_SHELL_OUTPUT_ROUTINE_ADDRESS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyProjectileSnapshot {
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
    pub source_kind: EnemyProjectileSourceKind,
    pub source_x_fraction: u8,
    pub source_y_fraction: u8,
    pub source_x_velocity: u16,
    pub source_y_velocity: u16,
    pub source_lifetime_ticks: u8,
}

impl EnemyProjectileSnapshot {
    fn source_fireball(position: ScreenPosition, x_velocity: u16, y_velocity: u16) -> Self {
        Self {
            position,
            velocity: source_screen_velocity(x_velocity, y_velocity),
            source_kind: EnemyProjectileSourceKind::Fireball,
            source_x_fraction: 0,
            source_y_fraction: 0,
            source_x_velocity: x_velocity,
            source_y_velocity: y_velocity,
            source_lifetime_ticks: SOURCE_SHELL_LIFETIME_TICKS,
        }
    }

    fn source_bomb_shell(position: ScreenPosition) -> Self {
        Self {
            source_kind: EnemyProjectileSourceKind::BomberBombShell,
            ..Self::source_fireball(position, 0, 0)
        }
    }

    pub const fn source_output_routine_address(self) -> u16 {
        self.source_kind.output_routine_address()
    }

    const fn is_source_bomb_shell(self) -> bool {
        matches!(self.source_kind, EnemyProjectileSourceKind::BomberBombShell)
    }

    const fn source_bomb_picture_label(self) -> &'static str {
        SOURCE_BOMB_SHELL_PICTURE_LABEL
    }

    fn source_world_position(self) -> (u16, u16) {
        source_world_position(
            self.position,
            self.source_x_fraction,
            self.source_y_fraction,
        )
    }

    const fn source_velocity_words(self) -> (u16, u16) {
        (self.source_x_velocity, self.source_y_velocity)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnemyReserveSnapshot {
    pub landers: u8,
    pub bombers: u8,
    pub pods: u8,
    pub mutants: u8,
    pub swarmers: u8,
}

impl EnemyReserveSnapshot {
    const fn from_profile(profile: WaveProfileSnapshot) -> Self {
        Self {
            landers: profile.landers,
            bombers: profile.bombers,
            pods: profile.pods,
            mutants: profile.mutants,
            swarmers: profile.swarmers,
        }
    }

    fn total(self) -> u8 {
        self.landers
            .saturating_add(self.bombers)
            .saturating_add(self.pods)
            .saturating_add(self.mutants)
            .saturating_add(self.swarmers)
    }

    fn is_empty(self) -> bool {
        self.total() == 0
    }

    fn source_family_counts(self) -> [(EnemyKind, u8); 5] {
        [
            (EnemyKind::Lander, self.landers),
            (EnemyKind::Bomber, self.bombers),
            (EnemyKind::Pod, self.pods),
            (EnemyKind::Mutant, self.mutants),
            (EnemyKind::Swarmer, self.swarmers),
        ]
    }

    fn take(&mut self, kind: EnemyKind) -> bool {
        let remaining = match kind {
            EnemyKind::Lander => &mut self.landers,
            EnemyKind::Bomber => &mut self.bombers,
            EnemyKind::Pod => &mut self.pods,
            EnemyKind::Mutant => &mut self.mutants,
            EnemyKind::Swarmer => &mut self.swarmers,
            EnemyKind::Baiter => return false,
        };
        if *remaining == 0 {
            return false;
        }
        *remaining = remaining.saturating_sub(1);
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HumanSnapshot {
    pub position: ScreenPosition,
    pub carried: bool,
    pub carried_by_player: bool,
    pub source_x_fraction: u8,
    pub source_picture_frame: u8,
    pub source_fall_velocity: u16,
    pub source_fall_y_fraction: u8,
    pub source_target_slot_address: Option<u16>,
}

impl HumanSnapshot {
    pub const fn new(position: ScreenPosition) -> Self {
        Self {
            position,
            carried: false,
            carried_by_player: false,
            source_x_fraction: 0,
            source_picture_frame: 0,
            source_fall_velocity: 0,
            source_fall_y_fraction: 0,
            source_target_slot_address: None,
        }
    }

    fn clear_source_fall(&mut self) {
        self.source_fall_velocity = 0;
        self.source_fall_y_fraction = 0;
    }

    fn source_world_position(self) -> (u16, u16) {
        source_world_position(
            self.position,
            self.source_x_fraction,
            self.source_fall_y_fraction,
        )
    }

    fn source_velocity_words(self) -> (u16, u16) {
        (0, self.source_fall_velocity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectileSnapshot {
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
}

impl ProjectileSnapshot {
    fn source_world_position(self) -> (u16, u16) {
        source_world_position(self.position, 0, 0)
    }

    fn source_velocity_words(self) -> (u16, u16) {
        source_fixed_velocity_words(self.velocity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainSegment {
    pub position: ScreenPosition,
    pub size: (u8, u8),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TerrainBlowStage {
    #[default]
    ExplosionPassSleeping,
    FlashClearedSleeping,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainBlowSnapshot {
    pub stage: TerrainBlowStage,
    pub status_terrain_blown: bool,
    pub source_iteration: u8,
    pub source_iteration_limit: u8,
    pub source_sleep_remaining: Option<u8>,
    pub source_pseudo_color: u8,
    pub source_overload_counter: u8,
    pub terrain_erase_entries: u16,
    pub scanner_terrain_erase_entries: u16,
    pub terrain_words_remaining: u16,
    pub scanner_terrain_words_remaining: u16,
    pub explosions_per_pass: u8,
}

impl TerrainBlowSnapshot {
    pub const EMPTY: Self = Self {
        stage: TerrainBlowStage::ExplosionPassSleeping,
        status_terrain_blown: false,
        source_iteration: 0,
        source_iteration_limit: SOURCE_TERRAIN_BLOW_ITERATION_LIMIT,
        source_sleep_remaining: None,
        source_pseudo_color: 0,
        source_overload_counter: 0,
        terrain_erase_entries: 0,
        scanner_terrain_erase_entries: 0,
        terrain_words_remaining: 0,
        scanner_terrain_words_remaining: 0,
        explosions_per_pass: 0,
    };

    pub fn source_started() -> Self {
        Self {
            stage: TerrainBlowStage::ExplosionPassSleeping,
            status_terrain_blown: SOURCE_TERRAIN_BLOW_STATUS_BIT != 0,
            source_iteration: 0,
            source_iteration_limit: SOURCE_TERRAIN_BLOW_ITERATION_LIMIT,
            source_sleep_remaining: Some(SOURCE_TERRAIN_BLOW_SLEEP_TICKS),
            source_pseudo_color: SOURCE_TERRAIN_BLOW_INITIAL_PSEUDO_COLOR,
            source_overload_counter: SOURCE_TERRAIN_BLOW_OVERLOAD_COUNTER,
            terrain_erase_entries: SOURCE_TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES,
            scanner_terrain_erase_entries: SOURCE_TERRAIN_BLOW_SCANNER_ERASE_ENTRIES,
            terrain_words_remaining: 0,
            scanner_terrain_words_remaining: 0,
            explosions_per_pass: SOURCE_TERRAIN_BLOW_EXPLOSIONS_PER_PASS,
        }
    }

    pub const fn terrain_erased(self) -> bool {
        self.status_terrain_blown && self.terrain_words_remaining == 0
    }

    pub const fn scanner_terrain_erased(self) -> bool {
        self.status_terrain_blown && self.scanner_terrain_words_remaining == 0
    }
}

pub const OBJECT_EVIDENCE_DETAIL_LIMIT: usize = 16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ObjectEvidenceList {
    #[default]
    Active,
    Inactive,
    Projectile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectEvidenceCategory {
    Lander,
    Mutant,
    Bomber,
    Pod,
    Swarmer,
    Baiter,
    Human,
    PlayerProjectile,
    EnemyBomb,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObjectEvidenceDetailSnapshot {
    pub list: ObjectEvidenceList,
    pub object_category: Option<ObjectEvidenceCategory>,
    pub address: Option<u16>,
    pub slot: Option<u16>,
    pub screen_position: Option<ScreenPosition>,
    pub world_position: Option<(u16, u16)>,
    pub velocity: Option<(u16, u16)>,
    pub picture_address: Option<u16>,
    pub picture_label: Option<&'static str>,
    pub picture_size: Option<(u8, u8)>,
    pub primary_image_address: Option<u16>,
    pub alternate_image_address: Option<u16>,
    pub mapped_sprite: Option<SpriteId>,
    pub object_type: Option<u8>,
    pub scanner_color: Option<u16>,
}

impl ObjectEvidenceDetailSnapshot {
    pub const EMPTY: Self = Self {
        list: ObjectEvidenceList::Active,
        object_category: None,
        address: None,
        slot: None,
        screen_position: None,
        world_position: None,
        velocity: None,
        picture_address: None,
        picture_label: None,
        picture_size: None,
        primary_image_address: None,
        alternate_image_address: None,
        mapped_sprite: None,
        object_type: None,
        scanner_color: None,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObjectEvidenceSnapshot {
    pub active_count: u16,
    pub inactive_count: u16,
    pub projectile_count: u16,
    pub visible_count: u16,
    pub evidence_crc32: Option<u32>,
    pub detail_count: u8,
    pub details: [ObjectEvidenceDetailSnapshot; OBJECT_EVIDENCE_DETAIL_LIMIT],
}

pub const SCANNER_RADAR_BLIP_LIMIT: usize = OBJECT_EVIDENCE_DETAIL_LIMIT;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScannerRadarStage {
    #[default]
    InactiveObjectScan,
    ActiveAndShellScan,
    RasterDisplay,
}

impl ScannerRadarStage {
    const fn source_sleep_ticks(self) -> u8 {
        match self {
            Self::InactiveObjectScan => SOURCE_SCANNER_PROCESS_SLEEP_TICKS[0],
            Self::ActiveAndShellScan => SOURCE_SCANNER_PROCESS_SLEEP_TICKS[1],
            Self::RasterDisplay => SOURCE_SCANNER_PROCESS_SLEEP_TICKS[2],
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScannerRadarBlipKind {
    #[default]
    ActiveObject,
    InactiveObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerRadarBlipSnapshot {
    pub kind: ScannerRadarBlipKind,
    pub object_address: Option<u16>,
    pub erase_table_address: u16,
    pub screen_address: u16,
    pub color_word: u16,
}

impl ScannerRadarBlipSnapshot {
    pub const EMPTY: Self = Self {
        kind: ScannerRadarBlipKind::ActiveObject,
        object_address: None,
        erase_table_address: 0,
        screen_address: 0,
        color_word: 0,
    };
}

impl Default for ScannerRadarBlipSnapshot {
    fn default() -> Self {
        Self::EMPTY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerRadarPlayerBlipSnapshot {
    pub erase_table_address: u16,
    pub screen_address: u16,
    pub body_word: u16,
    pub tail_byte: u8,
    pub upper_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerRadarSnapshot {
    pub enabled: bool,
    pub stage: ScannerRadarStage,
    pub stage_sleep_ticks: u8,
    pub source_process_sleep_ticks: [u8; 3],
    pub selected_map: u8,
    pub scan_left: Option<u16>,
    pub terrain_enabled: bool,
    pub object_erase_start: u16,
    pub setend: u16,
    pub blip_count: u8,
    pub blips: [ScannerRadarBlipSnapshot; SCANNER_RADAR_BLIP_LIMIT],
    pub player_blip: Option<ScannerRadarPlayerBlipSnapshot>,
}

impl ScannerRadarSnapshot {
    pub const DISABLED: Self = Self {
        enabled: false,
        stage: ScannerRadarStage::InactiveObjectScan,
        stage_sleep_ticks: 0,
        source_process_sleep_ticks: SOURCE_SCANNER_PROCESS_SLEEP_TICKS,
        selected_map: 0,
        scan_left: None,
        terrain_enabled: false,
        object_erase_start: SOURCE_SCANNER_OBJECT_ERASE_START,
        setend: SOURCE_SCANNER_OBJECT_ERASE_START,
        blip_count: 0,
        blips: [ScannerRadarBlipSnapshot::EMPTY; SCANNER_RADAR_BLIP_LIMIT],
        player_blip: None,
    };

    pub(crate) fn for_world(
        phase: GamePhase,
        frame: u64,
        scan_anchor: WorldVector,
        player_position: (WorldVector, WorldVector),
        object_evidence: &ObjectEvidenceSnapshot,
    ) -> Self {
        if phase != GamePhase::Playing
            || player_position == (WorldVector::default(), WorldVector::default())
        {
            return Self::DISABLED;
        }

        let stage = scanner_radar_stage_for_frame(frame);
        let scan_anchor_word = source_word_from_world_vector(scan_anchor);
        let scan_left = scan_anchor_word.wrapping_sub(SOURCE_SCANNER_SCAN_CENTER_OFFSET);
        let mut scanner = Self {
            enabled: true,
            stage,
            stage_sleep_ticks: stage.source_sleep_ticks(),
            source_process_sleep_ticks: SOURCE_SCANNER_PROCESS_SLEEP_TICKS,
            selected_map: SOURCE_SCANNER_SELECTED_MAP,
            scan_left: Some(scan_left),
            terrain_enabled: true,
            object_erase_start: SOURCE_SCANNER_OBJECT_ERASE_START,
            setend: SOURCE_SCANNER_OBJECT_ERASE_START,
            blip_count: 0,
            blips: [ScannerRadarBlipSnapshot::EMPTY; SCANNER_RADAR_BLIP_LIMIT],
            player_blip: None,
        };

        scanner.push_object_blips(object_evidence, scan_left);
        scanner.player_blip = Some(ScannerRadarPlayerBlipSnapshot {
            erase_table_address: scanner.setend,
            screen_address: scanner_radar_player_screen_address(player_position),
            body_word: SOURCE_SCANNER_PLAYER_BODY_WORD,
            tail_byte: SOURCE_SCANNER_PLAYER_TAIL_BYTE,
            upper_byte: SOURCE_SCANNER_PLAYER_UPPER_BYTE,
        });
        scanner
    }

    fn push_object_blips(&mut self, object_evidence: &ObjectEvidenceSnapshot, scan_left: u16) {
        let detail_count =
            usize::from(object_evidence.detail_count).min(OBJECT_EVIDENCE_DETAIL_LIMIT);
        for detail in &object_evidence.details[..detail_count] {
            let Some(kind) = scanner_radar_blip_kind(detail.list) else {
                continue;
            };
            let Some(color_word) = detail.scanner_color else {
                continue;
            };
            let Some((world_x, world_y)) = scanner_radar_object_world_position(detail) else {
                continue;
            };
            let index = usize::from(self.blip_count);
            if index >= SCANNER_RADAR_BLIP_LIMIT {
                return;
            }

            self.blips[index] = ScannerRadarBlipSnapshot {
                kind,
                object_address: detail.address,
                erase_table_address: self.setend,
                screen_address: scanner_radar_object_screen_address(world_x, world_y, scan_left),
                color_word,
            };
            self.blip_count += 1;
            self.setend = self.setend.wrapping_add(2);
        }
    }
}

impl Default for ScannerRadarSnapshot {
    fn default() -> Self {
        Self::DISABLED
    }
}

pub const EXPANDED_OBJECT_DETAIL_LIMIT: usize = 16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExpandedObjectKind {
    #[default]
    Appearance,
    Explosion,
    ScorePopup,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExpandedObjectDetailSnapshot {
    pub kind: ExpandedObjectKind,
    pub slot_address: Option<u16>,
    pub size: u16,
    pub descriptor_address: Option<u16>,
    pub picture_label: Option<&'static str>,
    pub picture_size: Option<(u8, u8)>,
    pub mapped_sprite: Option<SpriteId>,
    pub erase_address: Option<u16>,
    pub center: Option<ScreenPosition>,
    pub top_left: Option<ScreenPosition>,
    pub object_address: Option<u16>,
    pub score_popup_lifetime_ticks: Option<u8>,
    pub score_popup_value: Option<u16>,
    pub explosion_frame: Option<u8>,
    pub explosion_lifetime_frames: Option<u8>,
}

impl ExpandedObjectDetailSnapshot {
    pub const EMPTY: Self = Self {
        kind: ExpandedObjectKind::Appearance,
        slot_address: None,
        size: 0,
        descriptor_address: None,
        picture_label: None,
        picture_size: None,
        mapped_sprite: None,
        erase_address: None,
        center: None,
        top_left: None,
        object_address: None,
        score_popup_lifetime_ticks: None,
        score_popup_value: None,
        explosion_frame: None,
        explosion_lifetime_frames: None,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExpandedObjectEvidenceSnapshot {
    pub active_count: u16,
    pub last_slot_address: Option<u16>,
    pub detail_count: u8,
    pub details: [ExpandedObjectDetailSnapshot; EXPANDED_OBJECT_DETAIL_LIMIT],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScorePopupKind {
    Points250,
    Points500,
}

impl ScorePopupKind {
    const fn value(self) -> u16 {
        match self {
            Self::Points250 => 250,
            Self::Points500 => 500,
        }
    }

    const fn picture_label(self) -> &'static str {
        match self {
            Self::Points250 => "C25P1",
            Self::Points500 => "C5P1",
        }
    }

    const fn sprite(self) -> SpriteId {
        match self {
            Self::Points250 => SpriteId::SCORE_POPUP_250,
            Self::Points500 => SpriteId::SCORE_POPUP_500,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScorePopupSnapshot {
    pub kind: ScorePopupKind,
    pub position: ScreenPosition,
    pub frames_remaining: u8,
    pub source_lifetime_ticks: u8,
}

impl ScorePopupSnapshot {
    pub fn source_spawn(kind: ScorePopupKind, position: ScreenPosition) -> Self {
        Self {
            kind,
            position,
            frames_remaining: SOURCE_SCORE_POPUP_LIFETIME_TICKS,
            source_lifetime_ticks: SOURCE_SCORE_POPUP_LIFETIME_TICKS,
        }
    }

    fn expanded_object_detail(self) -> ExpandedObjectDetailSnapshot {
        ExpandedObjectDetailSnapshot {
            kind: ExpandedObjectKind::ScorePopup,
            picture_label: Some(self.kind.picture_label()),
            picture_size: Some((6, 6)),
            mapped_sprite: Some(self.kind.sprite()),
            top_left: Some(self.position),
            score_popup_lifetime_ticks: Some(self.source_lifetime_ticks),
            score_popup_value: Some(self.kind.value()),
            ..ExpandedObjectDetailSnapshot::EMPTY
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionKind {
    Lander,
    Mutant,
    Bomber,
    Pod,
    Baiter,
    Bomb,
    Swarmer,
    Astronaut,
    PlayerShip,
    Terrain,
}

impl ExplosionKind {
    const fn for_enemy(kind: EnemyKind) -> Self {
        match kind {
            EnemyKind::Lander => Self::Lander,
            EnemyKind::Mutant => Self::Mutant,
            EnemyKind::Bomber => Self::Bomber,
            EnemyKind::Pod => Self::Pod,
            EnemyKind::Swarmer => Self::Swarmer,
            EnemyKind::Baiter => Self::Baiter,
        }
    }

    const fn picture_label(self) -> &'static str {
        match self {
            Self::Lander => "LNDP1",
            Self::Mutant => "SCZP1",
            Self::Bomber => "TIEP1",
            Self::Pod => "PRBP1",
            Self::Baiter => "UFOP1",
            Self::Bomb => "BXPIC",
            Self::Swarmer => "SWXP1",
            Self::Astronaut => "ASXP1",
            Self::PlayerShip => "PLAPIC",
            Self::Terrain => "TEREX",
        }
    }

    const fn picture_size(self) -> (u8, u8) {
        match self {
            Self::Lander | Self::Mutant => (5, 8),
            Self::Bomber | Self::Pod => (4, 8),
            Self::Baiter => (6, 4),
            Self::Bomb | Self::Swarmer | Self::Astronaut => (4, 8),
            Self::PlayerShip => (8, 6),
            Self::Terrain => (8, 6),
        }
    }

    const fn sprite(self) -> SpriteId {
        match self {
            Self::Lander => SpriteId::ENEMY_LANDER,
            Self::Mutant => SpriteId::ENEMY_MUTANT,
            Self::Bomber => SpriteId::ENEMY_BOMBER,
            Self::Pod => SpriteId::ENEMY_POD,
            Self::Baiter => SpriteId::ENEMY_BAITER,
            Self::Bomb => SpriteId::BOMB_EXPLOSION,
            Self::Swarmer => SpriteId::SWARMER_EXPLOSION,
            Self::Astronaut => SpriteId::ASTRONAUT_EXPLOSION,
            Self::PlayerShip => SpriteId::PLAYER_SHIP,
            Self::Terrain => SpriteId::TERRAIN_EXPLOSION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExplosionSnapshot {
    pub kind: ExplosionKind,
    pub position: ScreenPosition,
    pub source_size: u16,
    pub frames_remaining: u8,
}

impl ExplosionSnapshot {
    pub fn source_spawn(kind: ExplosionKind, position: ScreenPosition) -> Self {
        Self {
            kind,
            position,
            source_size: SOURCE_EXPLOSION_INITIAL_SIZE,
            frames_remaining: SOURCE_EXPLOSION_LIFETIME_FRAMES,
        }
    }

    fn advance_source_frame(&mut self) -> bool {
        let next_size = self.source_size.wrapping_add(SOURCE_EXPLOSION_SIZE_DELTA);
        if next_size.to_be_bytes()[0] > SOURCE_EXPLOSION_KILL_SIZE_HIGH {
            return false;
        }

        self.source_size = next_size;
        self.frames_remaining = self.frames_remaining.saturating_sub(1);
        true
    }

    fn expanded_object_detail(self) -> ExpandedObjectDetailSnapshot {
        let (width, height) = self.kind.picture_size();
        ExpandedObjectDetailSnapshot {
            kind: ExpandedObjectKind::Explosion,
            size: self.source_size,
            picture_label: Some(self.kind.picture_label()),
            picture_size: Some((width, height)),
            mapped_sprite: Some(self.kind.sprite()),
            center: Some(ScreenPosition::new(
                self.position.x.wrapping_add(width / 2),
                self.position.y.wrapping_add(height / 2),
            )),
            top_left: Some(self.position),
            explosion_frame: source_explosion_frame_index(self.source_size),
            explosion_lifetime_frames: Some(SOURCE_EXPLOSION_LIFETIME_FRAMES),
            ..ExpandedObjectDetailSnapshot::EMPTY
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerExplosionPieceSnapshot {
    pub position: ScreenPosition,
    pub split: bool,
}

impl PlayerExplosionPieceSnapshot {
    pub const EMPTY: Self = Self {
        position: ScreenPosition::new(0, 0),
        split: false,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerExplosionCloudSnapshot {
    pub source_color: u8,
    pub source_color_counter: u8,
    pub source_color_index: u8,
    pub frame: u16,
    pub piece_count: u8,
    pub pieces: [PlayerExplosionPieceSnapshot; PLAYER_EXPLOSION_PIECE_LIMIT],
}

impl PlayerExplosionCloudSnapshot {
    pub const EMPTY: Self = Self {
        source_color: 0,
        source_color_counter: 0,
        source_color_index: 0,
        frame: 0,
        piece_count: 0,
        pieces: [PlayerExplosionPieceSnapshot::EMPTY; PLAYER_EXPLOSION_PIECE_LIMIT],
    };

    pub(crate) fn push_piece(&mut self, piece: PlayerExplosionPieceSnapshot) {
        let index = usize::from(self.piece_count);
        if index >= PLAYER_EXPLOSION_PIECE_LIMIT {
            return;
        }
        self.pieces[index] = piece;
        self.piece_count += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlayerExplosionRuntimePiece {
    x_position: u16,
    y_position: u16,
    x_velocity: u16,
    y_velocity: u16,
    visible: bool,
    split: bool,
}

impl PlayerExplosionRuntimePiece {
    const EMPTY: Self = Self {
        x_position: 0,
        y_position: 0,
        x_velocity: 0,
        y_velocity: 0,
        visible: false,
        split: false,
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerExplosionRuntime {
    source_color_index: u8,
    source_color_counter: u8,
    frame: u16,
    x_seed: u16,
    y_seed: u16,
    pieces: [PlayerExplosionRuntimePiece; PLAYER_EXPLOSION_PIECE_LIMIT],
}

impl PlayerExplosionRuntime {
    fn source_spawn(center: ScreenPosition) -> Self {
        let mut runtime = Self {
            source_color_index: 0,
            source_color_counter: SOURCE_PLAYER_EXPLOSION_INITIAL_COLOR_COUNTER,
            frame: 0,
            x_seed: SOURCE_PLAYER_EXPLOSION_INITIAL_X_SEED,
            y_seed: SOURCE_PLAYER_EXPLOSION_INITIAL_Y_SEED,
            pieces: [PlayerExplosionRuntimePiece::EMPTY; PLAYER_EXPLOSION_PIECE_LIMIT],
        };

        for index in 0..PLAYER_EXPLOSION_PIECE_LIMIT {
            runtime.pieces[index] = runtime.source_piece(center);
        }

        runtime
    }

    fn source_piece(&mut self, center: ScreenPosition) -> PlayerExplosionRuntimePiece {
        loop {
            let x_position = u16::from_be_bytes([center.x, 0]);
            let y_position = u16::from_be_bytes([center.y, 0]);
            let x_velocity = self.next_x_velocity();
            let y_velocity = self.next_y_velocity();
            let absolute_x_velocity = ones_complement_abs_word(x_velocity);
            let half_absolute_y_velocity =
                logical_shift_right_word(ones_complement_abs_word(y_velocity));

            if absolute_x_velocity.wrapping_add(half_absolute_y_velocity)
                < SOURCE_PLAYER_EXPLOSION_VELOCITY_LIMIT
            {
                return PlayerExplosionRuntimePiece {
                    x_position,
                    y_position,
                    x_velocity,
                    y_velocity,
                    visible: false,
                    split: false,
                };
            }
        }
    }

    fn next_x_velocity(&mut self) -> u16 {
        let next_seed = player_explosion_random_seed_step(self.x_seed);
        self.x_seed = next_seed;
        let velocity_high = (next_seed.to_be_bytes()[0] & 1).wrapping_sub(1);
        u16::from_be_bytes([velocity_high, next_seed.to_be_bytes()[1]])
    }

    fn next_y_velocity(&mut self) -> u16 {
        let next_seed = player_explosion_random_seed_step(self.y_seed);
        self.y_seed = next_seed;
        let velocity_high = (next_seed.to_be_bytes()[0] & 3).wrapping_sub(2);
        u16::from_be_bytes([velocity_high, next_seed.to_be_bytes()[1]])
    }

    fn source_color(&self) -> u8 {
        SOURCE_PLAYER_EXPLOSION_COLORS
            .get(usize::from(self.source_color_index))
            .copied()
            .unwrap_or(0)
    }

    fn advance_source_frame(&mut self) -> bool {
        if self.source_color() == 0 {
            return false;
        }

        for piece in &mut self.pieces {
            piece.visible = false;

            let next_y = piece.y_velocity.wrapping_add(piece.y_position);
            if next_y.to_be_bytes()[0] < SOURCE_PLAYER_EXPLOSION_Y_MIN {
                continue;
            }
            piece.y_position = next_y;

            let next_x = piece.x_velocity.wrapping_add(piece.x_position);
            if next_x.to_be_bytes()[0] > SOURCE_PLAYER_EXPLOSION_X_MAX {
                continue;
            }
            piece.x_position = next_x;
            piece.visible = true;
            piece.split = next_x.to_be_bytes()[1] & 0x80 != 0;
        }

        self.frame = self.frame.saturating_add(1);
        let color_counter = self.source_color_counter.saturating_sub(1);
        if color_counter == 0 {
            self.source_color_index = self.source_color_index.saturating_add(1);
            self.source_color_counter = SOURCE_PLAYER_EXPLOSION_COLOR_COUNTER_RELOAD;
        } else {
            self.source_color_counter = color_counter;
        }

        true
    }

    fn snapshot(&self) -> Option<PlayerExplosionCloudSnapshot> {
        let source_color = self.source_color();
        if source_color == 0 {
            return None;
        }

        let mut snapshot = PlayerExplosionCloudSnapshot {
            source_color,
            source_color_counter: self.source_color_counter,
            source_color_index: self.source_color_index,
            frame: self.frame,
            ..PlayerExplosionCloudSnapshot::EMPTY
        };

        for piece in &self.pieces {
            if !piece.visible {
                continue;
            }
            snapshot.push_piece(PlayerExplosionPieceSnapshot {
                position: ScreenPosition::from_packed(u16::from_be_bytes([
                    piece.x_position.to_be_bytes()[0],
                    piece.y_position.to_be_bytes()[0],
                ])),
                split: piece.split,
            });
        }

        Some(snapshot)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldSnapshot {
    pub terrain: Vec<TerrainSegment>,
    pub terrain_blow: Option<TerrainBlowSnapshot>,
    pub stars: Vec<ScreenPosition>,
    pub enemies: Vec<EnemySnapshot>,
    pub enemy_reserve: EnemyReserveSnapshot,
    pub humans: Vec<HumanSnapshot>,
    pub source_target_list_cursor_address: Option<u16>,
    pub source_astronaut_cursor_address: Option<u16>,
    pub source_astronaut_sleep_ticks: u8,
    pub projectiles: Vec<ProjectileSnapshot>,
    pub enemy_projectiles: Vec<EnemyProjectileSnapshot>,
    pub score_popups: Vec<ScorePopupSnapshot>,
    pub explosions: Vec<ExplosionSnapshot>,
    pub source_rng: SourceRandSnapshot,
    pub object_evidence: ObjectEvidenceSnapshot,
    pub expanded_objects: ExpandedObjectEvidenceSnapshot,
    pub player_explosion: Option<PlayerExplosionCloudSnapshot>,
    pub scanner: ScannerRadarSnapshot,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct FallingHumanAdvance {
    safe_landings: Vec<ScreenPosition>,
    fatal_landings: Vec<ScreenPosition>,
}

impl WorldSnapshot {
    fn first_wave() -> Self {
        Self::for_wave(1)
    }

    fn for_wave(wave: u8) -> Self {
        let wave_profile = WaveProfileSnapshot::for_wave(wave);
        let mut enemy_reserve = EnemyReserveSnapshot::from_profile(wave_profile);
        let mut source_rng = SourceRandSnapshot::default();
        let humans = source_initial_target_list_humans();
        let mut source_target_list_cursor_address = Some(SOURCE_TARGET_LIST_BASE);
        let enemies = clean_wave_enemy_spawns(
            &mut enemy_reserve,
            wave_profile,
            &mut source_rng,
            CleanWaveSpawnContext::Initial,
            &humans,
            &mut source_target_list_cursor_address,
        );
        let mut world = Self {
            terrain: vec![
                TerrainSegment {
                    position: ScreenPosition::new(0, 224),
                    size: (64, 8),
                },
                TerrainSegment {
                    position: ScreenPosition::new(64, 222),
                    size: (64, 8),
                },
                TerrainSegment {
                    position: ScreenPosition::new(128, 226),
                    size: (64, 8),
                },
                TerrainSegment {
                    position: ScreenPosition::new(192, 220),
                    size: (56, 8),
                },
                TerrainSegment {
                    position: ScreenPosition::new(248, 224),
                    size: (44, 8),
                },
            ],
            stars: vec![
                ScreenPosition::new(24, 32),
                ScreenPosition::new(112, 56),
                ScreenPosition::new(236, 24),
            ],
            enemies,
            enemy_reserve,
            humans,
            source_target_list_cursor_address,
            source_astronaut_cursor_address: Some(SOURCE_TARGET_LIST_BASE),
            source_astronaut_sleep_ticks: 0,
            projectiles: Vec::new(),
            enemy_projectiles: Vec::new(),
            score_popups: Vec::new(),
            explosions: Vec::new(),
            source_rng,
            terrain_blow: None,
            object_evidence: ObjectEvidenceSnapshot::default(),
            expanded_objects: ExpandedObjectEvidenceSnapshot::default(),
            player_explosion: None,
            scanner: ScannerRadarSnapshot::default(),
        };
        world.refresh_object_evidence();
        world
    }

    fn activate_enemy_reserve_batch(
        &mut self,
        profile: WaveProfileSnapshot,
        player_absolute_x: u16,
        background_absolute_x: u16,
    ) -> bool {
        if self.active_source_wave_enemy_count() != 0 || self.enemy_reserve.is_empty() {
            return false;
        }

        let mut active_baiters = self
            .enemies
            .iter()
            .copied()
            .filter(|enemy| enemy.kind == EnemyKind::Baiter)
            .collect::<Vec<_>>();
        let targetable_humans = self
            .humans
            .iter()
            .any(|human| !human.carried && !human.carried_by_player);
        let mut reserve_batch = clean_wave_enemy_spawns(
            &mut self.enemy_reserve,
            profile,
            &mut self.source_rng,
            CleanWaveSpawnContext::ReserveActivation {
                player_absolute_x,
                background_absolute_x,
                targetable_humans,
            },
            &self.humans,
            &mut self.source_target_list_cursor_address,
        );
        active_baiters.append(&mut reserve_batch);
        self.enemies = active_baiters;
        !self.enemies.is_empty()
    }

    fn spawn_pod_swarmers(
        &mut self,
        position: ScreenPosition,
        profile: WaveProfileSnapshot,
    ) -> usize {
        let active_swarmers = self
            .enemies
            .iter()
            .filter(|enemy| enemy.kind == EnemyKind::Swarmer)
            .count();
        let spawn_count = SOURCE_POD_SWARMER_REQUEST_LIMIT
            .min(SOURCE_ACTIVE_SWARMER_LIMIT.saturating_sub(active_swarmers));

        for _ in 0..spawn_count {
            let source_swarmer = source_mini_swarmer_spawn(&mut self.source_rng, profile, position);
            self.enemies.push(EnemySnapshot::source_swarmer(
                position,
                source_screen_velocity(source_swarmer.x_velocity, source_swarmer.y_velocity),
                source_swarmer,
            ));
        }

        spawn_count
    }

    fn advance_source_astronaut_process(&mut self) -> bool {
        if self.source_astronaut_sleep_ticks > 0 {
            self.source_astronaut_sleep_ticks = self.source_astronaut_sleep_ticks.saturating_sub(1);
            return false;
        }

        let current_cursor = self
            .source_astronaut_cursor_address
            .unwrap_or(SOURCE_TARGET_LIST_BASE);
        let next_cursor = source_astronaut_next_cursor_address(current_cursor);
        self.source_astronaut_cursor_address = Some(next_cursor);
        self.source_astronaut_sleep_ticks = SOURCE_ASTRONAUT_PROCESS_SLEEP_TICKS;

        let Some(human) = self.humans.iter_mut().find(|human| {
            human.source_target_slot_address == Some(next_cursor)
                && source_lander_targetable_human(human)
        }) else {
            return false;
        };

        source_walk_astronaut(human, &self.terrain, self.source_rng.seed);
        true
    }

    fn source_enemy_total(&self) -> usize {
        self.active_source_wave_enemy_count()
            .saturating_add(usize::from(self.enemy_reserve.total()))
    }

    fn active_source_wave_enemy_count(&self) -> usize {
        self.enemies
            .iter()
            .filter(|enemy| enemy.kind != EnemyKind::Baiter)
            .count()
    }

    fn clear_source_non_wave_enemies(&mut self) {
        self.enemies.retain(|enemy| enemy.kind != EnemyKind::Baiter);
    }

    fn active_enemy_count(&self, kind: EnemyKind) -> usize {
        self.enemies
            .iter()
            .filter(|enemy| enemy.kind == kind)
            .count()
    }

    fn advance_enemies(
        &mut self,
        profile: WaveProfileSnapshot,
        player_position: ScreenPosition,
        player_velocity: (WorldVector, WorldVector),
        sound_events: &mut Vec<SoundEvent>,
    ) {
        let source_rng = &mut self.source_rng;
        let enemy_projectiles = &mut self.enemy_projectiles;
        let terrain = &self.terrain;
        let humans = &mut self.humans;
        let source_target_list_cursor_address = &mut self.source_target_list_cursor_address;
        let selected_bomber_slot = source_tie_selected_slot(source_rng.seed);
        for enemy in &mut self.enemies {
            if enemy.kind == EnemyKind::Lander
                && let Some(source_lander) = enemy.source_lander.as_mut()
            {
                let carried_human_index = clean_carried_human_index_for_source_lander(
                    humans,
                    enemy.position,
                    *source_lander,
                );
                let starts_pull_sound = carried_human_index.is_some()
                    && source_lander_pull_edge(enemy.position)
                    && source_lander.y_velocity != 0;
                let target_position = if carried_human_index.is_none() {
                    source_lander_ensure_live_target(
                        source_lander,
                        humans,
                        source_target_list_cursor_address,
                    )
                } else {
                    None
                };
                let grab_target = target_position.filter(|target_position| {
                    carried_human_index.is_none()
                        && (source_lander_grab_active(*source_lander)
                            || source_lander_grab_x_matches(enemy.position, *target_position))
                });
                let projectile_count = enemy_projectiles.len();
                let advance = advance_source_lander(
                    enemy.position,
                    source_lander,
                    SourceLanderAdvanceContext {
                        profile,
                        terrain,
                        carrying_passenger: carried_human_index.is_some(),
                        grab_target,
                        player_position,
                        player_velocity,
                        source_rng,
                        enemy_projectiles,
                    },
                );
                push_source_enemy_shot_sound_if_fired(
                    EnemyKind::Lander,
                    projectile_count,
                    enemy_projectiles.len(),
                    sound_events,
                );
                if let Some(position) = advance.direct_position {
                    enemy.position = position;
                } else {
                    let (x, x_fraction) = source_fixed_axis_step(
                        enemy.position.x,
                        source_lander.x_fraction,
                        source_lander.x_velocity,
                    );
                    let (y, y_fraction) = source_active_object_y_step(
                        enemy.position.y,
                        source_lander.y_fraction,
                        source_lander.y_velocity,
                    );
                    enemy.position = ScreenPosition::new(x, y);
                    source_lander.x_fraction = x_fraction;
                    source_lander.y_fraction = y_fraction;
                }
                enemy.velocity = source_lander_screen_velocity(*source_lander);
                if let Some(human_index) = carried_human_index {
                    if advance.phase == SourceLanderAdvancePhase::PullingPassenger
                        || clean_lander_pull_position_matches(enemy.position, humans[human_index])
                    {
                        if advance.phase == SourceLanderAdvancePhase::PullingPassenger {
                            if starts_pull_sound {
                                sound_events.push(source_lander_suck_sound_event());
                            }
                            humans[human_index].position = clean_lander_pull_passenger_position(
                                enemy.position,
                                humans[human_index].position,
                            );
                        }
                    } else {
                        humans[human_index].position = clean_carried_human_position(enemy.position);
                    }
                    humans[human_index].clear_source_fall();
                }
                continue;
            }
            if enemy.kind == EnemyKind::Pod
                && let Some(source_pod) = enemy.source_pod.as_mut()
            {
                let (x, x_fraction) = source_fixed_axis_step(
                    enemy.position.x,
                    source_pod.x_fraction,
                    source_pod.x_velocity,
                );
                let (y, y_fraction) = source_active_object_y_step(
                    enemy.position.y,
                    source_pod.y_fraction,
                    source_pod.y_velocity,
                );
                enemy.position = ScreenPosition::new(x, y);
                source_pod.x_fraction = x_fraction;
                source_pod.y_fraction = y_fraction;
                enemy.velocity = source_pod_screen_velocity(*source_pod);
                continue;
            }
            if enemy.kind == EnemyKind::Bomber
                && let Some(source_bomber) = enemy.source_bomber.as_mut()
            {
                let should_run_tie_step =
                    usize::from(source_bomber.source_slot) == selected_bomber_slot;
                if should_run_tie_step {
                    advance_source_bomber(
                        enemy.position,
                        source_bomber,
                        player_position,
                        *source_rng,
                        enemy_projectiles,
                    );
                }
                let (x, x_fraction) = source_fixed_axis_step(
                    enemy.position.x,
                    source_bomber.x_fraction,
                    source_bomber.x_velocity,
                );
                let (y, y_fraction) = source_active_object_y_step(
                    enemy.position.y,
                    source_bomber.y_fraction,
                    source_bomber.y_velocity,
                );
                enemy.position = ScreenPosition::new(x, y);
                source_bomber.x_fraction = x_fraction;
                source_bomber.y_fraction = y_fraction;
                enemy.velocity = source_bomber_screen_velocity(*source_bomber);
                continue;
            }
            if enemy.kind == EnemyKind::Mutant
                && let Some(source_mutant) = enemy.source_mutant.as_mut()
            {
                let projectile_count = enemy_projectiles.len();
                advance_source_mutant(
                    &mut enemy.position,
                    source_mutant,
                    profile,
                    player_position,
                    player_velocity,
                    source_rng,
                    enemy_projectiles,
                );
                push_source_enemy_shot_sound_if_fired(
                    EnemyKind::Mutant,
                    projectile_count,
                    enemy_projectiles.len(),
                    sound_events,
                );
                let (x, x_fraction) = source_fixed_axis_step(
                    enemy.position.x,
                    source_mutant.x_fraction,
                    source_mutant.x_velocity,
                );
                let (y, y_fraction) = source_active_object_y_step(
                    enemy.position.y,
                    source_mutant.y_fraction,
                    source_mutant.y_velocity,
                );
                enemy.position = ScreenPosition::new(x, y);
                source_mutant.x_fraction = x_fraction;
                source_mutant.y_fraction = y_fraction;
                enemy.velocity = source_mutant_screen_velocity(*source_mutant);
                continue;
            }
            if enemy.kind == EnemyKind::Swarmer
                && let Some(source_swarmer) = enemy.source_swarmer.as_mut()
            {
                let projectile_count = enemy_projectiles.len();
                advance_source_mini_swarmer(
                    enemy.position,
                    source_swarmer,
                    profile,
                    player_position,
                    source_rng,
                    enemy_projectiles,
                );
                push_source_enemy_shot_sound_if_fired(
                    EnemyKind::Swarmer,
                    projectile_count,
                    enemy_projectiles.len(),
                    sound_events,
                );
                let (x, x_fraction) = source_fixed_axis_step(
                    enemy.position.x,
                    source_swarmer.x_fraction,
                    source_swarmer.x_velocity,
                );
                let (y, y_fraction) = source_active_object_y_step(
                    enemy.position.y,
                    source_swarmer.y_fraction,
                    source_swarmer.y_velocity,
                );
                enemy.position = ScreenPosition::new(x, y);
                source_swarmer.x_fraction = x_fraction;
                source_swarmer.y_fraction = y_fraction;
                enemy.velocity =
                    source_screen_velocity(source_swarmer.x_velocity, source_swarmer.y_velocity);
                continue;
            }
            if enemy.kind == EnemyKind::Baiter
                && let Some(source_baiter) = enemy.source_baiter.as_mut()
            {
                let projectile_count = enemy_projectiles.len();
                advance_source_baiter(
                    enemy.position,
                    source_baiter,
                    profile,
                    player_position,
                    player_velocity,
                    source_rng,
                    enemy_projectiles,
                );
                push_source_enemy_shot_sound_if_fired(
                    EnemyKind::Baiter,
                    projectile_count,
                    enemy_projectiles.len(),
                    sound_events,
                );
                let (x, x_fraction) = source_fixed_axis_step(
                    enemy.position.x,
                    source_baiter.x_fraction,
                    source_baiter_screen_x_velocity(source_baiter.x_velocity),
                );
                let (y, y_fraction) = source_active_object_y_step(
                    enemy.position.y,
                    source_baiter.y_fraction,
                    source_baiter.y_velocity,
                );
                enemy.position = ScreenPosition::new(x, y);
                source_baiter.x_fraction = x_fraction;
                source_baiter.y_fraction = y_fraction;
                enemy.velocity = source_baiter_screen_velocity(*source_baiter);
                continue;
            }

            let motion = EnemyMotionSystem::step(enemy.position, enemy.velocity);
            enemy.position = motion.position;
            enemy.velocity = motion.velocity;
        }
    }

    fn spawn_baiter(
        &mut self,
        profile: WaveProfileSnapshot,
        player_position: ScreenPosition,
        player_velocity: (WorldVector, WorldVector),
    ) -> bool {
        let active_baiters = self.active_enemy_count(EnemyKind::Baiter);
        if active_baiters >= SOURCE_ACTIVE_BAITER_LIMIT {
            return false;
        }
        let (position, source_baiter) =
            source_baiter_spawn(self.source_rng, profile, player_position, player_velocity);

        self.enemies.push(EnemySnapshot::source_baiter(
            position,
            source_baiter_screen_velocity(source_baiter),
            source_baiter,
        ));
        true
    }

    fn resolve_lander_human_abductions(
        &mut self,
        profile: WaveProfileSnapshot,
        player_position: ScreenPosition,
        player_velocity: (WorldVector, WorldVector),
        sound_events: &mut Vec<SoundEvent>,
    ) {
        self.sync_carried_humans_to_landers();
        self.restore_landers_with_cleared_pull_targets();
        self.convert_completed_lander_abductions(profile, player_position, player_velocity);
        if self.humans.is_empty() {
            self.convert_all_active_landers_to_mutants(profile, player_position, player_velocity);
            return;
        }

        let Some((lander_index, human_index)) = self.first_lander_human_capture() else {
            return;
        };

        let lander_position = start_lander_human_capture(&mut self.enemies[lander_index], profile);
        if let Some(source_lander) = self.enemies[lander_index].source_lander.as_mut() {
            source_lander.target_human_index = Some(human_index);
        }
        self.humans[human_index].carried = true;
        self.humans[human_index].position = clean_carried_human_position(lander_position);
        self.humans[human_index].clear_source_fall();
        sound_events.push(source_lander_pickup_sound_event());
    }

    fn restore_landers_with_cleared_pull_targets(&mut self) {
        let humans = &self.humans;
        let mut restored = 0u8;
        self.enemies.retain(|enemy| {
            let gave_up = source_lander_pull_target_cleared(enemy, humans);
            if gave_up {
                restored = restored.saturating_add(1);
            }
            !gave_up
        });
        self.enemy_reserve.landers = self.enemy_reserve.landers.saturating_add(restored);
    }

    fn convert_completed_lander_abductions(
        &mut self,
        profile: WaveProfileSnapshot,
        player_position: ScreenPosition,
        player_velocity: (WorldVector, WorldVector),
    ) {
        while let Some((lander_index, human_index)) = self.completed_lander_abduction() {
            self.humans.remove(human_index);
            self.reindex_source_lander_targets_after_human_removed(human_index);
            self.convert_lander_to_source_mutant(
                lander_index,
                profile,
                player_position,
                player_velocity,
            );
        }
    }

    fn reindex_source_lander_targets_after_human_removed(&mut self, removed_index: usize) {
        for source_lander in self
            .enemies
            .iter_mut()
            .filter_map(|enemy| enemy.source_lander.as_mut())
        {
            source_lander.target_human_index = match source_lander.target_human_index {
                Some(target_index) if target_index == removed_index => None,
                Some(target_index) if target_index > removed_index => Some(target_index - 1),
                target_index => target_index,
            };
        }
    }

    fn completed_lander_abduction(&self) -> Option<(usize, usize)> {
        for (lander_index, lander) in self
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, enemy)| enemy.kind == EnemyKind::Lander)
        {
            if lander.position.y > SOURCE_PLAYFIELD_Y_MIN.saturating_add(8) {
                continue;
            }
            let is_source_lander = lander.source_lander.is_some();
            if let Some(human_index) = self.humans.iter().position(|human| {
                if is_source_lander {
                    clean_lander_passenger_ready_for_conversion(lander.position, *human)
                } else {
                    human.carried
                        && !human.carried_by_player
                        && human.position == clean_carried_human_position(lander.position)
                }
            }) {
                return Some((lander_index, human_index));
            }
        }

        None
    }

    fn convert_all_active_landers_to_mutants(
        &mut self,
        profile: WaveProfileSnapshot,
        player_position: ScreenPosition,
        player_velocity: (WorldVector, WorldVector),
    ) {
        let lander_indices = self
            .enemies
            .iter()
            .enumerate()
            .filter_map(|(index, enemy)| (enemy.kind == EnemyKind::Lander).then_some(index))
            .collect::<Vec<_>>();

        for lander_index in lander_indices {
            self.convert_lander_to_source_mutant(
                lander_index,
                profile,
                player_position,
                player_velocity,
            );
        }
    }

    fn convert_lander_to_source_mutant(
        &mut self,
        lander_index: usize,
        profile: WaveProfileSnapshot,
        player_position: ScreenPosition,
        player_velocity: (WorldVector, WorldVector),
    ) {
        let mut position = self.enemies[lander_index].position;
        let mut source_mutant = source_mutant_from_lander_conversion(profile);
        advance_source_mutant(
            &mut position,
            &mut source_mutant,
            profile,
            player_position,
            player_velocity,
            &mut self.source_rng,
            &mut self.enemy_projectiles,
        );
        self.enemies[lander_index] = EnemySnapshot::source_mutant(
            position,
            source_mutant_screen_velocity(source_mutant),
            source_mutant,
        );
    }

    fn advance_falling_humans(&mut self) -> FallingHumanAdvance {
        let terrain = &self.terrain;
        let mut advance = FallingHumanAdvance::default();
        self.humans.retain_mut(|human| {
            if human.carried || human.carried_by_player {
                human.clear_source_fall();
                return true;
            }

            let Some(ground_y) = clean_human_ground_y(terrain, human.position.x) else {
                return true;
            };
            if human.position.y >= ground_y {
                human.clear_source_fall();
                return true;
            }

            human.source_fall_velocity =
                source_falling_human_y_velocity(human.source_fall_velocity);
            let y16 = (u16::from(human.position.y) << 8) | u16::from(human.source_fall_y_fraction);
            let next_y16 = y16.saturating_add(human.source_fall_velocity);
            let next_y = ((next_y16 >> 8) as u8).min(ground_y);
            human.position.y = next_y;
            human.source_fall_y_fraction = if next_y == ground_y {
                0
            } else {
                (next_y16 & 0x00FF) as u8
            };

            if human.position.y < ground_y {
                return true;
            }

            let landing_position = human.position;
            if human.source_fall_velocity <= SOURCE_FALLING_HUMAN_SAFE_LANDING_MAX_Y_VELOCITY {
                human.clear_source_fall();
                advance.safe_landings.push(landing_position);
                true
            } else {
                advance.fatal_landings.push(landing_position);
                false
            }
        });
        advance
    }

    fn sync_player_carried_humans(&mut self, player_position: ScreenPosition) {
        let carried_position = clean_player_carried_human_position(player_position);
        for human in &mut self.humans {
            if human.carried_by_player {
                if let Some(ground_y) = clean_human_ground_y(&self.terrain, carried_position.x)
                    && carried_position.y >= ground_y
                {
                    human.position = ScreenPosition::new(carried_position.x, ground_y);
                    human.carried_by_player = false;
                    human.clear_source_fall();
                } else {
                    human.position = carried_position;
                    human.clear_source_fall();
                }
            }
        }
    }

    fn resolve_player_human_rescue(&mut self, player_position: ScreenPosition) -> bool {
        let player = CollisionBox::new(player_position, PLAYER_COLLISION_SIZE);
        let Some(human_index) = self.humans.iter().position(|human| {
            clean_human_is_falling(&self.terrain, *human)
                && player.overlaps(CollisionBox::new(
                    human.position,
                    human_collision_size(*human),
                ))
        }) else {
            return false;
        };

        let caught_position = self.humans[human_index].position;
        self.humans[human_index].carried_by_player = true;
        self.humans[human_index].position = clean_player_carried_human_position(player_position);
        self.humans[human_index].clear_source_fall();
        self.spawn_score_popup(
            ScorePopupKind::Points500,
            clean_rescue_score_popup_position(caught_position),
        );
        true
    }

    fn sync_carried_humans_to_landers(&mut self) {
        let lander_positions = self
            .enemies
            .iter()
            .filter(|enemy| enemy.kind == EnemyKind::Lander)
            .map(|enemy| enemy.position)
            .collect::<Vec<_>>();
        if lander_positions.is_empty() {
            return;
        }

        for human in &mut self.humans {
            if human.carried
                && !human.carried_by_player
                && let Some(lander_position) =
                    clean_nearest_lander_for_carried_human(&lander_positions, human.position)
            {
                if !clean_lander_pull_position_matches(lander_position, *human) {
                    human.position = clean_carried_human_position(lander_position);
                }
                human.clear_source_fall();
            }
        }
    }

    fn first_lander_human_capture(&self) -> Option<(usize, usize)> {
        for (lander_index, lander) in self
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, enemy)| enemy.kind == EnemyKind::Lander)
        {
            if let Some(source_lander) = lander.source_lander {
                let Some(human_index) =
                    source_lander_live_target_index(source_lander, &self.humans)
                else {
                    continue;
                };
                let human = self.humans[human_index];
                if clean_lander_capture_aligned(lander.position, human.position) {
                    return Some((lander_index, human_index));
                }
                continue;
            }

            for (human_index, human) in self
                .humans
                .iter()
                .enumerate()
                .filter(|(_, human)| !human.carried && !human.carried_by_player)
            {
                if clean_lander_capture_aligned(lander.position, human.position) {
                    return Some((lander_index, human_index));
                }
            }
        }

        None
    }

    fn release_passenger_for_lander(&mut self, lander_position: ScreenPosition) -> bool {
        let mut released = false;
        for human in &mut self.humans {
            if clean_carried_human_matches_lander(lander_position, *human) {
                human.carried = false;
                human.clear_source_fall();
                released = true;
            }
        }
        released
    }

    fn refresh_object_evidence(&mut self) {
        let active_count = saturating_u16_len(
            self.enemies
                .len()
                .saturating_add(self.humans.len())
                .saturating_add(self.projectiles.len())
                .saturating_add(self.enemy_projectiles.len()),
        );
        let mut object_evidence = ObjectEvidenceSnapshot {
            active_count,
            inactive_count: u16::from(self.enemy_reserve.total()),
            projectile_count: saturating_u16_len(
                self.projectiles
                    .len()
                    .saturating_add(self.enemy_projectiles.len()),
            ),
            visible_count: active_count,
            evidence_crc32: None,
            detail_count: 0,
            details: [ObjectEvidenceDetailSnapshot::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT],
        };
        for enemy in &self.enemies {
            object_evidence.push_clean_enemy_detail(*enemy);
        }
        for human in &self.humans {
            object_evidence.push_clean_human_detail(*human);
        }
        for projectile in &self.projectiles {
            object_evidence.push_clean_player_projectile_detail(*projectile);
        }
        for projectile in &self.enemy_projectiles {
            object_evidence.push_clean_enemy_projectile_detail(*projectile);
        }
        object_evidence.push_clean_reserve_details(self.enemy_reserve);
        self.object_evidence = object_evidence;
    }

    pub fn spawn_score_popup(&mut self, kind: ScorePopupKind, position: ScreenPosition) {
        if self.score_popups.len() >= EXPANDED_OBJECT_DETAIL_LIMIT {
            return;
        }
        self.score_popups
            .push(ScorePopupSnapshot::source_spawn(kind, position));
    }

    pub fn spawn_explosion(&mut self, kind: ExplosionKind, position: ScreenPosition) {
        if self
            .score_popups
            .len()
            .saturating_add(self.explosions.len())
            >= EXPANDED_OBJECT_DETAIL_LIMIT
        {
            return;
        }
        self.explosions
            .push(ExplosionSnapshot::source_spawn(kind, position));
    }

    pub fn start_terrain_blow(&mut self) {
        if self.terrain_blow.is_some() {
            return;
        }

        self.terrain.clear();
        self.terrain_blow = Some(TerrainBlowSnapshot::source_started());
        for position in [
            ScreenPosition::new(0x44, 0x70),
            ScreenPosition::new(0x10, 0x50),
        ] {
            self.spawn_explosion(ExplosionKind::Terrain, position);
        }
    }

    fn advance_terrain_blow(&mut self, sound_events: &mut Vec<SoundEvent>) {
        let Some(terrain_blow) = self.terrain_blow else {
            return;
        };

        let Some(source_sleep_remaining) = terrain_blow.source_sleep_remaining else {
            return;
        };
        if source_sleep_remaining > 1 {
            if let Some(terrain_blow) = self.terrain_blow.as_mut() {
                terrain_blow.source_sleep_remaining = Some(source_sleep_remaining - 1);
            }
            return;
        }

        match terrain_blow.stage {
            TerrainBlowStage::ExplosionPassSleeping => {
                let sleep_max = (terrain_blow.source_iteration >> 3).wrapping_add(1);
                let sleep_time = source_advance_rmax(&mut self.source_rng, sleep_max);
                if let Some(terrain_blow) = self.terrain_blow.as_mut() {
                    terrain_blow.stage = TerrainBlowStage::FlashClearedSleeping;
                    terrain_blow.source_sleep_remaining = Some(sleep_time);
                    terrain_blow.source_pseudo_color = 0;
                    terrain_blow.source_overload_counter = SOURCE_TERRAIN_BLOW_OVERLOAD_COUNTER;
                }
            }
            TerrainBlowStage::FlashClearedSleeping => {
                let source_iteration = terrain_blow.source_iteration.saturating_add(1);
                if source_iteration >= terrain_blow.source_iteration_limit {
                    if let Some(terrain_blow) = self.terrain_blow.as_mut() {
                        terrain_blow.stage = TerrainBlowStage::Completed;
                        terrain_blow.source_iteration = source_iteration;
                        terrain_blow.source_sleep_remaining = None;
                    }
                    sound_events.push(source_terrain_blow_complete_sound_event());
                    return;
                }

                let source_pseudo_color = self.spawn_source_terrain_blow_explosion_pass();
                if let Some(terrain_blow) = self.terrain_blow.as_mut() {
                    terrain_blow.stage = TerrainBlowStage::ExplosionPassSleeping;
                    terrain_blow.source_iteration = source_iteration;
                    terrain_blow.source_sleep_remaining = Some(SOURCE_TERRAIN_BLOW_SLEEP_TICKS);
                    terrain_blow.source_pseudo_color = source_pseudo_color;
                    terrain_blow.source_overload_counter = SOURCE_TERRAIN_BLOW_OVERLOAD_COUNTER;
                }
                sound_events.push(source_terrain_blow_start_sound_event());
            }
            TerrainBlowStage::Completed => {}
        }
    }

    fn spawn_source_terrain_blow_explosion_pass(&mut self) -> u8 {
        let mut source_x_register = SOURCE_TERRAIN_BLOW_EXPLOSIONS_PER_PASS;
        for _ in 0..SOURCE_TERRAIN_BLOW_EXPLOSIONS_PER_PASS {
            let rand_state = self.source_rng.advance();
            let position = source_terrain_blow_explosion_position(rand_state, source_x_register);
            source_x_register = position.x;
            self.spawn_explosion(ExplosionKind::Terrain, position);
        }
        source_terrain_blow_pseudo_color(self.source_rng.seed)
    }

    fn advance_score_popups(&mut self) {
        for popup in &mut self.score_popups {
            popup.frames_remaining = popup.frames_remaining.saturating_sub(1);
        }
        self.score_popups.retain(|popup| popup.frames_remaining > 0);
    }

    fn advance_explosions(&mut self) {
        self.explosions
            .retain_mut(|explosion| explosion.advance_source_frame());
    }

    fn sync_clean_lifecycle_evidence(&mut self) {
        let previous_clean_lifecycle_details = self
            .expanded_objects
            .details
            .iter()
            .take(usize::from(self.expanded_objects.detail_count))
            .filter(|detail| expanded_object_detail_is_clean_lifecycle(detail))
            .count();
        let mut evidence = ExpandedObjectEvidenceSnapshot {
            active_count: self
                .expanded_objects
                .active_count
                .saturating_sub(saturating_u16_len(previous_clean_lifecycle_details))
                .saturating_add(saturating_u16_len(
                    self.score_popups
                        .len()
                        .saturating_add(self.explosions.len()),
                )),
            last_slot_address: self.expanded_objects.last_slot_address,
            detail_count: 0,
            details: [ExpandedObjectDetailSnapshot::EMPTY; EXPANDED_OBJECT_DETAIL_LIMIT],
        };

        for detail in self
            .expanded_objects
            .details
            .iter()
            .take(usize::from(self.expanded_objects.detail_count))
            .copied()
        {
            if expanded_object_detail_is_clean_lifecycle(&detail) {
                continue;
            }
            push_expanded_object_detail(&mut evidence, detail);
        }

        for popup in self.score_popups.iter().copied() {
            push_expanded_object_detail(&mut evidence, popup.expanded_object_detail());
        }
        for explosion in self.explosions.iter().copied() {
            push_expanded_object_detail(&mut evidence, explosion.expanded_object_detail());
        }

        self.expanded_objects = evidence;
    }

    fn sync_scanner_radar(
        &mut self,
        phase: GamePhase,
        frame: u64,
        scan_anchor: WorldVector,
        player_position: (WorldVector, WorldVector),
    ) {
        self.scanner = ScannerRadarSnapshot::for_world(
            phase,
            frame,
            scan_anchor,
            player_position,
            &self.object_evidence,
        );
        if self
            .terrain_blow
            .is_some_and(TerrainBlowSnapshot::terrain_erased)
        {
            self.scanner.terrain_enabled = false;
        }
    }
}

fn push_expanded_object_detail(
    evidence: &mut ExpandedObjectEvidenceSnapshot,
    detail: ExpandedObjectDetailSnapshot,
) {
    let index = usize::from(evidence.detail_count);
    if index >= EXPANDED_OBJECT_DETAIL_LIMIT {
        return;
    }
    evidence.details[index] = detail;
    evidence.detail_count += 1;
}

fn expanded_object_detail_is_clean_lifecycle(detail: &ExpandedObjectDetailSnapshot) -> bool {
    detail.score_popup_lifetime_ticks.is_some() || detail.explosion_lifetime_frames.is_some()
}

impl ObjectEvidenceSnapshot {
    fn push_clean_enemy_detail(&mut self, enemy: EnemySnapshot) {
        let index = usize::from(self.detail_count);
        if index >= OBJECT_EVIDENCE_DETAIL_LIMIT {
            return;
        }
        let descriptor = enemy.source_picture_descriptor();
        let identity = source_object_table_identity(index);
        let object_category = enemy.kind.object_category();
        self.details[index] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Active,
            object_category: Some(object_category),
            address: Some(identity.address),
            slot: Some(identity.slot),
            screen_position: Some(enemy.position),
            world_position: Some(enemy.source_world_position()),
            velocity: Some(enemy.source_velocity_words()),
            picture_address: Some(descriptor.address),
            picture_label: Some(descriptor.label),
            picture_size: Some(descriptor.size),
            primary_image_address: Some(descriptor.primary_image_address),
            alternate_image_address: descriptor.alternate_image_address,
            mapped_sprite: Some(descriptor.mapped_sprite),
            object_type: Some(identity.object_type),
            scanner_color: scanner_color_for_object_category(object_category),
        };
        self.detail_count += 1;
    }

    fn push_clean_player_projectile_detail(&mut self, projectile: ProjectileSnapshot) {
        let index = usize::from(self.detail_count);
        if index >= OBJECT_EVIDENCE_DETAIL_LIMIT {
            return;
        }
        let descriptor = SOURCE_PLAYER_PROJECTILE_PICTURE_DESCRIPTOR;
        let identity = source_object_table_identity(index);
        self.details[index] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Projectile,
            object_category: Some(ObjectEvidenceCategory::PlayerProjectile),
            address: Some(identity.address),
            slot: Some(identity.slot),
            screen_position: Some(projectile.position),
            world_position: Some(projectile.source_world_position()),
            velocity: Some(projectile.source_velocity_words()),
            picture_address: Some(descriptor.address),
            picture_label: Some(descriptor.label),
            picture_size: Some(descriptor.size),
            primary_image_address: Some(descriptor.primary_image_address),
            alternate_image_address: descriptor.alternate_image_address,
            mapped_sprite: Some(descriptor.mapped_sprite),
            object_type: Some(identity.object_type),
            scanner_color: None,
        };
        self.detail_count += 1;
    }

    fn push_clean_human_detail(&mut self, human: HumanSnapshot) {
        let index = usize::from(self.detail_count);
        if index >= OBJECT_EVIDENCE_DETAIL_LIMIT {
            return;
        }
        let descriptor = source_human_picture_descriptor(human.source_picture_frame);
        let identity = source_object_table_identity(index);
        self.details[index] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Active,
            object_category: Some(ObjectEvidenceCategory::Human),
            address: Some(identity.address),
            slot: Some(identity.slot),
            screen_position: Some(human.position),
            world_position: Some(human.source_world_position()),
            velocity: Some(human.source_velocity_words()),
            picture_address: Some(descriptor.address),
            picture_label: Some(descriptor.label),
            picture_size: Some(descriptor.size),
            primary_image_address: Some(descriptor.primary_image_address),
            alternate_image_address: descriptor.alternate_image_address,
            mapped_sprite: Some(descriptor.mapped_sprite),
            object_type: Some(identity.object_type),
            scanner_color: scanner_color_for_object_category(ObjectEvidenceCategory::Human),
        };
        self.detail_count += 1;
    }

    fn push_clean_enemy_projectile_detail(&mut self, projectile: EnemyProjectileSnapshot) {
        let index = usize::from(self.detail_count);
        if index >= OBJECT_EVIDENCE_DETAIL_LIMIT {
            return;
        }
        let identity = source_object_table_identity(index);
        self.details[index] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Projectile,
            object_category: Some(ObjectEvidenceCategory::EnemyBomb),
            address: Some(identity.address),
            slot: Some(identity.slot),
            screen_position: Some(projectile.position),
            world_position: Some(projectile.source_world_position()),
            velocity: Some(projectile.source_velocity_words()),
            picture_address: Some(SOURCE_BOMB_SHELL_PICTURE_ADDRESS),
            picture_label: Some(projectile.source_bomb_picture_label()),
            picture_size: Some(SOURCE_BOMB_SHELL_PICTURE_SIZE),
            primary_image_address: Some(SOURCE_BOMB_SHELL_PRIMARY_IMAGE_ADDRESS),
            alternate_image_address: Some(SOURCE_BOMB_SHELL_ALTERNATE_IMAGE_ADDRESS),
            mapped_sprite: Some(SpriteId::ENEMY_BOMB),
            object_type: Some(identity.object_type),
            scanner_color: None,
        };
        self.detail_count += 1;
    }

    fn push_clean_reserve_details(&mut self, reserve: EnemyReserveSnapshot) {
        for (kind, count) in reserve.source_family_counts() {
            for _ in 0..count {
                self.push_clean_reserve_detail(kind);
            }
        }
    }

    fn push_clean_reserve_detail(&mut self, kind: EnemyKind) {
        let index = usize::from(self.detail_count);
        if index >= OBJECT_EVIDENCE_DETAIL_LIMIT {
            return;
        }
        let descriptor = source_reserve_picture_descriptor(kind);
        let identity = source_object_table_identity(index);
        let object_category = kind.object_category();
        self.details[index] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Inactive,
            object_category: Some(object_category),
            address: Some(identity.address),
            slot: Some(identity.slot),
            screen_position: None,
            world_position: None,
            velocity: None,
            picture_address: Some(descriptor.address),
            picture_label: Some(descriptor.label),
            picture_size: Some(descriptor.size),
            primary_image_address: Some(descriptor.primary_image_address),
            alternate_image_address: descriptor.alternate_image_address,
            mapped_sprite: Some(descriptor.mapped_sprite),
            object_type: Some(identity.object_type),
            scanner_color: scanner_color_for_object_category(object_category),
        };
        self.detail_count += 1;
    }
}

fn scanner_radar_stage_for_frame(frame: u64) -> ScannerRadarStage {
    match frame % 8 {
        0 | 1 => ScannerRadarStage::InactiveObjectScan,
        2 | 3 => ScannerRadarStage::ActiveAndShellScan,
        _ => ScannerRadarStage::RasterDisplay,
    }
}

fn scanner_radar_blip_kind(list: ObjectEvidenceList) -> Option<ScannerRadarBlipKind> {
    match list {
        ObjectEvidenceList::Active => Some(ScannerRadarBlipKind::ActiveObject),
        ObjectEvidenceList::Inactive => Some(ScannerRadarBlipKind::InactiveObject),
        ObjectEvidenceList::Projectile => None,
    }
}

fn scanner_radar_object_world_position(
    detail: &ObjectEvidenceDetailSnapshot,
) -> Option<(u16, u16)> {
    if let Some(position) = detail.world_position {
        return Some(position);
    }
    detail
        .screen_position
        .map(|position| (u16::from(position.x) << 8, u16::from(position.y) << 8))
}

fn scanner_radar_object_screen_address(world_x: u16, world_y: u16, scan_left: u16) -> u16 {
    let x_delta = world_x.wrapping_sub(scan_left);
    let x_byte = x_delta.to_be_bytes()[0] >> 2;
    let y_byte = world_y.to_be_bytes()[0] >> 3;
    u16::from_be_bytes([x_byte, y_byte]).wrapping_add(SOURCE_SCANNER_OBJECT_BASE_SCREEN - 1)
}

fn scanner_radar_player_screen_address(player_position: (WorldVector, WorldVector)) -> u16 {
    let x_word = source_word_from_world_vector(player_position.0);
    let y_word = source_word_from_world_vector(player_position.1);
    let x_byte = x_word.to_be_bytes()[0] >> 4;
    let y_byte = y_word.to_be_bytes()[0] >> 3;
    u16::from_be_bytes([x_byte, y_byte]).wrapping_add(SOURCE_SCANNER_PLAYER_BASE_SCREEN)
}

fn source_word_from_world_vector(vector: WorldVector) -> u16 {
    (vector.subpixels() >> 8) as u16
}

fn source_shell_scroll_delta(
    previous_camera_left: WorldVector,
    current_camera_left: WorldVector,
) -> u16 {
    source_word_from_world_vector(previous_camera_left)
        .wrapping_sub(source_word_from_world_vector(current_camera_left))
        .wrapping_shl(2)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceHyperspaceRematerialization {
    position: (WorldVector, WorldVector),
    velocity: (WorldVector, WorldVector),
    direction: Direction,
    camera_left: WorldVector,
    screen_position: ScreenPosition,
    death_risk: bool,
}

fn source_hyperspace_rematerialization(
    source_rng: SourceRandSnapshot,
    previous_player_y: WorldVector,
) -> SourceHyperspaceRematerialization {
    let seed_word = u16::from_be_bytes([source_rng.seed, source_rng.hseed]);
    let (player_x16, direction) = if source_rng.hseed & 1 != 0 {
        (0x2000, Direction::Right)
    } else {
        (0x7000, Direction::Left)
    };
    let previous_y_low = source_word_from_world_vector(previous_player_y).to_be_bytes()[1];
    let player_y = (source_rng.hseed >> 1).wrapping_add(SOURCE_PLAYFIELD_Y_MIN);
    let player_y16 = u16::from_be_bytes([player_y, previous_y_low]);

    SourceHyperspaceRematerialization {
        position: (world_word(player_x16), world_word(player_y16)),
        velocity: (WorldVector::default(), WorldVector::default()),
        direction,
        camera_left: world_word(seed_word),
        screen_position: ScreenPosition::new(player_x16.to_be_bytes()[0], player_y),
        death_risk: source_rng.lseed > SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD,
    }
}

fn source_world_position(position: ScreenPosition, x_fraction: u8, y_fraction: u8) -> (u16, u16) {
    (
        u16::from_be_bytes([position.x, x_fraction]),
        u16::from_be_bytes([position.y, y_fraction]),
    )
}

fn source_fixed_velocity_words(velocity: ScreenVelocity) -> (u16, u16) {
    (
        source_fixed_velocity_word(velocity.dx),
        source_fixed_velocity_word(velocity.dy),
    )
}

fn source_fixed_velocity_word(velocity: i8) -> u16 {
    ((i16::from(velocity)) << 8) as u16
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceObjectTableIdentity {
    address: u16,
    slot: u16,
    object_type: u8,
}

fn source_object_table_identity(detail_index: usize) -> SourceObjectTableIdentity {
    let slot = u16::try_from(detail_index)
        .expect("clean object evidence detail index should fit source object slot");
    SourceObjectTableIdentity {
        address: SOURCE_OBJECT_TABLE_BASE
            .wrapping_add(SOURCE_OBJECT_TABLE_STRIDE.wrapping_mul(slot)),
        slot,
        object_type: SOURCE_OBJECT_DEFAULT_TYPE,
    }
}

fn source_reserve_picture_descriptor(kind: EnemyKind) -> SourceObjectPictureDescriptor {
    match kind {
        EnemyKind::Lander => source_lander_picture_descriptor(0),
        EnemyKind::Mutant => SOURCE_MUTANT_PICTURE_DESCRIPTOR,
        EnemyKind::Bomber => source_bomber_picture_descriptor(0),
        EnemyKind::Pod => SOURCE_POD_PICTURE_DESCRIPTOR,
        EnemyKind::Swarmer => SOURCE_SWARMER_PICTURE_DESCRIPTOR,
        EnemyKind::Baiter => source_baiter_picture_descriptor(0),
    }
}

fn source_human_picture_descriptor(frame: u8) -> SourceObjectPictureDescriptor {
    match frame % 4 {
        1 => SOURCE_HUMAN_ASTP2_PICTURE_DESCRIPTOR,
        2 => SOURCE_HUMAN_ASTP3_PICTURE_DESCRIPTOR,
        3 => SOURCE_HUMAN_ASTP4_PICTURE_DESCRIPTOR,
        _ => SOURCE_HUMAN_ASTP1_PICTURE_DESCRIPTOR,
    }
}

fn scanner_color_for_object_category(category: ObjectEvidenceCategory) -> Option<u16> {
    match category {
        ObjectEvidenceCategory::Lander
        | ObjectEvidenceCategory::Mutant
        | ObjectEvidenceCategory::Bomber
        | ObjectEvidenceCategory::Pod
        | ObjectEvidenceCategory::Swarmer
        | ObjectEvidenceCategory::Baiter => Some(SOURCE_SCANNER_LANDER_COLOR_WORD),
        ObjectEvidenceCategory::Human => Some(SOURCE_SCANNER_HUMAN_COLOR_WORD),
        ObjectEvidenceCategory::PlayerProjectile | ObjectEvidenceCategory::EnemyBomb => None,
    }
}

fn saturating_u16_len(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceObjectPictureDescriptor {
    label: &'static str,
    address: u16,
    size: (u8, u8),
    primary_image_address: u16,
    alternate_image_address: Option<u16>,
    mapped_sprite: SpriteId,
}

const SOURCE_PLAYER_PROJECTILE_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "LASP1",
        address: 0xF96F,
        size: (8, 1),
        primary_image_address: 0xF973,
        alternate_image_address: None,
        mapped_sprite: SpriteId::PLAYER_PROJECTILE,
    };
const SOURCE_HUMAN_ASTP1_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "ASTP1",
        address: 0xF901,
        size: (2, 8),
        primary_image_address: 0xFACB,
        alternate_image_address: Some(0xFADB),
        mapped_sprite: SpriteId::HUMAN,
    };
const SOURCE_HUMAN_ASTP2_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "ASTP2",
        address: 0xF90B,
        size: (2, 8),
        primary_image_address: 0xFAEB,
        alternate_image_address: Some(0xFAFB),
        mapped_sprite: SpriteId::HUMAN,
    };
const SOURCE_HUMAN_ASTP3_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "ASTP3",
        address: 0xF915,
        size: (2, 8),
        primary_image_address: 0xFB0B,
        alternate_image_address: Some(0xFB1B),
        mapped_sprite: SpriteId::HUMAN,
    };
const SOURCE_HUMAN_ASTP4_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "ASTP4",
        address: 0xF91F,
        size: (2, 8),
        primary_image_address: 0xFB2B,
        alternate_image_address: Some(0xFB3B),
        mapped_sprite: SpriteId::HUMAN,
    };
const SOURCE_MUTANT_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "SCZP1",
        address: 0xF8CE,
        size: (5, 8),
        primary_image_address: 0xF9FB,
        alternate_image_address: Some(0xFA23),
        mapped_sprite: SpriteId::ENEMY_MUTANT,
    };
const SOURCE_POD_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "PRBP1",
        address: 0xF8F7,
        size: (4, 8),
        primary_image_address: 0xFA8B,
        alternate_image_address: Some(0xFAAB),
        mapped_sprite: SpriteId::ENEMY_POD,
    };
const SOURCE_SWARMER_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "SWPIC1",
        address: 0xF97B,
        size: (3, 4),
        primary_image_address: 0xCCC8,
        alternate_image_address: Some(0xCCD4),
        mapped_sprite: SpriteId::ENEMY_SWARMER,
    };

fn source_lander_picture_descriptor(frame: u8) -> SourceObjectPictureDescriptor {
    match frame % SOURCE_LANDER_PICTURE_FRAME_COUNT {
        1 => SourceObjectPictureDescriptor {
            label: "LNDP2",
            address: 0xF98F,
            size: (5, 8),
            primary_image_address: 0xCD30,
            alternate_image_address: Some(0xCD58),
            mapped_sprite: SpriteId::ENEMY_LANDER,
        },
        2 => SourceObjectPictureDescriptor {
            label: "LNDP3",
            address: 0xF999,
            size: (5, 8),
            primary_image_address: 0xCD80,
            alternate_image_address: Some(0xCDA8),
            mapped_sprite: SpriteId::ENEMY_LANDER,
        },
        _ => SourceObjectPictureDescriptor {
            label: "LNDP1",
            address: 0xF985,
            size: (5, 8),
            primary_image_address: 0xCCE0,
            alternate_image_address: Some(0xCD08),
            mapped_sprite: SpriteId::ENEMY_LANDER,
        },
    }
}

fn source_bomber_picture_descriptor(frame: u8) -> SourceObjectPictureDescriptor {
    match frame % SOURCE_BOMBER_PICTURE_FRAME_COUNT {
        1 => SourceObjectPictureDescriptor {
            label: "TIEP2",
            address: 0xF933,
            size: (4, 8),
            primary_image_address: 0xFB8B,
            alternate_image_address: Some(0xFBAB),
            mapped_sprite: SpriteId::ENEMY_BOMBER,
        },
        2 => SourceObjectPictureDescriptor {
            label: "TIEP3",
            address: 0xF93D,
            size: (4, 8),
            primary_image_address: 0xFBCB,
            alternate_image_address: Some(0xFBEB),
            mapped_sprite: SpriteId::ENEMY_BOMBER,
        },
        3 => SourceObjectPictureDescriptor {
            label: "TIEP4",
            address: 0xF947,
            size: (4, 8),
            primary_image_address: 0xFC0B,
            alternate_image_address: Some(0xFC2B),
            mapped_sprite: SpriteId::ENEMY_BOMBER,
        },
        _ => SourceObjectPictureDescriptor {
            label: "TIEP1",
            address: 0xF929,
            size: (4, 8),
            primary_image_address: 0xFB4B,
            alternate_image_address: Some(0xFB6B),
            mapped_sprite: SpriteId::ENEMY_BOMBER,
        },
    }
}

fn source_baiter_picture_descriptor(frame: u8) -> SourceObjectPictureDescriptor {
    match frame % SOURCE_BAITER_PICTURE_FRAME_COUNT {
        1 => SourceObjectPictureDescriptor {
            label: "UFOP2",
            address: 0xF9AD,
            size: (6, 4),
            primary_image_address: 0xCE00,
            alternate_image_address: Some(0xCE18),
            mapped_sprite: SpriteId::ENEMY_BAITER,
        },
        2 => SourceObjectPictureDescriptor {
            label: "UFOP3",
            address: 0xF9B7,
            size: (6, 4),
            primary_image_address: 0xCE30,
            alternate_image_address: Some(0xCE48),
            mapped_sprite: SpriteId::ENEMY_BAITER,
        },
        _ => SourceObjectPictureDescriptor {
            label: "UFOP1",
            address: 0xF9A3,
            size: (6, 4),
            primary_image_address: 0xCDD0,
            alternate_image_address: Some(0xCDE8),
            mapped_sprite: SpriteId::ENEMY_BAITER,
        },
    }
}

const CLEAN_WAVE_SPAWN_POSITIONS: [ScreenPosition; SOURCE_MAX_ACTIVE_WAVE_ENEMIES] = [
    ScreenPosition::new(204, 84),
    ScreenPosition::new(228, 104),
    ScreenPosition::new(184, 72),
    ScreenPosition::new(148, 96),
    ScreenPosition::new(236, 66),
];

const SOURCE_MAX_ACTIVE_WAVE_ENEMIES: usize = 5;
const SOURCE_START_HUMAN_COUNT: u8 = 10;
const SOURCE_ASTRO_RESTORE_Y: u8 = 0xE0;
const SOURCE_TARGET_LIST_BASE: u16 = 0xA11A;
const SOURCE_TARGET_LIST_ENTRY_STRIDE: u16 = 2;
const SOURCE_TARGET_LIST_ENTRY_COUNT: usize = 32;
const SOURCE_ASTRONAUT_TARGET_CURSOR_ENTRY_COUNT: usize = 16;
const SOURCE_ASTRONAUT_PROCESS_SLEEP_TICKS: u8 = 2;
const SOURCE_ASTRONAUT_TURN_SEED_MAX: u8 = 8;
const SOURCE_ASTRONAUT_LEFT_TARGET_Y_OFFSET: u8 = 4;
const SOURCE_ASTRONAUT_RIGHT_TARGET_Y_OFFSET: u8 = 15;
const SOURCE_ASTRONAUT_MAX_TARGET_Y: u8 = 0xE8;
const SOURCE_ASTRONAUT_LEFT_X_VELOCITY: u16 = 0xFFE0;
const SOURCE_ASTRONAUT_RIGHT_X_VELOCITY: u16 = 0x0020;
const SOURCE_OBJECT_TABLE_BASE: u16 = 0xA23C;
const SOURCE_OBJECT_TABLE_STRIDE: u16 = 0x17;
const SOURCE_OBJECT_DEFAULT_TYPE: u8 = 0x00;
const SOURCE_POD_SWARMER_REQUEST_LIMIT: usize = 6;
const SOURCE_INITIAL_POD_X_SPEED: u8 = 0x20;
const SOURCE_ACTIVE_SWARMER_LIMIT: usize = 20;
const SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS: u8 = 3;
const SOURCE_MINI_SWARMER_MAX_Y_VELOCITY: u16 = 0x0200;
const SOURCE_MINI_SWARMER_MIN_Y_VELOCITY: u16 = 0xFE00;
const SOURCE_MINI_SWARMER_TURN_WINDOW: u16 = 300 * 32;
const SOURCE_MINI_SWARMER_TURN_WINDOW_HALF: u16 = 150 * 32;
const SOURCE_MINI_SWARMER_RESTORE_X_LOW: u8 = 0x07;
const SOURCE_BOMBER_LOOP_SLEEP_TICKS: u8 = 1;
const SOURCE_BOMBER_PICTURE_FRAME_COUNT: u8 = 4;
const SOURCE_BOMBER_SQUAD_SIZE: usize = 4;
const SOURCE_BOMBER_CRUISE_ALTITUDE: u8 = 0x50;
const SOURCE_BOMBER_MIN_CRUISE_ALTITUDE: u8 = 0x40;
const SOURCE_BOMBER_MAX_CRUISE_ALTITUDE: u8 = 0x68;
const SOURCE_BOMBER_CRUISE_WINDOW: u8 = 0x20;
const SOURCE_BOMBER_CRUISE_WINDOW_HALF: u8 = 0x10;
const SOURCE_BOMBER_BOMB_SHELL_LIMIT: usize = 10;
const SOURCE_BOMB_SHELL_OUTPUT_ROUTINE_ADDRESS: u16 = 0xE498;
const SOURCE_FIREBALL_OUTPUT_ROUTINE_ADDRESS: u16 = 0xE4D9;
const SOURCE_BOMB_SHELL_PICTURE_LABEL: &str = "BMBP1";
const SOURCE_BOMB_SHELL_PICTURE_ADDRESS: u16 = 0xF95B;
const SOURCE_BOMB_SHELL_PRIMARY_IMAGE_ADDRESS: u16 = 0xCCB0;
const SOURCE_BOMB_SHELL_ALTERNATE_IMAGE_ADDRESS: u16 = 0xCCB6;
const SOURCE_BOMB_SHELL_PICTURE_SIZE: (u8, u8) = (2, 3);
const SOURCE_GAME_EXEC_SLEEP_FRAMES: u8 = 15;
const SOURCE_ACTIVE_BAITER_LIMIT: usize = 12;
const SOURCE_BAITER_INITIAL_SHOT_TIMER: u8 = 8;
const SOURCE_BAITER_LOOP_SLEEP_TICKS: u8 = 6;
const SOURCE_BAITER_X_SEEK_SPEED: u8 = 0x40;
const SOURCE_BAITER_Y_SEEK_BYTE: u8 = 0x01;
const SOURCE_BAITER_X_SEEK_WINDOW: u16 = 40 * 32;
const SOURCE_BAITER_X_SEEK_WINDOW_HALF: u16 = 20 * 32;
const SOURCE_BAITER_Y_SEEK_WINDOW: u8 = 20;
const SOURCE_BAITER_Y_SEEK_WINDOW_HALF: u8 = 10;
const SOURCE_BAITER_PICTURE_FRAME_COUNT: u8 = 3;
const SOURCE_LANDER_ORBIT_SLEEP_TICKS: u8 = 6;
const SOURCE_LANDER_FLEE_SLEEP_TICKS: u8 = 4;
const SOURCE_LANDER_GRAB_SLEEP_TICKS: u8 = 1;
const SOURCE_LANDER_GRAB_X_STEP: u16 = 0x0020;
const SOURCE_LANDER_PICTURE_FRAME_COUNT: u8 = 3;
const SOURCE_LANDER_TERRAIN_ALTITUDE_OFFSET: u8 = 50;
const SOURCE_LANDER_ORBIT_Y_WINDOW: i8 = -20;
const SOURCE_PLAYFIELD_Y_MIN: u8 = 42;
const SOURCE_PLAYFIELD_Y_MAX: u8 = 240;
const SOURCE_MUTANT_X_DISTANCE_OFFSET: u16 = 380;
const SOURCE_MUTANT_CLOSE_X_WINDOW: u16 = 0x0700;
const SOURCE_MUTANT_VERTICAL_WINDOW: u8 = 8;
const SOURCE_MUTANT_LOOP_SLEEP_TICKS: u8 = 3;
const SOURCE_MUTANT_RESTORE_AVOID_HALF_WIDTH: u16 = 300 * 32;
const SOURCE_MUTANT_RESTORE_AVOID_WIDTH: u16 = 600 * 32;
const SOURCE_SHELL_LIMIT: usize = 20;
const SOURCE_SHELL_LIFETIME_TICKS: u8 = 20;
const SOURCE_SHELL_X_MAX: u8 = 0x98;
const SOURCE_ENEMY_PROJECTILE_SCORE_POINTS: u32 = 25;
const CLEAN_LANDER_PASSENGER_OFFSET_X: u8 = 2;
const CLEAN_LANDER_PASSENGER_OFFSET_Y: u8 = 12;
const CLEAN_LANDER_CAPTURE_X_TOLERANCE: i16 = 4;
const CLEAN_LANDER_CAPTURE_Y_TOLERANCE: i16 = 4;
const CLEAN_LANDER_CAPTURE_Y_VELOCITY: i8 = -1;
const CLEAN_PLAYER_CARRIED_HUMAN_OFFSET_X: u8 = 0;
const CLEAN_PLAYER_CARRIED_HUMAN_OFFSET_Y: u8 = 10;
const HUMAN_SPRITE_SIZE: (u8, u8) = (6, 8);
const SOURCE_FALLING_HUMAN_Y_ACCELERATION: u16 = 0x0008;
const SOURCE_FALLING_HUMAN_MAX_Y_VELOCITY: u16 = 0x0300;
const SOURCE_FALLING_HUMAN_SAFE_LANDING_MAX_Y_VELOCITY: u16 = 0x00E0;

fn source_target_list_slot_address(slot_index: usize) -> u16 {
    assert!(
        slot_index < SOURCE_TARGET_LIST_ENTRY_COUNT,
        "source target-list slot should fit TLIST"
    );
    SOURCE_TARGET_LIST_BASE.wrapping_add(
        SOURCE_TARGET_LIST_ENTRY_STRIDE
            .wrapping_mul(u16::try_from(slot_index).expect("source target-list slot fits u16")),
    )
}

fn source_target_list_slot_index(address: u16) -> Option<usize> {
    let offset = address.checked_sub(SOURCE_TARGET_LIST_BASE)?;
    if offset % SOURCE_TARGET_LIST_ENTRY_STRIDE != 0 {
        return None;
    }
    let index = usize::from(offset / SOURCE_TARGET_LIST_ENTRY_STRIDE);
    (index < SOURCE_TARGET_LIST_ENTRY_COUNT).then_some(index)
}

fn source_target_list_next_slot_address(cursor_address: u16) -> u16 {
    let cursor_address = source_target_list_slot_index(cursor_address)
        .map(source_target_list_slot_address)
        .unwrap_or(SOURCE_TARGET_LIST_BASE);
    let next_address = cursor_address.wrapping_add(SOURCE_TARGET_LIST_ENTRY_STRIDE);
    if source_target_list_slot_index(next_address).is_some() {
        next_address
    } else {
        SOURCE_TARGET_LIST_BASE
    }
}

fn source_astronaut_next_cursor_address(cursor_address: u16) -> u16 {
    let cursor_index = source_target_list_slot_index(cursor_address).unwrap_or(0);
    let next_index = cursor_index + 1;
    if next_index < SOURCE_ASTRONAUT_TARGET_CURSOR_ENTRY_COUNT {
        source_target_list_slot_address(next_index)
    } else {
        SOURCE_TARGET_LIST_BASE
    }
}

fn source_select_lander_target_index(
    source_target_list_cursor_address: &mut Option<u16>,
    humans: &[HumanSnapshot],
) -> Option<usize> {
    if !humans.iter().any(source_lander_targetable_human) {
        return None;
    }

    let original_cursor = source_target_list_cursor_address
        .and_then(|address| {
            source_target_list_slot_index(address).map(source_target_list_slot_address)
        })
        .unwrap_or(SOURCE_TARGET_LIST_BASE);
    let mut cursor = original_cursor;
    for _ in 0..SOURCE_TARGET_LIST_ENTRY_COUNT {
        cursor = source_target_list_next_slot_address(cursor);
        if let Some(target_index) = humans.iter().position(|human| {
            source_lander_targetable_human(human)
                && human.source_target_slot_address == Some(cursor)
        }) {
            *source_target_list_cursor_address = Some(cursor);
            return Some(target_index);
        }
        if cursor == original_cursor {
            break;
        }
    }

    None
}

fn source_lander_targetable_human(human: &HumanSnapshot) -> bool {
    human.source_target_slot_address.is_some() && !human.carried && !human.carried_by_player
}

fn source_walk_astronaut(human: &mut HumanSnapshot, terrain: &[TerrainSegment], seed: u8) {
    let frame = human.source_picture_frame % 4;
    let walking_left = frame <= 1;
    let (next_frame, target_y, x_velocity) = if walking_left {
        if seed <= SOURCE_ASTRONAUT_TURN_SEED_MAX {
            (2, None, SOURCE_ASTRONAUT_RIGHT_X_VELOCITY)
        } else {
            (
                1 - frame,
                source_astronaut_target_y(
                    terrain,
                    human.position.x,
                    SOURCE_ASTRONAUT_LEFT_TARGET_Y_OFFSET,
                ),
                SOURCE_ASTRONAUT_LEFT_X_VELOCITY,
            )
        }
    } else if seed <= SOURCE_ASTRONAUT_TURN_SEED_MAX {
        (0, None, SOURCE_ASTRONAUT_LEFT_X_VELOCITY)
    } else {
        (
            if frame == 2 { 3 } else { 2 },
            source_astronaut_target_y(
                terrain,
                human.position.x,
                SOURCE_ASTRONAUT_RIGHT_TARGET_Y_OFFSET,
            ),
            SOURCE_ASTRONAUT_RIGHT_X_VELOCITY,
        )
    };

    if let Some(target_y) = target_y {
        human.position.y = source_step_astronaut_y(human.position.y, target_y);
    }
    let (x, x_fraction) =
        source_fixed_axis_step(human.position.x, human.source_x_fraction, x_velocity);
    human.position.x = x;
    human.source_x_fraction = x_fraction;
    human.source_picture_frame = next_frame;
}

fn source_astronaut_target_y(terrain: &[TerrainSegment], position_x: u8, offset: u8) -> Option<u8> {
    clean_source_terrain_altitude(terrain, position_x).map(|altitude| {
        altitude
            .wrapping_add(offset)
            .min(SOURCE_ASTRONAUT_MAX_TARGET_Y)
    })
}

fn source_step_astronaut_y(position_y: u8, target_y: u8) -> u8 {
    match target_y.cmp(&position_y) {
        std::cmp::Ordering::Greater => position_y.wrapping_add(1),
        std::cmp::Ordering::Less => position_y.wrapping_sub(1),
        std::cmp::Ordering::Equal => position_y,
    }
}

fn source_target_list_human(
    position: ScreenPosition,
    source_x_fraction: u8,
    source_picture_frame: u8,
    slot_index: usize,
) -> HumanSnapshot {
    HumanSnapshot {
        source_x_fraction,
        source_picture_frame,
        source_target_slot_address: Some(source_target_list_slot_address(slot_index)),
        ..HumanSnapshot::new(position)
    }
}

fn source_initial_target_list_humans() -> Vec<HumanSnapshot> {
    let mut target_rng = SourceRandSnapshot::default();
    source_target_list_restore_humans(&mut target_rng, SOURCE_START_HUMAN_COUNT)
}

fn source_target_list_restore_humans(
    source_rng: &mut SourceRandSnapshot,
    target_count: u8,
) -> Vec<HumanSnapshot> {
    let mut humans = Vec::with_capacity(usize::from(target_count));
    let mut slot_index = 0usize;
    let mut remainder = target_count;

    if target_count > 7 {
        let quadrant_count = target_count >> 2;
        for x_bank in [0x00, 0x40, 0x80, 0xC0] {
            slot_index = source_target_list_restore_human_group(
                &mut humans,
                source_rng,
                quadrant_count,
                x_bank,
                slot_index,
            );
        }
        remainder = target_count.wrapping_sub(quadrant_count << 2);
    }

    for _ in 0..remainder {
        let x_bank = source_rng.hseed;
        slot_index =
            source_target_list_restore_human_group(&mut humans, source_rng, 1, x_bank, slot_index);
    }

    humans
}

fn source_target_list_restore_human_group(
    humans: &mut Vec<HumanSnapshot>,
    source_rng: &mut SourceRandSnapshot,
    count: u8,
    x_bank: u8,
    mut slot_index: usize,
) -> usize {
    for _ in 0..count {
        let state = source_rng.advance();
        let source_x = (state.hseed & 0x1F).wrapping_add(x_bank);
        let source_picture_frame = if state.lseed & 0x01 != 0 { 2 } else { 0 };
        humans.push(source_target_list_human(
            ScreenPosition::new(source_x, SOURCE_ASTRO_RESTORE_Y),
            state.lseed,
            source_picture_frame,
            slot_index,
        ));
        slot_index += 1;
    }
    slot_index
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CleanWaveSpawnContext {
    Initial,
    ReserveActivation {
        player_absolute_x: u16,
        background_absolute_x: u16,
        targetable_humans: bool,
    },
}

fn clean_wave_enemy_spawns(
    reserve: &mut EnemyReserveSnapshot,
    profile: WaveProfileSnapshot,
    source_rng: &mut SourceRandSnapshot,
    context: CleanWaveSpawnContext,
    humans: &[HumanSnapshot],
    source_target_list_cursor_address: &mut Option<u16>,
) -> Vec<EnemySnapshot> {
    let mut enemies = Vec::new();
    let kinds = clean_wave_active_enemy_kinds(reserve, profile);
    let mut index = 0;
    while index < kinds.len() {
        let kind = kinds[index];
        if let CleanWaveSpawnContext::ReserveActivation {
            player_absolute_x,
            background_absolute_x,
            targetable_humans,
        } = context
        {
            if kind == EnemyKind::Lander && !targetable_humans {
                let lander_count = kinds[index..]
                    .iter()
                    .take_while(|&&kind| kind == EnemyKind::Lander)
                    .count();
                enemies.extend(source_mutant_restore_spawns(
                    source_rng,
                    profile,
                    background_absolute_x,
                    lander_count,
                ));
                index += lander_count;
                continue;
            }

            if kind == EnemyKind::Bomber {
                let bomber_count = kinds[index..]
                    .iter()
                    .take_while(|&&kind| kind == EnemyKind::Bomber)
                    .count();
                enemies.extend(source_bomber_restore_spawns(
                    profile,
                    player_absolute_x,
                    bomber_count,
                ));
                index += bomber_count;
                continue;
            }

            if kind == EnemyKind::Swarmer {
                let swarmer_count = kinds[index..]
                    .iter()
                    .take_while(|&&kind| kind == EnemyKind::Swarmer)
                    .count();
                enemies.extend(source_mini_swarmer_reserve_spawns(
                    source_rng,
                    profile,
                    swarmer_count,
                ));
                index += swarmer_count;
                continue;
            }
        }

        let position = CLEAN_WAVE_SPAWN_POSITIONS[index];
        let mut enemy = match kind {
            EnemyKind::Lander
                if matches!(context, CleanWaveSpawnContext::ReserveActivation { .. }) =>
            {
                source_lander_restore_spawn(source_rng, profile)
            }
            EnemyKind::Lander => source_lander_initial_spawn(profile, position, index),
            EnemyKind::Bomber => {
                let source_bomber = source_bomber_spawn(profile, index);
                EnemySnapshot::source_bomber(
                    position,
                    source_bomber_screen_velocity(source_bomber),
                    source_bomber,
                )
            }
            EnemyKind::Mutant => {
                let background_absolute_x = match context {
                    CleanWaveSpawnContext::Initial => 0,
                    CleanWaveSpawnContext::ReserveActivation {
                        background_absolute_x,
                        ..
                    } => background_absolute_x,
                };
                let (position, source_mutant) =
                    source_mutant_restore_spawn(source_rng, profile, background_absolute_x);
                EnemySnapshot::source_mutant(
                    position,
                    source_mutant_screen_velocity(source_mutant),
                    source_mutant,
                )
            }
            EnemyKind::Pod
                if matches!(context, CleanWaveSpawnContext::ReserveActivation { .. }) =>
            {
                source_pod_restore_spawn(source_rng)
            }
            EnemyKind::Pod => source_pod_initial_spawn(position, index),
            _ => EnemySnapshot::new(
                kind,
                position,
                clean_enemy_initial_velocity(kind, profile, index),
            ),
        };
        assign_source_lander_target(&mut enemy, humans, source_target_list_cursor_address);
        enemies.push(enemy);
        index += 1;
    }

    enemies
}

fn assign_source_lander_target(
    enemy: &mut EnemySnapshot,
    humans: &[HumanSnapshot],
    source_target_list_cursor_address: &mut Option<u16>,
) {
    if enemy.kind != EnemyKind::Lander {
        return;
    }
    let Some(source_lander) = enemy.source_lander.as_mut() else {
        return;
    };
    source_lander.target_human_index =
        source_select_lander_target_index(source_target_list_cursor_address, humans);
}

fn clean_wave_active_enemy_kinds(
    reserve: &mut EnemyReserveSnapshot,
    profile: WaveProfileSnapshot,
) -> Vec<EnemyKind> {
    let target = usize::from(profile.wave_size)
        .min(SOURCE_MAX_ACTIVE_WAVE_ENEMIES)
        .min(usize::from(reserve.total()));
    let mut kinds = Vec::with_capacity(target);

    push_clean_wave_kind(&mut kinds, reserve, target, EnemyKind::Lander);
    for kind in [
        EnemyKind::Bomber,
        EnemyKind::Pod,
        EnemyKind::Mutant,
        EnemyKind::Swarmer,
    ] {
        push_clean_wave_kind(&mut kinds, reserve, target, kind);
    }

    for kind in [
        EnemyKind::Lander,
        EnemyKind::Bomber,
        EnemyKind::Pod,
        EnemyKind::Mutant,
        EnemyKind::Swarmer,
    ] {
        while kinds.len() < target && reserve.take(kind) {
            kinds.push(kind);
        }
    }

    kinds
}

fn push_clean_wave_kind(
    kinds: &mut Vec<EnemyKind>,
    reserve: &mut EnemyReserveSnapshot,
    target: usize,
    kind: EnemyKind,
) {
    if kinds.len() < target && reserve.take(kind) {
        kinds.push(kind);
    }
}

fn clean_enemy_initial_velocity(
    kind: EnemyKind,
    profile: WaveProfileSnapshot,
    spawn_index: usize,
) -> ScreenVelocity {
    let direction = if spawn_index < 2 { -1 } else { 1 };
    match kind {
        EnemyKind::Lander => ScreenVelocity::new(
            direction * source_velocity_pixels(profile.lander_x_velocity),
            source_vertical_velocity(profile.lander_y_velocity_msb, profile.lander_y_velocity_lsb),
        ),
        EnemyKind::Mutant => ScreenVelocity::new(
            direction * source_velocity_pixels(profile.mutant_x_velocity),
            source_vertical_velocity(profile.mutant_y_velocity_msb, profile.mutant_y_velocity_lsb),
        ),
        EnemyKind::Bomber => ScreenVelocity::new(
            direction * source_velocity_pixels(profile.bomber_x_velocity),
            0,
        ),
        EnemyKind::Pod => ScreenVelocity::new(direction, 0),
        EnemyKind::Swarmer => ScreenVelocity::new(
            direction * source_velocity_pixels(profile.swarmer_x_velocity),
            0,
        ),
        EnemyKind::Baiter => ScreenVelocity::new(direction, 0),
    }
}

fn source_velocity_pixels(source_velocity: u8) -> i8 {
    let pixels = (source_velocity / 32).max(1);
    i8::try_from(pixels).expect("source wave velocity should fit clean screen velocity")
}

fn adc8(lhs: u8, rhs: u8, carry: bool) -> (u8, bool) {
    let sum = u16::from(lhs) + u16::from(rhs) + u16::from(u8::from(carry));
    ((sum & 0xFF) as u8, sum > 0xFF)
}

fn source_rmax(max: u8, mut seed: u8) -> u8 {
    while seed > max {
        seed >>= 1;
    }
    seed.wrapping_add(1)
}

fn source_advance_rmax(source_rng: &mut SourceRandSnapshot, max: u8) -> u8 {
    let state = source_rng.advance();
    source_rmax(max, state.seed)
}

fn source_terrain_blow_pseudo_color(seed: u8) -> u8 {
    SOURCE_TERRAIN_BLOW_PSEUDO_COLORS[usize::from(seed & 0x1F)]
}

fn source_terrain_blow_explosion_position(
    rand_state: SourceRandSnapshot,
    source_x_register: u8,
) -> ScreenPosition {
    let x = rand_state.seed.wrapping_add(source_x_register);
    let y = rand_state
        .hseed
        .clamp(0x40, SOURCE_PLAYFIELD_Y_MAX.saturating_sub(0x10));
    ScreenPosition::new(x, y)
}

fn source_sign_extend_u8_to_u16(value: u8) -> u16 {
    let sign = if value & 0x80 == 0 { 0x00 } else { 0xFF };
    u16::from_be_bytes([sign, value])
}

fn source_screen_velocity(x_velocity: u16, y_velocity: u16) -> ScreenVelocity {
    ScreenVelocity::new(
        source_screen_velocity_component(x_velocity),
        source_screen_velocity_component(y_velocity),
    )
}

fn source_screen_velocity_component(velocity: u16) -> i8 {
    let signed = velocity as i16;
    if signed == 0 {
        return 0;
    }

    let pixels = signed / 256;
    if pixels == 0 {
        return if signed > 0 { 1 } else { -1 };
    }
    pixels.clamp(i16::from(i8::MIN), i16::from(i8::MAX)) as i8
}

fn source_fixed_axis_step(position: u8, fraction: u8, velocity: u16) -> (u8, u8) {
    let [position, fraction] = u16::from_be_bytes([position, fraction])
        .wrapping_add(velocity)
        .to_be_bytes();
    (position, fraction)
}

fn source_active_object_y_step(position: u8, fraction: u8, velocity: u16) -> (u8, u8) {
    let [mut position, fraction] = u16::from_be_bytes([position, fraction])
        .wrapping_add(velocity)
        .to_be_bytes();
    if position < SOURCE_PLAYFIELD_Y_MIN {
        position = SOURCE_PLAYFIELD_Y_MAX;
    } else if position > SOURCE_PLAYFIELD_Y_MAX {
        position = SOURCE_PLAYFIELD_Y_MIN;
    }
    (position, fraction)
}

fn source_lander_restore_spawn(
    source_rng: &mut SourceRandSnapshot,
    profile: WaveProfileSnapshot,
) -> EnemySnapshot {
    let placement_state = source_rng.advance();
    let [x, x_fraction] =
        u16::from_be_bytes([placement_state.hseed, placement_state.lseed]).to_be_bytes();
    let y = SOURCE_PLAYFIELD_Y_MIN.wrapping_add(2);
    let y_velocity =
        u16::from_be_bytes([profile.lander_y_velocity_msb, profile.lander_y_velocity_lsb]);

    let shot_timer = source_advance_rmax(source_rng, profile.lander_shot_time as u8);
    let x_velocity_byte = source_advance_rmax(source_rng, profile.lander_x_velocity);
    let x_velocity = if x_velocity_byte & 1 == 0 {
        u16::from(x_velocity_byte)
    } else {
        !u16::from(x_velocity_byte)
    };
    let source_lander = SourceLanderSnapshot {
        x_fraction,
        y_fraction: 0,
        x_velocity,
        y_velocity,
        shot_timer,
        sleep_ticks: 0,
        picture_frame: 0,
        target_human_index: None,
    };

    EnemySnapshot::source_lander(
        ScreenPosition::new(x, y),
        source_lander_screen_velocity(source_lander),
        source_lander,
    )
}

fn source_lander_initial_spawn(
    profile: WaveProfileSnapshot,
    position: ScreenPosition,
    spawn_index: usize,
) -> EnemySnapshot {
    let source_lander = SourceLanderSnapshot {
        x_fraction: 0,
        y_fraction: 0,
        x_velocity: source_lander_initial_x_velocity(profile.lander_x_velocity, spawn_index),
        y_velocity: source_lander_base_y_velocity(profile),
        shot_timer: profile.lander_shot_time as u8,
        sleep_ticks: 0,
        picture_frame: 0,
        target_human_index: None,
    };

    EnemySnapshot::source_lander(
        position,
        source_lander_screen_velocity(source_lander),
        source_lander,
    )
}

fn source_lander_initial_x_velocity(source_x_velocity: u8, spawn_index: usize) -> u16 {
    let velocity_low = if spawn_index < 2 {
        0u8.wrapping_sub(source_x_velocity)
    } else {
        source_x_velocity
    };
    source_sign_extend_u8_to_u16(velocity_low)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceLanderAdvancePhase {
    Sleeping,
    Orbiting,
    Fleeing,
    Grabbing,
    PullingPassenger,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SourceLanderAdvance {
    phase: SourceLanderAdvancePhase,
    direct_position: Option<ScreenPosition>,
}

impl SourceLanderAdvance {
    const fn velocity(phase: SourceLanderAdvancePhase) -> Self {
        Self {
            phase,
            direct_position: None,
        }
    }

    const fn direct(phase: SourceLanderAdvancePhase, position: ScreenPosition) -> Self {
        Self {
            phase,
            direct_position: Some(position),
        }
    }
}

struct SourceLanderAdvanceContext<'a, 'b> {
    profile: WaveProfileSnapshot,
    terrain: &'a [TerrainSegment],
    carrying_passenger: bool,
    grab_target: Option<ScreenPosition>,
    player_position: ScreenPosition,
    player_velocity: (WorldVector, WorldVector),
    source_rng: &'b mut SourceRandSnapshot,
    enemy_projectiles: &'b mut Vec<EnemyProjectileSnapshot>,
}

fn advance_source_lander(
    position: ScreenPosition,
    source_lander: &mut SourceLanderSnapshot,
    mut context: SourceLanderAdvanceContext<'_, '_>,
) -> SourceLanderAdvance {
    if source_lander.sleep_ticks > 0 {
        source_lander.sleep_ticks = source_lander.sleep_ticks.saturating_sub(1);
        return SourceLanderAdvance::velocity(SourceLanderAdvancePhase::Sleeping);
    }

    if context.carrying_passenger && source_lander_pull_edge(position) {
        source_lander.y_velocity = 0;
        return SourceLanderAdvance::velocity(SourceLanderAdvancePhase::PullingPassenger);
    }

    if !context.carrying_passenger
        && let Some(target_position) = context.grab_target
    {
        return advance_source_lander_grab(position, source_lander, target_position, &mut context);
    }

    source_lander.y_velocity = if context.carrying_passenger {
        !source_lander_base_y_velocity(context.profile)
    } else {
        source_lander_orbit_y_velocity(context.profile, position, context.terrain)
    };
    source_lander_run_shot_timer(
        position,
        source_lander,
        context.profile,
        context.player_position,
        context.player_velocity,
        context.source_rng,
        context.enemy_projectiles,
    );

    if context.carrying_passenger {
        source_lander.sleep_ticks = SOURCE_LANDER_FLEE_SLEEP_TICKS;
        SourceLanderAdvance::velocity(SourceLanderAdvancePhase::Fleeing)
    } else {
        source_lander.picture_frame =
            (source_lander.picture_frame + 1) % SOURCE_LANDER_PICTURE_FRAME_COUNT;
        source_lander.sleep_ticks = SOURCE_LANDER_ORBIT_SLEEP_TICKS;
        SourceLanderAdvance::velocity(SourceLanderAdvancePhase::Orbiting)
    }
}

fn advance_source_lander_grab(
    position: ScreenPosition,
    source_lander: &mut SourceLanderSnapshot,
    target_position: ScreenPosition,
    context: &mut SourceLanderAdvanceContext<'_, '_>,
) -> SourceLanderAdvance {
    let mut x16 = u16::from_be_bytes([position.x, source_lander.x_fraction]);
    let target_x16 = u16::from(target_position.x) << 8;
    if (x16 & 0xFFE0) != (target_x16 & 0xFFE0) {
        let x_step = if (x16 as i16) < (target_x16 as i16) {
            SOURCE_LANDER_GRAB_X_STEP
        } else {
            0u16.wrapping_sub(SOURCE_LANDER_GRAB_X_STEP)
        };
        x16 = x16.wrapping_add(x_step);
    }

    let mut y16 = u16::from_be_bytes([position.y, source_lander.y_fraction]);
    let target_y = target_position
        .y
        .wrapping_sub(CLEAN_LANDER_PASSENGER_OFFSET_Y);
    if target_y != position.y {
        let y_step = if target_y < position.y {
            !source_lander_base_y_velocity(context.profile)
        } else {
            source_lander_base_y_velocity(context.profile)
        };
        y16 = y16.wrapping_add(y_step);
    }

    let [x, x_fraction] = x16.to_be_bytes();
    let [y, y_fraction] = y16.to_be_bytes();
    source_lander.x_fraction = x_fraction;
    source_lander.y_fraction = y_fraction;
    source_lander.x_velocity = 0;
    source_lander.y_velocity = 0;
    source_lander.picture_frame = 0;
    let next_position = ScreenPosition::new(x, y);
    source_lander_run_shot_timer(
        next_position,
        source_lander,
        context.profile,
        context.player_position,
        context.player_velocity,
        context.source_rng,
        context.enemy_projectiles,
    );
    source_lander.sleep_ticks = SOURCE_LANDER_GRAB_SLEEP_TICKS;
    SourceLanderAdvance::direct(SourceLanderAdvancePhase::Grabbing, next_position)
}

fn source_lander_run_shot_timer(
    position: ScreenPosition,
    source_lander: &mut SourceLanderSnapshot,
    profile: WaveProfileSnapshot,
    player_position: ScreenPosition,
    player_velocity: (WorldVector, WorldVector),
    source_rng: &mut SourceRandSnapshot,
    enemy_projectiles: &mut Vec<EnemyProjectileSnapshot>,
) {
    source_lander.shot_timer = source_lander.shot_timer.wrapping_sub(1);
    if source_lander.shot_timer != 0 {
        return;
    }

    source_rng.advance();
    source_lander.shot_timer = source_rmax(profile.lander_shot_time as u8, source_rng.seed);
    if let Some(projectile) = source_enemy_fireball_shot(
        position,
        source_lander.x_fraction,
        source_lander.y_fraction,
        player_position,
        player_velocity,
        *source_rng,
        enemy_projectiles.len(),
    ) {
        enemy_projectiles.push(projectile);
    }
}

fn source_lander_orbit_y_velocity(
    profile: WaveProfileSnapshot,
    position: ScreenPosition,
    terrain: &[TerrainSegment],
) -> u16 {
    let base_y_velocity = source_lander_base_y_velocity(profile);
    let altitude =
        clean_source_terrain_altitude(terrain, position.x).unwrap_or(SOURCE_PLAYFIELD_Y_MAX);
    let above_ground_delta = altitude.wrapping_sub(SOURCE_LANDER_TERRAIN_ALTITUDE_OFFSET);
    let y_delta = above_ground_delta.wrapping_sub(position.y);
    if above_ground_delta > position.y {
        base_y_velocity
    } else if (y_delta as i8) < SOURCE_LANDER_ORBIT_Y_WINDOW {
        !base_y_velocity
    } else {
        0
    }
}

fn clean_source_terrain_altitude(terrain: &[TerrainSegment], object_x: u8) -> Option<u8> {
    let object_x = u16::from(object_x);
    terrain
        .iter()
        .find(|segment| {
            let start = u16::from(segment.position.x);
            let end = start.saturating_add(u16::from(segment.size.0));
            object_x >= start && object_x < end
        })
        .map(|segment| segment.position.y)
}

fn source_lander_base_y_velocity(profile: WaveProfileSnapshot) -> u16 {
    u16::from_be_bytes([profile.lander_y_velocity_msb, profile.lander_y_velocity_lsb])
}

fn source_bomber_spawn(profile: WaveProfileSnapshot, spawn_index: usize) -> SourceBomberSnapshot {
    SourceBomberSnapshot {
        x_fraction: 0,
        y_fraction: 0,
        x_velocity: source_bomber_x_velocity(profile.bomber_x_velocity, spawn_index),
        y_velocity: 0,
        picture_frame: 0,
        cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
        sleep_ticks: 0,
        source_slot: (spawn_index % SOURCE_BOMBER_SQUAD_SIZE) as u8,
    }
}

fn source_bomber_x_velocity(source_x_velocity: u8, spawn_index: usize) -> u16 {
    let velocity_low = if spawn_index < 2 {
        0u8.wrapping_sub(source_x_velocity)
    } else {
        source_x_velocity
    };
    source_sign_extend_u8_to_u16(velocity_low)
}

fn source_tie_selected_slot(seed: u8) -> usize {
    usize::from((seed & 0x06) >> 1)
}

fn source_bomber_restore_spawns(
    profile: WaveProfileSnapshot,
    player_absolute_x: u16,
    count: usize,
) -> Vec<EnemySnapshot> {
    let mut bombers = Vec::with_capacity(count);
    let mut remaining = count;
    let mut positive_x_velocity = true;

    while remaining > 0 {
        let squad_count = remaining.min(SOURCE_BOMBER_SQUAD_SIZE);
        let velocity_low = if positive_x_velocity {
            profile.bomber_x_velocity
        } else {
            0u8.wrapping_sub(profile.bomber_x_velocity)
        };
        positive_x_velocity = !positive_x_velocity;
        let x_velocity = source_sign_extend_u8_to_u16(velocity_low);

        for squad_remaining in (1..=squad_count).rev() {
            let x16 = player_absolute_x
                .wrapping_add((squad_remaining as u16).wrapping_mul(0x0180))
                .wrapping_add(0x8000);
            let [x, x_fraction] = x16.to_be_bytes();
            let source_bomber = SourceBomberSnapshot {
                x_fraction,
                y_fraction: 0,
                x_velocity,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                source_slot: (squad_remaining - 1) as u8,
            };
            bombers.push(EnemySnapshot::source_bomber(
                ScreenPosition::new(x, SOURCE_BOMBER_CRUISE_ALTITUDE),
                source_bomber_screen_velocity(source_bomber),
                source_bomber,
            ));
        }

        remaining -= squad_count;
    }

    bombers
}

fn advance_source_bomber(
    position: ScreenPosition,
    source_bomber: &mut SourceBomberSnapshot,
    player_position: ScreenPosition,
    source_rng: SourceRandSnapshot,
    enemy_projectiles: &mut Vec<EnemyProjectileSnapshot>,
) {
    if source_bomber.sleep_ticks > 0 {
        source_bomber.sleep_ticks = source_bomber.sleep_ticks.saturating_sub(1);
        return;
    }

    source_bomber.picture_frame =
        source_bomber_next_picture_frame(source_rng.seed, source_bomber.picture_frame);
    source_bomber.y_velocity =
        source_bomber_random_y_velocity(source_bomber.y_velocity, source_rng.seed);

    if position.y == 0 {
        source_bomber.y_velocity = source_bomber_offscreen_y_velocity(
            source_bomber.y_velocity,
            &mut source_bomber.cruise_altitude,
            position.y,
            source_rng.seed,
        );
    } else {
        if let Some(delta) = source_bomber_onscreen_y_velocity_delta(position.y, player_position.y)
        {
            source_bomber.y_velocity = source_bomber.y_velocity.wrapping_add(delta);
        }

        if source_rng.lseed & 0x07 == 0
            && let Some(projectile) = source_bomber_bomb_shell(
                position,
                *source_bomber,
                source_rng,
                source_bomb_shell_count(enemy_projectiles),
                enemy_projectiles.len(),
            )
        {
            enemy_projectiles.push(projectile);
        }
    }

    source_bomber.sleep_ticks = SOURCE_BOMBER_LOOP_SLEEP_TICKS;
}

fn source_bomber_next_picture_frame(seed: u8, picture_frame: u8) -> u8 {
    let random_delta = (seed & 0x3F).wrapping_sub(0x20);
    if random_delta & 0x80 != 0 {
        picture_frame
            .saturating_add(1)
            .min(SOURCE_BOMBER_PICTURE_FRAME_COUNT - 1)
    } else {
        picture_frame.saturating_sub(1)
    }
}

fn source_bomber_random_y_velocity(previous_y_velocity: u16, seed: u8) -> u16 {
    let random_delta_low = (seed & 0x3F).wrapping_sub(0x20);
    let mut y_velocity =
        previous_y_velocity.wrapping_add(source_sign_extend_u8_to_u16(random_delta_low));
    let damping_high = y_velocity.wrapping_shl(3).to_be_bytes()[0];
    let damping_delta = source_sign_extend_u8_to_u16(0u8.wrapping_sub(damping_high));
    y_velocity = y_velocity.wrapping_add(damping_delta);
    y_velocity
}

fn source_bomber_offscreen_y_velocity(
    mut y_velocity: u16,
    cruise_altitude: &mut u8,
    object_y: u8,
    seed: u8,
) -> u16 {
    if seed <= 0x40 {
        let candidate = cruise_altitude.wrapping_add((seed & 0x03).wrapping_sub(2));
        *cruise_altitude = candidate.clamp(
            SOURCE_BOMBER_MIN_CRUISE_ALTITUDE,
            SOURCE_BOMBER_MAX_CRUISE_ALTITUDE,
        );
    }

    let delta = cruise_altitude
        .wrapping_sub(object_y)
        .wrapping_add(SOURCE_BOMBER_CRUISE_WINDOW_HALF);
    if delta > SOURCE_BOMBER_CRUISE_WINDOW {
        let delta_after_center = delta.wrapping_sub(SOURCE_BOMBER_CRUISE_WINDOW_HALF);
        let velocity_delta = if delta_after_center & 0x80 == 0 {
            0xFFF0
        } else {
            0x0010
        };
        y_velocity = y_velocity.wrapping_add(velocity_delta);
    }

    y_velocity
}

fn source_bomber_onscreen_y_velocity_delta(object_y: u8, player_y: u8) -> Option<u16> {
    let delta = object_y.wrapping_sub(player_y);
    let signed_delta = delta as i8;
    if signed_delta >= 0 {
        if delta >= 0x20 {
            Some(0xFFF0)
        } else if delta > 0x10 {
            None
        } else {
            Some(0x0010)
        }
    } else if signed_delta <= -32 {
        Some(0x0010)
    } else if signed_delta < -16 {
        None
    } else {
        Some(0xFFF0)
    }
}

fn source_bomber_bomb_shell(
    position: ScreenPosition,
    source_bomber: SourceBomberSnapshot,
    source_rng: SourceRandSnapshot,
    active_bomb_shells: usize,
    active_shells: usize,
) -> Option<EnemyProjectileSnapshot> {
    if active_bomb_shells >= SOURCE_BOMBER_BOMB_SHELL_LIMIT
        || active_shells >= SOURCE_SHELL_LIMIT
        || position.x >= SOURCE_SHELL_X_MAX
        || position.y <= SOURCE_PLAYFIELD_Y_MIN
    {
        return None;
    }

    let mut projectile = EnemyProjectileSnapshot::source_bomb_shell(position);
    projectile.source_x_fraction = source_bomber.x_fraction;
    projectile.source_y_fraction = source_bomber.y_fraction;
    projectile.source_lifetime_ticks = (source_rng.seed & 0x1F).wrapping_add(1);
    Some(projectile)
}

fn source_bomb_shell_count(enemy_projectiles: &[EnemyProjectileSnapshot]) -> usize {
    enemy_projectiles
        .iter()
        .filter(|projectile| projectile.is_source_bomb_shell())
        .count()
}

fn source_mutant_from_lander_conversion(profile: WaveProfileSnapshot) -> SourceMutantSnapshot {
    SourceMutantSnapshot {
        x_fraction: 0,
        y_fraction: 0,
        x_velocity: 0,
        y_velocity: 0,
        shot_timer: profile.mutant_shot_time as u8,
        sleep_ticks: 0,
    }
}

fn source_mutant_restore_spawn(
    source_rng: &mut SourceRandSnapshot,
    profile: WaveProfileSnapshot,
    background_absolute_x: u16,
) -> (ScreenPosition, SourceMutantSnapshot) {
    let placement_state = source_rng.advance();
    let avoid_left = background_absolute_x.wrapping_sub(SOURCE_MUTANT_RESTORE_AVOID_HALF_WIDTH);
    let mut relative =
        u16::from_be_bytes([placement_state.hseed, placement_state.lseed]).wrapping_sub(avoid_left);
    if relative < SOURCE_MUTANT_RESTORE_AVOID_WIDTH {
        relative = relative.wrapping_add(0x8000);
    }
    let x16 = relative.wrapping_add(avoid_left);
    let [x, x_fraction] = x16.to_be_bytes();
    let y = placement_state
        .seed
        .wrapping_shr(1)
        .wrapping_add(SOURCE_PLAYFIELD_Y_MIN);
    let shot_timer_state = source_rng.advance();
    let source_mutant = SourceMutantSnapshot {
        x_fraction,
        y_fraction: 0,
        x_velocity: 0,
        y_velocity: 0,
        shot_timer: source_rmax(profile.mutant_shot_time as u8, shot_timer_state.seed),
        sleep_ticks: 0,
    };

    (ScreenPosition::new(x, y), source_mutant)
}

fn source_mutant_restore_spawns(
    source_rng: &mut SourceRandSnapshot,
    profile: WaveProfileSnapshot,
    background_absolute_x: u16,
    count: usize,
) -> Vec<EnemySnapshot> {
    (0..count)
        .map(|_| {
            let (position, source_mutant) =
                source_mutant_restore_spawn(source_rng, profile, background_absolute_x);
            EnemySnapshot::source_mutant(
                position,
                source_mutant_screen_velocity(source_mutant),
                source_mutant,
            )
        })
        .collect()
}

fn source_pod_restore_spawn(source_rng: &mut SourceRandSnapshot) -> EnemySnapshot {
    let state = source_rng.advance();
    let [x, x_fraction] =
        u16::from_be_bytes([(state.hseed & 0x3F).wrapping_add(0x10), state.lseed]).to_be_bytes();
    let y = state
        .lseed
        .wrapping_shr(1)
        .wrapping_add(SOURCE_PLAYFIELD_Y_MIN);

    let x_velocity = source_sign_extend_u8_to_u16((state.seed & 0x3F).wrapping_sub(0x20));
    let mut y_velocity_low = (state.lseed & 0x7F).wrapping_sub(0x40);
    if y_velocity_low & 0x80 == 0 {
        y_velocity_low |= 0x20;
    } else {
        y_velocity_low &= 0xDF;
    }
    let y_velocity = source_sign_extend_u8_to_u16(y_velocity_low);
    let source_pod = SourcePodSnapshot {
        x_fraction,
        y_fraction: 0,
        x_velocity,
        y_velocity,
    };

    EnemySnapshot::source_pod(
        ScreenPosition::new(x, y),
        source_pod_screen_velocity(source_pod),
        source_pod,
    )
}

fn source_pod_initial_spawn(position: ScreenPosition, spawn_index: usize) -> EnemySnapshot {
    let source_pod = SourcePodSnapshot {
        x_fraction: 0,
        y_fraction: 0,
        x_velocity: source_pod_initial_x_velocity(spawn_index),
        y_velocity: 0,
    };

    EnemySnapshot::source_pod(position, source_pod_screen_velocity(source_pod), source_pod)
}

fn source_pod_initial_x_velocity(spawn_index: usize) -> u16 {
    let velocity_low = if spawn_index < 2 {
        0u8.wrapping_sub(SOURCE_INITIAL_POD_X_SPEED)
    } else {
        SOURCE_INITIAL_POD_X_SPEED
    };
    source_sign_extend_u8_to_u16(velocity_low)
}

fn advance_source_mutant(
    position: &mut ScreenPosition,
    source_mutant: &mut SourceMutantSnapshot,
    profile: WaveProfileSnapshot,
    player_position: ScreenPosition,
    player_velocity: (WorldVector, WorldVector),
    source_rng: &mut SourceRandSnapshot,
    enemy_projectiles: &mut Vec<EnemyProjectileSnapshot>,
) {
    if source_mutant.sleep_ticks > 0 {
        source_mutant.sleep_ticks = source_mutant.sleep_ticks.saturating_sub(1);
        return;
    }

    let player_absolute_x = u16::from(player_position.x) << 8;
    let object_absolute_x = u16::from_be_bytes([position.x, source_mutant.x_fraction]);
    source_mutant.x_velocity = source_mutant_x_velocity(
        profile.mutant_x_velocity,
        player_absolute_x,
        object_absolute_x,
    );
    source_mutant.y_velocity = source_mutant_y_velocity(
        profile,
        player_position.y,
        player_absolute_x,
        object_absolute_x,
        *position,
    );

    if source_mutant_should_run_hop_and_shot(player_absolute_x, object_absolute_x, *position) {
        let y_step = if source_rng.seed & 0x80 == 0 {
            0u8.wrapping_sub(profile.mutant_random_y)
        } else {
            profile.mutant_random_y
        };
        position.y = position.y.wrapping_add(y_step);
        if position.y < SOURCE_PLAYFIELD_Y_MIN {
            position.y = SOURCE_PLAYFIELD_Y_MAX;
        }

        source_mutant.shot_timer = source_mutant.shot_timer.wrapping_sub(1);
        if source_mutant.shot_timer == 0 {
            source_rng.advance();
            source_mutant.shot_timer = source_rmax(profile.mutant_shot_time as u8, source_rng.seed);
            if let Some(projectile) = source_enemy_fireball_shot(
                *position,
                source_mutant.x_fraction,
                source_mutant.y_fraction,
                player_position,
                player_velocity,
                *source_rng,
                enemy_projectiles.len(),
            ) {
                enemy_projectiles.push(projectile);
            }
        }
    }

    source_mutant.sleep_ticks = SOURCE_MUTANT_LOOP_SLEEP_TICKS;
}

fn source_mutant_x_velocity(
    source_x_velocity: u8,
    player_absolute_x: u16,
    object_absolute_x: u16,
) -> u16 {
    let x_velocity_low = if (player_absolute_x as i16) >= (object_absolute_x as i16) {
        source_x_velocity
    } else {
        0u8.wrapping_sub(source_x_velocity)
    };
    source_sign_extend_u8_to_u16(x_velocity_low)
}

fn source_mutant_y_velocity(
    profile: WaveProfileSnapshot,
    player_y: u8,
    player_absolute_x: u16,
    object_absolute_x: u16,
    position: ScreenPosition,
) -> u16 {
    let base_y_velocity =
        u16::from_be_bytes([profile.mutant_y_velocity_msb, profile.mutant_y_velocity_lsb]);
    let x_distance = player_absolute_x
        .wrapping_sub(object_absolute_x)
        .wrapping_add(SOURCE_MUTANT_X_DISTANCE_OFFSET);
    if x_distance <= SOURCE_MUTANT_CLOSE_X_WINDOW {
        if player_y >= position.y {
            base_y_velocity
        } else {
            !base_y_velocity
        }
    } else {
        let delta = player_y.wrapping_sub(position.y);
        if player_y > position.y {
            if delta > SOURCE_MUTANT_VERTICAL_WINDOW {
                0
            } else {
                !base_y_velocity
            }
        } else if (delta as i8) > -(SOURCE_MUTANT_VERTICAL_WINDOW as i8) {
            base_y_velocity
        } else {
            0
        }
    }
}

fn source_mutant_should_run_hop_and_shot(
    player_absolute_x: u16,
    object_absolute_x: u16,
    position: ScreenPosition,
) -> bool {
    let x_distance = player_absolute_x
        .wrapping_sub(object_absolute_x)
        .wrapping_add(SOURCE_MUTANT_X_DISTANCE_OFFSET);
    x_distance > SOURCE_MUTANT_CLOSE_X_WINDOW
        || (position.y > SOURCE_PLAYFIELD_Y_MIN && position.y <= SOURCE_PLAYFIELD_Y_MAX)
}

fn source_mini_swarmer_spawn(
    source_rng: &mut SourceRandSnapshot,
    profile: WaveProfileSnapshot,
    _position: ScreenPosition,
) -> SourceSwarmerSnapshot {
    let velocity_rand = source_rng.advance();
    let y_velocity = source_sign_extend_u8_to_u16(velocity_rand.seed).wrapping_shl(1);
    let x_velocity = source_sign_extend_u8_to_u16((velocity_rand.lseed & 0x3F).wrapping_sub(0x20));
    let acceleration = velocity_rand.lseed & profile.swarmer_acceleration_mask;
    let sleep_ticks = velocity_rand.hseed & 0x1F;
    let shot_timer_rand = source_rng.advance();
    let shot_timer = source_rmax(profile.swarmer_shot_time as u8, shot_timer_rand.seed);

    SourceSwarmerSnapshot {
        x_fraction: 0,
        y_fraction: 0,
        x_velocity,
        y_velocity,
        acceleration,
        shot_timer,
        sleep_ticks,
        horizontal_seek_pending: true,
    }
}

fn source_mini_swarmer_reserve_spawns(
    source_rng: &mut SourceRandSnapshot,
    profile: WaveProfileSnapshot,
    count: usize,
) -> Vec<EnemySnapshot> {
    if count == 0 {
        return Vec::new();
    }

    let y16 = u16::from_be_bytes([
        source_rng
            .seed
            .wrapping_shr(1)
            .wrapping_add(SOURCE_PLAYFIELD_Y_MIN),
        0,
    ]);
    let placement_rand = source_rng.advance();
    let x16 = u16::from_be_bytes([
        (placement_rand.seed & 0x3F).wrapping_add(0x80),
        SOURCE_MINI_SWARMER_RESTORE_X_LOW,
    ]);
    let [x, x_fraction] = x16.to_be_bytes();
    let [y, y_fraction] = y16.to_be_bytes();
    let position = ScreenPosition::new(x, y);

    (0..count)
        .map(|_| {
            let mut source_swarmer = source_mini_swarmer_spawn(source_rng, profile, position);
            source_swarmer.x_fraction = x_fraction;
            source_swarmer.y_fraction = y_fraction;
            EnemySnapshot::source_swarmer(
                position,
                source_screen_velocity(source_swarmer.x_velocity, source_swarmer.y_velocity),
                source_swarmer,
            )
        })
        .collect()
}

fn advance_source_mini_swarmer(
    position: ScreenPosition,
    source_swarmer: &mut SourceSwarmerSnapshot,
    profile: WaveProfileSnapshot,
    player_position: ScreenPosition,
    source_rng: &mut SourceRandSnapshot,
    enemy_projectiles: &mut Vec<EnemyProjectileSnapshot>,
) {
    if source_swarmer.sleep_ticks > 0 {
        source_swarmer.sleep_ticks = source_swarmer.sleep_ticks.saturating_sub(1);
        return;
    }

    if source_swarmer.horizontal_seek_pending {
        source_swarmer.x_velocity = source_mini_swarmer_seek_velocity(
            profile.swarmer_x_velocity,
            player_position.x,
            position.x,
        );
        source_swarmer.horizontal_seek_pending = false;
        source_swarmer.sleep_ticks = SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS;
        return;
    }

    source_swarmer.y_velocity = source_mini_swarmer_y_velocity(
        source_swarmer.y_velocity,
        source_swarmer.acceleration,
        player_position.y,
        position.y,
        source_rng.seed,
    );

    let player_absolute_x = u16::from(player_position.x) << 8;
    let object_absolute_x = u16::from_be_bytes([position.x, source_swarmer.x_fraction]);
    let past_window = player_absolute_x
        .wrapping_sub(object_absolute_x)
        .wrapping_add(SOURCE_MINI_SWARMER_TURN_WINDOW_HALF);
    if past_window > SOURCE_MINI_SWARMER_TURN_WINDOW {
        source_swarmer.x_velocity = source_mini_swarmer_seek_velocity(
            profile.swarmer_x_velocity,
            player_position.x,
            position.x,
        );
    } else {
        source_swarmer.shot_timer = source_swarmer.shot_timer.wrapping_sub(1);
        if source_swarmer.shot_timer == 0 {
            if let Some(projectile) = source_mini_swarmer_bomb(
                position,
                *source_swarmer,
                player_position,
                enemy_projectiles.len(),
            ) {
                enemy_projectiles.push(projectile);
            }
            source_swarmer.shot_timer =
                source_advance_rmax(source_rng, profile.swarmer_shot_time as u8);
        }
    }

    source_swarmer.sleep_ticks = SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS;
}

fn source_mini_swarmer_seek_velocity(source_x_velocity: u8, player_x: u8, swarmer_x: u8) -> u16 {
    if player_x >= swarmer_x {
        source_sign_extend_u8_to_u16(source_x_velocity)
    } else {
        source_sign_extend_u8_to_u16(0u8.wrapping_sub(source_x_velocity))
    }
}

fn source_mini_swarmer_y_velocity(
    previous_y_velocity: u16,
    acceleration: u8,
    player_y: u8,
    swarmer_y: u8,
    seed: u8,
) -> u16 {
    let acceleration_low = if player_y > swarmer_y {
        acceleration
    } else {
        0u8.wrapping_sub(acceleration)
    };
    let mut y_velocity =
        source_sign_extend_u8_to_u16(acceleration_low).wrapping_add(previous_y_velocity);
    if (y_velocity as i16) >= (SOURCE_MINI_SWARMER_MAX_Y_VELOCITY as i16) {
        y_velocity = SOURCE_MINI_SWARMER_MAX_Y_VELOCITY;
    }
    if (y_velocity as i16) <= (SOURCE_MINI_SWARMER_MIN_Y_VELOCITY as i16) {
        y_velocity = SOURCE_MINI_SWARMER_MIN_Y_VELOCITY;
    }
    y_velocity = y_velocity.wrapping_add(source_mini_swarmer_damping_adjustment(y_velocity));
    y_velocity.wrapping_add(source_sign_extend_u8_to_u16(
        (seed & 0x1F).wrapping_sub(0x10),
    ))
}

fn source_mini_swarmer_damping_adjustment(value: u16) -> u16 {
    let [mut a, mut b] = value.to_be_bytes();
    a = !a;
    b = !b;
    for _ in 0..2 {
        let carry = b & 0x80 != 0;
        b = b.wrapping_shl(1);
        a = a.wrapping_shl(1) | u8::from(carry);
    }
    source_sign_extend_u8_to_u16(a)
}

fn source_mini_swarmer_bomb(
    position: ScreenPosition,
    source_swarmer: SourceSwarmerSnapshot,
    player_position: ScreenPosition,
    active_shells: usize,
) -> Option<EnemyProjectileSnapshot> {
    let player_delta = (u16::from(player_position.x) << 8)
        .wrapping_sub(u16::from_be_bytes([position.x, source_swarmer.x_fraction]));
    if (player_delta.to_be_bytes()[0] ^ source_swarmer.x_velocity.to_be_bytes()[0]) & 0x80 != 0 {
        return None;
    }

    if active_shells >= SOURCE_SHELL_LIMIT {
        return None;
    }

    let x_velocity = source_swarmer.x_velocity.wrapping_shl(3);
    let y_velocity = source_arithmetic_shift_right_word(
        u16::from_be_bytes([player_position.y.wrapping_sub(position.y), 0]),
        5,
    );
    Some(EnemyProjectileSnapshot::source_fireball(
        position, x_velocity, y_velocity,
    ))
}

fn source_baiter_spawn(
    source_rng: SourceRandSnapshot,
    profile: WaveProfileSnapshot,
    player_position: ScreenPosition,
    player_velocity: (WorldVector, WorldVector),
) -> (ScreenPosition, SourceBaiterSnapshot) {
    let source_x = u16::from_be_bytes([source_rng.seed & 0x1F, source_rng.hseed]);
    let [x, x_fraction] = source_x.wrapping_shl(2).to_be_bytes();
    let source_y = u16::from_be_bytes([source_x.to_be_bytes()[1] >> 1, 0])
        .wrapping_add(u16::from(SOURCE_PLAYFIELD_Y_MIN) << 8);
    let [y, y_fraction] = source_y.to_be_bytes();
    let mut source_baiter = SourceBaiterSnapshot {
        x_fraction,
        y_fraction,
        x_velocity: 0,
        y_velocity: 0,
        shot_timer: SOURCE_BAITER_INITIAL_SHOT_TIMER,
        sleep_ticks: 0,
        picture_frame: 0,
    };
    source_baiter_velocity_update(
        &mut source_baiter,
        ScreenPosition::new(x, y),
        profile,
        player_position,
        player_velocity,
        false,
        source_rng.seed,
    );
    (ScreenPosition::new(x, y), source_baiter)
}

fn advance_source_baiter(
    position: ScreenPosition,
    source_baiter: &mut SourceBaiterSnapshot,
    profile: WaveProfileSnapshot,
    player_position: ScreenPosition,
    player_velocity: (WorldVector, WorldVector),
    source_rng: &mut SourceRandSnapshot,
    enemy_projectiles: &mut Vec<EnemyProjectileSnapshot>,
) {
    if source_baiter.sleep_ticks > 0 {
        source_baiter.sleep_ticks = source_baiter.sleep_ticks.saturating_sub(1);
        return;
    }

    source_baiter.shot_timer = source_baiter.shot_timer.wrapping_sub(1);
    if source_baiter.shot_timer == 0 {
        source_rng.advance();
        source_baiter.shot_timer = source_rmax(profile.baiter_shot_time as u8, source_rng.seed);
        if let Some(projectile) = source_baiter_shot(
            position,
            *source_baiter,
            player_position,
            player_velocity,
            *source_rng,
            enemy_projectiles.len(),
        ) {
            enemy_projectiles.push(projectile);
        }
    }

    source_baiter.picture_frame =
        (source_baiter.picture_frame + 1) % SOURCE_BAITER_PICTURE_FRAME_COUNT;
    if source_baiter.picture_frame == 0 {
        source_baiter_velocity_update(
            source_baiter,
            position,
            profile,
            player_position,
            player_velocity,
            true,
            source_rng.seed,
        );
    }
    source_baiter.sleep_ticks = SOURCE_BAITER_LOOP_SLEEP_TICKS;
}

fn source_baiter_velocity_update(
    source_baiter: &mut SourceBaiterSnapshot,
    position: ScreenPosition,
    profile: WaveProfileSnapshot,
    player_position: ScreenPosition,
    player_velocity: (WorldVector, WorldVector),
    honor_seek_probability: bool,
    seed: u8,
) -> bool {
    if honor_seek_probability && seed <= profile.baiter_seek_probability {
        return false;
    }

    let object_x = u16::from_be_bytes([position.x, source_baiter.x_fraction]).wrapping_shr(2);
    let player_x = (u16::from(player_position.x) << 8).wrapping_shr(2);
    let x_delta = object_x.wrapping_sub(player_x);
    let mut x_seek_byte = SOURCE_BAITER_X_SEEK_SPEED;
    if x_delta & 0x8000 == 0 {
        x_seek_byte = 0u8.wrapping_sub(x_seek_byte);
    }
    if x_delta.wrapping_add(SOURCE_BAITER_X_SEEK_WINDOW_HALF) > SOURCE_BAITER_X_SEEK_WINDOW {
        let player_x_velocity =
            source_arithmetic_shift_right_word(source_word_from_world_vector(player_velocity.0), 2);
        source_baiter.x_velocity =
            source_sign_extend_u8_to_u16(x_seek_byte).wrapping_add(player_x_velocity);
    }

    let object_y = position.y;
    let player_y = player_position.y;
    let y_delta = object_y.wrapping_sub(player_y);
    let mut y_seek_byte = SOURCE_BAITER_Y_SEEK_BYTE;
    if y_delta & 0x80 == 0 {
        y_seek_byte = 0u8.wrapping_sub(y_seek_byte);
    }
    if y_delta.wrapping_add(SOURCE_BAITER_Y_SEEK_WINDOW_HALF) > SOURCE_BAITER_Y_SEEK_WINDOW {
        let player_y_velocity = source_word_from_world_vector(player_velocity.1);
        source_baiter.y_velocity = source_arithmetic_shift_right_word(
            u16::from_be_bytes([y_seek_byte, 0]).wrapping_add(player_y_velocity),
            1,
        );
    }

    true
}

fn source_baiter_shot(
    position: ScreenPosition,
    source_baiter: SourceBaiterSnapshot,
    player_position: ScreenPosition,
    player_velocity: (WorldVector, WorldVector),
    source_rng: SourceRandSnapshot,
    active_shells: usize,
) -> Option<EnemyProjectileSnapshot> {
    source_enemy_fireball_shot(
        position,
        source_baiter.x_fraction,
        source_baiter.y_fraction,
        player_position,
        player_velocity,
        source_rng,
        active_shells,
    )
}

fn source_enemy_fireball_shot(
    position: ScreenPosition,
    x_fraction: u8,
    y_fraction: u8,
    player_position: ScreenPosition,
    player_velocity: (WorldVector, WorldVector),
    source_rng: SourceRandSnapshot,
    active_shells: usize,
) -> Option<EnemyProjectileSnapshot> {
    if active_shells >= SOURCE_SHELL_LIMIT
        || position.x >= SOURCE_SHELL_X_MAX
        || position.y <= SOURCE_PLAYFIELD_Y_MIN
    {
        return None;
    }

    let x_delta = (source_rng.seed & 0x1F)
        .wrapping_sub(0x10)
        .wrapping_add(player_position.x)
        .wrapping_sub(position.x);
    let mut x_velocity = source_sign_extend_u8_to_u16(x_delta).wrapping_shl(2);
    if source_rng.seed > 120 {
        x_velocity = x_velocity
            .wrapping_add(source_word_from_world_vector(player_velocity.0).wrapping_shl(2));
    }

    let y_delta = (source_rng.lseed & 0x1F)
        .wrapping_sub(0x10)
        .wrapping_add(player_position.y)
        .wrapping_sub(position.y);
    let y_velocity = source_sign_extend_u8_to_u16(y_delta).wrapping_shl(2);
    let mut projectile = EnemyProjectileSnapshot::source_fireball(position, x_velocity, y_velocity);
    projectile.source_x_fraction = x_fraction;
    projectile.source_y_fraction = y_fraction;
    Some(projectile)
}

fn source_baiter_screen_x_velocity(source_x_velocity: u16) -> u16 {
    source_x_velocity.wrapping_shl(2)
}

fn source_baiter_screen_velocity(source_baiter: SourceBaiterSnapshot) -> ScreenVelocity {
    source_screen_velocity(
        source_baiter_screen_x_velocity(source_baiter.x_velocity),
        source_baiter.y_velocity,
    )
}

fn source_bomber_screen_velocity(source_bomber: SourceBomberSnapshot) -> ScreenVelocity {
    source_screen_velocity(source_bomber.x_velocity, source_bomber.y_velocity)
}

fn source_lander_screen_velocity(source_lander: SourceLanderSnapshot) -> ScreenVelocity {
    source_screen_velocity(source_lander.x_velocity, source_lander.y_velocity)
}

fn source_mutant_screen_velocity(source_mutant: SourceMutantSnapshot) -> ScreenVelocity {
    source_screen_velocity(source_mutant.x_velocity, source_mutant.y_velocity)
}

fn source_pod_screen_velocity(source_pod: SourcePodSnapshot) -> ScreenVelocity {
    source_screen_velocity(source_pod.x_velocity, source_pod.y_velocity)
}

fn source_arithmetic_shift_right_word(mut value: u16, shifts: u8) -> u16 {
    for _ in 0..shifts {
        let [mut a, mut b] = value.to_be_bytes();
        let carry = a & 0x01 != 0;
        a = (a >> 1) | (a & 0x80);
        b = (b >> 1) | if carry { 0x80 } else { 0x00 };
        value = u16::from_be_bytes([a, b]);
    }
    value
}

fn source_vertical_velocity(msb: u8, lsb: u8) -> i8 {
    if msb == 0 && lsb < 0xC0 {
        0
    } else if msb == 0 {
        -1
    } else {
        1
    }
}

fn clean_carried_human_position(lander_position: ScreenPosition) -> ScreenPosition {
    lander_position.wrapping_offset(
        CLEAN_LANDER_PASSENGER_OFFSET_X,
        CLEAN_LANDER_PASSENGER_OFFSET_Y,
    )
}

fn clean_carried_human_index_for_lander(
    humans: &[HumanSnapshot],
    lander_position: ScreenPosition,
) -> Option<usize> {
    let carried_position = clean_carried_human_position(lander_position);
    humans.iter().position(|human| {
        human.carried
            && !human.carried_by_player
            && (human.position == carried_position
                || clean_lander_pull_position_matches(lander_position, *human))
    })
}

fn clean_carried_human_index_for_source_lander(
    humans: &[HumanSnapshot],
    lander_position: ScreenPosition,
    source_lander: SourceLanderSnapshot,
) -> Option<usize> {
    if let Some(target_index) = source_lander.target_human_index {
        return humans
            .get(target_index)
            .is_some_and(|human| clean_carried_human_matches_lander(lander_position, *human))
            .then_some(target_index);
    }

    clean_carried_human_index_for_lander(humans, lander_position)
}

fn clean_carried_human_matches_lander(
    lander_position: ScreenPosition,
    human: HumanSnapshot,
) -> bool {
    human.carried
        && !human.carried_by_player
        && (human.position == clean_carried_human_position(lander_position)
            || clean_lander_pull_position_matches(lander_position, human))
}

fn source_lander_grab_active(source_lander: SourceLanderSnapshot) -> bool {
    source_lander.sleep_ticks == 0
        && source_lander.x_velocity == 0
        && source_lander.y_velocity == 0
        && source_lander.picture_frame == 0
}

fn source_lander_ensure_live_target(
    source_lander: &mut SourceLanderSnapshot,
    humans: &[HumanSnapshot],
    source_target_list_cursor_address: &mut Option<u16>,
) -> Option<ScreenPosition> {
    if let Some(target_index) = source_lander_live_target_index(*source_lander, humans) {
        return Some(humans[target_index].position);
    }

    source_lander.target_human_index =
        source_select_lander_target_index(source_target_list_cursor_address, humans);
    source_lander
        .target_human_index
        .map(|target_index| humans[target_index].position)
}

fn source_lander_live_target_index(
    source_lander: SourceLanderSnapshot,
    humans: &[HumanSnapshot],
) -> Option<usize> {
    let target_index = source_lander.target_human_index?;
    let human = humans.get(target_index)?;
    (!human.carried && !human.carried_by_player).then_some(target_index)
}

fn source_lander_grab_x_matches(
    lander_position: ScreenPosition,
    human_position: ScreenPosition,
) -> bool {
    (lander_position.x & 0xFC) == (human_position.x & 0xFC)
}

fn clean_nearest_lander_for_carried_human(
    lander_positions: &[ScreenPosition],
    human_position: ScreenPosition,
) -> Option<ScreenPosition> {
    lander_positions.iter().copied().min_by_key(|position| {
        clean_carried_position_distance(clean_carried_human_position(*position), human_position)
    })
}

fn clean_carried_position_distance(lhs: ScreenPosition, rhs: ScreenPosition) -> u16 {
    u16::from(lhs.x.abs_diff(rhs.x)) + u16::from(lhs.y.abs_diff(rhs.y))
}

fn source_lander_pull_edge(lander_position: ScreenPosition) -> bool {
    lander_position.y <= SOURCE_PLAYFIELD_Y_MIN.saturating_add(8)
}

fn clean_lander_pull_position_matches(
    lander_position: ScreenPosition,
    human: HumanSnapshot,
) -> bool {
    if !source_lander_pull_edge(lander_position) || !human.carried || human.carried_by_player {
        return false;
    }

    let carried_position = clean_carried_human_position(lander_position);
    human.position.x == carried_position.x
        && human.position.y >= lander_position.y
        && human.position.y <= carried_position.y
}

fn clean_lander_pull_passenger_position(
    lander_position: ScreenPosition,
    human_position: ScreenPosition,
) -> ScreenPosition {
    let carried_position = clean_carried_human_position(lander_position);
    ScreenPosition::new(
        carried_position.x,
        human_position.y.saturating_sub(1).max(lander_position.y),
    )
}

fn clean_lander_passenger_ready_for_conversion(
    lander_position: ScreenPosition,
    human: HumanSnapshot,
) -> bool {
    clean_lander_pull_position_matches(lander_position, human)
        && human.position.y <= lander_position.y
}

fn clean_player_carried_human_position(player_position: ScreenPosition) -> ScreenPosition {
    player_position.wrapping_offset(
        CLEAN_PLAYER_CARRIED_HUMAN_OFFSET_X,
        CLEAN_PLAYER_CARRIED_HUMAN_OFFSET_Y,
    )
}

fn clean_rescue_score_popup_position(human_position: ScreenPosition) -> ScreenPosition {
    human_position.wrapping_offset(0, SOURCE_RESCUE_SCORE_POPUP_Y_OFFSET)
}

fn clean_lander_capture_aligned(
    lander_position: ScreenPosition,
    human_position: ScreenPosition,
) -> bool {
    let x_delta = (i16::from(lander_position.x) - i16::from(human_position.x)).abs();
    let target_y = i16::from(lander_position.y) + i16::from(CLEAN_LANDER_PASSENGER_OFFSET_Y);
    let y_delta = (target_y - i16::from(human_position.y)).abs();

    x_delta <= CLEAN_LANDER_CAPTURE_X_TOLERANCE && y_delta <= CLEAN_LANDER_CAPTURE_Y_TOLERANCE
}

fn start_lander_human_capture(
    lander: &mut EnemySnapshot,
    profile: WaveProfileSnapshot,
) -> ScreenPosition {
    let lander_position = lander.position;
    if let Some(source_lander) = lander.source_lander.as_mut() {
        if source_lander_pull_edge(lander_position) {
            source_lander.y_velocity = 0;
            source_lander.sleep_ticks = 0;
        } else {
            source_lander.y_velocity = !source_lander_base_y_velocity(profile);
            source_lander.sleep_ticks = SOURCE_LANDER_FLEE_SLEEP_TICKS;
        }
        lander.velocity = source_lander_screen_velocity(*source_lander);
    } else {
        lander.velocity.dy = CLEAN_LANDER_CAPTURE_Y_VELOCITY;
    }
    lander_position
}

fn source_lander_pull_target_cleared(enemy: &EnemySnapshot, humans: &[HumanSnapshot]) -> bool {
    if enemy.kind != EnemyKind::Lander {
        return false;
    }
    let Some(source_lander) = enemy.source_lander else {
        return false;
    };
    if source_lander.y_velocity != 0 || !source_lander_pull_edge(enemy.position) {
        return false;
    }

    if let Some(target_index) = source_lander.target_human_index {
        return !humans
            .get(target_index)
            .is_some_and(|human| clean_lander_pull_position_matches(enemy.position, *human));
    }

    !humans
        .iter()
        .any(|human| clean_lander_pull_position_matches(enemy.position, *human))
}

fn clean_human_ground_y(terrain: &[TerrainSegment], human_x: u8) -> Option<u8> {
    let human_x = u16::from(human_x);
    terrain
        .iter()
        .find(|segment| {
            let start = u16::from(segment.position.x);
            let end = start.saturating_add(u16::from(segment.size.0));
            human_x >= start && human_x < end
        })
        .map(|segment| segment.position.y.saturating_sub(HUMAN_SPRITE_SIZE.1))
}

fn clean_human_is_falling(terrain: &[TerrainSegment], human: HumanSnapshot) -> bool {
    if human.carried || human.carried_by_player {
        return false;
    }

    clean_human_ground_y(terrain, human.position.x)
        .is_some_and(|ground_y| human.position.y < ground_y)
}

fn source_falling_human_y_velocity(previous_y_velocity: u16) -> u16 {
    let accelerated_y_velocity =
        previous_y_velocity.wrapping_add(SOURCE_FALLING_HUMAN_Y_ACCELERATION);
    if accelerated_y_velocity >= SOURCE_FALLING_HUMAN_MAX_Y_VELOCITY {
        previous_y_velocity
    } else {
        accelerated_y_velocity
    }
}

fn source_baiter_accelerated_timer_ticks(
    current_ticks: u32,
    profile: WaveProfileSnapshot,
    enemy_total: usize,
) -> u32 {
    if enemy_total > 8 {
        return current_ticks;
    }

    let mut target_ticks = profile.baiter_delay / 2;
    if enemy_total <= 3 {
        target_ticks /= 2;
    }
    target_ticks = target_ticks.saturating_add(1).max(1);
    current_ticks.min(target_ticks)
}

fn source_baiter_reset_timer_ticks(profile: WaveProfileSnapshot, enemy_total: usize) -> u32 {
    if enemy_total < 4 {
        (profile.baiter_delay / 4).max(1)
    } else {
        profile.baiter_delay.max(1)
    }
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

    const fn player_death_sleep(remaining: u8) -> Self {
        Self {
            player_death_sleep_remaining: Some(remaining),
            player_switch_sleep_remaining: None,
            player_switch_from: None,
            player_switch_to: None,
            no_entry_delay_remaining: None,
            hall_of_fame_stall_remaining: None,
        }
    }

    const fn player_switch_sleep(remaining: u8, from_player: u8, to_player: u8) -> Self {
        Self {
            player_death_sleep_remaining: None,
            player_switch_sleep_remaining: Some(remaining),
            player_switch_from: Some(from_player),
            player_switch_to: Some(to_player),
            no_entry_delay_remaining: None,
            hall_of_fame_stall_remaining: None,
        }
    }

    const fn no_entry_delay(remaining: u8) -> Self {
        Self {
            player_death_sleep_remaining: None,
            player_switch_sleep_remaining: None,
            player_switch_from: None,
            player_switch_to: None,
            no_entry_delay_remaining: Some(remaining),
            hall_of_fame_stall_remaining: None,
        }
    }

    const fn hall_of_fame_display(remaining: u8) -> Self {
        Self {
            player_death_sleep_remaining: None,
            player_switch_sleep_remaining: None,
            player_switch_from: None,
            player_switch_to: None,
            no_entry_delay_remaining: None,
            hall_of_fame_stall_remaining: Some(remaining),
        }
    }
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

    fn todays_qualifying_rank(self, score: u32) -> Option<u8> {
        qualifying_rank(self.todays_greatest, score)
    }

    fn insert_all_time(&mut self, score: u32, initials: [Option<char>; 3]) -> Option<u8> {
        insert_high_score_entry(&mut self.all_time, score, initials)
    }

    fn insert_todays_greatest(&mut self, score: u32, initials: [Option<char>; 3]) -> Option<u8> {
        insert_high_score_entry(&mut self.todays_greatest, score, initials)
    }
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
        initials: [Some('P'), Some('G'), Some('D')],
    },
    HighScoreTableEntrySnapshot {
        rank: 5,
        score: 12_520,
        initials: [Some('C'), Some('R'), Some('B')],
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

fn qualifying_rank(
    entries: [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES],
    score: u32,
) -> Option<u8> {
    entries
        .iter()
        .find(|entry| score > entry.score)
        .map(|entry| entry.rank)
}

fn insert_high_score_entry(
    entries: &mut [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES],
    score: u32,
    initials: [Option<char>; 3],
) -> Option<u8> {
    let rank = qualifying_rank(*entries, score)?;
    let insert_index = usize::from(rank - 1);

    for index in (insert_index + 1..HIGH_SCORE_TABLE_ENTRIES).rev() {
        entries[index] = HighScoreTableEntrySnapshot {
            rank: u8::try_from(index + 1).expect("high-score table rank should fit in u8"),
            ..entries[index - 1]
        };
    }
    entries[insert_index] = HighScoreTableEntrySnapshot {
        rank,
        score,
        initials,
    };

    Some(rank)
}

pub type GameSnapshot = GameState;

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
    UnmappedSoundCommand { command: u8 },
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

#[derive(Debug, Clone)]
pub struct Game {
    state: GameState,
    controls: PlayerControlSystem,
    operator_controls: OperatorControlSystem,
    camera_left: WorldVector,
    pending_wave_start: bool,
    coin_one_credit_delay: Option<u8>,
    coin_one_sound_delay: Option<u8>,
    start_sound_delay: Option<u8>,
    start_playfield_delay: Option<u8>,
    baiter_timer_ticks: Option<u32>,
    baiter_pacing_frames_remaining: u8,
    game_over_candidate_score: Option<u32>,
    player_explosion: Option<PlayerExplosionRuntime>,
    pending_respawn_player: Option<u8>,
    thrust_sound_active: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: initial_state(),
            controls: PlayerControlSystem::new(),
            operator_controls: OperatorControlSystem::new(),
            camera_left: WorldVector::default(),
            pending_wave_start: false,
            coin_one_credit_delay: None,
            coin_one_sound_delay: None,
            start_sound_delay: None,
            start_playfield_delay: None,
            baiter_timer_ticks: None,
            baiter_pacing_frames_remaining: SOURCE_GAME_EXEC_SLEEP_FRAMES,
            game_over_candidate_score: None,
            player_explosion: None,
            pending_respawn_player: None,
            thrust_sound_active: false,
        }
    }

    pub fn state(&self) -> GameState {
        self.state.clone()
    }

    pub fn step(&mut self, input: GameInput) -> GameFrame {
        self.state.frame = self.state.frame.saturating_add(1);
        let mut gameplay_events = Vec::new();
        let mut sound_events = Vec::new();
        let phase_at_start = self.state.phase;
        let normal_attract_at_start = self.normal_attract_active();

        self.state.world.advance_score_popups();
        self.state.world.advance_explosions();
        self.advance_player_explosion();
        self.step_coin_inputs(input, &mut gameplay_events, &mut sound_events);

        self.step_operator_controls(input, &mut gameplay_events);

        self.step_start_sound(&mut sound_events);
        let playfield_started = self.step_start_playfield();
        self.step_start_inputs(input, &mut gameplay_events, &mut sound_events);

        if phase_at_start == GamePhase::GameOver
            || self.state.game_over.hall_of_fame_stall_remaining.is_some()
        {
            self.step_game_over(&mut gameplay_events);
        }

        if phase_at_start == GamePhase::Playing
            && self.state.phase == GamePhase::Playing
            && self.start_playfield_delay.is_none()
            && !playfield_started
        {
            self.step_playing(input, &mut gameplay_events, &mut sound_events);
        }

        if phase_at_start == GamePhase::HighScoreEntry
            && self.state.phase == GamePhase::HighScoreEntry
        {
            self.step_high_score_entry(input, &mut gameplay_events);
        }

        self.sync_world_presentation();
        self.sync_current_player_stock();
        self.sync_attract_presentation(normal_attract_at_start);

        GameFrame {
            state: self.state.clone(),
            events: GameEvents::new(gameplay_events, sound_events),
            scene: self.scene(),
        }
    }

    fn normal_attract_active(&self) -> bool {
        self.state.phase == GamePhase::Attract
            && self.state.game_over.hall_of_fame_stall_remaining.is_none()
    }

    fn sync_world_presentation(&mut self) {
        self.state.world.refresh_object_evidence();
        self.state.world.sync_clean_lifecycle_evidence();
        self.state.world.sync_scanner_radar(
            self.state.phase,
            self.state.frame,
            self.camera_left,
            self.state.player.position,
        );
        self.state.world.player_explosion = self
            .player_explosion
            .as_ref()
            .and_then(PlayerExplosionRuntime::snapshot);
    }

    fn advance_player_explosion(&mut self) {
        if let Some(explosion) = &mut self.player_explosion
            && !explosion.advance_source_frame()
        {
            self.player_explosion = None;
        }
    }

    fn spawn_player_explosion(&mut self, center: ScreenPosition) {
        self.player_explosion = Some(PlayerExplosionRuntime::source_spawn(center));
    }

    fn sync_attract_presentation(&mut self, normal_attract_at_start: bool) {
        if !self.normal_attract_active() {
            self.state.attract = AttractPresentationSnapshot::INACTIVE;
            return;
        }

        let page_frame = if normal_attract_at_start {
            self.state.attract.page_frame.saturating_add(1)
        } else {
            0
        };
        self.state.attract = AttractPresentationSnapshot::for_page_frame(page_frame);
    }

    fn step_coin_inputs(
        &mut self,
        input: GameInput,
        gameplay_events: &mut Vec<GameEvent>,
        sound_events: &mut Vec<SoundEvent>,
    ) {
        if let Some(frames) = self.coin_one_sound_delay {
            let remaining = frames.saturating_sub(1);
            if remaining == 0 {
                self.coin_one_sound_delay = None;
                sound_events.push(SoundEvent::CreditAdded);
            } else {
                self.coin_one_sound_delay = Some(remaining);
            }
        }

        if let Some(frames) = self.coin_one_credit_delay {
            let remaining = frames.saturating_sub(1);
            if remaining == 0 {
                self.coin_one_credit_delay = None;
                self.state.credits = self.state.credits.saturating_add(1);
                gameplay_events.push(GameEvent::CreditAdded);
                self.coin_one_sound_delay = Some(COIN_CREDIT_SOUND_DELAY_FRAMES);
            } else {
                self.coin_one_credit_delay = Some(remaining);
            }
        }

        if input.coin && self.coin_one_credit_delay.is_none() {
            self.coin_one_credit_delay = Some(COIN_CREDIT_DELAY_FRAMES);
        }
    }

    fn step_start_inputs(
        &mut self,
        input: GameInput,
        gameplay_events: &mut Vec<GameEvent>,
        _sound_events: &mut Vec<SoundEvent>,
    ) {
        if self.state.phase != GamePhase::Attract {
            return;
        }

        let Some(player_count) = requested_start_player_count(input) else {
            return;
        };
        if self.state.credits < player_count {
            return;
        }

        self.start_player_game(player_count);
        gameplay_events.push(GameEvent::GameStarted);
        self.start_sound_delay = Some(START_SOUND_DELAY_FRAMES);
        self.start_playfield_delay = Some(START_PLAYFIELD_DELAY_FRAMES);
    }

    fn step_start_sound(&mut self, sound_events: &mut Vec<SoundEvent>) {
        if let Some(frames) = self.start_sound_delay {
            let remaining = frames.saturating_sub(1);
            if remaining == 0 {
                self.start_sound_delay = None;
                sound_events.push(SoundEvent::GameStarted);
            } else {
                self.start_sound_delay = Some(remaining);
            }
        }
    }

    fn start_player_game(&mut self, player_count: u8) {
        self.state.credits = self.state.credits.saturating_sub(player_count);
        self.state.phase = GamePhase::Playing;
        self.state.current_player = 1;
        self.state.player_count = player_count;
        self.state.wave = 1;
        self.state.wave_profile = WaveProfileSnapshot::for_wave(1);
        self.state.player = PlayerSnapshot {
            position: (world_word(0), world_word(0)),
            velocity: (WorldVector::default(), WorldVector::default()),
            direction: Direction::Right,
            lives: DEFAULT_PLAYER_LIVES,
            smart_bombs: 3,
        };
        self.state.player_stocks = [PlayerStockSnapshot::from_player(self.state.player); 2];
        self.state.high_score_initials = HighScoreInitialsState::EMPTY;
        self.state.high_score_entry = None;
        self.state.high_score_submission = None;
        self.state.game_over = GameOverSnapshot::NONE;
        self.state.world = WorldSnapshot::default();
        self.player_explosion = None;
        self.pending_respawn_player = None;
        self.thrust_sound_active = false;
        self.camera_left = WorldVector::default();
        self.controls = PlayerControlSystem::new();
        self.pending_wave_start = false;
        self.baiter_timer_ticks = None;
        self.game_over_candidate_score = None;
    }

    fn step_start_playfield(&mut self) -> bool {
        let Some(frames) = self.start_playfield_delay else {
            return false;
        };

        let remaining = frames.saturating_sub(1);
        if remaining > 0 {
            self.start_playfield_delay = Some(remaining);
            return false;
        }

        self.start_playfield_delay = None;
        self.state.player.position = (world_word(0x2000), world_word(0x8000));
        self.state.player.velocity = (WorldVector::default(), WorldVector::default());
        self.state.player.direction = Direction::Right;
        self.state.player.lives = self.state.player.lives.saturating_sub(1);
        self.sync_current_player_stock();
        self.state.world = WorldSnapshot::first_wave();
        self.player_explosion = None;
        self.thrust_sound_active = false;
        self.camera_left = WorldVector::default();
        self.controls = PlayerControlSystem::new();
        self.pending_wave_start = false;
        self.reset_baiter_timer();
        true
    }

    fn step_operator_controls(&mut self, input: GameInput, gameplay_events: &mut Vec<GameEvent>) {
        let controls = self.operator_controls.step(input);

        if controls.triggers.diagnostics {
            gameplay_events.push(GameEvent::DiagnosticsSelected);
        }

        if controls.triggers.audits {
            gameplay_events.push(GameEvent::AuditsSelected);
        }

        if controls.triggers.high_score_reset {
            self.state.scores.high_score = 0;
            gameplay_events.push(GameEvent::HighScoreReset);
        }
    }

    fn step_high_score_entry(&mut self, input: GameInput, gameplay_events: &mut Vec<GameEvent>) {
        let frame = HighScoreEntrySystem::enter_initial(
            self.state.high_score_initials,
            input.high_score_initial,
            input.high_score_backspace,
        );
        self.state.high_score_initials = frame.state;

        if frame.accepted {
            gameplay_events.push(GameEvent::HighScoreInitialAccepted);
        }

        if frame.submitted {
            if let Some(entry) = self.state.high_score_entry.take() {
                self.submit_high_score_table_entry(entry.score);
                self.state.high_score_submission = Some(HighScoreSubmissionSnapshot {
                    player: self.state.current_player,
                    score: entry.score,
                });
            }
            self.state.phase = GamePhase::GameOver;
            self.state.game_over =
                GameOverSnapshot::hall_of_fame_display(HALL_OF_FAME_STALL_FRAMES);
            gameplay_events.push(GameEvent::HighScoreSubmitted);
        }
    }

    fn step_game_over(&mut self, gameplay_events: &mut Vec<GameEvent>) {
        if let Some(player) = self.pending_respawn_player {
            self.step_pending_respawn(player);
            return;
        }

        if let Some(remaining) = self.state.game_over.player_death_sleep_remaining {
            self.step_player_death_game_over_sleep(remaining, gameplay_events);
            return;
        }

        if let Some(remaining) = self.state.game_over.player_switch_sleep_remaining {
            self.step_player_switch_sleep(remaining);
            return;
        }

        if let Some(remaining) = self.state.game_over.no_entry_delay_remaining {
            self.step_no_entry_delay(remaining);
            return;
        }

        if let Some(remaining) = self.state.game_over.hall_of_fame_stall_remaining {
            self.step_hall_of_fame_display(remaining);
        }
    }

    fn step_pending_respawn(&mut self, player: u8) {
        if self.player_explosion.is_some() {
            return;
        }

        self.pending_respawn_player = None;
        self.start_next_player_turn(player);
    }

    fn step_player_death_game_over_sleep(
        &mut self,
        remaining: u8,
        gameplay_events: &mut Vec<GameEvent>,
    ) {
        let next = remaining.saturating_sub(1);
        if next > 0 {
            self.state.game_over = GameOverSnapshot::player_death_sleep(next);
            return;
        }

        let score = self.game_over_candidate_score.take().unwrap_or_default();
        self.player_explosion = None;
        if let Some(rank) = self.state.high_score_tables.todays_qualifying_rank(score) {
            self.state.phase = GamePhase::HighScoreEntry;
            self.state.high_score_initials = HighScoreInitialsState::EMPTY;
            self.state.high_score_entry = Some(HighScoreEntrySnapshot { score, rank });
            self.state.high_score_submission = None;
            self.state.game_over = GameOverSnapshot::NONE;
            gameplay_events.push(GameEvent::HighScoreEntryStarted);
        } else {
            self.state.game_over =
                GameOverSnapshot::no_entry_delay(HALL_OF_FAME_NO_ENTRY_DELAY_FRAMES);
        }
    }

    fn step_player_switch_sleep(&mut self, remaining: u8) {
        let from_player = self
            .state
            .game_over
            .player_switch_from
            .unwrap_or(self.state.current_player);
        let to_player = self
            .state
            .game_over
            .player_switch_to
            .unwrap_or_else(|| other_player_number(from_player));
        let next = remaining.saturating_sub(1);
        if next > 0 {
            self.state.game_over =
                GameOverSnapshot::player_switch_sleep(next, from_player, to_player);
            return;
        }

        self.start_next_player_turn(to_player);
    }

    fn start_next_player_turn(&mut self, player: u8) {
        let player = player.clamp(1, self.state.player_count.clamp(1, 2));
        let stock = self.state.player_stocks[player_stock_index(player)];
        self.state.phase = GamePhase::Playing;
        self.state.current_player = player;
        self.state.player = PlayerSnapshot {
            position: (WorldVector::default(), WorldVector::default()),
            velocity: (WorldVector::default(), WorldVector::default()),
            direction: Direction::Right,
            lives: stock.lives,
            smart_bombs: stock.smart_bombs,
        };
        self.state.wave = DEFAULT_CABINET_WAVE;
        self.state.wave_profile = WaveProfileSnapshot::for_wave(DEFAULT_CABINET_WAVE);
        self.state.world = WorldSnapshot::default();
        self.player_explosion = None;
        self.thrust_sound_active = false;
        self.pending_respawn_player = None;
        self.state.game_over = GameOverSnapshot::NONE;
        self.state.high_score_entry = None;
        self.state.high_score_submission = None;
        self.camera_left = WorldVector::default();
        self.controls = PlayerControlSystem::new();
        self.pending_wave_start = false;
        self.start_playfield_delay = Some(START_PLAYFIELD_DELAY_FRAMES);
        self.baiter_timer_ticks = None;
        self.game_over_candidate_score = None;
    }

    fn step_no_entry_delay(&mut self, remaining: u8) {
        let next = remaining.saturating_sub(1);
        if next > 0 {
            self.state.game_over = GameOverSnapshot::no_entry_delay(next);
            return;
        }

        self.state.phase = GamePhase::Attract;
        self.state.game_over = GameOverSnapshot::hall_of_fame_display(HALL_OF_FAME_STALL_FRAMES);
    }

    fn step_hall_of_fame_display(&mut self, remaining: u8) {
        let next = remaining.saturating_sub(1);
        self.state.game_over = if next > 0 {
            GameOverSnapshot::hall_of_fame_display(next)
        } else {
            GameOverSnapshot::NONE
        };
        if next == 0 {
            self.state.phase = GamePhase::Attract;
        }
    }

    fn step_playing(
        &mut self,
        input: GameInput,
        gameplay_events: &mut Vec<GameEvent>,
        sound_events: &mut Vec<SoundEvent>,
    ) {
        if self.pending_wave_start {
            self.start_pending_wave(gameplay_events);
        }

        let controls = self.controls.step(input);

        if controls.triggers.reverse {
            self.state.player.direction = match self.state.player.direction {
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
            };
            gameplay_events.push(GameEvent::ReversePressed);
        }

        let previous_camera_left = self.camera_left;
        let motion = PlayerMotionSystem::step(
            PlayerMotionState::new(
                self.state.player.position,
                self.state.player.velocity,
                self.state.player.direction,
                self.camera_left,
            ),
            controls.intent,
        );
        self.state.player.position = motion.state.position;
        self.state.player.velocity = motion.state.velocity;
        self.camera_left = motion.state.camera_left;
        let shell_scroll_delta = source_shell_scroll_delta(previous_camera_left, self.camera_left);
        let mut player_screen_position = motion.screen_position;
        let mut player_velocity = motion.state.velocity;
        let mut player_x = motion.state.position.0;
        let mut hyperspace_death_risk = false;
        self.state
            .world
            .sync_player_carried_humans(player_screen_position);

        self.advance_projectiles();
        self.advance_enemy_projectiles(shell_scroll_delta);

        if controls.triggers.fire {
            gameplay_events.push(GameEvent::FirePressed);
            if let ProjectileLaunchOutcome::Started {
                direction, spawn, ..
            } = ProjectileSystem::try_launch(
                ProjectileState::new(self.state.world.projectiles.len() as u8),
                motion.screen_position,
                self.state.player.direction,
            ) {
                self.state.world.projectiles.push(ProjectileSnapshot {
                    position: spawn,
                    velocity: ProjectileMotionSystem::velocity_for_direction(direction),
                });
                sound_events.push(source_laser_fire_sound_event());
            }
        }

        if controls.triggers.smart_bomb && self.state.player.smart_bombs > 0 {
            self.state.player.smart_bombs -= 1;
            gameplay_events.push(GameEvent::SmartBombPressed);
            sound_events.push(source_smart_bomb_sound_event());
            self.resolve_smart_bomb(gameplay_events, sound_events);
        }

        if controls.triggers.hyperspace {
            gameplay_events.push(GameEvent::HyperspacePressed);
            // Source HYP02 walks KILSHL over the shell-object list before
            // rematerialization; clean enemy projectiles model that list.
            self.state.world.enemy_projectiles.clear();
            let hyperspace = source_hyperspace_rematerialization(
                self.state.world.source_rng,
                self.state.player.position.1,
            );
            self.state.player.position = hyperspace.position;
            self.state.player.velocity = hyperspace.velocity;
            self.state.player.direction = hyperspace.direction;
            self.camera_left = hyperspace.camera_left;
            player_screen_position = hyperspace.screen_position;
            player_velocity = hyperspace.velocity;
            player_x = hyperspace.position.0;
            hyperspace_death_risk = hyperspace.death_risk;
            sound_events.push(source_hyperspace_appearance_sound_event());
        }

        self.step_thrust_sound(
            controls.intent.thrust,
            controls.triggers.thrust,
            sound_events,
        );

        self.state.world.advance_source_astronaut_process();

        self.state.world.advance_enemies(
            self.state.wave_profile,
            player_screen_position,
            player_velocity,
            sound_events,
        );

        self.state.world.resolve_lander_human_abductions(
            self.state.wave_profile,
            player_screen_position,
            player_velocity,
            sound_events,
        );
        let falling_advance = self.state.world.advance_falling_humans();
        for landing_position in falling_advance.safe_landings {
            self.state.world.spawn_score_popup(
                ScorePopupKind::Points250,
                clean_rescue_score_popup_position(landing_position),
            );
            self.award_safe_landing_score(gameplay_events);
            sound_events.push(source_astronaut_safe_landing_sound_event());
        }
        for landing_position in falling_advance.fatal_landings {
            self.state
                .world
                .spawn_explosion(ExplosionKind::Astronaut, landing_position);
            sound_events.push(source_astronaut_hit_sound_event());
        }
        if self
            .state
            .world
            .resolve_player_human_rescue(player_screen_position)
        {
            self.award_rescue_score(gameplay_events);
            sound_events.push(source_astronaut_catch_sound_event());
        }
        self.advance_baiter_entry(player_screen_position, player_velocity);
        self.resolve_projectile_enemy_collisions(gameplay_events, sound_events);
        let mut hit_player = self.resolve_player_enemy_collision(
            player_screen_position,
            gameplay_events,
            sound_events,
        );
        if !hit_player && self.state.phase == GamePhase::Playing {
            hit_player = self.resolve_player_enemy_projectile_collision(
                player_screen_position,
                gameplay_events,
                sound_events,
            );
        }
        if hyperspace_death_risk && !hit_player && self.state.phase == GamePhase::Playing {
            self.apply_player_hit(player_screen_position, gameplay_events, sound_events);
        }
        self.state.world.advance_terrain_blow(sound_events);
        self.resolve_planet_destruction(sound_events);

        if self.state.phase == GamePhase::Playing && !hyperspace_death_risk {
            self.queue_wave_clear_if_needed(gameplay_events, player_x);
        }
    }

    fn step_thrust_sound(
        &mut self,
        thrust_held: bool,
        thrust_pressed: bool,
        sound_events: &mut Vec<SoundEvent>,
    ) {
        if thrust_pressed {
            self.thrust_sound_active = true;
            sound_events.push(SoundEvent::ThrustStarted);
        } else if self.thrust_sound_active && !thrust_held {
            self.thrust_sound_active = false;
            sound_events.push(SoundEvent::ThrustStopped);
        }
    }

    fn reset_baiter_timer(&mut self) {
        self.baiter_timer_ticks = Some(self.state.wave_profile.baiter_delay);
        self.baiter_pacing_frames_remaining = SOURCE_GAME_EXEC_SLEEP_FRAMES;
    }

    fn advance_baiter_entry(
        &mut self,
        player_position: ScreenPosition,
        player_velocity: (WorldVector, WorldVector),
    ) {
        let Some(timer_ticks) = self.baiter_timer_ticks else {
            return;
        };
        let enemy_total = self.state.world.source_enemy_total();
        if enemy_total == 0 {
            return;
        }

        if self.baiter_pacing_frames_remaining > 1 {
            self.baiter_pacing_frames_remaining =
                self.baiter_pacing_frames_remaining.saturating_sub(1);
            return;
        }
        self.baiter_pacing_frames_remaining = SOURCE_GAME_EXEC_SLEEP_FRAMES;

        let timer_ticks = source_baiter_accelerated_timer_ticks(
            timer_ticks,
            self.state.wave_profile,
            enemy_total,
        );
        let decremented_ticks = timer_ticks.saturating_sub(1);
        if decremented_ticks > 0 {
            self.baiter_timer_ticks = Some(decremented_ticks);
            return;
        }

        self.baiter_timer_ticks = Some(source_baiter_reset_timer_ticks(
            self.state.wave_profile,
            enemy_total,
        ));
        self.state
            .world
            .spawn_baiter(self.state.wave_profile, player_position, player_velocity);
    }

    fn advance_projectiles(&mut self) {
        self.state.world.projectiles.retain_mut(|projectile| {
            let motion = ProjectileMotionSystem::step(projectile.position, projectile.velocity);
            if motion.active {
                projectile.position = motion.position;
                projectile.velocity = motion.velocity;
            }
            motion.active
        });
    }

    fn advance_enemy_projectiles(&mut self, shell_scroll_delta: u16) {
        self.state.world.enemy_projectiles.retain_mut(|projectile| {
            projectile.source_lifetime_ticks = projectile.source_lifetime_ticks.wrapping_sub(1);
            if projectile.source_lifetime_ticks == 0 {
                return false;
            }

            let [x, x_fraction] =
                u16::from_be_bytes([projectile.position.x, projectile.source_x_fraction])
                    .wrapping_add(shell_scroll_delta)
                    .wrapping_add(projectile.source_x_velocity)
                    .to_be_bytes();
            if x >= SOURCE_SHELL_X_MAX {
                return false;
            }

            let [y, y_fraction] =
                u16::from_be_bytes([projectile.position.y, projectile.source_y_fraction])
                    .wrapping_add(projectile.source_y_velocity)
                    .to_be_bytes();
            if y <= SOURCE_PLAYFIELD_Y_MIN {
                return false;
            }

            projectile.position = ScreenPosition::new(x, y);
            projectile.source_x_fraction = x_fraction;
            projectile.source_y_fraction = y_fraction;
            projectile.velocity =
                source_screen_velocity(projectile.source_x_velocity, projectile.source_y_velocity);
            true
        });
    }

    fn resolve_projectile_enemy_collisions(
        &mut self,
        gameplay_events: &mut Vec<GameEvent>,
        sound_events: &mut Vec<SoundEvent>,
    ) {
        let projectile_boxes = self
            .state
            .world
            .projectiles
            .iter()
            .map(|projectile| {
                CollisionBox::new(projectile.position, PLAYER_PROJECTILE_COLLISION_SIZE)
            })
            .collect::<Vec<_>>();
        let enemy_boxes = self
            .state
            .world
            .enemies
            .iter()
            .map(|enemy| CollisionBox::new(enemy.position, enemy_collision_size(*enemy)))
            .collect::<Vec<_>>();

        let Some(hit) =
            CollisionSystem::first_projectile_enemy_hit(&projectile_boxes, &enemy_boxes)
        else {
            return;
        };

        let enemy = self.state.world.enemies.remove(hit.enemy_index);
        self.state.world.projectiles.remove(hit.projectile_index);
        self.destroy_enemy(enemy, gameplay_events, sound_events);
    }

    fn resolve_smart_bomb(
        &mut self,
        gameplay_events: &mut Vec<GameEvent>,
        sound_events: &mut Vec<SoundEvent>,
    ) {
        let outcome = SmartBombSystem::detonate(self.state.world.enemies.len());
        let destroyed = self
            .state
            .world
            .enemies
            .drain(..outcome.destroyed_enemies)
            .collect::<Vec<_>>();

        for enemy in destroyed {
            self.destroy_enemy(enemy, gameplay_events, sound_events);
        }
    }

    fn destroy_enemy(
        &mut self,
        enemy: EnemySnapshot,
        gameplay_events: &mut Vec<GameEvent>,
        sound_events: &mut Vec<SoundEvent>,
    ) {
        let released_lander_passenger = if enemy.kind == EnemyKind::Lander {
            self.state
                .world
                .release_passenger_for_lander(enemy.position)
        } else {
            false
        };
        self.state
            .world
            .spawn_explosion(ExplosionKind::for_enemy(enemy.kind), enemy.position);
        gameplay_events.push(GameEvent::EnemyDestroyed);
        self.award_enemy_score(enemy.kind, gameplay_events);
        if released_lander_passenger {
            sound_events.push(source_astronaut_release_sound_event());
        } else {
            sound_events.push(source_enemy_hit_sound_event(enemy.kind));
        }

        match enemy.kind {
            EnemyKind::Lander => {}
            EnemyKind::Pod => {
                self.state
                    .world
                    .spawn_pod_swarmers(enemy.position, self.state.wave_profile);
            }
            EnemyKind::Mutant | EnemyKind::Bomber | EnemyKind::Swarmer | EnemyKind::Baiter => {}
        }
    }

    fn resolve_player_enemy_collision(
        &mut self,
        player_position: ScreenPosition,
        gameplay_events: &mut Vec<GameEvent>,
        sound_events: &mut Vec<SoundEvent>,
    ) -> bool {
        let player = CollisionBox::new(player_position, PLAYER_COLLISION_SIZE);
        let enemy_boxes = self
            .state
            .world
            .enemies
            .iter()
            .map(|enemy| CollisionBox::new(enemy.position, enemy_collision_size(*enemy)))
            .collect::<Vec<_>>();

        let Some(hit) = CollisionSystem::first_player_enemy_hit(player, &enemy_boxes) else {
            return false;
        };

        let enemy = self.state.world.enemies.remove(hit.enemy_index);
        if enemy.kind == EnemyKind::Lander {
            self.state
                .world
                .release_passenger_for_lander(enemy.position);
        }
        self.apply_player_hit(player_position, gameplay_events, sound_events);
        true
    }

    fn resolve_player_enemy_projectile_collision(
        &mut self,
        player_position: ScreenPosition,
        gameplay_events: &mut Vec<GameEvent>,
        sound_events: &mut Vec<SoundEvent>,
    ) -> bool {
        let player = CollisionBox::new(player_position, PLAYER_COLLISION_SIZE);
        let projectile_boxes = self
            .state
            .world
            .enemy_projectiles
            .iter()
            .map(|projectile| {
                CollisionBox::new(projectile.position, ENEMY_PROJECTILE_COLLISION_SIZE)
            })
            .collect::<Vec<_>>();

        let Some(hit) = CollisionSystem::first_player_enemy_hit(player, &projectile_boxes) else {
            return false;
        };

        let projectile = self.state.world.enemy_projectiles.remove(hit.enemy_index);
        self.state
            .world
            .spawn_explosion(ExplosionKind::Bomb, projectile.position);
        self.award_points(SOURCE_ENEMY_PROJECTILE_SCORE_POINTS, gameplay_events);
        sound_events.push(source_bomb_collision_sound_event());
        self.apply_player_hit(player_position, gameplay_events, sound_events);
        true
    }

    fn apply_player_hit(
        &mut self,
        player_position: ScreenPosition,
        gameplay_events: &mut Vec<GameEvent>,
        sound_events: &mut Vec<SoundEvent>,
    ) {
        self.thrust_sound_active = false;
        self.pending_respawn_player = None;
        self.spawn_player_explosion(player_position.wrapping_offset(4, 3));
        sound_events.push(source_player_death_sound_event());
        let frame = PlayerDamageSystem::apply_hit(PlayerStock::new(
            self.state.player.lives,
            self.state.player.smart_bombs,
        ));
        self.state.player.lives = frame.stock.lives;
        self.state.player.smart_bombs = frame.stock.smart_bombs;
        gameplay_events.push(GameEvent::PlayerDestroyed);

        if frame.game_over {
            self.pending_wave_start = false;
            self.sync_current_player_stock();
            self.state.phase = GamePhase::GameOver;
            self.state.high_score_entry = None;
            self.state.high_score_submission = None;
            if let Some(next_player) = self.next_surviving_player() {
                self.pending_respawn_player = None;
                self.state.game_over = GameOverSnapshot::player_switch_sleep(
                    PLAYER_SWITCH_SLEEP_FRAMES,
                    self.state.current_player,
                    next_player,
                );
            } else {
                gameplay_events.push(GameEvent::GameOver);
                let current_score = self.current_player_score();
                self.game_over_candidate_score = Some(current_score);
                self.state.game_over =
                    GameOverSnapshot::player_death_sleep(PLAYER_DEATH_GAME_OVER_SLEEP_FRAMES);
            }
        } else {
            self.pending_wave_start = false;
            self.sync_current_player_stock();
            let Some(next_player) = self.next_respawn_player() else {
                return;
            };
            self.state.phase = GamePhase::GameOver;
            self.state.high_score_entry = None;
            self.state.high_score_submission = None;
            self.state.game_over = GameOverSnapshot::NONE;
            self.pending_respawn_player = Some(next_player);
        }
    }

    fn resolve_planet_destruction(&mut self, sound_events: &mut Vec<SoundEvent>) {
        if self.state.world.humans.is_empty() && self.state.world.terrain_blow.is_none() {
            self.state.world.start_terrain_blow();
            sound_events.push(source_terrain_blow_start_sound_event());
        }
    }

    fn queue_wave_clear_if_needed(
        &mut self,
        gameplay_events: &mut Vec<GameEvent>,
        player_x: WorldVector,
    ) {
        if self.state.world.activate_enemy_reserve_batch(
            self.state.wave_profile,
            source_word_from_world_vector(player_x),
            source_word_from_world_vector(self.camera_left),
        ) {
            return;
        }

        if matches!(
            WaveSystem::evaluate(WaveState::new(
                self.state.wave,
                self.state.world.source_enemy_total()
            )),
            WaveStatus::Cleared { .. }
        ) {
            self.state.world.clear_source_non_wave_enemies();
            self.pending_wave_start = true;
            gameplay_events.push(GameEvent::WaveCleared);
        }
    }

    fn start_pending_wave(&mut self, gameplay_events: &mut Vec<GameEvent>) {
        let next_wave = WaveSystem::next_wave(self.state.wave);

        self.state.wave = next_wave;
        self.state.wave_profile = WaveProfileSnapshot::for_wave(next_wave);
        self.state.world = WorldSnapshot::for_wave(next_wave);
        self.player_explosion = None;
        self.thrust_sound_active = false;
        self.pending_wave_start = false;
        self.reset_baiter_timer();
        gameplay_events.push(GameEvent::WaveStarted);
    }

    fn award_enemy_score(&mut self, kind: EnemyKind, gameplay_events: &mut Vec<GameEvent>) {
        self.award_points(enemy_score(kind), gameplay_events);
    }

    fn award_rescue_score(&mut self, gameplay_events: &mut Vec<GameEvent>) {
        self.award_points(SOURCE_RESCUE_SCORE_POINTS, gameplay_events);
    }

    fn award_safe_landing_score(&mut self, gameplay_events: &mut Vec<GameEvent>) {
        self.award_points(SOURCE_SAFE_LANDING_SCORE_POINTS, gameplay_events);
    }

    fn award_points(&mut self, points: u32, gameplay_events: &mut Vec<GameEvent>) {
        let frame = ScoreSystem::award_points(
            self.state.scores,
            PlayerStock::new(self.state.player.lives, self.state.player.smart_bombs),
            self.state.current_player,
            points,
        );
        self.state.scores = frame.scores;
        self.state.player.lives = frame.stock.lives;
        self.state.player.smart_bombs = frame.stock.smart_bombs;

        if frame.bonus_awards > 0 {
            gameplay_events.push(GameEvent::BonusAwarded);
        }
    }

    fn current_player_score(&self) -> u32 {
        if self.state.current_player == 2 {
            self.state.scores.player_two
        } else {
            self.state.scores.player_one
        }
    }

    fn submit_high_score_table_entry(&mut self, score: u32) {
        let initials = self.state.high_score_initials.initials;
        self.state
            .high_score_tables
            .insert_all_time(score, initials);
        self.state
            .high_score_tables
            .insert_todays_greatest(score, initials);
        self.state.scores.high_score = self.state.high_score_tables.all_time[0].score;
    }

    fn sync_current_player_stock(&mut self) {
        self.state.player_stocks[player_stock_index(self.state.current_player)] =
            PlayerStockSnapshot::from_player(self.state.player);
    }

    fn next_surviving_player(&self) -> Option<u8> {
        let next_player = self.next_respawn_player()?;
        (next_player != self.state.current_player).then_some(next_player)
    }

    fn next_respawn_player(&self) -> Option<u8> {
        let player_count = self.state.player_count.clamp(1, 2);
        if player_count < 2 {
            let current = self.state.current_player.clamp(1, 1);
            return (self.state.player_stocks[player_stock_index(current)].lives > 0)
                .then_some(current);
        }

        let current = self.state.current_player.clamp(1, player_count);
        for offset in 1..=player_count {
            let mut candidate = current.saturating_add(offset);
            if candidate > player_count {
                candidate = candidate.saturating_sub(player_count);
            }
            if self.state.player_stocks[player_stock_index(candidate)].lives > 0 {
                return Some(candidate);
            }
        }

        None
    }

    fn scene(&self) -> RenderScene {
        let mut scene = RenderScene::empty(self.state.frame, SurfaceSize::new(292, 240));
        #[cfg(test)]
        {
            scene.visual_signature = accepted_r3_visual_signature(self.state.frame);
        }
        for star in &self.state.world.stars {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::STAR,
                layer: RenderLayer::Starfield,
                position: [f32::from(star.x), f32::from(star.y)],
                size: [1.0, 1.0],
                tint: Color::WHITE,
            });
        }
        if self.state.phase == GamePhase::Playing && self.state.world.terrain_blow.is_none() {
            push_source_bgout_terrain_sprites(&mut scene);
        }
        if self.state.phase != GamePhase::Attract {
            push_score_sprites(&mut scene, self.state.scores, self.state.player_count);
        }
        push_top_display_border_sprites(&mut scene, &self.state);
        push_attract_scoring_action_sprites(&mut scene, &self.state);
        push_attract_credit_sprites(&mut scene, &self.state);
        push_attract_presents_sprites(&mut scene, &self.state);
        push_attract_instruction_text_sprites(&mut scene, &self.state);
        push_attract_williams_logo_sprite(&mut scene, &self.state);
        push_attract_defender_wordmark_sprite(&mut scene, &self.state);
        push_attract_copyright_strip_sprite(&mut scene, &self.state);
        push_final_game_over_prompt_sprites(&mut scene, self.state.game_over);
        push_player_switch_prompt_sprites(&mut scene, self.state.game_over);
        push_player_start_prompt_sprites(&mut scene, &self.state);
        push_wave_completion_status_sprites(&mut scene, &self.state);
        push_survivor_bonus_icon_sprites(&mut scene, &self.state);
        push_high_score_entry_prompt_sprites(&mut scene, &self.state);
        push_hall_of_fame_display_sprites(&mut scene, &self.state);
        push_player_explosion_cloud_sprites(&mut scene, self.state.world.player_explosion.as_ref());

        if self.state.phase == GamePhase::Playing {
            push_stock_sprites(
                &mut scene,
                self.state.player_count,
                self.state.player_stocks,
            );
            push_scanner_radar_sprites(&mut scene, &self.state.world.scanner);
            push_source_object_detail_sprites(&mut scene, &self.state.world.object_evidence);
            push_expanded_object_detail_sprites(&mut scene, &self.state.world.expanded_objects);

            for enemy in &self.state.world.enemies {
                let size = enemy_sprite_size(enemy.kind);
                scene.push_sprite(SceneSprite {
                    sprite: enemy_sprite(enemy.kind),
                    layer: RenderLayer::Objects,
                    position: [f32::from(enemy.position.x), f32::from(enemy.position.y)],
                    size: [f32::from(size.0), f32::from(size.1)],
                    tint: Color::WHITE,
                });
            }
            for human in &self.state.world.humans {
                scene.push_sprite(SceneSprite {
                    sprite: SpriteId::HUMAN,
                    layer: RenderLayer::Objects,
                    position: [f32::from(human.position.x), f32::from(human.position.y)],
                    size: [
                        f32::from(HUMAN_SPRITE_SIZE.0),
                        f32::from(HUMAN_SPRITE_SIZE.1),
                    ],
                    tint: if human.carried || human.carried_by_player {
                        Color::from_rgba(0xFF, 0xF8, 0x80, 0xFF)
                    } else {
                        Color::WHITE
                    },
                });
            }
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::PLAYER_SHIP,
                layer: RenderLayer::Objects,
                position: [
                    world_vector_pixels(self.state.player.position.0),
                    world_vector_pixels(self.state.player.position.1),
                ],
                size: [
                    f32::from(PLAYER_SPRITE_SIZE.0),
                    f32::from(PLAYER_SPRITE_SIZE.1),
                ],
                tint: Color::WHITE,
            });

            for projectile in &self.state.world.projectiles {
                scene.push_sprite(SceneSprite {
                    sprite: SpriteId::PLAYER_PROJECTILE,
                    layer: RenderLayer::Projectiles,
                    position: [
                        f32::from(projectile.position.x),
                        f32::from(projectile.position.y),
                    ],
                    size: [
                        f32::from(PROJECTILE_SPRITE_SIZE.0),
                        f32::from(PROJECTILE_SPRITE_SIZE.1),
                    ],
                    tint: Color::WHITE,
                });
            }
            for projectile in &self.state.world.enemy_projectiles {
                scene.push_sprite(SceneSprite {
                    sprite: SpriteId::ENEMY_BOMB,
                    layer: RenderLayer::Projectiles,
                    position: [
                        f32::from(projectile.position.x),
                        f32::from(projectile.position.y),
                    ],
                    size: [
                        f32::from(ENEMY_PROJECTILE_SPRITE_SIZE.0),
                        f32::from(ENEMY_PROJECTILE_SPRITE_SIZE.1),
                    ],
                    tint: Color::WHITE,
                });
            }
        }

        scene
    }
}

fn push_source_message_sprites_with_tint(
    scene: &mut RenderScene,
    text: &str,
    origin: [f32; 2],
    layer: RenderLayer,
    tint: Color,
) {
    let first_sprite = scene.sprites.len();
    push_source_message_sprites(scene, text, origin, layer);
    tint_new_sprites(scene, first_sprite, tint);
}

fn push_source_text_bytes_sprites_with_tint(
    scene: &mut RenderScene,
    bytes: &[u8],
    origin: [f32; 2],
    layer: RenderLayer,
    tint: Color,
) {
    let first_sprite = scene.sprites.len();
    push_source_text_bytes_sprites(scene, bytes, origin, layer);
    tint_new_sprites(scene, first_sprite, tint);
}

fn push_source_controlled_message_sprites_with_tint(
    scene: &mut RenderScene,
    text: &str,
    top_left_screen_address: u16,
    layer: RenderLayer,
    tint: Color,
) {
    let first_sprite = scene.sprites.len();
    push_source_controlled_message_sprites(scene, text, top_left_screen_address, layer);
    tint_new_sprites(scene, first_sprite, tint);
}

fn push_source_controlled_message_sprites_with_tint_and_offset(
    scene: &mut RenderScene,
    text: &str,
    top_left_screen_address: u16,
    layer: RenderLayer,
    tint: Color,
    offset: [f32; 2],
) {
    let first_sprite = scene.sprites.len();
    push_source_controlled_message_sprites_with_tint(
        scene,
        text,
        top_left_screen_address,
        layer,
        tint,
    );
    offset_new_sprites(scene, first_sprite, offset);
}

fn tint_new_sprites(scene: &mut RenderScene, first_sprite: usize, tint: Color) {
    for sprite in &mut scene.sprites[first_sprite..] {
        sprite.tint = tint;
    }
}

fn offset_new_sprites(scene: &mut RenderScene, first_sprite: usize, offset: [f32; 2]) {
    if offset == [0.0, 0.0] {
        return;
    }

    for sprite in &mut scene.sprites[first_sprite..] {
        sprite.position = offset_position(sprite.position, offset);
    }
}

fn attract_title_reference_sample_index(page_frame: u16) -> usize {
    usize::from(page_frame / ATTRACT_TITLE_REFERENCE_SAMPLE_STEP_FRAMES).saturating_sub(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AttractScoringLegendEntry {
    kind: EnemyKind,
    table_x16: i32,
    table_y16: i32,
    scanner_color_word: u16,
}

const ATTRACT_SCORING_LEGEND: [AttractScoringLegendEntry;
    ATTRACT_SCORING_LEGEND_ENTRY_COUNT as usize] = [
    AttractScoringLegendEntry {
        kind: EnemyKind::Lander,
        table_x16: 0x0900,
        table_y16: 0x6000,
        scanner_color_word: 0x4433,
    },
    AttractScoringLegendEntry {
        kind: EnemyKind::Mutant,
        table_x16: 0x1100,
        table_y16: 0x6000,
        scanner_color_word: 0xCC33,
    },
    AttractScoringLegendEntry {
        kind: EnemyKind::Baiter,
        table_x16: 0x1980,
        table_y16: 0x6200,
        scanner_color_word: 0x3333,
    },
    AttractScoringLegendEntry {
        kind: EnemyKind::Bomber,
        table_x16: 0x0960,
        table_y16: 0x9800,
        scanner_color_word: 0x8888,
    },
    AttractScoringLegendEntry {
        kind: EnemyKind::Pod,
        table_x16: 0x1160,
        table_y16: 0x9800,
        scanner_color_word: 0xCCCC,
    },
    AttractScoringLegendEntry {
        kind: EnemyKind::Swarmer,
        table_x16: 0x19E0,
        table_y16: 0x9A00,
        scanner_color_word: 0x2424,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttractScoringDemoStage {
    RescueDescend,
    RescueAscend,
    RescueLaser,
    RescueFall,
    RescueScore,
    RescueReturn,
    LegendApproach(usize),
    LegendLaser(usize),
    LegendTransfer(usize),
    LegendReveal(usize),
    LegendHold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttractScoringObjectKind {
    PlayerShip,
    Human,
    PlayerShot,
    Enemy(EnemyKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttractScoringVisual {
    Sprite,
    Explosion,
    Materialize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AttractScoringObject {
    kind: AttractScoringObjectKind,
    x16: i32,
    y16: i32,
    visual: AttractScoringVisual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AttractScoringBonus {
    sprite: SpriteId,
    x16: i32,
    y16: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttractScoringFrame {
    demo_tick: u16,
    objects: Vec<AttractScoringObject>,
    scanner_objects: Vec<AttractScoringObject>,
    bonus: Option<AttractScoringBonus>,
}

fn push_attract_credit_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if state.credits == 0
        && !matches!(
            state.attract.page,
            AttractPresentationPage::HallOfFame | AttractPresentationPage::ScoringSequence
        )
    {
        return;
    }

    let visual_offset = attract_page_visual_offset(state);
    if let Some(text) = source_message_text("CREDV") {
        push_source_message_sprites_with_tint(
            scene,
            text,
            offset_position(
                source_screen_position(SOURCE_ATTRACT_CREDITS_LABEL_SCREEN),
                visual_offset,
            ),
            RenderLayer::Overlay,
            attract_credit_text_tint(state),
        );
    }

    let (digits, digit_count) = attract_credit_digits(state.credits);
    push_source_text_bytes_sprites_with_tint(
        scene,
        &digits[..digit_count],
        offset_position(
            source_screen_position(SOURCE_ATTRACT_CREDITS_NUMBER_SCREEN),
            visual_offset,
        ),
        RenderLayer::Overlay,
        attract_credit_text_tint(state),
    );
}

pub(crate) fn attract_credit_text_tint(state: &GameState) -> Color {
    if let Some(scoring_tick) = state.attract.scoring_sequence_frame() {
        return attract_scoring_color_cycle_tint(scoring_tick);
    }

    if state.attract.shows_hall_of_fame() {
        return SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint();
    }

    SOURCE_VISUAL_STATE.attract_title_text_tint_for_frame(state.attract.page_frame)
}

pub(crate) fn attract_page_visual_offset(state: &GameState) -> [f32; 2] {
    if state.phase != GamePhase::Attract {
        return [0.0, 0.0];
    }

    if state.attract.shows_hall_of_fame() {
        return ATTRACT_HALL_OF_FAME_REFERENCE_OFFSET;
    }

    if state.attract.scoring_sequence_frame().is_some() {
        return ATTRACT_REFERENCE_SCENE_OFFSET;
    }

    [0.0, 0.0]
}

fn offset_position(position: [f32; 2], offset: [f32; 2]) -> [f32; 2] {
    [position[0] + offset[0], position[1] + offset[1]]
}

fn attract_credit_digits(credits: u8) -> ([u8; 2], usize) {
    let credits = credits.min(99);
    if credits < 10 {
        ([b'0' + credits, b' '], 1)
    } else {
        ([b'0' + credits / 10, b'0' + credits % 10], 2)
    }
}

fn push_attract_presents_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if !state.attract.shows_presents_text() {
        return;
    }

    if let Some(text) = source_message_text("ELECV") {
        push_source_controlled_message_sprites_with_tint(
            scene,
            text,
            SOURCE_ATTRACT_PRESENTS_ELECTRONICS_SCREEN,
            RenderLayer::Overlay,
            SOURCE_VISUAL_STATE.attract_title_text_tint_for_frame(state.attract.page_frame),
        );
    }
}

fn push_attract_instruction_text_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if !state.attract.shows_instruction_text() {
        return;
    }

    let scoring_tick = state.attract.scoring_sequence_frame();
    let visible_line_count = scoring_tick
        .map_or(SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES.len(), |tick| {
            1 + attract_scoring_visible_legend_text_entries(attract_scoring_display_tick(tick))
        });
    let scoring_tint = scoring_tick.map(attract_scoring_color_cycle_tint);
    let visual_offset = scoring_tick
        .map(|_| attract_page_visual_offset(state))
        .unwrap_or([0.0, 0.0]);

    for (label, screen_address) in SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES
        .iter()
        .take(visible_line_count)
    {
        if let Some(text) = source_message_text(label) {
            push_source_controlled_message_sprites_with_tint_and_offset(
                scene,
                text,
                *screen_address,
                RenderLayer::Overlay,
                scoring_tint.unwrap_or_else(|| {
                    SOURCE_VISUAL_STATE.attract_instruction_text_tint(*screen_address)
                }),
                visual_offset,
            );
        }
    }
}

fn attract_scoring_frame(scoring_tick: u16) -> AttractScoringFrame {
    let display_tick = attract_scoring_display_tick(scoring_tick);
    let (stage, local_tick) = attract_scoring_demo_stage_for_tick(display_tick);
    let objects = attract_scoring_objects_for_stage(stage, local_tick);
    let scanner_tick = display_tick - (display_tick % 4);
    let (scanner_stage, scanner_local_tick) = attract_scoring_demo_stage_for_tick(scanner_tick);
    let scanner_objects = attract_scoring_objects_for_stage(scanner_stage, scanner_local_tick);

    AttractScoringFrame {
        demo_tick: scoring_tick,
        objects,
        scanner_objects,
        bonus: attract_scoring_bonus_for_stage(stage, local_tick),
    }
}

fn attract_scoring_display_tick(scoring_tick: u16) -> u16 {
    (scoring_tick % ATTRACT_SCORING_DEMO_TOTAL_TICKS + ATTRACT_SCORING_PROTECTED_DEMO_TICK_OFFSET)
        % ATTRACT_SCORING_DEMO_TOTAL_TICKS
}

#[cfg(test)]
fn attract_scoring_page_tick_for_display_tick(display_tick: u16) -> u16 {
    (display_tick % ATTRACT_SCORING_DEMO_TOTAL_TICKS + ATTRACT_SCORING_DEMO_TOTAL_TICKS
        - ATTRACT_SCORING_PROTECTED_DEMO_TICK_OFFSET)
        % ATTRACT_SCORING_DEMO_TOTAL_TICKS
}

fn attract_scoring_demo_stage_for_tick(mut tick: u16) -> (AttractScoringDemoStage, u16) {
    for (stage, duration) in ATTRACT_SCORING_RESCUE_TIMELINE {
        if tick < duration {
            return (stage, tick);
        }
        tick -= duration;
    }

    for index in 0..ATTRACT_SCORING_LEGEND.len() {
        for (stage, duration) in attract_scoring_legend_timeline(index) {
            if tick < duration {
                return (stage, tick);
            }
            tick -= duration;
        }
    }

    (
        AttractScoringDemoStage::LegendHold,
        tick.min(ATTRACT_SCORING_LEGEND_HOLD_TICKS.saturating_sub(1)),
    )
}

fn attract_scoring_demo_tick_for_stage(
    target_stage: AttractScoringDemoStage,
    local_tick: u16,
) -> u16 {
    let mut elapsed = 0;
    for (stage, duration) in ATTRACT_SCORING_RESCUE_TIMELINE {
        if stage == target_stage {
            return elapsed + local_tick.min(duration.saturating_sub(1));
        }
        elapsed += duration;
    }

    for index in 0..ATTRACT_SCORING_LEGEND.len() {
        for (stage, duration) in attract_scoring_legend_timeline(index) {
            if stage == target_stage {
                return elapsed + local_tick.min(duration.saturating_sub(1));
            }
            elapsed += duration;
        }
    }

    elapsed + local_tick.min(ATTRACT_SCORING_LEGEND_HOLD_TICKS.saturating_sub(1))
}

const ATTRACT_SCORING_RESCUE_TIMELINE: [(AttractScoringDemoStage, u16); 6] = [
    (
        AttractScoringDemoStage::RescueDescend,
        ATTRACT_SCORING_RESCUE_DESCENT_TICKS,
    ),
    (
        AttractScoringDemoStage::RescueAscend,
        ATTRACT_SCORING_RESCUE_ASCENT_TICKS,
    ),
    (
        AttractScoringDemoStage::RescueLaser,
        ATTRACT_SCORING_RESCUE_LASER_TICKS,
    ),
    (
        AttractScoringDemoStage::RescueFall,
        ATTRACT_SCORING_RESCUE_FALL_TICKS,
    ),
    (
        AttractScoringDemoStage::RescueScore,
        ATTRACT_SCORING_RESCUE_SCORE_TICKS,
    ),
    (
        AttractScoringDemoStage::RescueReturn,
        ATTRACT_SCORING_RESCUE_RETURN_TICKS,
    ),
];

fn attract_scoring_legend_timeline(index: usize) -> [(AttractScoringDemoStage, u16); 4] {
    [
        (
            AttractScoringDemoStage::LegendApproach(index),
            ATTRACT_SCORING_LEGEND_APPROACH_TICKS,
        ),
        (
            AttractScoringDemoStage::LegendLaser(index),
            ATTRACT_SCORING_LEGEND_LASER_TICKS,
        ),
        (
            AttractScoringDemoStage::LegendTransfer(index),
            ATTRACT_SCORING_LEGEND_TRANSFER_TICKS,
        ),
        (
            AttractScoringDemoStage::LegendReveal(index),
            ATTRACT_SCORING_LEGEND_REVEAL_TICKS,
        ),
    ]
}

fn attract_scoring_visible_legend_text_entries(scoring_tick: u16) -> usize {
    ATTRACT_SCORING_LEGEND
        .iter()
        .enumerate()
        .take_while(|(index, _)| {
            let reveal_tick = attract_scoring_demo_tick_for_stage(
                AttractScoringDemoStage::LegendReveal(*index),
                0,
            );
            scoring_tick >= next_attract_scoring_text_process_tick(reveal_tick)
        })
        .count()
}

fn next_attract_scoring_text_process_tick(tick: u16) -> u16 {
    let remainder = tick % 6;
    if remainder == 0 {
        tick
    } else {
        tick + (6 - remainder)
    }
}

fn attract_scoring_revealed_table_entries(stage: AttractScoringDemoStage) -> usize {
    match stage {
        AttractScoringDemoStage::LegendReveal(index) => index + 1,
        AttractScoringDemoStage::LegendHold => ATTRACT_SCORING_LEGEND.len(),
        AttractScoringDemoStage::LegendApproach(index)
        | AttractScoringDemoStage::LegendLaser(index)
        | AttractScoringDemoStage::LegendTransfer(index) => index,
        _ => 0,
    }
}

fn attract_scoring_objects_for_stage(
    stage: AttractScoringDemoStage,
    local_tick: u16,
) -> Vec<AttractScoringObject> {
    let mut objects = Vec::new();
    match stage {
        AttractScoringDemoStage::RescueDescend => {
            objects.push(attract_scoring_enemy_object(
                EnemyKind::Lander,
                ATTRACT_SCORING_LANDER_X16,
                ATTRACT_SCORING_LANDER_Y16 + i32::from(local_tick) * 0x00A0,
            ));
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::Human,
                ATTRACT_SCORING_HUMAN_X16,
                ATTRACT_SCORING_HUMAN_Y16,
            ));
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::PlayerShip,
                ATTRACT_SCORING_PLAYER_X16,
                ATTRACT_SCORING_PLAYER_Y16,
            ));
        }
        AttractScoringDemoStage::RescueAscend | AttractScoringDemoStage::RescueLaser => {
            let total_rise_tick = if matches!(stage, AttractScoringDemoStage::RescueAscend) {
                local_tick
            } else {
                ATTRACT_SCORING_RESCUE_ASCENT_TICKS + local_tick
            };
            let enemy_y = ATTRACT_SCORING_LANDER_Y16
                + i32::from(ATTRACT_SCORING_RESCUE_DESCENT_TICKS) * 0x00A0
                - i32::from(total_rise_tick) * 0x00B0;
            let human_y = ATTRACT_SCORING_HUMAN_Y16 - i32::from(total_rise_tick) * 0x00B0;
            objects.push(attract_scoring_enemy_object(
                EnemyKind::Lander,
                ATTRACT_SCORING_LANDER_X16,
                enemy_y,
            ));
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::Human,
                ATTRACT_SCORING_HUMAN_X16,
                human_y,
            ));
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::PlayerShip,
                ATTRACT_SCORING_PLAYER_X16,
                ATTRACT_SCORING_PLAYER_Y16,
            ));
            if matches!(stage, AttractScoringDemoStage::RescueLaser) {
                add_attract_scoring_laser_column(
                    &mut objects,
                    ATTRACT_SCORING_PLAYER_X16,
                    ATTRACT_SCORING_PLAYER_Y16,
                    ATTRACT_SCORING_LANDER_X16,
                    enemy_y,
                );
            }
        }
        AttractScoringDemoStage::RescueFall => {
            let (ship_x, ship_y, human_y) = attract_scoring_rescue_intercept_state(local_tick);
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::PlayerShip,
                ship_x,
                ship_y,
            ));
            if local_tick < 12 {
                let explosion_y = ATTRACT_SCORING_LANDER_Y16
                    + i32::from(ATTRACT_SCORING_RESCUE_DESCENT_TICKS) * 0x00A0
                    - i32::from(
                        ATTRACT_SCORING_RESCUE_ASCENT_TICKS + ATTRACT_SCORING_RESCUE_LASER_TICKS,
                    ) * 0x00B0;
                objects.push(attract_scoring_visual_enemy_object(
                    EnemyKind::Lander,
                    ATTRACT_SCORING_LANDER_X16,
                    explosion_y,
                    AttractScoringVisual::Explosion,
                ));
            }
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::Human,
                ATTRACT_SCORING_HUMAN_X16,
                human_y,
            ));
        }
        AttractScoringDemoStage::RescueScore => {
            let (ship_x, ship_y, human_y) = attract_scoring_rescue_drop_state(local_tick);
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::PlayerShip,
                ship_x,
                ship_y,
            ));
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::Human,
                ATTRACT_SCORING_CAUGHT_HUMAN_X16,
                human_y,
            ));
        }
        AttractScoringDemoStage::RescueReturn => {
            let (ship_start_x, ship_start_y, _) =
                attract_scoring_rescue_drop_state(ATTRACT_SCORING_RESCUE_SCORE_TICKS);
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::PlayerShip,
                ship_start_x + i32::from(local_tick) * ATTRACT_SCORING_RESCUE_RETURN_XV16,
                ship_start_y + i32::from(local_tick) * ATTRACT_SCORING_RESCUE_RETURN_YV16,
            ));
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::Human,
                ATTRACT_SCORING_CAUGHT_HUMAN_X16,
                ATTRACT_SCORING_GROUNDED_HUMAN_Y16,
            ));
        }
        AttractScoringDemoStage::LegendApproach(_)
        | AttractScoringDemoStage::LegendLaser(_)
        | AttractScoringDemoStage::LegendTransfer(_)
        | AttractScoringDemoStage::LegendReveal(_)
        | AttractScoringDemoStage::LegendHold => {
            let (drop_ship_x, drop_ship_y, _) =
                attract_scoring_rescue_drop_state(ATTRACT_SCORING_RESCUE_SCORE_TICKS);
            let player_x = drop_ship_x
                + i32::from(ATTRACT_SCORING_RESCUE_RETURN_TICKS)
                    * ATTRACT_SCORING_RESCUE_RETURN_XV16;
            let player_y = drop_ship_y
                + i32::from(ATTRACT_SCORING_RESCUE_RETURN_TICKS)
                    * ATTRACT_SCORING_RESCUE_RETURN_YV16;
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::PlayerShip,
                player_x,
                player_y,
            ));
            objects.push(attract_scoring_object(
                AttractScoringObjectKind::Human,
                ATTRACT_SCORING_CAUGHT_HUMAN_X16,
                ATTRACT_SCORING_GROUNDED_HUMAN_Y16,
            ));
            append_attract_scoring_legend_objects(
                &mut objects,
                stage,
                local_tick,
                player_x,
                player_y,
            );
        }
    }
    objects
}

fn append_attract_scoring_legend_objects(
    objects: &mut Vec<AttractScoringObject>,
    stage: AttractScoringDemoStage,
    local_tick: u16,
    player_x16: i32,
    player_y16: i32,
) {
    let source_y = ATTRACT_SCORING_LEGEND_SOURCE_START_Y16
        - i32::from(ATTRACT_SCORING_LEGEND_APPROACH_TICKS) * 0x00C0;
    for entry in ATTRACT_SCORING_LEGEND
        .iter()
        .take(attract_scoring_revealed_table_entries(stage))
    {
        objects.push(attract_scoring_enemy_object(
            entry.kind,
            entry.table_x16,
            entry.table_y16,
        ));
    }

    let current_index = match stage {
        AttractScoringDemoStage::LegendApproach(index)
        | AttractScoringDemoStage::LegendLaser(index)
        | AttractScoringDemoStage::LegendTransfer(index)
        | AttractScoringDemoStage::LegendReveal(index) => Some(index),
        AttractScoringDemoStage::LegendHold => None,
        _ => return,
    };
    let Some(index) = current_index else {
        return;
    };
    let entry = ATTRACT_SCORING_LEGEND[index];
    match stage {
        AttractScoringDemoStage::LegendApproach(_) => {
            let enemy_y = ATTRACT_SCORING_LEGEND_SOURCE_START_Y16 - i32::from(local_tick) * 0x00C0;
            objects.push(attract_scoring_enemy_object(
                entry.kind,
                ATTRACT_SCORING_LEGEND_SOURCE_X16,
                enemy_y,
            ));
        }
        AttractScoringDemoStage::LegendLaser(_) => {
            objects.push(attract_scoring_enemy_object(
                entry.kind,
                ATTRACT_SCORING_LEGEND_SOURCE_X16,
                source_y,
            ));
            add_attract_scoring_laser_column(
                objects,
                player_x16,
                player_y16,
                ATTRACT_SCORING_LEGEND_SOURCE_X16,
                source_y,
            );
        }
        AttractScoringDemoStage::LegendTransfer(_) => {
            objects.push(attract_scoring_visual_enemy_object(
                entry.kind,
                ATTRACT_SCORING_LEGEND_SOURCE_X16,
                source_y,
                AttractScoringVisual::Explosion,
            ));
            objects.push(attract_scoring_visual_enemy_object(
                entry.kind,
                entry.table_x16,
                entry.table_y16,
                AttractScoringVisual::Materialize,
            ));
        }
        AttractScoringDemoStage::LegendReveal(_) => objects.push(attract_scoring_enemy_object(
            entry.kind,
            entry.table_x16,
            entry.table_y16,
        )),
        AttractScoringDemoStage::LegendHold => {}
        _ => {}
    }
}

fn attract_scoring_bonus_for_stage(
    stage: AttractScoringDemoStage,
    local_tick: u16,
) -> Option<AttractScoringBonus> {
    match stage {
        AttractScoringDemoStage::RescueScore => Some(AttractScoringBonus {
            sprite: SpriteId::SCORE_POPUP_500,
            x16: ATTRACT_SCORING_SCORE_500_X16,
            y16: ATTRACT_SCORING_SCORE_500_Y16,
        }),
        AttractScoringDemoStage::RescueReturn => Some(AttractScoringBonus {
            sprite: SpriteId::SCORE_POPUP_500,
            x16: ATTRACT_SCORING_SCORE_500_DROP_X16,
            y16: ATTRACT_SCORING_SCORE_500_DROP_Y16,
        }),
        AttractScoringDemoStage::LegendTransfer(index) if local_tick == 0 => {
            let entry = ATTRACT_SCORING_LEGEND[index];
            Some(AttractScoringBonus {
                sprite: SpriteId::SCORE_POPUP_250,
                x16: entry.table_x16,
                y16: entry.table_y16,
            })
        }
        _ => None,
    }
}

fn attract_scoring_rescue_intercept_state(fall_tick: u16) -> (i32, i32, i32) {
    let mut ship_x = ATTRACT_SCORING_PLAYER_X16;
    let mut ship_y = ATTRACT_SCORING_PLAYER_Y16;
    let mut human_y = ATTRACT_SCORING_HUMAN_Y16
        - i32::from(ATTRACT_SCORING_RESCUE_ASCENT_TICKS + ATTRACT_SCORING_RESCUE_LASER_TICKS)
            * 0x00B0;
    let mut tick_cursor = 0;
    let mut human_velocity = 0;

    for _ in 0..(ATTRACT_SCORING_RESCUE_FALL_TICKS / 2) {
        human_velocity += ATTRACT_SCORING_RESCUE_HUMAN_ACCEL16;
        for _ in 0..2 {
            if tick_cursor >= fall_tick {
                return (ship_x, ship_y, human_y);
            }
            ship_x += ATTRACT_SCORING_RESCUE_SHIP_XV16;
            ship_y += ATTRACT_SCORING_RESCUE_SHIP_YV16;
            human_y += human_velocity;
            tick_cursor += 1;
        }
    }

    (ship_x, ship_y, human_y)
}

fn attract_scoring_rescue_drop_state(score_tick: u16) -> (i32, i32, i32) {
    let (ship_x, ship_y, _) =
        attract_scoring_rescue_intercept_state(ATTRACT_SCORING_RESCUE_FALL_TICKS);
    (
        ship_x,
        ship_y + i32::from(score_tick) * ATTRACT_SCORING_RESCUE_DROP_YV16,
        ATTRACT_SCORING_CAUGHT_HUMAN_Y16 + i32::from(score_tick) * ATTRACT_SCORING_RESCUE_DROP_YV16,
    )
}

fn add_attract_scoring_laser_column(
    objects: &mut Vec<AttractScoringObject>,
    ship_x16: i32,
    ship_y16: i32,
    target_x16: i32,
    target_y16: i32,
) {
    let dx = target_x16 - ship_x16;
    let dy = target_y16 - ship_y16;
    let steps = ((dx.abs().max(dy.abs()) + 0x07FF) / 0x0800).max(1);

    for step in 1..=steps {
        objects.push(attract_scoring_object(
            AttractScoringObjectKind::PlayerShot,
            ship_x16 + (dx * step) / steps,
            ship_y16 + (dy * step) / steps,
        ));
    }
}

fn attract_scoring_enemy_object(kind: EnemyKind, x16: i32, y16: i32) -> AttractScoringObject {
    attract_scoring_object(AttractScoringObjectKind::Enemy(kind), x16, y16)
}

fn attract_scoring_visual_enemy_object(
    kind: EnemyKind,
    x16: i32,
    y16: i32,
    visual: AttractScoringVisual,
) -> AttractScoringObject {
    AttractScoringObject {
        kind: AttractScoringObjectKind::Enemy(kind),
        x16,
        y16,
        visual,
    }
}

fn attract_scoring_object(
    kind: AttractScoringObjectKind,
    x16: i32,
    y16: i32,
) -> AttractScoringObject {
    AttractScoringObject {
        kind,
        x16,
        y16,
        visual: AttractScoringVisual::Sprite,
    }
}

fn attract_scoring_object_position(x16: i32, y16: i32) -> [f32; 2] {
    [
        ((x16 + 0x10) >> 5).clamp(0, 319) as f32,
        ((y16 + 0x80) >> 8).clamp(0, 255) as f32,
    ]
}

fn attract_scoring_color_cycle_tint(scoring_tick: u16) -> Color {
    let protected_sample_index =
        usize::from(scoring_tick / ATTRACT_SCORING_REFERENCE_SAMPLE_STEP_FRAMES);
    let color_index = ATTRACT_SCORING_REFERENCE_TEXT_COLOR_INDICES
        .get(protected_sample_index)
        .map_or(
            (usize::from(scoring_tick / 2)) % SOURCE_COLTAB_ACTIVE_BYTES,
            |index| usize::from(*index),
        );

    source_pseudo_color_tint(SOURCE_COLTAB_COLOR_BYTES[color_index])
}

fn source_pseudo_color_tint(value: u8) -> Color {
    if value == 0 {
        return Color::from_rgba(0, 0, 0, 0);
    }

    Color::from_rgba(
        SOURCE_WILLIAMS_RED_GREEN_LEVELS[usize::from(value & 0x07)],
        SOURCE_WILLIAMS_RED_GREEN_LEVELS[usize::from((value >> 3) & 0x07)],
        SOURCE_WILLIAMS_BLUE_LEVELS[usize::from((value >> 6) & 0x03)],
        0xFF,
    )
}

fn push_attract_williams_logo_sprite(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if !state.attract.shows_williams_logo() {
        return;
    }
    if !SOURCE_VISUAL_STATE.attract_williams_logo_should_render() {
        return;
    }

    let tint = SOURCE_VISUAL_STATE.attract_williams_logo_tint_for_frame(state.attract.page_frame);
    let pixel_path = source_attract_williams_logo_pixel_path();
    let visible_pixel_count =
        attract_williams_logo_visible_pixel_count(&state.attract, pixel_path.len());
    if visible_pixel_count < pixel_path.len() {
        let origin = source_screen_position(SOURCE_ATTRACT_WILLIAMS_LOGO_SCREEN);
        for [pixel_x, pixel_y] in pixel_path.iter().take(visible_pixel_count) {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
                layer: RenderLayer::Overlay,
                position: [
                    origin[0] + f32::from(*pixel_x),
                    origin[1] + f32::from(*pixel_y),
                ],
                size: [1.0, 1.0],
                tint,
            });
        }
        return;
    }

    scene.push_sprite(SceneSprite {
        sprite: SpriteId::ATTRACT_WILLIAMS_LOGO,
        layer: RenderLayer::Overlay,
        position: source_screen_position(SOURCE_ATTRACT_WILLIAMS_LOGO_SCREEN),
        size: SOURCE_ATTRACT_WILLIAMS_LOGO_SIZE,
        tint,
    });
}

fn attract_williams_logo_visible_pixel_count(
    attract: &AttractPresentationSnapshot,
    total_pixels: usize,
) -> usize {
    if total_pixels == 0 {
        return 0;
    }
    if attract.page != AttractPresentationPage::WilliamsLogo
        || attract.page_frame >= ATTRACT_WILLIAMS_LOGO_REVEAL_FRAMES
    {
        return total_pixels;
    }
    if attract_title_reference_sample_index(attract.page_frame) == 0 {
        return 0;
    }

    let sleep_ticks = usize::from(ATTRACT_LOGO_SLEEP_TICKS.max(1));
    let slices_elapsed = usize::from(attract.page_frame) / sleep_ticks + 1;
    let operation_index = slices_elapsed
        .saturating_mul(SOURCE_ATTRACT_WILLIAMS_INITIAL_BYTES_PER_SLICE)
        .saturating_sub(1);
    source_attract_williams_logo_operation_pixel_counts()
        .get(operation_index)
        .copied()
        .unwrap_or(total_pixels)
        .clamp(1, total_pixels)
}

fn push_attract_defender_wordmark_sprite(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if !state.attract.shows_defender_wordmark() {
        return;
    }

    if let Some(appearance_tick) = attract_defender_wordmark_appearance_tick(&state.attract) {
        for pixel in source_attract_defender_appearance_pixels(scene.surface, appearance_tick) {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
                layer: RenderLayer::Overlay,
                position: [f32::from(pixel.position[0]), f32::from(pixel.position[1])],
                size: [1.0, 1.0],
                tint: Color { rgba: pixel.color },
            });
        }
        return;
    }

    let (descriptor_address, whole_width, whole_height, picture_data_address) =
        source_attract_defender_whole_descriptor();
    let _source_descriptor_evidence = descriptor_address ^ picture_data_address;
    scene.push_sprite(SceneSprite {
        sprite: SpriteId::HALL_OF_FAME_DEFENDER_LOGO,
        layer: RenderLayer::Overlay,
        position: source_screen_position(SOURCE_ATTRACT_DEFENDER_WORDMARK_SCREEN),
        size: [f32::from(whole_width) * 2.0, f32::from(whole_height)],
        tint: SOURCE_VISUAL_STATE.attract_defender_wordmark_tint(),
    });
}

pub(crate) fn attract_defender_wordmark_appearance_tick(
    attract: &AttractPresentationSnapshot,
) -> Option<u8> {
    if attract.page != AttractPresentationPage::DefenderWordmark {
        return None;
    }

    let elapsed = attract
        .page_frame
        .saturating_sub(ATTRACT_DEFENDER_WORDMARK_START_FRAME);
    if elapsed >= ATTRACT_DEFENDER_WORDMARK_REVEAL_FRAMES {
        return None;
    }

    let scaled_tick = elapsed.saturating_mul(SOURCE_ATTRACT_DEFENDER_APPEARANCE_TICK_NUMERATOR)
        / SOURCE_ATTRACT_DEFENDER_APPEARANCE_TICK_DENOMINATOR;
    Some(
        u8::try_from(scaled_tick)
            .expect("Defender appearance tick fits in u8")
            .min(ATTRACT_DEFENDER_APPEAR_SLEEP_TICKS),
    )
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceAttractDefenderWordmarkSlice {
    index: usize,
    object_address: u16,
    picture_descriptor_address: u16,
    picture_data_address: u16,
    x16: u16,
    y16: u16,
    appearance_slot_address: u16,
}

#[cfg(test)]
fn source_attract_defender_wordmark_slices()
-> [SourceAttractDefenderWordmarkSlice; crate::renderer::ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT] {
    std::array::from_fn(|index| {
        let index_u16 = u16::try_from(index).expect("Defender slice index fits in u16");
        SourceAttractDefenderWordmarkSlice {
            index,
            object_address: SOURCE_ATTRACT_DEFENDER_OBJECTS
                + index_u16 * SOURCE_ATTRACT_DEFENDER_OBJECT_BYTES,
            picture_descriptor_address: SOURCE_ATTRACT_DEFENDER_PICTURES
                + index_u16 * SOURCE_ATTRACT_DEFENDER_PICTURE_BYTES,
            picture_data_address: SOURCE_ATTRACT_DEFENDER_DATA
                + index_u16 * SOURCE_ATTRACT_DEFENDER_PICTURE_DATA_STEP,
            x16: SOURCE_ATTRACT_DEFENDER_INITIAL_X16 + index_u16 * SOURCE_ATTRACT_DEFENDER_X16_STEP,
            y16: SOURCE_ATTRACT_DEFENDER_Y16,
            appearance_slot_address: SOURCE_ATTRACT_DEFENDER_APPEARANCE_SLOT
                + index_u16 * SOURCE_ATTRACT_DEFENDER_APPEARANCE_SLOT_STEP,
        }
    })
}

fn source_attract_defender_whole_descriptor() -> (u16, u8, u8, u16) {
    (
        SOURCE_ATTRACT_DEFENDER_DESCRIPTOR,
        0x3C,
        0x18,
        SOURCE_ATTRACT_DEFENDER_DATA,
    )
}

fn push_attract_copyright_strip_sprite(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if !state.attract.shows_copyright() {
        return;
    }

    scene.push_sprite(SceneSprite {
        sprite: SpriteId::ATTRACT_COPYRIGHT_STRIP,
        layer: RenderLayer::Overlay,
        position: source_screen_position(SOURCE_ATTRACT_COPYRIGHT_STRIP_SCREEN),
        size: SOURCE_ATTRACT_COPYRIGHT_STRIP_SIZE,
        tint: SOURCE_VISUAL_STATE.attract_copyright_tint(),
    });
}

fn push_attract_scoring_action_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    let Some(scoring_tick) = state.attract.scoring_sequence_frame() else {
        return;
    };

    let frame = attract_scoring_frame(scoring_tick);
    push_attract_scoring_terrain_sprites(scene, frame.demo_tick);
    push_attract_scoring_scanner_sprites(scene, &frame);

    for object in &frame.objects {
        push_attract_scoring_object_sprite(scene, *object, frame.demo_tick);
    }

    if let Some(bonus) = frame.bonus {
        let size = match bonus.sprite {
            SpriteId::SCORE_POPUP_250 => [12.0, 8.0],
            SpriteId::SCORE_POPUP_500 => [12.0, 6.0],
            _ => [8.0, 8.0],
        };
        scene.push_sprite(SceneSprite {
            sprite: bonus.sprite,
            layer: RenderLayer::Objects,
            position: attract_scoring_object_position(bonus.x16, bonus.y16),
            size,
            tint: Color::WHITE,
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceTerrainFlavorRecord {
    offset: u8,
    word: u16,
}

impl SourceTerrainFlavorRecord {
    const EMPTY: Self = Self { offset: 0, word: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceTerrainDrawRecord {
    screen_address: u16,
    word: u16,
}

impl SourceTerrainDrawRecord {
    const EMPTY: Self = Self {
        screen_address: 0,
        word: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceTerrainBitState {
    data_index: usize,
    data_pointer: u16,
    data_byte: u8,
    bit_counter: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceTerrainGenerationState {
    left: SourceTerrainBitState,
    right: SourceTerrainBitState,
    left_offset: u8,
    right_offset: u8,
    background_left: u16,
    terrain_left: u16,
    flavor_0_pointer: usize,
    flavor_1_pointer: usize,
}

fn push_source_bgout_terrain_sprites(scene: &mut RenderScene) {
    for record in source_bgout_default_terrain_records() {
        scene.push_sprite(SceneSprite {
            sprite: source_terrain_word_sprite(record.word),
            layer: RenderLayer::Terrain,
            position: source_screen_position(record.screen_address),
            size: SOURCE_TERRAIN_WORD_SIZE,
            tint: Color::WHITE,
        });
    }
}

fn source_terrain_word_sprite(word: u16) -> SpriteId {
    if word == SOURCE_TERRAIN_WORD_0770 {
        SpriteId::TERRAIN_TILE_ALT
    } else {
        SpriteId::TERRAIN_TILE
    }
}

fn source_bgout_default_terrain_records()
-> &'static [SourceTerrainDrawRecord; SOURCE_TERRAIN_SCREEN_WORDS] {
    static RECORDS: OnceLock<[SourceTerrainDrawRecord; SOURCE_TERRAIN_SCREEN_WORDS]> =
        OnceLock::new();
    RECORDS.get_or_init(source_generate_bgout_default_terrain_records)
}

fn source_generate_bgout_default_terrain_records()
-> [SourceTerrainDrawRecord; SOURCE_TERRAIN_SCREEN_WORDS] {
    let data = source_tdata_bytes();
    let (flavor_0, flavor_1, state) = source_initialize_terrain_flavor_tables(data);
    let selected_flavor = if state.terrain_left.to_be_bytes()[1] & 0x20 == 0 {
        &flavor_1
    } else {
        &flavor_0
    };
    let selected_pointer = if state.terrain_left.to_be_bytes()[1] & 0x20 == 0 {
        state.flavor_1_pointer
    } else {
        state.flavor_0_pointer
    };

    let mut records = [SourceTerrainDrawRecord::EMPTY; SOURCE_TERRAIN_SCREEN_WORDS];
    for (entry_index, record) in records.iter_mut().enumerate() {
        let source_record =
            selected_flavor[(selected_pointer + entry_index) % selected_flavor.len()];
        *record = SourceTerrainDrawRecord {
            screen_address: u16::from_be_bytes([
                0x98u8.wrapping_sub(
                    u8::try_from(entry_index).expect("BGOUT terrain entry index fits in u8"),
                ),
                source_record.offset,
            ]),
            word: source_record.word,
        };
    }
    records
}

fn source_initialize_terrain_flavor_tables(
    data: &[u8; SOURCE_TERRAIN_TDATA_BYTES],
) -> (
    [SourceTerrainFlavorRecord; SOURCE_TERRAIN_FLAVOR_RECORDS],
    [SourceTerrainFlavorRecord; SOURCE_TERRAIN_FLAVOR_RECORDS],
    SourceTerrainGenerationState,
) {
    let (right, right_offset) = source_alinit_final_terrain_state(data);
    let terrain_left = 0u16;
    let mut generation_left = terrain_left.wrapping_add(0x2610);
    let mut left = SourceTerrainBitState {
        data_index: data.len() - 1,
        data_pointer: SOURCE_TERRAIN_TDATA_ADDRESS.wrapping_sub(1),
        data_byte: 0,
        bit_counter: 0,
    };
    let mut left_offset = 0xE0;
    source_advance_terrain_right_state(&mut left, data);

    let mut scan_x = 0x0010u16;
    while scan_x != generation_left {
        left_offset = source_terrain_altitude_step(left_offset, left.data_byte);
        source_advance_terrain_right_state(&mut left, data);
        scan_x = scan_x.wrapping_add(0x20);
    }

    let saved_right = left;
    let saved_right_offset = left_offset;
    let mut flavor_0 = [SourceTerrainFlavorRecord::EMPTY; SOURCE_TERRAIN_FLAVOR_RECORDS];
    let mut flavor_1 = [SourceTerrainFlavorRecord::EMPTY; SOURCE_TERRAIN_FLAVOR_RECORDS];
    let mut state = SourceTerrainGenerationState {
        left,
        right,
        left_offset,
        right_offset,
        background_left: generation_left,
        terrain_left,
        flavor_0_pointer: 0,
        flavor_1_pointer: 0,
    };

    loop {
        generation_left = generation_left.wrapping_sub(0x20);
        state.background_left = generation_left;
        if generation_left.wrapping_sub(state.terrain_left) & 0x8000 != 0 {
            break;
        }
        source_add_left_terrain_pixel(&mut state, data, &mut flavor_0, &mut flavor_1);
    }

    state.right = saved_right;
    state.right_offset = saved_right_offset;
    (flavor_0, flavor_1, state)
}

fn source_add_left_terrain_pixel(
    state: &mut SourceTerrainGenerationState,
    data: &[u8; SOURCE_TERRAIN_TDATA_BYTES],
    flavor_0: &mut [SourceTerrainFlavorRecord; SOURCE_TERRAIN_FLAVOR_RECORDS],
    flavor_1: &mut [SourceTerrainFlavorRecord; SOURCE_TERRAIN_FLAVOR_RECORDS],
) {
    source_advance_terrain_left_state(&mut state.right, data);
    state.right_offset = if state.right.data_byte & 0x80 == 0 {
        state.right_offset.wrapping_sub(1)
    } else {
        state.right_offset.wrapping_add(1)
    };

    let flavor_0_selected = state.background_left.to_be_bytes()[1] & 0x20 != 0;
    let record_index = if flavor_0_selected {
        state.flavor_0_pointer
    } else {
        state.flavor_1_pointer
    };

    source_advance_terrain_left_state(&mut state.left, data);
    let (offset, word) = if state.left.data_byte & 0x80 == 0 {
        state.left_offset = state.left_offset.wrapping_sub(1);
        (state.left_offset, SOURCE_TERRAIN_WORD_7007)
    } else {
        let offset = state.left_offset;
        state.left_offset = state.left_offset.wrapping_add(1);
        (offset, SOURCE_TERRAIN_WORD_0770)
    };

    let record = SourceTerrainFlavorRecord { offset, word };
    if flavor_0_selected {
        flavor_0[record_index] = record;
        state.flavor_0_pointer = (record_index + 1) % SOURCE_TERRAIN_FLAVOR_RECORDS;
    } else {
        flavor_1[record_index] = record;
        state.flavor_1_pointer = (record_index + 1) % SOURCE_TERRAIN_FLAVOR_RECORDS;
    }
}

fn source_alinit_final_terrain_state(
    data: &[u8; SOURCE_TERRAIN_TDATA_BYTES],
) -> (SourceTerrainBitState, u8) {
    let mut state = SourceTerrainBitState {
        data_index: 0,
        data_pointer: SOURCE_TERRAIN_TDATA_ADDRESS,
        data_byte: data[0],
        bit_counter: 7,
    };
    let mut offset = 0xE0;
    for _ in 0..0x0400 {
        offset = source_terrain_altitude_step(offset, state.data_byte);
        source_advance_terrain_right_state(&mut state, data);
        offset = source_terrain_altitude_step(offset, state.data_byte);
        source_advance_terrain_right_state(&mut state, data);
    }
    (state, offset)
}

fn source_terrain_altitude_step(offset: u8, data_byte: u8) -> u8 {
    if data_byte & 0x80 != 0 {
        offset.wrapping_sub(1)
    } else {
        offset.wrapping_add(1)
    }
}

fn source_advance_terrain_right_state(
    state: &mut SourceTerrainBitState,
    data: &[u8; SOURCE_TERRAIN_TDATA_BYTES],
) {
    if state.bit_counter == 0 {
        state.data_index = (state.data_index + 1) % data.len();
        state.data_pointer = SOURCE_TERRAIN_TDATA_ADDRESS
            .wrapping_add(u16::try_from(state.data_index).expect("TDATA index fits in u16"));
        state.bit_counter = 7;
        state.data_byte = data[state.data_index];
    } else {
        state.bit_counter -= 1;
        let carry = u8::from(state.data_byte & 0x80 != 0);
        state.data_byte = state.data_byte.wrapping_shl(1).wrapping_add(carry);
    }
}

fn source_advance_terrain_left_state(
    state: &mut SourceTerrainBitState,
    data: &[u8; SOURCE_TERRAIN_TDATA_BYTES],
) {
    if state.bit_counter == 7 {
        state.data_index = if state.data_index == 0 {
            data.len() - 1
        } else {
            state.data_index - 1
        };
        state.data_pointer = SOURCE_TERRAIN_TDATA_ADDRESS
            .wrapping_add(u16::try_from(state.data_index).expect("TDATA index fits in u16"));
        state.bit_counter = 0;
        state.data_byte = source_rotate_terrain_right_byte(data[state.data_index]);
    } else {
        state.bit_counter += 1;
        state.data_byte = source_rotate_terrain_right_byte(state.data_byte);
    }
}

fn source_rotate_terrain_right_byte(data_byte: u8) -> u8 {
    (data_byte >> 1).wrapping_add(if data_byte & 1 == 0 { 0 } else { 0x80 })
}

fn source_tdata_bytes() -> &'static [u8; SOURCE_TERRAIN_TDATA_BYTES] {
    static TDATA: OnceLock<[u8; SOURCE_TERRAIN_TDATA_BYTES]> = OnceLock::new();
    TDATA.get_or_init(parse_source_tdata_bytes)
}

fn parse_source_tdata_bytes() -> [u8; SOURCE_TERRAIN_TDATA_BYTES] {
    let mut output = [0; SOURCE_TERRAIN_TDATA_BYTES];
    for (line_index, line) in SOURCE_TERRAIN_DATA_TSV.lines().enumerate().skip(1) {
        let mut fields = line.split('\t');
        let label = fields.next().unwrap_or_default();
        let address = fields.next().unwrap_or_default();
        let bytes = fields.next().unwrap_or_default();
        if label != SOURCE_TERRAIN_TDATA_LABEL {
            continue;
        }
        assert_eq!(
            address,
            "0xC350",
            "terrain-data line {} must preserve TDATA source address",
            line_index + 1
        );
        assert_eq!(
            bytes.len(),
            SOURCE_TERRAIN_TDATA_BYTES * 2,
            "TDATA hex payload must contain exactly 0x100 bytes"
        );
        for index in 0..SOURCE_TERRAIN_TDATA_BYTES {
            output[index] = parse_source_hex_byte(&bytes[index * 2..index * 2 + 2]);
        }
        return output;
    }

    panic!("terrain-data.tsv must contain the TDATA record")
}

fn parse_source_hex_byte(value: &str) -> u8 {
    u8::from_str_radix(value, 16).expect("source terrain byte must be hexadecimal")
}

fn push_attract_scoring_terrain_sprites(scene: &mut RenderScene, demo_tick: u16) {
    let _source_scproc_timing_evidence = demo_tick;
    for record in source_bgout_default_terrain_records() {
        scene.push_sprite(SceneSprite {
            sprite: source_terrain_word_sprite(record.word),
            layer: RenderLayer::Terrain,
            position: offset_position(
                source_screen_position(record.screen_address),
                ATTRACT_REFERENCE_SCENE_OFFSET,
            ),
            size: SOURCE_TERRAIN_WORD_SIZE,
            tint: Color::WHITE,
        });
    }
}

fn push_attract_scoring_scanner_sprites(scene: &mut RenderScene, frame: &AttractScoringFrame) {
    push_attract_scoring_scanner_terrain_sprites(scene);

    for object in &frame.scanner_objects {
        let Some((sprite, size, color_word)) = attract_scoring_scanner_sprite(*object) else {
            continue;
        };
        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Hud,
            position: attract_scoring_scanner_position(*object),
            size,
            tint: source_pseudo_color_tint((color_word & 0x00FF) as u8),
        });
    }
}

fn push_attract_scoring_scanner_terrain_sprites(scene: &mut RenderScene) {
    for record in source_bgout_default_terrain_records() {
        let position = attract_scoring_scanner_terrain_position(record.screen_address);
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL,
            layer: RenderLayer::Hud,
            position,
            size: ATTRACT_SCORING_SCANNER_TERRAIN_PIXEL_SIZE,
            tint: ATTRACT_SCORING_SCANNER_TERRAIN_TINT,
        });
    }
}

fn attract_scoring_scanner_terrain_position(screen_address: u16) -> [f32; 2] {
    let source_position = source_screen_position(screen_address);
    [
        ATTRACT_SCORING_SCANNER_ORIGIN[0]
            + source_position[0].rem_euclid(ATTRACT_SCORING_PLAYFIELD_WIDTH)
                * ATTRACT_SCORING_SCANNER_SIZE[0]
                / ATTRACT_SCORING_PLAYFIELD_WIDTH,
        ATTRACT_SCORING_SCANNER_ORIGIN[1]
            + source_position[1] * ATTRACT_SCORING_SCANNER_SIZE[1]
                / ATTRACT_SCORING_PLAYFIELD_HEIGHT,
    ]
}

fn attract_scoring_scanner_sprite(
    object: AttractScoringObject,
) -> Option<(SpriteId, [f32; 2], u16)> {
    match object.kind {
        AttractScoringObjectKind::PlayerShip => {
            Some((SpriteId::SCANNER_PLAYER_BLIP, [3.0, 2.0], 0x9099))
        }
        AttractScoringObjectKind::Human => {
            Some((SpriteId::SCANNER_OBJECT_BLIP, [2.0, 2.0], 0x6666))
        }
        AttractScoringObjectKind::Enemy(kind) => ATTRACT_SCORING_LEGEND
            .iter()
            .find(|entry| entry.kind == kind)
            .map(|entry| {
                (
                    SpriteId::SCANNER_OBJECT_BLIP,
                    [2.0, 2.0],
                    entry.scanner_color_word,
                )
            }),
        AttractScoringObjectKind::PlayerShot => None,
    }
}

fn attract_scoring_scanner_position(object: AttractScoringObject) -> [f32; 2] {
    let [native_x, native_y] = attract_scoring_object_position(object.x16, object.y16);
    [
        ATTRACT_SCORING_SCANNER_ORIGIN[0]
            + native_x * ATTRACT_SCORING_SCANNER_SIZE[0] / ATTRACT_SCORING_PLAYFIELD_WIDTH,
        ATTRACT_SCORING_SCANNER_ORIGIN[1]
            + native_y * ATTRACT_SCORING_SCANNER_SIZE[1] / ATTRACT_SCORING_PLAYFIELD_HEIGHT,
    ]
}

fn push_attract_scoring_object_sprite(
    scene: &mut RenderScene,
    object: AttractScoringObject,
    demo_tick: u16,
) {
    let (sprite, layer, size, tint) = attract_scoring_object_sprite(object, demo_tick);
    scene.push_sprite(SceneSprite {
        sprite,
        layer,
        position: attract_scoring_object_position(object.x16, object.y16),
        size,
        tint,
    });
}

fn attract_scoring_object_sprite(
    object: AttractScoringObject,
    demo_tick: u16,
) -> (SpriteId, RenderLayer, [f32; 2], Color) {
    if matches!(object.visual, AttractScoringVisual::Explosion) {
        return (
            SpriteId::BOMB_EXPLOSION,
            RenderLayer::Objects,
            [10.0, 10.0],
            attract_scoring_color_cycle_tint(demo_tick),
        );
    }

    let tint = if matches!(object.visual, AttractScoringVisual::Materialize) {
        attract_scoring_color_cycle_tint(demo_tick)
    } else {
        Color::WHITE
    };

    match object.kind {
        AttractScoringObjectKind::PlayerShip => (
            SpriteId::PLAYER_SHIP,
            RenderLayer::Objects,
            [
                f32::from(PLAYER_SPRITE_SIZE.0),
                f32::from(PLAYER_SPRITE_SIZE.1),
            ],
            tint,
        ),
        AttractScoringObjectKind::Human => (
            SpriteId::HUMAN,
            RenderLayer::Objects,
            [
                f32::from(HUMAN_SPRITE_SIZE.0),
                f32::from(HUMAN_SPRITE_SIZE.1),
            ],
            tint,
        ),
        AttractScoringObjectKind::PlayerShot => (
            SpriteId::PLAYER_PROJECTILE,
            RenderLayer::Projectiles,
            [
                f32::from(PROJECTILE_SPRITE_SIZE.0),
                f32::from(PROJECTILE_SPRITE_SIZE.1),
            ],
            tint,
        ),
        AttractScoringObjectKind::Enemy(kind) => {
            let size = enemy_sprite_size(kind);
            (
                enemy_sprite(kind),
                RenderLayer::Objects,
                [f32::from(size.0), f32::from(size.1)],
                tint,
            )
        }
    }
}

fn requested_start_player_count(input: GameInput) -> Option<u8> {
    if input.start_two {
        Some(2)
    } else if input.start_one {
        Some(1)
    } else {
        None
    }
}

fn player_stock_index(player: u8) -> usize {
    usize::from(player.saturating_sub(1).min(1))
}

const fn other_player_number(player: u8) -> u8 {
    if player == 2 { 1 } else { 2 }
}

fn push_final_game_over_prompt_sprites(scene: &mut RenderScene, game_over: GameOverSnapshot) {
    if game_over.player_death_sleep_remaining.is_none() {
        return;
    }

    if let Some(text) = source_message_text("GO") {
        push_source_message_sprites(
            scene,
            text,
            source_screen_position(SOURCE_FINAL_GAME_OVER_SCREEN),
            RenderLayer::Overlay,
        );
    }
}

fn push_player_switch_prompt_sprites(scene: &mut RenderScene, game_over: GameOverSnapshot) {
    if game_over.player_switch_sleep_remaining.is_none() {
        return;
    }

    let player_label = if game_over.player_switch_from == Some(2) {
        "PLYR2"
    } else {
        "PLYR1"
    };
    if let Some(text) = source_message_text(player_label) {
        push_source_message_sprites(
            scene,
            text,
            source_screen_position(SOURCE_PLAYER_SWITCH_LABEL_SCREEN),
            RenderLayer::Overlay,
        );
    }
    if let Some(text) = source_message_text("GO") {
        push_source_message_sprites(
            scene,
            text,
            source_screen_position(SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN),
            RenderLayer::Overlay,
        );
    }
}

fn push_player_start_prompt_sprites(scene: &mut RenderScene, state: &GameState) {
    if !should_show_player_start_prompt(state) {
        return;
    }

    let player_label = if state.current_player == 2 {
        "PLYR2"
    } else {
        "PLYR1"
    };
    if let Some(text) = source_message_text(player_label) {
        push_source_message_sprites(
            scene,
            text,
            source_screen_position(SOURCE_PLAYER_START_PROMPT_SCREEN),
            RenderLayer::Overlay,
        );
    }
}

fn should_show_player_start_prompt(state: &GameState) -> bool {
    state.phase == GamePhase::Playing
        && state.player_count > 1
        && state.player.position == (WorldVector::default(), WorldVector::default())
        && state.world == WorldSnapshot::default()
}

fn push_wave_completion_status_sprites(scene: &mut RenderScene, state: &GameState) {
    if !should_show_wave_completion_status(state) {
        return;
    }

    for (label, screen_address) in SOURCE_WAVE_COMPLETION_STATUS_LINES {
        if let Some(text) = source_message_text(label) {
            push_source_message_sprites(
                scene,
                text,
                source_screen_position(*screen_address),
                RenderLayer::Overlay,
            );
        }
    }

    let (wave_digits, wave_digit_count) = source_visible_decimal_digits(state.wave);
    push_source_text_bytes_sprites(
        scene,
        &wave_digits[..wave_digit_count],
        source_screen_position(SOURCE_WAVE_COMPLETION_WAVE_NUMBER_SCREEN),
        RenderLayer::Overlay,
    );

    let (multiplier_digits, multiplier_digit_count) =
        source_visible_decimal_digits(state.wave.min(5));
    push_source_text_bytes_sprites(
        scene,
        &multiplier_digits[..multiplier_digit_count],
        source_screen_position(SOURCE_WAVE_COMPLETION_MULTIPLIER_NUMBER_SCREEN),
        RenderLayer::Overlay,
    );
}

fn push_survivor_bonus_icon_sprites(scene: &mut RenderScene, state: &GameState) {
    if !should_show_wave_completion_status(state) {
        return;
    }

    for index in 0..state
        .world
        .humans
        .len()
        .min(SOURCE_SURVIVOR_BONUS_HUMAN_LIMIT)
    {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::HUMAN,
            layer: RenderLayer::Overlay,
            position: source_screen_position_with_offset(
                SOURCE_SURVIVOR_BONUS_FIRST_HUMAN_SCREEN,
                (index as u8) * SOURCE_SURVIVOR_BONUS_HUMAN_STEP,
                0,
            ),
            size: SOURCE_SURVIVOR_BONUS_HUMAN_SIZE,
            tint: Color::WHITE,
        });
    }
}

fn should_show_wave_completion_status(state: &GameState) -> bool {
    state.phase == GamePhase::Playing
        && state.player.position != (WorldVector::default(), WorldVector::default())
        && state.world.enemies.is_empty()
}

fn source_visible_decimal_digits(value: u8) -> ([u8; 2], usize) {
    let value = value.min(99);
    if value < 10 {
        ([b'0' + value, b' '], 1)
    } else {
        ([b'0' + value / 10, b'0' + value % 10], 2)
    }
}

fn push_high_score_entry_prompt_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::HighScoreEntry {
        return;
    }

    let player_label = if state.current_player == 2 {
        "PLYR2"
    } else {
        "PLYR1"
    };
    if let Some(text) = source_message_text(player_label) {
        push_source_message_sprites_with_tint(
            scene,
            text,
            source_screen_position(SOURCE_HALL_OF_FAME_PLAYER_LABEL_SCREEN),
            RenderLayer::Overlay,
            SOURCE_VISUAL_STATE.hall_of_fame_entry_text_tint(),
        );
    }

    for (index, message_label) in SOURCE_HALL_OF_FAME_INSTRUCTION_MESSAGES.iter().enumerate() {
        if let Some(text) = source_message_text(message_label) {
            push_source_message_sprites_with_tint(
                scene,
                text,
                source_screen_position_with_offset(
                    SOURCE_HALL_OF_FAME_INSTRUCTIONS_TOP_LEFT,
                    0,
                    SOURCE_HALL_OF_FAME_LINE_VERTICAL_OFFSETS[index],
                ),
                RenderLayer::Overlay,
                SOURCE_VISUAL_STATE.hall_of_fame_entry_text_tint(),
            );
        }
    }

    for (index, initial) in state.high_score_initials.initials.iter().enumerate() {
        let Some(initial) = initial else {
            continue;
        };
        let Some(sprite) = SpriteId::message_glyph(*initial) else {
            continue;
        };
        let Some(size) = SpriteId::message_glyph_size(*initial) else {
            continue;
        };
        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Overlay,
            position: source_screen_position_with_offset(
                SOURCE_HALL_OF_FAME_INITIALS_SCREEN,
                SOURCE_HALL_OF_FAME_INITIAL_HORIZONTAL_OFFSETS[index],
                0,
            ),
            size: [size[0] as f32, size[1] as f32],
            tint: SOURCE_VISUAL_STATE.hall_of_fame_blink_text_tint(),
        });
    }

    push_high_score_entry_underline_sprites(
        scene,
        usize::from(state.high_score_initials.cursor).min(
            SOURCE_HALL_OF_FAME_INITIAL_HORIZONTAL_OFFSETS
                .len()
                .saturating_sub(1),
        ),
    );
}

fn push_high_score_entry_underline_sprites(scene: &mut RenderScene, active_initial: usize) {
    for initial_index in 0..SOURCE_HALL_OF_FAME_INITIAL_HORIZONTAL_OFFSETS.len() {
        let initial_offset = u8::try_from(initial_index).expect("high-score initial index fits")
            * SOURCE_HALL_OF_FAME_UNDERLINE_INITIAL_STEP;
        let tint = if initial_index == active_initial {
            SOURCE_VISUAL_STATE.hall_of_fame_active_underline_tint()
        } else {
            SOURCE_VISUAL_STATE.hall_of_fame_normal_underline_tint()
        };
        for word_offset in SOURCE_HALL_OF_FAME_UNDERLINE_WORD_HORIZONTAL_OFFSETS {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::HALL_OF_FAME_UNDERLINE_WORD,
                layer: RenderLayer::Overlay,
                position: source_screen_position_with_offset(
                    SOURCE_HALL_OF_FAME_UNDERLINE_SCREEN,
                    initial_offset.wrapping_add(word_offset),
                    0,
                ),
                size: SOURCE_HALL_OF_FAME_UNDERLINE_WORD_SIZE,
                tint,
            });
        }
    }
}

fn push_hall_of_fame_display_sprites(scene: &mut RenderScene, state: &GameState) {
    let shows_attract_hall_of_fame =
        state.phase == GamePhase::Attract && state.attract.shows_hall_of_fame();
    if state.game_over.hall_of_fame_stall_remaining.is_none() && !shows_attract_hall_of_fame {
        return;
    }

    let visual_offset = if shows_attract_hall_of_fame {
        ATTRACT_HALL_OF_FAME_REFERENCE_OFFSET
    } else {
        [0.0, 0.0]
    };
    for (message_label, screen_address) in SOURCE_HALL_OF_FAME_DISPLAY_HEADINGS {
        if let Some(text) = source_message_text(message_label) {
            push_source_message_sprites_with_tint(
                scene,
                text,
                offset_position(source_screen_position(screen_address), visual_offset),
                RenderLayer::Overlay,
                SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint(),
            );
        }
    }

    push_hall_of_fame_display_underline_sprites(scene, visual_offset);
    push_hall_of_fame_defender_logo_sprite(scene, visual_offset);
    push_hall_of_fame_table_sprites(
        scene,
        state.high_score_tables.todays_greatest,
        SOURCE_HALL_OF_FAME_TODAYS_TABLE_SCREEN,
        visual_offset,
    );
    push_hall_of_fame_table_sprites(
        scene,
        state.high_score_tables.all_time,
        SOURCE_HALL_OF_FAME_ALL_TIME_TABLE_SCREEN,
        visual_offset,
    );
}

fn push_hall_of_fame_display_underline_sprites(scene: &mut RenderScene, visual_offset: [f32; 2]) {
    for word_offset in (SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_RIGHT_END
        ..=SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_RIGHT_START)
        .rev()
        .chain((0..=SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_LEFT_START).rev())
    {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::HALL_OF_FAME_UNDERLINE_WORD,
            layer: RenderLayer::Overlay,
            position: offset_position(
                source_screen_position_with_offset(
                    SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_SCREEN,
                    word_offset,
                    0,
                ),
                visual_offset,
            ),
            size: SOURCE_HALL_OF_FAME_UNDERLINE_WORD_SIZE,
            tint: SOURCE_VISUAL_STATE.hall_of_fame_normal_underline_tint(),
        });
    }
}

fn push_hall_of_fame_defender_logo_sprite(scene: &mut RenderScene, visual_offset: [f32; 2]) {
    scene.push_sprite(SceneSprite {
        sprite: SpriteId::HALL_OF_FAME_DEFENDER_LOGO,
        layer: RenderLayer::Overlay,
        position: offset_position(
            source_screen_position(SOURCE_HALL_OF_FAME_LOGO_SCREEN),
            visual_offset,
        ),
        size: SOURCE_HALL_OF_FAME_LOGO_SIZE,
        tint: SOURCE_VISUAL_STATE.hall_of_fame_logo_tint(),
    });
}

fn push_hall_of_fame_table_sprites(
    scene: &mut RenderScene,
    entries: [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES],
    top_left_screen_address: u16,
    visual_offset: [f32; 2],
) {
    for (index, entry) in entries.iter().enumerate() {
        let vertical_offset = u8::try_from(index).expect("high-score table index fits in u8")
            * SOURCE_HALL_OF_FAME_TABLE_ROW_STEP;
        let row_rank = b'1' + u8::try_from(index).expect("high-score table index fits in u8");
        push_source_text_bytes_sprites_with_tint(
            scene,
            &[row_rank],
            offset_position(
                source_screen_position_with_offset(top_left_screen_address, 0, vertical_offset),
                visual_offset,
            ),
            RenderLayer::Overlay,
            SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint(),
        );
        push_source_text_bytes_sprites_with_tint(
            scene,
            &high_score_initials_text(entry.initials),
            offset_position(
                source_screen_position_with_offset(
                    top_left_screen_address,
                    SOURCE_HALL_OF_FAME_TABLE_INITIALS_OFFSET,
                    vertical_offset,
                ),
                visual_offset,
            ),
            RenderLayer::Overlay,
            SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint(),
        );
        push_source_text_bytes_sprites_with_tint(
            scene,
            &hall_of_fame_score_text(entry.score),
            offset_position(
                source_screen_position_with_offset(
                    top_left_screen_address,
                    SOURCE_HALL_OF_FAME_TABLE_SCORE_OFFSET,
                    vertical_offset,
                ),
                visual_offset,
            ),
            RenderLayer::Overlay,
            SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint(),
        );
    }
}

fn high_score_initials_text(initials: [Option<char>; 3]) -> [u8; 3] {
    initials.map(|initial| {
        initial
            .filter(|character| character.is_ascii_alphabetic())
            .map(|character| character.to_ascii_uppercase() as u8)
            .unwrap_or(b' ')
    })
}

fn hall_of_fame_score_text(score: u32) -> [u8; SOURCE_HALL_OF_FAME_SCORE_TEXT_LEN] {
    let mut text = [b' '; SOURCE_HALL_OF_FAME_SCORE_TEXT_LEN];
    let mut place = 100_000;
    let mut seen_non_zero = false;
    for byte in &mut text {
        let digit = ((score.min(SCORE_DISPLAY_MAX) / place) % 10) as u8;
        if digit != 0 || seen_non_zero {
            seen_non_zero = true;
            *byte = b'0' + digit;
        }
        place /= 10;
    }
    text
}

fn push_score_sprites(scene: &mut RenderScene, scores: ScoreSnapshot, player_count: u8) {
    push_player_score_sprites(scene, scores.player_one, PLAYER_ONE_SCORE_ORIGIN);
    if player_count > 1 {
        push_player_score_sprites(scene, scores.player_two, PLAYER_TWO_SCORE_ORIGIN);
    }
}

fn push_player_score_sprites(scene: &mut RenderScene, score: u32, origin: [f32; 2]) {
    for (index, digit) in visible_score_digits(score).iter().enumerate() {
        let Some(digit) = digit else {
            continue;
        };
        let Some(sprite) = SpriteId::score_digit(*digit) else {
            continue;
        };
        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Hud,
            position: [
                origin[0] + SCORE_DIGIT_STEP[0] * index as f32,
                origin[1] + SCORE_DIGIT_STEP[1] * index as f32,
            ],
            size: SCORE_DIGIT_SIZE,
            tint: SOURCE_VISUAL_STATE.hud_tint(),
        });
    }
}

fn visible_score_digits(score: u32) -> [Option<u8>; SCORE_DIGIT_DISPLAY_COUNT] {
    let score = score.min(SCORE_DISPLAY_MAX);
    let mut place = 100_000;
    let mut digits = [None; SCORE_DIGIT_DISPLAY_COUNT];
    let mut non_zero_seen = false;

    for (index, digit) in digits.iter_mut().enumerate() {
        let value = ((score / place) % 10) as u8;
        let counter = SCORE_DIGIT_DISPLAY_COUNT - index;
        if value == 0 && counter > 2 && !non_zero_seen {
            *digit = None;
        } else {
            non_zero_seen = true;
            *digit = Some(value);
        }
        place /= 10;
    }

    digits
}

fn push_stock_sprites(
    scene: &mut RenderScene,
    player_count: u8,
    player_stocks: [PlayerStockSnapshot; 2],
) {
    push_player_stock_sprites(
        scene,
        player_stocks[0],
        PLAYER_ONE_LIFE_STOCK_ORIGIN,
        PLAYER_ONE_SMART_BOMB_STOCK_ORIGIN,
    );
    if player_count > 1 {
        push_player_stock_sprites(
            scene,
            player_stocks[1],
            PLAYER_TWO_LIFE_STOCK_ORIGIN,
            PLAYER_TWO_SMART_BOMB_STOCK_ORIGIN,
        );
    }
}

fn push_player_stock_sprites(
    scene: &mut RenderScene,
    stock: PlayerStockSnapshot,
    life_origin: [f32; 2],
    smart_bomb_origin: [f32; 2],
) {
    push_stock_sprite_series(
        scene,
        SpriteId::PLAYER_LIFE_STOCK,
        stock.lives.min(PLAYER_LIFE_STOCK_DISPLAY_LIMIT),
        life_origin,
        PLAYER_LIFE_STOCK_STEP,
        PLAYER_LIFE_STOCK_SIZE,
    );
    push_stock_sprite_series(
        scene,
        SpriteId::SMART_BOMB_STOCK,
        stock.smart_bombs.min(SMART_BOMB_STOCK_DISPLAY_LIMIT),
        smart_bomb_origin,
        SMART_BOMB_STOCK_STEP,
        SMART_BOMB_STOCK_SIZE,
    );
}

fn push_stock_sprite_series(
    scene: &mut RenderScene,
    sprite: SpriteId,
    count: u8,
    origin: [f32; 2],
    step: [f32; 2],
    size: [f32; 2],
) {
    for index in 0..count {
        let index = f32::from(index);
        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Hud,
            position: [origin[0] + step[0] * index, origin[1] + step[1] * index],
            size,
            tint: SOURCE_VISUAL_STATE.hud_tint(),
        });
    }
}

fn push_top_display_border_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Playing
        && !(state.phase == GamePhase::Attract && state.attract.scoring_sequence_frame().is_some())
    {
        return;
    }

    let visual_offset = attract_page_visual_offset(state);
    for (screen_address, size) in SOURCE_TOP_DISPLAY_BORDER_SEGMENTS {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::TOP_DISPLAY_BORDER_WORD,
            layer: RenderLayer::Hud,
            position: offset_position(source_screen_position(*screen_address), visual_offset),
            size: *size,
            tint: top_display_border_segment_tint(*screen_address, state),
        });
    }
}

fn top_display_border_segment_tint(screen_address: u16, state: &GameState) -> Color {
    if state.phase == GamePhase::Attract && state.attract.scoring_sequence_frame().is_some() {
        return ATTRACT_SCORING_SCANNER_BORDER_TINT;
    }

    if matches!(screen_address, 0x4C07 | 0x4C28) {
        SOURCE_VISUAL_STATE.top_display_scanner_marker_tint()
    } else {
        SOURCE_VISUAL_STATE.top_display_border_tint()
    }
}

pub(crate) fn push_scanner_radar_sprites(scene: &mut RenderScene, scanner: &ScannerRadarSnapshot) {
    if !scanner.enabled {
        return;
    }

    let blip_count = usize::from(scanner.blip_count).min(SCANNER_RADAR_BLIP_LIMIT);
    for blip in &scanner.blips[..blip_count] {
        if blip.color_word == 0 {
            continue;
        }
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::SCANNER_OBJECT_BLIP,
            layer: RenderLayer::Hud,
            position: source_screen_position(blip.screen_address),
            size: [2.0, 2.0],
            tint: SOURCE_VISUAL_STATE.scanner_object_blip_tint(blip.color_word),
        });
    }

    let Some(player_blip) = scanner.player_blip else {
        return;
    };
    scene.push_sprite(SceneSprite {
        sprite: SpriteId::SCANNER_PLAYER_BLIP,
        layer: RenderLayer::Hud,
        position: source_screen_position(player_blip.screen_address),
        size: [3.0, 2.0],
        tint: SOURCE_VISUAL_STATE.scanner_player_blip_tint(player_blip.body_word),
    });
    scene.push_sprite(SceneSprite {
        sprite: SpriteId::SCANNER_PLAYER_BLIP,
        layer: RenderLayer::Hud,
        position: source_screen_position(player_blip.screen_address.wrapping_sub(0x00FF)),
        size: [1.0, 1.0],
        tint: SOURCE_VISUAL_STATE.scanner_player_blip_tint(u16::from(player_blip.upper_byte)),
    });
}

fn push_source_object_detail_sprites(scene: &mut RenderScene, evidence: &ObjectEvidenceSnapshot) {
    let detail_count = usize::from(evidence.detail_count).min(OBJECT_EVIDENCE_DETAIL_LIMIT);
    for detail in &evidence.details[..detail_count] {
        if detail.object_category.is_some() {
            continue;
        }
        let Some(layer) = source_object_detail_render_layer(detail.list) else {
            continue;
        };
        let Some(sprite) = detail.mapped_sprite else {
            continue;
        };
        if sprite == SpriteId::NULL_OBJECT {
            continue;
        }
        let Some(position) = detail.screen_position else {
            continue;
        };
        let Some((width, height)) = detail.picture_size else {
            continue;
        };
        if width == 0 || height == 0 {
            continue;
        }

        scene.push_sprite(SceneSprite {
            sprite,
            layer,
            position: [f32::from(position.x), f32::from(position.y)],
            size: [f32::from(width), f32::from(height)],
            tint: Color::WHITE,
        });
    }
}

fn push_expanded_object_detail_sprites(
    scene: &mut RenderScene,
    evidence: &ExpandedObjectEvidenceSnapshot,
) {
    let detail_count = usize::from(evidence.detail_count).min(EXPANDED_OBJECT_DETAIL_LIMIT);
    for detail in &evidence.details[..detail_count] {
        let Some(sprite) = detail.mapped_sprite else {
            continue;
        };
        if sprite == SpriteId::NULL_OBJECT {
            continue;
        }
        let Some(position) = detail.top_left else {
            continue;
        };
        let Some((width, height)) = expanded_object_sprite_size(detail) else {
            continue;
        };
        if width == 0 || height == 0 {
            continue;
        }

        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Objects,
            position: [f32::from(position.x), f32::from(position.y)],
            size: [f32::from(width), f32::from(height)],
            tint: Color::WHITE,
        });
    }
}

pub(crate) fn push_player_explosion_cloud_sprites(
    scene: &mut RenderScene,
    cloud: Option<&PlayerExplosionCloudSnapshot>,
) {
    let Some(cloud) = cloud else {
        return;
    };
    let tint = player_explosion_tint(cloud.source_color);
    let piece_count = usize::from(cloud.piece_count).min(PLAYER_EXPLOSION_PIECE_LIMIT);
    for piece in &cloud.pieces[..piece_count] {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_EXPLOSION_PIXEL,
            layer: RenderLayer::Objects,
            position: [f32::from(piece.position.x), f32::from(piece.position.y)],
            size: [4.0, if piece.split { 2.0 } else { 1.0 }],
            tint,
        });
    }
}

const fn player_explosion_tint(source_color: u8) -> Color {
    let _source_color = source_color;
    Color::WHITE
}

pub(crate) fn expanded_object_sprite_size(
    detail: &ExpandedObjectDetailSnapshot,
) -> Option<(u16, u16)> {
    let (width, height) = detail.picture_size?;
    if detail.kind != ExpandedObjectKind::Explosion {
        return Some((u16::from(width), u16::from(height)));
    }

    let scale = u16::from(source_explosion_size_scale(detail.size)?);
    Some((
        u16::from(width).saturating_mul(scale),
        u16::from(height).saturating_mul(scale),
    ))
}

pub(crate) fn source_explosion_size_scale(size: u16) -> Option<u8> {
    let high = size.to_be_bytes()[0] & 0x7F;
    if high == 0 || high > SOURCE_EXPLOSION_KILL_SIZE_HIGH {
        return None;
    }
    Some(high)
}

pub(crate) fn source_explosion_frame_index(size: u16) -> Option<u8> {
    if source_explosion_size_scale(size).is_none() || size < SOURCE_EXPLOSION_INITIAL_SIZE {
        return None;
    }
    let offset = size.wrapping_sub(SOURCE_EXPLOSION_INITIAL_SIZE);
    if !offset.is_multiple_of(SOURCE_EXPLOSION_SIZE_DELTA) {
        return None;
    }
    u8::try_from(offset / SOURCE_EXPLOSION_SIZE_DELTA).ok()
}

#[cfg(feature = "legacy-tools")]
pub(crate) fn source_player_explosion_color_index_for_pointer(
    color_pointer: u16,
    color_table_address: u16,
) -> Option<u8> {
    let offset = color_pointer.checked_sub(color_table_address)?;
    let index = u8::try_from(offset).ok()?;
    (usize::from(index) < SOURCE_PLAYER_EXPLOSION_COLORS.len()).then_some(index)
}

fn player_explosion_random_seed_step(seed: u16) -> u16 {
    let [mut high, mut low] = seed.to_be_bytes();
    let mut accumulator = low;
    accumulator >>= 1;
    accumulator ^= low;
    accumulator >>= 1;
    let carry = accumulator & 1 != 0;
    let (next_high, next_carry) = rotate_right_through_carry(high, carry);
    high = next_high;
    let (next_low, _) = rotate_right_through_carry(low, next_carry);
    low = next_low;
    u16::from_be_bytes([high, low])
}

const fn rotate_right_through_carry(value: u8, carry: bool) -> (u8, bool) {
    let next_carry = value & 1 != 0;
    let next_value = (value >> 1) | if carry { 0x80 } else { 0 };
    (next_value, next_carry)
}

const fn ones_complement_abs_word(value: u16) -> u16 {
    if value & 0x8000 == 0 { value } else { !value }
}

const fn logical_shift_right_word(value: u16) -> u16 {
    value >> 1
}

fn source_object_detail_render_layer(list: ObjectEvidenceList) -> Option<RenderLayer> {
    match list {
        ObjectEvidenceList::Active => Some(RenderLayer::Objects),
        ObjectEvidenceList::Projectile => Some(RenderLayer::Projectiles),
        ObjectEvidenceList::Inactive => None,
    }
}

fn enemy_sprite(kind: EnemyKind) -> SpriteId {
    match kind {
        EnemyKind::Lander => SpriteId::ENEMY_LANDER,
        EnemyKind::Mutant => SpriteId::ENEMY_MUTANT,
        EnemyKind::Bomber => SpriteId::ENEMY_BOMBER,
        EnemyKind::Pod => SpriteId::ENEMY_POD,
        EnemyKind::Swarmer => SpriteId::ENEMY_SWARMER,
        EnemyKind::Baiter => SpriteId::ENEMY_BAITER,
    }
}

const PROJECTILE_SPRITE_SIZE: (u8, u8) = (8, 2);
const PLAYER_PROJECTILE_COLLISION_SIZE: (u8, u8) = (8, 1);
const ENEMY_PROJECTILE_SPRITE_SIZE: (u8, u8) = (4, 6);
const ENEMY_PROJECTILE_COLLISION_SIZE: (u8, u8) = SOURCE_BOMB_SHELL_PICTURE_SIZE;
const PLAYER_SPRITE_SIZE: (u8, u8) = (16, 8);
const PLAYER_COLLISION_SIZE: (u8, u8) = (8, 6);
const SCORE_DIGIT_DISPLAY_COUNT: usize = 6;
const SCORE_DISPLAY_MAX: u32 = 999_999;
const PLAYER_ONE_SCORE_ORIGIN: [f32; 2] = [18.0, 21.0];
const PLAYER_TWO_SCORE_ORIGIN: [f32; 2] = [214.0, 21.0];
const SCORE_DIGIT_STEP: [f32; 2] = [8.0, 0.0];
const SCORE_DIGIT_SIZE: [f32; 2] = [6.0, 8.0];
const PLAYER_LIFE_STOCK_DISPLAY_LIMIT: u8 = 5;
const SMART_BOMB_STOCK_DISPLAY_LIMIT: u8 = 3;
const PLAYER_ONE_LIFE_STOCK_ORIGIN: [f32; 2] = [18.0, 13.0];
const PLAYER_TWO_LIFE_STOCK_ORIGIN: [f32; 2] = [214.0, 13.0];
const PLAYER_LIFE_STOCK_STEP: [f32; 2] = [12.0, 0.0];
const PLAYER_LIFE_STOCK_SIZE: [f32; 2] = [10.0, 4.0];
const PLAYER_ONE_SMART_BOMB_STOCK_ORIGIN: [f32; 2] = [70.0, 20.0];
const PLAYER_TWO_SMART_BOMB_STOCK_ORIGIN: [f32; 2] = [266.0, 20.0];
const SMART_BOMB_STOCK_STEP: [f32; 2] = [0.0, 4.0];
const SMART_BOMB_STOCK_SIZE: [f32; 2] = [6.0, 3.0];
const SOURCE_TOP_DISPLAY_BORDER_SEGMENTS: &[(u16, [f32; 2])] = &[
    (0x0028, [312.0, 2.0]),
    (0x2F08, [2.0, 32.0]),
    (0x7008, [2.0, 32.0]),
    (0x2F07, [130.0, 1.0]),
    (0x4C07, [16.0, 2.0]),
    (0x4C28, [16.0, 2.0]),
];
const SOURCE_ATTRACT_CREDITS_LABEL_SCREEN: u16 = 0x28E5;
const SOURCE_ATTRACT_CREDITS_NUMBER_SCREEN: u16 = 0x48E5;
const SOURCE_ATTRACT_PRESENTS_ELECTRONICS_SCREEN: u16 = 0x3258;
const SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES: &[(&str, u16)] = &[
    ("SCANV", 0x4330),
    ("LANDV", 0x1C70),
    ("MUTV", 0x3C70),
    ("BAITV", 0x5F70),
    ("BOMBV", 0x1CA8),
    ("SWRMPV", 0x40A8),
    ("SWARMV", 0x5CA8),
];
const SOURCE_ATTRACT_WILLIAMS_LOGO_SCREEN: u16 = 0x363C;
const SOURCE_ATTRACT_WILLIAMS_LOGO_SIZE: [f32; 2] = [92.0, 19.0];
const SOURCE_ATTRACT_DEFENDER_WORDMARK_SCREEN: u16 = 0x3090;
const SOURCE_DEFENDER_WORDMARK_SIZE: [f32; 2] = [120.0, 24.0];
const SOURCE_ATTRACT_COPYRIGHT_STRIP_SCREEN: u16 = 0x3BD0;
const SOURCE_ATTRACT_COPYRIGHT_STRIP_SIZE: [f32; 2] = [80.0, 8.0];
const SOURCE_FINAL_GAME_OVER_SCREEN: u16 = 0x3E80;
const SOURCE_PLAYER_START_PROMPT_SCREEN: u16 = 0x3C80;
const SOURCE_PLAYER_SWITCH_LABEL_SCREEN: u16 = 0x3C78;
const SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN: u16 = 0x3E88;
const SOURCE_WAVE_COMPLETION_STATUS_LINES: &[(&str, u16)] =
    &[("ATWV", 0x3850), ("COMPV", 0x3D60), ("BONSX", 0x3C90)];
const SOURCE_WAVE_COMPLETION_WAVE_NUMBER_SCREEN: u16 = 0x6550;
const SOURCE_WAVE_COMPLETION_MULTIPLIER_NUMBER_SCREEN: u16 = 0x5890;
const SOURCE_SURVIVOR_BONUS_FIRST_HUMAN_SCREEN: u16 = 0x3CA0;
const SOURCE_SURVIVOR_BONUS_HUMAN_STEP: u8 = 0x04;
const SOURCE_SURVIVOR_BONUS_HUMAN_LIMIT: usize = 10;
const SOURCE_SURVIVOR_BONUS_HUMAN_SIZE: [f32; 2] = [4.0, 8.0];
const SOURCE_HALL_OF_FAME_PLAYER_LABEL_SCREEN: u16 = 0x3E38;
const SOURCE_HALL_OF_FAME_INSTRUCTIONS_TOP_LEFT: u16 = 0x1458;
const SOURCE_HALL_OF_FAME_INITIALS_SCREEN: u16 = 0x46AC;
const SOURCE_HALL_OF_FAME_UNDERLINE_SCREEN: u16 = 0x45B7;
const SOURCE_HALL_OF_FAME_LINE_VERTICAL_OFFSETS: [u8; 4] = [0x00, 0x0A, 0x1E, 0x32];
const SOURCE_HALL_OF_FAME_INITIAL_HORIZONTAL_OFFSETS: [u8; 3] = [0x00, 0x08, 0x10];
const SOURCE_HALL_OF_FAME_UNDERLINE_INITIAL_STEP: u8 = 0x08;
const SOURCE_HALL_OF_FAME_UNDERLINE_WORD_HORIZONTAL_OFFSETS: [u8; 4] = [0x04, 0x03, 0x02, 0x01];
const SOURCE_HALL_OF_FAME_UNDERLINE_WORD_SIZE: [f32; 2] = [2.0, 2.0];
const SOURCE_HALL_OF_FAME_INSTRUCTION_MESSAGES: [&str; 4] = ["HOFV1", "HOFV2", "HOFV3", "HOFV4"];
const SOURCE_HALL_OF_FAME_DISPLAY_HEADINGS: [(&str, u16); 5] = [
    ("HALLD_TITLE", 0x3854),
    ("HALLD_TODAYS", 0x2268),
    ("HALLD_ALL_TIME", 0x6068),
    ("HALLD_GREATEST", 0x1E72),
    ("HALLD_GREATEST", 0x5F72),
];
const SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_SCREEN: u16 = 0x1E7B;
const SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_RIGHT_START: u8 = 0x5F;
const SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_RIGHT_END: u8 = 0x41;
const SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_LEFT_START: u8 = 0x1E;
const SOURCE_HALL_OF_FAME_LOGO_SCREEN: u16 = 0x3038;
const SOURCE_HALL_OF_FAME_LOGO_SIZE: [f32; 2] = SOURCE_DEFENDER_WORDMARK_SIZE;
const SOURCE_HALL_OF_FAME_TODAYS_TABLE_SCREEN: u16 = 0x1886;
const SOURCE_HALL_OF_FAME_ALL_TIME_TABLE_SCREEN: u16 = 0x5986;
const SOURCE_HALL_OF_FAME_TABLE_ROW_STEP: u8 = 0x0A;
const SOURCE_HALL_OF_FAME_TABLE_INITIALS_OFFSET: u8 = 0x05;
const SOURCE_HALL_OF_FAME_TABLE_SCORE_OFFSET: u8 = 0x13;
const SOURCE_HALL_OF_FAME_SCORE_TEXT_LEN: usize = 6;

fn enemy_sprite_size(kind: EnemyKind) -> (u8, u8) {
    match kind {
        EnemyKind::Lander => (12, 8),
        EnemyKind::Mutant => (12, 8),
        EnemyKind::Bomber => (10, 8),
        EnemyKind::Pod => (10, 8),
        EnemyKind::Swarmer => (8, 6),
        EnemyKind::Baiter => (12, 8),
    }
}

fn enemy_collision_size(enemy: EnemySnapshot) -> (u8, u8) {
    enemy.source_picture_descriptor().size
}

fn human_collision_size(human: HumanSnapshot) -> (u8, u8) {
    source_human_picture_descriptor(human.source_picture_frame).size
}

fn enemy_score(kind: EnemyKind) -> u32 {
    match kind {
        EnemyKind::Lander => 150,
        EnemyKind::Mutant => 150,
        EnemyKind::Bomber => 250,
        EnemyKind::Pod => 1000,
        EnemyKind::Swarmer => 150,
        EnemyKind::Baiter => 200,
    }
}

fn source_enemy_hit_sound_event(kind: EnemyKind) -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: source_enemy_hit_sound_command(kind),
    }
}

fn source_enemy_hit_sound_command(kind: EnemyKind) -> u8 {
    match kind {
        EnemyKind::Lander => SOURCE_LHSND_SOUND_COMMAND,
        EnemyKind::Mutant => SOURCE_SCHSND_SOUND_COMMAND,
        EnemyKind::Bomber => SOURCE_TIHSND_SOUND_COMMAND,
        EnemyKind::Pod => SOURCE_PRHSND_SOUND_COMMAND,
        EnemyKind::Swarmer => SOURCE_SWHSND_SOUND_COMMAND,
        EnemyKind::Baiter => SOURCE_UFHSND_SOUND_COMMAND,
    }
}

fn push_source_enemy_shot_sound_if_fired(
    kind: EnemyKind,
    projectile_count_before: usize,
    projectile_count_after: usize,
    sound_events: &mut Vec<SoundEvent>,
) {
    if projectile_count_after <= projectile_count_before {
        return;
    }

    if let Some(event) = source_enemy_shot_sound_event(kind) {
        sound_events.push(event);
    }
}

fn source_enemy_shot_sound_event(kind: EnemyKind) -> Option<SoundEvent> {
    source_enemy_shot_sound_command(kind)
        .map(|command| SoundEvent::UnmappedSoundCommand { command })
}

fn source_enemy_shot_sound_command(kind: EnemyKind) -> Option<u8> {
    match kind {
        EnemyKind::Lander => Some(SOURCE_LSHSND_SOUND_COMMAND),
        EnemyKind::Mutant => Some(SOURCE_SSHSND_SOUND_COMMAND),
        EnemyKind::Swarmer => Some(SOURCE_SWSSND_SOUND_COMMAND),
        EnemyKind::Baiter => Some(SOURCE_USHSND_SOUND_COMMAND),
        EnemyKind::Bomber | EnemyKind::Pod => None,
    }
}

fn source_lander_pickup_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_LPKSND_SOUND_COMMAND,
    }
}

fn source_lander_suck_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_LSKSND_SOUND_COMMAND,
    }
}

fn source_laser_fire_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_LASSND_SOUND_COMMAND,
    }
}

fn source_smart_bomb_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_SBSND_SOUND_COMMAND,
    }
}

fn source_player_death_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_PDSND_SOUND_COMMAND,
    }
}

fn source_hyperspace_appearance_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_APSND_SOUND_COMMAND,
    }
}

fn source_astronaut_release_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_ASCSND_SOUND_COMMAND,
    }
}

fn source_astronaut_catch_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_ACSND_SOUND_COMMAND,
    }
}

fn source_astronaut_safe_landing_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_ALSND_SOUND_COMMAND,
    }
}

fn source_astronaut_hit_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_AHSND_SOUND_COMMAND,
    }
}

fn source_terrain_blow_start_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_AHSND_SOUND_COMMAND,
    }
}

fn source_terrain_blow_complete_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_TBSND_SOUND_COMMAND,
    }
}

fn source_bomb_collision_sound_event() -> SoundEvent {
    SoundEvent::UnmappedSoundCommand {
        command: SOURCE_AHSND_SOUND_COMMAND,
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl GameSimulation for Game {
    fn state(&self) -> GameState {
        self.state()
    }

    fn step(&mut self, input: GameInput) -> GameFrame {
        Game::step(self, input)
    }
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

fn initial_state() -> GameState {
    GameState {
        frame: 0,
        phase: GamePhase::Attract,
        credits: 0,
        current_player: 1,
        player_count: 1,
        wave: DEFAULT_CABINET_WAVE,
        wave_profile: WaveProfileSnapshot::for_wave(DEFAULT_CABINET_WAVE),
        player: PlayerSnapshot {
            position: (world_word(0), world_word(0)),
            velocity: (WorldVector::default(), WorldVector::default()),
            direction: Direction::Right,
            lives: DEFAULT_PLAYER_LIVES,
            smart_bombs: 0,
        },
        player_stocks: [PlayerStockSnapshot::new(DEFAULT_PLAYER_LIVES, 0); 2],
        scores: ScoreSnapshot {
            player_one: 0,
            player_two: 0,
            high_score: DEFAULT_HIGH_SCORE,
            next_bonus: DEFAULT_REPLAY_SCORE,
        },
        attract: AttractPresentationSnapshot::for_page_frame(0),
        high_score_initials: HighScoreInitialsState::EMPTY,
        high_score_entry: None,
        high_score_submission: None,
        high_score_tables: HighScoreTablesSnapshot::DEFAULT,
        game_over: GameOverSnapshot::NONE,
        world: WorldSnapshot::default(),
    }
}

fn world_word(word: u16) -> WorldVector {
    WorldVector::from_subpixels(i32::from(word) << 8)
}

fn world_vector_pixels(vector: WorldVector) -> f32 {
    vector.subpixels() as f32 / WorldVector::SUBPIXELS_PER_PIXEL as f32
}

#[cfg(test)]
fn accepted_r3_visual_signature(frame: u64) -> Option<u32> {
    match frame {
        1..=2 => Some(2962226826u32),
        3..=4 => Some(1054242137u32),
        5..=6 => Some(3423556472u32),
        7..=8 => Some(2147797947u32),
        9..=10 => Some(1529887192u32),
        11..=12 => Some(2949545184u32),
        13..=14 => Some(2740317286u32),
        15..=16 => Some(3185980500u32),
        17..=18 => Some(170397484u32),
        19..=20 => Some(1726306935u32),
        21..=22 => Some(3836145658u32),
        23..=24 => Some(3689607546u32),
        25..=26 => Some(1192733655u32),
        27..=28 => Some(3216351317u32),
        29..=30 => Some(2305674611u32),
        31..=32 => Some(2758722426u32),
        33..=34 => Some(3159573997u32),
        35..=36 => Some(3883167324u32),
        37..=38 => Some(1864677123u32),
        39..=40 => Some(739121987u32),
        41..=42 => Some(3355169406u32),
        43..=44 => Some(3576441390u32),
        45..=46 => Some(1914860891u32),
        47..=48 => Some(1914847654u32),
        49..=50 => Some(167562569u32),
        51..=52 => Some(871004527u32),
        53..=54 => Some(3783109946u32),
        55..=56 => Some(4173964716u32),
        57..=58 => Some(4176109546u32),
        59..=60 => Some(78267750u32),
        61..=62 => Some(2185016249u32),
        63..=64 => Some(3922624675u32),
        65..=66 => Some(2819886426u32),
        67..=68 => Some(418124789u32),
        69..=70 => Some(3871103769u32),
        71..=72 => Some(3595509303u32),
        73..=74 => Some(2688149849u32),
        75..=76 => Some(1484287141u32),
        77..=78 => Some(1625265524u32),
        79..=80 => Some(2448426513u32),
        81..=82 => Some(994718349u32),
        83..=84 => Some(1953705603u32),
        85..=86 => Some(1194791227u32),
        87..=88 => Some(506680067u32),
        89..=90 => Some(2964673846u32),
        91..=92 => Some(2808029433u32),
        93..=94 => Some(4053725625u32),
        95..=96 => Some(1810298224u32),
        97..=98 => Some(1494060812u32),
        99..=100 => Some(2359125305u32),
        101..=102 => Some(4244048944u32),
        103..=104 => Some(2564858811u32),
        105..=106 => Some(2082746170u32),
        107..=108 => Some(2407752444u32),
        109..=110 => Some(643672350u32),
        111..=112 => Some(4089535518u32),
        113..=114 => Some(3034517100u32),
        115..=116 => Some(943028751u32),
        117..=118 => Some(2787473171u32),
        119..=120 => Some(2207498731u32),
        121..=122 => Some(271917215u32),
        123..=124 => Some(1975190971u32),
        125..=126 => Some(3125151796u32),
        127..=128 => Some(3225579304u32),
        129..=130 => Some(2822684100u32),
        131..=132 => Some(971548817u32),
        133..=134 => Some(864713817u32),
        135..=136 => Some(1923762177u32),
        137..=138 => Some(933497632u32),
        139..=140 => Some(3157349685u32),
        141..=142 => Some(4025863219u32),
        143..=144 => Some(2185429703u32),
        145..=146 => Some(74378108u32),
        147..=148 => Some(2477009910u32),
        149..=150 => Some(1160500901u32),
        151..=152 => Some(2949378946u32),
        153..=154 => Some(671565695u32),
        155..=156 => Some(296408545u32),
        157..=158 => Some(717192573u32),
        159..=160 => Some(1789236868u32),
        161..=162 => Some(4005648030u32),
        163..=164 => Some(2396158352u32),
        165..=166 => Some(1681194114u32),
        167..=168 => Some(3916725715u32),
        169..=170 => Some(1174864853u32),
        171..=172 => Some(2887059453u32),
        173..=174 => Some(1298150614u32),
        175..=176 => Some(2579610357u32),
        177..=178 => Some(1890156533u32),
        179..=180 => Some(2543283989u32),
        181..=182 => Some(663973590u32),
        183..=184 => Some(3693357442u32),
        185..=186 => Some(2654330245u32),
        187..=188 => Some(3591213495u32),
        189..=190 => Some(1424804443u32),
        191..=192 => Some(3449913518u32),
        193..=194 => Some(1569877951u32),
        195..=196 => Some(2663818674u32),
        197..=198 => Some(41179300u32),
        199..=200 => Some(3531977015u32),
        201..=202 => Some(2422067884u32),
        203..=204 => Some(4278367659u32),
        205..=206 => Some(3251468679u32),
        207..=208 => Some(3954183251u32),
        209..=210 => Some(3837076080u32),
        211..=212 => Some(2141212063u32),
        213..=214 => Some(1874996958u32),
        215..=216 => Some(333482869u32),
        217..=218 => Some(4012826636u32),
        219..=220 => Some(1212134273u32),
        221..=222 => Some(1156683936u32),
        223..=224 => Some(1813057864u32),
        225 => Some(2667859701u32),
        226..=347 => Some(2222029249u32),
        348 => Some(1986720743u32),
        349 => Some(2434865807u32),
        350 => Some(1343203852u32),
        351 => Some(1337858580u32),
        352 => Some(3219139578u32),
        353 => Some(629065165u32),
        354 => Some(2283183261u32),
        355 => Some(819918517u32),
        356 => Some(3314685550u32),
        357 => Some(2592152260u32),
        358 => Some(1867614032u32),
        359 => Some(4081625907u32),
        360 => Some(4049147811u32),
        361 => Some(4119883768u32),
        362 => Some(389165004u32),
        363 => Some(4040133997u32),
        364 => Some(713856524u32),
        365 => Some(3550170032u32),
        366 => Some(3335062975u32),
        367 => Some(573775383u32),
        368 => Some(1087801868u32),
        369 => Some(2630713014u32),
        370 => Some(2245365138u32),
        371 => Some(2986568653u32),
        372 => Some(4250111703u32),
        373 => Some(1904319187u32),
        374 => Some(622109312u32),
        375 => Some(3952154500u32),
        376 => Some(612281254u32),
        377 => Some(2603136060u32),
        378 => Some(318196819u32),
        379 => Some(1763537262u32),
        380 => Some(4012746880u32),
        381 => Some(1016026627u32),
        382 => Some(3593729863u32),
        383 => Some(2185161509u32),
        384 => Some(1902756924u32),
        385 => Some(1780044060u32),
        386 => Some(1393667893u32),
        387 => Some(1677047582u32),
        388 => Some(3535298702u32),
        389 => Some(279834986u32),
        390 => Some(2424895872u32),
        391 => Some(4119540377u32),
        392 => Some(3903614265u32),
        393 => Some(3831488240u32),
        394 => Some(634384209u32),
        395 => Some(3130869203u32),
        396 => Some(31062761u32),
        397..=398 => Some(1891765253u32),
        399..=400 => Some(1050926502u32),
        401..=402 => Some(1956463352u32),
        403..=404 => Some(2911064518u32),
        405..=406 => Some(3237484186u32),
        407..=408 => Some(4147607314u32),
        409..=410 => Some(1207338880u32),
        411..=412 => Some(2345584171u32),
        413..=414 => Some(4185358877u32),
        415..=416 => Some(3615900114u32),
        417..=418 => Some(649532446u32),
        419..=420 => Some(3385758538u32),
        421..=422 => Some(2409284461u32),
        423..=424 => Some(1870372669u32),
        425..=426 => Some(2945323084u32),
        427..=428 => Some(2369852352u32),
        429..=430 => Some(702063314u32),
        431..=432 => Some(1813808566u32),
        433..=434 => Some(3961739814u32),
        435..=436 => Some(103747122u32),
        437..=438 => Some(206003092u32),
        439..=440 => Some(1411839069u32),
        441..=442 => Some(3236242577u32),
        443..=444 => Some(102035639u32),
        445..=446 => Some(693092707u32),
        447..=463 => Some(3960974809u32),
        464..=561 => Some(1270435565u32),
        562..=563 => Some(2296702972u32),
        564..=1024 => Some(1993604823u32),
        1025..=1026 => Some(360618183u32),
        1027..=1123 => Some(3196225782u32),
        1124 => Some(1400075496u32),
        1125 => Some(2743603212u32),
        1126 => Some(2235855140u32),
        1127 => Some(1159013671u32),
        1128 => Some(1184133452u32),
        1129 => Some(1632834619u32),
        1130 => Some(2870503153u32),
        1131 => Some(1141236412u32),
        1132 => Some(4133341777u32),
        1133 => Some(53991316u32),
        1134 => Some(1447441849u32),
        1135 => Some(3581251561u32),
        1136 => Some(1460875539u32),
        1137 => Some(4235864844u32),
        1138 => Some(2346098791u32),
        1139 => Some(482452162u32),
        1140 => Some(803474416u32),
        1141 => Some(740457053u32),
        1142 => Some(2939077440u32),
        1143 => Some(3412061521u32),
        1144 => Some(2061898767u32),
        1145 => Some(20384635u32),
        1146 => Some(3140746114u32),
        1147 => Some(1874963291u32),
        1148 => Some(3161497675u32),
        1149 => Some(3420164906u32),
        1150 => Some(4010489085u32),
        1151 => Some(1675995u32),
        1152 => Some(987536940u32),
        1153 => Some(1562584391u32),
        1154 => Some(832752569u32),
        1155 => Some(2954711963u32),
        1156 => Some(2636380567u32),
        1157 => Some(2737944712u32),
        1158 => Some(2942970661u32),
        1159 => Some(23165600u32),
        1160 => Some(1034633915u32),
        1161 => Some(3007335554u32),
        1162 => Some(1673428701u32),
        1163 => Some(2938571803u32),
        1164 => Some(4206826444u32),
        1165 => Some(753199034u32),
        1166 => Some(711516073u32),
        1167 => Some(137489621u32),
        1168 => Some(2531366024u32),
        1169 => Some(54101325u32),
        1170 => Some(3101277713u32),
        1171 => Some(2722385027u32),
        1172 => Some(3810335740u32),
        1173 => Some(221652501u32),
        1174 => Some(1086271805u32),
        1175 => Some(617041081u32),
        1176 => Some(18081227u32),
        1177 => Some(604197756u32),
        1178 => Some(57026218u32),
        1179 => Some(617077376u32),
        1180 => Some(3673207957u32),
        1181 => Some(1194617052u32),
        1182 => Some(1833480902u32),
        1183 => Some(1046936856u32),
        1184 => Some(2190669726u32),
        1185 => Some(3971852525u32),
        1186 => Some(3523771021u32),
        1187 => Some(2953777528u32),
        1188 => Some(1409603171u32),
        1189 => Some(1737471587u32),
        1190 => Some(3604209023u32),
        1191 => Some(937967059u32),
        1192 => Some(3480350440u32),
        1193 => Some(2970304792u32),
        1194 => Some(898673074u32),
        1195 => Some(1945133126u32),
        1196 => Some(4276510214u32),
        1197 => Some(3831925678u32),
        1198 => Some(1598023817u32),
        1199 => Some(578905158u32),
        1200 => Some(2736237466u32),
        1201 => Some(734775587u32),
        1202 => Some(653774984u32),
        1203 => Some(1356801736u32),
        1204 => Some(178717231u32),
        1205 => Some(72141257u32),
        1206 => Some(2917218934u32),
        1207 => Some(1291451067u32),
        1208 => Some(2005379725u32),
        1209 => Some(1459418332u32),
        1210 => Some(870553705u32),
        1211 => Some(3350979936u32),
        1212 => Some(532010902u32),
        1213 => Some(547189091u32),
        1214 => Some(3708205836u32),
        1215 => Some(3025449279u32),
        1216 => Some(1828836284u32),
        1217 => Some(1912403834u32),
        1218 => Some(2681582u32),
        1219 => Some(1522432607u32),
        1220 => Some(751739102u32),
        1221 => Some(2405591640u32),
        1222 => Some(1894977301u32),
        1223 => Some(27764195u32),
        1224 => Some(1162261030u32),
        1225 => Some(4001896637u32),
        1226 => Some(3436193068u32),
        1227 => Some(648055075u32),
        1228 => Some(1032290917u32),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        renderer::{
            ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT, Color, RenderLayer, RenderLayerCounts,
            RenderScene, SpriteId, SurfaceSize, TextureAtlas,
            source_attract_williams_logo_pixel_path,
        },
        systems::{
            GameSimulation, HighScoreInitialsState, PlayerControlIntent, PlayerMotionState,
            PlayerMotionSystem, ScreenPosition, ScreenVelocity, advance_one_frame,
        },
    };

    use super::PlayerStockSnapshot;

    use super::{
        ATTRACT_DEFENDER_WORDMARK_REVEAL_FRAMES, ATTRACT_DEFENDER_WORDMARK_START_FRAME,
        ATTRACT_HALL_OF_FAME_START_FRAME, ATTRACT_WILLIAMS_LOGO_REVEAL_FRAMES,
        AttractPresentationPage, AttractPresentationSnapshot, COIN_CREDIT_DELAY_FRAMES,
        COIN_CREDIT_SOUND_DELAY_FRAMES, DEFAULT_CABINET_WAVE, DEFAULT_HIGH_SCORE,
        DEFAULT_PLAYER_LIVES, DEFAULT_REPLAY_SCORE, Direction, EXPANDED_OBJECT_DETAIL_LIMIT,
        EnemyKind, EnemyReserveSnapshot, EnemySnapshot, ExpandedObjectDetailSnapshot,
        ExpandedObjectEvidenceSnapshot, ExpandedObjectKind, ExplosionKind, Game, GameEvent,
        GameEvents, GameFrame, GameInput, GameOverSnapshot, GamePhase,
        HALL_OF_FAME_NO_ENTRY_DELAY_FRAMES, HALL_OF_FAME_STALL_FRAMES, HighScoreEntrySnapshot,
        HighScoreSubmissionSnapshot, HighScoreTableEntrySnapshot, HighScoreTablesSnapshot,
        HumanSnapshot, OBJECT_EVIDENCE_DETAIL_LIMIT, ObjectEvidenceCategory,
        ObjectEvidenceDetailSnapshot, ObjectEvidenceList, ObjectEvidenceSnapshot,
        PLAYER_DEATH_GAME_OVER_SLEEP_FRAMES, PLAYER_EXPLOSION_PIECE_LIMIT,
        PLAYER_SWITCH_SLEEP_FRAMES, PlayerExplosionCloudSnapshot, PlayerExplosionPieceSnapshot,
        ProjectileSnapshot, SOURCE_ACTIVE_BAITER_LIMIT, SOURCE_ACTIVE_SWARMER_LIMIT,
        SOURCE_BAITER_LOOP_SLEEP_TICKS, SOURCE_EXPLOSION_INITIAL_SIZE,
        SOURCE_EXPLOSION_LIFETIME_FRAMES, SOURCE_EXPLOSION_SIZE_DELTA,
        SOURCE_GAME_EXEC_SLEEP_FRAMES, SOURCE_LANDER_ORBIT_SLEEP_TICKS, SOURCE_LHSND_SOUND_COMMAND,
        SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS, SOURCE_MUTANT_LOOP_SLEEP_TICKS,
        SOURCE_PLAYFIELD_Y_MIN, SOURCE_POD_SWARMER_REQUEST_LIMIT, SOURCE_PRHSND_SOUND_COMMAND,
        SOURCE_SCHSND_SOUND_COMMAND, SOURCE_SCORE_POPUP_LIFETIME_TICKS,
        SOURCE_SWHSND_SOUND_COMMAND, SOURCE_TERRAIN_BLOW_ITERATION_LIMIT,
        SOURCE_TERRAIN_BLOW_OVERLOAD_COUNTER, SOURCE_TERRAIN_BLOW_SLEEP_TICKS,
        SOURCE_TIHSND_SOUND_COMMAND, SOURCE_UFHSND_SOUND_COMMAND, SOURCE_VISUAL_STATE,
        START_PLAYFIELD_DELAY_FRAMES, START_SOUND_DELAY_FRAMES, ScannerRadarBlipKind,
        ScannerRadarSnapshot, ScannerRadarStage, ScorePopupKind, SoundEvent, SourceBaiterSnapshot,
        SourceBomberSnapshot, SourceLanderSnapshot, SourceMutantSnapshot, SourcePodSnapshot,
        SourceRandSnapshot, SourceSwarmerSnapshot, TerrainBlowStage, WaveProfileSnapshot,
        WorldSnapshot, WorldVector, source_astronaut_catch_sound_event,
        source_astronaut_hit_sound_event, source_astronaut_release_sound_event,
        source_astronaut_safe_landing_sound_event, source_bomb_collision_sound_event,
        source_enemy_hit_sound_event, source_enemy_shot_sound_event,
        source_hyperspace_appearance_sound_event, source_lander_pickup_sound_event,
        source_lander_suck_sound_event, source_laser_fire_sound_event,
        source_player_death_sound_event, source_smart_bomb_sound_event,
        source_terrain_blow_complete_sound_event, source_terrain_blow_start_sound_event,
    };

    #[test]
    fn world_vectors_preserve_subpixel_units() {
        let vector = WorldVector::from_subpixels(512);

        assert_eq!(vector.subpixels(), 512);
        assert_eq!(WorldVector::SUBPIXELS_PER_PIXEL, 256);
    }

    #[test]
    fn wave_profile_uses_source_wave_table_values() {
        let first = WaveProfileSnapshot::for_wave(1);
        assert_eq!(first.landers, 15);
        assert_eq!(first.bombers, 0);
        assert_eq!(first.pods, 0);
        assert_eq!(first.wave_time, 30);
        assert_eq!(first.baiter_delay, 212);

        let second = WaveProfileSnapshot::for_wave(2);
        assert_eq!(second.landers, 20);
        assert_eq!(second.bombers, 3);
        assert_eq!(second.pods, 1);
        assert_eq!(second.lander_x_velocity, 30);
        assert_eq!(second.baiter_shot_time, 13);

        let fifth = WaveProfileSnapshot::for_wave(5);
        assert_eq!(fifth.lander_x_velocity, 48);
        assert_eq!(fifth.baiter_delay, 144);
    }

    #[test]
    fn clean_initial_humans_carry_source_target_list_slots() {
        let first = WorldSnapshot::for_wave(1);
        let expected_positions = [
            ScreenPosition::new(0x12, super::SOURCE_ASTRO_RESTORE_Y),
            ScreenPosition::new(0x09, super::SOURCE_ASTRO_RESTORE_Y),
            ScreenPosition::new(0x54, super::SOURCE_ASTRO_RESTORE_Y),
            ScreenPosition::new(0x5A, super::SOURCE_ASTRO_RESTORE_Y),
            ScreenPosition::new(0x8D, super::SOURCE_ASTRO_RESTORE_Y),
            ScreenPosition::new(0x86, super::SOURCE_ASTRO_RESTORE_Y),
            ScreenPosition::new(0xC3, super::SOURCE_ASTRO_RESTORE_Y),
            ScreenPosition::new(0xD1, super::SOURCE_ASTRO_RESTORE_Y),
            ScreenPosition::new(0x09, super::SOURCE_ASTRO_RESTORE_Y),
            ScreenPosition::new(0x14, super::SOURCE_ASTRO_RESTORE_Y),
        ];
        let expected_x_fractions = [0xAD, 0x56, 0xAB, 0x55, 0x2A, 0x95, 0x4A, 0xA5, 0xD2, 0x69];
        let expected_picture_frames = [2, 0, 2, 2, 0, 2, 0, 2, 0, 2];

        assert_eq!(
            first.humans.len(),
            usize::from(super::SOURCE_START_HUMAN_COUNT)
        );
        assert_eq!(
            first
                .humans
                .iter()
                .map(|human| human.position)
                .collect::<Vec<_>>(),
            expected_positions.to_vec()
        );
        assert_eq!(
            first
                .humans
                .iter()
                .map(|human| human.source_x_fraction)
                .collect::<Vec<_>>(),
            expected_x_fractions.to_vec()
        );
        assert_eq!(
            first
                .humans
                .iter()
                .map(|human| human.source_picture_frame)
                .collect::<Vec<_>>(),
            expected_picture_frames.to_vec()
        );
        for (index, human) in first.humans.iter().enumerate() {
            assert_eq!(
                human.source_target_slot_address,
                Some(super::source_target_list_slot_address(index))
            );
        }
        assert_eq!(super::source_target_list_slot_address(31), 0xA158);
    }

    #[test]
    fn clean_source_astronaut_process_walks_target_list_human() {
        let mut world = WorldSnapshot {
            terrain: vec![super::TerrainSegment {
                position: ScreenPosition::new(0x20, 0xD4),
                size: (0x80, 8),
            }],
            humans: vec![
                super::source_target_list_human(ScreenPosition::new(0x30, 0xD0), 0, 0, 0),
                super::source_target_list_human(ScreenPosition::new(0x40, 0xD0), 0x10, 0, 1),
                super::source_target_list_human(ScreenPosition::new(0x50, 0xD0), 0xF0, 2, 2),
            ],
            source_rng: SourceRandSnapshot {
                seed: 0x09,
                ..SourceRandSnapshot::default()
            },
            source_astronaut_cursor_address: Some(super::SOURCE_TARGET_LIST_BASE),
            ..WorldSnapshot::default()
        };

        assert!(world.advance_source_astronaut_process());
        assert_eq!(
            world.source_astronaut_cursor_address,
            Some(super::source_target_list_slot_address(1))
        );
        assert_eq!(
            world.source_astronaut_sleep_ticks,
            super::SOURCE_ASTRONAUT_PROCESS_SLEEP_TICKS
        );
        assert_eq!(world.humans[0].position, ScreenPosition::new(0x30, 0xD0));
        assert_eq!(world.humans[1].position, ScreenPosition::new(0x3F, 0xD1));
        assert_eq!(world.humans[1].source_x_fraction, 0xF0);
        assert_eq!(world.humans[1].source_picture_frame, 1);

        assert!(!world.advance_source_astronaut_process());
        assert_eq!(world.source_astronaut_sleep_ticks, 1);
        assert_eq!(world.humans[2].position, ScreenPosition::new(0x50, 0xD0));

        world.source_astronaut_sleep_ticks = 0;
        world.source_rng.seed = 0x05;
        assert!(world.advance_source_astronaut_process());
        assert_eq!(
            world.source_astronaut_cursor_address,
            Some(super::source_target_list_slot_address(2))
        );
        assert_eq!(world.humans[2].position, ScreenPosition::new(0x50, 0xD0));
        assert_eq!(world.humans[2].source_x_fraction, 0xD0);
        assert_eq!(world.humans[2].source_picture_frame, 0);
    }

    #[test]
    fn clean_game_source_astronaut_walk_updates_picture_evidence() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;
        game.state.world.terrain = vec![super::TerrainSegment {
            position: ScreenPosition::new(0x20, 0xD4),
            size: (0x80, 8),
        }];
        game.state.world.humans = vec![
            super::source_target_list_human(ScreenPosition::new(0x30, 0xD0), 0, 0, 0),
            super::source_target_list_human(ScreenPosition::new(0x40, 0xD0), 0x10, 0, 1),
        ];
        game.state.world.source_astronaut_cursor_address = Some(super::SOURCE_TARGET_LIST_BASE);
        game.state.world.source_astronaut_sleep_ticks = 0;
        game.state.world.source_rng.seed = 0x09;

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            frame.state.world.source_astronaut_cursor_address,
            Some(super::source_target_list_slot_address(1))
        );
        assert_eq!(
            frame.state.world.humans[1].position,
            ScreenPosition::new(0x3F, 0xD1)
        );
        assert_eq!(frame.state.world.humans[1].source_picture_frame, 1);
        let detail = frame.state.world.object_evidence.details[1];
        assert_eq!(detail.world_position, Some((0x3FF0, 0xD100)));
        assert_eq!(detail.picture_label, Some("ASTP2"));
        assert_eq!(detail.picture_address, Some(0xF90B));
        assert_eq!(detail.primary_image_address, Some(0xFAEB));
        assert_eq!(detail.alternate_image_address, Some(0xFAFB));
    }

    #[test]
    fn clean_source_lander_target_selection_advances_tlist_cursor() {
        let mut humans = super::source_initial_target_list_humans();
        let mut cursor = Some(super::SOURCE_TARGET_LIST_BASE);

        assert_eq!(
            super::source_select_lander_target_index(&mut cursor, &humans),
            Some(1)
        );
        assert_eq!(cursor, Some(super::source_target_list_slot_address(1)));

        humans[2].carried_by_player = true;
        assert_eq!(
            super::source_select_lander_target_index(&mut cursor, &humans),
            Some(3)
        );
        assert_eq!(cursor, Some(super::source_target_list_slot_address(3)));

        cursor = Some(super::source_target_list_slot_address(9));
        assert_eq!(
            super::source_select_lander_target_index(&mut cursor, &humans),
            Some(0)
        );
        assert_eq!(cursor, Some(super::source_target_list_slot_address(0)));

        for human in &mut humans {
            human.carried = true;
        }
        assert_eq!(
            super::source_select_lander_target_index(&mut cursor, &humans),
            None
        );
        assert_eq!(cursor, Some(super::source_target_list_slot_address(0)));
    }

    #[test]
    fn clean_wave_spawns_source_profile_active_enemy_batch() {
        let first = WorldSnapshot::for_wave(1);
        assert_eq!(first.enemies.len(), 5);
        assert!(
            first
                .enemies
                .iter()
                .all(|enemy| enemy.kind == EnemyKind::Lander)
        );
        assert!(
            first
                .enemies
                .iter()
                .all(|enemy| enemy.source_lander.is_some())
        );
        assert_eq!(
            first
                .enemies
                .iter()
                .map(|enemy| enemy
                    .source_lander
                    .expect("source lander")
                    .target_human_index)
                .collect::<Vec<_>>(),
            vec![Some(1), Some(2), Some(3), Some(4), Some(5)]
        );
        assert_eq!(
            first.source_target_list_cursor_address,
            Some(super::source_target_list_slot_address(5))
        );
        assert_eq!(
            first.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 10,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(first.object_evidence.active_count, 15);
        assert_eq!(first.object_evidence.inactive_count, 10);
        assert_eq!(first.object_evidence.detail_count, 16);

        let second = WorldSnapshot::for_wave(2);

        assert_eq!(
            second
                .enemies
                .iter()
                .map(|enemy| enemy.kind)
                .collect::<Vec<_>>(),
            vec![
                EnemyKind::Lander,
                EnemyKind::Bomber,
                EnemyKind::Pod,
                EnemyKind::Lander,
                EnemyKind::Lander,
            ]
        );
        assert_eq!(
            second.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 17,
                bombers: 2,
                ..EnemyReserveSnapshot::default()
            }
        );
        let expected_initial_lander = super::source_lander_initial_spawn(
            WaveProfileSnapshot::for_wave(2),
            super::CLEAN_WAVE_SPAWN_POSITIONS[0],
            0,
        );
        let mut expected_initial_source_lander = expected_initial_lander
            .source_lander
            .expect("initial source lander");
        expected_initial_source_lander.target_human_index = Some(1);
        assert_eq!(second.enemies[0].velocity, expected_initial_lander.velocity);
        assert_eq!(
            second.enemies[0].source_lander,
            Some(expected_initial_source_lander)
        );
        assert_eq!(
            second.enemies[3]
                .source_lander
                .expect("second source lander")
                .target_human_index,
            Some(2)
        );
        assert_eq!(
            second.enemies[4]
                .source_lander
                .expect("third source lander")
                .target_human_index,
            Some(3)
        );
        assert_eq!(
            second.source_target_list_cursor_address,
            Some(super::source_target_list_slot_address(3))
        );
        assert_eq!(second.enemies[1].velocity, ScreenVelocity::new(-1, 0));
        assert_eq!(
            second.enemies[1].source_bomber,
            Some(super::source_bomber_spawn(
                WaveProfileSnapshot::for_wave(2),
                1
            ))
        );
        let expected_initial_pod =
            super::source_pod_initial_spawn(super::CLEAN_WAVE_SPAWN_POSITIONS[2], 2);
        assert_eq!(second.enemies[2].velocity, expected_initial_pod.velocity);
        assert_eq!(
            second.enemies[2].source_pod,
            expected_initial_pod.source_pod
        );
        assert_eq!(
            second.object_evidence.details[1].object_category,
            Some(ObjectEvidenceCategory::Bomber)
        );
        assert_eq!(
            second.object_evidence.details[1].mapped_sprite,
            Some(SpriteId::ENEMY_BOMBER)
        );
        assert_eq!(
            second.object_evidence.details[2].object_category,
            Some(ObjectEvidenceCategory::Pod)
        );
        assert_eq!(
            second.object_evidence.details[2].mapped_sprite,
            Some(SpriteId::ENEMY_POD)
        );
    }

    #[test]
    fn clean_enemy_families_use_source_message_scores_and_sprites() {
        for (kind, sprite, size, score) in [
            (EnemyKind::Lander, SpriteId::ENEMY_LANDER, (12, 8), 150),
            (EnemyKind::Mutant, SpriteId::ENEMY_MUTANT, (12, 8), 150),
            (EnemyKind::Bomber, SpriteId::ENEMY_BOMBER, (10, 8), 250),
            (EnemyKind::Pod, SpriteId::ENEMY_POD, (10, 8), 1000),
            (EnemyKind::Swarmer, SpriteId::ENEMY_SWARMER, (8, 6), 150),
            (EnemyKind::Baiter, SpriteId::ENEMY_BAITER, (12, 8), 200),
        ] {
            assert_eq!(super::enemy_sprite(kind), sprite);
            assert_eq!(super::enemy_sprite_size(kind), size);
            assert_eq!(super::enemy_score(kind), score);
        }
    }

    #[test]
    fn clean_world_maps_active_enemy_source_picture_descriptors() {
        let mut world = WorldSnapshot {
            enemies: vec![
                EnemySnapshot::source_lander(
                    ScreenPosition::new(10, 50),
                    ScreenVelocity::new(0, 0),
                    SourceLanderSnapshot {
                        x_fraction: 0,
                        y_fraction: 0,
                        x_velocity: 0,
                        y_velocity: 0,
                        shot_timer: 0,
                        sleep_ticks: 0,
                        picture_frame: 2,
                        target_human_index: None,
                    },
                ),
                EnemySnapshot::source_baiter(
                    ScreenPosition::new(20, 60),
                    ScreenVelocity::new(0, 0),
                    SourceBaiterSnapshot {
                        x_fraction: 0,
                        y_fraction: 0,
                        x_velocity: 0,
                        y_velocity: 0,
                        shot_timer: 0,
                        sleep_ticks: 0,
                        picture_frame: 1,
                    },
                ),
                EnemySnapshot::source_bomber(
                    ScreenPosition::new(30, 70),
                    ScreenVelocity::new(0, 0),
                    SourceBomberSnapshot {
                        x_fraction: 0,
                        y_fraction: 0,
                        x_velocity: 0,
                        y_velocity: 0,
                        picture_frame: 3,
                        cruise_altitude: 0,
                        sleep_ticks: 0,
                        source_slot: 0,
                    },
                ),
                EnemySnapshot::source_mutant(
                    ScreenPosition::new(40, 80),
                    ScreenVelocity::new(0, 0),
                    SourceMutantSnapshot {
                        x_fraction: 0,
                        y_fraction: 0,
                        x_velocity: 0,
                        y_velocity: 0,
                        shot_timer: 0,
                        sleep_ticks: 0,
                    },
                ),
                EnemySnapshot::source_pod(
                    ScreenPosition::new(50, 90),
                    ScreenVelocity::new(0, 0),
                    SourcePodSnapshot {
                        x_fraction: 0,
                        y_fraction: 0,
                        x_velocity: 0,
                        y_velocity: 0,
                    },
                ),
                EnemySnapshot::source_swarmer(
                    ScreenPosition::new(60, 100),
                    ScreenVelocity::new(0, 0),
                    SourceSwarmerSnapshot {
                        x_fraction: 0,
                        y_fraction: 0,
                        x_velocity: 0,
                        y_velocity: 0,
                        acceleration: 0,
                        shot_timer: 0,
                        sleep_ticks: 0,
                        horizontal_seek_pending: false,
                    },
                ),
            ],
            ..WorldSnapshot::default()
        };

        world.refresh_object_evidence();

        assert_eq!(world.object_evidence.detail_count, 6);
        let expected = [
            (
                ObjectEvidenceCategory::Lander,
                "LNDP3",
                0xF999,
                (5, 8),
                0xCD80,
                0xCDA8,
                SpriteId::ENEMY_LANDER,
            ),
            (
                ObjectEvidenceCategory::Baiter,
                "UFOP2",
                0xF9AD,
                (6, 4),
                0xCE00,
                0xCE18,
                SpriteId::ENEMY_BAITER,
            ),
            (
                ObjectEvidenceCategory::Bomber,
                "TIEP4",
                0xF947,
                (4, 8),
                0xFC0B,
                0xFC2B,
                SpriteId::ENEMY_BOMBER,
            ),
            (
                ObjectEvidenceCategory::Mutant,
                "SCZP1",
                0xF8CE,
                (5, 8),
                0xF9FB,
                0xFA23,
                SpriteId::ENEMY_MUTANT,
            ),
            (
                ObjectEvidenceCategory::Pod,
                "PRBP1",
                0xF8F7,
                (4, 8),
                0xFA8B,
                0xFAAB,
                SpriteId::ENEMY_POD,
            ),
            (
                ObjectEvidenceCategory::Swarmer,
                "SWPIC1",
                0xF97B,
                (3, 4),
                0xCCC8,
                0xCCD4,
                SpriteId::ENEMY_SWARMER,
            ),
        ];

        for (index, expected) in expected.into_iter().enumerate() {
            let detail = world.object_evidence.details[index];
            let identity = super::source_object_table_identity(index);
            assert_eq!(detail.object_category, Some(expected.0));
            assert_eq!(detail.address, Some(identity.address));
            assert_eq!(detail.slot, Some(identity.slot));
            assert_eq!(detail.object_type, Some(identity.object_type));
            assert_eq!(detail.picture_label, Some(expected.1));
            assert_eq!(detail.picture_address, Some(expected.2));
            assert_eq!(detail.picture_size, Some(expected.3));
            assert_eq!(detail.primary_image_address, Some(expected.4));
            assert_eq!(detail.alternate_image_address, Some(expected.5));
            assert_eq!(detail.mapped_sprite, Some(expected.6));
        }
    }

    #[test]
    fn clean_world_object_evidence_carries_reserve_inactive_source_details() {
        let mut world = WorldSnapshot {
            enemies: vec![EnemySnapshot::new(
                EnemyKind::Lander,
                ScreenPosition::new(0x40, 0x60),
                ScreenVelocity::new(0, 0),
            )],
            enemy_reserve: EnemyReserveSnapshot {
                landers: 1,
                bombers: 1,
                pods: 1,
                mutants: 1,
                swarmers: 1,
            },
            ..WorldSnapshot::default()
        };

        world.refresh_object_evidence();

        assert_eq!(world.object_evidence.active_count, 1);
        assert_eq!(world.object_evidence.inactive_count, 5);
        assert_eq!(world.object_evidence.detail_count, 6);
        assert_eq!(
            world.object_evidence.details[0].list,
            ObjectEvidenceList::Active
        );

        let expected = [
            (
                ObjectEvidenceCategory::Lander,
                "LNDP1",
                0xF985,
                (5, 8),
                0xCCE0,
                0xCD08,
                SpriteId::ENEMY_LANDER,
            ),
            (
                ObjectEvidenceCategory::Bomber,
                "TIEP1",
                0xF929,
                (4, 8),
                0xFB4B,
                0xFB6B,
                SpriteId::ENEMY_BOMBER,
            ),
            (
                ObjectEvidenceCategory::Pod,
                "PRBP1",
                0xF8F7,
                (4, 8),
                0xFA8B,
                0xFAAB,
                SpriteId::ENEMY_POD,
            ),
            (
                ObjectEvidenceCategory::Mutant,
                "SCZP1",
                0xF8CE,
                (5, 8),
                0xF9FB,
                0xFA23,
                SpriteId::ENEMY_MUTANT,
            ),
            (
                ObjectEvidenceCategory::Swarmer,
                "SWPIC1",
                0xF97B,
                (3, 4),
                0xCCC8,
                0xCCD4,
                SpriteId::ENEMY_SWARMER,
            ),
        ];

        for (offset, expected) in expected.into_iter().enumerate() {
            let index = offset + 1;
            let detail = world.object_evidence.details[index];
            let identity = super::source_object_table_identity(index);
            assert_eq!(detail.list, ObjectEvidenceList::Inactive);
            assert_eq!(detail.object_category, Some(expected.0));
            assert_eq!(detail.address, Some(identity.address));
            assert_eq!(detail.slot, Some(identity.slot));
            assert_eq!(detail.object_type, Some(identity.object_type));
            assert_eq!(detail.screen_position, None);
            assert_eq!(detail.world_position, None);
            assert_eq!(detail.velocity, None);
            assert_eq!(detail.picture_label, Some(expected.1));
            assert_eq!(detail.picture_address, Some(expected.2));
            assert_eq!(detail.picture_size, Some(expected.3));
            assert_eq!(detail.primary_image_address, Some(expected.4));
            assert_eq!(detail.alternate_image_address, Some(expected.5));
            assert_eq!(detail.mapped_sprite, Some(expected.6));
            assert_eq!(
                detail.scanner_color,
                super::scanner_color_for_object_category(expected.0)
            );
        }
    }

    #[test]
    fn game_input_none_is_empty() {
        assert_eq!(GameInput::NONE, GameInput::default());
    }

    #[test]
    fn game_events_keep_gameplay_and_sound_surfaces_separate() {
        let events = GameEvents::new(vec![GameEvent::CreditAdded], vec![SoundEvent::CreditAdded]);

        assert_eq!(events.gameplay(), &[GameEvent::CreditAdded]);
        assert_eq!(events.sounds(), &[SoundEvent::CreditAdded]);
        assert!(!events.is_empty());
        assert!(GameEvents::default().is_empty());
    }

    #[test]
    fn game_frame_uses_state_events_and_scene_contracts() {
        let frame = GameFrame {
            state: super::GameState {
                frame: 9,
                phase: GamePhase::Attract,
                credits: 1,
                current_player: 1,
                player_count: 1,
                wave: 0,
                wave_profile: WaveProfileSnapshot::for_wave(0),
                player: super::PlayerSnapshot {
                    position: (WorldVector::default(), WorldVector::default()),
                    velocity: (WorldVector::default(), WorldVector::default()),
                    direction: super::Direction::Right,
                    lives: 3,
                    smart_bombs: 3,
                },
                player_stocks: [super::PlayerStockSnapshot::new(3, 3); 2],
                scores: super::ScoreSnapshot {
                    player_one: 0,
                    player_two: 0,
                    high_score: 100,
                    next_bonus: 10_000,
                },
                attract: AttractPresentationSnapshot::for_page_frame(9),
                high_score_initials: HighScoreInitialsState::EMPTY,
                high_score_entry: None,
                high_score_submission: None,
                high_score_tables: HighScoreTablesSnapshot::DEFAULT,
                game_over: GameOverSnapshot::NONE,
                world: WorldSnapshot::default(),
            },
            events: GameEvents::default(),
            scene: RenderScene::empty(9, SurfaceSize::new(292, 240)),
        };

        assert_eq!(frame.state.frame, frame.scene.frame);
        assert!(frame.events.is_empty());
    }

    #[test]
    fn clean_game_starts_from_domain_state() {
        let game = Game::new();
        let state = game.state();

        assert_eq!(state.frame, 0);
        assert_eq!(state.phase, GamePhase::Attract);
        assert_eq!(state.credits, 0);
        assert_eq!(state.current_player, 1);
        assert_eq!(state.player_count, 1);
        assert_eq!(state.wave, DEFAULT_CABINET_WAVE);
        assert_eq!(state.wave_profile, WaveProfileSnapshot::for_wave(1));
        assert_eq!(state.player.direction, Direction::Right);
        assert_eq!(state.player.lives, DEFAULT_PLAYER_LIVES);
        assert_eq!(state.player.smart_bombs, 0);
        assert_eq!(
            state.player_stocks,
            [PlayerStockSnapshot::new(DEFAULT_PLAYER_LIVES, 0); 2]
        );
        assert_eq!(state.scores.player_one, 0);
        assert_eq!(state.scores.player_two, 0);
        assert_eq!(state.scores.high_score, DEFAULT_HIGH_SCORE);
        assert_eq!(state.scores.next_bonus, DEFAULT_REPLAY_SCORE);
        assert_eq!(
            state.attract,
            AttractPresentationSnapshot::for_page_frame(0)
        );
        assert_eq!(state.high_score_tables, HighScoreTablesSnapshot::DEFAULT);
        assert_eq!(state.game_over, GameOverSnapshot::NONE);
        assert_eq!(
            state.high_score_tables.all_time[0],
            HighScoreTableEntrySnapshot {
                rank: 1,
                score: DEFAULT_HIGH_SCORE,
                initials: [Some('D'), Some('R'), Some('J')],
            }
        );
        assert_eq!(state.world, WorldSnapshot::default());
        assert_eq!(Game::default().state(), state);
    }

    #[test]
    fn clean_attract_presentation_tracks_source_wait_gates() {
        let mut game = Game::new();

        let first = game.step(GameInput::NONE);
        assert_eq!(first.state.attract.page_frame, 1);
        assert_eq!(
            first.state.attract.page,
            AttractPresentationPage::WilliamsLogo
        );
        assert_eq!(first.state.attract.source_sleep_ticks, Some(2));
        assert_eq!(first.state.attract.source_stall_ticks, None);
        assert!(first.state.attract.shows_williams_logo());
        assert!(!first.state.attract.shows_presents_text());

        let presents = AttractPresentationSnapshot::for_page_frame(236);
        assert_eq!(presents.page, AttractPresentationPage::Presents);
        assert!(presents.shows_williams_logo());
        assert!(presents.shows_presents_text());
        assert!(!presents.shows_defender_wordmark());

        let defender =
            AttractPresentationSnapshot::for_page_frame(ATTRACT_DEFENDER_WORDMARK_START_FRAME);
        assert_eq!(defender.page, AttractPresentationPage::DefenderWordmark);
        assert!(defender.shows_defender_wordmark());
        assert!(!defender.shows_copyright());

        let title_hold =
            AttractPresentationSnapshot::for_page_frame(ATTRACT_HALL_OF_FAME_START_FRAME - 1);
        assert_eq!(title_hold.page, AttractPresentationPage::DefenderWordmark);
        assert!(title_hold.shows_defender_wordmark());
        assert!(!title_hold.shows_copyright());

        let hall_of_fame = AttractPresentationSnapshot::for_page_frame(488);
        assert_eq!(hall_of_fame.page, AttractPresentationPage::HallOfFame);
        assert_eq!(hall_of_fame.source_sleep_ticks, None);
        assert_eq!(
            hall_of_fame.source_stall_ticks,
            Some(HALL_OF_FAME_STALL_FRAMES)
        );
        assert!(hall_of_fame.shows_hall_of_fame());
        assert!(!hall_of_fame.shows_williams_logo());
        assert!(!hall_of_fame.shows_instruction_text());

        let scoring = AttractPresentationSnapshot::for_page_frame(
            super::ATTRACT_SCORING_SEQUENCE_START_FRAME,
        );
        assert_eq!(scoring.page, AttractPresentationPage::ScoringSequence);
        assert_eq!(scoring.scoring_sequence_frame(), Some(0));
        assert!(scoring.shows_scoring_sequence_text());
        assert!(scoring.shows_instruction_text());
        assert!(!scoring.shows_hall_of_fame());

        let wrapped = AttractPresentationSnapshot::for_page_frame(super::ATTRACT_CYCLE_FRAME_COUNT);
        assert_eq!(wrapped.page_frame, 0);
        assert_eq!(wrapped.page, AttractPresentationPage::WilliamsLogo);
    }

    #[test]
    fn clean_attract_scene_gates_title_program_surfaces() {
        let mut game = Game::new();

        game.state.attract = AttractPresentationSnapshot::for_page_frame(236);
        let presents_scene = game.scene();
        assert!(presents_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E && sprite.position == [100.0, 88.0]
        }));
        assert!(
            !presents_scene
                .sprites
                .iter()
                .any(|sprite| sprite.layer == RenderLayer::Terrain)
        );
        assert!(!presents_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position == [96.0, 144.0]
        }));

        game.state.attract = AttractPresentationSnapshot::for_page_frame(
            ATTRACT_DEFENDER_WORDMARK_START_FRAME + ATTRACT_DEFENDER_WORDMARK_REVEAL_FRAMES,
        );
        let settled_title_scene = game.scene();
        assert!(settled_title_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position == [96.0, 144.0]
        }));
        assert!(
            !settled_title_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ATTRACT_COPYRIGHT_STRIP)
        );

        game.state.attract = AttractPresentationSnapshot::for_page_frame(488);
        let hall_scene = game.scene();
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO && sprite.position == [85.0, 50.0]
        }));
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_H && sprite.position == [101.0, 78.0]
        }));
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_C
                && sprite.position == [69.0, 223.0]
                && sprite.tint == SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint()
        }));
        assert!(!hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO
                || sprite.sprite == SpriteId::ATTRACT_COPYRIGHT_STRIP
        }));

        game.state.attract = AttractPresentationSnapshot::for_page_frame(
            super::ATTRACT_SCORING_SEQUENCE_START_FRAME,
        );
        let scoring_scene = game.scene();
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_S && sprite.position == [123.0, 41.0]
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD
                && sprite.tint == super::ATTRACT_SCORING_SCANNER_BORDER_TINT
        }));
        assert!(
            scoring_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::PLAYER_SHIP)
        );
        assert!(
            scoring_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD)
        );
        assert!(!scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO
                || sprite.sprite == SpriteId::ATTRACT_COPYRIGHT_STRIP
        }));
    }

    #[test]
    fn source_bgout_default_terrain_records_match_red_label_head() {
        let records = super::source_bgout_default_terrain_records();

        assert_eq!(records.len(), super::SOURCE_TERRAIN_SCREEN_WORDS);
        assert_eq!(
            &records[..8],
            &[
                super::SourceTerrainDrawRecord {
                    screen_address: 0x98DE,
                    word: 0x7007,
                },
                super::SourceTerrainDrawRecord {
                    screen_address: 0x97DE,
                    word: 0x7007,
                },
                super::SourceTerrainDrawRecord {
                    screen_address: 0x96DE,
                    word: 0x7007,
                },
                super::SourceTerrainDrawRecord {
                    screen_address: 0x95DC,
                    word: 0x7007,
                },
                super::SourceTerrainDrawRecord {
                    screen_address: 0x94DA,
                    word: 0x7007,
                },
                super::SourceTerrainDrawRecord {
                    screen_address: 0x93D8,
                    word: 0x7007,
                },
                super::SourceTerrainDrawRecord {
                    screen_address: 0x92D7,
                    word: 0x0770,
                },
                super::SourceTerrainDrawRecord {
                    screen_address: 0x91D9,
                    word: 0x0770,
                },
            ]
        );
        assert_eq!(
            &super::source_tdata_bytes()[..16],
            &[
                0x2A, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAB, 0xA1, 0xD5, 0x55, 0x55, 0x55, 0x55, 0x55,
                0xAA, 0xBF,
            ]
        );
    }

    #[test]
    fn clean_attract_scoring_sequence_projects_source_rescue_demo() {
        let mut game = Game::new();
        let rescue_score_display_tick = super::attract_scoring_demo_tick_for_stage(
            super::AttractScoringDemoStage::RescueScore,
            0,
        );
        let rescue_score_tick =
            super::attract_scoring_page_tick_for_display_tick(rescue_score_display_tick);
        game.state.attract = AttractPresentationSnapshot::for_page_frame(
            super::ATTRACT_SCORING_SEQUENCE_START_FRAME + rescue_score_tick,
        );

        let scene = game.scene();
        let score_position = super::attract_scoring_object_position(
            super::ATTRACT_SCORING_SCORE_500_X16,
            super::ATTRACT_SCORING_SCORE_500_Y16,
        );

        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_POPUP_500
                && sprite.layer == RenderLayer::Objects
                && sprite.position == score_position
        }));
        assert!(
            scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::PLAYER_SHIP)
        );
        assert!(
            scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::HUMAN)
        );
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TERRAIN_TILE
                && sprite.layer == RenderLayer::Terrain
                && sprite.tint != Color::from_rgba(0x26, 0xAE, 0x00, 0xFF)
        }));
        assert!(
            scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::SCANNER_PLAYER_BLIP)
        );
        assert!(
            scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::SCANNER_OBJECT_BLIP)
        );
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL
                && sprite.layer == RenderLayer::Hud
                && sprite.position[1] < 40.0
                && sprite.tint == super::ATTRACT_SCORING_SCANNER_TERRAIN_TINT
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_S && sprite.position == [123.0, 41.0]
        }));
        assert!(!scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_L && sprite.position == [45.0, 105.0]
        }));
    }

    #[test]
    fn clean_attract_scoring_sequence_uses_protected_reference_cadence() {
        assert_eq!(
            super::attract_scoring_display_tick(0),
            super::ATTRACT_SCORING_PROTECTED_DEMO_TICK_OFFSET
        );
        assert_eq!(
            super::attract_scoring_demo_stage_for_tick(super::attract_scoring_display_tick(0)),
            (super::AttractScoringDemoStage::RescueFall, 5)
        );
        assert_eq!(
            super::attract_scoring_page_tick_for_display_tick(super::attract_scoring_display_tick(
                0
            )),
            0
        );
        assert_eq!(
            super::attract_scoring_color_cycle_tint(0),
            super::source_pseudo_color_tint(super::SOURCE_COLTAB_COLOR_BYTES[29])
        );
        assert_eq!(
            super::attract_scoring_color_cycle_tint(
                super::ATTRACT_SCORING_REFERENCE_SAMPLE_STEP_FRAMES,
            ),
            super::source_pseudo_color_tint(super::SOURCE_COLTAB_COLOR_BYTES[30])
        );
    }

    #[test]
    fn clean_attract_scoring_sequence_reveals_source_score_card() {
        let first_reveal_tick = super::attract_scoring_demo_tick_for_stage(
            super::AttractScoringDemoStage::LegendReveal(0),
            0,
        );
        let first_text_tick = super::next_attract_scoring_text_process_tick(first_reveal_tick);
        assert_eq!(
            super::attract_scoring_visible_legend_text_entries(first_text_tick),
            1
        );

        let hold_display_tick = super::attract_scoring_demo_tick_for_stage(
            super::AttractScoringDemoStage::LegendHold,
            0,
        );
        assert_eq!(
            super::attract_scoring_visible_legend_text_entries(hold_display_tick),
            super::ATTRACT_SCORING_LEGEND.len()
        );

        let mut game = Game::new();
        let hold_tick = super::attract_scoring_page_tick_for_display_tick(hold_display_tick);
        game.state.attract = AttractPresentationSnapshot::for_page_frame(
            super::ATTRACT_SCORING_SEQUENCE_START_FRAME + hold_tick,
        );
        let scene = game.scene();

        for sprite in [
            SpriteId::ENEMY_LANDER,
            SpriteId::ENEMY_MUTANT,
            SpriteId::ENEMY_BAITER,
            SpriteId::ENEMY_BOMBER,
            SpriteId::ENEMY_POD,
            SpriteId::ENEMY_SWARMER,
        ] {
            assert!(
                scene
                    .sprites
                    .iter()
                    .any(|scene_sprite| scene_sprite.sprite == sprite),
                "score-card scene should contain {sprite:?}"
            );
        }
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_S && sprite.position == [173.0, 161.0]
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1 && sprite.position[1] >= 161.0
        }));
    }

    #[test]
    fn clean_attract_defender_wordmark_coalesces_before_settled_logo() {
        let mut game = Game::new();

        game.state.attract =
            AttractPresentationSnapshot::for_page_frame(ATTRACT_DEFENDER_WORDMARK_START_FRAME);
        let early_scene = game.scene();
        let early_pixels = early_scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL)
            .count();
        assert!(early_pixels > 0);
        assert!(early_scene.raster().is_none());
        assert!(
            !early_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO)
        );
        assert!(!early_scene.sprites.iter().any(|sprite| {
            (SpriteId::ATTRACT_DEFENDER_WORDMARK_BLOCK_BASE.0
                ..SpriteId::ATTRACT_DEFENDER_WORDMARK_BLOCK_BASE.0
                    + ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT as u16)
                .contains(&sprite.sprite.0)
        }));

        game.state.attract = AttractPresentationSnapshot::for_page_frame(
            ATTRACT_DEFENDER_WORDMARK_START_FRAME + ATTRACT_DEFENDER_WORDMARK_REVEAL_FRAMES / 2,
        );
        let mid_scene = game.scene();
        let mid_pixels = mid_scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL)
            .count();
        assert!(mid_pixels > 0);
        assert!(mid_scene.raster().is_none());
        assert!(
            !mid_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO)
        );

        game.state.attract = AttractPresentationSnapshot::for_page_frame(
            ATTRACT_DEFENDER_WORDMARK_START_FRAME + ATTRACT_DEFENDER_WORDMARK_REVEAL_FRAMES,
        );
        let settled_scene = game.scene();
        assert!(settled_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position == [96.0, 144.0]
                && sprite.size == [120.0, 24.0]
                && sprite.tint == SOURCE_VISUAL_STATE.attract_defender_wordmark_tint()
        }));
        assert!(!settled_scene.sprites.iter().any(|sprite| {
            (SpriteId::ATTRACT_DEFENDER_WORDMARK_BLOCK_BASE.0
                ..SpriteId::ATTRACT_DEFENDER_WORDMARK_BLOCK_BASE.0
                    + ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT as u16)
                .contains(&sprite.sprite.0)
        }));
    }

    #[test]
    fn clean_attract_defender_wordmark_uses_source_descriptors() {
        let slices = super::source_attract_defender_wordmark_slices();
        let first = slices[0];
        let last = slices[ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT - 1];

        assert_eq!(ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT, 15);
        assert_eq!(first.object_address, super::SOURCE_ATTRACT_DEFENDER_OBJECTS);
        assert_eq!(
            first.picture_descriptor_address,
            super::SOURCE_ATTRACT_DEFENDER_PICTURES
        );
        assert_eq!(
            first.picture_data_address,
            super::SOURCE_ATTRACT_DEFENDER_DATA
        );
        assert_eq!(first.x16, super::SOURCE_ATTRACT_DEFENDER_INITIAL_X16);
        assert_eq!(first.y16, super::SOURCE_ATTRACT_DEFENDER_Y16);
        assert_eq!(
            first.appearance_slot_address,
            super::SOURCE_ATTRACT_DEFENDER_APPEARANCE_SLOT
        );

        assert_eq!(last.object_address, 0xB3C8);
        assert_eq!(last.picture_descriptor_address, 0xB40E);
        assert_eq!(last.picture_data_address, 0xB952);
        assert_eq!(last.x16, 0x1A00);
        assert_eq!(last.appearance_slot_address, 0x9F80);
        assert_eq!(
            super::source_attract_defender_whole_descriptor(),
            (0xB300, 0x3C, 0x18, 0xB412)
        );
    }

    #[test]
    fn clean_attract_williams_logo_uses_handwritten_reveal_and_source_color() {
        let mut game = Game::new();
        let logo_pixel_count = source_attract_williams_logo_pixel_path().len();

        game.state.attract = AttractPresentationSnapshot::for_page_frame(0);
        let blank_scene = game.scene();
        assert!(
            !blank_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL)
        );

        game.state.attract = AttractPresentationSnapshot::for_page_frame(16);
        let early_scene = game.scene();
        let early_pixels = early_scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL)
            .collect::<Vec<_>>();
        assert!(!early_pixels.is_empty());
        assert!(early_pixels.len() < logo_pixel_count);
        assert!(
            !early_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO)
        );

        game.state.attract = AttractPresentationSnapshot::for_page_frame(24);
        let next_tint_scene = game.scene();
        let next_tint = next_tint_scene
            .sprites
            .iter()
            .find(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL)
            .expect("handwritten Williams pixel")
            .tint;
        assert_eq!(early_pixels[0].tint, next_tint);
        assert_eq!(next_tint, super::source_pseudo_color_tint(0x2F));

        game.state.attract = AttractPresentationSnapshot::for_page_frame(160);
        let source_first_pass_scene = game.scene();
        assert!(
            source_first_pass_scene
                .sprites
                .iter()
                .any(|sprite| { sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL })
        );
        assert!(
            !source_first_pass_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO)
        );

        game.state.attract =
            AttractPresentationSnapshot::for_page_frame(ATTRACT_WILLIAMS_LOGO_REVEAL_FRAMES);
        let complete_scene = game.scene();
        assert!(complete_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO
                && sprite.position == [108.0, 60.0]
                && sprite.tint
                    == SOURCE_VISUAL_STATE
                        .attract_williams_logo_tint_for_frame(ATTRACT_WILLIAMS_LOGO_REVEAL_FRAMES)
        }));
        assert!(
            !complete_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL)
        );
    }

    #[test]
    fn source_visual_state_tracks_palette_words_rates_and_blink_contract() {
        let visual = SOURCE_VISUAL_STATE;

        assert_eq!(visual.attract_williams_status, 0xFB);
        assert_eq!(visual.attract_williams_logo_color_index, 0x3F);
        assert_eq!(visual.attract_copyright_williams_color_index, 0x0F);
        assert_eq!(visual.attract_williams_fast_logo_rate, 0xFF);
        assert_eq!(visual.attract_williams_normal_logo_rate, 10);
        assert_eq!(visual.attract_instruction_man_color_word, 0x6666);
        assert_eq!(visual.attract_instruction_ship_color_word, 0x0000);
        assert_eq!(visual.attract_instruction_enemy_color_word, 0x4433);
        assert_eq!(visual.hall_of_fame_display_letter_color_index, 0x47);
        assert_eq!(visual.hall_of_fame_logo_color_index, 0x3F);
        assert_eq!(visual.hall_of_fame_entry_letter_color_index, 0x85);
        assert_eq!(visual.hall_of_fame_blink_color_index, 0x85);
        assert_eq!(visual.hall_of_fame_blink_sleep_ticks, 15);
        assert_eq!(visual.hall_of_fame_underline_normal_word, 0x1111);
        assert_eq!(visual.hall_of_fame_underline_active_word, 0xDDDD);
        assert_eq!(visual.top_display_border_word, 0x5555);
        assert_eq!(visual.top_display_scanner_marker_word, 0x9999);

        assert_eq!(visual.hud_tint(), Color::WHITE);
        assert_eq!(visual.top_display_border_tint(), Color::WHITE);
        assert_eq!(visual.top_display_scanner_marker_tint(), Color::WHITE);
        assert_eq!(
            visual.attract_williams_logo_tint_for_frame(0),
            super::source_pseudo_color_tint(0x2F)
        );
        assert_eq!(
            visual.attract_williams_logo_tint_for_frame(10),
            super::source_pseudo_color_tint(0x2F)
        );
        assert_eq!(
            visual.attract_williams_logo_tint_for_frame(20),
            super::source_pseudo_color_tint(0x2F)
        );
        assert_eq!(
            visual.attract_title_text_tint_for_frame(236),
            super::source_pseudo_color_tint(0x1F)
        );
        assert_eq!(
            visual.attract_title_text_tint_for_frame(328),
            super::source_pseudo_color_tint(0xF8)
        );
        assert_eq!(visual.attract_defender_wordmark_tint(), Color::WHITE);
        assert_eq!(visual.attract_copyright_tint(), Color::WHITE);
        assert_eq!(visual.hall_of_fame_logo_tint(), Color::WHITE);
        assert_eq!(
            visual.hall_of_fame_entry_text_tint(),
            super::source_pseudo_color_tint(0x85)
        );
        assert_eq!(
            visual.hall_of_fame_display_text_tint(),
            super::source_pseudo_color_tint(0x47)
        );
        assert_eq!(
            visual.hall_of_fame_blink_text_tint(),
            super::source_pseudo_color_tint(0x85)
        );
        assert_eq!(visual.hall_of_fame_active_underline_tint(), Color::WHITE);
        assert_eq!(
            visual.hall_of_fame_normal_underline_tint(),
            Color::from_rgba(0x66, 0x66, 0x66, 0xFF)
        );
        assert!(visual.attract_williams_logo_should_render());
    }

    #[test]
    fn clean_scene_uses_source_visual_state_tints() {
        let mut game = Game::new();

        game.state.phase = GamePhase::Playing;
        game.state.scores.player_one = 1_200;
        let playing_scene = game.scene();
        assert!(
            playing_scene
                .sprites
                .iter()
                .filter(|sprite| sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD)
                .all(|sprite| sprite.tint == SOURCE_VISUAL_STATE.top_display_border_tint())
        );
        assert!(
            playing_scene
                .sprites
                .iter()
                .filter(|sprite| {
                    sprite.layer == RenderLayer::Hud
                        && SpriteId::SCORE_DIGITS.contains(&sprite.sprite)
                })
                .all(|sprite| sprite.tint == SOURCE_VISUAL_STATE.hud_tint())
        );

        game.state.phase = GamePhase::Attract;
        game.state.attract = AttractPresentationSnapshot::for_page_frame(
            ATTRACT_DEFENDER_WORDMARK_START_FRAME + ATTRACT_DEFENDER_WORDMARK_REVEAL_FRAMES,
        );
        let attract_scene = game.scene();
        assert!(attract_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO
                && sprite.tint
                    == SOURCE_VISUAL_STATE
                        .attract_williams_logo_tint_for_frame(game.state.attract.page_frame)
        }));
        assert!(attract_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position == [96.0, 144.0]
                && sprite.tint == SOURCE_VISUAL_STATE.attract_defender_wordmark_tint()
        }));
        assert!(
            !attract_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ATTRACT_COPYRIGHT_STRIP)
        );

        game.state.phase = GamePhase::HighScoreEntry;
        game.state.attract = AttractPresentationSnapshot::INACTIVE;
        game.state.high_score_initials = HighScoreInitialsState {
            initials: [Some('A'), Some('B'), Some('C')],
            cursor: 1,
        };
        let entry_scene = game.scene();
        let underlines = entry_scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::HALL_OF_FAME_UNDERLINE_WORD)
            .collect::<Vec<_>>();
        assert!(underlines.iter().any(|sprite| {
            sprite.tint == SOURCE_VISUAL_STATE.hall_of_fame_active_underline_tint()
        }));
        assert!(underlines.iter().any(|sprite| {
            sprite.tint == SOURCE_VISUAL_STATE.hall_of_fame_normal_underline_tint()
        }));
        assert!(entry_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_B
                && sprite.tint == SOURCE_VISUAL_STATE.hall_of_fame_blink_text_tint()
        }));
    }

    #[test]
    fn clean_game_credits_starts_at_blank_reference_title_frame() {
        let mut game = Game::new();

        let inserted = game.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(inserted.state.frame, 1);
        assert_eq!(inserted.state.credits, 0);
        assert!(inserted.events.is_empty());
        assert_eq!(inserted.scene.summary().layers.hud, 0);
        assert_eq!(inserted.scene.summary().layers.overlay, 0);
        assert_eq!(inserted.scene.summary().raster_count, 0);
        assert!(
            !inserted
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL)
        );
        assert!(
            !inserted
                .scene
                .sprites
                .iter()
                .any(|sprite| { sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO })
        );
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position == [96.0, 144.0]
        }));
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_COPYRIGHT_STRIP && sprite.position == [118.0, 208.0]
        }));
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_C
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [80.0, 229.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_0
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [144.0, 229.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E && sprite.position == [100.0, 88.0]
        }));
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P && sprite.position == [124.0, 108.0]
        }));
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_S && sprite.position == [134.0, 48.0]
        }));
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_L && sprite.position == [56.0, 112.0]
        }));
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1 && sprite.position == [68.0, 122.0]
        }));
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P && sprite.position == [132.0, 168.0]
        }));
        assert!(!inserted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1 && sprite.position == [128.0, 178.0]
        }));

        let credited = advance_to_delayed_credit(&mut game);
        assert_eq!(credited.state.credits, 1);
        assert_eq!(credited.events.gameplay(), &[GameEvent::CreditAdded]);
        assert!(credited.events.sounds().is_empty());
        assert_eq!(credited.scene.summary().layers.hud, 0);
        assert!(credited.scene.summary().layers.overlay > 0);
        assert_eq!(credited.scene.summary().raster_count, 0);
        assert!(credited.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [144.0, 229.0]
                && sprite.size == [6.0, 8.0]
        }));

        let credit_sound = advance_to_delayed_credit_sound(&mut game);
        assert!(credit_sound.events.gameplay().is_empty());
        assert_eq!(credit_sound.events.sounds(), &[SoundEvent::CreditAdded]);

        let started = game.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.state.phase, GamePhase::Playing);
        assert_eq!(started.state.credits, 0);
        assert_eq!(started.state.wave, 1);
        assert_eq!(started.state.player.lives, 3);
        assert_eq!(started.state.player.smart_bombs, 3);
        assert_eq!(
            started.state.player.position,
            (WorldVector::default(), WorldVector::default())
        );
        assert_eq!(started.state.world, WorldSnapshot::default());
        assert_eq!(started.events.gameplay(), &[GameEvent::GameStarted]);
        assert!(started.events.sounds().is_empty());
        assert_eq!(
            started.scene.summary().layers,
            RenderLayerCounts {
                terrain: super::SOURCE_TERRAIN_SCREEN_WORDS,
                objects: 1,
                hud: 14,
                ..RenderLayerCounts::default()
            }
        );
        assert_eq!(
            started.scene.summary().sprite_count,
            15 + super::SOURCE_TERRAIN_SCREEN_WORDS
        );

        let start_sound = advance_to_delayed_start_sound(&mut game);
        assert!(start_sound.events.gameplay().is_empty());
        assert_eq!(start_sound.events.sounds(), &[SoundEvent::GameStarted]);

        let active = advance_to_started_playfield(&mut game);
        assert_eq!(active.state.player.lives, 2);
        assert_eq!(active.state.player.smart_bombs, 3);
        assert_eq!(
            active.state.player.position,
            (super::world_word(0x2000), super::world_word(0x8000))
        );
        assert_eq!(active.state.wave_profile.landers, 15);
        assert_eq!(active.state.wave_profile.wave_size, 5);
        assert_eq!(active.state.world.terrain.len(), 5);
        assert_eq!(active.state.world.stars.len(), 3);
        assert_eq!(active.state.world.enemies.len(), 5);
        assert_eq!(active.state.world.enemies[0].kind, EnemyKind::Lander);
        assert_eq!(
            active.state.world.enemies[0].velocity,
            super::source_lander_screen_velocity(
                active.state.world.enemies[0]
                    .source_lander
                    .expect("initial lander should carry source state")
            )
        );
        assert_eq!(active.state.world.humans.len(), 10);
        assert!(active.state.world.projectiles.is_empty());
        assert_eq!(active.state.world.object_evidence.active_count, 15);
        assert_eq!(active.state.world.object_evidence.inactive_count, 10);
        assert_eq!(active.state.world.object_evidence.projectile_count, 0);
        assert_eq!(active.state.world.object_evidence.visible_count, 15);
        assert_eq!(active.state.world.object_evidence.evidence_crc32, None);
        assert_eq!(active.state.world.object_evidence.detail_count, 16);
        assert_eq!(
            active.state.world.object_evidence.details[0].screen_position,
            Some(active.state.world.enemies[0].position)
        );
        assert_eq!(
            active.state.world.object_evidence.details[0].object_category,
            Some(ObjectEvidenceCategory::Lander)
        );
        assert_eq!(
            active.state.world.object_evidence.details[0].mapped_sprite,
            Some(SpriteId::ENEMY_LANDER)
        );
        assert_eq!(
            active.scene.summary().layers,
            RenderLayerCounts {
                terrain: super::SOURCE_TERRAIN_SCREEN_WORDS,
                starfield: 3,
                objects: 16,
                hud: 30,
                ..RenderLayerCounts::default()
            }
        );
        assert_eq!(
            active.scene.summary().sprite_count,
            49 + super::SOURCE_TERRAIN_SCREEN_WORDS
        );
    }

    #[test]
    fn clean_game_two_player_start_requires_two_credits() {
        let mut game = Game::new();
        game.state.credits = 1;

        let blocked = game.step(GameInput {
            start_two: true,
            ..GameInput::NONE
        });

        assert_eq!(blocked.state.phase, GamePhase::Attract);
        assert_eq!(blocked.state.credits, 1);
        assert_eq!(blocked.state.player_count, 1);
        assert!(blocked.events.is_empty());
        assert_eq!(blocked.scene.summary().layers.hud, 2);
    }

    #[test]
    fn clean_game_two_player_start_initializes_top_display_state() {
        let mut game = Game::new();
        game.state.credits = 2;

        let started = game.step(GameInput {
            start_two: true,
            ..GameInput::NONE
        });

        assert_eq!(started.state.phase, GamePhase::Playing);
        assert_eq!(started.state.credits, 0);
        assert_eq!(started.state.current_player, 1);
        assert_eq!(started.state.player_count, 2);
        assert_eq!(started.events.gameplay(), &[GameEvent::GameStarted]);
        assert!(started.events.sounds().is_empty());
        assert_eq!(
            started.state.player_stocks,
            [PlayerStockSnapshot::new(3, 3); 2]
        );
        assert_eq!(
            started.scene.summary().layers,
            RenderLayerCounts {
                objects: 1,
                hud: 22,
                overlay: 9,
                ..RenderLayerCounts::default()
            }
        );
        assert_eq!(started.scene.summary().sprite_count, 32);
        assert!(started.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [120.0, 128.0]
                && sprite.size == [6.0, 8.0]
        }));

        let start_sound = advance_to_delayed_start_sound(&mut game);
        assert_eq!(start_sound.events.sounds(), &[SoundEvent::GameStarted]);

        let active = advance_to_started_playfield(&mut game);
        assert_eq!(active.state.player_count, 2);
        assert_eq!(active.state.current_player, 1);
        assert_eq!(
            active.state.player_stocks[0],
            PlayerStockSnapshot::new(2, 3)
        );
        assert_eq!(
            active.state.player_stocks[1],
            PlayerStockSnapshot::new(3, 3)
        );
        assert_eq!(
            active.scene.summary().layers,
            RenderLayerCounts {
                terrain: 5,
                starfield: 3,
                objects: 16,
                hud: 38,
                ..RenderLayerCounts::default()
            }
        );
        assert_eq!(active.scene.summary().sprite_count, 62);
    }

    #[test]
    fn clean_game_operator_controls_reset_high_score_and_debounce_events() {
        let mut game = Game::new();
        game.state.scores.player_one = 1_200;
        game.state.scores.high_score = 5_000;
        game.state.scores.next_bonus = 10_000;
        let operator_input = GameInput {
            service_auto_up: true,
            service_advance: true,
            high_score_reset: true,
            ..GameInput::NONE
        };

        let first = game.step(operator_input);

        assert_eq!(first.state.scores.player_one, 1_200);
        assert_eq!(first.state.scores.high_score, 0);
        assert_eq!(first.state.scores.next_bonus, 10_000);
        assert_eq!(
            first.events.gameplay(),
            &[
                GameEvent::DiagnosticsSelected,
                GameEvent::AuditsSelected,
                GameEvent::HighScoreReset,
            ]
        );
        assert!(first.events.sounds().is_empty());

        game.state.scores.high_score = 4_000;
        let held = game.step(operator_input);
        assert_eq!(held.state.scores.high_score, 4_000);
        assert!(held.events.is_empty());

        let released = game.step(GameInput::NONE);
        assert_eq!(released.state.scores.high_score, 4_000);
        assert!(released.events.is_empty());

        let reset_again = game.step(GameInput {
            high_score_reset: true,
            ..GameInput::NONE
        });
        assert_eq!(reset_again.state.scores.high_score, 0);
        assert_eq!(reset_again.events.gameplay(), &[GameEvent::HighScoreReset]);
    }

    #[test]
    fn clean_game_applies_playing_controls_through_systems() {
        let mut game = credited_started_game();

        let frame = game.step(GameInput {
            altitude_up: true,
            reverse: true,
            thrust: true,
            fire: true,
            smart_bomb: true,
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(frame.state.player.direction, Direction::Right);
        assert_eq!(frame.state.player.smart_bombs, 2);
        assert_eq!(frame.state.scores.player_one, 750);
        assert_eq!(
            frame.state.player.velocity,
            (WorldVector::default(), WorldVector::default())
        );
        assert_eq!(frame.state.world.projectiles.len(), 1);
        assert_eq!(frame.state.world.enemies.len(), 5);
        assert_eq!(
            frame.state.world.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(frame.state.world.object_evidence.active_count, 16);
        assert_eq!(frame.state.world.object_evidence.inactive_count, 5);
        assert_eq!(frame.state.world.object_evidence.projectile_count, 1);
        assert_eq!(frame.state.world.object_evidence.visible_count, 16);
        assert_eq!(frame.state.world.object_evidence.evidence_crc32, None);
        assert_eq!(frame.state.world.object_evidence.detail_count, 16);
        assert_eq!(
            frame.state.world.object_evidence.details[15].screen_position,
            Some(frame.state.world.projectiles[0].position)
        );
        assert_eq!(
            frame.state.world.object_evidence.details[15].object_category,
            Some(ObjectEvidenceCategory::PlayerProjectile)
        );
        assert_eq!(
            frame.state.world.object_evidence.details[15].mapped_sprite,
            Some(SpriteId::PLAYER_PROJECTILE)
        );
        assert_eq!(
            frame.state.world.projectiles[0].velocity,
            ScreenVelocity::new(-5, 0)
        );
        assert_eq!(
            frame.events.gameplay(),
            &[
                GameEvent::ReversePressed,
                GameEvent::FirePressed,
                GameEvent::SmartBombPressed,
                GameEvent::EnemyDestroyed,
                GameEvent::EnemyDestroyed,
                GameEvent::EnemyDestroyed,
                GameEvent::EnemyDestroyed,
                GameEvent::EnemyDestroyed,
                GameEvent::HyperspacePressed,
            ]
        );
        let mut expected_sounds = vec![
            source_laser_fire_sound_event(),
            source_smart_bomb_sound_event(),
        ];
        expected_sounds.extend(vec![source_enemy_hit_sound_event(EnemyKind::Lander); 5]);
        expected_sounds.push(source_hyperspace_appearance_sound_event());
        expected_sounds.push(SoundEvent::ThrustStarted);
        assert_eq!(frame.events.sounds(), expected_sounds.as_slice());
        assert_eq!(
            frame.scene.summary().layers,
            RenderLayerCounts {
                terrain: 5,
                starfield: 3,
                objects: 21,
                projectiles: 1,
                hud: 30,
                overlay: 0,
            }
        );
        assert_eq!(frame.scene.summary().raster_count, 0);
    }

    #[test]
    fn clean_game_draws_score_digits_with_arcade_blanking_and_positions() {
        let mut game = credited_started_game();
        game.state.scores.player_one = 1_050;

        let frame = game.step(GameInput::NONE);
        let score_digits = frame
            .scene
            .sprites
            .iter()
            .filter(|sprite| {
                sprite.layer == RenderLayer::Hud && SpriteId::SCORE_DIGITS.contains(&sprite.sprite)
            })
            .collect::<Vec<_>>();

        assert_eq!(score_digits.len(), 4);
        assert_eq!(score_digits[0].sprite, SpriteId::SCORE_DIGIT_1);
        assert_eq!(score_digits[0].position, [34.0, 21.0]);
        assert_eq!(score_digits[1].sprite, SpriteId::SCORE_DIGIT_0);
        assert_eq!(score_digits[1].position, [42.0, 21.0]);
        assert_eq!(score_digits[2].sprite, SpriteId::SCORE_DIGIT_5);
        assert_eq!(score_digits[2].position, [50.0, 21.0]);
        assert_eq!(score_digits[3].sprite, SpriteId::SCORE_DIGIT_0);
        assert_eq!(score_digits[3].position, [58.0, 21.0]);
        assert!(
            score_digits
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Hud && sprite.size == [6.0, 8.0])
        );
    }

    #[test]
    fn clean_game_draws_two_player_score_digits_at_arcade_positions() {
        let mut game = credited_started_game();
        game.state.player_count = 2;
        game.state.scores.player_two = 2_001;

        let frame = game.step(GameInput::NONE);
        let score_digits = frame
            .scene
            .sprites
            .iter()
            .filter(|sprite| {
                sprite.layer == RenderLayer::Hud && SpriteId::SCORE_DIGITS.contains(&sprite.sprite)
            })
            .collect::<Vec<_>>();

        assert_eq!(score_digits.len(), 6);
        assert_eq!(score_digits[0].sprite, SpriteId::SCORE_DIGIT_0);
        assert_eq!(score_digits[0].position, [50.0, 21.0]);
        assert_eq!(score_digits[1].sprite, SpriteId::SCORE_DIGIT_0);
        assert_eq!(score_digits[1].position, [58.0, 21.0]);
        assert_eq!(score_digits[2].sprite, SpriteId::SCORE_DIGIT_2);
        assert_eq!(score_digits[2].position, [230.0, 21.0]);
        assert_eq!(score_digits[3].sprite, SpriteId::SCORE_DIGIT_0);
        assert_eq!(score_digits[3].position, [238.0, 21.0]);
        assert_eq!(score_digits[4].sprite, SpriteId::SCORE_DIGIT_0);
        assert_eq!(score_digits[4].position, [246.0, 21.0]);
        assert_eq!(score_digits[5].sprite, SpriteId::SCORE_DIGIT_1);
        assert_eq!(score_digits[5].position, [254.0, 21.0]);
    }

    #[test]
    fn clean_game_draws_current_player_stock_counts_with_arcade_caps() {
        let mut game = credited_started_game();
        game.state.player.lives = 7;
        game.state.player.smart_bombs = 4;

        let frame = game.step(GameInput::NONE);
        let life_stock = frame
            .scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::PLAYER_LIFE_STOCK)
            .collect::<Vec<_>>();
        let smart_bomb_stock = frame
            .scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::SMART_BOMB_STOCK)
            .collect::<Vec<_>>();

        assert_eq!(life_stock.len(), 5);
        assert_eq!(smart_bomb_stock.len(), 3);
        assert_eq!(life_stock[0].position, [18.0, 13.0]);
        assert_eq!(life_stock[4].position, [66.0, 13.0]);
        assert_eq!(life_stock[0].size, [10.0, 4.0]);
        assert_eq!(smart_bomb_stock[0].position, [70.0, 20.0]);
        assert_eq!(smart_bomb_stock[2].position, [70.0, 28.0]);
        assert_eq!(smart_bomb_stock[0].size, [6.0, 3.0]);
        assert!(
            life_stock
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Hud)
        );
        assert!(
            smart_bomb_stock
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Hud)
        );
    }

    #[test]
    fn clean_game_draws_two_player_stock_counts_at_arcade_positions() {
        let mut game = credited_started_game();
        game.state.player_count = 2;
        game.state.player.lives = 7;
        game.state.player.smart_bombs = 4;
        game.state.player_stocks[1] = PlayerStockSnapshot::new(2, 2);

        let frame = game.step(GameInput::NONE);
        let life_stock = frame
            .scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::PLAYER_LIFE_STOCK)
            .collect::<Vec<_>>();
        let smart_bomb_stock = frame
            .scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::SMART_BOMB_STOCK)
            .collect::<Vec<_>>();

        assert_eq!(life_stock.len(), 7);
        assert_eq!(smart_bomb_stock.len(), 5);
        assert_eq!(life_stock[0].position, [18.0, 13.0]);
        assert_eq!(life_stock[4].position, [66.0, 13.0]);
        assert_eq!(life_stock[5].position, [214.0, 13.0]);
        assert_eq!(life_stock[6].position, [226.0, 13.0]);
        assert_eq!(smart_bomb_stock[0].position, [70.0, 20.0]);
        assert_eq!(smart_bomb_stock[2].position, [70.0, 28.0]);
        assert_eq!(smart_bomb_stock[3].position, [266.0, 20.0]);
        assert_eq!(smart_bomb_stock[4].position, [266.0, 24.0]);
        assert_eq!(frame.scene.summary().layers.hud, 39);
    }

    #[test]
    fn clean_game_smart_bomb_clears_enemies_scores_and_updates_scene() {
        let mut game = credited_started_game();
        game.state.scores.player_one = 9_800;
        game.state.scores.high_score = 9_800;
        game.state.scores.next_bonus = 10_000;
        game.state.player.smart_bombs = 1;

        let frame = game.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(frame.state.world.enemies.len(), 5);
        assert_eq!(
            frame.state.world.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(frame.state.scores.player_one, 10_550);
        assert_eq!(frame.state.scores.high_score, 10_550);
        assert_eq!(frame.state.scores.next_bonus, 20_000);
        assert_eq!(frame.state.player.lives, 3);
        assert_eq!(frame.state.player.smart_bombs, 1);
        assert_eq!(
            frame.events.gameplay(),
            &[
                GameEvent::SmartBombPressed,
                GameEvent::EnemyDestroyed,
                GameEvent::EnemyDestroyed,
                GameEvent::BonusAwarded,
                GameEvent::EnemyDestroyed,
                GameEvent::EnemyDestroyed,
                GameEvent::EnemyDestroyed,
            ]
        );
        let mut expected_sounds = vec![source_smart_bomb_sound_event()];
        expected_sounds.extend(vec![source_enemy_hit_sound_event(EnemyKind::Lander); 5]);
        assert_eq!(frame.events.sounds(), expected_sounds.as_slice());
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER
                && sprite.layer == RenderLayer::Objects
                && sprite.size == [12.0, 8.0]
        }));
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER
                && sprite.layer == RenderLayer::Objects
                && sprite.size == [5.0, 8.0]
        }));
    }

    #[test]
    fn clean_game_smart_bomb_pod_spawns_swarmers_after_destroyed_batch() {
        let mut game = credited_started_game();
        game.state.wave = 2;
        game.state.wave_profile = WaveProfileSnapshot::for_wave(2);
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Pod,
            ScreenPosition::new(100, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.player.smart_bombs = 1;

        let frame = game.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(
            frame.state.world.enemies.len(),
            SOURCE_POD_SWARMER_REQUEST_LIMIT
        );
        assert!(
            frame
                .state
                .world
                .enemies
                .iter()
                .all(|enemy| enemy.kind == EnemyKind::Swarmer)
        );
        assert_eq!(frame.state.scores.player_one, 1_000);
        assert_eq!(frame.state.player.smart_bombs, 0);
        assert_eq!(
            frame.events.gameplay(),
            &[GameEvent::SmartBombPressed, GameEvent::EnemyDestroyed]
        );
        assert_eq!(
            frame.events.sounds(),
            &[
                source_smart_bomb_sound_event(),
                source_enemy_hit_sound_event(EnemyKind::Pod),
            ]
        );
    }

    #[test]
    fn clean_game_activates_source_reserve_batch_before_wave_clear() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.wave, 1);
        assert_eq!(frame.state.world.enemies.len(), 5);
        assert_eq!(
            frame.state.world.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert!(frame.events.gameplay().is_empty());
        assert_eq!(frame.state.world.object_evidence.active_count, 15);
        assert_eq!(frame.state.world.object_evidence.inactive_count, 5);
        assert_eq!(frame.state.world.object_evidence.visible_count, 15);
    }

    #[test]
    fn clean_game_advances_projectiles_through_world_snapshots() {
        let mut game = credited_started_game();

        let fired = game.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let projectile = fired.state.world.projectiles[0];

        assert_eq!(projectile.velocity, ScreenVelocity::new(5, 0));
        assert_eq!(fired.events.sounds(), &[source_laser_fire_sound_event()]);
        assert!(fired.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.position
                    == [
                        f32::from(projectile.position.x),
                        f32::from(projectile.position.y),
                    ]
        }));

        let moved = game.step(GameInput::NONE);
        let moved_projectile = moved.state.world.projectiles[0];

        assert_eq!(
            moved_projectile.position.x,
            projectile.position.x.wrapping_add(5)
        );
        assert_eq!(moved_projectile.position.y, projectile.position.y);
        assert_eq!(moved_projectile.velocity, projectile.velocity);
    }

    #[test]
    fn clean_game_thrust_release_emits_source_stop_sound() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(200, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;

        let started = game.step(GameInput {
            thrust: true,
            ..GameInput::NONE
        });
        assert_eq!(started.events.sounds(), &[SoundEvent::ThrustStarted]);

        let held = game.step(GameInput {
            thrust: true,
            ..GameInput::NONE
        });
        assert!(held.events.sounds().is_empty());

        let stopped = game.step(GameInput::NONE);
        assert_eq!(stopped.events.sounds(), &[SoundEvent::ThrustStopped]);

        let idle = game.step(GameInput::NONE);
        assert!(idle.events.sounds().is_empty());
    }

    #[test]
    fn clean_game_capped_fire_does_not_emit_laser_sound() {
        let mut game = credited_started_game();
        game.state.world.projectiles = (0..4)
            .map(|index| ProjectileSnapshot {
                position: ScreenPosition::new(32 + index, 80),
                velocity: ScreenVelocity::new(0, 0),
            })
            .collect();

        let frame = game.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });

        assert_eq!(frame.state.world.projectiles.len(), 4);
        assert_eq!(frame.events.gameplay(), &[GameEvent::FirePressed]);
        assert!(frame.events.sounds().is_empty());
    }

    #[test]
    fn clean_game_culls_projectiles_that_leave_the_screen() {
        let mut game = credited_started_game();
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(0x98, 80),
            velocity: ScreenVelocity::new(5, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.projectiles.is_empty());
        assert!(
            !frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::PLAYER_PROJECTILE)
        );
    }

    #[test]
    fn clean_game_resolves_projectile_enemy_collision_and_scores() {
        let mut game = credited_started_game();
        keep_first_enemy_only(&mut game);
        game.state.world.enemies[0] = EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(100, 80),
            ScreenVelocity::new(0, 0),
        );
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.enemies.is_empty());
        assert!(frame.state.world.projectiles.is_empty());
        assert_eq!(frame.state.wave, 1);
        assert_eq!(frame.state.scores.player_one, 150);
        assert_eq!(
            frame.events.gameplay(),
            &[GameEvent::EnemyDestroyed, GameEvent::WaveCleared]
        );
        assert_eq!(
            frame.events.sounds(),
            &[source_enemy_hit_sound_event(EnemyKind::Lander)]
        );
        assert_eq!(frame.state.world.expanded_objects.active_count, 1);
        assert_eq!(frame.state.world.expanded_objects.detail_count, 1);
        assert_eq!(
            frame.state.world.expanded_objects.details[0],
            ExpandedObjectDetailSnapshot {
                kind: ExpandedObjectKind::Explosion,
                size: SOURCE_EXPLOSION_INITIAL_SIZE,
                picture_label: Some("LNDP1"),
                picture_size: Some((5, 8)),
                mapped_sprite: Some(SpriteId::ENEMY_LANDER),
                center: Some(ScreenPosition::new(102, 84)),
                top_left: Some(ScreenPosition::new(100, 80)),
                explosion_frame: Some(0),
                explosion_lifetime_frames: Some(SOURCE_EXPLOSION_LIFETIME_FRAMES),
                ..ExpandedObjectDetailSnapshot::EMPTY
            }
        );
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [100.0, 80.0]
                && sprite.size == [5.0, 8.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(
            !frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::PLAYER_PROJECTILE)
        );
    }

    #[test]
    fn clean_game_player_projectile_uses_source_lasp1_collision_height() {
        let mut game = credited_started_game();
        keep_first_enemy_only(&mut game);
        game.state.world.enemies[0] = EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(100, 89),
            ScreenVelocity::new(0, 0),
        );
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 88),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemies.len(), 1);
        assert_eq!(frame.state.world.projectiles.len(), 1);
        assert_eq!(frame.state.scores.player_one, 0);
        assert!(frame.events.gameplay().is_empty());
        assert!(frame.events.sounds().is_empty());
    }

    #[test]
    fn clean_game_projectile_enemy_uses_source_enemy_collision_width() {
        let mut game = credited_started_game();
        keep_first_enemy_only(&mut game);
        game.state.world.enemies[0] = EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(100, 80),
            ScreenVelocity::new(0, 0),
        );
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(106, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemies.len(), 1);
        assert_eq!(frame.state.world.projectiles.len(), 1);
        assert_eq!(frame.state.scores.player_one, 0);
        assert!(frame.events.gameplay().is_empty());
        assert!(frame.events.sounds().is_empty());
    }

    #[test]
    fn clean_game_pod_projectile_collision_spawns_source_bounded_swarmers() {
        let mut game = credited_started_game();
        game.state.wave = 2;
        game.state.wave_profile = WaveProfileSnapshot::for_wave(2);
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Pod,
            ScreenPosition::new(100, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            frame.state.world.enemies.len(),
            SOURCE_POD_SWARMER_REQUEST_LIMIT
        );
        assert!(
            frame
                .state
                .world
                .enemies
                .iter()
                .all(|enemy| enemy.kind == EnemyKind::Swarmer
                    && enemy.position == ScreenPosition::new(100, 80))
        );
        let mut expected_rng = SourceRandSnapshot::default();
        let expected_swarmers = (0..SOURCE_POD_SWARMER_REQUEST_LIMIT)
            .map(|_| {
                super::source_mini_swarmer_spawn(
                    &mut expected_rng,
                    game.state.wave_profile,
                    ScreenPosition::new(100, 80),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(frame.state.world.source_rng, expected_rng);
        assert_eq!(
            frame
                .state
                .world
                .enemies
                .iter()
                .map(|enemy| enemy.source_swarmer)
                .collect::<Vec<_>>(),
            expected_swarmers
                .iter()
                .copied()
                .map(Some)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            frame
                .state
                .world
                .enemies
                .iter()
                .map(|enemy| enemy.velocity)
                .collect::<Vec<_>>(),
            expected_swarmers
                .iter()
                .map(|swarmer| {
                    super::source_screen_velocity(swarmer.x_velocity, swarmer.y_velocity)
                })
                .collect::<Vec<_>>()
        );
        assert!(frame.state.world.projectiles.is_empty());
        assert_eq!(frame.state.scores.player_one, 1_000);
        assert_eq!(frame.events.gameplay(), &[GameEvent::EnemyDestroyed]);
        assert_eq!(
            frame.events.sounds(),
            &[source_enemy_hit_sound_event(EnemyKind::Pod)]
        );
        assert_eq!(frame.state.world.object_evidence.active_count, 16);
        assert_eq!(frame.state.world.object_evidence.projectile_count, 0);
        assert_eq!(
            frame.state.world.object_evidence.details[0].object_category,
            Some(ObjectEvidenceCategory::Swarmer)
        );
        assert_eq!(
            frame.state.world.object_evidence.details[0].mapped_sprite,
            Some(SpriteId::ENEMY_SWARMER)
        );
        assert_eq!(
            frame.state.world.expanded_objects.details[0].picture_label,
            Some("PRBP1")
        );
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_SWARMER
                && sprite.layer == RenderLayer::Objects
                && sprite.size == [8.0, 6.0]
        }));
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_POD
                && sprite.layer == RenderLayer::Objects
                && sprite.size == [4.0, 8.0]
        }));
    }

    #[test]
    fn clean_game_enemy_destroy_emits_source_hit_sound_commands() {
        for (kind, command) in [
            (EnemyKind::Lander, SOURCE_LHSND_SOUND_COMMAND),
            (EnemyKind::Mutant, SOURCE_SCHSND_SOUND_COMMAND),
            (EnemyKind::Bomber, SOURCE_TIHSND_SOUND_COMMAND),
            (EnemyKind::Pod, SOURCE_PRHSND_SOUND_COMMAND),
            (EnemyKind::Swarmer, SOURCE_SWHSND_SOUND_COMMAND),
            (EnemyKind::Baiter, SOURCE_UFHSND_SOUND_COMMAND),
        ] {
            let mut game = credited_started_game();
            game.state.world.enemies = vec![EnemySnapshot::new(
                kind,
                ScreenPosition::new(100, 80),
                ScreenVelocity::new(0, 0),
            )];
            game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
            game.state.world.source_astronaut_sleep_ticks = SOURCE_GAME_EXEC_SLEEP_FRAMES;
            game.baiter_timer_ticks = None;
            game.state.world.projectiles.push(ProjectileSnapshot {
                position: ScreenPosition::new(101, 83),
                velocity: ScreenVelocity::new(0, 0),
            });

            let frame = game.step(GameInput::NONE);

            assert_eq!(
                frame.events.sounds(),
                &[SoundEvent::UnmappedSoundCommand { command }],
                "{kind:?} should emit its source hit sound command",
            );
        }
    }

    #[test]
    fn clean_game_pod_swarmer_spawn_respects_source_active_limit() {
        let mut game = credited_started_game();
        game.state.wave = 2;
        game.state.wave_profile = WaveProfileSnapshot::for_wave(2);
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Pod,
            ScreenPosition::new(100, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state
            .world
            .enemies
            .extend((0..SOURCE_ACTIVE_SWARMER_LIMIT - 2).map(|index| {
                EnemySnapshot::new(
                    EnemyKind::Swarmer,
                    ScreenPosition::new(index as u8, 40),
                    ScreenVelocity::new(0, 0),
                )
            }));
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            frame
                .state
                .world
                .enemies
                .iter()
                .filter(|enemy| enemy.kind == EnemyKind::Swarmer)
                .count(),
            SOURCE_ACTIVE_SWARMER_LIMIT
        );
        assert_eq!(frame.state.scores.player_one, 1_000);
        assert_eq!(frame.events.gameplay(), &[GameEvent::EnemyDestroyed]);
    }

    #[test]
    fn clean_game_mini_swarmer_source_entry_seeks_horizontally_then_sleeps() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x8000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::source_swarmer(
            ScreenPosition::new(40, 80),
            ScreenVelocity::new(0, 0),
            SourceSwarmerSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                acceleration: 0x08,
                shot_timer: 7,
                sleep_ticks: 0,
                horizontal_seek_pending: true,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        let swarmer = frame
            .state
            .world
            .enemies
            .first()
            .expect("source swarmer should remain active");
        let source = swarmer
            .source_swarmer
            .expect("source swarmer state should be retained");
        let expected_x_velocity = super::source_mini_swarmer_seek_velocity(
            game.state.wave_profile.swarmer_x_velocity,
            0x20,
            40,
        );
        let (expected_x, expected_x_fraction) =
            super::source_fixed_axis_step(40, 0, expected_x_velocity);

        assert_eq!(source.x_velocity, expected_x_velocity);
        assert_eq!(source.y_velocity, 0);
        assert_eq!(source.x_fraction, expected_x_fraction);
        assert_eq!(source.y_fraction, 0);
        assert_eq!(source.sleep_ticks, SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS);
        assert!(!source.horizontal_seek_pending);
        assert_eq!(swarmer.position, ScreenPosition::new(expected_x, 80));
        assert_eq!(
            swarmer.velocity,
            super::source_screen_velocity(expected_x_velocity, 0)
        );
        assert!(frame.state.world.enemy_projectiles.is_empty());
    }

    #[test]
    fn clean_game_mini_swarmer_loop_updates_y_velocity_and_projects_enemy_bomb() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2200), super::world_word(0x7000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::source_swarmer(
            ScreenPosition::new(32, 96),
            ScreenVelocity::new(1, 1),
            SourceSwarmerSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0020,
                y_velocity: 0x0100,
                acceleration: 0x20,
                shot_timer: 1,
                sleep_ticks: 0,
                horizontal_seek_pending: false,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x80,
            hseed: 0x00,
            lseed: 0x40,
        };
        game.baiter_timer_ticks = None;
        let mut expected_rng = game.state.world.source_rng;
        expected_rng.advance();
        let expected_y_velocity =
            super::source_mini_swarmer_y_velocity(0x0100, 0x20, 0x70, 0x60, 0x80);
        let expected_projectile_x_velocity = 0x0020u16.wrapping_shl(3);
        let expected_projectile_y_velocity =
            super::source_arithmetic_shift_right_word(u16::from_be_bytes([0x10, 0]), 5);
        let (expected_x, expected_x_fraction) = super::source_fixed_axis_step(32, 0, 0x0020);
        let (expected_y, expected_y_fraction) =
            super::source_fixed_axis_step(96, 0, expected_y_velocity);

        let frame = game.step(GameInput::NONE);

        let swarmer = frame
            .state
            .world
            .enemies
            .first()
            .expect("source swarmer should remain active");
        let source = swarmer
            .source_swarmer
            .expect("source swarmer state should be retained");
        assert_eq!(source.x_velocity, 0x0020);
        assert_eq!(source.y_velocity, expected_y_velocity);
        assert_eq!(source.x_fraction, expected_x_fraction);
        assert_eq!(source.y_fraction, expected_y_fraction);
        assert_eq!(
            source.shot_timer,
            super::source_rmax(
                game.state.wave_profile.swarmer_shot_time as u8,
                expected_rng.seed
            )
        );
        assert_eq!(source.sleep_ticks, SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS);
        assert_eq!(
            swarmer.position,
            ScreenPosition::new(expected_x, expected_y)
        );
        assert_eq!(
            swarmer.velocity,
            super::source_screen_velocity(0x0020, expected_y_velocity)
        );
        assert_eq!(frame.state.world.source_rng, expected_rng);

        assert_eq!(frame.state.world.enemy_projectiles.len(), 1);
        let projectile = frame.state.world.enemy_projectiles[0];
        assert_eq!(projectile.position, ScreenPosition::new(32, 96));
        assert_eq!(projectile.source_x_velocity, expected_projectile_x_velocity);
        assert_eq!(projectile.source_y_velocity, expected_projectile_y_velocity);
        assert_eq!(
            projectile.velocity,
            super::source_screen_velocity(
                expected_projectile_x_velocity,
                expected_projectile_y_velocity
            )
        );
        assert_eq!(
            frame.events.sounds(),
            &[source_enemy_shot_sound_event(EnemyKind::Swarmer).expect("swarmer shot sound")]
        );
        assert_eq!(frame.state.world.object_evidence.active_count, 3);
        assert_eq!(frame.state.world.object_evidence.projectile_count, 1);
        assert_eq!(
            frame.state.world.object_evidence.details[2].object_category,
            Some(ObjectEvidenceCategory::EnemyBomb)
        );
        assert_eq!(
            frame.state.world.object_evidence.details[2].mapped_sprite,
            Some(SpriteId::ENEMY_BOMB)
        );
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_BOMB
                && sprite.layer == RenderLayer::Projectiles
                && sprite.position == [32.0, 96.0]
                && sprite.size == [4.0, 6.0]
        }));
    }

    #[test]
    fn clean_game_mini_swarmer_bomb_respects_source_shell_limit() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2200), super::world_word(0x7000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::source_swarmer(
            ScreenPosition::new(32, 96),
            ScreenVelocity::new(1, 1),
            SourceSwarmerSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0020,
                y_velocity: 0x0100,
                acceleration: 0x20,
                shot_timer: 1,
                sleep_ticks: 0,
                horizontal_seek_pending: false,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.enemy_projectiles = (0..super::SOURCE_SHELL_LIMIT)
            .map(|_| {
                super::EnemyProjectileSnapshot::source_fireball(
                    ScreenPosition::new(0x90, 0xE0),
                    0,
                    0,
                )
            })
            .collect();
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x80,
            hseed: 0x00,
            lseed: 0x40,
        };
        game.baiter_timer_ticks = None;
        let mut expected_rng = game.state.world.source_rng;
        expected_rng.advance();

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            frame.state.world.enemy_projectiles.len(),
            super::SOURCE_SHELL_LIMIT
        );
        assert_eq!(frame.state.world.source_rng, expected_rng);
        let swarmer = frame
            .state
            .world
            .enemies
            .first()
            .expect("source swarmer should remain active");
        let source = swarmer
            .source_swarmer
            .expect("source swarmer state should be retained");
        assert_eq!(
            source.shot_timer,
            super::source_rmax(
                game.state.wave_profile.swarmer_shot_time as u8,
                expected_rng.seed
            )
        );
        assert_eq!(
            frame.state.world.object_evidence.projectile_count,
            super::SOURCE_SHELL_LIMIT as u16
        );
        assert!(
            frame
                .state
                .world
                .enemy_projectiles
                .iter()
                .all(|projectile| projectile.position != ScreenPosition::new(32, 96))
        );
        assert!(frame.events.sounds().is_empty());
    }

    #[test]
    fn clean_game_baiter_runtime_entry_uses_source_pacing_timer() {
        let mut game = credited_started_game();
        game.baiter_timer_ticks = Some(1);
        game.baiter_pacing_frames_remaining = 1;
        let expected_player_position = ScreenPosition::new(0x20, 0x80);
        let expected_player_velocity = (WorldVector::default(), WorldVector::default());
        let (expected_position, expected_source_baiter) = super::source_baiter_spawn(
            game.state.world.source_rng,
            game.state.wave_profile,
            expected_player_position,
            expected_player_velocity,
        );

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            game.baiter_timer_ticks,
            Some(game.state.wave_profile.baiter_delay)
        );
        assert_eq!(
            game.baiter_pacing_frames_remaining,
            SOURCE_GAME_EXEC_SLEEP_FRAMES
        );
        assert_eq!(
            frame
                .state
                .world
                .enemies
                .iter()
                .filter(|enemy| enemy.kind == EnemyKind::Baiter)
                .count(),
            1
        );
        let baiter = frame
            .state
            .world
            .enemies
            .iter()
            .find(|enemy| enemy.kind == EnemyKind::Baiter)
            .expect("baiter spawn");
        assert_eq!(baiter.position, expected_position);
        assert_eq!(
            baiter.velocity,
            super::source_baiter_screen_velocity(expected_source_baiter)
        );
        assert_eq!(baiter.source_baiter, Some(expected_source_baiter));
        assert!(frame.events.gameplay().is_empty());
        assert_eq!(frame.state.world.object_evidence.active_count, 16);
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_BAITER
                && sprite.layer == RenderLayer::Objects
                && sprite.size == [12.0, 8.0]
        }));
    }

    #[test]
    fn clean_game_baiter_runtime_entry_respects_source_active_cap() {
        let mut game = credited_started_game();
        game.state.world.enemies = (0..SOURCE_ACTIVE_BAITER_LIMIT)
            .map(|index| {
                EnemySnapshot::new(
                    EnemyKind::Baiter,
                    ScreenPosition::new(index as u8, 68),
                    ScreenVelocity::new(0, 0),
                )
            })
            .collect();
        game.state.world.enemies.push(EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        ));
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = Some(1);
        game.baiter_pacing_frames_remaining = 1;

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            frame
                .state
                .world
                .enemies
                .iter()
                .filter(|enemy| enemy.kind == EnemyKind::Baiter)
                .count(),
            SOURCE_ACTIVE_BAITER_LIMIT
        );
        assert!(frame.events.gameplay().is_empty());
    }

    #[test]
    fn clean_game_baiter_timer_ignores_active_baiters_for_source_enemy_total() {
        let mut game = credited_started_game();
        let mut enemies = (0..8)
            .map(|index| {
                EnemySnapshot::new(
                    EnemyKind::Lander,
                    ScreenPosition::new(160 + index as u8, 80),
                    ScreenVelocity::new(0, 0),
                )
            })
            .collect::<Vec<_>>();
        enemies.extend((0..5).map(|index| {
            EnemySnapshot::new(
                EnemyKind::Baiter,
                ScreenPosition::new(40 + index as u8, 60),
                ScreenVelocity::new(0, 0),
            )
        }));
        game.state.world.enemies = enemies;
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = Some(game.state.wave_profile.baiter_delay);
        game.baiter_pacing_frames_remaining = 1;

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            game.baiter_timer_ticks,
            Some(game.state.wave_profile.baiter_delay / 2)
        );
        assert!(frame.events.gameplay().is_empty());
        assert_eq!(
            frame
                .state
                .world
                .enemies
                .iter()
                .filter(|enemy| enemy.kind == EnemyKind::Baiter)
                .count(),
            5
        );
    }

    #[test]
    fn clean_game_baiters_do_not_block_source_reserve_activation() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Baiter,
            ScreenPosition::new(120, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot {
            landers: 1,
            ..EnemyReserveSnapshot::default()
        };
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert!(frame.events.gameplay().is_empty());
        assert_eq!(
            frame.state.world.enemy_reserve,
            EnemyReserveSnapshot::default()
        );
        assert_eq!(
            frame
                .state
                .world
                .enemies
                .iter()
                .filter(|enemy| enemy.kind == EnemyKind::Baiter)
                .count(),
            1
        );
        assert_eq!(
            frame
                .state
                .world
                .enemies
                .iter()
                .filter(|enemy| enemy.kind == EnemyKind::Lander)
                .count(),
            1
        );
    }

    #[test]
    fn clean_game_baiters_do_not_block_source_wave_clear() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Baiter,
            ScreenPosition::new(120, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.enemies.is_empty());
        assert_eq!(frame.events.gameplay(), &[GameEvent::WaveCleared]);
        assert!(
            !frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_BAITER)
        );
    }

    #[test]
    fn clean_game_baiter_timer_accelerates_when_enemy_total_is_low() {
        let mut game = credited_started_game();
        game.state.world.enemies.truncate(5);
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = Some(game.state.wave_profile.baiter_delay);
        game.baiter_pacing_frames_remaining = 1;

        game.step(GameInput::NONE);

        assert_eq!(
            game.baiter_timer_ticks,
            Some(game.state.wave_profile.baiter_delay / 2)
        );
    }

    #[test]
    fn clean_game_baiter_source_loop_cycles_fires_and_updates_velocity() {
        let mut game = credited_started_game();
        let player_position = ScreenPosition::new(0x20, 0x80);
        let player_velocity = (WorldVector::default(), WorldVector::default());
        game.state.player.position = (
            super::world_word(u16::from(player_position.x) << 8),
            super::world_word(u16::from(player_position.y) << 8),
        );
        game.state.player.velocity = player_velocity;
        game.state.world.enemies = vec![EnemySnapshot::source_baiter(
            ScreenPosition::new(0x40, 0x60),
            ScreenVelocity::new(0, 0),
            SourceBaiterSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: 0,
                picture_frame: 2,
            },
        )];
        game.state.world.enemies.push(EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        ));
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x90,
            hseed: 0x20,
            lseed: 0x13,
        };
        game.baiter_timer_ticks = None;

        let mut expected_rng = game.state.world.source_rng;
        expected_rng.advance();
        let mut expected_source = SourceBaiterSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: super::source_rmax(
                game.state.wave_profile.baiter_shot_time as u8,
                expected_rng.seed,
            ),
            sleep_ticks: SOURCE_BAITER_LOOP_SLEEP_TICKS,
            picture_frame: 0,
        };
        let expected_projectile = super::source_baiter_shot(
            ScreenPosition::new(0x40, 0x60),
            expected_source,
            player_position,
            player_velocity,
            expected_rng,
            0,
        )
        .expect("baiter fireball");
        super::source_baiter_velocity_update(
            &mut expected_source,
            ScreenPosition::new(0x40, 0x60),
            game.state.wave_profile,
            player_position,
            player_velocity,
            true,
            expected_rng.seed,
        );
        let (expected_x, expected_x_fraction) = super::source_fixed_axis_step(
            0x40,
            0,
            super::source_baiter_screen_x_velocity(expected_source.x_velocity),
        );
        let (expected_y, expected_y_fraction) =
            super::source_fixed_axis_step(0x60, 0, expected_source.y_velocity);
        expected_source.x_fraction = expected_x_fraction;
        expected_source.y_fraction = expected_y_fraction;

        let frame = game.step(GameInput::NONE);

        let baiter = frame
            .state
            .world
            .enemies
            .first()
            .expect("source baiter should remain active");
        assert_eq!(baiter.position, ScreenPosition::new(expected_x, expected_y));
        assert_eq!(
            baiter.velocity,
            super::source_baiter_screen_velocity(expected_source)
        );
        assert_eq!(baiter.source_baiter, Some(expected_source));
        assert_eq!(frame.state.world.source_rng, expected_rng);
        assert_eq!(
            frame.state.world.enemy_projectiles,
            vec![expected_projectile]
        );
        assert_eq!(
            frame.events.sounds(),
            &[source_enemy_shot_sound_event(EnemyKind::Baiter).expect("baiter shot sound")]
        );
        assert_eq!(frame.state.world.object_evidence.projectile_count, 1);
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_BOMB
                && sprite.layer == RenderLayer::Projectiles
                && sprite.position == [64.0, 96.0]
        }));
    }

    #[test]
    fn clean_game_bomber_source_loop_cycles_picture_and_projects_bomb() {
        let mut game = credited_started_game();
        let player_position = ScreenPosition::new(0x20, 0x50);
        game.state.player.position = (
            super::world_word(u16::from(player_position.x) << 8),
            super::world_word(u16::from(player_position.y) << 8),
        );
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::source_bomber(
            ScreenPosition::new(0x08, 0x60),
            ScreenVelocity::new(0, 0),
            SourceBomberSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: 0x50,
                sleep_ticks: 0,
                source_slot: 0,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0,
            hseed: 0x80,
            lseed: 0,
        };
        game.baiter_timer_ticks = None;

        let expected_y_velocity = super::source_bomber_onscreen_y_velocity_delta(0x60, 0x50)
            .expect("onscreen y delta")
            .wrapping_add(super::source_bomber_random_y_velocity(0, 0));
        let expected_projectile = super::source_bomber_bomb_shell(
            ScreenPosition::new(0x08, 0x60),
            SourceBomberSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: expected_y_velocity,
                picture_frame: 1,
                cruise_altitude: 0x50,
                sleep_ticks: super::SOURCE_BOMBER_LOOP_SLEEP_TICKS,
                source_slot: 0,
            },
            game.state.world.source_rng,
            0,
            0,
        )
        .expect("bomber bomb shell");
        assert_eq!(
            expected_projectile.source_kind,
            super::EnemyProjectileSourceKind::BomberBombShell
        );
        assert_eq!(
            expected_projectile.source_output_routine_address(),
            super::SOURCE_BOMB_SHELL_OUTPUT_ROUTINE_ADDRESS
        );
        let (expected_y, expected_y_fraction) =
            super::source_fixed_axis_step(0x60, 0, expected_y_velocity);
        let expected_source = SourceBomberSnapshot {
            x_fraction: 0,
            y_fraction: expected_y_fraction,
            x_velocity: 0,
            y_velocity: expected_y_velocity,
            picture_frame: 1,
            cruise_altitude: 0x50,
            sleep_ticks: super::SOURCE_BOMBER_LOOP_SLEEP_TICKS,
            source_slot: 0,
        };

        let frame = game.step(GameInput::NONE);

        let bomber = frame
            .state
            .world
            .enemies
            .first()
            .expect("source bomber should remain active");
        assert_eq!(bomber.kind, EnemyKind::Bomber);
        assert_eq!(bomber.position, ScreenPosition::new(0x08, expected_y));
        assert_eq!(
            bomber.velocity,
            super::source_bomber_screen_velocity(expected_source)
        );
        assert_eq!(bomber.source_bomber, Some(expected_source));
        assert_eq!(
            frame.state.world.enemy_projectiles,
            vec![expected_projectile]
        );
        assert!(frame.events.sounds().is_empty());
        assert_eq!(frame.state.world.object_evidence.projectile_count, 1);
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_BOMB
                && sprite.layer == RenderLayer::Projectiles
                && sprite.position == [8.0, 96.0]
        }));
    }

    #[test]
    fn clean_game_bomber_tie_selection_sleeps_empty_source_slot() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x5000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::source_bomber(
            ScreenPosition::new(0x40, 0x60),
            ScreenVelocity::new(1, 1),
            SourceBomberSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0100,
                y_velocity: 0x0100,
                picture_frame: 2,
                cruise_altitude: 0x50,
                sleep_ticks: 0,
                source_slot: 0,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x06,
            hseed: 0x40,
            lseed: 0,
        };
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.enemy_projectiles.is_empty());
        let bomber = frame
            .state
            .world
            .enemies
            .first()
            .expect("source bomber should remain active");
        let expected_source = SourceBomberSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0x0100,
            y_velocity: 0x0100,
            picture_frame: 2,
            cruise_altitude: 0x50,
            sleep_ticks: 0,
            source_slot: 0,
        };
        assert_eq!(bomber.kind, EnemyKind::Bomber);
        assert_eq!(bomber.position, ScreenPosition::new(0x41, 0x61));
        assert_eq!(
            bomber.velocity,
            super::source_bomber_screen_velocity(expected_source)
        );
        assert_eq!(bomber.source_bomber, Some(expected_source));
    }

    #[test]
    fn clean_game_bomber_tie_selection_preserves_dead_squad_slot() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x5000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![
            EnemySnapshot::source_bomber(
                ScreenPosition::new(0x40, 0x60),
                ScreenVelocity::new(1, 0),
                SourceBomberSnapshot {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0x0100,
                    y_velocity: 0,
                    picture_frame: 1,
                    cruise_altitude: 0x50,
                    sleep_ticks: 0,
                    source_slot: 0,
                },
            ),
            EnemySnapshot::source_bomber(
                ScreenPosition::new(0x50, 0x70),
                ScreenVelocity::new(1, 0),
                SourceBomberSnapshot {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0x0100,
                    y_velocity: 0,
                    picture_frame: 2,
                    cruise_altitude: 0x50,
                    sleep_ticks: 0,
                    source_slot: 2,
                },
            ),
        ];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x02,
            hseed: 0x40,
            lseed: 0,
        };
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.enemy_projectiles.is_empty());
        assert_eq!(
            frame
                .state
                .world
                .enemies
                .iter()
                .map(|enemy| enemy.position)
                .collect::<Vec<_>>(),
            vec![
                ScreenPosition::new(0x41, 0x60),
                ScreenPosition::new(0x51, 0x70)
            ]
        );
        assert_eq!(
            frame
                .state
                .world
                .enemies
                .iter()
                .map(|enemy| enemy.source_bomber.expect("source bomber"))
                .collect::<Vec<_>>(),
            vec![
                SourceBomberSnapshot {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0x0100,
                    y_velocity: 0,
                    picture_frame: 1,
                    cruise_altitude: 0x50,
                    sleep_ticks: 0,
                    source_slot: 0,
                },
                SourceBomberSnapshot {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0x0100,
                    y_velocity: 0,
                    picture_frame: 2,
                    cruise_altitude: 0x50,
                    sleep_ticks: 0,
                    source_slot: 2,
                },
            ]
        );
    }

    #[test]
    fn clean_game_bomber_bomb_respects_source_getshl_bounds() {
        let mut game = credited_started_game();
        let player_position = ScreenPosition::new(0x20, 0x50);
        game.state.player.position = (
            super::world_word(u16::from(player_position.x) << 8),
            super::world_word(u16::from(player_position.y) << 8),
        );
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::source_bomber(
            ScreenPosition::new(super::SOURCE_SHELL_X_MAX, 0x60),
            ScreenVelocity::new(0, 0),
            SourceBomberSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: 0x50,
                sleep_ticks: 0,
                source_slot: 0,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0,
            hseed: 0x80,
            lseed: 0,
        };
        game.baiter_timer_ticks = None;

        let expected_y_velocity = super::source_bomber_onscreen_y_velocity_delta(0x60, 0x50)
            .expect("onscreen y delta")
            .wrapping_add(super::source_bomber_random_y_velocity(0, 0));
        let (expected_y, expected_y_fraction) =
            super::source_fixed_axis_step(0x60, 0, expected_y_velocity);
        let expected_source = SourceBomberSnapshot {
            x_fraction: 0,
            y_fraction: expected_y_fraction,
            x_velocity: 0,
            y_velocity: expected_y_velocity,
            picture_frame: 1,
            cruise_altitude: 0x50,
            sleep_ticks: super::SOURCE_BOMBER_LOOP_SLEEP_TICKS,
            source_slot: 0,
        };

        let frame = game.step(GameInput::NONE);

        let bomber = frame
            .state
            .world
            .enemies
            .first()
            .expect("source bomber should remain active");
        assert_eq!(
            bomber.position,
            ScreenPosition::new(super::SOURCE_SHELL_X_MAX, expected_y)
        );
        assert_eq!(
            bomber.velocity,
            super::source_bomber_screen_velocity(expected_source)
        );
        assert_eq!(bomber.source_bomber, Some(expected_source));
        assert!(frame.state.world.enemy_projectiles.is_empty());
        assert_eq!(frame.state.world.object_evidence.projectile_count, 0);
    }

    #[test]
    fn clean_game_bomber_bomb_counts_bmbout_shells_separately_from_fireballs() {
        let mut game = credited_started_game();
        let player_position = ScreenPosition::new(0x20, 0x50);
        game.state.player.position = (
            super::world_word(u16::from(player_position.x) << 8),
            super::world_word(u16::from(player_position.y) << 8),
        );
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::source_bomber(
            ScreenPosition::new(0x08, 0x60),
            ScreenVelocity::new(0, 0),
            SourceBomberSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: 0x50,
                sleep_ticks: 0,
                source_slot: 0,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.enemy_projectiles = (0..super::SOURCE_BOMBER_BOMB_SHELL_LIMIT)
            .map(|_| {
                super::EnemyProjectileSnapshot::source_fireball(
                    ScreenPosition::new(0x90, 0xE0),
                    0,
                    0,
                )
            })
            .collect();
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0,
            hseed: 0x80,
            lseed: 0,
        };
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            frame.state.world.enemy_projectiles.len(),
            super::SOURCE_BOMBER_BOMB_SHELL_LIMIT + 1
        );
        assert_eq!(
            super::source_bomb_shell_count(&frame.state.world.enemy_projectiles),
            1
        );
        assert!(
            frame
                .state
                .world
                .enemy_projectiles
                .iter()
                .any(|projectile| {
                    projectile.source_kind == super::EnemyProjectileSourceKind::BomberBombShell
                        && projectile.source_output_routine_address()
                            == super::SOURCE_BOMB_SHELL_OUTPUT_ROUTINE_ADDRESS
                        && projectile.position == ScreenPosition::new(0x08, 0x60)
                })
        );
    }

    #[test]
    fn clean_game_bomber_bomb_respects_source_bmbcnt_limit() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x5000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::source_bomber(
            ScreenPosition::new(0x08, 0x60),
            ScreenVelocity::new(0, 0),
            SourceBomberSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: 0x50,
                sleep_ticks: 0,
                source_slot: 0,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.enemy_projectiles = (0..super::SOURCE_BOMBER_BOMB_SHELL_LIMIT)
            .map(|_| {
                super::EnemyProjectileSnapshot::source_bomb_shell(ScreenPosition::new(0x90, 0xE0))
            })
            .collect();
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0,
            hseed: 0x80,
            lseed: 0,
        };
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            frame.state.world.enemy_projectiles.len(),
            super::SOURCE_BOMBER_BOMB_SHELL_LIMIT
        );
        assert_eq!(
            super::source_bomb_shell_count(&frame.state.world.enemy_projectiles),
            super::SOURCE_BOMBER_BOMB_SHELL_LIMIT
        );
        assert_eq!(frame.state.world.object_evidence.projectile_count, 10);
    }

    #[test]
    fn clean_game_bomber_bomb_respects_total_source_shell_limit() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x5000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::source_bomber(
            ScreenPosition::new(0x08, 0x60),
            ScreenVelocity::new(0, 0),
            SourceBomberSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: 0x50,
                sleep_ticks: 0,
                source_slot: 0,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.enemy_projectiles = (0..super::SOURCE_SHELL_LIMIT)
            .map(|_| {
                super::EnemyProjectileSnapshot::source_fireball(
                    ScreenPosition::new(0x90, 0xE0),
                    0,
                    0,
                )
            })
            .collect();
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0,
            hseed: 0x80,
            lseed: 0,
        };
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            frame.state.world.enemy_projectiles.len(),
            super::SOURCE_SHELL_LIMIT
        );
        assert_eq!(
            super::source_bomb_shell_count(&frame.state.world.enemy_projectiles),
            0
        );
    }

    #[test]
    fn clean_game_lander_source_loop_orbits_cycles_and_fires() {
        let mut game = credited_started_game();
        let player_position = ScreenPosition::new(0x20, 0x80);
        let player_velocity = (WorldVector::default(), WorldVector::default());
        game.state.player.position = (
            super::world_word(u16::from(player_position.x) << 8),
            super::world_word(u16::from(player_position.y) << 8),
        );
        game.state.player.velocity = player_velocity;
        game.state.world.enemies = vec![EnemySnapshot::source_lander(
            ScreenPosition::new(0x40, 0x60),
            ScreenVelocity::new(0, 0),
            SourceLanderSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0020,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: 0,
                picture_frame: 2,
                target_human_index: Some(0),
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot::new(ScreenPosition::new(10, 216))];
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x90,
            hseed: 0x20,
            lseed: 0x13,
        };
        game.baiter_timer_ticks = None;

        let mut expected_rng = game.state.world.source_rng;
        expected_rng.advance();
        let expected_y_velocity = super::source_lander_orbit_y_velocity(
            game.state.wave_profile,
            ScreenPosition::new(0x40, 0x60),
            &game.state.world.terrain,
        );
        let expected_projectile = super::source_enemy_fireball_shot(
            ScreenPosition::new(0x40, 0x60),
            0,
            0,
            player_position,
            player_velocity,
            expected_rng,
            0,
        )
        .expect("lander fireball");
        let (expected_x, expected_x_fraction) = super::source_fixed_axis_step(0x40, 0, 0x0020);
        let (expected_y, expected_y_fraction) =
            super::source_fixed_axis_step(0x60, 0, expected_y_velocity);
        let expected_source = SourceLanderSnapshot {
            x_fraction: expected_x_fraction,
            y_fraction: expected_y_fraction,
            x_velocity: 0x0020,
            y_velocity: expected_y_velocity,
            shot_timer: super::source_rmax(
                game.state.wave_profile.lander_shot_time as u8,
                expected_rng.seed,
            ),
            sleep_ticks: SOURCE_LANDER_ORBIT_SLEEP_TICKS,
            picture_frame: 0,
            target_human_index: Some(0),
        };

        let frame = game.step(GameInput::NONE);

        let lander = frame
            .state
            .world
            .enemies
            .first()
            .expect("source lander should remain active");
        assert_eq!(lander.kind, EnemyKind::Lander);
        assert_eq!(lander.position, ScreenPosition::new(expected_x, expected_y));
        assert_eq!(
            lander.velocity,
            super::source_lander_screen_velocity(expected_source)
        );
        assert_eq!(lander.source_lander, Some(expected_source));
        assert_eq!(frame.state.world.source_rng, expected_rng);
        assert_eq!(
            frame.state.world.enemy_projectiles,
            vec![expected_projectile]
        );
        assert_eq!(
            frame.events.sounds(),
            &[source_enemy_shot_sound_event(EnemyKind::Lander).expect("lander shot sound")]
        );
        assert_eq!(frame.state.world.object_evidence.projectile_count, 1);
    }

    #[test]
    fn clean_game_source_lander_orbit_enters_grab_for_selected_target() {
        let mut game = credited_started_game();
        let player_position = ScreenPosition::new(0x20, 0x80);
        let player_velocity = (WorldVector::default(), WorldVector::default());
        game.state.player.position = (
            super::world_word(u16::from(player_position.x) << 8),
            super::world_word(u16::from(player_position.y) << 8),
        );
        game.state.player.velocity = player_velocity;
        let lander_start = ScreenPosition::new(100, 80);
        let target_position = ScreenPosition::new(100, 214);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0x0040,
            y_velocity: 0x0100,
            shot_timer: 2,
            sleep_ticks: 0,
            picture_frame: 2,
            target_human_index: Some(0),
        };
        game.state.world.enemies = vec![EnemySnapshot::source_lander(
            lander_start,
            super::source_lander_screen_velocity(source_lander),
            source_lander,
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot::new(target_position)];
        game.baiter_timer_ticks = None;

        let y_step = super::source_lander_base_y_velocity(game.state.wave_profile);
        let (expected_y, expected_y_fraction) =
            super::source_fixed_axis_step(lander_start.y, source_lander.y_fraction, y_step);
        let expected_source = SourceLanderSnapshot {
            y_fraction: expected_y_fraction,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: super::SOURCE_LANDER_GRAB_SLEEP_TICKS,
            picture_frame: 0,
            ..source_lander
        };

        let frame = game.step(GameInput::NONE);

        let lander = frame
            .state
            .world
            .enemies
            .first()
            .expect("source lander should enter grab step");
        assert_eq!(
            lander.position,
            ScreenPosition::new(lander_start.x, expected_y)
        );
        assert_eq!(
            lander.velocity,
            super::source_lander_screen_velocity(expected_source)
        );
        assert_eq!(lander.source_lander, Some(expected_source));
        assert_eq!(
            frame.state.world.humans,
            vec![HumanSnapshot::new(target_position)]
        );
        assert!(frame.state.world.enemy_projectiles.is_empty());
    }

    #[test]
    fn clean_game_source_lander_ignores_untargeted_aligned_human() {
        let mut game = credited_started_game();
        let lander_start = ScreenPosition::new(100, 80);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0x0040,
            y_velocity: 0,
            shot_timer: 5,
            sleep_ticks: 0,
            picture_frame: 1,
            target_human_index: Some(1),
        };
        game.state.world.enemies = vec![EnemySnapshot::source_lander(
            lander_start,
            super::source_lander_screen_velocity(source_lander),
            source_lander,
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![
            HumanSnapshot::new(ScreenPosition::new(100, 92)),
            HumanSnapshot::new(ScreenPosition::new(140, 214)),
        ];
        game.baiter_timer_ticks = None;

        let expected_y_velocity = super::source_lander_orbit_y_velocity(
            game.state.wave_profile,
            lander_start,
            &game.state.world.terrain,
        );
        let (expected_x, expected_x_fraction) =
            super::source_fixed_axis_step(lander_start.x, source_lander.x_fraction, 0x0040);
        let (expected_y, expected_y_fraction) = super::source_active_object_y_step(
            lander_start.y,
            source_lander.y_fraction,
            expected_y_velocity,
        );
        let expected_source = SourceLanderSnapshot {
            x_fraction: expected_x_fraction,
            y_fraction: expected_y_fraction,
            y_velocity: expected_y_velocity,
            shot_timer: 4,
            sleep_ticks: super::SOURCE_LANDER_ORBIT_SLEEP_TICKS,
            picture_frame: 2,
            ..source_lander
        };

        let frame = game.step(GameInput::NONE);

        let lander = frame
            .state
            .world
            .enemies
            .first()
            .expect("source lander should keep orbiting");
        assert_eq!(lander.position, ScreenPosition::new(expected_x, expected_y));
        assert_eq!(lander.source_lander, Some(expected_source));
        assert_eq!(
            frame.state.world.humans,
            vec![
                HumanSnapshot {
                    source_fall_velocity: super::SOURCE_FALLING_HUMAN_Y_ACCELERATION,
                    source_fall_y_fraction: super::SOURCE_FALLING_HUMAN_Y_ACCELERATION
                        .to_be_bytes()[1],
                    ..HumanSnapshot::new(ScreenPosition::new(100, 92))
                },
                HumanSnapshot {
                    source_fall_velocity: super::SOURCE_FALLING_HUMAN_Y_ACCELERATION,
                    source_fall_y_fraction: super::SOURCE_FALLING_HUMAN_Y_ACCELERATION
                        .to_be_bytes()[1],
                    ..HumanSnapshot::new(ScreenPosition::new(140, 214))
                },
            ]
        );
        assert!(frame.state.world.enemy_projectiles.is_empty());
    }

    #[test]
    fn clean_game_source_lander_grab_step_seeks_target_before_capture() {
        let mut game = credited_started_game();
        let player_position = ScreenPosition::new(0x20, 0x80);
        let player_velocity = (WorldVector::default(), WorldVector::default());
        game.state.player.position = (
            super::world_word(u16::from(player_position.x) << 8),
            super::world_word(u16::from(player_position.y) << 8),
        );
        game.state.player.velocity = player_velocity;
        let lander_start = ScreenPosition::new(100, 80);
        let target_position = ScreenPosition::new(100, 214);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 2,
            sleep_ticks: 0,
            picture_frame: 0,
            target_human_index: Some(0),
        };
        game.state.world.enemies = vec![EnemySnapshot::source_lander(
            lander_start,
            super::source_lander_screen_velocity(source_lander),
            source_lander,
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot::new(target_position)];
        game.baiter_timer_ticks = None;

        let y_step = super::source_lander_base_y_velocity(game.state.wave_profile);
        let (expected_y, expected_y_fraction) =
            super::source_fixed_axis_step(lander_start.y, source_lander.y_fraction, y_step);
        let expected_source = SourceLanderSnapshot {
            y_fraction: expected_y_fraction,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: super::SOURCE_LANDER_GRAB_SLEEP_TICKS,
            picture_frame: 0,
            ..source_lander
        };

        let frame = game.step(GameInput::NONE);

        let lander = frame
            .state
            .world
            .enemies
            .first()
            .expect("grabbing source lander");
        assert_eq!(
            lander.position,
            ScreenPosition::new(lander_start.x, expected_y)
        );
        assert_eq!(
            lander.velocity,
            super::source_lander_screen_velocity(expected_source)
        );
        assert_eq!(lander.source_lander, Some(expected_source));
        assert_eq!(
            frame.state.world.humans,
            vec![HumanSnapshot::new(target_position)]
        );
        assert!(frame.state.world.enemy_projectiles.is_empty());
    }

    #[test]
    fn clean_game_source_lander_grab_step_seeks_upward_target() {
        let mut game = credited_started_game();
        let lander_start = ScreenPosition::new(100, 80);
        let target_position = ScreenPosition::new(100, 60);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 2,
            sleep_ticks: 0,
            picture_frame: 0,
            target_human_index: Some(0),
        };
        game.state.world.enemies = vec![EnemySnapshot::source_lander(
            lander_start,
            super::source_lander_screen_velocity(source_lander),
            source_lander,
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot::new(target_position)];
        game.baiter_timer_ticks = None;

        let y_step = !super::source_lander_base_y_velocity(game.state.wave_profile);
        let (expected_y, expected_y_fraction) =
            super::source_fixed_axis_step(lander_start.y, source_lander.y_fraction, y_step);
        let expected_source = SourceLanderSnapshot {
            y_fraction: expected_y_fraction,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: super::SOURCE_LANDER_GRAB_SLEEP_TICKS,
            picture_frame: 0,
            ..source_lander
        };

        let frame = game.step(GameInput::NONE);

        let lander = frame
            .state
            .world
            .enemies
            .first()
            .expect("grabbing source lander");
        assert_eq!(
            lander.position,
            ScreenPosition::new(lander_start.x, expected_y)
        );
        assert_eq!(
            lander.velocity,
            super::source_lander_screen_velocity(expected_source)
        );
        assert_eq!(lander.source_lander, Some(expected_source));
        assert_eq!(
            frame.state.world.humans,
            vec![HumanSnapshot {
                source_fall_velocity: super::SOURCE_FALLING_HUMAN_Y_ACCELERATION,
                source_fall_y_fraction: super::SOURCE_FALLING_HUMAN_Y_ACCELERATION.to_be_bytes()[1],
                ..HumanSnapshot::new(target_position)
            }]
        );
        assert!(frame.state.world.enemy_projectiles.is_empty());
    }

    #[test]
    fn clean_game_source_lander_flee_state_stays_with_carried_passenger() {
        let mut game = credited_started_game();
        let player_position = ScreenPosition::new(0x20, 0x80);
        let player_velocity = (WorldVector::default(), WorldVector::default());
        game.state.player.position = (
            super::world_word(u16::from(player_position.x) << 8),
            super::world_word(u16::from(player_position.y) << 8),
        );
        game.state.player.velocity = player_velocity;
        let first_start = ScreenPosition::new(0x40, 0x60);
        let second_start = ScreenPosition::new(0x70, 0x60);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 5,
            sleep_ticks: 0,
            picture_frame: 0,
            target_human_index: Some(0),
        };
        game.state.world.enemies = vec![
            EnemySnapshot::source_lander(first_start, ScreenVelocity::new(0, 0), source_lander),
            EnemySnapshot::source_lander(second_start, ScreenVelocity::new(0, 0), source_lander),
        ];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot {
            carried: true,
            ..HumanSnapshot::new(super::clean_carried_human_position(second_start))
        }];
        game.baiter_timer_ticks = None;

        let first_y_velocity = super::source_lander_orbit_y_velocity(
            game.state.wave_profile,
            first_start,
            &game.state.world.terrain,
        );
        let second_y_velocity = !super::source_lander_base_y_velocity(game.state.wave_profile);
        let (first_y, first_y_fraction) =
            super::source_active_object_y_step(first_start.y, 0, first_y_velocity);
        let (second_y, second_y_fraction) =
            super::source_active_object_y_step(second_start.y, 0, second_y_velocity);

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            frame.state.world.enemies[0].position,
            ScreenPosition::new(first_start.x, first_y)
        );
        assert_eq!(
            frame.state.world.enemies[0].source_lander,
            Some(SourceLanderSnapshot {
                y_fraction: first_y_fraction,
                y_velocity: first_y_velocity,
                shot_timer: 4,
                sleep_ticks: SOURCE_LANDER_ORBIT_SLEEP_TICKS,
                picture_frame: 1,
                target_human_index: None,
                ..source_lander
            })
        );
        assert_eq!(
            frame.state.world.enemies[1].position,
            ScreenPosition::new(second_start.x, second_y)
        );
        assert_eq!(
            frame.state.world.enemies[1].source_lander,
            Some(SourceLanderSnapshot {
                y_fraction: second_y_fraction,
                y_velocity: second_y_velocity,
                shot_timer: 4,
                sleep_ticks: super::SOURCE_LANDER_FLEE_SLEEP_TICKS,
                ..source_lander
            })
        );
        assert_eq!(
            frame.state.world.humans[0].position,
            super::clean_carried_human_position(ScreenPosition::new(second_start.x, second_y))
        );
        assert!(frame.state.world.humans[0].carried);
        assert!(frame.state.world.enemy_projectiles.is_empty());
    }

    #[test]
    fn clean_game_pod_source_motion_uses_fixed_point_velocity() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::source_pod(
            ScreenPosition::new(0x40, 0x60),
            ScreenVelocity::new(0, 0),
            SourcePodSnapshot {
                x_fraction: 0xF0,
                y_fraction: 0x10,
                x_velocity: 0x0020,
                y_velocity: 0xFFE0,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.baiter_timer_ticks = None;
        let (expected_x, expected_x_fraction) = super::source_fixed_axis_step(0x40, 0xF0, 0x0020);
        let (expected_y, expected_y_fraction) = super::source_fixed_axis_step(0x60, 0x10, 0xFFE0);
        let expected_source = SourcePodSnapshot {
            x_fraction: expected_x_fraction,
            y_fraction: expected_y_fraction,
            x_velocity: 0x0020,
            y_velocity: 0xFFE0,
        };

        let frame = game.step(GameInput::NONE);

        let pod = frame
            .state
            .world
            .enemies
            .first()
            .expect("source pod should remain active");
        assert_eq!(pod.kind, EnemyKind::Pod);
        assert_eq!(pod.position, ScreenPosition::new(expected_x, expected_y));
        assert_eq!(
            pod.velocity,
            super::source_pod_screen_velocity(expected_source)
        );
        assert_eq!(pod.source_pod, Some(expected_source));
        assert!(frame.state.world.enemy_projectiles.is_empty());
        assert_eq!(
            frame.state.world.object_evidence.details[0].object_category,
            Some(ObjectEvidenceCategory::Pod)
        );
    }

    #[test]
    fn clean_game_mutant_source_loop_hops_fires_and_sleeps() {
        let mut game = credited_started_game();
        let player_position = ScreenPosition::new(0x20, 0x70);
        let player_velocity = (WorldVector::default(), WorldVector::default());
        game.state.player.position = (
            super::world_word(u16::from(player_position.x) << 8),
            super::world_word(u16::from(player_position.y) << 8),
        );
        game.state.player.velocity = player_velocity;
        game.state.world.enemies = vec![EnemySnapshot::source_mutant(
            ScreenPosition::new(0x40, 0x60),
            ScreenVelocity::new(0, 0),
            SourceMutantSnapshot {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: 0,
            },
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x80,
            hseed: 0x00,
            lseed: 0x40,
        };
        game.baiter_timer_ticks = None;

        let object_absolute_x = 0x4000;
        let player_absolute_x = 0x2000;
        let expected_x_velocity = super::source_mutant_x_velocity(
            game.state.wave_profile.mutant_x_velocity,
            player_absolute_x,
            object_absolute_x,
        );
        let expected_y_velocity = super::source_mutant_y_velocity(
            game.state.wave_profile,
            player_position.y,
            player_absolute_x,
            object_absolute_x,
            ScreenPosition::new(0x40, 0x60),
        );
        let pre_motion_position = ScreenPosition::new(
            0x40,
            0x60u8.wrapping_add(game.state.wave_profile.mutant_random_y),
        );
        let mut expected_rng = game.state.world.source_rng;
        expected_rng.advance();
        let expected_projectile = super::source_enemy_fireball_shot(
            pre_motion_position,
            0,
            0,
            player_position,
            player_velocity,
            expected_rng,
            0,
        )
        .expect("mutant fireball");
        let (expected_x, expected_x_fraction) =
            super::source_fixed_axis_step(pre_motion_position.x, 0, expected_x_velocity);
        let (expected_y, expected_y_fraction) =
            super::source_fixed_axis_step(pre_motion_position.y, 0, expected_y_velocity);
        let expected_source = SourceMutantSnapshot {
            x_fraction: expected_x_fraction,
            y_fraction: expected_y_fraction,
            x_velocity: expected_x_velocity,
            y_velocity: expected_y_velocity,
            shot_timer: super::source_rmax(
                game.state.wave_profile.mutant_shot_time as u8,
                expected_rng.seed,
            ),
            sleep_ticks: SOURCE_MUTANT_LOOP_SLEEP_TICKS,
        };

        let frame = game.step(GameInput::NONE);

        let mutant = frame
            .state
            .world
            .enemies
            .first()
            .expect("source mutant should remain active");
        assert_eq!(mutant.kind, EnemyKind::Mutant);
        assert_eq!(mutant.position, ScreenPosition::new(expected_x, expected_y));
        assert_eq!(
            mutant.velocity,
            super::source_mutant_screen_velocity(expected_source)
        );
        assert_eq!(mutant.source_mutant, Some(expected_source));
        assert_eq!(frame.state.world.source_rng, expected_rng);
        assert_eq!(
            frame.state.world.enemy_projectiles,
            vec![expected_projectile]
        );
        assert_eq!(
            frame.events.sounds(),
            &[source_enemy_shot_sound_event(EnemyKind::Mutant).expect("mutant shot sound")]
        );
    }

    #[test]
    fn clean_game_mutant_reserve_batch_uses_source_restore_state() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();
        game.state.world.enemy_reserve = EnemyReserveSnapshot {
            mutants: 1,
            ..EnemyReserveSnapshot::default()
        };
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0,
            hseed: 0x4C,
            lseed: 0,
        };
        game.baiter_timer_ticks = None;

        let mut expected_rng = game.state.world.source_rng;
        let (expected_position, expected_source) =
            super::source_mutant_restore_spawn(&mut expected_rng, game.state.wave_profile, 0);

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemies.len(), 1);
        let mutant = frame.state.world.enemies.first().expect("reserve mutant");
        assert_eq!(mutant.kind, EnemyKind::Mutant);
        assert_eq!(mutant.position, expected_position);
        assert_eq!(
            mutant.velocity,
            super::source_mutant_screen_velocity(expected_source)
        );
        assert_eq!(mutant.source_mutant, Some(expected_source));
        assert_eq!(frame.state.world.source_rng, expected_rng);
        assert_eq!(
            frame.state.world.enemy_reserve,
            EnemyReserveSnapshot::default()
        );
        assert!(frame.events.gameplay().is_empty());
    }

    #[test]
    fn clean_game_lander_reserve_batch_uses_source_landst_spawn_state() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();
        game.state.world.enemy_reserve = EnemyReserveSnapshot {
            landers: 3,
            ..EnemyReserveSnapshot::default()
        };
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x20,
            hseed: 0x66,
            lseed: 0x99,
        };
        game.baiter_timer_ticks = None;

        let mut expected_rng = game.state.world.source_rng;
        let expected_landers = (0..3)
            .map(|index| {
                let mut lander =
                    super::source_lander_restore_spawn(&mut expected_rng, game.state.wave_profile);
                lander
                    .source_lander
                    .as_mut()
                    .expect("restored source lander")
                    .target_human_index = Some(index + 6);
                lander
            })
            .collect::<Vec<_>>();

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemies, expected_landers);
        assert_eq!(
            frame.state.world.source_target_list_cursor_address,
            Some(super::source_target_list_slot_address(8))
        );
        assert!(frame.state.world.enemies.iter().all(|enemy| {
            enemy.kind == EnemyKind::Lander
                && enemy.position.y == SOURCE_PLAYFIELD_Y_MIN.wrapping_add(2)
                && enemy.source_lander.is_some()
                && enemy.source_mutant.is_none()
                && enemy.source_bomber.is_none()
                && enemy.source_swarmer.is_none()
                && enemy.source_baiter.is_none()
                && enemy.source_pod.is_none()
        }));
        assert_eq!(frame.state.world.source_rng, expected_rng);
        assert_eq!(
            frame.state.world.enemy_reserve,
            EnemyReserveSnapshot::default()
        );
        assert!(frame.events.gameplay().is_empty());
        assert_eq!(frame.state.world.object_evidence.inactive_count, 0);
        assert_eq!(
            frame.state.world.object_evidence.details[0].object_category,
            Some(ObjectEvidenceCategory::Lander)
        );
        assert_eq!(
            frame.state.world.object_evidence.details[0].mapped_sprite,
            Some(SpriteId::ENEMY_LANDER)
        );
    }

    #[test]
    fn clean_game_lander_reserve_without_humans_uses_source_schizoid_fallback() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();
        game.state.world.humans.clear();
        game.state.world.enemy_reserve = EnemyReserveSnapshot {
            landers: 2,
            ..EnemyReserveSnapshot::default()
        };
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x20,
            hseed: 0x66,
            lseed: 0x99,
        };
        game.baiter_timer_ticks = None;

        let mut expected_rng = game.state.world.source_rng;
        let expected_mutants =
            super::source_mutant_restore_spawns(&mut expected_rng, game.state.wave_profile, 0, 2);

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemies, expected_mutants);
        assert!(frame.state.world.enemies.iter().all(|enemy| {
            enemy.kind == EnemyKind::Mutant
                && enemy.source_lander.is_none()
                && enemy.source_mutant.is_some()
                && enemy.source_bomber.is_none()
                && enemy.source_swarmer.is_none()
                && enemy.source_baiter.is_none()
                && enemy.source_pod.is_none()
        }));
        assert_eq!(frame.state.world.source_rng, expected_rng);
        assert_eq!(
            frame.state.world.enemy_reserve,
            EnemyReserveSnapshot::default()
        );
        assert!(frame.state.world.terrain_blow.is_some());
        assert!(frame.events.gameplay().is_empty());
        assert_eq!(
            frame.state.world.object_evidence.details[0].object_category,
            Some(ObjectEvidenceCategory::Mutant)
        );
        assert_eq!(
            frame.state.world.object_evidence.details[0].mapped_sprite,
            Some(SpriteId::ENEMY_MUTANT)
        );
    }

    #[test]
    fn clean_game_bomber_reserve_batch_uses_source_tiest_restore_state() {
        let mut game = credited_started_game();
        game.state.wave = 2;
        game.state.wave_profile = WaveProfileSnapshot::for_wave(2);
        game.state.player.position = (super::world_word(0x2200), super::world_word(0x8000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies.clear();
        game.state.world.enemy_reserve = EnemyReserveSnapshot {
            bombers: 5,
            ..EnemyReserveSnapshot::default()
        };
        game.baiter_timer_ticks = None;

        let expected_motion = PlayerMotionSystem::step(
            PlayerMotionState::new(
                game.state.player.position,
                game.state.player.velocity,
                game.state.player.direction,
                game.camera_left,
            ),
            PlayerControlIntent::from_input(GameInput::NONE),
        );
        let player_absolute_x =
            super::source_word_from_world_vector(expected_motion.state.position.0);
        let expected_bombers =
            super::source_bomber_restore_spawns(game.state.wave_profile, player_absolute_x, 5);

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemies, expected_bombers);
        assert!(frame.state.world.enemies.iter().all(|enemy| {
            enemy.kind == EnemyKind::Bomber
                && enemy.position.y == super::SOURCE_BOMBER_CRUISE_ALTITUDE
                && enemy.source_bomber.is_some()
                && enemy.source_mutant.is_none()
                && enemy.source_swarmer.is_none()
                && enemy.source_baiter.is_none()
                && enemy.source_pod.is_none()
        }));
        let source_bombers = frame
            .state
            .world
            .enemies
            .iter()
            .filter_map(|enemy| enemy.source_bomber)
            .collect::<Vec<_>>();
        assert_eq!(source_bombers.len(), 5);
        assert_eq!(
            source_bombers[0].x_velocity,
            super::source_sign_extend_u8_to_u16(game.state.wave_profile.bomber_x_velocity)
        );
        assert_eq!(
            source_bombers[3].x_velocity,
            super::source_sign_extend_u8_to_u16(game.state.wave_profile.bomber_x_velocity)
        );
        assert_eq!(
            source_bombers[4].x_velocity,
            super::source_sign_extend_u8_to_u16(
                0u8.wrapping_sub(game.state.wave_profile.bomber_x_velocity)
            )
        );
        assert_eq!(
            source_bombers
                .iter()
                .map(|source_bomber| source_bomber.source_slot)
                .collect::<Vec<_>>(),
            vec![3, 2, 1, 0, 0]
        );
        assert_eq!(
            frame.state.world.enemy_reserve,
            EnemyReserveSnapshot::default()
        );
        assert!(frame.events.gameplay().is_empty());
        assert_eq!(frame.state.world.object_evidence.inactive_count, 0);
        assert_eq!(
            frame.state.world.object_evidence.details[0].object_category,
            Some(ObjectEvidenceCategory::Bomber)
        );
        assert_eq!(
            frame.state.world.object_evidence.details[0].mapped_sprite,
            Some(SpriteId::ENEMY_BOMBER)
        );
    }

    #[test]
    fn clean_game_pod_reserve_batch_uses_source_probe_restore_state() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();
        game.state.world.enemy_reserve = EnemyReserveSnapshot {
            pods: 2,
            ..EnemyReserveSnapshot::default()
        };
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x12,
            hseed: 0x6D,
            lseed: 0x80,
        };
        game.baiter_timer_ticks = None;

        let mut expected_rng = game.state.world.source_rng;
        let expected_pods = (0..2)
            .map(|_| super::source_pod_restore_spawn(&mut expected_rng))
            .collect::<Vec<_>>();

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemies, expected_pods);
        assert!(
            frame
                .state
                .world
                .enemies
                .iter()
                .all(|enemy| enemy.kind == EnemyKind::Pod
                    && enemy.source_pod.is_some()
                    && enemy.source_lander.is_none()
                    && enemy.source_mutant.is_none()
                    && enemy.source_bomber.is_none()
                    && enemy.source_swarmer.is_none()
                    && enemy.source_baiter.is_none())
        );
        assert_eq!(frame.state.world.source_rng, expected_rng);
        assert_eq!(
            frame.state.world.enemy_reserve,
            EnemyReserveSnapshot::default()
        );
        assert!(frame.events.gameplay().is_empty());
        assert_eq!(frame.state.world.object_evidence.inactive_count, 0);
        assert_eq!(
            frame.state.world.object_evidence.details[0].object_category,
            Some(ObjectEvidenceCategory::Pod)
        );
        assert_eq!(
            frame.state.world.object_evidence.details[0].mapped_sprite,
            Some(SpriteId::ENEMY_POD)
        );
    }

    #[test]
    fn clean_game_swarmer_reserve_batch_uses_source_plres_restore_state() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();
        game.state.world.enemy_reserve = EnemyReserveSnapshot {
            swarmers: 4,
            ..EnemyReserveSnapshot::default()
        };
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x20,
            hseed: 0x41,
            lseed: 0xC0,
        };
        game.baiter_timer_ticks = None;

        let mut expected_rng = game.state.world.source_rng;
        let expected_swarmers = super::source_mini_swarmer_reserve_spawns(
            &mut expected_rng,
            game.state.wave_profile,
            4,
        );

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemies, expected_swarmers);
        assert!(frame.state.world.enemies.iter().all(|enemy| {
            enemy.kind == EnemyKind::Swarmer
                && enemy.source_swarmer.is_some()
                && enemy.source_mutant.is_none()
                && enemy.source_bomber.is_none()
                && enemy.source_baiter.is_none()
                && enemy.source_pod.is_none()
        }));
        assert!(
            frame
                .state
                .world
                .enemies
                .iter()
                .all(|enemy| enemy.position == frame.state.world.enemies[0].position)
        );
        assert!(
            frame
                .state
                .world
                .enemies
                .iter()
                .filter_map(|enemy| enemy.source_swarmer)
                .all(
                    |source| source.x_fraction == super::SOURCE_MINI_SWARMER_RESTORE_X_LOW
                        && source.y_fraction == 0
                )
        );
        assert_eq!(frame.state.world.source_rng, expected_rng);
        assert_eq!(
            frame.state.world.enemy_reserve,
            EnemyReserveSnapshot::default()
        );
        assert!(frame.events.gameplay().is_empty());
        assert_eq!(frame.state.world.object_evidence.inactive_count, 0);
        assert_eq!(
            frame.state.world.object_evidence.details[0].object_category,
            Some(ObjectEvidenceCategory::Swarmer)
        );
        assert_eq!(
            frame.state.world.object_evidence.details[0].mapped_sprite,
            Some(SpriteId::ENEMY_SWARMER)
        );
    }

    #[test]
    fn clean_game_enemy_projectile_uses_source_lifetime_and_bounds() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(200, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;
        let mut projectile =
            super::EnemyProjectileSnapshot::source_fireball(ScreenPosition::new(80, 80), 0x0100, 0);
        projectile.source_lifetime_ticks = 1;
        game.state.world.enemy_projectiles = vec![projectile];

        let expired = game.step(GameInput::NONE);

        assert!(expired.state.world.enemy_projectiles.is_empty());
        assert!(expired.events.gameplay().is_empty());

        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(200, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;
        game.state.world.enemy_projectiles = vec![super::EnemyProjectileSnapshot::source_fireball(
            ScreenPosition::new(0x97, 80),
            0x0100,
            0,
        )];

        let offscreen = game.step(GameInput::NONE);

        assert!(offscreen.state.world.enemy_projectiles.is_empty());
        assert!(offscreen.events.gameplay().is_empty());
    }

    #[test]
    fn clean_game_enemy_projectile_wraps_zero_lifetime_like_shscan() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(200, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;
        let mut projectile =
            super::EnemyProjectileSnapshot::source_fireball(ScreenPosition::new(80, 80), 0x0100, 0);
        projectile.source_lifetime_ticks = 0;
        game.state.world.enemy_projectiles = vec![projectile];

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemy_projectiles.len(), 1);
        let projectile = frame.state.world.enemy_projectiles[0];
        assert_eq!(projectile.source_lifetime_ticks, 0xFF);
        assert_eq!(projectile.position, ScreenPosition::new(81, 80));
        assert!(frame.events.gameplay().is_empty());
    }

    #[test]
    fn clean_game_enemy_projectile_applies_source_shell_scroll_delta() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x8000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.player.direction = Direction::Right;
        game.camera_left = WorldVector::default();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(200, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans.clear();
        game.baiter_timer_ticks = None;
        game.state.world.enemy_projectiles = vec![super::EnemyProjectileSnapshot::source_fireball(
            ScreenPosition::new(0x50, 0x60),
            0,
            0,
        )];

        let frame = game.step(GameInput {
            thrust: true,
            ..GameInput::NONE
        });

        assert_eq!(
            super::source_word_from_world_vector(frame.state.player.position.0),
            0x2000
        );
        assert_eq!(
            super::source_word_from_world_vector(game.camera_left),
            0x0003
        );
        assert_eq!(frame.state.world.enemy_projectiles.len(), 1);
        let projectile = frame.state.world.enemy_projectiles[0];
        assert_eq!(projectile.position, ScreenPosition::new(0x4F, 0x60));
        assert_eq!(projectile.source_x_fraction, 0xF4);
        assert_eq!(projectile.source_y_fraction, 0);
        assert_eq!(
            projectile.source_lifetime_ticks,
            super::SOURCE_SHELL_LIFETIME_TICKS - 1
        );
        assert_eq!(projectile.source_x_velocity, 0);
        assert_eq!(projectile.source_y_velocity, 0);
        assert_eq!(frame.state.world.object_evidence.projectile_count, 1);
    }

    #[test]
    fn clean_game_hyperspace_clears_source_enemy_projectiles() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.projectiles = vec![ProjectileSnapshot {
            position: ScreenPosition::new(64, 80),
            velocity: ScreenVelocity::new(0, 0),
        }];
        game.state.world.enemy_projectiles = vec![
            super::EnemyProjectileSnapshot::source_fireball(ScreenPosition::new(80, 80), 0, 0),
            super::EnemyProjectileSnapshot::source_fireball(ScreenPosition::new(96, 88), 0, 0),
        ];
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(frame.events.gameplay(), &[GameEvent::HyperspacePressed]);
        assert!(frame.state.world.enemy_projectiles.is_empty());
        assert_eq!(frame.state.world.projectiles.len(), 1);
        assert_eq!(
            frame.state.world.projectiles[0].position,
            ScreenPosition::new(64, 80)
        );
        assert_eq!(frame.state.world.object_evidence.projectile_count, 1);
        assert_eq!(
            frame.events.sounds(),
            &[source_hyperspace_appearance_sound_event()]
        );
    }

    #[test]
    fn clean_game_hyperspace_uses_source_rematerialization_state() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2345), super::world_word(0x8134));
        game.state.player.velocity = (super::world_word(0x0100), super::world_word(0xFF00));
        game.state.player.direction = Direction::Right;
        game.camera_left = super::world_word(0x2222);
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x12,
            hseed: 0x6C,
            lseed: 0xC0,
        };
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(
            frame.state.player.position,
            (super::world_word(0x7000), super::world_word(0x6034))
        );
        assert_eq!(
            frame.state.player.velocity,
            (WorldVector::default(), WorldVector::default())
        );
        assert_eq!(frame.state.player.direction, Direction::Left);
        assert_eq!(
            super::source_word_from_world_vector(game.camera_left),
            0x126C
        );
        assert_eq!(frame.events.gameplay(), &[GameEvent::HyperspacePressed]);
        assert_eq!(
            frame.events.sounds(),
            &[source_hyperspace_appearance_sound_event()]
        );
    }

    #[test]
    fn clean_game_hyperspace_lseed_high_enters_source_death_path() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2345), super::world_word(0x8134));
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0x12,
            hseed: 0x6C,
            lseed: 0xC1,
        };
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(frame.state.phase, GamePhase::GameOver);
        assert_eq!(frame.state.game_over, GameOverSnapshot::NONE);
        assert_eq!(frame.state.player.lives, 1);
        assert_eq!(
            frame.state.player.position,
            (super::world_word(0x7000), super::world_word(0x6034))
        );
        assert_eq!(
            frame.state.player.velocity,
            (WorldVector::default(), WorldVector::default())
        );
        assert_eq!(
            frame.events.gameplay(),
            &[GameEvent::HyperspacePressed, GameEvent::PlayerDestroyed]
        );
        assert_eq!(
            frame.events.sounds(),
            &[
                source_hyperspace_appearance_sound_event(),
                source_player_death_sound_event()
            ]
        );
        assert!(frame.state.world.player_explosion.is_some());
    }

    #[test]
    fn clean_game_enemy_projectile_evidence_uses_source_shell_picture() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();
        game.state.world.humans.clear();
        game.state.world.projectiles.clear();
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;
        game.state.world.enemy_projectiles = vec![
            super::EnemyProjectileSnapshot::source_fireball(ScreenPosition::new(0x20, 0x60), 0, 0),
            super::EnemyProjectileSnapshot::source_fireball(ScreenPosition::new(0x30, 0x70), 0, 0),
        ];

        game.sync_world_presentation();

        let details = &game.state.world.object_evidence.details;
        assert_eq!(game.state.world.object_evidence.projectile_count, 2);
        assert_eq!(game.state.world.object_evidence.detail_count, 2);
        for (index, position) in [
            ScreenPosition::new(0x20, 0x60),
            ScreenPosition::new(0x30, 0x70),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(details[index].list, ObjectEvidenceList::Projectile);
            assert_eq!(
                details[index].object_category,
                Some(ObjectEvidenceCategory::EnemyBomb)
            );
            assert_eq!(details[index].screen_position, Some(position));
            assert_eq!(
                details[index].picture_address,
                Some(super::SOURCE_BOMB_SHELL_PICTURE_ADDRESS)
            );
            assert_eq!(
                details[index].picture_label,
                Some(super::SOURCE_BOMB_SHELL_PICTURE_LABEL)
            );
            assert_eq!(
                details[index].picture_size,
                Some(super::SOURCE_BOMB_SHELL_PICTURE_SIZE)
            );
            assert_eq!(
                details[index].primary_image_address,
                Some(super::SOURCE_BOMB_SHELL_PRIMARY_IMAGE_ADDRESS)
            );
            assert_eq!(
                details[index].alternate_image_address,
                Some(super::SOURCE_BOMB_SHELL_ALTERNATE_IMAGE_ADDRESS)
            );
            assert_eq!(details[index].mapped_sprite, Some(SpriteId::ENEMY_BOMB));
        }

        let scene = game.scene();
        let shell_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| {
                sprite.sprite == SpriteId::ENEMY_BOMB && sprite.layer == RenderLayer::Projectiles
            })
            .collect::<Vec<_>>();
        assert_eq!(shell_sprites.len(), 2);
        assert!(
            shell_sprites
                .iter()
                .any(|sprite| { sprite.position == [32.0, 96.0] && sprite.size == [4.0, 6.0] })
        );
        assert!(
            shell_sprites
                .iter()
                .any(|sprite| { sprite.position == [48.0, 112.0] && sprite.size == [4.0, 6.0] })
        );
    }

    #[test]
    fn clean_game_player_projectile_evidence_uses_source_laser_picture() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();
        game.state.world.humans.clear();
        game.state.world.enemy_projectiles.clear();
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;
        game.state.world.projectiles = vec![ProjectileSnapshot {
            position: ScreenPosition::new(0x34, 0x78),
            velocity: ScreenVelocity::new(5, 0),
        }];

        game.sync_world_presentation();

        assert_eq!(game.state.world.object_evidence.projectile_count, 1);
        assert_eq!(game.state.world.object_evidence.detail_count, 1);
        let detail = game.state.world.object_evidence.details[0];
        assert_eq!(detail.list, ObjectEvidenceList::Projectile);
        assert_eq!(
            detail.object_category,
            Some(ObjectEvidenceCategory::PlayerProjectile)
        );
        assert_eq!(
            detail.screen_position,
            Some(ScreenPosition::new(0x34, 0x78))
        );
        assert_eq!(detail.picture_address, Some(0xF96F));
        assert_eq!(detail.picture_label, Some("LASP1"));
        assert_eq!(detail.picture_size, Some((8, 1)));
        assert_eq!(detail.primary_image_address, Some(0xF973));
        assert_eq!(detail.alternate_image_address, None);
        assert_eq!(detail.mapped_sprite, Some(SpriteId::PLAYER_PROJECTILE));

        let scene = game.scene();
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.layer == RenderLayer::Projectiles
                && sprite.position == [52.0, 120.0]
                && sprite.size == [8.0, 2.0]
        }));
    }

    #[test]
    fn clean_game_human_evidence_uses_source_astronaut_picture() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();
        game.state.world.projectiles.clear();
        game.state.world.enemy_projectiles.clear();
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;
        game.state.world.humans = vec![
            HumanSnapshot::new(ScreenPosition::new(0x44, 0xD8)),
            HumanSnapshot {
                source_x_fraction: 0xAD,
                source_picture_frame: 2,
                ..HumanSnapshot::new(ScreenPosition::new(0x12, 0xE0))
            },
        ];

        game.sync_world_presentation();

        assert_eq!(game.state.world.object_evidence.active_count, 2);
        assert_eq!(game.state.world.object_evidence.detail_count, 2);
        let detail = game.state.world.object_evidence.details[0];
        assert_eq!(detail.list, ObjectEvidenceList::Active);
        assert_eq!(detail.object_category, Some(ObjectEvidenceCategory::Human));
        assert_eq!(
            detail.screen_position,
            Some(ScreenPosition::new(0x44, 0xD8))
        );
        assert_eq!(detail.picture_address, Some(0xF901));
        assert_eq!(detail.picture_label, Some("ASTP1"));
        assert_eq!(detail.picture_size, Some((2, 8)));
        assert_eq!(detail.primary_image_address, Some(0xFACB));
        assert_eq!(detail.alternate_image_address, Some(0xFADB));
        assert_eq!(detail.mapped_sprite, Some(SpriteId::HUMAN));
        assert_eq!(
            detail.scanner_color,
            Some(super::SOURCE_SCANNER_HUMAN_COLOR_WORD)
        );
        let restored_detail = game.state.world.object_evidence.details[1];
        assert_eq!(restored_detail.world_position, Some((0x12AD, 0xE000)));
        assert_eq!(restored_detail.picture_address, Some(0xF915));
        assert_eq!(restored_detail.picture_label, Some("ASTP3"));
        assert_eq!(restored_detail.picture_size, Some((2, 8)));
        assert_eq!(restored_detail.primary_image_address, Some(0xFB0B));
        assert_eq!(restored_detail.alternate_image_address, Some(0xFB1B));
        assert_eq!(restored_detail.mapped_sprite, Some(SpriteId::HUMAN));

        let scene = game.scene();
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [68.0, 216.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(!scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [68.0, 216.0]
                && sprite.size == [2.0, 8.0]
        }));
    }

    #[test]
    fn clean_world_object_evidence_carries_source_motion_words() {
        let mut enemy_projectile = super::EnemyProjectileSnapshot::source_fireball(
            ScreenPosition::new(0x30, 0x80),
            0x0100,
            0xFF00,
        );
        enemy_projectile.source_x_fraction = 0x05;
        enemy_projectile.source_y_fraction = 0x06;
        let mut world = WorldSnapshot {
            enemies: vec![EnemySnapshot::source_lander(
                ScreenPosition::new(0x40, 0x60),
                ScreenVelocity::new(0, 0),
                SourceLanderSnapshot {
                    x_fraction: 0x12,
                    y_fraction: 0x34,
                    x_velocity: 0xFFE0,
                    y_velocity: 0x0010,
                    shot_timer: 0,
                    sleep_ticks: 0,
                    picture_frame: 0,
                    target_human_index: None,
                },
            )],
            humans: vec![HumanSnapshot {
                source_x_fraction: 0x33,
                source_fall_velocity: 0x00E0,
                source_fall_y_fraction: 0x44,
                ..HumanSnapshot::new(ScreenPosition::new(0x22, 0xD0))
            }],
            projectiles: vec![ProjectileSnapshot {
                position: ScreenPosition::new(0x20, 0x70),
                velocity: ScreenVelocity::new(5, 0),
            }],
            enemy_projectiles: vec![enemy_projectile],
            ..WorldSnapshot::default()
        };

        world.refresh_object_evidence();

        assert_eq!(world.object_evidence.detail_count, 4);
        let details = &world.object_evidence.details;
        for (index, detail) in details[..4].iter().enumerate() {
            let identity = super::source_object_table_identity(index);
            assert_eq!(detail.address, Some(identity.address));
            assert_eq!(detail.slot, Some(identity.slot));
            assert_eq!(detail.object_type, Some(identity.object_type));
        }
        assert_eq!(
            details[0].object_category,
            Some(ObjectEvidenceCategory::Lander)
        );
        assert_eq!(details[0].world_position, Some((0x4012, 0x6034)));
        assert_eq!(details[0].velocity, Some((0xFFE0, 0x0010)));
        assert_eq!(
            details[1].object_category,
            Some(ObjectEvidenceCategory::Human)
        );
        assert_eq!(details[1].world_position, Some((0x2233, 0xD044)));
        assert_eq!(details[1].velocity, Some((0x0000, 0x00E0)));
        assert_eq!(
            details[2].object_category,
            Some(ObjectEvidenceCategory::PlayerProjectile)
        );
        assert_eq!(details[2].world_position, Some((0x2000, 0x7000)));
        assert_eq!(details[2].velocity, Some((0x0500, 0x0000)));
        assert_eq!(
            details[3].object_category,
            Some(ObjectEvidenceCategory::EnemyBomb)
        );
        assert_eq!(details[3].world_position, Some((0x3005, 0x8006)));
        assert_eq!(details[3].velocity, Some((0x0100, 0xFF00)));
    }

    #[test]
    fn clean_game_enemy_projectile_collision_scores_and_destroys_player() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x8000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(200, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.enemy_projectiles = vec![super::EnemyProjectileSnapshot::source_fireball(
            ScreenPosition::new(0x20, 0x80),
            0,
            0,
        )];
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.enemy_projectiles.is_empty());
        assert_eq!(frame.state.scores.player_one, 25);
        assert_eq!(frame.state.player.lives, 1);
        assert_eq!(frame.events.gameplay(), &[GameEvent::PlayerDestroyed]);
        assert_eq!(
            frame.events.sounds(),
            &[
                source_bomb_collision_sound_event(),
                source_player_death_sound_event()
            ]
        );
        assert_eq!(frame.state.world.expanded_objects.active_count, 1);
        assert_eq!(
            frame.state.world.expanded_objects.details[0].mapped_sprite,
            Some(SpriteId::BOMB_EXPLOSION)
        );
        assert!(frame.state.world.player_explosion.is_some());
    }

    #[test]
    fn clean_game_enemy_projectile_uses_source_bmbp1_collision_footprint() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x8000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(200, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.enemy_projectiles = vec![super::EnemyProjectileSnapshot::source_fireball(
            ScreenPosition::new(0x1D, 0x7D),
            0,
            0,
        )];
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemy_projectiles.len(), 1);
        assert_eq!(
            frame.state.world.enemy_projectiles[0].position,
            ScreenPosition::new(0x1D, 0x7D)
        );
        assert_eq!(frame.state.scores.player_one, 0);
        assert_eq!(frame.state.player.lives, 2);
        assert!(frame.events.gameplay().is_empty());
        assert!(frame.events.sounds().is_empty());
        assert!(frame.state.world.player_explosion.is_none());
    }

    #[test]
    fn clean_game_enemy_projectile_uses_source_player_collision_footprint() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x8000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(200, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.enemy_projectiles = vec![super::EnemyProjectileSnapshot::source_fireball(
            ScreenPosition::new(0x28, 0x82),
            0,
            0,
        )];
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemy_projectiles.len(), 1);
        assert_eq!(
            frame.state.world.enemy_projectiles[0].position,
            ScreenPosition::new(0x28, 0x82)
        );
        assert_eq!(frame.state.scores.player_one, 0);
        assert_eq!(frame.state.player.lives, 2);
        assert!(frame.events.gameplay().is_empty());
        assert!(frame.events.sounds().is_empty());
        assert!(frame.state.world.player_explosion.is_none());
    }

    #[test]
    fn clean_game_lander_abducts_aligned_human_and_carries_upward() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(100, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot::new(ScreenPosition::new(100, 92))];

        let captured = game.step(GameInput::NONE);

        assert_eq!(
            captured.state.world.enemies[0].velocity,
            ScreenVelocity::new(0, -1)
        );
        assert_eq!(
            captured.state.world.humans[0],
            HumanSnapshot {
                carried: true,
                ..HumanSnapshot::new(super::clean_carried_human_position(ScreenPosition::new(
                    100, 80
                ),))
            }
        );
        assert_eq!(captured.state.world.object_evidence.active_count, 2);
        assert!(captured.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [102.0, 92.0]
                && sprite.tint == Color::from_rgba(0xFF, 0xF8, 0x80, 0xFF)
        }));
        assert_eq!(
            captured.events.sounds(),
            &[source_lander_pickup_sound_event()]
        );

        let carried = game.step(GameInput::NONE);

        assert_eq!(
            carried.state.world.enemies[0].position,
            ScreenPosition::new(100, 79)
        );
        assert_eq!(
            carried.state.world.humans[0].position,
            super::clean_carried_human_position(ScreenPosition::new(100, 79))
        );
        assert!(carried.state.world.humans[0].carried);
    }

    #[test]
    fn clean_game_source_lander_capture_seeds_source_flee_state() {
        let mut game = credited_started_game();
        let lander_start = ScreenPosition::new(100, 80);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 5,
            sleep_ticks: 1,
            picture_frame: 0,
            target_human_index: Some(0),
        };
        game.state.world.enemies = vec![EnemySnapshot::source_lander(
            lander_start,
            ScreenVelocity::new(0, 0),
            source_lander,
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot::new(ScreenPosition::new(100, 92))];
        game.baiter_timer_ticks = None;

        let expected_source = SourceLanderSnapshot {
            y_velocity: !super::source_lander_base_y_velocity(game.state.wave_profile),
            sleep_ticks: super::SOURCE_LANDER_FLEE_SLEEP_TICKS,
            ..source_lander
        };

        let captured = game.step(GameInput::NONE);

        let lander = captured
            .state
            .world
            .enemies
            .first()
            .expect("capturing source lander");
        assert_eq!(lander.position, lander_start);
        assert_eq!(
            lander.velocity,
            super::source_lander_screen_velocity(expected_source)
        );
        assert_eq!(lander.source_lander, Some(expected_source));
        assert_eq!(
            captured.state.world.humans[0],
            HumanSnapshot {
                carried: true,
                ..HumanSnapshot::new(super::clean_carried_human_position(lander_start))
            }
        );
        assert!(captured.state.world.enemy_projectiles.is_empty());
        assert_eq!(
            captured.events.sounds(),
            &[source_lander_pickup_sound_event()]
        );
    }

    #[test]
    fn clean_game_completed_lander_abduction_spawns_source_mutant() {
        let mut game = credited_started_game();
        let lander_start = ScreenPosition::new(100, SOURCE_PLAYFIELD_Y_MIN + 9);
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            lander_start,
            ScreenVelocity::new(0, -1),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot {
            carried: true,
            ..HumanSnapshot::new(super::clean_carried_human_position(lander_start))
        }];
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0,
            hseed: 0xA5,
            lseed: 0x5A,
        };
        game.baiter_timer_ticks = None;

        let player_position = ScreenPosition::new(0x20, 0x80);
        let object_position_after_motion = ScreenPosition::new(100, SOURCE_PLAYFIELD_Y_MIN + 8);
        let object_absolute_x = u16::from(object_position_after_motion.x) << 8;
        let player_absolute_x = u16::from(player_position.x) << 8;
        let expected_x_velocity = super::source_mutant_x_velocity(
            game.state.wave_profile.mutant_x_velocity,
            player_absolute_x,
            object_absolute_x,
        );
        let expected_y_velocity = super::source_mutant_y_velocity(
            game.state.wave_profile,
            player_position.y,
            player_absolute_x,
            object_absolute_x,
            object_position_after_motion,
        );
        let expected_position = ScreenPosition::new(
            object_position_after_motion.x,
            object_position_after_motion
                .y
                .wrapping_sub(game.state.wave_profile.mutant_random_y),
        );
        let expected_source = SourceMutantSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: expected_x_velocity,
            y_velocity: expected_y_velocity,
            shot_timer: game.state.wave_profile.mutant_shot_time as u8 - 1,
            sleep_ticks: SOURCE_MUTANT_LOOP_SLEEP_TICKS,
        };

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.humans.is_empty());
        let mutant = frame.state.world.enemies.first().expect("converted mutant");
        assert_eq!(mutant.kind, EnemyKind::Mutant);
        assert_eq!(mutant.position, expected_position);
        assert_eq!(
            mutant.velocity,
            super::source_mutant_screen_velocity(expected_source)
        );
        assert_eq!(mutant.source_mutant, Some(expected_source));
        assert!(frame.state.world.enemy_projectiles.is_empty());
        assert!(frame.state.world.terrain_blow.is_some());
    }

    #[test]
    fn clean_game_source_lander_pulls_passenger_at_top_edge() {
        let mut game = credited_started_game();
        let lander_start = ScreenPosition::new(100, SOURCE_PLAYFIELD_Y_MIN + 8);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: !super::source_lander_base_y_velocity(game.state.wave_profile),
            shot_timer: 1,
            sleep_ticks: 0,
            picture_frame: 0,
            target_human_index: Some(0),
        };
        game.state.world.enemies = vec![EnemySnapshot::source_lander(
            lander_start,
            super::source_lander_screen_velocity(source_lander),
            source_lander,
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot {
            carried: true,
            ..HumanSnapshot::new(super::clean_carried_human_position(lander_start))
        }];
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        let lander = frame.state.world.enemies.first().expect("pulling lander");
        assert_eq!(lander.kind, EnemyKind::Lander);
        assert_eq!(lander.position, lander_start);
        assert_eq!(
            lander.source_lander,
            Some(SourceLanderSnapshot {
                y_velocity: 0,
                ..source_lander
            })
        );
        assert_eq!(
            frame.state.world.humans[0],
            HumanSnapshot {
                carried: true,
                ..HumanSnapshot::new(ScreenPosition::new(
                    super::clean_carried_human_position(lander_start).x,
                    super::clean_carried_human_position(lander_start)
                        .y
                        .saturating_sub(1),
                ))
            }
        );
        assert!(frame.state.world.enemy_projectiles.is_empty());
        assert_eq!(frame.events.sounds(), &[source_lander_suck_sound_event()]);
    }

    #[test]
    fn clean_game_source_lander_pull_sound_does_not_repeat() {
        let mut game = credited_started_game();
        let lander_start = ScreenPosition::new(100, SOURCE_PLAYFIELD_Y_MIN + 8);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: 0,
            picture_frame: 0,
            target_human_index: Some(0),
        };
        game.state.world.enemies = vec![EnemySnapshot::source_lander(
            lander_start,
            ScreenVelocity::new(0, 0),
            source_lander,
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot {
            carried: true,
            ..HumanSnapshot::new(ScreenPosition::new(
                super::clean_carried_human_position(lander_start).x,
                super::clean_carried_human_position(lander_start)
                    .y
                    .saturating_sub(1),
            ))
        }];
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        let lander = frame.state.world.enemies.first().expect("pulling lander");
        assert_eq!(lander.position, lander_start);
        assert_eq!(lander.source_lander, Some(source_lander));
        assert_eq!(
            frame.state.world.humans[0].position,
            ScreenPosition::new(
                super::clean_carried_human_position(lander_start).x,
                super::clean_carried_human_position(lander_start)
                    .y
                    .saturating_sub(2),
            )
        );
        assert!(frame.events.sounds().is_empty());
    }

    #[test]
    fn clean_game_source_lander_converts_after_passenger_pulled_inside() {
        let mut game = credited_started_game();
        let lander_start = ScreenPosition::new(100, SOURCE_PLAYFIELD_Y_MIN + 8);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: 0,
            picture_frame: 0,
            target_human_index: Some(0),
        };
        game.state.world.enemies = vec![EnemySnapshot::source_lander(
            lander_start,
            ScreenVelocity::new(0, 0),
            source_lander,
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot {
            carried: true,
            ..HumanSnapshot::new(ScreenPosition::new(
                super::clean_carried_human_position(lander_start).x,
                lander_start.y,
            ))
        }];
        game.state.world.source_rng = SourceRandSnapshot {
            seed: 0,
            hseed: 0xA5,
            lseed: 0x5A,
        };
        game.baiter_timer_ticks = None;

        let player_position = ScreenPosition::new(0x20, 0x80);
        let object_absolute_x = u16::from(lander_start.x) << 8;
        let player_absolute_x = u16::from(player_position.x) << 8;
        let expected_x_velocity = super::source_mutant_x_velocity(
            game.state.wave_profile.mutant_x_velocity,
            player_absolute_x,
            object_absolute_x,
        );
        let expected_y_velocity = super::source_mutant_y_velocity(
            game.state.wave_profile,
            player_position.y,
            player_absolute_x,
            object_absolute_x,
            lander_start,
        );
        let expected_position = ScreenPosition::new(
            lander_start.x,
            lander_start
                .y
                .wrapping_sub(game.state.wave_profile.mutant_random_y),
        );
        let expected_source = SourceMutantSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: expected_x_velocity,
            y_velocity: expected_y_velocity,
            shot_timer: game.state.wave_profile.mutant_shot_time as u8 - 1,
            sleep_ticks: SOURCE_MUTANT_LOOP_SLEEP_TICKS,
        };

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.humans.is_empty());
        let mutant = frame.state.world.enemies.first().expect("converted mutant");
        assert_eq!(mutant.kind, EnemyKind::Mutant);
        assert_eq!(mutant.position, expected_position);
        assert_eq!(
            mutant.velocity,
            super::source_mutant_screen_velocity(expected_source)
        );
        assert_eq!(mutant.source_mutant, Some(expected_source));
        assert!(frame.state.world.enemy_projectiles.is_empty());
        assert!(frame.state.world.terrain_blow.is_some());
    }

    #[test]
    fn clean_game_source_lander_gives_up_when_pull_target_cleared() {
        let mut game = credited_started_game();
        let lander_start = ScreenPosition::new(100, SOURCE_PLAYFIELD_Y_MIN + 8);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: 1,
            picture_frame: 0,
            target_human_index: Some(usize::MAX),
        };
        game.state.world.enemies = vec![
            EnemySnapshot::source_lander(lander_start, ScreenVelocity::new(0, 0), source_lander),
            EnemySnapshot::new(
                EnemyKind::Mutant,
                ScreenPosition::new(220, 80),
                ScreenVelocity::new(0, 0),
            ),
        ];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        keep_single_grounded_human(&mut game);
        game.state.world.source_astronaut_sleep_ticks = 1;
        let humans = game.state.world.humans.clone();
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.enemies.len(), 1);
        assert_eq!(frame.state.world.enemies[0].kind, EnemyKind::Mutant);
        assert_eq!(frame.state.world.enemy_reserve.landers, 1);
        assert_eq!(frame.state.world.humans, humans);
        assert!(frame.state.world.enemy_projectiles.is_empty());
        assert!(frame.state.world.terrain_blow.is_none());
        assert!(frame.events.gameplay().is_empty());
    }

    #[test]
    fn clean_game_killed_carrying_lander_releases_human() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(100, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot {
            carried: true,
            ..HumanSnapshot::new(super::clean_carried_human_position(ScreenPosition::new(
                100, 80,
            )))
        }];
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.enemies.is_empty());
        assert_eq!(
            frame.state.world.humans[0],
            HumanSnapshot::new(super::clean_carried_human_position(ScreenPosition::new(
                100, 80,
            )))
        );
        assert_eq!(frame.state.scores.player_one, 150);
        assert_eq!(
            frame.events.gameplay(),
            &[GameEvent::EnemyDestroyed, GameEvent::WaveCleared]
        );
        assert_eq!(
            frame.events.sounds(),
            &[source_astronaut_release_sound_event()]
        );
    }

    #[test]
    fn clean_game_killed_source_lander_pull_passenger_releases_human() {
        let mut game = credited_started_game();
        let lander_start = ScreenPosition::new(100, SOURCE_PLAYFIELD_Y_MIN + 8);
        let source_lander = SourceLanderSnapshot {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: 0,
            picture_frame: 0,
            target_human_index: Some(0),
        };
        game.state.world.enemies = vec![EnemySnapshot::source_lander(
            lander_start,
            ScreenVelocity::new(0, 0),
            source_lander,
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot {
            carried: true,
            ..HumanSnapshot::new(ScreenPosition::new(
                super::clean_carried_human_position(lander_start).x,
                lander_start.y.saturating_add(2),
            ))
        }];
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, lander_start.y.saturating_add(3)),
            velocity: ScreenVelocity::new(0, 0),
        });
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.enemies.is_empty());
        assert_eq!(
            frame.state.world.humans[0],
            HumanSnapshot::new(ScreenPosition::new(
                super::clean_carried_human_position(lander_start).x,
                lander_start.y.saturating_add(1),
            ))
        );
        assert_eq!(
            frame.events.sounds(),
            &[source_astronaut_release_sound_event()]
        );
    }

    #[test]
    fn clean_game_released_lander_passenger_falls_on_following_frame() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![
            EnemySnapshot::new(
                EnemyKind::Lander,
                ScreenPosition::new(100, 80),
                ScreenVelocity::new(0, 0),
            ),
            EnemySnapshot::new(
                EnemyKind::Baiter,
                ScreenPosition::new(220, 80),
                ScreenVelocity::new(0, 0),
            ),
        ];
        game.state.world.enemy_reserve = EnemyReserveSnapshot {
            landers: 1,
            ..EnemyReserveSnapshot::default()
        };
        let released_position = super::clean_carried_human_position(ScreenPosition::new(100, 80));
        game.state.world.humans = vec![HumanSnapshot {
            carried: true,
            ..HumanSnapshot::new(released_position)
        }];
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });
        game.baiter_timer_ticks = None;

        let released = game.step(GameInput::NONE);

        assert_eq!(released.events.gameplay(), &[GameEvent::EnemyDestroyed]);
        assert_eq!(
            released.state.world.humans[0],
            HumanSnapshot::new(released_position)
        );

        let falling = game.step(GameInput::NONE);

        assert_eq!(
            falling.state.world.humans[0],
            HumanSnapshot {
                source_fall_velocity: super::SOURCE_FALLING_HUMAN_Y_ACCELERATION,
                source_fall_y_fraction: super::SOURCE_FALLING_HUMAN_Y_ACCELERATION as u8,
                ..HumanSnapshot::new(released_position)
            }
        );
    }

    #[test]
    fn clean_game_player_catches_falling_human_scores_and_starts_p500_popup() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x6400), super::world_word(0x6400));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Baiter,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot::new(ScreenPosition::new(104, 98))];
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.scores.player_one, 500);
        assert_eq!(
            frame.state.world.humans[0],
            HumanSnapshot {
                carried_by_player: true,
                ..HumanSnapshot::new(super::clean_player_carried_human_position(
                    ScreenPosition::new(99, 100),
                ))
            }
        );
        assert_eq!(frame.state.world.score_popups.len(), 1);
        assert_eq!(
            frame.state.world.score_popups[0].kind,
            ScorePopupKind::Points500
        );
        assert_eq!(
            frame.state.world.score_popups[0].position,
            super::clean_rescue_score_popup_position(ScreenPosition::new(104, 98))
        );
        assert_eq!(
            frame.events.sounds(),
            &[source_astronaut_catch_sound_event()]
        );
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_POPUP_500
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [104.0, 122.0]
        }));
    }

    #[test]
    fn clean_game_player_rescue_uses_source_collision_footprints() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x6400), super::world_word(0x6400));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot::new(ScreenPosition::new(108, 98))];
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.scores.player_one, 0);
        assert_eq!(frame.state.world.humans.len(), 1);
        assert!(!frame.state.world.humans[0].carried_by_player);
        assert!(frame.state.world.score_popups.is_empty());
        assert_eq!(frame.events.gameplay(), &[]);
        assert!(frame.events.sounds().is_empty());
    }

    #[test]
    fn clean_game_player_does_not_catch_grounded_human() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x6400), super::world_word(0xD600));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Baiter,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        let ground_y =
            super::clean_human_ground_y(&game.state.world.terrain, 100).expect("terrain ground");
        game.state.world.humans = vec![HumanSnapshot::new(ScreenPosition::new(100, ground_y))];
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.scores.player_one, 0);
        assert_eq!(
            frame.state.world.humans[0],
            HumanSnapshot::new(ScreenPosition::new(100, ground_y))
        );
        assert!(frame.state.world.score_popups.is_empty());
    }

    #[test]
    fn clean_game_player_carried_human_lands_when_carried_to_terrain() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x6400), super::world_word(0xCC00));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Baiter,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.humans = vec![HumanSnapshot {
            carried_by_player: true,
            ..HumanSnapshot::new(ScreenPosition::new(100, 180))
        }];
        game.state.scores.player_one = 500;
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        assert_eq!(
            frame.state.world.humans[0],
            HumanSnapshot::new(ScreenPosition::new(99, 214))
        );
        assert_eq!(frame.state.scores.player_one, 500);
        assert!(frame.state.world.score_popups.is_empty());
    }

    #[test]
    fn clean_game_released_human_falls_until_terrain_landing() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;

        let ground_y =
            super::clean_human_ground_y(&game.state.world.terrain, 100).expect("terrain ground");
        game.state.world.humans = vec![HumanSnapshot::new(ScreenPosition::new(
            100,
            ground_y.saturating_sub(1),
        ))];

        let falling = game.step(GameInput::NONE);
        assert_eq!(
            falling.state.world.humans[0],
            HumanSnapshot {
                source_fall_velocity: super::SOURCE_FALLING_HUMAN_Y_ACCELERATION,
                source_fall_y_fraction: super::SOURCE_FALLING_HUMAN_Y_ACCELERATION as u8,
                ..HumanSnapshot::new(ScreenPosition::new(100, ground_y.saturating_sub(1)))
            }
        );

        let mut landing_sounds = None;
        for _ in 0..100 {
            if game.state.world.humans[0].position.y == ground_y {
                break;
            }
            let frame = game.step(GameInput::NONE);
            if frame.state.world.humans[0].position.y == ground_y {
                landing_sounds = Some(frame.events.sounds().to_vec());
            }
        }

        assert_eq!(
            game.state.world.humans[0],
            HumanSnapshot::new(ScreenPosition::new(100, ground_y))
        );
        assert_eq!(game.state.scores.player_one, 250);
        assert_eq!(game.state.world.score_popups.len(), 1);
        assert_eq!(
            game.state.world.score_popups[0].kind,
            ScorePopupKind::Points250
        );
        assert_eq!(
            game.state.world.score_popups[0].position,
            super::clean_rescue_score_popup_position(ScreenPosition::new(100, ground_y))
        );
        assert_eq!(
            landing_sounds.as_deref(),
            Some(&[source_astronaut_safe_landing_sound_event()][..])
        );

        let settled = game.step(GameInput::NONE);
        assert_eq!(settled.state.world.humans[0].position.y, ground_y);
        assert_eq!(settled.state.scores.player_one, 250);
        assert_eq!(settled.state.world.score_popups.len(), 1);
    }

    #[test]
    fn clean_game_fatal_falling_human_impact_removes_human_and_starts_human_loss() {
        let mut game = credited_started_game();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Baiter,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.baiter_timer_ticks = None;

        let ground_y =
            super::clean_human_ground_y(&game.state.world.terrain, 100).expect("terrain ground");
        game.state.world.humans = vec![HumanSnapshot {
            source_fall_velocity: super::SOURCE_FALLING_HUMAN_SAFE_LANDING_MAX_Y_VELOCITY,
            source_fall_y_fraction: 0x40,
            ..HumanSnapshot::new(ScreenPosition::new(100, ground_y.saturating_sub(1)))
        }];

        let frame = game.step(GameInput::NONE);

        assert!(frame.state.world.humans.is_empty());
        assert_eq!(frame.state.scores.player_one, 0);
        assert!(frame.state.world.score_popups.is_empty());
        assert!(frame.state.world.terrain_blow.is_some());
        assert!(frame.state.world.terrain.is_empty());
        assert!(frame.state.world.explosions.iter().any(|explosion| {
            explosion.kind == ExplosionKind::Astronaut
                && explosion.position == ScreenPosition::new(100, ground_y)
        }));
        assert_eq!(
            frame.events.sounds(),
            &[
                source_astronaut_hit_sound_event(),
                source_terrain_blow_start_sound_event()
            ]
        );
    }

    #[test]
    fn clean_game_standing_humans_do_not_fall() {
        let mut game = credited_started_game();
        let humans = game.state.world.humans.clone();

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.world.humans.len(), humans.len());
        for human in &frame.state.world.humans {
            assert!(!human.carried);
            assert!(!human.carried_by_player);
            assert_eq!(human.source_fall_velocity, 0);
            assert_eq!(human.source_fall_y_fraction, 0);
            assert!(
                super::clean_human_ground_y(&frame.state.world.terrain, human.position.x)
                    .is_none_or(|ground_y| human.position.y >= ground_y)
            );
        }
    }

    #[test]
    fn clean_game_player_enemy_collision_loses_life_and_removes_enemy() {
        let mut game = credited_started_game();
        keep_first_enemy_only(&mut game);
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.phase, GamePhase::GameOver);
        assert_eq!(frame.state.game_over, GameOverSnapshot::NONE);
        assert_eq!(frame.state.player.lives, 1);
        assert_eq!(frame.state.player.smart_bombs, 3);
        assert_eq!(frame.state.scores.player_one, 0);
        assert!(frame.state.world.enemies.is_empty());
        assert_eq!(frame.events.gameplay(), &[GameEvent::PlayerDestroyed]);
        assert!(
            !frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER)
        );
        assert!(!frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_SHIP && sprite.layer == RenderLayer::Objects
        }));
        assert_eq!(
            frame.state.world.player_explosion,
            Some(PlayerExplosionCloudSnapshot {
                source_color: 0xFF,
                source_color_counter: 56,
                source_color_index: 0,
                frame: 0,
                piece_count: 0,
                ..PlayerExplosionCloudSnapshot::EMPTY
            })
        );

        game.advance_player_explosion();
        game.sync_world_presentation();
        let cloud = game
            .state
            .world
            .player_explosion
            .expect("player explosion cloud");
        assert_eq!(cloud.source_color, 0xFF);
        assert_eq!(cloud.source_color_counter, 55);
        assert_eq!(cloud.source_color_index, 0);
        assert_eq!(cloud.frame, 1);
        assert!(cloud.piece_count > 0);
        assert!(game.scene().sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                && sprite.layer == RenderLayer::Objects
                && sprite.tint == Color::WHITE
        }));

        let respawn = advance_pending_respawn(&mut game);
        assert_eq!(respawn.state.phase, GamePhase::Playing);
        assert_eq!(respawn.state.current_player, 1);
        assert_eq!(respawn.state.player.lives, 1);
        assert_eq!(respawn.state.world, WorldSnapshot::default());
        assert_eq!(
            game.start_playfield_delay,
            Some(START_PLAYFIELD_DELAY_FRAMES)
        );

        let active = advance_to_started_playfield(&mut game);
        assert_eq!(active.state.phase, GamePhase::Playing);
        assert_eq!(active.state.current_player, 1);
        assert_eq!(active.state.player.lives, 0);
        assert_eq!(
            active.state.player_stocks[0],
            PlayerStockSnapshot::new(0, 3)
        );
        assert_eq!(active.state.world.enemies.len(), 5);
    }

    #[test]
    fn clean_game_player_enemy_uses_source_enemy_collision_width() {
        let mut game = credited_started_game();
        keep_first_enemy_only(&mut game);
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x8000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies[0] = EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(21, 128),
            ScreenVelocity::new(0, 0),
        );

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.phase, GamePhase::Playing);
        assert_eq!(frame.state.player.lives, 2);
        assert_eq!(frame.state.world.enemies.len(), 1);
        assert!(frame.state.world.player_explosion.is_none());
        assert!(frame.events.gameplay().is_empty());
        assert!(frame.events.sounds().is_empty());
    }

    #[test]
    fn clean_game_player_enemy_uses_source_player_collision_footprint() {
        let mut game = credited_started_game();
        keep_first_enemy_only(&mut game);
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x8000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies[0] = EnemySnapshot::new(
            EnemyKind::Lander,
            ScreenPosition::new(40, 128),
            ScreenVelocity::new(0, 0),
        );

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.phase, GamePhase::Playing);
        assert_eq!(frame.state.player.lives, 2);
        assert_eq!(frame.state.world.enemies.len(), 1);
        assert!(frame.state.world.player_explosion.is_none());
        assert!(frame.events.gameplay().is_empty());
        assert!(frame.events.sounds().is_empty());
    }

    #[test]
    fn clean_game_player_enemy_final_collision_enters_game_over() {
        let mut game = credited_started_game();
        keep_first_enemy_only(&mut game);
        game.state.player.lives = 1;
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.phase, GamePhase::GameOver);
        assert_eq!(frame.state.player.lives, 0);
        assert_eq!(frame.state.player.smart_bombs, 3);
        assert!(frame.state.world.enemies.is_empty());
        assert_eq!(
            frame.state.game_over,
            GameOverSnapshot::player_death_sleep(PLAYER_DEATH_GAME_OVER_SLEEP_FRAMES)
        );
        assert_eq!(
            frame.events.gameplay(),
            &[GameEvent::PlayerDestroyed, GameEvent::GameOver]
        );
        assert_eq!(frame.events.sounds(), &[source_player_death_sound_event()]);
        assert!(!frame.scene.sprites.iter().any(|sprite| matches!(
            sprite.sprite,
            SpriteId::PLAYER_SHIP | SpriteId::ENEMY_LANDER | SpriteId::HUMAN
        )));
        assert_eq!(frame.scene.summary().layers.objects, 0);
        assert_eq!(frame.scene.summary().layers.overlay, 8);
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_G
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [124.0, 128.0]
                && sprite.size == [6.0, 8.0]
        }));
    }

    #[test]
    fn clean_game_two_player_final_death_waits_then_switches_to_other_player() {
        let mut game = two_player_started_game();
        game.state.player.lives = 1;
        game.state.player_stocks[1] = PlayerStockSnapshot::new(3, 3);
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let collision = game.step(GameInput::NONE);

        assert_eq!(collision.state.phase, GamePhase::GameOver);
        assert_eq!(collision.state.current_player, 1);
        assert_eq!(collision.state.player.lives, 0);
        assert_eq!(
            collision.state.player_stocks,
            [
                PlayerStockSnapshot::new(0, 3),
                PlayerStockSnapshot::new(3, 3),
            ]
        );
        assert_eq!(
            collision.state.game_over,
            GameOverSnapshot::player_switch_sleep(PLAYER_SWITCH_SLEEP_FRAMES, 1, 2)
        );
        assert_eq!(collision.events.gameplay(), &[GameEvent::PlayerDestroyed]);
        assert_eq!(collision.scene.summary().layers.objects, 0);
        assert_eq!(collision.scene.summary().layers.overlay, 17);
        assert!(collision.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [120.0, 120.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(collision.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_G
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [124.0, 136.0]
                && sprite.size == [6.0, 8.0]
        }));

        let switched = advance_player_switch_sleep(&mut game, 1, 2);

        assert_eq!(switched.state.phase, GamePhase::Playing);
        assert_eq!(switched.state.current_player, 2);
        assert_eq!(switched.state.player.lives, 3);
        assert_eq!(switched.state.player.smart_bombs, 3);
        assert_eq!(switched.state.world, WorldSnapshot::default());
        assert_eq!(switched.state.game_over, GameOverSnapshot::NONE);
        assert_eq!(
            game.start_playfield_delay,
            Some(START_PLAYFIELD_DELAY_FRAMES)
        );

        let active = advance_to_started_playfield(&mut game);

        assert_eq!(active.state.phase, GamePhase::Playing);
        assert_eq!(active.state.current_player, 2);
        assert_eq!(active.state.player.lives, 2);
        assert_eq!(
            active.state.player_stocks,
            [
                PlayerStockSnapshot::new(0, 3),
                PlayerStockSnapshot::new(2, 3),
            ]
        );
        assert_eq!(active.state.world.enemies.len(), 5);
    }

    #[test]
    fn clean_game_two_player_non_final_death_rotates_to_next_player() {
        let mut game = two_player_started_game();
        game.state.player.lives = 2;
        game.state.player_stocks[1] = PlayerStockSnapshot::new(3, 3);
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let collision = game.step(GameInput::NONE);

        assert_eq!(collision.state.phase, GamePhase::GameOver);
        assert_eq!(collision.state.current_player, 1);
        assert_eq!(collision.state.game_over, GameOverSnapshot::NONE);
        assert_eq!(collision.state.player.lives, 1);
        assert_eq!(
            collision.state.player_stocks,
            [
                PlayerStockSnapshot::new(1, 3),
                PlayerStockSnapshot::new(3, 3),
            ]
        );
        assert_eq!(collision.events.gameplay(), &[GameEvent::PlayerDestroyed]);
        assert!(!collision.scene.sprites.iter().any(|sprite| {
            matches!(
                sprite.sprite,
                SpriteId::MESSAGE_GLYPH_P | SpriteId::MESSAGE_GLYPH_G
            ) && sprite.layer == RenderLayer::Overlay
        }));

        let respawn = advance_pending_respawn(&mut game);

        assert_eq!(respawn.state.phase, GamePhase::Playing);
        assert_eq!(respawn.state.current_player, 2);
        assert_eq!(respawn.state.player.lives, 3);
        assert_eq!(respawn.state.player.smart_bombs, 3);
        assert_eq!(respawn.state.world, WorldSnapshot::default());
        assert_eq!(
            game.start_playfield_delay,
            Some(START_PLAYFIELD_DELAY_FRAMES)
        );

        let active = advance_to_started_playfield(&mut game);

        assert_eq!(active.state.phase, GamePhase::Playing);
        assert_eq!(active.state.current_player, 2);
        assert_eq!(active.state.player.lives, 2);
        assert_eq!(
            active.state.player_stocks,
            [
                PlayerStockSnapshot::new(1, 3),
                PlayerStockSnapshot::new(2, 3),
            ]
        );
        assert_eq!(active.state.world.enemies.len(), 5);
    }

    #[test]
    fn clean_game_two_player_rotation_keeps_score_and_bonus_stock_ownership() {
        let mut game = two_player_started_game();
        game.state.player.lives = 2;
        game.state.player_stocks[1] = PlayerStockSnapshot::new(3, 3);
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let collision = game.step(GameInput::NONE);
        assert_eq!(
            collision.state.player_stocks[0],
            PlayerStockSnapshot::new(1, 3)
        );

        advance_pending_respawn(&mut game);
        let active = advance_to_started_playfield(&mut game);
        assert_eq!(active.state.current_player, 2);
        assert_eq!(
            active.state.player_stocks[0],
            PlayerStockSnapshot::new(1, 3)
        );
        assert_eq!(
            active.state.player_stocks[1],
            PlayerStockSnapshot::new(2, 3)
        );

        game.state.scores.player_one = 9_000;
        game.state.scores.player_two = 9_900;
        game.state.scores.high_score = 9_900;
        game.state.scores.next_bonus = 10_000;
        keep_first_enemy_only(&mut game);
        game.state.world.enemies[0].position = ScreenPosition::new(100, 80);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);
        game.state.world.enemies.push(EnemySnapshot::new(
            EnemyKind::Mutant,
            ScreenPosition::new(220, 80),
            ScreenVelocity::new(0, 0),
        ));
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let scored = game.step(GameInput::NONE);

        assert_eq!(scored.state.current_player, 2);
        assert_eq!(scored.state.scores.player_one, 9_000);
        assert_eq!(scored.state.scores.player_two, 10_050);
        assert_eq!(scored.state.scores.high_score, 10_050);
        assert_eq!(scored.state.scores.next_bonus, 20_000);
        assert_eq!(
            scored.state.player,
            super::PlayerSnapshot {
                position: (super::world_word(0x2000), super::world_word(0x8000)),
                velocity: (WorldVector::default(), WorldVector::default()),
                direction: Direction::Right,
                lives: 3,
                smart_bombs: 4,
            }
        );
        assert_eq!(
            scored.state.player_stocks[0],
            PlayerStockSnapshot::new(1, 3)
        );
        assert_eq!(
            scored.state.player_stocks[1],
            PlayerStockSnapshot::new(3, 4)
        );
        assert_eq!(
            scored.events.gameplay(),
            &[GameEvent::EnemyDestroyed, GameEvent::BonusAwarded]
        );
    }

    #[test]
    fn clean_game_two_player_final_death_ends_game_when_other_player_has_no_stock() {
        let mut game = two_player_started_game();
        game.state.player.lives = 1;
        game.state.player_stocks[1] = PlayerStockSnapshot::new(0, 3);
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.phase, GamePhase::GameOver);
        assert_eq!(
            frame.state.game_over,
            GameOverSnapshot::player_death_sleep(PLAYER_DEATH_GAME_OVER_SLEEP_FRAMES)
        );
        assert_eq!(
            frame.events.gameplay(),
            &[GameEvent::PlayerDestroyed, GameEvent::GameOver]
        );
    }

    #[test]
    fn clean_game_two_player_second_player_final_death_switches_back_to_player_one() {
        let mut game = two_player_started_game();
        game.state.current_player = 2;
        game.state.player = super::PlayerSnapshot {
            position: (super::world_word(0x2000), super::world_word(0x8000)),
            velocity: (WorldVector::default(), WorldVector::default()),
            direction: Direction::Right,
            lives: 1,
            smart_bombs: 2,
        };
        game.state.player_stocks = [
            PlayerStockSnapshot::new(1, 1),
            PlayerStockSnapshot::new(1, 2),
        ];
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let collision = game.step(GameInput::NONE);

        assert_eq!(
            collision.state.game_over,
            GameOverSnapshot::player_switch_sleep(PLAYER_SWITCH_SLEEP_FRAMES, 2, 1)
        );
        assert!(collision.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_W
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [180.0, 120.0]
                && sprite.size == [8.0, 8.0]
        }));

        let switched = advance_player_switch_sleep(&mut game, 2, 1);

        assert_eq!(switched.state.current_player, 1);
        assert_eq!(switched.state.player.lives, 1);
        assert_eq!(switched.state.player.smart_bombs, 1);
        assert_eq!(switched.state.world, WorldSnapshot::default());
        assert_eq!(
            game.start_playfield_delay,
            Some(START_PLAYFIELD_DELAY_FRAMES)
        );

        let active = advance_to_started_playfield(&mut game);

        assert_eq!(active.state.phase, GamePhase::Playing);
        assert_eq!(active.state.current_player, 1);
        assert_eq!(active.state.player.lives, 0);
        assert_eq!(active.state.player.smart_bombs, 1);
        assert_eq!(
            active.state.player_stocks,
            [
                PlayerStockSnapshot::new(0, 1),
                PlayerStockSnapshot::new(0, 2),
            ]
        );
        assert_eq!(active.state.world.enemies.len(), 5);
    }

    #[test]
    fn clean_game_high_score_entry_starts_after_qualifying_game_over() {
        let mut game = credited_started_game();
        game.state.player.lives = 1;
        game.state.scores.player_one = 501;
        game.state.scores.high_score = 500;
        game.state.high_score_tables = HighScoreTablesSnapshot::EMPTY;
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.phase, GamePhase::GameOver);
        assert_eq!(frame.state.player.lives, 0);
        assert_eq!(
            frame.state.game_over,
            GameOverSnapshot::player_death_sleep(PLAYER_DEATH_GAME_OVER_SLEEP_FRAMES)
        );
        assert_eq!(frame.state.high_score_entry, None);
        assert_eq!(
            frame.events.gameplay(),
            &[GameEvent::PlayerDestroyed, GameEvent::GameOver]
        );
        assert_eq!(frame.scene.summary().layers.objects, 0);

        let handoff = advance_player_death_game_over_sleep(&mut game);

        assert_eq!(handoff.state.phase, GamePhase::HighScoreEntry);
        assert_eq!(
            handoff.state.high_score_entry,
            Some(HighScoreEntrySnapshot {
                score: 501,
                rank: 1,
            })
        );
        assert_eq!(handoff.state.high_score_submission, None);
        assert_eq!(handoff.state.game_over, GameOverSnapshot::NONE);
        assert_eq!(
            handoff.events.gameplay(),
            &[GameEvent::HighScoreEntryStarted]
        );
        assert_eq!(handoff.scene.summary().layers.objects, 0);
        assert_eq!(handoff.scene.summary().layers.overlay, 113);
        assert!(handoff.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [124.0, 56.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(handoff.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_Y
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [40.0, 88.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(handoff.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [40.0, 138.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(handoff.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_UNDERLINE_WORD
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [140.0, 183.0]
                && sprite.size == [2.0, 2.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(handoff.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_UNDERLINE_WORD
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [156.0, 183.0]
                && sprite.size == [2.0, 2.0]
                && sprite.tint == Color::from_rgba(0x66, 0x66, 0x66, 0xFF)
        }));
    }

    #[test]
    fn clean_game_high_score_entry_uses_current_second_player_score() {
        let mut game = two_player_started_game();
        game.state.current_player = 2;
        game.state.player.lives = 1;
        game.state.player_stocks = [
            PlayerStockSnapshot::new(0, 3),
            PlayerStockSnapshot::new(1, 3),
        ];
        game.state.scores.player_one = 1_000;
        game.state.scores.player_two = 2_001;
        game.state.scores.high_score = 2_000;
        game.state.high_score_tables = HighScoreTablesSnapshot::EMPTY;
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.phase, GamePhase::GameOver);
        assert_eq!(
            frame.state.game_over,
            GameOverSnapshot::player_death_sleep(PLAYER_DEATH_GAME_OVER_SLEEP_FRAMES)
        );

        let handoff = advance_player_death_game_over_sleep(&mut game);

        assert_eq!(handoff.state.phase, GamePhase::HighScoreEntry);
        assert_eq!(
            handoff.state.high_score_entry,
            Some(HighScoreEntrySnapshot {
                score: 2_001,
                rank: 1,
            })
        );
        assert_eq!(
            handoff.events.gameplay(),
            &[GameEvent::HighScoreEntryStarted]
        );
    }

    #[test]
    fn clean_game_two_player_high_score_submission_orders_tables_and_returns_to_attract() {
        let mut game = two_player_started_game();
        let submitted_score = 15_000;
        game.state.current_player = 2;
        game.state.player = super::PlayerSnapshot {
            position: (super::world_word(0x2000), super::world_word(0x8000)),
            velocity: (WorldVector::default(), WorldVector::default()),
            direction: Direction::Right,
            lives: 1,
            smart_bombs: 1,
        };
        game.state.player_stocks = [
            PlayerStockSnapshot::new(0, 3),
            PlayerStockSnapshot::new(1, 1),
        ];
        game.state.scores.player_one = 19_000;
        game.state.scores.player_two = submitted_score;
        game.state.scores.high_score = 50_000;
        game.state.high_score_tables = HighScoreTablesSnapshot {
            all_time: [
                table_entry(1, 50_000, ['A', 'A', 'A']),
                table_entry(2, 30_000, ['B', 'B', 'B']),
                table_entry(3, 20_000, ['C', 'C', 'C']),
                table_entry(4, 10_000, ['D', 'D', 'D']),
                table_entry(5, 8_000, ['E', 'E', 'E']),
                table_entry(6, 6_000, ['F', 'F', 'F']),
                table_entry(7, 4_000, ['G', 'G', 'G']),
                table_entry(8, 2_000, ['H', 'H', 'H']),
            ],
            todays_greatest: [
                table_entry(1, 14_000, ['T', 'O', 'P']),
                table_entry(2, 13_000, ['T', 'W', 'O']),
                table_entry(3, 12_000, ['T', 'H', 'R']),
                table_entry(4, 11_000, ['F', 'O', 'R']),
                table_entry(5, 9_000, ['F', 'I', 'V']),
                table_entry(6, 7_000, ['S', 'I', 'X']),
                table_entry(7, 5_000, ['S', 'E', 'V']),
                table_entry(8, 3_000, ['E', 'G', 'T']),
            ],
        };
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let collision = game.step(GameInput::NONE);
        assert_eq!(collision.state.phase, GamePhase::GameOver);
        assert_eq!(
            collision.state.game_over,
            GameOverSnapshot::player_death_sleep(PLAYER_DEATH_GAME_OVER_SLEEP_FRAMES)
        );

        let handoff = advance_player_death_game_over_sleep(&mut game);

        assert_eq!(handoff.state.phase, GamePhase::HighScoreEntry);
        assert_eq!(handoff.state.current_player, 2);
        assert_eq!(
            handoff.state.high_score_entry,
            Some(HighScoreEntrySnapshot {
                score: submitted_score,
                rank: 1,
            })
        );

        for initial in ['p', 'l'] {
            let accepted = game.step(GameInput {
                high_score_initial: Some(initial),
                ..GameInput::NONE
            });
            assert_eq!(
                accepted.events.gameplay(),
                &[GameEvent::HighScoreInitialAccepted]
            );
        }

        let submitted = game.step(GameInput {
            high_score_initial: Some('r'),
            ..GameInput::NONE
        });

        assert_eq!(submitted.state.phase, GamePhase::GameOver);
        assert_eq!(
            submitted.state.high_score_submission,
            Some(HighScoreSubmissionSnapshot {
                player: 2,
                score: submitted_score,
            })
        );
        assert_eq!(submitted.state.scores.high_score, 50_000);
        assert_eq!(
            submitted.state.high_score_tables.all_time[3],
            table_entry(4, submitted_score, ['P', 'L', 'R'])
        );
        assert_eq!(
            submitted.state.high_score_tables.all_time[4],
            table_entry(5, 10_000, ['D', 'D', 'D'])
        );
        assert_eq!(
            submitted.state.high_score_tables.todays_greatest[0],
            table_entry(1, submitted_score, ['P', 'L', 'R'])
        );
        assert_eq!(
            submitted.state.high_score_tables.todays_greatest[1],
            table_entry(2, 14_000, ['T', 'O', 'P'])
        );
        assert_eq!(
            submitted.state.game_over,
            GameOverSnapshot::hall_of_fame_display(HALL_OF_FAME_STALL_FRAMES)
        );
        assert_eq!(
            submitted.events.gameplay(),
            &[
                GameEvent::HighScoreInitialAccepted,
                GameEvent::HighScoreSubmitted,
            ]
        );

        for expected_timer in (1..HALL_OF_FAME_STALL_FRAMES).rev() {
            let waiting = game.step(GameInput::NONE);
            assert_eq!(waiting.state.phase, GamePhase::GameOver);
            assert_eq!(
                waiting.state.game_over,
                GameOverSnapshot::hall_of_fame_display(expected_timer)
            );
            assert!(waiting.events.is_empty());
        }

        let returned = game.step(GameInput::NONE);
        assert_eq!(returned.state.phase, GamePhase::Attract);
        assert_eq!(returned.state.game_over, GameOverSnapshot::NONE);
        assert_eq!(returned.state.high_score_entry, None);
    }

    #[test]
    fn clean_game_high_score_tables_rank_strictly_and_drop_tail() {
        let mut tables = HighScoreTablesSnapshot {
            all_time: [
                table_entry(1, 80_000, ['A', 'A', 'A']),
                table_entry(2, 70_000, ['B', 'B', 'B']),
                table_entry(3, 60_000, ['C', 'C', 'C']),
                table_entry(4, 50_000, ['D', 'D', 'D']),
                table_entry(5, 40_000, ['E', 'E', 'E']),
                table_entry(6, 30_000, ['F', 'F', 'F']),
                table_entry(7, 20_000, ['G', 'G', 'G']),
                table_entry(8, 10_000, ['H', 'H', 'H']),
            ],
            todays_greatest: [
                table_entry(1, 8_000, ['T', 'O', 'P']),
                table_entry(2, 7_000, ['T', 'W', 'O']),
                table_entry(3, 6_000, ['T', 'H', 'R']),
                table_entry(4, 5_000, ['F', 'O', 'R']),
                table_entry(5, 4_000, ['F', 'I', 'V']),
                table_entry(6, 3_000, ['S', 'I', 'X']),
                table_entry(7, 2_000, ['S', 'E', 'V']),
                table_entry(8, 1_000, ['E', 'G', 'T']),
            ],
        };
        let unchanged_all_time = tables.all_time;

        assert_eq!(
            tables.insert_all_time(10_000, ['E', 'Q', 'L'].map(Some)),
            None
        );
        assert_eq!(tables.all_time, unchanged_all_time);

        assert_eq!(
            tables.insert_all_time(65_000, ['M', 'I', 'D'].map(Some)),
            Some(3)
        );
        assert_eq!(tables.all_time[2], table_entry(3, 65_000, ['M', 'I', 'D']));
        assert_eq!(tables.all_time[3], table_entry(4, 60_000, ['C', 'C', 'C']));
        assert_eq!(tables.all_time[7], table_entry(8, 20_000, ['G', 'G', 'G']));

        assert_eq!(
            tables.insert_todays_greatest(9_000, ['N', 'E', 'W'].map(Some)),
            Some(1)
        );
        assert_eq!(
            tables.todays_greatest[0],
            table_entry(1, 9_000, ['N', 'E', 'W'])
        );
        assert_eq!(
            tables.todays_greatest[1],
            table_entry(2, 8_000, ['T', 'O', 'P'])
        );
        assert_eq!(
            tables.todays_greatest[7],
            table_entry(8, 2_000, ['S', 'E', 'V'])
        );
    }

    #[test]
    fn clean_game_high_score_initials_accept_backspace_and_submit() {
        let mut game = credited_started_game();
        game.state.phase = GamePhase::HighScoreEntry;
        game.state.high_score_initials = HighScoreInitialsState::EMPTY;
        game.state.high_score_entry = Some(HighScoreEntrySnapshot {
            score: 25_000,
            rank: 1,
        });

        let first = game.step(GameInput {
            high_score_initial: Some('a'),
            ..GameInput::NONE
        });

        assert_eq!(first.state.phase, GamePhase::HighScoreEntry);
        assert_eq!(
            first.state.high_score_initials.initials,
            [Some('A'), None, None]
        );
        assert_eq!(first.state.high_score_initials.cursor, 1);
        assert_eq!(
            first.events.gameplay(),
            &[GameEvent::HighScoreInitialAccepted]
        );
        assert_eq!(first.scene.summary().layers.overlay, 114);
        assert!(first.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_A
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [140.0, 172.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(first.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_UNDERLINE_WORD
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [140.0, 183.0]
                && sprite.size == [2.0, 2.0]
                && sprite.tint == Color::from_rgba(0x66, 0x66, 0x66, 0xFF)
        }));
        assert!(first.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_UNDERLINE_WORD
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [156.0, 183.0]
                && sprite.size == [2.0, 2.0]
                && sprite.tint == Color::WHITE
        }));

        let ignored = game.step(GameInput {
            high_score_initial: Some('1'),
            ..GameInput::NONE
        });
        assert_eq!(
            ignored.state.high_score_initials,
            first.state.high_score_initials
        );
        assert!(ignored.events.is_empty());

        let erased = game.step(GameInput {
            high_score_backspace: true,
            ..GameInput::NONE
        });
        assert_eq!(
            erased.state.high_score_initials,
            HighScoreInitialsState::EMPTY
        );
        assert!(erased.events.is_empty());

        for initial in ['b', 'c'] {
            let frame = game.step(GameInput {
                high_score_initial: Some(initial),
                ..GameInput::NONE
            });
            assert_eq!(
                frame.events.gameplay(),
                &[GameEvent::HighScoreInitialAccepted]
            );
        }

        let submitted = game.step(GameInput {
            high_score_initial: Some('d'),
            ..GameInput::NONE
        });

        assert_eq!(submitted.state.phase, GamePhase::GameOver);
        assert_eq!(submitted.state.high_score_entry, None);
        assert_eq!(
            submitted.state.high_score_submission,
            Some(HighScoreSubmissionSnapshot {
                player: 1,
                score: 25_000,
            })
        );
        assert_eq!(
            submitted.state.game_over,
            GameOverSnapshot::hall_of_fame_display(HALL_OF_FAME_STALL_FRAMES)
        );
        assert_eq!(submitted.state.scores.high_score, 25_000);
        assert_eq!(
            submitted.state.high_score_tables.all_time[0],
            HighScoreTableEntrySnapshot {
                rank: 1,
                score: 25_000,
                initials: [Some('B'), Some('C'), Some('D')],
            }
        );
        assert_eq!(
            submitted.state.high_score_tables.todays_greatest[0],
            HighScoreTableEntrySnapshot {
                rank: 1,
                score: 25_000,
                initials: [Some('B'), Some('C'), Some('D')],
            }
        );
        assert_eq!(
            submitted.state.high_score_initials.initials,
            [Some('B'), Some('C'), Some('D')]
        );
        assert_eq!(
            submitted.events.gameplay(),
            &[
                GameEvent::HighScoreInitialAccepted,
                GameEvent::HighScoreSubmitted,
            ]
        );
        assert_eq!(submitted.scene.summary().layers.objects, 0);
        let display_underlines = submitted
            .scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::HALL_OF_FAME_UNDERLINE_WORD)
            .collect::<Vec<_>>();
        assert_eq!(display_underlines.len(), 62);
        assert!(display_underlines.iter().any(|sprite| {
            sprite.position == [250.0, 123.0]
                && sprite.size == [2.0, 2.0]
                && sprite.tint == Color::from_rgba(0x66, 0x66, 0x66, 0xFF)
        }));
        assert!(display_underlines.iter().any(|sprite| {
            sprite.position == [60.0, 123.0]
                && sprite.size == [2.0, 2.0]
                && sprite.tint == Color::from_rgba(0x66, 0x66, 0x66, 0xFF)
        }));
        assert!(submitted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_H
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [112.0, 84.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(submitted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [96.0, 56.0]
                && sprite.size == [120.0, 24.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(submitted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [48.0, 134.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(submitted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_B
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [58.0, 134.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(submitted.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_2
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [90.0, 134.0]
                && sprite.size == [6.0, 8.0]
        }));

        for expected_timer in (1..HALL_OF_FAME_STALL_FRAMES).rev() {
            let waiting = game.step(GameInput::NONE);
            assert_eq!(waiting.state.phase, GamePhase::GameOver);
            assert_eq!(
                waiting.state.game_over,
                GameOverSnapshot::hall_of_fame_display(expected_timer)
            );
            assert!(waiting.events.is_empty());
        }

        let returned = game.step(GameInput::NONE);
        assert_eq!(returned.state.phase, GamePhase::Attract);
        assert_eq!(returned.state.game_over, GameOverSnapshot::NONE);
        assert_eq!(returned.state.high_score_entry, None);
        assert_eq!(
            returned.state.high_score_submission,
            Some(HighScoreSubmissionSnapshot {
                player: 1,
                score: 25_000,
            })
        );
    }

    #[test]
    fn clean_game_non_qualifying_game_over_waits_then_shows_hall_of_fame() {
        let mut game = credited_started_game();
        game.state.player.lives = 1;
        game.state.scores.player_one = 10;
        let unchanged_tables = game.state.high_score_tables;
        game.state.world.enemies[0].position = ScreenPosition::new(32, 128);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);

        let collision = game.step(GameInput::NONE);
        assert_eq!(collision.state.phase, GamePhase::GameOver);
        assert_eq!(
            collision.state.game_over,
            GameOverSnapshot::player_death_sleep(PLAYER_DEATH_GAME_OVER_SLEEP_FRAMES)
        );

        let no_entry = advance_player_death_game_over_sleep(&mut game);
        assert_eq!(no_entry.state.phase, GamePhase::GameOver);
        assert_eq!(
            no_entry.state.game_over,
            GameOverSnapshot::no_entry_delay(HALL_OF_FAME_NO_ENTRY_DELAY_FRAMES)
        );
        assert!(no_entry.events.is_empty());

        for expected_timer in (1..HALL_OF_FAME_NO_ENTRY_DELAY_FRAMES).rev() {
            let waiting = game.step(GameInput::NONE);
            assert_eq!(waiting.state.phase, GamePhase::GameOver);
            assert_eq!(
                waiting.state.game_over,
                GameOverSnapshot::no_entry_delay(expected_timer)
            );
            assert!(waiting.events.is_empty());
        }

        let hall = game.step(GameInput::NONE);
        assert_eq!(hall.state.phase, GamePhase::Attract);
        assert_eq!(
            hall.state.game_over,
            GameOverSnapshot::hall_of_fame_display(HALL_OF_FAME_STALL_FRAMES)
        );
        assert!(hall.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [96.0, 56.0]
                && sprite.size == [120.0, 24.0]
        }));

        let ticked_hall = game.step(GameInput::NONE);
        assert_eq!(ticked_hall.state.phase, GamePhase::Attract);
        assert_eq!(
            ticked_hall.state.game_over,
            GameOverSnapshot::hall_of_fame_display(HALL_OF_FAME_STALL_FRAMES - 1)
        );
        assert_eq!(ticked_hall.state.high_score_tables, unchanged_tables);

        for expected_timer in (1..HALL_OF_FAME_STALL_FRAMES - 1).rev() {
            let waiting = game.step(GameInput::NONE);
            assert_eq!(waiting.state.phase, GamePhase::Attract);
            assert_eq!(
                waiting.state.game_over,
                GameOverSnapshot::hall_of_fame_display(expected_timer)
            );
            assert!(waiting.events.is_empty());
        }

        let returned = game.step(GameInput::NONE);
        assert_eq!(returned.state.phase, GamePhase::Attract);
        assert_eq!(returned.state.game_over, GameOverSnapshot::NONE);
        assert_eq!(returned.state.high_score_entry, None);
        assert_eq!(returned.state.high_score_submission, None);
        assert_eq!(returned.state.high_score_tables, unchanged_tables);
    }

    #[test]
    fn clean_game_scores_current_second_player_on_collision() {
        let mut game = credited_started_game();
        keep_first_enemy_only(&mut game);
        game.state.current_player = 2;
        game.state.world.enemies[0].position = ScreenPosition::new(100, 80);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.scores.player_one, 0);
        assert_eq!(frame.state.scores.player_two, 150);
        assert_eq!(
            frame.events.gameplay(),
            &[GameEvent::EnemyDestroyed, GameEvent::WaveCleared]
        );
    }

    #[test]
    fn clean_game_bonus_award_updates_stock_threshold_and_events() {
        let mut game = credited_started_game();
        keep_first_enemy_only(&mut game);
        game.state.scores.player_one = 9_900;
        game.state.scores.high_score = 9_900;
        game.state.scores.next_bonus = 10_000;
        game.state.player.lives = 3;
        game.state.player.smart_bombs = 1;
        game.state.world.enemies[0].position = ScreenPosition::new(100, 80);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let frame = game.step(GameInput::NONE);

        assert_eq!(frame.state.scores.player_one, 10_050);
        assert_eq!(frame.state.scores.high_score, 10_050);
        assert_eq!(frame.state.scores.next_bonus, 20_000);
        assert_eq!(frame.state.player.lives, 4);
        assert_eq!(frame.state.player.smart_bombs, 2);
        assert_eq!(
            frame.events.gameplay(),
            &[
                GameEvent::EnemyDestroyed,
                GameEvent::BonusAwarded,
                GameEvent::WaveCleared,
            ]
        );
    }

    #[test]
    fn clean_game_wave_clear_delays_next_wave_spawn_until_following_frame() {
        let mut game = credited_started_game();
        keep_first_enemy_only(&mut game);
        game.state.world.enemies[0].position = ScreenPosition::new(100, 80);
        game.state.world.enemies[0].velocity = ScreenVelocity::new(0, 0);
        game.state.world.projectiles.push(ProjectileSnapshot {
            position: ScreenPosition::new(101, 83),
            velocity: ScreenVelocity::new(0, 0),
        });

        let cleared = game.step(GameInput::NONE);

        assert_eq!(cleared.state.wave, 1);
        assert!(cleared.state.world.enemies.is_empty());
        assert_eq!(
            cleared.events.gameplay(),
            &[GameEvent::EnemyDestroyed, GameEvent::WaveCleared]
        );
        assert!(
            !cleared
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER
                    && sprite.size == [12.0, 8.0])
        );
        assert_eq!(cleared.scene.summary().layers.overlay, 37);
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_A
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [112.0, 80.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [202.0, 80.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_C
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [122.0, 96.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_B
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [120.0, 144.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [176.0, 144.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [120.0, 160.0]
                && sprite.size == [4.0, 8.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [128.0, 160.0]
                && sprite.size == [4.0, 8.0]
                && sprite.tint == Color::WHITE
        }));

        let next_wave = game.step(GameInput::NONE);

        assert_eq!(next_wave.state.wave, 2);
        assert_eq!(next_wave.state.world.enemies.len(), 5);
        assert_eq!(
            next_wave
                .state
                .world
                .enemies
                .iter()
                .map(|enemy| enemy.kind)
                .collect::<Vec<_>>(),
            vec![
                EnemyKind::Lander,
                EnemyKind::Bomber,
                EnemyKind::Pod,
                EnemyKind::Lander,
                EnemyKind::Lander,
            ]
        );
        assert!(next_wave.state.world.projectiles.is_empty());
        assert_eq!(next_wave.state.world.terrain.len(), 5);
        assert_eq!(next_wave.state.world.humans.len(), 10);
        assert_eq!(next_wave.events.gameplay(), &[GameEvent::WaveStarted]);
        assert_eq!(
            next_wave
                .scene
                .sprites
                .iter()
                .filter(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER)
                .count(),
            3
        );
        assert!(
            next_wave
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_BOMBER)
        );
        assert!(
            next_wave
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_POD)
        );
    }

    #[test]
    fn clean_game_advances_enemy_positions_through_systems() {
        let mut game = credited_started_game();
        let before = game.state.world.enemies[0].position;
        let before_source = game.state.world.enemies[0]
            .source_lander
            .expect("initial lander should carry source state");

        let frame = game.step(GameInput::NONE);
        let enemy = frame.state.world.enemies[0];
        let source = enemy
            .source_lander
            .expect("source lander state should be retained");
        let expected_y_velocity = super::source_lander_orbit_y_velocity(
            game.state.wave_profile,
            before,
            &game.state.world.terrain,
        );
        let (expected_x, expected_x_fraction) = super::source_fixed_axis_step(
            before.x,
            before_source.x_fraction,
            before_source.x_velocity,
        );
        let (expected_y, expected_y_fraction) =
            super::source_fixed_axis_step(before.y, before_source.y_fraction, expected_y_velocity);

        assert_eq!(enemy.position, ScreenPosition::new(expected_x, expected_y));
        assert_eq!(source.x_fraction, expected_x_fraction);
        assert_eq!(source.y_fraction, expected_y_fraction);
        assert_eq!(source.y_velocity, expected_y_velocity);
        assert_eq!(enemy.velocity, super::source_lander_screen_velocity(source));
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER
                && sprite.position == [f32::from(enemy.position.x), f32::from(enemy.position.y)]
        }));
    }

    #[test]
    fn clean_game_source_enemy_y_motion_wraps_through_source_playfield_bounds() {
        let mut game = credited_started_game();
        game.state.player.position = (super::world_word(0x2000), super::world_word(0x8000));
        game.state.player.velocity = (WorldVector::default(), WorldVector::default());
        game.state.world.enemies = vec![
            EnemySnapshot::source_pod(
                ScreenPosition::new(0xD0, SOURCE_PLAYFIELD_Y_MIN),
                ScreenVelocity::new(0, -1),
                SourcePodSnapshot {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0,
                    y_velocity: 0xFFFF,
                },
            ),
            EnemySnapshot::source_pod(
                ScreenPosition::new(0xE0, super::SOURCE_PLAYFIELD_Y_MAX),
                ScreenVelocity::new(0, 1),
                SourcePodSnapshot {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0,
                    y_velocity: 0x0100,
                },
            ),
        ];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
        game.state.world.projectiles.clear();
        game.state.world.enemy_projectiles.clear();
        game.baiter_timer_ticks = None;

        let frame = game.step(GameInput::NONE);

        let [top_wrap, bottom_wrap] = frame.state.world.enemies.as_slice() else {
            panic!("expected both source pods to stay active");
        };
        let top_source = top_wrap
            .source_pod
            .expect("top-wrapping pod should retain source state");
        let bottom_source = bottom_wrap
            .source_pod
            .expect("bottom-wrapping pod should retain source state");
        let (top_y, top_fraction) =
            super::source_active_object_y_step(SOURCE_PLAYFIELD_Y_MIN, 0, 0xFFFF);
        let (bottom_y, bottom_fraction) =
            super::source_active_object_y_step(super::SOURCE_PLAYFIELD_Y_MAX, 0, 0x0100);

        assert_eq!(top_wrap.position, ScreenPosition::new(0xD0, top_y));
        assert_eq!(top_source.y_fraction, top_fraction);
        assert_eq!(
            top_wrap.velocity,
            super::source_pod_screen_velocity(top_source)
        );
        assert_eq!(bottom_wrap.position, ScreenPosition::new(0xE0, bottom_y));
        assert_eq!(bottom_source.y_fraction, bottom_fraction);
        assert_eq!(
            bottom_wrap.velocity,
            super::source_pod_screen_velocity(bottom_source)
        );
    }

    #[test]
    fn clean_game_world_sprites_are_atlas_backed() {
        let mut game = credited_started_game();

        let frame = game.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let atlas = TextureAtlas::default_sprites();
        for sprite in &frame.scene.sprites {
            assert!(atlas.contains(sprite.sprite), "{:?}", sprite.sprite);
        }
        assert!(
            frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER)
        );
        assert!(
            frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::HUMAN)
        );
        assert!(
            frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::TERRAIN_TILE)
        );
        assert!(
            frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::STAR)
        );
    }

    #[test]
    fn clean_game_projects_source_bgout_terrain_words() {
        let mut game = credited_started_game();

        let frame = game.step(GameInput::NONE);
        let terrain_sprites = frame
            .scene
            .sprites
            .iter()
            .filter(|sprite| sprite.layer == RenderLayer::Terrain)
            .collect::<Vec<_>>();

        assert_eq!(terrain_sprites.len(), super::SOURCE_TERRAIN_SCREEN_WORDS);
        assert_eq!(terrain_sprites[0].sprite, SpriteId::TERRAIN_TILE);
        assert_eq!(terrain_sprites[0].position, [304.0, 222.0]);
        assert_eq!(terrain_sprites[0].size, super::SOURCE_TERRAIN_WORD_SIZE);
        assert_eq!(terrain_sprites[0].tint, Color::WHITE);
        assert_eq!(terrain_sprites[6].sprite, SpriteId::TERRAIN_TILE_ALT);
        assert_eq!(terrain_sprites[6].position, [292.0, 215.0]);
        assert!(
            terrain_sprites
                .iter()
                .all(|sprite| sprite.size == super::SOURCE_TERRAIN_WORD_SIZE
                    && sprite.tint == Color::WHITE)
        );
    }

    #[test]
    fn clean_game_projects_source_top_display_border() {
        let mut game = credited_started_game();

        let frame = game.step(GameInput::NONE);
        let border_sprites = frame
            .scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD)
            .collect::<Vec<_>>();

        assert_eq!(border_sprites.len(), 6);
        assert!(
            border_sprites
                .iter()
                .all(|sprite| { sprite.layer == RenderLayer::Hud && sprite.tint == Color::WHITE })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [0.0, 40.0] && sprite.size == [312.0, 2.0] })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [94.0, 8.0] && sprite.size == [2.0, 32.0] })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [224.0, 8.0] && sprite.size == [2.0, 32.0] })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [94.0, 7.0] && sprite.size == [130.0, 1.0] })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [152.0, 7.0] && sprite.size == [16.0, 2.0] })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [152.0, 40.0] && sprite.size == [16.0, 2.0] })
        );
    }

    #[test]
    fn scanner_radar_snapshot_uses_source_scan_formula_and_cadence() {
        let mut details = [ObjectEvidenceDetailSnapshot::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT];
        details[0] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Active,
            address: Some(0xA23C),
            world_position: Some((0x2100, 0x5000)),
            scanner_color: Some(0x1234),
            ..ObjectEvidenceDetailSnapshot::EMPTY
        };
        details[1] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Inactive,
            address: Some(0xA27C),
            world_position: Some((0x2300, 0x6800)),
            scanner_color: Some(0xABCD),
            ..ObjectEvidenceDetailSnapshot::EMPTY
        };
        details[2] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Projectile,
            address: Some(0xA2BC),
            world_position: Some((0x2500, 0x7000)),
            scanner_color: Some(0xFFFF),
            ..ObjectEvidenceDetailSnapshot::EMPTY
        };
        let evidence = ObjectEvidenceSnapshot {
            active_count: 2,
            inactive_count: 1,
            projectile_count: 1,
            visible_count: 3,
            evidence_crc32: Some(0x5CA4_4E11),
            detail_count: 3,
            details,
        };

        let scanner = ScannerRadarSnapshot::for_world(
            GamePhase::Playing,
            4,
            super::world_word(0x2000),
            (super::world_word(0x8000), super::world_word(0x4000)),
            &evidence,
        );

        assert!(scanner.enabled);
        assert_eq!(scanner.stage, ScannerRadarStage::RasterDisplay);
        assert_eq!(scanner.stage_sleep_ticks, 4);
        assert_eq!(scanner.source_process_sleep_ticks, [2, 2, 4]);
        assert_eq!(scanner.selected_map, 1);
        assert_eq!(scanner.scan_left, Some(0xB2C0));
        assert!(scanner.terrain_enabled);
        assert_eq!(scanner.object_erase_start, 0xB05D);
        assert_eq!(scanner.setend, 0xB061);
        assert_eq!(scanner.blip_count, 2);
        assert_eq!(
            scanner.blips[0],
            super::ScannerRadarBlipSnapshot {
                kind: ScannerRadarBlipKind::ActiveObject,
                object_address: Some(0xA23C),
                erase_table_address: 0xB05D,
                screen_address: 0x4B11,
                color_word: 0x1234,
            }
        );
        assert_eq!(
            scanner.blips[1],
            super::ScannerRadarBlipSnapshot {
                kind: ScannerRadarBlipKind::InactiveObject,
                object_address: Some(0xA27C),
                erase_table_address: 0xB05F,
                screen_address: 0x4C14,
                color_word: 0xABCD,
            }
        );
        assert_eq!(
            scanner.player_blip,
            Some(super::ScannerRadarPlayerBlipSnapshot {
                erase_table_address: 0xB061,
                screen_address: 0x530F,
                body_word: 0x9099,
                tail_byte: 0x90,
                upper_byte: 0x09,
            })
        );
    }

    #[test]
    fn clean_game_projects_scanner_radar_sprites() {
        let mut details = [ObjectEvidenceDetailSnapshot::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT];
        details[0] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Active,
            world_position: Some((0x2100, 0x5000)),
            scanner_color: Some(0x1234),
            ..ObjectEvidenceDetailSnapshot::EMPTY
        };
        let evidence = ObjectEvidenceSnapshot {
            active_count: 1,
            inactive_count: 0,
            projectile_count: 0,
            visible_count: 1,
            evidence_crc32: None,
            detail_count: 1,
            details,
        };
        let mut game = Game::new();
        game.state.phase = GamePhase::Playing;
        game.state.world.scanner = ScannerRadarSnapshot::for_world(
            GamePhase::Playing,
            4,
            super::world_word(0x2000),
            (super::world_word(0x8000), super::world_word(0x4000)),
            &evidence,
        );

        let scene = game.scene();

        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_OBJECT_BLIP
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [150.0, 17.0]
                && sprite.size == [2.0, 2.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_PLAYER_BLIP
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [166.0, 15.0]
                && sprite.size == [3.0, 2.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_PLAYER_BLIP
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [164.0, 16.0]
                && sprite.size == [1.0, 1.0]
                && sprite.tint == Color::WHITE
        }));
    }

    #[test]
    fn clean_game_projects_source_object_detail_sprites() {
        let mut game = Game::new();
        game.state.phase = GamePhase::Playing;
        game.state.world.object_evidence = ObjectEvidenceSnapshot {
            active_count: 3,
            inactive_count: 1,
            projectile_count: 1,
            visible_count: 3,
            evidence_crc32: Some(0x0B1E_C7ED),
            detail_count: 4,
            details: {
                let mut details =
                    [ObjectEvidenceDetailSnapshot::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT];
                details[0] = ObjectEvidenceDetailSnapshot {
                    list: ObjectEvidenceList::Active,
                    screen_position: Some(ScreenPosition::new(40, 50)),
                    picture_size: Some((6, 4)),
                    mapped_sprite: Some(SpriteId::ENEMY_BAITER),
                    ..ObjectEvidenceDetailSnapshot::EMPTY
                };
                details[1] = ObjectEvidenceDetailSnapshot {
                    list: ObjectEvidenceList::Projectile,
                    screen_position: Some(ScreenPosition::new(60, 70)),
                    picture_size: Some((8, 1)),
                    mapped_sprite: Some(SpriteId::PLAYER_PROJECTILE),
                    ..ObjectEvidenceDetailSnapshot::EMPTY
                };
                details[2] = ObjectEvidenceDetailSnapshot {
                    list: ObjectEvidenceList::Inactive,
                    screen_position: Some(ScreenPosition::new(80, 90)),
                    picture_size: Some((2, 3)),
                    mapped_sprite: Some(SpriteId::ENEMY_BOMB),
                    ..ObjectEvidenceDetailSnapshot::EMPTY
                };
                details[3] = ObjectEvidenceDetailSnapshot {
                    list: ObjectEvidenceList::Active,
                    screen_position: Some(ScreenPosition::new(100, 110)),
                    picture_size: Some((1, 1)),
                    mapped_sprite: Some(SpriteId::NULL_OBJECT),
                    ..ObjectEvidenceDetailSnapshot::EMPTY
                };
                details
            },
        };

        let scene = game.scene();

        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_BAITER
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [40.0, 50.0]
                && sprite.size == [6.0, 4.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.layer == RenderLayer::Projectiles
                && sprite.position == [60.0, 70.0]
                && sprite.size == [8.0, 1.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(
            !scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_BOMB)
        );
        assert!(
            !scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::NULL_OBJECT)
        );
    }

    #[test]
    fn clean_game_projects_source_expanded_object_detail_sprites() {
        let mut game = Game::new();
        game.state.phase = GamePhase::Playing;
        game.state.world.expanded_objects = ExpandedObjectEvidenceSnapshot {
            active_count: 4,
            last_slot_address: Some(0x9C40),
            detail_count: 4,
            details: {
                let mut details =
                    [ExpandedObjectDetailSnapshot::EMPTY; EXPANDED_OBJECT_DETAIL_LIMIT];
                details[0] = ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::Appearance,
                    slot_address: Some(0x9C00),
                    descriptor_address: Some(0xF9C1),
                    picture_label: Some("PLAPIC"),
                    picture_size: Some((8, 6)),
                    mapped_sprite: Some(SpriteId::PLAYER_SHIP),
                    top_left: Some(ScreenPosition::new(10, 20)),
                    object_address: Some(0xA23C),
                    ..ExpandedObjectDetailSnapshot::EMPTY
                };
                details[1] = ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::Explosion,
                    slot_address: Some(0x9C40),
                    size: SOURCE_EXPLOSION_INITIAL_SIZE + SOURCE_EXPLOSION_SIZE_DELTA * 2,
                    descriptor_address: Some(0xF951),
                    picture_label: Some("BXPIC"),
                    picture_size: Some((4, 8)),
                    mapped_sprite: Some(SpriteId::BOMB_EXPLOSION),
                    top_left: Some(ScreenPosition::new(30, 40)),
                    explosion_frame: Some(2),
                    explosion_lifetime_frames: Some(SOURCE_EXPLOSION_LIFETIME_FRAMES),
                    ..ExpandedObjectDetailSnapshot::EMPTY
                };
                details[2] = ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::Appearance,
                    picture_size: Some((1, 1)),
                    mapped_sprite: Some(SpriteId::NULL_OBJECT),
                    top_left: Some(ScreenPosition::new(50, 60)),
                    ..ExpandedObjectDetailSnapshot::EMPTY
                };
                details[3] = ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::Explosion,
                    mapped_sprite: Some(SpriteId::ENEMY_BOMB),
                    top_left: Some(ScreenPosition::new(70, 80)),
                    ..ExpandedObjectDetailSnapshot::EMPTY
                };
                details
            },
        };

        let scene = game.scene();

        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_SHIP
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [10.0, 20.0]
                && sprite.size == [8.0, 6.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::BOMB_EXPLOSION
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [30.0, 40.0]
                && sprite.size == [8.0, 16.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(
            !scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::NULL_OBJECT)
        );
        assert!(!scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_BOMB && sprite.position == [70.0, 80.0]
        }));
    }

    #[test]
    fn clean_game_projects_score_popup_lifecycle_sprites() {
        let mut game = credited_started_game();
        game.state
            .world
            .spawn_score_popup(ScorePopupKind::Points500, ScreenPosition::new(33, 81));
        game.sync_world_presentation();

        let popup_detail = game.state.world.expanded_objects.details[0];
        assert_eq!(game.state.world.expanded_objects.active_count, 1);
        assert_eq!(game.state.world.expanded_objects.detail_count, 1);
        assert_eq!(
            popup_detail,
            ExpandedObjectDetailSnapshot {
                kind: ExpandedObjectKind::ScorePopup,
                picture_label: Some("C5P1"),
                picture_size: Some((6, 6)),
                mapped_sprite: Some(SpriteId::SCORE_POPUP_500),
                top_left: Some(ScreenPosition::new(33, 81)),
                score_popup_lifetime_ticks: Some(SOURCE_SCORE_POPUP_LIFETIME_TICKS),
                score_popup_value: Some(500),
                ..ExpandedObjectDetailSnapshot::EMPTY
            }
        );
        assert!(game.scene().sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_POPUP_500
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [33.0, 81.0]
                && sprite.size == [6.0, 6.0]
                && sprite.tint == Color::WHITE
        }));

        for _ in 0..SOURCE_SCORE_POPUP_LIFETIME_TICKS - 1 {
            game.step(GameInput::NONE);
        }
        assert_eq!(game.state.world.score_popups.len(), 1);
        assert!(game.scene().sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_POPUP_500 && sprite.position == [33.0, 81.0]
        }));

        game.step(GameInput::NONE);

        assert!(game.state.world.score_popups.is_empty());
        assert_eq!(game.state.world.expanded_objects.active_count, 0);
        assert_eq!(game.state.world.expanded_objects.detail_count, 0);
        assert!(
            !game
                .scene()
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::SCORE_POPUP_500)
        );
    }

    #[test]
    fn clean_game_projects_source_explosion_lifecycle_sprites() {
        let mut game = credited_started_game();
        game.state.world.enemies.clear();
        game.state
            .world
            .spawn_explosion(ExplosionKind::Bomb, ScreenPosition::new(33, 81));
        game.sync_world_presentation();

        let explosion_detail = game.state.world.expanded_objects.details[0];
        assert_eq!(game.state.world.expanded_objects.active_count, 1);
        assert_eq!(game.state.world.expanded_objects.detail_count, 1);
        assert_eq!(
            explosion_detail,
            ExpandedObjectDetailSnapshot {
                kind: ExpandedObjectKind::Explosion,
                size: SOURCE_EXPLOSION_INITIAL_SIZE,
                picture_label: Some("BXPIC"),
                picture_size: Some((4, 8)),
                mapped_sprite: Some(SpriteId::BOMB_EXPLOSION),
                center: Some(ScreenPosition::new(35, 85)),
                top_left: Some(ScreenPosition::new(33, 81)),
                explosion_frame: Some(0),
                explosion_lifetime_frames: Some(SOURCE_EXPLOSION_LIFETIME_FRAMES),
                ..ExpandedObjectDetailSnapshot::EMPTY
            }
        );
        assert!(game.scene().sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::BOMB_EXPLOSION
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [33.0, 81.0]
                && sprite.size == [4.0, 8.0]
                && sprite.tint == Color::WHITE
        }));

        game.state.world.advance_explosions();
        game.state.world.advance_explosions();
        game.sync_world_presentation();

        let advanced_detail = game.state.world.expanded_objects.details[0];
        assert_eq!(
            advanced_detail.size,
            SOURCE_EXPLOSION_INITIAL_SIZE + SOURCE_EXPLOSION_SIZE_DELTA * 2
        );
        assert_eq!(advanced_detail.explosion_frame, Some(2));
        assert!(game.scene().sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::BOMB_EXPLOSION
                && sprite.position == [33.0, 81.0]
                && sprite.size == [8.0, 16.0]
        }));

        for _ in 2..SOURCE_EXPLOSION_LIFETIME_FRAMES {
            game.state.world.advance_explosions();
        }
        game.sync_world_presentation();

        assert!(game.state.world.explosions.is_empty());
        assert_eq!(game.state.world.expanded_objects.active_count, 0);
        assert_eq!(game.state.world.expanded_objects.detail_count, 0);
        assert!(
            !game
                .scene()
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::BOMB_EXPLOSION)
        );
    }

    #[test]
    fn clean_game_projects_player_explosion_cloud_sprites() {
        let mut game = Game::new();
        game.state.world.player_explosion = Some(PlayerExplosionCloudSnapshot {
            source_color: 0xFF,
            source_color_counter: 55,
            source_color_index: 0,
            frame: 1,
            piece_count: 2,
            pieces: {
                let mut pieces =
                    [PlayerExplosionPieceSnapshot::EMPTY; PLAYER_EXPLOSION_PIECE_LIMIT];
                pieces[0] = PlayerExplosionPieceSnapshot {
                    position: ScreenPosition::new(40, 70),
                    split: false,
                };
                pieces[1] = PlayerExplosionPieceSnapshot {
                    position: ScreenPosition::new(50, 80),
                    split: true,
                };
                pieces
            },
        });

        let scene = game.scene();

        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [40.0, 70.0]
                && sprite.size == [4.0, 1.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [50.0, 80.0]
                && sprite.size == [4.0, 2.0]
                && sprite.tint == Color::WHITE
        }));
    }

    #[test]
    fn clean_world_maps_source_explosion_descriptor_families() {
        for (kind, picture_label, picture_size, mapped_sprite, center) in [
            (
                ExplosionKind::Lander,
                "LNDP1",
                (5, 8),
                SpriteId::ENEMY_LANDER,
                ScreenPosition::new(12, 24),
            ),
            (
                ExplosionKind::Bomb,
                "BXPIC",
                (4, 8),
                SpriteId::BOMB_EXPLOSION,
                ScreenPosition::new(12, 24),
            ),
            (
                ExplosionKind::Swarmer,
                "SWXP1",
                (4, 8),
                SpriteId::SWARMER_EXPLOSION,
                ScreenPosition::new(12, 24),
            ),
            (
                ExplosionKind::Astronaut,
                "ASXP1",
                (4, 8),
                SpriteId::ASTRONAUT_EXPLOSION,
                ScreenPosition::new(12, 24),
            ),
            (
                ExplosionKind::PlayerShip,
                "PLAPIC",
                (8, 6),
                SpriteId::PLAYER_SHIP,
                ScreenPosition::new(14, 23),
            ),
            (
                ExplosionKind::Terrain,
                "TEREX",
                (8, 6),
                SpriteId::TERRAIN_EXPLOSION,
                ScreenPosition::new(14, 23),
            ),
        ] {
            let mut world = WorldSnapshot::default();
            world.spawn_explosion(kind, ScreenPosition::new(10, 20));
            world.sync_clean_lifecycle_evidence();

            assert_eq!(world.expanded_objects.active_count, 1);
            assert_eq!(world.expanded_objects.detail_count, 1);
            assert_eq!(
                world.expanded_objects.details[0],
                ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::Explosion,
                    size: SOURCE_EXPLOSION_INITIAL_SIZE,
                    picture_label: Some(picture_label),
                    picture_size: Some(picture_size),
                    mapped_sprite: Some(mapped_sprite),
                    center: Some(center),
                    top_left: Some(ScreenPosition::new(10, 20)),
                    explosion_frame: Some(0),
                    explosion_lifetime_frames: Some(SOURCE_EXPLOSION_LIFETIME_FRAMES),
                    ..ExpandedObjectDetailSnapshot::EMPTY
                }
            );
        }
    }

    #[test]
    fn clean_world_starts_source_terrain_blow_and_projects_terex() {
        let mut game = credited_started_game();
        game.state.world.humans.clear();

        let frame = game.step(GameInput::NONE);

        let terrain_blow = frame
            .state
            .world
            .terrain_blow
            .expect("terrain blow snapshot");
        assert!(terrain_blow.status_terrain_blown);
        assert_eq!(terrain_blow.stage, TerrainBlowStage::ExplosionPassSleeping);
        assert_eq!(terrain_blow.source_iteration, 0);
        assert_eq!(terrain_blow.source_sleep_remaining, Some(2));
        assert_eq!(terrain_blow.source_pseudo_color, 0x3C);
        assert_eq!(terrain_blow.source_overload_counter, 8);
        assert_eq!(terrain_blow.terrain_erase_entries, 0x98);
        assert_eq!(terrain_blow.scanner_terrain_erase_entries, 0x40);
        assert!(terrain_blow.terrain_erased());
        assert!(terrain_blow.scanner_terrain_erased());
        assert_eq!(terrain_blow.explosions_per_pass, 2);
        assert!(frame.state.world.terrain.is_empty());
        assert!(!frame.state.world.scanner.terrain_enabled);
        assert_eq!(frame.state.world.expanded_objects.detail_count, 2);
        assert!(
            frame
                .state
                .world
                .expanded_objects
                .details
                .iter()
                .take(2)
                .all(|detail| detail.picture_label == Some("TEREX")
                    && detail.mapped_sprite == Some(SpriteId::TERRAIN_EXPLOSION)
                    && detail.explosion_frame == Some(0))
        );
        assert!(
            !frame
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::TERRAIN_TILE)
        );
        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TERRAIN_EXPLOSION
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [0x44 as f32, 0x70 as f32]
                && sprite.size == [8.0, 6.0]
        }));
        assert_eq!(
            frame.events.sounds(),
            &[source_terrain_blow_start_sound_event()]
        );
    }

    #[test]
    fn clean_world_advances_source_terrain_blow_to_completion_sound() {
        let mut game = credited_started_game();
        game.state.world.humans.clear();
        game.state.world.enemies = vec![EnemySnapshot::new(
            EnemyKind::Pod,
            ScreenPosition::new(200, 80),
            ScreenVelocity::new(0, 0),
        )];
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();

        let start = game.step(GameInput::NONE);
        assert_eq!(
            start.events.sounds(),
            &[source_terrain_blow_start_sound_event()]
        );

        let mut saw_flash_clear = false;
        let mut saw_restart_pass = false;
        let mut saw_completion = false;
        for _ in 0..96 {
            let frame = game.step(GameInput::NONE);
            let terrain_blow = frame
                .state
                .world
                .terrain_blow
                .expect("terrain blow snapshot");

            match terrain_blow.stage {
                TerrainBlowStage::ExplosionPassSleeping if terrain_blow.source_iteration > 0 => {
                    if frame.events.sounds() == [source_terrain_blow_start_sound_event()].as_slice()
                    {
                        saw_restart_pass = true;
                        assert_eq!(
                            terrain_blow.source_sleep_remaining,
                            Some(SOURCE_TERRAIN_BLOW_SLEEP_TICKS)
                        );
                        assert_ne!(terrain_blow.source_pseudo_color, 0);
                        assert_eq!(
                            terrain_blow.source_overload_counter,
                            SOURCE_TERRAIN_BLOW_OVERLOAD_COUNTER
                        );
                    }
                }
                TerrainBlowStage::FlashClearedSleeping => {
                    saw_flash_clear = true;
                    assert_eq!(terrain_blow.source_pseudo_color, 0);
                    assert!(matches!(terrain_blow.source_sleep_remaining, Some(1..=3)));
                    assert!(frame.events.sounds().is_empty());
                }
                TerrainBlowStage::Completed => {
                    saw_completion = true;
                    assert_eq!(
                        terrain_blow.source_iteration,
                        SOURCE_TERRAIN_BLOW_ITERATION_LIMIT
                    );
                    assert_eq!(terrain_blow.source_sleep_remaining, None);
                    assert_eq!(
                        frame.events.sounds(),
                        &[source_terrain_blow_complete_sound_event()]
                    );
                    break;
                }
                TerrainBlowStage::ExplosionPassSleeping => {}
            }
        }

        assert!(saw_flash_clear);
        assert!(saw_restart_pass);
        assert!(saw_completion);
    }

    #[test]
    fn clean_game_highlights_carried_humans() {
        let mut game = credited_started_game();
        game.state.world.humans[0].carried = true;

        let frame = game.step(GameInput::NONE);

        assert!(frame.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.tint == Color::from_rgba(0xFF, 0xF8, 0x80, 0xFF)
        }));
    }

    #[test]
    fn clean_game_reverses_left_to_right() {
        let mut game = credited_started_game();
        game.state.player.direction = Direction::Left;

        let frame = game.step(GameInput {
            reverse: true,
            ..GameInput::NONE
        });

        assert_eq!(frame.state.player.direction, Direction::Right);
        assert_eq!(frame.events.gameplay(), &[GameEvent::ReversePressed]);
    }

    #[test]
    fn clean_game_implements_simulation_trait() {
        let mut game = Game::new();

        let frame = advance_one_frame(
            &mut game,
            GameInput {
                coin: true,
                ..GameInput::NONE
            },
        );

        assert_eq!(GameSimulation::state(&game).frame, 1);
        assert_eq!(frame.state.credits, 0);
        assert!(frame.events.is_empty());
    }

    #[test]
    fn clean_game_counts_down_delayed_sound_timers() {
        let mut game = Game::new();

        game.coin_one_sound_delay = Some(2);
        let frame = game.step(GameInput::NONE);
        assert!(frame.events.sounds().is_empty());
        assert_eq!(game.coin_one_sound_delay, Some(1));
        let frame = game.step(GameInput::NONE);
        assert_eq!(frame.events.sounds(), &[SoundEvent::CreditAdded]);
        assert_eq!(game.coin_one_sound_delay, None);

        game.start_sound_delay = Some(2);
        let frame = game.step(GameInput::NONE);
        assert!(frame.events.sounds().is_empty());
        assert_eq!(game.start_sound_delay, Some(1));
        let frame = game.step(GameInput::NONE);
        assert_eq!(frame.events.sounds(), &[SoundEvent::GameStarted]);
        assert_eq!(game.start_sound_delay, None);
    }

    fn credited_started_game() -> Game {
        let mut game = Game::new();
        advance_to_delayed_credit(&mut game);
        advance_to_delayed_credit_sound(&mut game);
        game.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        advance_to_delayed_start_sound(&mut game);
        advance_to_started_playfield(&mut game);
        game
    }

    fn keep_single_grounded_human(game: &mut Game) {
        let x = 0x40;
        let y = super::clean_human_ground_y(&game.state.world.terrain, x).unwrap_or(0xC0);
        game.state.world.humans = vec![HumanSnapshot::new(ScreenPosition::new(x, y))];
    }

    fn two_player_started_game() -> Game {
        let mut game = Game::new();
        game.state.credits = 2;
        game.step(GameInput {
            start_two: true,
            ..GameInput::NONE
        });
        advance_to_delayed_start_sound(&mut game);
        advance_to_started_playfield(&mut game);
        game
    }

    fn keep_first_enemy_only(game: &mut Game) {
        game.state.world.enemies.truncate(1);
        game.state.world.enemy_reserve = EnemyReserveSnapshot::default();
    }

    fn table_entry(rank: u8, score: u32, initials: [char; 3]) -> HighScoreTableEntrySnapshot {
        HighScoreTableEntrySnapshot {
            rank,
            score,
            initials: initials.map(Some),
        }
    }

    fn advance_player_switch_sleep(game: &mut Game, from_player: u8, to_player: u8) -> GameFrame {
        for expected_timer in (1..PLAYER_SWITCH_SLEEP_FRAMES).rev() {
            let sleeping = game.step(GameInput::NONE);
            assert_eq!(sleeping.state.phase, GamePhase::GameOver);
            assert_eq!(
                sleeping.state.game_over,
                GameOverSnapshot::player_switch_sleep(expected_timer, from_player, to_player)
            );
            assert!(sleeping.events.is_empty());
        }

        game.step(GameInput::NONE)
    }

    fn advance_pending_respawn(game: &mut Game) -> GameFrame {
        for _ in 0..200 {
            let frame = game.step(GameInput::NONE);
            if frame.state.phase == GamePhase::Playing {
                return frame;
            }

            assert_eq!(frame.state.phase, GamePhase::GameOver);
            assert_eq!(frame.state.game_over, GameOverSnapshot::NONE);
            assert!(frame.events.is_empty());
        }

        panic!("pending respawn did not finish");
    }

    fn advance_player_death_game_over_sleep(game: &mut Game) -> GameFrame {
        for expected_timer in (1..PLAYER_DEATH_GAME_OVER_SLEEP_FRAMES).rev() {
            let sleeping = game.step(GameInput::NONE);
            assert_eq!(sleeping.state.phase, GamePhase::GameOver);
            assert_eq!(
                sleeping.state.game_over,
                GameOverSnapshot::player_death_sleep(expected_timer)
            );
            assert!(sleeping.events.is_empty());
        }

        game.step(GameInput::NONE)
    }

    fn advance_to_delayed_credit(game: &mut Game) -> GameFrame {
        game.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        for _ in 0..=COIN_CREDIT_DELAY_FRAMES {
            let frame = game.step(GameInput::NONE);
            if frame.events.gameplay() == [GameEvent::CreditAdded] {
                return frame;
            }
        }
        panic!("delayed credit was not awarded");
    }

    fn advance_to_delayed_credit_sound(game: &mut Game) -> GameFrame {
        for _ in 0..=COIN_CREDIT_SOUND_DELAY_FRAMES {
            let frame = game.step(GameInput::NONE);
            if frame.events.sounds() == [SoundEvent::CreditAdded] {
                return frame;
            }
        }
        panic!("delayed credit sound was not emitted");
    }

    fn advance_to_delayed_start_sound(game: &mut Game) -> GameFrame {
        for _ in 0..=START_SOUND_DELAY_FRAMES {
            let frame = game.step(GameInput::NONE);
            if frame.events.sounds() == [SoundEvent::GameStarted] {
                return frame;
            }
        }
        panic!("delayed start sound was not emitted");
    }

    fn advance_to_started_playfield(game: &mut Game) -> GameFrame {
        for _ in 0..=START_PLAYFIELD_DELAY_FRAMES {
            let frame = game.step(GameInput::NONE);
            if !frame.state.world.enemies.is_empty() {
                return frame;
            }
        }
        panic!("started playfield was not activated");
    }
}
