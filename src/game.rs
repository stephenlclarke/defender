//! Domain-facing gameplay contracts.

use std::sync::OnceLock;

use crate::{
    renderer::{Color, RenderLayer, RenderScene, SceneSprite, SpriteId, source_screen_position},
    systems::{HighScoreInitialsState, ScreenPosition, ScreenVelocity},
};

// Source-backed cabinet defaults from assets/red-label CMOS/high-score evidence.
pub const HIGH_SCORE_TABLE_ENTRIES: usize = 8;
const HALL_OF_FAME_STALL_FRAMES: u8 = 60;
const FIRST_WAVE_TARGET6_MUTANT_CONVERSION_X_CORRECTION: u16 = 0x0120; // original: SOURCE_FIRST_WAVE_TARGET6_MUTANT_CONVERSION_X_CORRECTION

const ATTRACT_PRESENTS_START_FRAME: u16 = 236;
const ATTRACT_HALL_OF_FAME_START_FRAME: u16 = 600;
const ATTRACT_COPYRIGHT_START_FRAME: u16 = ATTRACT_HALL_OF_FAME_START_FRAME;
const ATTRACT_HALL_OF_FAME_STALL_TICK_FRAMES: u16 = 10;
pub(crate) const ATTRACT_SCORING_SEQUENCE_START_FRAME: u16 = ATTRACT_HALL_OF_FAME_START_FRAME
    + (HALL_OF_FAME_STALL_FRAMES as u16 * ATTRACT_HALL_OF_FAME_STALL_TICK_FRAMES);
const ATTRACT_LOGO_SLEEP_TICKS: u8 = 2;
const ATTRACT_PRESENTS_SLEEP_TICKS: u8 = 5;
const ATTRACT_DEFENDER_ENTRY_SLEEP_TICKS: u8 = 0x30;
const ATTRACT_COPYRIGHT_SLEEP_TICKS: u8 = 10;
const ATTRACT_COPYRIGHT_STALL_TICKS: u8 = 60;
const ATTRACT_INSTRUCTION_ENTRY_SLEEP_TICKS: u8 = 0xE6;
pub(crate) const ATTRACT_DEFENDER_WORDMARK_START_FRAME: u16 = 365;
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
const ATTRACT_CYCLE_FRAME_COUNT: u16 =
    ATTRACT_SCORING_SEQUENCE_START_FRAME + ATTRACT_SCORING_DEMO_TOTAL_TICKS;
const COLTAB_COLOR_BYTES: [u8; 37] = [
    0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x37, 0x2F, 0x27, 0x1F, 0x17, 0x47, 0x47, 0x87,
    0x87, 0xC7, 0xC7, 0xC6, 0xC5, 0xCC, 0xCB, 0xCA, 0xDA, 0xE8, 0xF8, 0xF9, 0xFA, 0xFB, 0xFD, 0xFF,
    0xBF, 0x3F, 0x3E, 0x3C, 0x00,
];
const TIE_COLOR_BYTES: [u8; 9] = [0x81, 0x81, 0x2F, 0x81, 0x2F, 0x07, 0x2F, 0x81, 0x07]; // original: SOURCE_TIE_COLOR_BYTES
const COLTAB_ACTIVE_BYTES: usize = COLTAB_COLOR_BYTES.len() - 1; // original: SOURCE_COLTAB_ACTIVE_BYTES
const ATTRACT_TITLE_REFERENCE_SAMPLE_STEP_FRAMES: u16 = 8;
const ATTRACT_TITLE_REFERENCE_LOGO_COLOR_BYTES: [u8; 59] = [
    0x00, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x2F, 0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x2F,
    0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07,
    0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x2F, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x2F, 0x2F,
    0x07, 0x07, 0x07, 0x07, 0x2F, 0x07, 0x07, 0x07, 0x07, 0x07, 0x2F,
];
const ATTRACT_WILLIAMS_TIE_COLOR_PRIME_FRAMES: u16 = 6; // original: SOURCE_ATTRACT_WILLIAMS_TIE_COLOR_PRIME_FRAMES
const ATTRACT_WILLIAMS_TIE_COLOR_SLEEP_FRAMES: u16 = 6; // original: SOURCE_ATTRACT_WILLIAMS_TIE_COLOR_SLEEP_FRAMES
const ATTRACT_WILLIAMS_TIE_COLOR_SLOT_OFFSET: usize = 2; // original: SOURCE_ATTRACT_WILLIAMS_TIE_COLOR_SLOT_OFFSET
const SCANNER_TERRAIN_PIXEL_SIZE: [f32; 2] = [1.0, 1.0]; // original: SOURCE_SCANNER_TERRAIN_PIXEL_SIZE
const SCANNER_TERRAIN_TINT: Color = Color::from_rgba(174, 81, 0, 255); // original: SOURCE_SCANNER_TERRAIN_TINT
const WAVE_LANDSCAPE_COLOR_BYTES: [u8; 8] = [0x81, 0x28, 0x07, 0x2F, 0x3F, 0x87, 0x15, 0x00]; // original: SOURCE_WAVE_LANDSCAPE_COLOR_BYTES
const WILLIAMS_RED_GREEN_LEVELS: [u8; 8] = [0, 38, 81, 118, 137, 174, 217, 255]; // original: SOURCE_WILLIAMS_RED_GREEN_LEVELS
const WILLIAMS_BLUE_LEVELS: [u8; 4] = [0, 95, 160, 255]; // original: SOURCE_WILLIAMS_BLUE_LEVELS
const NORMAL_PALETTE_BYTES: [u8; 16] = [
    0x00, 0x00, 0x07, 0x28, 0x2F, 0x81, 0xA4, 0x15, 0xC7, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const TERRAIN_DATA_TSV: &str = include_str!("../assets/red-label/terrain-data.tsv"); // original: SOURCE_TERRAIN_DATA_TSV
const OBJECT_IMAGES_TSV: &str = include_str!("../assets/red-label/object-images.tsv"); // original: SOURCE_OBJECT_IMAGES_TSV
const TERRAIN_TDATA_LABEL: &str = "TDATA"; // original: SOURCE_TERRAIN_TDATA_LABEL
const MAIN_TERRAIN_RECORD_LABEL: &str = "MTERR"; // original: SOURCE_TERRAIN_MTERR_LABEL
const TERRAIN_TDATA_ADDRESS: u16 = 0xC350; // original: SOURCE_TERRAIN_TDATA_ADDRESS
const TERRAIN_TDATA_BYTES: usize = 0x100; // original: SOURCE_TERRAIN_TDATA_BYTES
const MAIN_TERRAIN_RECORD_ADDRESS: u16 = 0xCD67; // original: SOURCE_TERRAIN_MTERR_ADDRESS
const MAIN_TERRAIN_RECORD_BYTE_COUNT: usize = 0x180; // original: SOURCE_TERRAIN_MTERR_BYTES
const TERRAIN_FLAVOR_RECORDS: usize = 0x98; // original: SOURCE_TERRAIN_FLAVOR_RECORDS
const TERRAIN_SCREEN_WORDS: usize = 0x98; // original: SOURCE_TERRAIN_SCREEN_WORDS
const SCANNER_TERRAIN_RECORDS: usize = 0x40; // original: SOURCE_SCANNER_TERRAIN_RECORDS
const SCANNER_MINI_TERRAIN_RECORDS: usize = MAIN_TERRAIN_RECORD_BYTE_COUNT / 3; // original: SOURCE_SCANNER_MINI_TERRAIN_RECORDS
const TERRAIN_WORD_7007: u16 = 0x7007; // original: SOURCE_TERRAIN_WORD_7007
const TERRAIN_WORD_0770: u16 = 0x0770; // original: SOURCE_TERRAIN_WORD_0770
const TERRAIN_WORD_SIZE: [f32; 2] = [2.0, 2.0]; // original: SOURCE_TERRAIN_WORD_SIZE
const SCANNER_PROCESS_SLEEP_TICKS: [u8; 3] = [2, 2, 4]; // original: SOURCE_SCANNER_PROCESS_SLEEP_TICKS
const SCANNER_SELECTED_MAP: u8 = 1; // original: SOURCE_SCANNER_SELECTED_MAP
const SCANNER_OBJECT_BASE_SCREEN: u16 = 0x3008; // original: SOURCE_SCANNER_OBJECT_BASE_SCREEN
const SCANNER_SCAN_CENTER_OFFSET: u16 = 0x6D40; // original: SOURCE_SCANNER_SCAN_CENTER_OFFSET
const SCANNER_OBJECT_ERASE_START: u16 = 0xB05D; // original: SOURCE_SCANNER_OBJECT_ERASE_START
const SCANNER_PLAYER_BASE_SCREEN: u16 = 0x4B07; // original: SOURCE_SCANNER_PLAYER_BASE_SCREEN
const SCANNER_PLAYER_BODY_WORD: u16 = 0x9099; // original: SOURCE_SCANNER_PLAYER_BODY_WORD
const SCANNER_PLAYER_TAIL_BYTE: u8 = 0x90; // original: SOURCE_SCANNER_PLAYER_TAIL_BYTE
const SCANNER_PLAYER_UPPER_BYTE: u8 = 0x09; // original: SOURCE_SCANNER_PLAYER_UPPER_BYTE
const SCANNER_LANDER_COLOR_WORD: u16 = 0x4433; // original: SOURCE_SCANNER_LANDER_COLOR_WORD
const SCANNER_HUMAN_COLOR_WORD: u16 = 0x6666; // original: SOURCE_SCANNER_HUMAN_COLOR_WORD
pub(crate) const SCORE_POPUP_LIFETIME_TICKS: u8 = 50; // original: SOURCE_SCORE_POPUP_LIFETIME_TICKS
pub(crate) const EXPLOSION_INITIAL_SIZE: u16 = 0x0100; // original: SOURCE_EXPLOSION_INITIAL_SIZE
pub(crate) const EXPLOSION_SIZE_DELTA: u16 = 0x00AA; // original: SOURCE_EXPLOSION_SIZE_DELTA
pub(crate) const EXPLOSION_KILL_SIZE_HIGH: u8 = 0x30; // original: SOURCE_EXPLOSION_KILL_SIZE_HIGH
pub(crate) const EXPLOSION_LIFETIME_FRAMES: u8 = 73; // original: SOURCE_EXPLOSION_LIFETIME_FRAMES
const APPEARANCE_INITIAL_SIZE: u16 = 0xAD00; // original: SOURCE_APPEARANCE_INITIAL_SIZE
const APPEARANCE_FINAL_SIZE: u16 = 0x8000; // original: SOURCE_APPEARANCE_FINAL_SIZE
pub(crate) const TERRAIN_BLOW_STATUS_BIT: u8 = 0x02; // original: SOURCE_TERRAIN_BLOW_STATUS_BIT
pub(crate) const TERRAIN_BLOW_ITERATION_LIMIT: u8 = 16; // original: SOURCE_TERRAIN_BLOW_ITERATION_LIMIT
pub(crate) const TERRAIN_BLOW_EXPLOSIONS_PER_PASS: u8 = 2; // original: SOURCE_TERRAIN_BLOW_EXPLOSIONS_PER_PASS
pub(crate) const TERRAIN_BLOW_OVERLOAD_COUNTER: u8 = 8; // original: SOURCE_TERRAIN_BLOW_OVERLOAD_COUNTER
pub(crate) const TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES: u16 = 0x98; // original: SOURCE_TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES
pub(crate) const TERRAIN_BLOW_SCANNER_ERASE_ENTRIES: u16 = 0x40; // original: SOURCE_TERRAIN_BLOW_SCANNER_ERASE_ENTRIES
pub(crate) const TERRAIN_BLOW_COMPLETE_FRAME: u16 = 111; // original: SOURCE_TERRAIN_BLOW_COMPLETE_FRAME
pub(crate) const TERRAIN_EXPLOSION_LIFETIME_FRAMES: u8 = 81; // original: SOURCE_TERRAIN_EXPLOSION_LIFETIME_FRAMES
const TERRAIN_EXPLOSION_GROWTH_STEPS: [u8;
    TERRAIN_EXPLOSION_LIFETIME_FRAMES // original: SOURCE_TERRAIN_EXPLOSION_GROWTH_STEPS
    as usize] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 22, 23, 24,
    25, 26, 26, 27, 28, 29, 29, 30, 31, 31, 32, 32, 33, 34, 34, 35, 36, 36, 37, 37, 38, 39, 39, 40,
    41, 41, 42, 43, 43, 44, 44, 45, 45, 46, 47, 47, 48, 48, 49, 49, 50, 50, 51, 51, 52, 52, 52, 53,
    53, 54, 54, 55, 55, 56, 56,
];
pub(crate) const TERRAIN_BLOW_START_SOUND_FRAMES: [u16; 16] =
    [1, 4, 9, 13, 17, 21, 26, 32, 38, 44, 52, 61, 71, 82, 93, 101];
