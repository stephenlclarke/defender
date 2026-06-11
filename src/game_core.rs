use std::sync::OnceLock;

use crate::{
    renderer::{Color, RenderLayer, RenderScene, SceneSprite, SpriteId, screen_position_from_cell},
    systems::{HighScoreInitialsState, ScreenPosition, ScreenVelocity},
};

// Evidence-backed cabinet defaults from CMOS/high-score evidence.
pub const HIGH_SCORE_TABLE_ENTRIES: usize = 8;
const HALL_OF_FAME_STALL_STEPS: u8 = 60;
const FIRST_WAVE_MUTANT_DIVE_CONVERSION_X_CORRECTION: u16 = 0x0120;

const ATTRACT_PRESENTS_START_STEP: u16 = 236;
const ATTRACT_HALL_OF_FAME_START_STEP: u16 = 600;
const ATTRACT_COPYRIGHT_START_STEP: u16 = ATTRACT_HALL_OF_FAME_START_STEP;
const ATTRACT_HALL_OF_FAME_STALL_INTERVAL_STEPS: u16 = 10;
pub(crate) const ATTRACT_SCORING_SEQUENCE_START_STEP: u16 = ATTRACT_HALL_OF_FAME_START_STEP
    + (HALL_OF_FAME_STALL_STEPS as u16 * ATTRACT_HALL_OF_FAME_STALL_INTERVAL_STEPS);
const ATTRACT_LOGO_SLEEP_TICKS: u8 = 2;
const ATTRACT_PRESENTS_SLEEP_TICKS: u8 = 5;
const ATTRACT_DEFENDER_ENTRY_SLEEP_TICKS: u8 = 0x30;
const ATTRACT_COPYRIGHT_SLEEP_TICKS: u8 = 10;
const ATTRACT_COPYRIGHT_STALL_TICKS: u8 = 60;
const ATTRACT_INSTRUCTION_ENTRY_SLEEP_TICKS: u8 = 0xE6;
pub(crate) const ATTRACT_DEFENDER_WORDMARK_START_STEP: u16 = 365;
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
const ATTRACT_CYCLE_STEPS: u16 =
    ATTRACT_SCORING_SEQUENCE_START_STEP + ATTRACT_SCORING_DEMO_TOTAL_TICKS;
const COLTAB_COLOR_BYTES: [u8; 37] = [
    0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x37, 0x2F, 0x27, 0x1F, 0x17, 0x47, 0x47, 0x87,
    0x87, 0xC7, 0xC7, 0xC6, 0xC5, 0xCC, 0xCB, 0xCA, 0xDA, 0xE8, 0xF8, 0xF9, 0xFA, 0xFB, 0xFD, 0xFF,
    0xBF, 0x3F, 0x3E, 0x3C, 0x00,
];
const TIE_COLOR_BYTES: [u8; 9] = [0x81, 0x81, 0x2F, 0x81, 0x2F, 0x07, 0x2F, 0x81, 0x07];
const COLTAB_ACTIVE_BYTES: usize = COLTAB_COLOR_BYTES.len() - 1;
const ATTRACT_TITLE_REFERENCE_SAMPLE_INTERVAL_STEPS: u16 = 8;
const ATTRACT_TITLE_REFERENCE_LOGO_COLOR_BYTES: [u8; 59] = [
    0x00, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x2F, 0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x2F,
    0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07,
    0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x2F, 0x2F,
    0x07, 0x07, 0x07, 0x07, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x07, 0x2F,
];
const ATTRACT_WILLIAMS_TIE_COLOR_PRIME_STEPS: u16 = 6;
const ATTRACT_WILLIAMS_TIE_COLOR_SLEEP_STEPS: u16 = 6;
const ATTRACT_WILLIAMS_TIE_COLOR_SLOT_OFFSET: usize = 2;
const SCANNER_TERRAIN_PIXEL_SIZE: [f32; 2] = [1.0, 1.0];
const SCANNER_TERRAIN_TINT: Color = Color::from_rgba(174, 81, 0, 255);
const WAVE_LANDSCAPE_COLOR_BYTES: [u8; 8] = [0x81, 0x28, 0x07, 0x2F, 0x3F, 0x87, 0x15, 0x00];
const WILLIAMS_RED_GREEN_LEVELS: [u8; 8] = [0, 38, 81, 118, 137, 174, 217, 255];
const WILLIAMS_BLUE_LEVELS: [u8; 4] = [0, 95, 160, 255];
const NORMAL_PALETTE_BYTES: [u8; 16] = [
    0x00, 0x00, 0x07, 0x28, 0x2F, 0x81, 0xA4, 0x15, 0xC7, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const TERRAIN_TDATA_BYTES: usize = 0x100;
const TERRAIN_PATTERN_STREAM_BASE: u16 = 0xC350;
const MAIN_TERRAIN_RECORD_BYTE_COUNT: usize = 0x180;
const TERRAIN_FLAVOR_RECORDS: usize = 0x98;
const TERRAIN_SCREEN_WORDS: usize = 0x98;
const SCANNER_TERRAIN_RECORDS: usize = 0x40;
const SCANNER_MINI_TERRAIN_RECORDS: usize = MAIN_TERRAIN_RECORD_BYTE_COUNT / 3;
const TERRAIN_WORD_7007: u16 = 0x7007;
const TERRAIN_WORD_0770: u16 = 0x0770;
const TERRAIN_WORD_SIZE: [f32; 2] = [2.0, 2.0];
const SCANNER_PROCESS_SLEEP_TICKS: [u8; 3] = [2, 2, 4];
const SCANNER_SELECTED_MAP: u8 = 1;
const SCANNER_OBJECT_BASE_SCREEN: u16 = 0x3008;
const SCANNER_SCAN_CENTER_OFFSET: u16 = 0x6D40;
const SCANNER_OBJECT_ERASE_START: u16 = 0xB05D;
const SCANNER_PLAYER_BASE_SCREEN: u16 = 0x4B07;
const SCANNER_PLAYER_BODY_WORD: u16 = 0x9099;
const SCANNER_PLAYER_TAIL_BYTE: u8 = 0x90;
const SCANNER_PLAYER_UPPER_BYTE: u8 = 0x09;
const SCANNER_LANDER_COLOR_WORD: u16 = 0x4433;
const SCANNER_HUMAN_COLOR_WORD: u16 = 0x6666;
pub(crate) const SCORE_POPUP_LIFETIME_TICKS: u8 = 50;
pub(crate) const EXPLOSION_INITIAL_SIZE: u16 = 0x0100;
pub(crate) const EXPLOSION_SIZE_DELTA: u16 = 0x00AA;
pub(crate) const EXPLOSION_KILL_SIZE_HIGH: u8 = 0x30;
pub(crate) const EXPLOSION_LIFETIME_STEPS: u8 = 73;
const APPEARANCE_INITIAL_SIZE: u16 = 0xAD00;
const APPEARANCE_FINAL_SIZE: u16 = 0x8000;
pub(crate) const TERRAIN_BLOW_STATUS_BIT: u8 = 0x02;
pub(crate) const TERRAIN_BLOW_ITERATION_LIMIT: u8 = 16;
pub(crate) const TERRAIN_BLOW_EXPLOSIONS_PER_PASS: u8 = 2;
pub(crate) const TERRAIN_BLOW_OVERLOAD_COUNTER: u8 = 8;
pub(crate) const TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES: u16 = 0x98;
pub(crate) const TERRAIN_BLOW_SCANNER_ERASE_ENTRIES: u16 = 0x40;
pub(crate) const TERRAIN_BLOW_COMPLETE_STEP: u16 = 111;
pub(crate) const TERRAIN_EXPLOSION_LIFETIME_STEPS: u8 = 81;
const TERRAIN_EXPLOSION_GROWTH_STEPS: [u8;
    TERRAIN_EXPLOSION_LIFETIME_STEPS
    as usize] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 22, 23, 24,
    25, 26, 26, 27, 28, 29, 29, 30, 31, 31, 32, 32, 33, 34, 34, 35, 36, 36, 37, 37, 38, 39, 39, 40,
    41, 41, 42, 43, 43, 44, 44, 45, 45, 46, 47, 47, 48, 48, 49, 49, 50, 50, 51, 51, 52, 52, 52, 53,
    53, 54, 54, 55, 55, 56, 56,
];
pub(crate) const TERRAIN_BLOW_START_SOUND_STEPS: [u16; 16] =
    [1, 4, 9, 13, 17, 21, 26, 32, 38, 44, 52, 61, 71, 82, 93, 101];