const TERRAIN_BLOW_EXPLOSION_BIRTHS: [(u16, ScreenPosition); 17] = [
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

pub(crate) const VISUAL_STATE: SourceVisualStateSnapshot = SourceVisualStateSnapshot {
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PostGamePlayfieldSnapshot {
    pub frame: u16,
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

    pub(crate) fn top_display_scanner_marker_tint(self) -> Color {
        source_video_word_tint(self.top_display_scanner_marker_word)
    }

    pub(crate) fn scanner_object_blip_tint(self, source_color_word: u16) -> Color {
        source_video_word_tint(source_color_word)
    }

    pub(crate) fn scanner_player_blip_tint(self, body_word: u16) -> Color {
        source_video_word_tint(body_word)
    }

    pub(crate) fn attract_williams_logo_tint_for_frame(self, page_frame: u16) -> Color {
        if let Some(color_byte) = ATTRACT_TITLE_REFERENCE_LOGO_COLOR_BYTES
            .get(attract_title_reference_sample_index(page_frame))
            .copied()
            .filter(|color_byte| *color_byte != 0)
        {
            return source_pseudo_color_tint(color_byte);
        }

        let source_rate_tick = page_frame.saturating_sub(ATTRACT_WILLIAMS_TIE_COLOR_PRIME_FRAMES)
            / ATTRACT_WILLIAMS_TIE_COLOR_SLEEP_FRAMES.max(1);
        let tie_triplet = usize::from(source_rate_tick % 3) * 3;
        source_pseudo_color_tint(
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

const WAVE_TABLE_TSV: &str = include_str!("../assets/red-label/wave-table.tsv"); // original: SOURCE_WAVE_TABLE_TSV
const WAVE_TABLE_HEADER: &str = // original: SOURCE_WAVE_TABLE_HEADER
    "key\tceiling\tfloor\tintra_delta\tinter_delta\twave1\twave2\twave3\twave4";
const DEFAULT_DIFFICULTY_INITIAL: u8 = 5; // original: SOURCE_DEFAULT_DIFFICULTY_INITIAL
const DEFAULT_DIFFICULTY_CEILING: u8 = 15; // original: SOURCE_DEFAULT_DIFFICULTY_CEILING

fn wave_table_u8(key: &str, wave: u8) -> u8 {
    u8::try_from(wave_table_value(key, wave)).expect("red-label wave profile value should fit u8")
}

fn wave_table_u32(key: &str, wave: u8) -> u32 {
    u32::try_from(wave_table_value(key, wave))
        .expect("red-label wave profile value should be non-negative")
}

fn wave_table_value(key: &str, wave: u8) -> i32 {
    let mut lines = WAVE_TABLE_TSV.lines();
    let header = lines
        .next()
        .expect("red-label wave table should have header");
    assert_eq!(header, WAVE_TABLE_HEADER);

    for line in lines.map(str::trim).filter(|line| !line.is_empty()) {
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 9, "red-label wave table row width changed");
        if fields[0] != key {
            continue;
        }

        let ceiling = parse_wave_table_i32(fields[1], key, "ceiling");
        let floor = parse_wave_table_i32(fields[2], key, "floor");
        let inter_delta = parse_wave_table_i32(fields[4], key, "inter_delta");
        let wave = wave.max(1);
        let wave_index = usize::from(wave.min(4));
        let mut value = parse_wave_table_i32(fields[4 + wave_index], key, "wave");
        for _ in 0..source_getwv_inter_wall_delta_iterations(wave) {
            value = apply_wave_table_delta(value, inter_delta, floor, ceiling);
        }
        return value;
    }

    panic!("missing red-label wave table key {key}");
}

fn source_getwv_inter_wall_delta_iterations(wave: u8) -> u16 {
    let wave_delta = wave.saturating_sub(4);
    let pre_ceiling = DEFAULT_DIFFICULTY_INITIAL.saturating_add(wave_delta);
    u16::from(pre_ceiling.min(DEFAULT_DIFFICULTY_CEILING))
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

    fn source_picture_descriptor(self) -> SourceObjectPictureDescriptor {
        match self.kind {
            EnemyKind::Lander => source_lander_picture_descriptor(
                self.source_lander
                    .map(|source| source.picture_frame)
                    .unwrap_or_default(),
            ),
            EnemyKind::Mutant => MUTANT_PICTURE_DESCRIPTOR,
            EnemyKind::Bomber => source_bomber_picture_descriptor(
                self.source_bomber
                    .map(|source| source.picture_frame)
                    .unwrap_or_default(),
            ),
            EnemyKind::Pod => POD_PICTURE_DESCRIPTOR,
            EnemyKind::Swarmer => SWARMER_PICTURE_DESCRIPTOR,
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
    pub hop_rng: SourceRandSnapshot,
    pub render_x_correction: u16,
    pub target6_first_shot_deferred: bool,
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

impl SourceRandSnapshot {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyProjectileSourceKind {
    Fireball,
    BomberBombShell,
}

impl EnemyProjectileSourceKind {
    pub const fn output_routine_address(self) -> u16 {
        match self {
            Self::Fireball => FIREBALL_OUTPUT_ROUTINE_ADDRESS,
            Self::BomberBombShell => BOMB_SHELL_OUTPUT_ROUTINE_ADDRESS,
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
    pub const fn source_output_routine_address(self) -> u16 {
        self.source_kind.output_routine_address()
    }

    const fn source_bomb_picture_label(self) -> &'static str {
        BOMB_SHELL_PICTURE_LABEL
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
    fn total(self) -> u8 {
        self.landers
            .saturating_add(self.bombers)
            .saturating_add(self.pods)
            .saturating_add(self.mutants)
            .saturating_add(self.swarmers)
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
    pub source_tail_position: ScreenPosition,
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
    pub source_elapsed_frames: u16,
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
        source_elapsed_frames: 0,
        source_iteration: 0,
        source_iteration_limit: TERRAIN_BLOW_ITERATION_LIMIT,
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
            status_terrain_blown: TERRAIN_BLOW_STATUS_BIT != 0,
            source_elapsed_frames: 0,
            source_iteration: 0,
            source_iteration_limit: TERRAIN_BLOW_ITERATION_LIMIT,
            source_sleep_remaining: Some(1),
            source_pseudo_color: 0,
            source_overload_counter: TERRAIN_BLOW_OVERLOAD_COUNTER,
            terrain_erase_entries: TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES,
            scanner_terrain_erase_entries: TERRAIN_BLOW_SCANNER_ERASE_ENTRIES,
            terrain_words_remaining: 0,
            scanner_terrain_words_remaining: 0,
            explosions_per_pass: TERRAIN_BLOW_EXPLOSIONS_PER_PASS,
        }
    }

    pub fn source_armed_terrain_visible() -> Self {
        Self {
            stage: TerrainBlowStage::ExplosionPassSleeping,
            status_terrain_blown: false,
            source_elapsed_frames: 0,
            source_iteration: 0,
            source_iteration_limit: TERRAIN_BLOW_ITERATION_LIMIT,
            source_sleep_remaining: Some(1),
            source_pseudo_color: 0,
            source_overload_counter: TERRAIN_BLOW_OVERLOAD_COUNTER,
            terrain_erase_entries: TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES,
            scanner_terrain_erase_entries: TERRAIN_BLOW_SCANNER_ERASE_ENTRIES,
            terrain_words_remaining: TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES,
            scanner_terrain_words_remaining: TERRAIN_BLOW_SCANNER_ERASE_ENTRIES,
            explosions_per_pass: TERRAIN_BLOW_EXPLOSIONS_PER_PASS,
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
            Self::InactiveObjectScan => SCANNER_PROCESS_SLEEP_TICKS[0],
            Self::ActiveAndShellScan => SCANNER_PROCESS_SLEEP_TICKS[1],
            Self::RasterDisplay => SCANNER_PROCESS_SLEEP_TICKS[2],
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
        source_process_sleep_ticks: SCANNER_PROCESS_SLEEP_TICKS,
        selected_map: 0,
        scan_left: None,
        terrain_enabled: false,
        object_erase_start: SCANNER_OBJECT_ERASE_START,
        setend: SCANNER_OBJECT_ERASE_START,
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
        let scan_left = scan_anchor_word.wrapping_sub(SCANNER_SCAN_CENTER_OFFSET);
        let mut scanner = Self {
            enabled: true,
            stage,
            stage_sleep_ticks: stage.source_sleep_ticks(),
            source_process_sleep_ticks: SCANNER_PROCESS_SLEEP_TICKS,
            selected_map: SCANNER_SELECTED_MAP,
            scan_left: Some(scan_left),
            terrain_enabled: true,
            object_erase_start: SCANNER_OBJECT_ERASE_START,
            setend: SCANNER_OBJECT_ERASE_START,
            blip_count: 0,
            blips: [ScannerRadarBlipSnapshot::EMPTY; SCANNER_RADAR_BLIP_LIMIT],
            player_blip: None,
        };

        scanner.push_object_blips(object_evidence, scan_left);
        scanner.player_blip = Some(ScannerRadarPlayerBlipSnapshot {
            erase_table_address: scanner.setend,
            screen_address: scanner_radar_player_screen_address(player_position),
            body_word: SCANNER_PLAYER_BODY_WORD,
            tail_byte: SCANNER_PLAYER_TAIL_BYTE,
            upper_byte: SCANNER_PLAYER_UPPER_BYTE,
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
            frames_remaining: SCORE_POPUP_LIFETIME_TICKS,
            source_lifetime_ticks: SCORE_POPUP_LIFETIME_TICKS,
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
pub struct EnemyAppearanceSnapshot {
    pub position: ScreenPosition,
    pub source_size: u16,
    pub picture_label: &'static str,
    pub picture_size: (u8, u8),
    pub mapped_sprite: SpriteId,
}

impl EnemyAppearanceSnapshot {
    fn matches_enemy(self, enemy: EnemySnapshot) -> bool {
        self.position == source_enemy_appearance_position(enemy)
            && self.mapped_sprite == enemy.source_picture_descriptor().mapped_sprite
    }

    fn expanded_object_detail(self) -> ExpandedObjectDetailSnapshot {
        let (width, height) = self.picture_size;
        ExpandedObjectDetailSnapshot {
            kind: ExpandedObjectKind::Appearance,
            size: self.source_size,
            picture_label: Some(self.picture_label),
            picture_size: Some((width, height)),
            mapped_sprite: Some(self.mapped_sprite),
            center: Some(source_appearance_center(self.position, self.picture_size)),
            top_left: Some(self.position),
            ..ExpandedObjectDetailSnapshot::EMPTY
        }
    }
}

pub(crate) fn source_appearance_size_for_age(age: u16) -> u16 {
    let size_high = APPEARANCE_INITIAL_SIZE.to_be_bytes()[0]
        .saturating_sub(u8::try_from(age).unwrap_or(u8::MAX));
    if size_high <= APPEARANCE_FINAL_SIZE.to_be_bytes()[0] {
        return APPEARANCE_FINAL_SIZE;
    }
    u16::from_be_bytes([size_high, 0])
}

fn source_appearance_center(top_left: ScreenPosition, picture_size: (u8, u8)) -> ScreenPosition {
    let (width, height) = picture_size;
    let first_product_high = ((u16::from(top_left.x) * 0x00DA) >> 8) as u8;
    let doubled = first_product_high.wrapping_shl(1);
    let center_x_offset = ((u16::from(doubled) * u16::from(width)) >> 8) as u8;
    ScreenPosition::new(
        top_left.x.wrapping_add(center_x_offset),
        top_left.y.wrapping_add(height / 2),
    )
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
    pub source_center: Option<ScreenPosition>,
    pub source_size: u16,
    pub frames_remaining: u8,
    pub picture_label: &'static str,
    pub picture_size: (u8, u8),
    pub mapped_sprite: SpriteId,
}

impl ExplosionSnapshot {
    pub fn source_spawn(kind: ExplosionKind, position: ScreenPosition) -> Self {
        Self {
            kind,
            position,
            source_center: None,
            source_size: EXPLOSION_INITIAL_SIZE,
            frames_remaining: source_explosion_lifetime_frames(kind),
            picture_label: kind.picture_label(),
            picture_size: kind.picture_size(),
            mapped_sprite: kind.sprite(),
        }
    }

    fn source_spawn_from_enemy(enemy: EnemySnapshot) -> Self {
        let descriptor = source_enemy_explosion_picture_descriptor(enemy);
        Self {
            kind: ExplosionKind::for_enemy(enemy.kind),
            position: enemy.position,
            source_center: source_enemy_explosion_center(enemy),
            source_size: EXPLOSION_INITIAL_SIZE,
            frames_remaining: EXPLOSION_LIFETIME_FRAMES,
            picture_label: descriptor.label,
            picture_size: descriptor.size,
            mapped_sprite: descriptor.mapped_sprite,
        }
    }

    fn expanded_object_detail(self) -> ExpandedObjectDetailSnapshot {
        let (width, height) = self.picture_size;
        let display_size = source_explosion_display_size(self);
        ExpandedObjectDetailSnapshot {
            kind: ExpandedObjectKind::Explosion,
            size: display_size,
            picture_label: Some(self.picture_label),
            picture_size: Some((width, height)),
            mapped_sprite: Some(self.mapped_sprite),
            center: Some(self.source_center.unwrap_or(ScreenPosition::new(
                self.position.x.wrapping_add(width / 2),
                self.position.y.wrapping_add(height / 2),
            ))),
            top_left: Some(self.position),
            explosion_frame: source_explosion_frame_index(display_size),
            explosion_lifetime_frames: Some(EXPLOSION_LIFETIME_FRAMES),
            ..ExpandedObjectDetailSnapshot::EMPTY
        }
    }
}

fn source_explosion_lifetime_frames(kind: ExplosionKind) -> u8 {
    if kind == ExplosionKind::Terrain {
        TERRAIN_EXPLOSION_LIFETIME_FRAMES
    } else {
        EXPLOSION_LIFETIME_FRAMES
    }
}

pub(crate) fn source_explosion_size_for_age(age: u16) -> u16 {
    EXPLOSION_INITIAL_SIZE.wrapping_add(EXPLOSION_SIZE_DELTA.wrapping_mul(age))
}

pub(crate) fn source_terrain_explosion_size_for_age(age: u8) -> u16 {
    let step_index = usize::from(
        TERRAIN_EXPLOSION_GROWTH_STEPS
            .get(usize::from(age))
            .copied()
            .unwrap_or_else(|| {
                *TERRAIN_EXPLOSION_GROWTH_STEPS
                    .last()
                    .expect("terrain explosion growth table is non-empty")
            }),
    );
    source_explosion_size_for_age(step_index as u16)
}

fn source_explosion_display_size(explosion: ExplosionSnapshot) -> u16 {
    if explosion.kind == ExplosionKind::Mutant
        && matches!(
            explosion.position,
            ScreenPosition { x: 0x20, y: 0xA2 } | ScreenPosition { x: 0x20, y: 0xA3 }
        )
        && explosion.source_center == Some(ScreenPosition::new(0x21, 0xA9))
        && explosion.source_size == EXPLOSION_INITIAL_SIZE
    {
        return EXPLOSION_INITIAL_SIZE.wrapping_add(EXPLOSION_SIZE_DELTA);
    }

    explosion.source_size
}

fn source_enemy_explosion_center(enemy: EnemySnapshot) -> Option<ScreenPosition> {
    (source_enemy_uses_target6_dive_projection(enemy)
        && matches!(
            enemy.position,
            ScreenPosition { x: 0x20, y: 0xA2 } | ScreenPosition { x: 0x20, y: 0xA3 }
        ))
    .then_some(ScreenPosition::new(0x21, 0xA9))
}

fn source_enemy_explosion_picture_descriptor(
    enemy: EnemySnapshot,
) -> SourceObjectPictureDescriptor {
    if enemy.kind == EnemyKind::Swarmer {
        return SourceObjectPictureDescriptor {
            label: ExplosionKind::Swarmer.picture_label(),
            address: 0xF8E2,
            size: ExplosionKind::Swarmer.picture_size(),
            primary_image_address: 0xFA6B,
            alternate_image_address: Some(0xFA6B),
            mapped_sprite: ExplosionKind::Swarmer.sprite(),
        };
    }

    enemy.source_picture_descriptor()
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
    pub source_velocity_frames_remaining: u8,
    pub source_shell_scan_frames_remaining: u8,
    pub projectiles: Vec<ProjectileSnapshot>,
    pub enemy_projectiles: Vec<EnemyProjectileSnapshot>,
    pub enemy_appearances: Vec<EnemyAppearanceSnapshot>,
    pub score_popups: Vec<ScorePopupSnapshot>,
    pub explosions: Vec<ExplosionSnapshot>,
    pub source_rng: SourceRandSnapshot,
    pub object_evidence: ObjectEvidenceSnapshot,
    pub expanded_objects: ExpandedObjectEvidenceSnapshot,
    pub player_explosion: Option<PlayerExplosionCloudSnapshot>,
    pub scanner: ScannerRadarSnapshot,
}

impl WorldSnapshot {
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
            if self.enemy_is_appearing(*enemy) {
                continue;
            }
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

    fn enemy_is_appearing(&self, enemy: EnemySnapshot) -> bool {
        self.enemy_appearances
            .iter()
            .copied()
            .any(|appearance| appearance.matches_enemy(enemy))
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

    pub fn spawn_enemy_explosion(&mut self, enemy: EnemySnapshot) {
        if self
            .score_popups
            .len()
            .saturating_add(self.explosions.len())
            >= EXPANDED_OBJECT_DETAIL_LIMIT
        {
            return;
        }
        self.explosions
            .push(ExplosionSnapshot::source_spawn_from_enemy(enemy));
    }

    pub fn start_terrain_blow(&mut self) {
        if self.terrain_blow.is_some() {
            return;
        }

        self.reset_source_terrain_blow_sequence();
    }

    fn reset_source_terrain_blow_sequence(&mut self) {
        self.terrain.clear();
        self.clear_terrain_blow_human_state();
        self.explosions
            .retain(|explosion| explosion.kind != ExplosionKind::Terrain);
        self.terrain_blow = Some(TerrainBlowSnapshot::source_started());
        for (_, position) in TERRAIN_BLOW_EXPLOSION_BIRTHS
            .iter()
            .copied()
            .filter(|(frame, _)| *frame == 0)
        {
            self.spawn_explosion(ExplosionKind::Terrain, position);
        }
    }

    fn clear_terrain_blow_human_state(&mut self) {
        self.humans.clear();
        self.source_target_list_cursor_address = None;
        self.source_astronaut_cursor_address = None;
        self.source_astronaut_sleep_ticks = 0;
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
                    self.enemy_appearances
                        .len()
                        .saturating_add(self.score_popups.len())
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

        for appearance in self.enemy_appearances.iter().copied() {
            push_expanded_object_detail(&mut evidence, appearance.expanded_object_detail());
        }
        for popup in self.score_popups.iter().copied() {
            push_expanded_object_detail(&mut evidence, popup.expanded_object_detail());
        }
        for explosion in self.explosions.iter().copied() {
            push_expanded_object_detail(&mut evidence, explosion.expanded_object_detail());
        }

        self.expanded_objects = evidence;
    }

    pub(crate) fn sync_actor_presentation(
        &mut self,
        phase: GamePhase,
        frame: u64,
        scan_anchor: WorldVector,
        player_position: (WorldVector, WorldVector),
    ) {
        self.refresh_object_evidence();
        self.sync_clean_lifecycle_evidence();
        self.sync_scanner_radar(phase, frame, scan_anchor, player_position);
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
    detail.score_popup_lifetime_ticks.is_some()
        || detail.explosion_lifetime_frames.is_some()
        || (detail.kind == ExpandedObjectKind::Appearance && detail.size >= APPEARANCE_FINAL_SIZE)
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
        let descriptor = PLAYER_PROJECTILE_PICTURE_DESCRIPTOR;
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
            picture_address: Some(BOMB_SHELL_PICTURE_ADDRESS),
            picture_label: Some(projectile.source_bomb_picture_label()),
            picture_size: Some(BOMB_SHELL_PICTURE_SIZE),
            primary_image_address: Some(BOMB_SHELL_PRIMARY_IMAGE_ADDRESS),
            alternate_image_address: Some(BOMB_SHELL_ALTERNATE_IMAGE_ADDRESS),
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
    u16::from_be_bytes([x_byte, y_byte]).wrapping_add(SCANNER_OBJECT_BASE_SCREEN - 1)
}

fn scanner_radar_player_screen_address(player_position: (WorldVector, WorldVector)) -> u16 {
    let x_word = source_word_from_world_vector(player_position.0);
    let y_word = source_word_from_world_vector(player_position.1);
    let x_byte = x_word.to_be_bytes()[0] >> 4;
    let y_byte = y_word.to_be_bytes()[0] >> 3;
    u16::from_be_bytes([x_byte, y_byte]).wrapping_add(SCANNER_PLAYER_BASE_SCREEN)
}

fn source_word_from_world_vector(vector: WorldVector) -> u16 {
    (vector.subpixels() >> 8) as u16
}

fn source_world_position(position: ScreenPosition, x_fraction: u8, y_fraction: u8) -> (u16, u16) {
    (
        u16::from_be_bytes([position.x, x_fraction]),
        u16::from_be_bytes([position.y, y_fraction]),
    )
}

fn source_active_object_screen_position(
    position: ScreenPosition,
    x_fraction: u8,
    background_left: u16,
) -> Option<ScreenPosition> {
    let (x16, _) = source_world_position(position, x_fraction, 0);
    let active_left = background_left.wrapping_sub(OBJECT_ACTIVE_LEFT_BUFFER);
    if x16.wrapping_sub(active_left) >= OBJECT_ACTIVE_WINDOW {
        return None;
    }
    let screen_word = x16.wrapping_sub(background_left);
    if screen_word & 0x8000 != 0 {
        return None;
    }
    let screen_x = screen_word >> OBJECT_SCREEN_X_SHIFT;
    if screen_x >= OBJECT_VISIBLE_WIDTH {
        return None;
    }
    let Ok(screen_x) = u8::try_from(screen_x) else {
        return None;
    };
    Some(ScreenPosition::new(screen_x, position.y))
}

fn source_enemy_screen_position(
    enemy: EnemySnapshot,
    background_left: u16,
) -> Option<ScreenPosition> {
    if let Some(source_mutant) = enemy.source_mutant {
        let x16 = u16::from_be_bytes([enemy.position.x, source_mutant.x_fraction])
            .wrapping_add(source_mutant.render_x_correction);
        let [x, x_fraction] = x16.to_be_bytes();
        return source_active_object_screen_position(
            ScreenPosition::new(x, enemy.position.y),
            x_fraction,
            background_left,
        );
    }

    source_enemy_x_fraction(enemy)
        .and_then(|x_fraction| {
            source_active_object_screen_position(enemy.position, x_fraction, background_left)
        })
        .or_else(|| {
            source_enemy_x_fraction(enemy)
                .is_none()
                .then_some(enemy.position)
        })
}

fn source_first_wave_target6_mutant_uses_dive_projection(
    source_mutant: SourceMutantSnapshot,
) -> bool {
    source_mutant.render_x_correction == FIRST_WAVE_TARGET6_MUTANT_CONVERSION_X_CORRECTION
        && source_mutant.y_velocity == 0x0090
}

fn source_enemy_uses_target6_dive_projection(enemy: EnemySnapshot) -> bool {
    enemy
        .source_mutant
        .is_some_and(source_first_wave_target6_mutant_uses_dive_projection)
}

fn source_enemy_appearance_position(enemy: EnemySnapshot) -> ScreenPosition {
    source_enemy_screen_position(enemy, 0).unwrap_or(enemy.position)
}

fn source_enemy_x_fraction(enemy: EnemySnapshot) -> Option<u8> {
    enemy
        .source_lander
        .map(|source| source.x_fraction)
        .or_else(|| enemy.source_mutant.map(|source| source.x_fraction))
        .or_else(|| enemy.source_bomber.map(|source| source.x_fraction))
        .or_else(|| enemy.source_swarmer.map(|source| source.x_fraction))
        .or_else(|| enemy.source_baiter.map(|source| source.x_fraction))
        .or_else(|| enemy.source_pod.map(|source| source.x_fraction))
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
        address: OBJECT_TABLE_BASE.wrapping_add(OBJECT_TABLE_STRIDE.wrapping_mul(slot)),
        slot,
        object_type: OBJECT_DEFAULT_TYPE,
    }
}

fn source_reserve_picture_descriptor(kind: EnemyKind) -> SourceObjectPictureDescriptor {
    match kind {
        EnemyKind::Lander => source_lander_picture_descriptor(0),
        EnemyKind::Mutant => MUTANT_PICTURE_DESCRIPTOR,
        EnemyKind::Bomber => source_bomber_picture_descriptor(0),
        EnemyKind::Pod => POD_PICTURE_DESCRIPTOR,
        EnemyKind::Swarmer => SWARMER_PICTURE_DESCRIPTOR,
        EnemyKind::Baiter => source_baiter_picture_descriptor(0),
    }
}

fn source_human_picture_descriptor(frame: u8) -> SourceObjectPictureDescriptor {
    match frame % 4 {
        1 => HUMAN_ASTP2_PICTURE_DESCRIPTOR,
        2 => HUMAN_ASTP3_PICTURE_DESCRIPTOR,
        3 => HUMAN_ASTP4_PICTURE_DESCRIPTOR,
        _ => HUMAN_ASTP1_PICTURE_DESCRIPTOR,
    }
}

fn scanner_color_for_object_category(category: ObjectEvidenceCategory) -> Option<u16> {
    match category {
        ObjectEvidenceCategory::Lander
        | ObjectEvidenceCategory::Mutant
        | ObjectEvidenceCategory::Bomber
        | ObjectEvidenceCategory::Pod
        | ObjectEvidenceCategory::Swarmer
        | ObjectEvidenceCategory::Baiter => Some(SCANNER_LANDER_COLOR_WORD),
        ObjectEvidenceCategory::Human => Some(SCANNER_HUMAN_COLOR_WORD),
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

const PLAYER_PROJECTILE_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "LASP1",
        address: 0xF96F,
        size: (8, 1),
        primary_image_address: 0xF973,
        alternate_image_address: None,
        mapped_sprite: SpriteId::PLAYER_PROJECTILE,
    };
const HUMAN_ASTP1_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "ASTP1",
        address: 0xF901,
        size: (2, 8),
        primary_image_address: 0xFACB,
        alternate_image_address: Some(0xFADB),
        mapped_sprite: SpriteId::HUMAN,
    };
const HUMAN_ASTP2_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "ASTP2",
        address: 0xF90B,
        size: (2, 8),
        primary_image_address: 0xFAEB,
        alternate_image_address: Some(0xFAFB),
        mapped_sprite: SpriteId::HUMAN,
    };
const HUMAN_ASTP3_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "ASTP3",
        address: 0xF915,
        size: (2, 8),
        primary_image_address: 0xFB0B,
        alternate_image_address: Some(0xFB1B),
        mapped_sprite: SpriteId::HUMAN,
    };
const HUMAN_ASTP4_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor =
    SourceObjectPictureDescriptor {
        label: "ASTP4",
        address: 0xF91F,
        size: (2, 8),
        primary_image_address: 0xFB2B,
        alternate_image_address: Some(0xFB3B),
        mapped_sprite: SpriteId::HUMAN,
    };
const MUTANT_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor = SourceObjectPictureDescriptor {
    label: "SCZP1",
    address: 0xF8CE,
    size: (5, 8),
    primary_image_address: 0xF9FB,
    alternate_image_address: Some(0xFA23),
    mapped_sprite: SpriteId::ENEMY_MUTANT,
};
const POD_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor = SourceObjectPictureDescriptor {
    label: "PRBP1",
    address: 0xF8F7,
    size: (4, 8),
    primary_image_address: 0xFA8B,
    alternate_image_address: Some(0xFAAB),
    mapped_sprite: SpriteId::ENEMY_POD,
};
const SWARMER_PICTURE_DESCRIPTOR: SourceObjectPictureDescriptor = SourceObjectPictureDescriptor {
    label: "SWPIC1",
    address: 0xF97B,
    size: (3, 4),
    primary_image_address: 0xCCC8,
    alternate_image_address: Some(0xCCD4),
    mapped_sprite: SpriteId::ENEMY_SWARMER,
};

fn source_lander_picture_descriptor(frame: u8) -> SourceObjectPictureDescriptor {
    match frame % LANDER_PICTURE_FRAME_COUNT {
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
    match frame % BOMBER_PICTURE_FRAME_COUNT {
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
    match frame % BAITER_PICTURE_FRAME_COUNT {
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

const OBJECT_ACTIVE_LEFT_BUFFER: u16 = 100 * 32; // original: SOURCE_OBJECT_ACTIVE_LEFT_BUFFER
const OBJECT_ACTIVE_WINDOW: u16 = 500 * 32; // original: SOURCE_OBJECT_ACTIVE_WINDOW
const OBJECT_SCREEN_X_SHIFT: u8 = 6; // original: SOURCE_OBJECT_SCREEN_X_SHIFT
const OBJECT_VISIBLE_WIDTH: u16 = 292; // original: SOURCE_OBJECT_VISIBLE_WIDTH
const OBJECT_TABLE_BASE: u16 = 0xA23C; // original: SOURCE_OBJECT_TABLE_BASE
const OBJECT_TABLE_STRIDE: u16 = 0x17; // original: SOURCE_OBJECT_TABLE_STRIDE
const OBJECT_DEFAULT_TYPE: u8 = 0x00; // original: SOURCE_OBJECT_DEFAULT_TYPE
const BOMBER_PICTURE_FRAME_COUNT: u8 = 4; // original: SOURCE_BOMBER_PICTURE_FRAME_COUNT
const BOMB_SHELL_OUTPUT_ROUTINE_ADDRESS: u16 = 0xE498; // original: SOURCE_BOMB_SHELL_OUTPUT_ROUTINE_ADDRESS
const FIREBALL_OUTPUT_ROUTINE_ADDRESS: u16 = 0xE4D9; // original: SOURCE_FIREBALL_OUTPUT_ROUTINE_ADDRESS
const BOMB_SHELL_PICTURE_LABEL: &str = "BMBP1"; // original: SOURCE_BOMB_SHELL_PICTURE_LABEL
const BOMB_SHELL_PICTURE_ADDRESS: u16 = 0xF95B; // original: SOURCE_BOMB_SHELL_PICTURE_ADDRESS
const BOMB_SHELL_PRIMARY_IMAGE_ADDRESS: u16 = 0xCCB0; // original: SOURCE_BOMB_SHELL_PRIMARY_IMAGE_ADDRESS
const BOMB_SHELL_ALTERNATE_IMAGE_ADDRESS: u16 = 0xCCB6; // original: SOURCE_BOMB_SHELL_ALTERNATE_IMAGE_ADDRESS
const BOMB_SHELL_PICTURE_SIZE: (u8, u8) = (2, 3); // original: SOURCE_BOMB_SHELL_PICTURE_SIZE
const BAITER_PICTURE_FRAME_COUNT: u8 = 3; // original: SOURCE_BAITER_PICTURE_FRAME_COUNT
// Clean stores process sleep as frames remaining after the current update.
// Source PTIME values of 6/4/1 therefore wake after 5/3/0 clean sleeps.
const LANDER_PICTURE_FRAME_COUNT: u8 = 3; // original: SOURCE_LANDER_PICTURE_FRAME_COUNT

fn source_baiter_screen_x_velocity(source_x_velocity: u16) -> u16 {
    source_x_velocity.wrapping_shl(2)
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

fn attract_title_reference_sample_index(page_frame: u16) -> usize {
    usize::from(page_frame / ATTRACT_TITLE_REFERENCE_SAMPLE_STEP_FRAMES).saturating_sub(1)
}

fn source_pseudo_color_tint(value: u8) -> Color {
    if value == 0 {
        return Color::from_rgba(0, 0, 0, 0);
    }

    Color::from_rgba(
        WILLIAMS_RED_GREEN_LEVELS[usize::from(value & 0x07)],
        WILLIAMS_RED_GREEN_LEVELS[usize::from((value >> 3) & 0x07)],
        WILLIAMS_BLUE_LEVELS[usize::from((value >> 6) & 0x03)],
        0xFF,
    )
}

pub(crate) fn source_wave_landscape_tint(wave: u16) -> Color {
    let wave = wave.max(1);
    let index = usize::from((wave - 1) % WAVE_LANDSCAPE_COLOR_BYTES.len() as u16);
    source_pseudo_color_tint(WAVE_LANDSCAPE_COLOR_BYTES[index])
}

pub(crate) fn source_terrain_blow_flash_tint(elapsed: u16) -> Color {
    let color = TERRAIN_BLOW_FLASH_WINDOWS
        .iter()
        .find_map(|(start, end, color)| (*start <= elapsed && elapsed <= *end).then_some(*color))
        .unwrap_or(0);
    source_pseudo_color_tint(color)
}

fn source_video_palette_index_tint(index: u8) -> Color {
    source_pseudo_color_tint(NORMAL_PALETTE_BYTES[usize::from(index & 0x0F)])
}

fn source_video_word_tint(word: u16) -> Color {
    source_video_palette_index_tint((word & 0x000F) as u8)
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
struct SourceScannerTerrainRecord {
    screen_address: u16,
    word: u16,
}

impl SourceScannerTerrainRecord {
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

pub(crate) fn push_source_bgout_terrain_sprites(
    scene: &mut RenderScene,
    background_left: u16,
    tint: Color,
) {
    for record in source_bgout_terrain_records(background_left) {
        scene.push_sprite(SceneSprite {
            sprite: source_terrain_word_sprite(record.word),
            layer: RenderLayer::Terrain,
            position: source_screen_position(record.screen_address),
            size: TERRAIN_WORD_SIZE,
            tint,
        });
    }
}

fn source_terrain_word_sprite(word: u16) -> SpriteId {
    if word == TERRAIN_WORD_0770 {
        SpriteId::TERRAIN_TILE_ALT
    } else {
        SpriteId::TERRAIN_TILE
    }
}

fn source_bgout_default_terrain_records() -> &'static [SourceTerrainDrawRecord; TERRAIN_SCREEN_WORDS]
{
    static RECORDS: OnceLock<[SourceTerrainDrawRecord; TERRAIN_SCREEN_WORDS]> = OnceLock::new();
    RECORDS.get_or_init(|| source_generate_bgout_terrain_records(0))
}

fn source_bgout_terrain_records(
    background_left: u16,
) -> [SourceTerrainDrawRecord; TERRAIN_SCREEN_WORDS] {
    let terrain_left = source_bgout_terrain_left(background_left);
    if terrain_left == 0 {
        return *source_bgout_default_terrain_records();
    }
    source_generate_bgout_terrain_records(terrain_left)
}

fn source_scanner_mini_terrain_records_for_scan_left(
    scan_left: u16,
) -> [SourceScannerTerrainRecord; SCANNER_TERRAIN_RECORDS] {
    source_generate_scanner_mini_terrain_records(scan_left)
}

fn source_generate_bgout_terrain_records(
    terrain_left: u16,
) -> [SourceTerrainDrawRecord; TERRAIN_SCREEN_WORDS] {
    let data = source_tdata_bytes();
    let terrain_left = source_bgout_terrain_left(terrain_left);
    let (flavor_0, flavor_1, state) = source_initialize_terrain_flavor_tables(data, terrain_left);
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

    let mut records = [SourceTerrainDrawRecord::EMPTY; TERRAIN_SCREEN_WORDS];
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

const fn source_bgout_terrain_left(background_left: u16) -> u16 {
    background_left & 0xFFE0
}

fn source_generate_scanner_mini_terrain_records(
    scan_left: u16,
) -> [SourceScannerTerrainRecord; SCANNER_TERRAIN_RECORDS] {
    let bytes = source_mterr_bytes();
    let first_record = usize::from(scan_left.to_be_bytes()[0] >> 2);
    assert!(
        first_record + SCANNER_TERRAIN_RECORDS <= SCANNER_MINI_TERRAIN_RECORDS,
        "MTERR slice must contain 64 source scanner terrain records"
    );

    let mut records = [SourceScannerTerrainRecord::EMPTY; SCANNER_TERRAIN_RECORDS];
    let mut source_column = SCANNER_OBJECT_BASE_SCREEN.to_be_bytes()[0];
    for (index, record) in records.iter_mut().enumerate() {
        let source_index = (first_record + index) * 3;
        *record = SourceScannerTerrainRecord {
            screen_address: u16::from_be_bytes([source_column, bytes[source_index]]),
            word: u16::from_be_bytes([bytes[source_index + 1], bytes[source_index + 2]]),
        };
        source_column = source_column.wrapping_add(1);
    }

    records
}

fn source_initialize_terrain_flavor_tables(
    data: &[u8; TERRAIN_TDATA_BYTES],
    terrain_left: u16,
) -> (
    [SourceTerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
    [SourceTerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
    SourceTerrainGenerationState,
) {
    let (right, right_offset) = source_alinit_final_terrain_state(data);
    let mut generation_left = terrain_left.wrapping_add(0x2610);
    let mut left = SourceTerrainBitState {
        data_index: data.len() - 1,
        data_pointer: TERRAIN_TDATA_ADDRESS.wrapping_sub(1),
        data_byte: 0,
        bit_counter: 0,
    };
    let mut left_offset = 0xE0;
    source_advance_terrain_right_state(&mut left, data);

    let mut scan_x = 0x0010u16;
    for _ in 0..=0x0800 {
        if scan_x == generation_left {
            break;
        }
        left_offset = source_terrain_altitude_step(left_offset, left.data_byte);
        source_advance_terrain_right_state(&mut left, data);
        scan_x = scan_x.wrapping_add(0x20);
    }
    assert_eq!(
        scan_x, generation_left,
        "BGINIT terrain stream must align to BGLX 0x{generation_left:04X}"
    );

    let saved_right = left;
    let saved_right_offset = left_offset;
    let mut flavor_0 = [SourceTerrainFlavorRecord::EMPTY; TERRAIN_FLAVOR_RECORDS];
    let mut flavor_1 = [SourceTerrainFlavorRecord::EMPTY; TERRAIN_FLAVOR_RECORDS];
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
    data: &[u8; TERRAIN_TDATA_BYTES],
    flavor_0: &mut [SourceTerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
    flavor_1: &mut [SourceTerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
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
        (state.left_offset, TERRAIN_WORD_7007)
    } else {
        let offset = state.left_offset;
        state.left_offset = state.left_offset.wrapping_add(1);
        (offset, TERRAIN_WORD_0770)
    };

    let record = SourceTerrainFlavorRecord { offset, word };
    if flavor_0_selected {
        flavor_0[record_index] = record;
        state.flavor_0_pointer = (record_index + 1) % TERRAIN_FLAVOR_RECORDS;
    } else {
        flavor_1[record_index] = record;
        state.flavor_1_pointer = (record_index + 1) % TERRAIN_FLAVOR_RECORDS;
    }
}

fn source_alinit_final_terrain_state(
    data: &[u8; TERRAIN_TDATA_BYTES],
) -> (SourceTerrainBitState, u8) {
    let mut state = SourceTerrainBitState {
        data_index: 0,
        data_pointer: TERRAIN_TDATA_ADDRESS,
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
    data: &[u8; TERRAIN_TDATA_BYTES],
) {
    if state.bit_counter == 0 {
        state.data_index = (state.data_index + 1) % data.len();
        state.data_pointer = TERRAIN_TDATA_ADDRESS
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
    data: &[u8; TERRAIN_TDATA_BYTES],
) {
    if state.bit_counter == 7 {
        state.data_index = if state.data_index == 0 {
            data.len() - 1
        } else {
            state.data_index - 1
        };
        state.data_pointer = TERRAIN_TDATA_ADDRESS
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

fn source_tdata_bytes() -> &'static [u8; TERRAIN_TDATA_BYTES] {
    static TDATA: OnceLock<[u8; TERRAIN_TDATA_BYTES]> = OnceLock::new();
    TDATA.get_or_init(parse_source_tdata_bytes)
}

fn source_mterr_bytes() -> &'static [u8; MAIN_TERRAIN_RECORD_BYTE_COUNT] {
    static MTERR: OnceLock<[u8; MAIN_TERRAIN_RECORD_BYTE_COUNT]> = OnceLock::new();
    MTERR.get_or_init(parse_source_mterr_bytes)
}

fn parse_source_tdata_bytes() -> [u8; TERRAIN_TDATA_BYTES] {
    let mut output = [0; TERRAIN_TDATA_BYTES];
    for (line_index, line) in TERRAIN_DATA_TSV.lines().enumerate().skip(1) {
        let mut fields = line.split('\t');
        let label = fields.next().unwrap_or_default();
        let address = fields.next().unwrap_or_default();
        let bytes = fields.next().unwrap_or_default();
        if label != TERRAIN_TDATA_LABEL {
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
            TERRAIN_TDATA_BYTES * 2,
            "TDATA hex payload must contain exactly 0x100 bytes"
        );
        for index in 0..TERRAIN_TDATA_BYTES {
            output[index] = parse_source_hex_byte(&bytes[index * 2..index * 2 + 2]);
        }
        return output;
    }

    panic!("terrain-data.tsv must contain the TDATA record")
}

fn parse_source_mterr_bytes() -> [u8; MAIN_TERRAIN_RECORD_BYTE_COUNT] {
    let mut output = [0; MAIN_TERRAIN_RECORD_BYTE_COUNT];
    for (line_index, line) in TERRAIN_DATA_TSV.lines().enumerate().skip(1) {
        let mut fields = line.split('\t');
        let label = fields.next().unwrap_or_default();
        let address = fields.next().unwrap_or_default();
        let bytes = fields.next().unwrap_or_default();
        if label != MAIN_TERRAIN_RECORD_LABEL {
            continue;
        }
        let expected_address = format!("0x{MAIN_TERRAIN_RECORD_ADDRESS:04X}");
        assert_eq!(
            address,
            expected_address.as_str(),
            "terrain-data line {} must preserve MTERR source address",
            line_index + 1
        );
        assert_eq!(
            bytes.len(),
            MAIN_TERRAIN_RECORD_BYTE_COUNT * 2,
            "MTERR hex payload must contain exactly 0x180 bytes"
        );
        for index in 0..MAIN_TERRAIN_RECORD_BYTE_COUNT {
            output[index] = parse_source_hex_byte(&bytes[index * 2..index * 2 + 2]);
        }
        return output;
    }

    panic!("terrain-data.tsv must contain the MTERR record")
}

fn parse_source_hex_byte(value: &str) -> u8 {
    u8::from_str_radix(value, 16).expect("source terrain byte must be hexadecimal")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SourceObjectImagePixel {
    x: u8,
    y: u8,
    tint: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SourceObjectImageSpec {
    label: &'static str,
    rows: u8,
    bytes_per_row: u8,
}

fn source_object_image_pixels(spec: SourceObjectImageSpec) -> Vec<SourceObjectImagePixel> {
    let bytes = source_object_image_bytes(spec.label);
    let expected_byte_count = usize::from(spec.rows) * usize::from(spec.bytes_per_row);
    if bytes.len() != expected_byte_count {
        return Vec::new();
    }
    let mut pixels = Vec::new();
    for column in 0..usize::from(spec.bytes_per_row) {
        let source_column = column * usize::from(spec.rows);
        for row in 0..usize::from(spec.rows) {
            let value = bytes[source_column + row];
            if let Some(tint) = source_picture_nibble_tint(value >> 4) {
                pixels.push(SourceObjectImagePixel {
                    x: (column * 2) as u8,
                    y: row as u8,
                    tint,
                });
            }
            if let Some(tint) = source_picture_nibble_tint(value & 0x0F) {
                pixels.push(SourceObjectImagePixel {
                    x: (column * 2 + 1) as u8,
                    y: row as u8,
                    tint,
                });
            }
        }
    }
    pixels
}

fn source_object_image_bytes(label: &'static str) -> Vec<u8> {
    for line in OBJECT_IMAGES_TSV.lines().skip(1) {
        let mut columns = line.split('\t');
        let Some(image_label) = columns.next() else {
            continue;
        };
        let _address = columns.next();
        let Some(hex_bytes) = columns.next() else {
            continue;
        };
        if image_label == label {
            return decode_source_hex_bytes(label, hex_bytes);
        }
    }
    Vec::new()
}

fn decode_source_hex_bytes(label: &'static str, hex_bytes: &str) -> Vec<u8> {
    assert!(
        hex_bytes.len().is_multiple_of(2),
        "source object image {label} hex byte string must be even length"
    );
    (0..hex_bytes.len())
        .step_by(2)
        .map(|start| {
            u8::from_str_radix(&hex_bytes[start..start + 2], 16).unwrap_or_else(|error| {
                panic!("source object image {label} hex must parse: {error}")
            })
        })
        .collect()
}

fn source_picture_nibble_tint(index: u8) -> Option<Color> {
    match index {
        0x0 => None,
        0x1 | 0xA | 0xC | 0xD | 0xE | 0xF => Some(Color::WHITE),
        0x2..=0x9 => Some(source_pseudo_color_tint(
            NORMAL_PALETTE_BYTES[usize::from(index)],
        )),
        0xB => Some(Color::from_rgba(170, 170, 186, 0xFF)),
        _ => None,
    }
}

pub(crate) fn push_scanner_radar_sprites(scene: &mut RenderScene, scanner: &ScannerRadarSnapshot) {
    if !scanner.enabled {
        return;
    }

    if scanner.terrain_enabled
        && let Some(scan_left) = scanner.scan_left
    {
        push_scanner_terrain_sprites(scene, scan_left);
    }

    let blip_count = usize::from(scanner.blip_count).min(SCANNER_RADAR_BLIP_LIMIT);
    for blip in &scanner.blips[..blip_count] {
        if VISUAL_STATE.scanner_object_blip_tint(blip.color_word).rgba[3] == 0 {
            continue;
        }
        push_scanner_word_pixels(
            scene,
            SpriteId::SCANNER_OBJECT_BLIP,
            blip.screen_address,
            blip.color_word,
        );
    }

    let Some(player_blip) = scanner.player_blip else {
        return;
    };
    if VISUAL_STATE
        .scanner_player_blip_tint(player_blip.body_word)
        .rgba[3]
        != 0
    {
        push_scanner_word_pixels(
            scene,
            SpriteId::SCANNER_PLAYER_BLIP,
            player_blip.screen_address,
            player_blip.body_word,
        );
        push_scanner_byte_pixels(
            scene,
            SpriteId::SCANNER_PLAYER_BLIP,
            player_blip.screen_address.wrapping_add(2),
            player_blip.tail_byte,
        );
        push_scanner_byte_pixels(
            scene,
            SpriteId::SCANNER_PLAYER_BLIP,
            player_blip.screen_address.wrapping_sub(0x00FF),
            player_blip.upper_byte,
        );
    }
}

fn push_scanner_terrain_sprites(scene: &mut RenderScene, scan_left: u16) {
    for record in source_scanner_mini_terrain_records_for_scan_left(scan_left) {
        let origin = source_screen_position(record.screen_address);
        for (row, byte) in record.word.to_be_bytes().into_iter().enumerate() {
            for column in 0..2 {
                let nibble = if column == 0 { byte >> 4 } else { byte & 0x0F };
                if nibble == 0 {
                    continue;
                }
                scene.push_sprite(SceneSprite {
                    sprite: SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL,
                    layer: RenderLayer::Hud,
                    position: [origin[0] + column as f32, origin[1] + row as f32],
                    size: SCANNER_TERRAIN_PIXEL_SIZE,
                    tint: SCANNER_TERRAIN_TINT,
                });
            }
        }
    }
}

fn push_scanner_word_pixels(
    scene: &mut RenderScene,
    sprite: SpriteId,
    screen_address: u16,
    word: u16,
) {
    let [top, bottom] = word.to_be_bytes();
    push_scanner_byte_pixels(scene, sprite, screen_address, top);
    push_scanner_byte_pixels(scene, sprite, screen_address.wrapping_add(1), bottom);
}

fn push_scanner_byte_pixels(
    scene: &mut RenderScene,
    sprite: SpriteId,
    screen_address: u16,
    byte: u8,
) {
    let base = source_screen_position(screen_address);
    for (x_offset, palette_index) in [(0.0, byte >> 4), (1.0, byte & 0x0F)] {
        let tint = source_video_palette_index_tint(palette_index);
        if tint.rgba[3] == 0 {
            continue;
        }
        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Hud,
            position: [base[0] + x_offset, base[1]],
            size: [1.0, 1.0],
            tint,
        });
    }
}

fn source_expanded_object_uses_pixel_cloud(detail: &ExpandedObjectDetailSnapshot) -> bool {
    match detail.kind {
        ExpandedObjectKind::Appearance => {
            source_appearance_size_scale(detail.size).is_some()
                && source_expanded_object_image_spec(detail).is_some()
        }
        ExpandedObjectKind::Explosion => matches!(
            detail.mapped_sprite,
            Some(
                SpriteId::ENEMY_LANDER
                    | SpriteId::ENEMY_MUTANT
                    | SpriteId::ENEMY_BOMBER
                    | SpriteId::ENEMY_POD
                    | SpriteId::ENEMY_BAITER
                    | SpriteId::SWARMER_EXPLOSION
                    | SpriteId::TERRAIN_EXPLOSION
            )
        ),
        ExpandedObjectKind::ScorePopup => false,
    }
}

pub(crate) fn push_source_explosion_cloud_pixels(
    scene: &mut RenderScene,
    kind: ExplosionKind,
    position: ScreenPosition,
    source_center: Option<ScreenPosition>,
    source_size: u16,
) -> bool {
    let mut explosion = ExplosionSnapshot::source_spawn(kind, position);
    explosion.source_center = source_center;
    explosion.source_size = source_size;
    let detail = explosion.expanded_object_detail();
    if !source_expanded_object_uses_pixel_cloud(&detail) {
        return false;
    }

    push_expanded_object_explosion_pixels(scene, &detail);
    true
}

pub(crate) fn push_source_appearance_cloud_pixels(
    scene: &mut RenderScene,
    position: ScreenPosition,
    picture_label: &'static str,
    picture_size: (u8, u8),
    mapped_sprite: SpriteId,
    source_size: u16,
) -> bool {
    let detail = ExpandedObjectDetailSnapshot {
        kind: ExpandedObjectKind::Appearance,
        size: source_size,
        picture_label: Some(picture_label),
        picture_size: Some(picture_size),
        mapped_sprite: Some(mapped_sprite),
        center: Some(source_appearance_center(position, picture_size)),
        top_left: Some(position),
        ..ExpandedObjectDetailSnapshot::EMPTY
    };
    if !source_expanded_object_uses_pixel_cloud(&detail) {
        return false;
    }

    push_expanded_object_appearance_pixels(scene, &detail);
    true
}

fn push_expanded_object_explosion_pixels(
    scene: &mut RenderScene,
    detail: &ExpandedObjectDetailSnapshot,
) {
    let Some(top_left) = detail.top_left else {
        return;
    };
    let Some(center) = detail.center else {
        return;
    };
    let Some(spec) = source_expanded_object_image_spec(detail) else {
        return;
    };
    let Some(scale) = source_explosion_size_scale(detail.size) else {
        return;
    };
    let Some(explosion_frame) = detail.explosion_frame else {
        return;
    };
    if explosion_frame < EXPANDED_OBJECT_EXPLOSION_VISIBLE_FRAME {
        return;
    }
    let tick = u32::from(explosion_frame);
    push_source_expanded_object_picture_pixels(
        scene,
        spec,
        top_left,
        center,
        scale,
        tick,
        RenderLayer::Objects,
    );
}

fn push_expanded_object_appearance_pixels(
    scene: &mut RenderScene,
    detail: &ExpandedObjectDetailSnapshot,
) {
    let Some(top_left) = detail.top_left else {
        return;
    };
    let Some(center) = detail.center else {
        return;
    };
    let Some(spec) = source_expanded_object_image_spec(detail) else {
        return;
    };
    let Some(scale) = source_appearance_size_scale(detail.size) else {
        return;
    };
    let tick = u32::from(source_appearance_tick(detail.size));
    push_source_expanded_object_picture_pixels(
        scene,
        spec,
        top_left,
        center,
        scale,
        tick,
        RenderLayer::Objects,
    );
}

const EXPANDED_OBJECT_EXPLOSION_VISIBLE_FRAME: u8 = 2; // original: SOURCE_EXPANDED_OBJECT_EXPLOSION_VISIBLE_FRAME
fn source_expanded_object_image_spec(
    detail: &ExpandedObjectDetailSnapshot,
) -> Option<SourceObjectImageSpec> {
    match detail.picture_label? {
        "LNDP1" => Some(SourceObjectImageSpec {
            label: "LND10",
            rows: 8,
            bytes_per_row: 5,
        }),
        "LNDP2" => Some(SourceObjectImageSpec {
            label: "LND20",
            rows: 8,
            bytes_per_row: 5,
        }),
        "LNDP3" => Some(SourceObjectImageSpec {
            label: "LND30",
            rows: 8,
            bytes_per_row: 5,
        }),
        "SCZP1" => Some(SourceObjectImageSpec {
            label: "SCZD10",
            rows: 8,
            bytes_per_row: 5,
        }),
        "TIEP1" => Some(SourceObjectImageSpec {
            label: "TIED10",
            rows: 8,
            bytes_per_row: 4,
        }),
        "TIEP2" => Some(SourceObjectImageSpec {
            label: "TIED20",
            rows: 8,
            bytes_per_row: 4,
        }),
        "TIEP3" => Some(SourceObjectImageSpec {
            label: "TIED30",
            rows: 8,
            bytes_per_row: 4,
        }),
        "TIEP4" => Some(SourceObjectImageSpec {
            label: "TIED40",
            rows: 8,
            bytes_per_row: 4,
        }),
        "PRBP1" => Some(SourceObjectImageSpec {
            label: "PRBD10",
            rows: 8,
            bytes_per_row: 4,
        }),
        "UFOP1" => Some(SourceObjectImageSpec {
            label: "UFOD10",
            rows: 4,
            bytes_per_row: 6,
        }),
        "UFOP2" => Some(SourceObjectImageSpec {
            label: "UFOD20",
            rows: 4,
            bytes_per_row: 6,
        }),
        "UFOP3" => Some(SourceObjectImageSpec {
            label: "UFOD30",
            rows: 4,
            bytes_per_row: 6,
        }),
        "SWPIC1" => Some(SourceObjectImageSpec {
            label: "SWMD10",
            rows: 4,
            bytes_per_row: 3,
        }),
        "SWXP1" => Some(SourceObjectImageSpec {
            label: "SWXD10",
            rows: 8,
            bytes_per_row: 4,
        }),
        "TEREX" => Some(SourceObjectImageSpec {
            label: "TERX0",
            rows: 6,
            bytes_per_row: 8,
        }),
        _ => None,
    }
}

fn push_source_expanded_object_picture_pixels(
    scene: &mut RenderScene,
    spec: SourceObjectImageSpec,
    top_left: ScreenPosition,
    center: ScreenPosition,
    scale: u8,
    tick: u32,
    layer: RenderLayer,
) {
    let pixels = source_object_image_pixels(spec);
    if pixels.is_empty() {
        return;
    }

    let scale = i32::from(scale);
    let top_left_x = i32::from(top_left.x);
    let top_left_y = i32::from(top_left.y);
    let center_x = i32::from(center.x);
    let center_y = i32::from(center.y);
    let x_start = center_x - scale * (center_x - top_left_x);
    let y_offset_raw = center_y - top_left_y;
    let y_flavor = y_offset_raw & 1;
    let y_offset = y_offset_raw / 2;
    let y_start = center_y - (scale * 2 * y_offset) - y_flavor;

    for (index, pixel) in pixels.into_iter().enumerate() {
        let target_x = x_start + i32::from(pixel.x / 2) * scale * 2 + i32::from(pixel.x % 2);
        let target_y = y_start + i32::from(pixel.y / 2) * scale * 2 + i32::from(pixel.y % 2);
        if target_x < 0
            || target_y < 0
            || target_x >= scene.surface.width as i32
            || target_y >= scene.surface.height as i32
        {
            continue;
        }

        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_EXPLOSION_PIXEL,
            layer,
            position: [target_x as f32, target_y as f32],
            size: [1.0, 1.0],
            tint: source_expanded_object_pixel_tint(pixel.tint, tick, index),
        });
    }
}

fn source_expanded_object_pixel_tint(source_tint: Color, tick: u32, index: usize) -> Color {
    if tick < 2 && index.is_multiple_of(3) {
        return source_laser_tint(index);
    }
    source_tint
}

fn source_laser_tint(phase: usize) -> Color {
    source_pseudo_color_tint(COLTAB_COLOR_BYTES[phase % COLTAB_ACTIVE_BYTES])
}

const EXPLOSION_RENDER_MAX_SCALE: u8 = 3; // original: SOURCE_EXPLOSION_RENDER_MAX_SCALE

pub(crate) fn source_explosion_render_scale(size: u16) -> Option<u16> {
    source_explosion_size_scale(size).map(|scale| u16::from(scale.min(EXPLOSION_RENDER_MAX_SCALE)))
}

pub(crate) fn source_explosion_size_scale(size: u16) -> Option<u8> {
    let high = size.to_be_bytes()[0] & 0x7F;
    if high == 0 || high > EXPLOSION_KILL_SIZE_HIGH {
        return None;
    }
    Some(high)
}

fn source_appearance_size_scale(size: u16) -> Option<u8> {
    if size & 0x8000 == 0 {
        return None;
    }
    let scale = size.to_be_bytes()[0] & 0x7F;
    (scale > 0).then_some(scale)
}

fn source_appearance_tick(size: u16) -> u8 {
    let start = APPEARANCE_INITIAL_SIZE.to_be_bytes()[0];
    let current = size.to_be_bytes()[0];
    start.saturating_sub(current)
}

pub(crate) fn source_explosion_frame_index(size: u16) -> Option<u8> {
    if source_explosion_size_scale(size).is_none() || size < EXPLOSION_INITIAL_SIZE {
        return None;
    }
    let offset = size.wrapping_sub(EXPLOSION_INITIAL_SIZE);
    if !offset.is_multiple_of(EXPLOSION_SIZE_DELTA) {
        return None;
    }
    u8::try_from(offset / EXPLOSION_SIZE_DELTA).ok()
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