pub(crate) const TERRAIN_BLOW_EXPLOSION_BIRTHS: [(u16, ScreenPosition); 17] = [
    (0, ScreenPosition::new(0x4C, 0xC2)),
    (4, ScreenPosition::new(0x14, 0xE2)),
    (4, ScreenPosition::new(0x5C, 0xDE)),
    (8, ScreenPosition::new(0x80, 0xDE)),
    (12, ScreenPosition::new(0x00, 0xE0)),
    (16, ScreenPosition::new(0x68, 0xDC)),
    (21, ScreenPosition::new(0x30, 0xE0)),
    (26, ScreenPosition::new(0x80, 0xDE)),
    (31, ScreenPosition::new(0x44, 0xD2)),
    (31, ScreenPosition::new(0x50, 0xC6)),
    (51, ScreenPosition::new(0x20, 0xE2)),
    (52, ScreenPosition::new(0x70, 0xD8)),
    (60, ScreenPosition::new(0x6C, 0xD4)),
    (60, ScreenPosition::new(0x28, 0xE0)),
    (70, ScreenPosition::new(0x94, 0xDC)),
    (70, ScreenPosition::new(0x00, 0xE0)),
    (81, ScreenPosition::new(0x0C, 0xE2)),
];
pub(crate) const TERRAIN_BLOW_FLASH_COLOR_BYTES: [u8; 16] = [
    0xC6, 0xCA, 0xC6, 0xC7, 0xF8, 0x87, 0x38, 0xC5, 0xE8, 0x47, 0xFA, 0x27, 0x38, 0x47, 0xCC, 0xDA,
];
const TERRAIN_BLOW_FLASH_WINDOWS: [(u16, u16, u8); 16] = [
    (2, 3, 0xC6),
    (6, 7, 0xCA),
    (10, 11, 0xC6),
    (14, 15, 0xC7),
    (18, 20, 0xF8),
    (23, 24, 0x87),
    (28, 29, 0x38),
    (33, 35, 0xC5),
    (39, 42, 0xE8),
    (46, 48, 0x47),
    (53, 56, 0xFA),
    (62, 65, 0x27),
    (72, 76, 0x38),
    (83, 87, 0x47),
    (94, 98, 0xCC),
    (103, 106, 0xDA),
];
pub const PLAYER_EXPLOSION_PIECE_LIMIT: usize = 128;

pub(crate) const VISUAL_STATE: VisualStateSnapshot = VisualStateSnapshot {
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
    pub page_step: u16,
    pub page: AttractPresentationPage,
    pub stage_sleep_ticks: Option<u8>,
    pub stall_ticks: Option<u8>,
}

impl AttractPresentationSnapshot {
    pub const INACTIVE: Self = Self {
        page_step: 0,
        page: AttractPresentationPage::Inactive,
        stage_sleep_ticks: None,
        stall_ticks: None,
    };

    pub fn for_page_step(page_step: u16) -> Self {
        let page_step = if ATTRACT_CYCLE_STEPS == 0 {
            page_step
        } else {
            page_step % ATTRACT_CYCLE_STEPS
        };
        let (page, stage_sleep_ticks, stall_ticks) =
            if page_step >= ATTRACT_SCORING_SEQUENCE_START_STEP {
                (
                    AttractPresentationPage::ScoringSequence,
                    Some(ATTRACT_INSTRUCTION_ENTRY_SLEEP_TICKS),
                    None,
                )
            } else if page_step >= ATTRACT_HALL_OF_FAME_START_STEP {
                (
                    AttractPresentationPage::HallOfFame,
                    None,
                    Some(HALL_OF_FAME_STALL_STEPS),
                )
            } else if page_step >= ATTRACT_COPYRIGHT_START_STEP {
                (
                    AttractPresentationPage::CopyrightWait,
                    Some(ATTRACT_COPYRIGHT_SLEEP_TICKS),
                    Some(ATTRACT_COPYRIGHT_STALL_TICKS),
                )
            } else if page_step >= ATTRACT_DEFENDER_WORDMARK_START_STEP {
                (
                    AttractPresentationPage::DefenderWordmark,
                    Some(ATTRACT_DEFENDER_ENTRY_SLEEP_TICKS),
                    None,
                )
            } else if page_step >= ATTRACT_PRESENTS_START_STEP {
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
            page_step,
            page,
            stage_sleep_ticks,
            stall_ticks,
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

    pub const fn scoring_sequence_step(self) -> Option<u16> {
        if matches!(self.page, AttractPresentationPage::ScoringSequence) {
            Some(
                self.page_step
                    .saturating_sub(ATTRACT_SCORING_SEQUENCE_START_STEP),
            )
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PostGamePlayfieldSnapshot {
    pub step: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VisualStateSnapshot {
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

impl VisualStateSnapshot {
    pub(crate) const fn hud_tint(self) -> Color {
        let _top_display_border_word = self.top_display_border_word;
        Color::WHITE
    }

    pub(crate) fn top_display_scanner_marker_tint(self) -> Color {
        video_word_tint(self.top_display_scanner_marker_word)
    }

    pub(crate) fn scanner_object_blip_tint(self, color_word: u16) -> Color {
        video_word_tint(color_word)
    }

    pub(crate) fn scanner_player_blip_tint(self, body_word: u16) -> Color {
        video_word_tint(body_word)
    }

    pub(crate) fn attract_williams_logo_tint_for_step(self, page_step: u16) -> Color {
        if let Some(color_byte) = ATTRACT_TITLE_REFERENCE_LOGO_COLOR_BYTES
            .get(attract_title_sample_index(page_step))
            .copied()
            .filter(|color_byte| *color_byte != 0)
        {
            return williams_color_byte_tint(color_byte);
        }

        let color_cycle_tick = page_step.saturating_sub(ATTRACT_WILLIAMS_TIE_COLOR_PRIME_STEPS)
            / ATTRACT_WILLIAMS_TIE_COLOR_SLEEP_STEPS.max(1);
        let tie_triplet = usize::from(color_cycle_tick % 3) * 3;
        williams_color_byte_tint(
            TIE_COLOR_BYTES[tie_triplet + ATTRACT_WILLIAMS_TIE_COLOR_SLOT_OFFSET],
        )
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
    pub lander_y_velocity: u16,
    pub mutant_random_y: u8,
    pub mutant_y_velocity: u16,
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
            landers: wave_table_u8(crate::arcade_assets::WaveMetric::Landers, wave),
            bombers: wave_table_u8(crate::arcade_assets::WaveMetric::Bombers, wave),
            pods: wave_table_u8(crate::arcade_assets::WaveMetric::Pods, wave),
            mutants: wave_table_u8(crate::arcade_assets::WaveMetric::Mutants, wave),
            swarmers: wave_table_u8(crate::arcade_assets::WaveMetric::Swarmers, wave),
            lander_x_velocity: wave_table_u8(crate::arcade_assets::WaveMetric::LanderXVelocity, wave),
            lander_y_velocity: wave_table_u16(
                crate::arcade_assets::WaveMetric::LanderYVelocityHigh,
                crate::arcade_assets::WaveMetric::LanderYVelocityLow,
                wave,
            ),
            mutant_random_y: wave_table_u8(crate::arcade_assets::WaveMetric::MutantRandomY, wave),
            mutant_y_velocity: wave_table_u16(
                crate::arcade_assets::WaveMetric::MutantYVelocityHigh,
                crate::arcade_assets::WaveMetric::MutantYVelocityLow,
                wave,
            ),
            mutant_x_velocity: wave_table_u8(crate::arcade_assets::WaveMetric::MutantXVelocity, wave),
            swarmer_x_velocity: wave_table_u8(crate::arcade_assets::WaveMetric::SwarmerXVelocity, wave),
            wave_time: wave_table_u32(crate::arcade_assets::WaveMetric::WaveTime, wave),
            wave_size: wave_table_u8(crate::arcade_assets::WaveMetric::WaveSize, wave),
            lander_shot_time: wave_table_u32(crate::arcade_assets::WaveMetric::LanderShotTime, wave),
            bomber_x_velocity: wave_table_u8(crate::arcade_assets::WaveMetric::BomberXVelocity, wave),
            mutant_shot_time: wave_table_u32(crate::arcade_assets::WaveMetric::MutantShotTime, wave),
            swarmer_shot_time: wave_table_u32(crate::arcade_assets::WaveMetric::SwarmerShotTime, wave),
            swarmer_acceleration_mask: wave_table_u8(
                crate::arcade_assets::WaveMetric::SwarmerAccelerationMask,
                wave,
            ),
            baiter_delay: wave_table_u32(crate::arcade_assets::WaveMetric::BaiterDelay, wave),
            baiter_shot_time: wave_table_u32(crate::arcade_assets::WaveMetric::BaiterShotTime, wave),
            baiter_seek_probability: wave_table_u8(
                crate::arcade_assets::WaveMetric::BaiterSeekProbability,
                wave,
            ),
        }
    }
}

const DEFAULT_DIFFICULTY_INITIAL: u8 = 5;
const DEFAULT_DIFFICULTY_CEILING: u8 = 15;

fn wave_table_u8(metric: crate::arcade_assets::WaveMetric, wave: u8) -> u8 {
    u8::try_from(wave_table_value(metric, wave)).expect("wave profile value should fit u8")
}

fn wave_table_u16(
    high_metric: crate::arcade_assets::WaveMetric,
    low_metric: crate::arcade_assets::WaveMetric,
    wave: u8,
) -> u16 {
    u16::from_be_bytes([
        wave_table_u8(high_metric, wave),
        wave_table_u8(low_metric, wave),
    ])
}

fn wave_table_u32(metric: crate::arcade_assets::WaveMetric, wave: u8) -> u32 {
    u32::try_from(wave_table_value(metric, wave)).expect("wave profile value should be non-negative")
}

fn wave_table_value(metric: crate::arcade_assets::WaveMetric, wave: u8) -> i32 {
    crate::arcade_assets::wave_metric_value(
        metric,
        wave,
        wave_tuning_difficulty_iterations(wave.max(1)),
    )
}

fn wave_tuning_difficulty_iterations(wave: u8) -> u16 {
    let wave_delta = wave.saturating_sub(4);
    let pre_ceiling = DEFAULT_DIFFICULTY_INITIAL.saturating_add(wave_delta);
    u16::from(pre_ceiling.min(DEFAULT_DIFFICULTY_CEILING))
}
