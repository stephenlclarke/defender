//! Source-cited Williams Defender main-board memory helpers.
//!
//! The MAME Williams driver documents Defender's main CPU map as fixed ROM at
//! `0xd000..=0xffff`, a selectable `0xc000..=0xcfff` bank window, and writes to
//! `0xd000` selecting bank `0` for I/O or ROM banks `1`, `2`, `3`, and `7`.
//! It also wires PIA0 port A/B to IN0/IN1, PIA1 port A to IN2, and PIA1 port B
//! output to `williams_state::snd_cmd_w`.
//! Source: <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams.cpp>.
//! Source: <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams_m.cpp>.
//! Source: <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams_v.cpp>.
//! This module exposes RAM bytes, raw palette register writes, ROM reads, the
//! MAME PIA data/control register surface needed for input reads and sound
//! command writes, and address classification. It also models the simple MAME
//! handlers for Defender CMOS 4-bit writes, video-control cocktail state,
//! watchdog reset recognition, video-counter reads, and MAME's older Williams
//! VA11/COUNT240 inputs to PIA1 CB1/CA1. It can also expose native visible
//! palette-index and RGBA frames from video RAM and palette RAM, can apply the
//! ROM-derived CMOS defaults from `romc8.src`, and can route the CMOS-visible
//! `romc0.src` power-up branch. CMOS persistence, `AUDITG` live diagnostic text
//! transfer/full-loop integration, `CROM0` CMOS/color/sound later-test
//! execution, physical advance/lamp timing beyond the modeled ROM-stage
//! screen/LED output, and full video timing remain explicit fidelity gaps.

use crate::{
    input::{
        CabinetInput, DEFENDER_IN2_ADVANCE, DEFENDER_IN2_AUTO_UP_MANUAL_DOWN,
        DEFENDER_IN2_HIGH_SCORE_RESET, DefenderInputPorts,
    },
    pia::{Pia6821, PiaOutputEvent},
    red_label_memory::{
        RedLabelAuditAdjustment, RedLabelCmosDefault, RedLabelCmosLayoutEntry,
        RedLabelRamLayoutEntry, pack_sram_byte, pack_sram_word, unpack_sram_byte, unpack_sram_word,
    },
    red_label_message::{
        RedLabelMessage, RedLabelMessageGlyphImage, RedLabelScoreDigitImage, red_label_message,
        red_label_message_glyph, red_label_score_digit_image,
    },
    rom::{
        RED_LABEL_CROM0_FAILURE_COLOR, RED_LABEL_CROM0_OK_COLOR, RedLabelCrom0AdvanceGate,
        RedLabelCrom0RomStage, RedLabelCrom0RomStageStatus, RedLabelCrom0RomStageTarget,
        RedLabelRomImages,
    },
    sound::SoundCommandLatch,
    video::{
        RenderedImage, defender_visible_palette_index, render_defender_visible_palette_indices,
        render_defender_visible_rgba,
    },
};

pub const MAIN_CPU_RAM_START: u16 = 0x0000;
pub const MAIN_CPU_RAM_END: u16 = 0xBFFF;
pub const MAIN_CPU_RAM_SIZE: usize = (MAIN_CPU_RAM_END - MAIN_CPU_RAM_START + 1) as usize;
pub const MAIN_CPU_BANKED_ROM_START: u16 = 0xC000;
pub const MAIN_CPU_BANKED_ROM_END: u16 = 0xCFFF;
pub const MAIN_CPU_FIXED_ROM_START: u16 = 0xD000;
pub const MAIN_CPU_FIXED_ROM_END: u16 = 0xFFFF;
pub const MAIN_CPU_BANK_SELECT_WRITE: u16 = 0xD000;
pub const MAIN_CPU_IO_BANK: u8 = 0;
pub const MAIN_CPU_ROM_BANKS: [u8; 4] = [1, 2, 3, 7];
pub const PALETTE_RAM_SIZE: usize = 16;
pub const CMOS_RAM_SIZE: usize = 0x100;
pub const RED_LABEL_CRHSTD_CELL_OFFSET: u8 = 0x1D;
pub const RED_LABEL_DIPFLG_CELL_OFFSET: u8 = 0x00;
pub const RED_LABEL_DIPSW_CELL_OFFSET: u8 = 0x7D;
pub const RED_LABEL_CMOSCK_CELL_OFFSET: u8 = 0x7F;
pub const RED_LABEL_REPLAY_CELL_OFFSET: u8 = 0x81;
pub const RED_LABEL_COINSL_CELL_OFFSET: u8 = 0x87;
pub const RED_LABEL_AUDIT_ADJUSTMENT_COUNT: u8 = 28;
pub const RED_LABEL_AUDIT_DISPLAY_VISIBLE_CHARS: usize = 31;
pub const RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS: u8 = 100;
pub const RED_LABEL_AUDIT_REPEAT_SCAN_DELAY_TICKS: u8 = 6;
pub const RED_LABEL_AUDIT_DEBOUNCE_SHIFT_REGISTER: u8 = 0xFF;
pub const RED_LABEL_AUDIT_DEBOUNCE_INPUT_MASK: u8 =
    DEFENDER_IN2_ADVANCE | DEFENDER_IN2_HIGH_SCORE_RESET;
pub const RED_LABEL_HIGH_SCORE_DEFAULT_BYTES: usize = 48;
pub const RED_LABEL_HIGH_SCORE_CELLS: usize = RED_LABEL_HIGH_SCORE_DEFAULT_BYTES * 2;
pub const RED_LABEL_THSTAB_START: u16 = 0xB260;
pub const RED_LABEL_CLRAUD_PACKED_BYTE_WRITES: usize = 0x0E;
pub const RED_LABEL_CLRALL_PACKED_BYTE_WRITES: usize = CMOS_RAM_SIZE / 2;
pub const RED_LABEL_RESET_PALETTE_BYTES: [u8; PALETTE_RAM_SIZE] = [
    0xC0, 0x87, 0x5F, 0x43, 0x2F, 0x21, 0x17, 0x10, 0x0B, 0x07, 0x04, 0x02, 0x01, 0x00, 0x00, 0x00,
];
pub const RED_LABEL_DIAGNOSTIC_LED_FLASH_REPETITIONS: u8 = 2;
pub const RED_LABEL_DIAGNOSTIC_LED_FLASH_DELAY_MS: u16 = 500;
pub const RED_LABEL_DIAGNOSTIC_LETTER_COLOR_ADDRESS: u16 = 0xC001;
pub const RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX: u8 = 1;
pub const RED_LABEL_CROM0_ROM_FAILURE_TEXT: &str = "ROM FAILURE";
pub const RED_LABEL_CROM0_ALL_ROMS_OK_TEXT: &str = "ALL ROMS OK";
pub const RED_LABEL_CROM0_BAD_ROM_LABEL_TEXT: &str = "ROM";
pub const RED_LABEL_CROM0_OPERATOR_PROMPT_TEXT: &str = "PRESS ADVANCE WITH SWITCH SET FOR:";
pub const RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_TEXT: &str = "AUTO FOR RAM TEST";
pub const RED_LABEL_CROM0_RAM_TEST_TEXT: &str = "RAM TEST";
pub const RED_LABEL_CROM0_RAM_FAILURE_TEXT: &str = "RAM FAILURE";
pub const RED_LABEL_CROM0_BAD_RAM_LABEL_TEXT: &str = "RAM";
pub const RED_LABEL_CROM0_NO_RAM_ERRORS_TEXT: &str = "NO RAM ERRORS DETECTED";
pub const RED_LABEL_CROM0_AUTO_TO_EXIT_TEST_TEXT: &str = "AUTO TO EXIT TEST";
pub const RED_LABEL_CROM0_AUTO_FOR_CMOS_RAM_TEST_TEXT: &str = "AUTO FOR CMOS RAM TEST";
pub const RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_INSTRUCTIONS: &[&str] =
    &[RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_TEXT];
pub const RED_LABEL_CROM0_RAM_TEST_START_INSTRUCTIONS: &[&str] =
    &[RED_LABEL_CROM0_AUTO_TO_EXIT_TEST_TEXT];
pub const RED_LABEL_CROM0_RAM_TEST_DONE_INSTRUCTIONS: &[&str] =
    &[RED_LABEL_CROM0_AUTO_FOR_CMOS_RAM_TEST_TEXT];
pub const RED_LABEL_CROM0_OPERATOR_PROMPT_ADDRESS: u16 = 0x18CE;
pub const RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES: [u16; 2] = [0x10DA, 0x10E4];
pub const RED_LABEL_CROM0_RAM_TEST_TEXT_ADDRESS: u16 = 0x4080;
pub const RED_LABEL_CROM0_RAM_FAILURE_TEXT_ADDRESS: u16 = 0x3870;
pub const RED_LABEL_CROM0_BAD_RAM_TEXT_ADDRESS: u16 = 0x4290;
pub const RED_LABEL_CROM0_NO_RAM_ERRORS_TEXT_ADDRESS: u16 = 0x2880;
pub const RED_LABEL_CROM0_RAM_TEST_COLOR: u8 = 0xA5;
pub const RED_LABEL_CROM0_RAM_TEST_LED: u8 = 0x04;
pub const RED_LABEL_CROM0_RAM_TEST_DELAY_MS: u16 = 5000;
pub const RED_LABEL_CROM0_RAM_TEST_ACTIVE_LOOP_DELAY_MS: u16 = 10;
pub const RED_LABEL_CROM0_RAM_TEST_START_SEED: u16 = 0x0000;
pub const RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER: u16 = 0x000A;
pub const RED_LABEL_CROM0_RAM_TEST_END_ADDRESS: u16 = MAIN_CPU_RAM_SIZE as u16;
pub const RED_LABEL_CROM0_RAM_TEST_WORDS: usize = MAIN_CPU_RAM_SIZE / 2;
pub const RED_LABEL_SCREEN_CLEAR_END: u16 = 0x9C00;
pub const WATCHDOG_RESET_BYTE: u8 = 0x39;
pub const VIDEO_COUNTER_CLAMPED_VALUE: u8 = 0xFC;
pub const VIDEO_COUNTER_CLAMP_VPOS: u16 = 0x100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenderIoWindow {
    PaletteRam { index: u8 },
    VideoControl { register: u8 },
    WatchdogReset,
    Cmos { offset: u8 },
    VideoCounter { offset: u16 },
    Pia1 { register: u8 },
    Pia2 { register: u8 },
    Unused { offset: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainCpuReadTarget {
    MainRam { offset: u16 },
    BankedIo(DefenderIoWindow),
    BankedRom { bank: u8, offset: u16 },
    EmptyBank { bank: u8, offset: u16 },
    FixedRom { offset: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainCpuWriteTarget {
    MainRam { offset: u16 },
    BankedIo(DefenderIoWindow),
    BankedRom { bank: u8, offset: u16 },
    EmptyBank { bank: u8, offset: u16 },
    BankSelect { offset: u16 },
    FixedRom { offset: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainCpuRomRead {
    Fixed(u8),
    Banked { bank: u8, byte: u8 },
}

pub type MainCpuRam = Box<[u8; MAIN_CPU_RAM_SIZE]>;
pub type PaletteRam = [u8; PALETTE_RAM_SIZE];
pub type CmosRam = [u8; CMOS_RAM_SIZE];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainCpuReadError {
    Hardware(DefenderIoWindow),
    EmptyBank { bank: u8, offset: u16 },
    UnmappedRom { address: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainCpuWriteError {
    Hardware(DefenderIoWindow),
    ReadOnlyBankedRom { bank: u8, offset: u16 },
    EmptyBank { bank: u8, offset: u16 },
    ReadOnlyFixedRom { offset: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelPowerUpAction {
    NoSpecialFunction,
    InitializeCmosAndAudit,
    AutoCycleRomTest,
    ResetHighScoreTables,
    ClearAudits,
    UnknownSpecialFunction(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelPowerUpDispatchTarget {
    ReturnToCaller,
    AuditGate,
    ComprehensiveRomTest,
    ResetHighScoreTables,
    ClearAudits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelDiagnosticLedOutput {
    pub source_value: u8,
    pub pia1_port_a: u8,
    pub pia1_port_b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelDiagnosticLedFlash {
    pub source_value: u8,
    pub repetitions: u8,
    pub delay_ms: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelDiagnosticPaletteWrite {
    pub address: u16,
    pub index: u8,
    pub value: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelDiagnosticTextWrite {
    pub address: u16,
    pub vector_label: &'static str,
    pub text: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelDiagnosticInstructionWrite {
    pub table_label: &'static str,
    pub lines: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelCrom0BadRomScreenWrite {
    pub row_address: u16,
    pub text_vector_label: &'static str,
    pub text: &'static str,
    pub rom_number: u8,
    pub rom_number_bcd: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedLabelCrom0DiagnosticScreen {
    pub letter_color: Option<RedLabelDiagnosticPaletteWrite>,
    pub headline: Option<RedLabelDiagnosticTextWrite>,
    pub instructions: Vec<RedLabelDiagnosticInstructionWrite>,
    pub bad_roms: Vec<RedLabelCrom0BadRomScreenWrite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelDiagnosticBitmapTextWrite {
    pub address: u16,
    pub vector_label: &'static str,
    pub text: String,
    pub cursor_after: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCrom0BadRomBitmapTextWrite {
    pub row_address: u16,
    pub text: String,
    pub rom_number: u8,
    pub rom_number_bcd: u8,
    pub cursor_after: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelDiagnosticInstructionBitmapTextWrite {
    pub table_label: &'static str,
    pub prompt: RedLabelDiagnosticBitmapTextWrite,
    pub lines: Vec<RedLabelDiagnosticBitmapTextWrite>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedLabelCrom0DiagnosticTextTransfer {
    pub headline: Option<RedLabelDiagnosticBitmapTextWrite>,
    pub instructions: Vec<RedLabelDiagnosticInstructionBitmapTextWrite>,
    pub bad_roms: Vec<RedLabelCrom0BadRomBitmapTextWrite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCrom0RamTestStartTransfer {
    pub screen_clear_end: u16,
    pub palette_zeroed: bool,
    pub led_output: RedLabelDiagnosticLedOutput,
    pub letter_color: RedLabelDiagnosticPaletteWrite,
    pub headline: RedLabelDiagnosticBitmapTextWrite,
    pub instructions: RedLabelDiagnosticInstructionBitmapTextWrite,
    pub delay_ms: u16,
    pub active_loop_delay_ms: u16,
    pub test_counter: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelCrom0RamFailure {
    pub failing_address: u16,
    pub expected_word: u16,
    pub actual_word: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCrom0BadRamBitmapTextWrite {
    pub row_address: u16,
    pub text: String,
    pub block_number: u8,
    pub bit_number: u8,
    pub ram_number_bcd: u8,
    pub cursor_after: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCrom0RamFailureTransfer {
    pub screen_clear_end: u16,
    pub palette_zeroed: bool,
    pub failing_address: u16,
    pub expected_word: u16,
    pub actual_word: u16,
    pub error_mask: u16,
    pub led_output: RedLabelDiagnosticLedOutput,
    pub letter_color: RedLabelDiagnosticPaletteWrite,
    pub headline: RedLabelDiagnosticBitmapTextWrite,
    pub instructions: RedLabelDiagnosticInstructionBitmapTextWrite,
    pub ram_row: RedLabelCrom0BadRamBitmapTextWrite,
    pub block_led_output: RedLabelDiagnosticLedOutput,
    pub bit_led_output: RedLabelDiagnosticLedOutput,
    pub advance_gates: Vec<RedLabelCrom0AdvanceGate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelCrom0RamTestAbortStatus {
    EarlyAbort,
    NoErrorsDetected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelCrom0RamTestTarget {
    CmosRamTest,
    WaitForNextSwitch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCrom0RamTestAbortTransfer {
    pub screen_clear_end: u16,
    pub palette_zeroed: bool,
    pub test_counter: u16,
    pub status: RedLabelCrom0RamTestAbortStatus,
    pub target: RedLabelCrom0RamTestTarget,
    pub letter_color: Option<RedLabelDiagnosticPaletteWrite>,
    pub headline: Option<RedLabelDiagnosticBitmapTextWrite>,
    pub instructions: Option<RedLabelDiagnosticInstructionBitmapTextWrite>,
    pub flash_led: Option<RedLabelDiagnosticLedFlash>,
    pub advance_gates: Vec<RedLabelCrom0AdvanceGate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelCrom0RamTestPatternFill {
    pub seed: u16,
    pub next_seed: u16,
    pub start_address: u16,
    pub end_address: u16,
    pub words_written: usize,
    pub watchdog_reset_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCrom0RamTestPatternVerification {
    pub seed: u16,
    pub next_seed: Option<u16>,
    pub start_address: u16,
    pub end_address: u16,
    pub words_verified: usize,
    pub watchdog_reset_count: usize,
    pub failure: Option<RedLabelCrom0RamFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCrom0RamTestPass {
    pub test_counter: u16,
    pub next_test_counter: Option<u16>,
    pub fill: RedLabelCrom0RamTestPatternFill,
    pub verification: RedLabelCrom0RamTestPatternVerification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelCrom0RamTestLoopStatus {
    Continue,
    Failure,
    OperatorAbort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelCrom0RamTestLoopTarget {
    RamTestActiveLoop,
    RamFailureScreen,
    RamTestAbortScreen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCrom0RamTestLoopStep {
    pub status: RedLabelCrom0RamTestLoopStatus,
    pub target: RedLabelCrom0RamTestLoopTarget,
    pub pass: RedLabelCrom0RamTestPass,
    pub next_seed: Option<u16>,
    pub next_test_counter: Option<u16>,
    pub abort_test_counter: Option<u16>,
    pub failure: Option<RedLabelCrom0RamFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelAuditAdjustmentValue {
    PackedByte(u8),
    PackedWord(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelAuditAdjustmentDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelAuditAdjustmentChange {
    ReadOnly,
    CoinageLocked,
    Changed(RedLabelAuditAdjustmentValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelAuditDisplayLine {
    row_number: u8,
    value: RedLabelAuditAdjustmentValue,
    visible_text: String,
}

impl RedLabelAuditDisplayLine {
    pub fn row_number(&self) -> u8 {
        self.row_number
    }

    pub fn value(&self) -> RedLabelAuditAdjustmentValue {
        self.value
    }

    pub fn visible_text(&self) -> &str {
        &self.visible_text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelAuditDebounceState {
    scan_delay: u8,
    remaining_ticks: u8,
    shift_register: u8,
}

impl Default for RedLabelAuditDebounceState {
    fn default() -> Self {
        Self {
            scan_delay: 0,
            remaining_ticks: 0,
            shift_register: RED_LABEL_AUDIT_DEBOUNCE_SHIFT_REGISTER,
        }
    }
}

impl RedLabelAuditDebounceState {
    pub fn scan_delay(self) -> u8 {
        self.scan_delay
    }

    pub fn remaining_ticks(self) -> u8 {
        self.remaining_ticks
    }

    pub fn shift_register(self) -> u8 {
        self.shift_register
    }

    /// Enter the post-`DISAUD` delay/debounce block at `AUDT3A`.
    ///
    /// `scan_delay` is the source `TEMP1A` value: zero selects the first
    /// 100-tick delay, any other non-six value selects the six-tick scan rate,
    /// and six resumes the already-scanning branch without reinitializing the
    /// shift register.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/romc8.src#L85-L97>.
    pub fn begin_after_display(&mut self) {
        if self.scan_delay == RED_LABEL_AUDIT_REPEAT_SCAN_DELAY_TICKS {
            self.remaining_ticks = RED_LABEL_AUDIT_REPEAT_SCAN_DELAY_TICKS;
            return;
        }

        self.scan_delay = if self.scan_delay == 0 {
            RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS
        } else {
            RED_LABEL_AUDIT_REPEAT_SCAN_DELAY_TICKS
        };
        self.remaining_ticks = self.scan_delay.wrapping_add(1);
        self.shift_register = RED_LABEL_AUDIT_DEBOUNCE_SHIFT_REGISTER;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelAuditDebounceStep {
    Waiting {
        remaining_ticks: u8,
        shift_register: u8,
    },
    Released {
        shift_register: u8,
    },
    TimedOut {
        scan_delay: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RedLabelAuditCycleState {
    operator: RedLabelAuditOperatorState,
    debounce: RedLabelAuditDebounceState,
}

impl RedLabelAuditCycleState {
    pub fn for_displayed_row_number(row_number: u8) -> Option<Self> {
        Some(Self {
            operator: RedLabelAuditOperatorState::for_displayed_row_number(row_number)?,
            debounce: RedLabelAuditDebounceState::default(),
        })
    }

    pub fn operator(self) -> RedLabelAuditOperatorState {
        self.operator
    }

    pub fn debounce(self) -> RedLabelAuditDebounceState {
        self.debounce
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedLabelAuditCycleStep {
    Idle {
        row_number: u8,
        change: Option<RedLabelAuditAdjustmentChange>,
    },
    Display {
        line: RedLabelAuditDisplayLine,
        change: Option<RedLabelAuditAdjustmentChange>,
    },
    Debounce(RedLabelAuditDebounceStep),
    ReturnToGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelAuditOperatorState {
    row_index: u8,
    display_pending: bool,
}

impl Default for RedLabelAuditOperatorState {
    fn default() -> Self {
        Self {
            row_index: 0,
            display_pending: true,
        }
    }
}

impl RedLabelAuditOperatorState {
    pub fn for_displayed_row_number(row_number: u8) -> Option<Self> {
        if !(1..=RED_LABEL_AUDIT_ADJUSTMENT_COUNT).contains(&row_number) {
            return None;
        }

        Some(Self {
            row_index: row_number - 1,
            display_pending: false,
        })
    }

    pub fn row_index(self) -> u8 {
        self.row_index
    }

    pub fn row_number(self) -> u8 {
        self.row_index + 1
    }

    pub fn display_pending(self) -> bool {
        self.display_pending
    }

    pub fn adjustment(
        self,
        adjustments: &[RedLabelAuditAdjustment],
    ) -> Option<&RedLabelAuditAdjustment> {
        let row_number = self.row_number();
        adjustments
            .iter()
            .find(|adjustment| adjustment.number == row_number)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelAuditOperatorStep {
    Idle {
        row_number: u8,
        change: Option<RedLabelAuditAdjustmentChange>,
    },
    Display {
        row_number: u8,
        change: Option<RedLabelAuditAdjustmentChange>,
    },
    ReturnToGame,
}

impl MainCpuRomRead {
    pub fn byte(self) -> u8 {
        match self {
            Self::Fixed(byte) | Self::Banked { byte, .. } => byte,
        }
    }
}

impl RedLabelPowerUpAction {
    /// Source target reached after `PWRUP` handles CMOS/DIP visible effects.
    ///
    /// Source: <https://github.com/mwenge/defender/blob/master/src/romc0.src#L142-L177>.
    pub fn dispatch_target(self) -> RedLabelPowerUpDispatchTarget {
        match self {
            Self::NoSpecialFunction | Self::UnknownSpecialFunction(_) => {
                RedLabelPowerUpDispatchTarget::ReturnToCaller
            }
            Self::InitializeCmosAndAudit => RedLabelPowerUpDispatchTarget::AuditGate,
            Self::AutoCycleRomTest => RedLabelPowerUpDispatchTarget::ComprehensiveRomTest,
            Self::ResetHighScoreTables => RedLabelPowerUpDispatchTarget::ResetHighScoreTables,
            Self::ClearAudits => RedLabelPowerUpDispatchTarget::ClearAudits,
        }
    }
}

impl RedLabelCrom0RamFailure {
    pub fn error_mask(self) -> u16 {
        self.expected_word ^ self.actual_word
    }

    pub fn bad_bit_number(self) -> Result<u8, String> {
        let [high, low] = self.error_mask().to_be_bytes();
        let value = if high == 0 { low } else { high };
        if value == 0 {
            return Err(format!(
                "red-label RAM failure at 0x{:04X} has no differing bits",
                self.failing_address
            ));
        }
        Ok(value.trailing_zeros() as u8 + 1)
    }

    pub fn bad_block_number(self) -> u8 {
        let [page, _] = self.failing_address.to_be_bytes();
        page % 3 + 1
    }

    pub fn ram_number_bcd(self) -> Result<u8, String> {
        Ok((self.bad_block_number() << 4) | self.bad_bit_number()?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainCpuReadWindow {
    FixedRom,
    BankedRom(u8),
    NonRom,
}

#[derive(Debug, Clone, Copy)]
pub struct DefenderMainCpuRomBus<'a> {
    roms: &'a RedLabelRomImages,
    bank_select: u8,
}

impl<'a> DefenderMainCpuRomBus<'a> {
    pub fn new(roms: &'a RedLabelRomImages) -> Self {
        Self {
            roms,
            bank_select: MAIN_CPU_IO_BANK,
        }
    }

    pub fn bank_select(&self) -> u8 {
        self.bank_select
    }

    pub fn write_bank_select(&mut self, bank: u8) {
        self.bank_select = bank & 0x0F;
    }

    pub fn read_window(&self, address: u16) -> MainCpuReadWindow {
        match main_cpu_read_target(address, self.bank_select) {
            MainCpuReadTarget::FixedRom { .. } => MainCpuReadWindow::FixedRom,
            MainCpuReadTarget::BankedRom { bank, .. } => MainCpuReadWindow::BankedRom(bank),
            _ => MainCpuReadWindow::NonRom,
        }
    }

    pub fn read(&self, address: u16) -> Option<MainCpuRomRead> {
        match self.read_window(address) {
            MainCpuReadWindow::FixedRom => self.roms.fixed_byte(address).map(MainCpuRomRead::Fixed),
            MainCpuReadWindow::BankedRom(bank) => self
                .roms
                .banked_byte(bank, address)
                .map(|byte| MainCpuRomRead::Banked { bank, byte }),
            MainCpuReadWindow::NonRom => None,
        }
    }

    pub fn read_byte(&self, address: u16) -> Option<u8> {
        self.read(address).map(MainCpuRomRead::byte)
    }
}

#[derive(Debug)]
pub struct DefenderMainBoard<'a> {
    rom_bus: DefenderMainCpuRomBus<'a>,
    ram: MainCpuRam,
    palette_ram: PaletteRam,
    cmos_ram: CmosRam,
    input_ports: DefenderInputPorts,
    pia1: Pia6821,
    pia2: Pia6821,
    cocktail: bool,
    watchdog_reset_count: u64,
    video_counter_vpos: u16,
    last_sound_command_latch: Option<SoundCommandLatch>,
    diagnostic_led_output: RedLabelDiagnosticLedOutput,
    diagnostic_led_flashes: Vec<RedLabelDiagnosticLedFlash>,
    crom0_diagnostic_screen: RedLabelCrom0DiagnosticScreen,
    crom0_advance_gates: Vec<RedLabelCrom0AdvanceGate>,
}

impl<'a> DefenderMainBoard<'a> {
    pub fn from_ram(roms: &'a RedLabelRomImages, ram: MainCpuRam) -> Self {
        Self {
            rom_bus: DefenderMainCpuRomBus::new(roms),
            ram,
            palette_ram: [0; PALETTE_RAM_SIZE],
            cmos_ram: [0; CMOS_RAM_SIZE],
            input_ports: DefenderInputPorts::EMPTY,
            pia1: Pia6821::default(),
            pia2: Pia6821::default(),
            cocktail: false,
            watchdog_reset_count: 0,
            video_counter_vpos: 0,
            last_sound_command_latch: None,
            diagnostic_led_output: red_label_diagnostic_led_output(0),
            diagnostic_led_flashes: Vec::new(),
            crom0_diagnostic_screen: RedLabelCrom0DiagnosticScreen::default(),
            crom0_advance_gates: Vec::new(),
        }
    }

    /// Deterministic harness constructor. The exact cabinet boot RAM contents
    /// are not modeled until red-label initialization is translated.
    pub fn with_cleared_ram(roms: &'a RedLabelRomImages) -> Self {
        Self::from_ram(roms, cleared_main_cpu_ram())
    }

    pub fn bank_select(&self) -> u8 {
        self.rom_bus.bank_select()
    }

    pub fn ram(&self) -> &[u8] {
        self.ram.as_slice()
    }

    pub fn ram_range(&self, range: std::ops::Range<u16>) -> Option<&[u8]> {
        let start = usize::from(range.start);
        let end = usize::from(range.end);
        if start > end || end > self.ram.len() {
            return None;
        }
        Some(&self.ram[start..end])
    }

    pub fn red_label_ram_field(
        &self,
        field: &RedLabelRamLayoutEntry,
        entry_index: u16,
    ) -> Option<&[u8]> {
        self.ram_range(field.field_range_for_entry(entry_index)?)
    }

    pub fn palette_ram(&self) -> &PaletteRam {
        &self.palette_ram
    }

    pub fn visible_palette_index(&self, visible_x: u16, visible_y: u16) -> Option<u8> {
        defender_visible_palette_index(self.ram.as_slice(), &self.palette_ram, visible_x, visible_y)
    }

    pub fn visible_palette_indices(&self) -> Option<Vec<u8>> {
        render_defender_visible_palette_indices(self.ram.as_slice(), &self.palette_ram)
    }

    pub fn visible_rgba_image(&self) -> Option<RenderedImage> {
        render_defender_visible_rgba(self.ram.as_slice(), &self.palette_ram)
    }

    pub fn diagnostic_led_output(&self) -> RedLabelDiagnosticLedOutput {
        self.diagnostic_led_output
    }

    pub fn diagnostic_led_flashes(&self) -> &[RedLabelDiagnosticLedFlash] {
        &self.diagnostic_led_flashes
    }

    pub fn crom0_diagnostic_screen(&self) -> &RedLabelCrom0DiagnosticScreen {
        &self.crom0_diagnostic_screen
    }

    pub fn crom0_advance_gates(&self) -> &[RedLabelCrom0AdvanceGate] {
        &self.crom0_advance_gates
    }

    pub fn red_label_set_diagnostic_leds(
        &mut self,
        source_value: u8,
    ) -> RedLabelDiagnosticLedOutput {
        let output = red_label_diagnostic_led_output(source_value);
        self.diagnostic_led_output = output;
        output
    }

    pub fn red_label_flash_diagnostic_leds(&mut self, source_value: u8) {
        self.diagnostic_led_flashes
            .push(RedLabelDiagnosticLedFlash {
                source_value,
                repetitions: RED_LABEL_DIAGNOSTIC_LED_FLASH_REPETITIONS,
                delay_ms: RED_LABEL_DIAGNOSTIC_LED_FLASH_DELAY_MS,
            });
        self.red_label_set_diagnostic_leds(0);
    }

    pub fn red_label_apply_crom0_rom_stage(&mut self, stage: &RedLabelCrom0RomStage) {
        let screen = red_label_crom0_diagnostic_screen(stage);
        if let Some(letter_color) = screen.letter_color {
            self.palette_ram[usize::from(letter_color.index)] = letter_color.value;
        }
        self.crom0_diagnostic_screen = screen;
        self.crom0_advance_gates.clone_from(&stage.advance_gates);

        if let Some(led) = stage.initial_led {
            self.red_label_set_diagnostic_leds(led);
        }
        if let Some(led) = stage.flash_led {
            self.red_label_flash_diagnostic_leds(led);
        }
        if let Some(led) = stage.final_led {
            self.red_label_set_diagnostic_leds(led);
        }
    }

    /// Transfer the modeled `CROM0` diagnostic text into video RAM using
    /// message-ROM glyph bytes.
    ///
    /// This covers the `VWTEXT`/`VWNUMB` writes for the ROM-test headline,
    /// operator instruction prompt/lines, and bad-ROM rows. Physical timing
    /// remains represented by `crom0_advance_gates`.
    pub fn red_label_write_crom0_diagnostic_text(
        &mut self,
        stage: &RedLabelCrom0RomStage,
    ) -> Result<RedLabelCrom0DiagnosticTextTransfer, String> {
        let screen = red_label_crom0_diagnostic_screen(stage);
        let mut transfer = RedLabelCrom0DiagnosticTextTransfer::default();

        if let Some(headline) = screen.headline {
            let message = red_label_message(headline.vector_label)?;
            transfer.headline = Some(self.red_label_write_message_text(
                headline.address,
                headline.vector_label,
                message,
            )?);
        }

        let mut instructions = screen.instructions.iter();
        if let Some(instruction) = instructions.next() {
            transfer
                .instructions
                .push(self.red_label_write_crom0_operator_instruction_text(instruction)?);
        }

        for bad_rom in &screen.bad_roms {
            let message = red_label_message(bad_rom.text_vector_label)?;
            let rom_label = self.red_label_write_message_text(
                bad_rom.row_address,
                bad_rom.text_vector_label,
                message,
            )?;
            let cursor_after = self
                .red_label_write_bcd_number_text(rom_label.cursor_after, bad_rom.rom_number_bcd)?;
            transfer.bad_roms.push(RedLabelCrom0BadRomBitmapTextWrite {
                row_address: bad_rom.row_address,
                text: format!("{} {}", bad_rom.text, bad_rom.rom_number),
                rom_number: bad_rom.rom_number,
                rom_number_bcd: bad_rom.rom_number_bcd,
                cursor_after,
            });
        }

        for instruction in instructions {
            transfer
                .instructions
                .push(self.red_label_write_crom0_operator_instruction_text(instruction)?);
        }

        self.crom0_diagnostic_screen = screen;
        Ok(transfer)
    }

    /// Transfer the visible `CRAM0` RAM-test start screen into video RAM.
    ///
    /// This models the source-visible start of the comprehensive RAM test after
    /// the CROM0 stage: `NEWTST`, LEDs off, white diagnostic text, `VRAMTS`,
    /// `IRAMTS`, the 5-second delay intent, and the initial active-loop counter.
    pub fn red_label_write_crom0_ram_test_start(
        &mut self,
    ) -> Result<RedLabelCrom0RamTestStartTransfer, String> {
        self.red_label_clear_screen();
        self.palette_ram = [0; PALETTE_RAM_SIZE];
        self.crom0_diagnostic_screen = RedLabelCrom0DiagnosticScreen::default();
        self.crom0_advance_gates.clear();

        let led_output = self.red_label_set_diagnostic_leds(0);
        let letter_color = RedLabelDiagnosticPaletteWrite {
            address: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_ADDRESS,
            index: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX,
            value: RED_LABEL_CROM0_RAM_TEST_COLOR,
        };
        self.palette_ram[usize::from(letter_color.index)] = letter_color.value;

        let message = red_label_message("VRAMTS")?;
        let headline = self.red_label_write_message_text(
            RED_LABEL_CROM0_RAM_TEST_TEXT_ADDRESS,
            "VRAMTS",
            message,
        )?;
        if headline.text != RED_LABEL_CROM0_RAM_TEST_TEXT {
            return Err(format!(
                "red-label RAM-test vector `VRAMTS` text `{}` does not match source text `{}`",
                headline.text, RED_LABEL_CROM0_RAM_TEST_TEXT
            ));
        }

        let instruction = RedLabelDiagnosticInstructionWrite {
            table_label: "IRAMTS",
            lines: RED_LABEL_CROM0_RAM_TEST_START_INSTRUCTIONS,
        };
        let instructions = self.red_label_write_crom0_operator_instruction_text(&instruction)?;

        Ok(RedLabelCrom0RamTestStartTransfer {
            screen_clear_end: RED_LABEL_SCREEN_CLEAR_END,
            palette_zeroed: true,
            led_output,
            letter_color,
            headline,
            instructions,
            delay_ms: RED_LABEL_CROM0_RAM_TEST_DELAY_MS,
            active_loop_delay_ms: RED_LABEL_CROM0_RAM_TEST_ACTIVE_LOOP_DELAY_MS,
            test_counter: RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER,
        })
    }

    /// Run one source-shaped `RAM2` comprehensive RAM-test fill/verify pass.
    ///
    /// In comprehensive mode the ROM repeats this pass until the operator aborts
    /// through the advance/auto input or a mismatch jumps to `CRAM10`; this
    /// method exposes the pass boundary and the counter value that would feed
    /// `CRAM20` if the caller models an operator abort.
    pub fn red_label_run_crom0_ram_test_pass(
        &mut self,
        seed: u16,
        test_counter: u16,
    ) -> RedLabelCrom0RamTestPass {
        let fill = self.red_label_fill_crom0_ram_test_pattern(seed);
        let verification = self.red_label_verify_crom0_ram_test_pattern(seed);
        let next_test_counter = if verification.failure.is_some() {
            None
        } else {
            Some(test_counter.wrapping_sub(1))
        };

        RedLabelCrom0RamTestPass {
            test_counter,
            next_test_counter,
            fill,
            verification,
        }
    }

    /// Run one pass and route the CROM0 RAM-test loop at the pass boundary.
    ///
    /// The ROM polls the advance/auto switch at watchdog page boundaries inside
    /// `RAM2`; this pass-level helper preserves the same dispatch targets while
    /// leaving sub-pass polling cadence as a physical timing concern.
    pub fn red_label_step_crom0_ram_test_loop(
        &mut self,
        seed: u16,
        test_counter: u16,
        operator_abort: bool,
    ) -> RedLabelCrom0RamTestLoopStep {
        let pass = self.red_label_run_crom0_ram_test_pass(seed, test_counter);
        red_label_crom0_ram_test_loop_step(pass, operator_abort)
    }

    /// Execute the source `RAM3`-`RAM6` pattern write over `0x0000..0xC000`.
    ///
    /// Source: <https://github.com/mwenge/defender/blob/master/src/romf8.src#L99-L133>.
    pub fn red_label_fill_crom0_ram_test_pattern(
        &mut self,
        seed: u16,
    ) -> RedLabelCrom0RamTestPatternFill {
        let mut next_seed = seed;
        let mut words_written = 0;
        let mut watchdog_reset_count = 0;

        for address in (0..MAIN_CPU_RAM_SIZE).step_by(2) {
            next_seed = red_label_crom0_ram_test_next_word(next_seed);
            let [high, low] = next_seed.to_be_bytes();
            self.ram[address] = high;
            self.ram[address + 1] = low;
            words_written += 1;

            if (address + 2) & 0x00FF == 0 {
                watchdog_reset_count += 1;
            }
        }

        RedLabelCrom0RamTestPatternFill {
            seed,
            next_seed,
            start_address: 0,
            end_address: RED_LABEL_CROM0_RAM_TEST_END_ADDRESS,
            words_written,
            watchdog_reset_count,
        }
    }

    /// Execute the source `RAM7`-`RAM17` verification pass.
    ///
    /// On mismatch, `failure` carries the values that the ROM leaves for
    /// `CRAM10`: the expected random word in `Y`, the actual RAM word, and `X`
    /// rewound to the failing word address.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/romf8.src#L136-L206>.
    pub fn red_label_verify_crom0_ram_test_pattern(
        &self,
        seed: u16,
    ) -> RedLabelCrom0RamTestPatternVerification {
        let mut next_seed = seed;
        let mut words_verified = 0;
        let mut watchdog_reset_count = 0;

        for address in (0..MAIN_CPU_RAM_SIZE).step_by(2) {
            next_seed = red_label_crom0_ram_test_next_word(next_seed);
            let actual_word = u16::from_be_bytes([self.ram[address], self.ram[address + 1]]);
            words_verified += 1;
            if actual_word != next_seed {
                let failing_address =
                    u16::try_from(address).expect("RAM-test address is inside main RAM");
                return RedLabelCrom0RamTestPatternVerification {
                    seed,
                    next_seed: None,
                    start_address: 0,
                    end_address: RED_LABEL_CROM0_RAM_TEST_END_ADDRESS,
                    words_verified,
                    watchdog_reset_count,
                    failure: Some(RedLabelCrom0RamFailure {
                        failing_address,
                        expected_word: next_seed,
                        actual_word,
                    }),
                };
            }

            if (address + 2) & 0x00FF == 0 {
                watchdog_reset_count += 1;
            }
        }

        RedLabelCrom0RamTestPatternVerification {
            seed,
            next_seed: Some(next_seed),
            start_address: 0,
            end_address: RED_LABEL_CROM0_RAM_TEST_END_ADDRESS,
            words_verified,
            watchdog_reset_count,
            failure: None,
        }
    }

    /// Transfer the visible `CRAM10` RAM-failure screen into video RAM.
    ///
    /// This derives the source `RAM NN` display from the failing address and
    /// expected/actual word mismatch, then records the possible manual LED and
    /// advance-switch sequence. The physical switch branch to CMOS is left to
    /// the caller.
    pub fn red_label_write_crom0_ram_failure(
        &mut self,
        failure: RedLabelCrom0RamFailure,
    ) -> Result<RedLabelCrom0RamFailureTransfer, String> {
        let error_mask = failure.error_mask();
        let block_number = failure.bad_block_number();
        let bit_number = failure.bad_bit_number()?;
        let ram_number_bcd = (block_number << 4) | bit_number;

        self.red_label_clear_screen();
        self.palette_ram = [0; PALETTE_RAM_SIZE];
        self.crom0_diagnostic_screen = RedLabelCrom0DiagnosticScreen::default();
        self.crom0_advance_gates = vec![
            RedLabelCrom0AdvanceGate::AdvanceSwitchReleaseThenPress,
            RedLabelCrom0AdvanceGate::AdvanceSwitchReleaseThenPress,
            RedLabelCrom0AdvanceGate::NextTestAutoCounter,
        ];

        let led_output = self.red_label_set_diagnostic_leds(RED_LABEL_CROM0_RAM_TEST_LED);
        let letter_color = RedLabelDiagnosticPaletteWrite {
            address: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_ADDRESS,
            index: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX,
            value: RED_LABEL_CROM0_FAILURE_COLOR,
        };
        self.palette_ram[usize::from(letter_color.index)] = letter_color.value;

        let headline = self.red_label_write_message_text(
            RED_LABEL_CROM0_RAM_FAILURE_TEXT_ADDRESS,
            "VRAMFL",
            red_label_message("VRAMFL")?,
        )?;
        if headline.text != RED_LABEL_CROM0_RAM_FAILURE_TEXT {
            return Err(format!(
                "red-label RAM-failure vector `VRAMFL` text `{}` does not match source text `{}`",
                headline.text, RED_LABEL_CROM0_RAM_FAILURE_TEXT
            ));
        }

        let instruction = RedLabelDiagnosticInstructionWrite {
            table_label: "IRAMFL",
            lines: RED_LABEL_CROM0_RAM_TEST_DONE_INSTRUCTIONS,
        };
        let instructions = self.red_label_write_crom0_operator_instruction_text(&instruction)?;

        let ram_label = self.red_label_write_message_text(
            RED_LABEL_CROM0_BAD_RAM_TEXT_ADDRESS,
            "VWRAM",
            red_label_message("VWRAM")?,
        )?;
        let cursor_after =
            self.red_label_write_bcd_number_text(ram_label.cursor_after, ram_number_bcd)?;
        let ram_row = RedLabelCrom0BadRamBitmapTextWrite {
            row_address: RED_LABEL_CROM0_BAD_RAM_TEXT_ADDRESS,
            text: format!(
                "{} {}{}",
                RED_LABEL_CROM0_BAD_RAM_LABEL_TEXT, block_number, bit_number
            ),
            block_number,
            bit_number,
            ram_number_bcd,
            cursor_after,
        };

        Ok(RedLabelCrom0RamFailureTransfer {
            screen_clear_end: RED_LABEL_SCREEN_CLEAR_END,
            palette_zeroed: true,
            failing_address: failure.failing_address,
            expected_word: failure.expected_word,
            actual_word: failure.actual_word,
            error_mask,
            led_output,
            letter_color,
            headline,
            instructions,
            ram_row,
            block_led_output: red_label_diagnostic_led_output(red_label_ram_block_led_source(
                block_number,
            )?),
            bit_led_output: red_label_diagnostic_led_output(bit_number),
            advance_gates: self.crom0_advance_gates.clone(),
        })
    }

    /// Transfer the visible `CRAM20` RAM-test abort/no-error screen.
    ///
    /// A counter equal to the initial source counter is the early-abort path
    /// into the CMOS test. Later abort/completion displays `NO RAM ERRORS
    /// DETECTED`, flashes LED 4 twice, and waits at `NEXTST`.
    pub fn red_label_write_crom0_ram_test_abort(
        &mut self,
        test_counter: u16,
    ) -> Result<RedLabelCrom0RamTestAbortTransfer, String> {
        self.red_label_clear_screen();
        self.palette_ram = [0; PALETTE_RAM_SIZE];
        self.crom0_diagnostic_screen = RedLabelCrom0DiagnosticScreen::default();
        self.crom0_advance_gates.clear();

        if test_counter == RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER {
            return Ok(RedLabelCrom0RamTestAbortTransfer {
                screen_clear_end: RED_LABEL_SCREEN_CLEAR_END,
                palette_zeroed: true,
                test_counter,
                status: RedLabelCrom0RamTestAbortStatus::EarlyAbort,
                target: RedLabelCrom0RamTestTarget::CmosRamTest,
                letter_color: None,
                headline: None,
                instructions: None,
                flash_led: None,
                advance_gates: Vec::new(),
            });
        }

        let letter_color = RedLabelDiagnosticPaletteWrite {
            address: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_ADDRESS,
            index: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX,
            value: RED_LABEL_CROM0_OK_COLOR,
        };
        self.palette_ram[usize::from(letter_color.index)] = letter_color.value;

        let headline = self.red_label_write_message_text(
            RED_LABEL_CROM0_NO_RAM_ERRORS_TEXT_ADDRESS,
            "VNORAM",
            red_label_message("VNORAM")?,
        )?;
        if headline.text != RED_LABEL_CROM0_NO_RAM_ERRORS_TEXT {
            return Err(format!(
                "red-label no-RAM-errors vector `VNORAM` text `{}` does not match source text `{}`",
                headline.text, RED_LABEL_CROM0_NO_RAM_ERRORS_TEXT
            ));
        }

        let instruction = RedLabelDiagnosticInstructionWrite {
            table_label: "IRAMDO",
            lines: RED_LABEL_CROM0_RAM_TEST_DONE_INSTRUCTIONS,
        };
        let instructions = self.red_label_write_crom0_operator_instruction_text(&instruction)?;
        let flash_led = RedLabelDiagnosticLedFlash {
            source_value: RED_LABEL_CROM0_RAM_TEST_LED,
            repetitions: RED_LABEL_DIAGNOSTIC_LED_FLASH_REPETITIONS,
            delay_ms: RED_LABEL_DIAGNOSTIC_LED_FLASH_DELAY_MS,
        };
        self.red_label_flash_diagnostic_leds(RED_LABEL_CROM0_RAM_TEST_LED);
        self.crom0_advance_gates = vec![RedLabelCrom0AdvanceGate::NextTestAutoCounter];

        Ok(RedLabelCrom0RamTestAbortTransfer {
            screen_clear_end: RED_LABEL_SCREEN_CLEAR_END,
            palette_zeroed: true,
            test_counter,
            status: RedLabelCrom0RamTestAbortStatus::NoErrorsDetected,
            target: RedLabelCrom0RamTestTarget::WaitForNextSwitch,
            letter_color: Some(letter_color),
            headline: Some(headline),
            instructions: Some(instructions),
            flash_led: Some(flash_led),
            advance_gates: self.crom0_advance_gates.clone(),
        })
    }

    fn red_label_clear_screen(&mut self) {
        self.ram[..usize::from(RED_LABEL_SCREEN_CLEAR_END)].fill(0);
    }

    fn red_label_write_crom0_operator_instruction_text(
        &mut self,
        instruction: &RedLabelDiagnosticInstructionWrite,
    ) -> Result<RedLabelDiagnosticInstructionBitmapTextWrite, String> {
        let prompt_message = red_label_message("VINS1")?;
        let prompt = self.red_label_write_message_text(
            RED_LABEL_CROM0_OPERATOR_PROMPT_ADDRESS,
            "VINS1",
            prompt_message,
        )?;
        if prompt.text != RED_LABEL_CROM0_OPERATOR_PROMPT_TEXT {
            return Err(format!(
                "red-label operator prompt asset text `{}` does not match source text `{}`",
                prompt.text, RED_LABEL_CROM0_OPERATOR_PROMPT_TEXT
            ));
        }

        let mut lines = Vec::with_capacity(instruction.lines.len());
        for (line_index, line) in instruction.lines.iter().copied().enumerate() {
            let address = *RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES
                .get(line_index)
                .ok_or_else(|| {
                    format!(
                        "red-label operator instruction table `{}` has no screen address for line {}",
                        instruction.table_label,
                        line_index + 1
                    )
                })?;
            let vector_label = red_label_crom0_operator_instruction_vector(line)?;
            let message = red_label_message(vector_label)?;
            let text_write = self.red_label_write_message_text(address, vector_label, message)?;
            if text_write.text != line {
                return Err(format!(
                    "red-label operator instruction vector `{vector_label}` text `{}` does not match source text `{line}`",
                    text_write.text
                ));
            }
            lines.push(text_write);
        }

        Ok(RedLabelDiagnosticInstructionBitmapTextWrite {
            table_label: instruction.table_label,
            prompt,
            lines,
        })
    }

    fn red_label_write_message_text(
        &mut self,
        screen_address: u16,
        vector_label: &'static str,
        message: &RedLabelMessage,
    ) -> Result<RedLabelDiagnosticBitmapTextWrite, String> {
        let mut cursor = screen_address;
        for word in &message.words {
            for character in word.chars() {
                let glyph = red_label_message_glyph(character)?;
                self.red_label_write_message_glyph(cursor, glyph)?;
                cursor = red_label_text_cursor_advance(cursor, glyph.width);
            }
            let space = red_label_message_glyph(' ')?;
            self.red_label_write_message_glyph(cursor, space)?;
            cursor = red_label_text_cursor_advance(cursor, space.width);
        }

        Ok(RedLabelDiagnosticBitmapTextWrite {
            address: screen_address,
            vector_label,
            text: message.words.join(" "),
            cursor_after: cursor,
        })
    }

    fn red_label_write_bcd_number_text(
        &mut self,
        screen_address: u16,
        bcd_number: u8,
    ) -> Result<u16, String> {
        let mut cursor = screen_address;
        for digit in red_label_bcd_number_visible_digits(bcd_number) {
            let image = red_label_score_digit_image(digit)?;
            self.red_label_write_score_digit_image(cursor, image)?;
            cursor = red_label_text_cursor_advance(cursor, image.width);
        }
        Ok(cursor)
    }

    fn red_label_write_message_glyph(
        &mut self,
        screen_address: u16,
        glyph: &RedLabelMessageGlyphImage,
    ) -> Result<(), String> {
        for column in 0..glyph.width {
            let column_address = red_label_screen_offset(screen_address, u16::from(column) << 8)?;
            let source_column = usize::from(column) * usize::from(glyph.height);
            for row in 0..glyph.height {
                let source_byte = glyph.bytes[source_column + usize::from(row)];
                let address = red_label_screen_offset(column_address, u16::from(row))?;
                self.ram[usize::from(address)] = source_byte;
            }
        }
        Ok(())
    }

    fn red_label_write_score_digit_image(
        &mut self,
        screen_address: u16,
        image: &RedLabelScoreDigitImage,
    ) -> Result<(), String> {
        for column in 0..image.width {
            let column_address = red_label_screen_offset(screen_address, u16::from(column) << 8)?;
            let source_column = usize::from(column) * usize::from(image.height);
            for row in 0..image.height {
                let source_byte = image.bytes[source_column + usize::from(row)];
                let address = red_label_screen_offset(column_address, u16::from(row))?;
                self.ram[usize::from(address)] = source_byte;
            }
        }
        Ok(())
    }

    pub fn cmos_ram(&self) -> &CmosRam {
        &self.cmos_ram
    }

    pub fn cmos_range(&self, range: std::ops::Range<u16>) -> Option<&[u8]> {
        let start = usize::from(range.start);
        let end = usize::from(range.end);
        if start > end || end > self.cmos_ram.len() {
            return None;
        }
        Some(&self.cmos_ram[start..end])
    }

    pub fn red_label_cmos_field(&self, field: &RedLabelCmosLayoutEntry) -> Option<&[u8]> {
        self.cmos_range(field.offset_range()?)
    }

    pub fn red_label_audit_adjustment_value(
        &self,
        adjustment: &RedLabelAuditAdjustment,
    ) -> Option<RedLabelAuditAdjustmentValue> {
        let offset = u8::try_from(adjustment.offset).ok()?;
        match adjustment.cells {
            2 => self
                .cmos_sram_read_byte(offset)
                .map(RedLabelAuditAdjustmentValue::PackedByte),
            4 => self
                .cmos_sram_read_word(offset)
                .map(RedLabelAuditAdjustmentValue::PackedWord),
            _ => None,
        }
    }

    /// Source-shaped `DISAUD` stack-buffer text for one audit row.
    ///
    /// This models the 31 visible characters that precede the slash terminator:
    /// row number at columns 0-1, row value at source-selected columns, and the
    /// `MSGAUD` text at column 12. It intentionally leaves `VDISST` bitmap text
    /// transfer and live screen erasure outside this deterministic board helper.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/romc8.src#L117-L173>.
    pub fn red_label_audit_display_line(
        &self,
        adjustment: &RedLabelAuditAdjustment,
    ) -> Option<RedLabelAuditDisplayLine> {
        let value = self.red_label_audit_adjustment_value(adjustment)?;
        let visible_text = red_label_audit_display_text(adjustment, value)?;

        Some(RedLabelAuditDisplayLine {
            row_number: adjustment.number,
            value,
            visible_text,
        })
    }

    /// Run one source `DELY10` tick from the `AUDT3D` post-display debounce.
    ///
    /// This models the `DECA`, `BITB #$0A`, and `RORB` behavior after each
    /// displayed row. A release requires the advance and high-score-reset bits
    /// to shift the register down to zero; timeout leaves `TEMP1A` at the
    /// current scan delay for the next display.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/romc8.src#L97-L112>.
    pub fn red_label_audit_debounce_tick(
        &self,
        state: &mut RedLabelAuditDebounceState,
    ) -> Option<RedLabelAuditDebounceStep> {
        if state.remaining_ticks == 0 {
            return None;
        }

        state.remaining_ticks = state.remaining_ticks.wrapping_sub(1);
        if state.remaining_ticks == 0 {
            return Some(RedLabelAuditDebounceStep::TimedOut {
                scan_delay: state.scan_delay,
            });
        }

        let input = self.input_ports.pia1_port_a();
        let carry_in = input & RED_LABEL_AUDIT_DEBOUNCE_INPUT_MASK != 0;
        state.shift_register = (state.shift_register >> 1) | if carry_in { 0x80 } else { 0 };
        if state.shift_register == 0 {
            state.scan_delay = 0;
            state.remaining_ticks = 0;
            Some(RedLabelAuditDebounceStep::Released { shift_register: 0 })
        } else {
            Some(RedLabelAuditDebounceStep::Waiting {
                remaining_ticks: state.remaining_ticks,
                shift_register: state.shift_register,
            })
        }
    }

    /// Source-shaped deterministic `AUDIT0` cycle.
    ///
    /// This ties the translated row navigation/change decision, `DISAUD` line
    /// formatting, and post-display debounce gate into one step. It keeps live
    /// video text transfer, screen erasure, watchdog refresh, and the outer
    /// post-`PWRUP` entry/exit wiring outside the helper.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/romc8.src#L41-L113>.
    pub fn red_label_audit_cycle_step(
        &mut self,
        state: &mut RedLabelAuditCycleState,
        adjustments: &[RedLabelAuditAdjustment],
    ) -> Option<RedLabelAuditCycleStep> {
        if state.debounce.remaining_ticks != 0 {
            return self
                .red_label_audit_debounce_tick(&mut state.debounce)
                .map(RedLabelAuditCycleStep::Debounce);
        }

        match self.red_label_audit_operator_step(&mut state.operator, adjustments)? {
            RedLabelAuditOperatorStep::Idle { row_number, change } => {
                Some(RedLabelAuditCycleStep::Idle { row_number, change })
            }
            RedLabelAuditOperatorStep::Display { row_number, change } => {
                let adjustment = adjustments
                    .iter()
                    .find(|adjustment| adjustment.number == row_number)?;
                let line = self.red_label_audit_display_line(adjustment)?;
                state.debounce.begin_after_display();
                Some(RedLabelAuditCycleStep::Display { line, change })
            }
            RedLabelAuditOperatorStep::ReturnToGame => Some(RedLabelAuditCycleStep::ReturnToGame),
        }
    }

    /// Source-shaped visible `ALTER` / `HYSCRE` CMOS mutation for one row.
    ///
    /// This models the alterability guards, `DIPSW` flag side effect, and
    /// `BMPNUP` / `BMPNDN` packed-BCD changes. It intentionally leaves the
    /// `AUDITG` screen loop, switch debounce, and display refresh outside this
    /// deterministic board helper.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/romc8.src#L216-L307>.
    pub fn red_label_alter_audit_adjustment(
        &mut self,
        adjustment: &RedLabelAuditAdjustment,
        direction: RedLabelAuditAdjustmentDirection,
    ) -> Option<RedLabelAuditAdjustmentChange> {
        if adjustment.number <= 7 {
            return Some(RedLabelAuditAdjustmentChange::ReadOnly);
        }

        if (11..=16).contains(&adjustment.number)
            && self.cmos_sram_read_byte(RED_LABEL_COINSL_CELL_OFFSET)? != 0
        {
            return Some(RedLabelAuditAdjustmentChange::CoinageLocked);
        }

        let offset = usize::from(u8::try_from(adjustment.offset).ok()?);
        if offset == usize::from(RED_LABEL_DIPSW_CELL_OFFSET) {
            self.cmos_ram[usize::from(RED_LABEL_DIPFLG_CELL_OFFSET)] = cmos_4bit_write_value(1);
        }

        if offset == usize::from(RED_LABEL_REPLAY_CELL_OFFSET) {
            let value = cmos_sram_read_word(&self.cmos_ram, offset)?;
            let adjusted = red_label_adjust_replay_bcd(value, direction);
            cmos_sram_write_word(&mut self.cmos_ram, offset, adjusted)?;
            return Some(RedLabelAuditAdjustmentChange::Changed(
                RedLabelAuditAdjustmentValue::PackedWord(adjusted),
            ));
        }

        if adjustment.cells != 2 {
            return None;
        }

        let value = cmos_sram_read_byte(&self.cmos_ram, offset)?;
        let adjusted = red_label_adjust_bcd_byte(value, direction);
        cmos_sram_write_byte(&mut self.cmos_ram, offset, adjusted)?;
        Some(RedLabelAuditAdjustmentChange::Changed(
            RedLabelAuditAdjustmentValue::PackedByte(adjusted),
        ))
    }

    /// Source-shaped `AUDITG` operator row navigation and change decision.
    ///
    /// This covers the `AUDIT0` service-advance up/down row movement,
    /// auto-advance return after the last row, and the high-score-reset switch
    /// handoff to `ALTER`/`HYSCRE`. It intentionally does not model the
    /// diagnostic text rendering, 10 ms delay cadence, or switch debounce loop.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/romc8.src#L41-L113>.
    pub fn red_label_audit_operator_step(
        &mut self,
        state: &mut RedLabelAuditOperatorState,
        adjustments: &[RedLabelAuditAdjustment],
    ) -> Option<RedLabelAuditOperatorStep> {
        let input = self.input_ports.pia1_port_a();
        if input & DEFENDER_IN2_ADVANCE != 0 {
            state.display_pending = true;
            if input & DEFENDER_IN2_AUTO_UP_MANUAL_DOWN != 0 {
                state.row_index = state.row_index.wrapping_add(1);
                if state.row_index == RED_LABEL_AUDIT_ADJUSTMENT_COUNT {
                    return Some(RedLabelAuditOperatorStep::ReturnToGame);
                }
            } else if state.row_index == 0 {
                state.row_index = RED_LABEL_AUDIT_ADJUSTMENT_COUNT - 1;
            } else {
                state.row_index -= 1;
            }
        }

        let adjustment = state.adjustment(adjustments)?;
        let change = if input & DEFENDER_IN2_HIGH_SCORE_RESET != 0 {
            let direction = if input & DEFENDER_IN2_AUTO_UP_MANUAL_DOWN != 0 {
                RedLabelAuditAdjustmentDirection::Up
            } else {
                RedLabelAuditAdjustmentDirection::Down
            };
            let change = self.red_label_alter_audit_adjustment(adjustment, direction)?;
            if matches!(change, RedLabelAuditAdjustmentChange::Changed(_)) {
                state.display_pending = true;
            }
            Some(change)
        } else {
            None
        };

        let row_number = state.row_number();
        if state.display_pending {
            state.display_pending = false;
            Some(RedLabelAuditOperatorStep::Display { row_number, change })
        } else {
            Some(RedLabelAuditOperatorStep::Idle { row_number, change })
        }
    }

    pub fn cmos_sram_read_byte(&self, nibble_offset: u8) -> Option<u8> {
        cmos_sram_read_byte(&self.cmos_ram, usize::from(nibble_offset))
    }

    pub fn cmos_sram_write_byte(&mut self, nibble_offset: u8, value: u8) -> Option<()> {
        cmos_sram_write_byte(&mut self.cmos_ram, usize::from(nibble_offset), value)
    }

    pub fn cmos_sram_read_word(&self, nibble_offset: u8) -> Option<u16> {
        cmos_sram_read_word(&self.cmos_ram, usize::from(nibble_offset))
    }

    pub fn cmos_sram_write_word(&mut self, nibble_offset: u8, value: u16) -> Option<()> {
        cmos_sram_write_word(&mut self.cmos_ram, usize::from(nibble_offset), value)
    }

    pub fn red_label_clear_cmos_audit_cells(&mut self) -> Option<()> {
        cmos_sram_clear_packed_bytes(&mut self.cmos_ram, 0, RED_LABEL_CLRAUD_PACKED_BYTE_WRITES)
    }

    pub fn red_label_clear_cmos_all_cells(&mut self) {
        let _ = cmos_sram_clear_packed_bytes(
            &mut self.cmos_ram,
            0,
            RED_LABEL_CLRALL_PACKED_BYTE_WRITES,
        );
    }

    pub fn red_label_cmos_init(&mut self, defaults: &[RedLabelCmosDefault]) -> Option<()> {
        self.red_label_clear_cmos_all_cells();
        self.apply_cmos_defaults(defaults)
    }

    pub fn red_label_reset_todays_high_scores(
        &mut self,
        defaults: &[RedLabelCmosDefault],
    ) -> Option<()> {
        let cells = red_label_high_score_default_cells(defaults)?;
        let start = usize::from(RED_LABEL_THSTAB_START);
        let end = start.checked_add(cells.len())?;
        if end > self.ram.len() {
            return None;
        }

        self.ram[start..end].copy_from_slice(&cells);
        Some(())
    }

    pub fn red_label_reset_high_scores(&mut self, defaults: &[RedLabelCmosDefault]) -> Option<()> {
        let cells = red_label_high_score_default_cells(defaults)?;
        let start = usize::from(RED_LABEL_CRHSTD_CELL_OFFSET);
        let end = start.checked_add(cells.len())?;
        if end > self.cmos_ram.len() {
            return None;
        }

        self.cmos_ram[start..end].copy_from_slice(&cells);
        self.red_label_reset_todays_high_scores(defaults)
    }

    pub fn red_label_power_up(
        &mut self,
        defaults: &[RedLabelCmosDefault],
    ) -> Option<RedLabelPowerUpAction> {
        if self.cmos_sram_read_byte(RED_LABEL_CMOSCK_CELL_OFFSET)? != 0x5A {
            self.red_label_cmos_init(defaults)?;
            return Some(RedLabelPowerUpAction::InitializeCmosAndAudit);
        }

        if self.cmos_ram[usize::from(RED_LABEL_DIPFLG_CELL_OFFSET)] & 0x0F == 0 {
            return Some(RedLabelPowerUpAction::NoSpecialFunction);
        }

        self.cmos_ram[usize::from(RED_LABEL_DIPFLG_CELL_OFFSET)] = cmos_4bit_write_value(0);
        let special_function = self.cmos_sram_read_byte(RED_LABEL_DIPSW_CELL_OFFSET)?;
        self.cmos_sram_write_byte(RED_LABEL_DIPSW_CELL_OFFSET, 0)?;

        match special_function {
            0x15 => Some(RedLabelPowerUpAction::AutoCycleRomTest),
            0x25 => {
                self.red_label_reset_high_scores(defaults)?;
                Some(RedLabelPowerUpAction::ResetHighScoreTables)
            }
            0x35 => {
                self.red_label_clear_cmos_audit_cells()?;
                Some(RedLabelPowerUpAction::ClearAudits)
            }
            0x45 => {
                self.red_label_cmos_init(defaults)?;
                Some(RedLabelPowerUpAction::InitializeCmosAndAudit)
            }
            other => Some(RedLabelPowerUpAction::UnknownSpecialFunction(other)),
        }
    }

    /// Source-shaped visible `RESET` setup before the power-up RAM test.
    ///
    /// This covers the `MAPC` bank select clear, PIA register setup, and the
    /// `RESET1` 16-byte palette write loop. It intentionally stops before
    /// condition-code changes and the RAM/ROM diagnostics.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1463-L1489>.
    pub fn red_label_reset_setup(&mut self) -> Result<(), MainCpuWriteError> {
        self.write_byte(MAIN_CPU_BANK_SELECT_WRITE, 0)?;
        self.write_byte(0xCC01, 0)?;
        self.write_byte(0xCC03, 0)?;
        self.write_byte(0xCC05, 0)?;
        self.write_byte(0xCC07, 0)?;
        self.write_byte(0xCC00, 0xC0)?;
        self.write_byte(0xCC02, 0xFF)?;
        self.write_byte(0xCC04, 0)?;
        self.write_byte(0xCC06, 0)?;
        self.write_byte(0xCC03, 0x04)?;
        self.write_byte(0xCC05, 0x04)?;
        self.write_byte(0xCC07, 0x04)?;
        self.write_byte(0xCC01, 0x14)?;

        for (offset, byte) in RED_LABEL_RESET_PALETTE_BYTES.iter().copied().enumerate() {
            self.write_byte(0xC000 + offset as u16, byte)?;
        }

        Ok(())
    }

    pub fn apply_cmos_default(&mut self, default: &RedLabelCmosDefault) -> Option<()> {
        let range = default.cell_range()?;
        let start = usize::from(range.start);
        let end = usize::from(range.end);
        if start > end || end > self.cmos_ram.len() {
            return None;
        }

        let cells = default.encoded_cells();
        if cells.len() != end - start {
            return None;
        }

        self.cmos_ram[start..end].copy_from_slice(&cells);
        Some(())
    }

    pub fn apply_cmos_defaults(&mut self, defaults: &[RedLabelCmosDefault]) -> Option<()> {
        for default in defaults {
            self.apply_cmos_default(default)?;
        }
        Some(())
    }

    pub fn input_ports(&self) -> DefenderInputPorts {
        self.input_ports
    }

    pub fn set_input_ports(&mut self, input_ports: DefenderInputPorts) {
        self.input_ports = input_ports;
    }

    pub fn set_cabinet_input(&mut self, input: CabinetInput) {
        self.set_input_ports(input.defender_input_ports());
    }

    pub fn pia0_port_a_input(&self) -> u8 {
        self.input_ports.pia0_port_a()
    }

    pub fn pia0_port_b_input(&self) -> u8 {
        self.input_ports.pia0_port_b()
    }

    pub fn pia1_port_a_input(&self) -> u8 {
        self.input_ports.pia1_port_a()
    }

    pub fn pia1(&self) -> Pia6821 {
        self.pia1
    }

    pub fn pia2(&self) -> Pia6821 {
        self.pia2
    }

    pub fn cocktail(&self) -> bool {
        self.cocktail
    }

    pub fn watchdog_reset_count(&self) -> u64 {
        self.watchdog_reset_count
    }

    pub fn video_counter_vpos(&self) -> u16 {
        self.video_counter_vpos
    }

    pub fn set_video_counter_vpos(&mut self, vpos: u16) {
        self.video_counter_vpos = vpos;
    }

    pub fn video_counter_read(&self) -> u8 {
        video_counter_read_value(self.video_counter_vpos)
    }

    pub fn last_sound_command_latch(&self) -> Option<SoundCommandLatch> {
        self.last_sound_command_latch
    }

    pub fn write_pia1_port_b_output(&mut self, value: u8) -> SoundCommandLatch {
        let latch = SoundCommandLatch::from_main_board_pia_port_b(value);
        self.last_sound_command_latch = Some(latch);
        latch
    }

    pub fn read_byte(&mut self, address: u16) -> Result<u8, MainCpuReadError> {
        match main_cpu_read_target(address, self.bank_select()) {
            MainCpuReadTarget::MainRam { offset } => Ok(self.ram[usize::from(offset)]),
            MainCpuReadTarget::BankedIo(DefenderIoWindow::Cmos { offset }) => {
                Ok(self.cmos_ram[usize::from(offset)])
            }
            MainCpuReadTarget::BankedIo(DefenderIoWindow::VideoCounter { .. }) => {
                Ok(self.video_counter_read())
            }
            MainCpuReadTarget::BankedIo(DefenderIoWindow::Pia1 { register }) => {
                Ok(self.pia1.read(register, self.input_ports.in2, 0x00))
            }
            MainCpuReadTarget::BankedIo(DefenderIoWindow::Pia2 { register }) => {
                Ok(self
                    .pia2
                    .read(register, self.input_ports.in0, self.input_ports.in1))
            }
            MainCpuReadTarget::BankedIo(window) => Err(MainCpuReadError::Hardware(window)),
            MainCpuReadTarget::BankedRom { .. } | MainCpuReadTarget::FixedRom { .. } => self
                .rom_bus
                .read_byte(address)
                .ok_or(MainCpuReadError::UnmappedRom { address }),
            MainCpuReadTarget::EmptyBank { bank, offset } => {
                Err(MainCpuReadError::EmptyBank { bank, offset })
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) -> Result<(), MainCpuWriteError> {
        match main_cpu_write_target(address, self.bank_select()) {
            MainCpuWriteTarget::MainRam { offset } => {
                self.ram[usize::from(offset)] = value;
                Ok(())
            }
            MainCpuWriteTarget::BankedIo(DefenderIoWindow::PaletteRam { index }) => {
                self.palette_ram[usize::from(index)] = value;
                Ok(())
            }
            MainCpuWriteTarget::BankedIo(DefenderIoWindow::Cmos { offset }) => {
                self.cmos_ram[usize::from(offset)] = cmos_4bit_write_value(value);
                Ok(())
            }
            MainCpuWriteTarget::BankedIo(DefenderIoWindow::VideoControl { .. }) => {
                self.cocktail = video_control_cocktail(value);
                Ok(())
            }
            MainCpuWriteTarget::BankedIo(DefenderIoWindow::WatchdogReset) => {
                if value == WATCHDOG_RESET_BYTE {
                    self.watchdog_reset_count = self.watchdog_reset_count.saturating_add(1);
                }
                Ok(())
            }
            MainCpuWriteTarget::BankedIo(DefenderIoWindow::Pia1 { register }) => {
                let output_event = self.pia1.write(register, value, self.input_ports.in2, 0x00);
                if let Some(PiaOutputEvent::PortB { data, .. }) = output_event {
                    self.write_pia1_port_b_output(data);
                }
                Ok(())
            }
            MainCpuWriteTarget::BankedIo(DefenderIoWindow::Pia2 { register }) => {
                self.pia2
                    .write(register, value, self.input_ports.in0, self.input_ports.in1);
                Ok(())
            }
            MainCpuWriteTarget::BankedIo(window) => Err(MainCpuWriteError::Hardware(window)),
            MainCpuWriteTarget::BankedRom { bank, offset } => {
                Err(MainCpuWriteError::ReadOnlyBankedRom { bank, offset })
            }
            MainCpuWriteTarget::EmptyBank { bank, offset } => {
                Err(MainCpuWriteError::EmptyBank { bank, offset })
            }
            MainCpuWriteTarget::BankSelect { .. } => {
                self.rom_bus.write_bank_select(value);
                Ok(())
            }
            MainCpuWriteTarget::FixedRom { offset } => {
                Err(MainCpuWriteError::ReadOnlyFixedRom { offset })
            }
        }
    }

    pub fn update_video_interrupt_lines(&mut self, scanline: u16) {
        if scanline != 256 {
            self.pia1.cb1_w(scanline & 0x20 != 0);
        }
        self.pia1.ca1_w(scanline >= 240);
    }

    pub fn main_irq_asserted(&self) -> bool {
        self.pia1.irq_a_asserted() || self.pia1.irq_b_asserted()
    }
}

pub fn cleared_main_cpu_ram() -> MainCpuRam {
    Box::new([0; MAIN_CPU_RAM_SIZE])
}

pub fn cmos_4bit_write_value(value: u8) -> u8 {
    value | 0xF0
}

pub fn cmos_sram_read_byte(cmos_ram: &CmosRam, nibble_offset: usize) -> Option<u8> {
    let ms_nibble = cmos_ram.get(nibble_offset)?;
    let ls_nibble = cmos_ram.get(nibble_offset + 1)?;
    Some(pack_sram_byte(*ms_nibble, *ls_nibble))
}

pub fn cmos_sram_write_byte(cmos_ram: &mut CmosRam, nibble_offset: usize, value: u8) -> Option<()> {
    if nibble_offset + 1 >= cmos_ram.len() {
        return None;
    }

    let (ms_nibble, ls_nibble) = unpack_sram_byte(value);
    cmos_ram[nibble_offset] = cmos_4bit_write_value(ms_nibble);
    cmos_ram[nibble_offset + 1] = cmos_4bit_write_value(ls_nibble);
    Some(())
}

pub fn cmos_sram_read_word(cmos_ram: &CmosRam, nibble_offset: usize) -> Option<u16> {
    let nibbles = [
        *cmos_ram.get(nibble_offset)?,
        *cmos_ram.get(nibble_offset + 1)?,
        *cmos_ram.get(nibble_offset + 2)?,
        *cmos_ram.get(nibble_offset + 3)?,
    ];
    Some(pack_sram_word(nibbles))
}

pub fn cmos_sram_write_word(
    cmos_ram: &mut CmosRam,
    nibble_offset: usize,
    value: u16,
) -> Option<()> {
    if nibble_offset + 3 >= cmos_ram.len() {
        return None;
    }

    let nibbles = unpack_sram_word(value);
    for (index, nibble) in nibbles.into_iter().enumerate() {
        cmos_ram[nibble_offset + index] = cmos_4bit_write_value(nibble);
    }
    Some(())
}

pub fn cmos_sram_clear_packed_bytes(
    cmos_ram: &mut CmosRam,
    nibble_offset: usize,
    packed_byte_count: usize,
) -> Option<()> {
    let cell_count = packed_byte_count.checked_mul(2)?;
    let end = nibble_offset.checked_add(cell_count)?;
    if end > cmos_ram.len() {
        return None;
    }

    cmos_ram[nibble_offset..end].fill(cmos_4bit_write_value(0));
    Some(())
}

fn red_label_adjust_bcd_byte(value: u8, direction: RedLabelAuditAdjustmentDirection) -> u8 {
    let addend = match direction {
        RedLabelAuditAdjustmentDirection::Up => 0x01,
        RedLabelAuditAdjustmentDirection::Down => 0x99,
    };
    red_label_bcd_add_byte(value, addend, false).0
}

fn red_label_adjust_replay_bcd(value: u16, direction: RedLabelAuditAdjustmentDirection) -> u16 {
    let ms_byte = (value >> 8) as u8;
    let ls_byte = value as u8;
    let (adjusted_ms_byte, adjusted_ls_byte) = match direction {
        RedLabelAuditAdjustmentDirection::Up => {
            let (adjusted_ls_byte, carry) = red_label_bcd_add_byte(ls_byte, 0x10, false);
            let adjusted_ms_byte = if carry {
                red_label_bcd_add_byte(ms_byte, 0x01, false).0
            } else {
                ms_byte
            };
            (adjusted_ms_byte, adjusted_ls_byte)
        }
        RedLabelAuditAdjustmentDirection::Down => {
            let (adjusted_ls_byte, carry) = red_label_bcd_add_byte(ls_byte, 0x90, false);
            let adjusted_ms_byte = red_label_bcd_add_byte(ms_byte, 0x99, carry).0;
            (adjusted_ms_byte, adjusted_ls_byte)
        }
    };

    u16::from_be_bytes([adjusted_ms_byte, adjusted_ls_byte])
}

fn red_label_audit_display_text(
    adjustment: &RedLabelAuditAdjustment,
    value: RedLabelAuditAdjustmentValue,
) -> Option<String> {
    let mut buffer = [b' '; RED_LABEL_AUDIT_DISPLAY_VISIBLE_CHARS];
    let row_number = red_label_decimal_to_bcd_byte(adjustment.number);
    red_label_write_bcd_byte_ascii(&mut buffer, 0, row_number)?;

    match value {
        RedLabelAuditAdjustmentValue::PackedWord(value) if adjustment.number <= 7 => {
            red_label_write_bcd_word_ascii(&mut buffer, 7, value)?;
        }
        RedLabelAuditAdjustmentValue::PackedWord(value) if adjustment.number == 8 => {
            red_label_write_bcd_word_ascii(&mut buffer, 5, value)?;
            red_label_write_bcd_byte_ascii(&mut buffer, 9, 0)?;
        }
        RedLabelAuditAdjustmentValue::PackedByte(value) if adjustment.number > 8 => {
            red_label_write_bcd_byte_ascii(&mut buffer, 9, value)?;
        }
        _ => return None,
    }

    let message_start = 12;
    let message = adjustment.message.as_bytes();
    let message_end = message_start + message.len();
    if message_end > buffer.len() || !message.is_ascii() {
        return None;
    }
    buffer[message_start..message_end].copy_from_slice(message);

    String::from_utf8(buffer.to_vec()).ok()
}

fn red_label_write_bcd_word_ascii(buffer: &mut [u8], start: usize, value: u16) -> Option<()> {
    let [ms_byte, ls_byte] = value.to_be_bytes();
    red_label_write_bcd_byte_ascii(buffer, start, ms_byte)?;
    red_label_write_bcd_byte_ascii(buffer, start + 2, ls_byte)
}

fn red_label_write_bcd_byte_ascii(buffer: &mut [u8], start: usize, value: u8) -> Option<()> {
    let end = start.checked_add(2)?;
    if end > buffer.len() {
        return None;
    }
    buffer[start] = b'0' + ((value >> 4) & 0x0F);
    buffer[start + 1] = b'0' + (value & 0x0F);
    Some(())
}

fn red_label_bcd_add_byte(lhs: u8, rhs: u8, carry: bool) -> (u8, bool) {
    let decimal_sum =
        red_label_bcd_byte_to_u16(lhs) + red_label_bcd_byte_to_u16(rhs) + u16::from(carry);
    (
        red_label_decimal_to_bcd_byte((decimal_sum % 100) as u8),
        decimal_sum >= 100,
    )
}

fn red_label_bcd_byte_to_u16(value: u8) -> u16 {
    u16::from(value >> 4) * 10 + u16::from(value & 0x0F)
}

fn red_label_decimal_to_bcd_byte(value: u8) -> u8 {
    ((value / 10) << 4) | (value % 10)
}

fn red_label_high_score_default_cells(defaults: &[RedLabelCmosDefault]) -> Option<Vec<u8>> {
    let start = usize::from(RED_LABEL_CRHSTD_CELL_OFFSET);
    let end = start.checked_add(RED_LABEL_HIGH_SCORE_CELLS)?;
    let mut high_score_defaults = defaults
        .iter()
        .filter(|default| {
            let offset = usize::from(default.offset);
            offset >= start && offset < end
        })
        .collect::<Vec<_>>();
    high_score_defaults.sort_by_key(|default| default.offset);

    let mut cells = Vec::with_capacity(RED_LABEL_HIGH_SCORE_CELLS);
    for default in high_score_defaults {
        cells.extend(default.encoded_cells());
    }

    (cells.len() == RED_LABEL_HIGH_SCORE_CELLS).then_some(cells)
}

pub fn video_control_cocktail(value: u8) -> bool {
    value & 0x01 != 0
}

pub fn video_counter_read_value(vpos: u16) -> u8 {
    if vpos < VIDEO_COUNTER_CLAMP_VPOS {
        (vpos as u8) & VIDEO_COUNTER_CLAMPED_VALUE
    } else {
        VIDEO_COUNTER_CLAMPED_VALUE
    }
}

pub fn is_main_cpu_rom_bank(bank: u8) -> bool {
    MAIN_CPU_ROM_BANKS.contains(&bank)
}

pub fn main_cpu_read_target(address: u16, bank_select: u8) -> MainCpuReadTarget {
    match address {
        MAIN_CPU_RAM_START..=MAIN_CPU_RAM_END => MainCpuReadTarget::MainRam { offset: address },
        MAIN_CPU_BANKED_ROM_START..=MAIN_CPU_BANKED_ROM_END => {
            banked_read_target(address, bank_select)
        }
        MAIN_CPU_FIXED_ROM_START..=MAIN_CPU_FIXED_ROM_END => MainCpuReadTarget::FixedRom {
            offset: address - MAIN_CPU_FIXED_ROM_START,
        },
    }
}

pub fn main_cpu_write_target(address: u16, bank_select: u8) -> MainCpuWriteTarget {
    match address {
        MAIN_CPU_RAM_START..=MAIN_CPU_RAM_END => MainCpuWriteTarget::MainRam { offset: address },
        MAIN_CPU_BANKED_ROM_START..=MAIN_CPU_BANKED_ROM_END => {
            banked_write_target(address, bank_select)
        }
        MAIN_CPU_BANK_SELECT_WRITE..=0xDFFF => MainCpuWriteTarget::BankSelect {
            offset: address - MAIN_CPU_BANK_SELECT_WRITE,
        },
        0xE000..=MAIN_CPU_FIXED_ROM_END => MainCpuWriteTarget::FixedRom {
            offset: address - MAIN_CPU_FIXED_ROM_START,
        },
    }
}

pub fn defender_io_window(address: u16) -> DefenderIoWindow {
    debug_assert!((MAIN_CPU_BANKED_ROM_START..=MAIN_CPU_BANKED_ROM_END).contains(&address));

    if address == 0xC3FF {
        return DefenderIoWindow::WatchdogReset;
    }

    if let Some(offset) = mirrored_offset(address, 0xC000, 0xC00F, 0x03E0) {
        return DefenderIoWindow::PaletteRam {
            index: offset as u8,
        };
    }

    if let Some(offset) = mirrored_offset(address, 0xC010, 0xC01F, 0x03E0) {
        return DefenderIoWindow::VideoControl {
            register: offset as u8,
        };
    }

    if let Some(offset) = mirrored_offset(address, 0xC400, 0xC4FF, 0x0300) {
        return DefenderIoWindow::Cmos {
            offset: offset as u8,
        };
    }

    if (0xC800..=0xCBFF).contains(&address) {
        return DefenderIoWindow::VideoCounter {
            offset: address - 0xC800,
        };
    }

    if let Some(offset) = mirrored_offset(address, 0xCC00, 0xCC03, 0x03E0) {
        return DefenderIoWindow::Pia1 {
            register: offset as u8,
        };
    }

    if let Some(offset) = mirrored_offset(address, 0xCC04, 0xCC07, 0x03E0) {
        return DefenderIoWindow::Pia2 {
            register: offset as u8,
        };
    }

    DefenderIoWindow::Unused {
        offset: address - MAIN_CPU_BANKED_ROM_START,
    }
}

fn banked_read_target(address: u16, bank_select: u8) -> MainCpuReadTarget {
    let offset = address - MAIN_CPU_BANKED_ROM_START;
    if bank_select == MAIN_CPU_IO_BANK {
        MainCpuReadTarget::BankedIo(defender_io_window(address))
    } else if is_main_cpu_rom_bank(bank_select) {
        MainCpuReadTarget::BankedRom {
            bank: bank_select,
            offset,
        }
    } else {
        MainCpuReadTarget::EmptyBank {
            bank: bank_select,
            offset,
        }
    }
}

fn banked_write_target(address: u16, bank_select: u8) -> MainCpuWriteTarget {
    let offset = address - MAIN_CPU_BANKED_ROM_START;
    if bank_select == MAIN_CPU_IO_BANK {
        MainCpuWriteTarget::BankedIo(defender_io_window(address))
    } else if is_main_cpu_rom_bank(bank_select) {
        MainCpuWriteTarget::BankedRom {
            bank: bank_select,
            offset,
        }
    } else {
        MainCpuWriteTarget::EmptyBank {
            bank: bank_select,
            offset,
        }
    }
}

/// Source-shaped `CROM0` text/palette intent.
///
/// This records the message vectors and values around `SETCOL`, `VWTEXT`,
/// `VOPERI`, and `VWNUMB`; bitmap text transfer into video RAM remains a
/// separate translation step.
/// Source: <https://github.com/mwenge/defender/blob/master/src/romc0.src#L193-L250>.
/// Source: <https://github.com/mwenge/defender/blob/master/src/mess0.src#L73-L96>.
pub fn red_label_crom0_diagnostic_screen(
    stage: &RedLabelCrom0RomStage,
) -> RedLabelCrom0DiagnosticScreen {
    let mut screen = RedLabelCrom0DiagnosticScreen {
        letter_color: stage
            .text_color
            .map(|value| RedLabelDiagnosticPaletteWrite {
                address: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_ADDRESS,
                index: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX,
                value,
            }),
        ..RedLabelCrom0DiagnosticScreen::default()
    };

    match stage.status {
        RedLabelCrom0RomStageStatus::RomFailure => {
            if let Some(address) = stage.headline_address {
                screen.headline = Some(RedLabelDiagnosticTextWrite {
                    address,
                    vector_label: "VROMFL",
                    text: RED_LABEL_CROM0_ROM_FAILURE_TEXT,
                });
                screen
                    .instructions
                    .push(RedLabelDiagnosticInstructionWrite {
                        table_label: "IROMFL",
                        lines: RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_INSTRUCTIONS,
                    });
            }

            screen
                .bad_roms
                .extend(stage.bad_rom_displays.iter().map(|display| {
                    RedLabelCrom0BadRomScreenWrite {
                        row_address: display.cursor_address,
                        text_vector_label: "VWROM",
                        text: RED_LABEL_CROM0_BAD_ROM_LABEL_TEXT,
                        rom_number: display.rom_number,
                        rom_number_bcd: red_label_decimal_to_bcd_byte(display.rom_number),
                    }
                }));

            if matches!(stage.target, RedLabelCrom0RomStageTarget::WaitForNextSwitch)
                && stage.final_led.is_some()
            {
                screen
                    .instructions
                    .push(RedLabelDiagnosticInstructionWrite {
                        table_label: "IROMDO",
                        lines: RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_INSTRUCTIONS,
                    });
            }
        }
        RedLabelCrom0RomStageStatus::AllRomsOk => {
            if let Some(address) = stage.headline_address {
                screen.headline = Some(RedLabelDiagnosticTextWrite {
                    address,
                    vector_label: "VALROM",
                    text: RED_LABEL_CROM0_ALL_ROMS_OK_TEXT,
                });
                screen
                    .instructions
                    .push(RedLabelDiagnosticInstructionWrite {
                        table_label: "IROMDO",
                        lines: RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_INSTRUCTIONS,
                    });
            }
        }
    }

    screen
}

pub fn red_label_diagnostic_led_output(source_value: u8) -> RedLabelDiagnosticLedOutput {
    let (mut pia1_port_a, mut carry) = lsr(source_value);
    for _ in 0..3 {
        (pia1_port_a, carry) = ror(pia1_port_a, carry);
    }
    if pia1_port_a & 0x80 == 0 {
        pia1_port_a = pia1_port_a.wrapping_add(1);
    }
    for _ in 0..2 {
        (pia1_port_a, carry) = ror(pia1_port_a, carry);
    }

    RedLabelDiagnosticLedOutput {
        source_value,
        pia1_port_a,
        pia1_port_b: pia1_port_a.wrapping_shl(3) | 0x3F,
    }
}

fn red_label_bcd_number_visible_digits(bcd_number: u8) -> Vec<u8> {
    let high = bcd_number >> 4;
    let low = bcd_number & 0x0F;
    if high == 0 {
        vec![low]
    } else {
        vec![high, low]
    }
}

fn red_label_crom0_operator_instruction_vector(line: &str) -> Result<&'static str, String> {
    match line {
        RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_TEXT => Ok("VINS4"),
        RED_LABEL_CROM0_AUTO_TO_EXIT_TEST_TEXT => Ok("VINS5"),
        RED_LABEL_CROM0_AUTO_FOR_CMOS_RAM_TEST_TEXT => Ok("VINS6"),
        _ => Err(format!(
            "red-label CROM0 operator instruction `{line}` has no message vector"
        )),
    }
}

fn red_label_ram_block_led_source(block_number: u8) -> Result<u8, String> {
    if !(1..=3).contains(&block_number) {
        return Err(format!(
            "red-label RAM failure block {block_number} is outside 1..=3"
        ));
    }
    Ok(0x10 >> block_number)
}

fn red_label_crom0_ram_test_loop_step(
    pass: RedLabelCrom0RamTestPass,
    operator_abort: bool,
) -> RedLabelCrom0RamTestLoopStep {
    if let Some(failure) = pass.verification.failure {
        return RedLabelCrom0RamTestLoopStep {
            status: RedLabelCrom0RamTestLoopStatus::Failure,
            target: RedLabelCrom0RamTestLoopTarget::RamFailureScreen,
            pass,
            next_seed: None,
            next_test_counter: None,
            abort_test_counter: None,
            failure: Some(failure),
        };
    }

    if operator_abort {
        return RedLabelCrom0RamTestLoopStep {
            status: RedLabelCrom0RamTestLoopStatus::OperatorAbort,
            target: RedLabelCrom0RamTestLoopTarget::RamTestAbortScreen,
            next_seed: pass.verification.next_seed,
            next_test_counter: None,
            abort_test_counter: Some(pass.test_counter),
            failure: None,
            pass,
        };
    }

    RedLabelCrom0RamTestLoopStep {
        status: RedLabelCrom0RamTestLoopStatus::Continue,
        target: RedLabelCrom0RamTestLoopTarget::RamTestActiveLoop,
        next_seed: pass.verification.next_seed,
        next_test_counter: pass.next_test_counter,
        abort_test_counter: None,
        failure: None,
        pass,
    }
}

/// Source `RAM3` / `RAM4` / `RAM5` pseudo-random word step used by `RAM2`.
///
/// Source: <https://github.com/mwenge/defender/blob/master/src/romf8.src#L99-L116>.
pub fn red_label_crom0_ram_test_next_word(seed: u16) -> u16 {
    let [mut a, mut b] = seed.to_be_bytes();
    b = !b;
    if b & 0x09 != 0 {
        b = !b;
        if b & 0x09 != 0 {
            let carry = a & 0x01 != 0;
            a >>= 1;
            (b, _) = ror(b, carry);
        } else {
            let carry;
            (a, carry) = ror(a, true);
            (b, _) = ror(b, carry);
        }
    } else {
        b = !b;
        let carry;
        (a, carry) = ror(a, true);
        (b, _) = ror(b, carry);
    }
    u16::from_be_bytes([a, b])
}

fn red_label_text_cursor_advance(cursor: u16, width: u8) -> u16 {
    let [x, y] = cursor.to_be_bytes();
    u16::from_be_bytes([x.wrapping_add(width).wrapping_add(1), y])
}

fn red_label_screen_offset(address: u16, offset: u16) -> Result<u16, String> {
    let target = address.checked_add(offset).ok_or_else(|| {
        format!("red-label screen address 0x{address:04X}+0x{offset:04X} overflows")
    })?;
    if target > MAIN_CPU_RAM_END {
        return Err(format!(
            "red-label screen address 0x{target:04X} is outside main RAM"
        ));
    }
    Ok(target)
}

fn lsr(value: u8) -> (u8, bool) {
    (value >> 1, value & 1 != 0)
}

fn ror(value: u8, carry: bool) -> (u8, bool) {
    ((value >> 1) | if carry { 0x80 } else { 0 }, value & 1 != 0)
}

fn mirrored_offset(address: u16, start: u16, end: u16, mirror_mask: u16) -> Option<u16> {
    let canonical = address & !mirror_mask;
    if (start..=end).contains(&canonical) {
        Some(canonical - start)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{
            CMOS_RAM_SIZE, DefenderIoWindow, DefenderMainBoard, DefenderMainCpuRomBus,
            MAIN_CPU_BANK_SELECT_WRITE, MAIN_CPU_IO_BANK, MAIN_CPU_RAM_SIZE, MainCpuReadError,
            MainCpuReadTarget, MainCpuReadWindow, MainCpuRomRead, MainCpuWriteError,
            MainCpuWriteTarget, PALETTE_RAM_SIZE, RED_LABEL_AUDIT_DISPLAY_VISIBLE_CHARS,
            RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS, RED_LABEL_AUDIT_REPEAT_SCAN_DELAY_TICKS,
            RED_LABEL_CLRALL_PACKED_BYTE_WRITES, RED_LABEL_CLRAUD_PACKED_BYTE_WRITES,
            RED_LABEL_CMOSCK_CELL_OFFSET, RED_LABEL_CRHSTD_CELL_OFFSET,
            RED_LABEL_CROM0_ALL_ROMS_OK_TEXT, RED_LABEL_CROM0_AUTO_FOR_CMOS_RAM_TEST_TEXT,
            RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_INSTRUCTIONS, RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_TEXT,
            RED_LABEL_CROM0_AUTO_TO_EXIT_TEST_TEXT, RED_LABEL_CROM0_BAD_RAM_LABEL_TEXT,
            RED_LABEL_CROM0_BAD_RAM_TEXT_ADDRESS, RED_LABEL_CROM0_BAD_ROM_LABEL_TEXT,
            RED_LABEL_CROM0_NO_RAM_ERRORS_TEXT, RED_LABEL_CROM0_NO_RAM_ERRORS_TEXT_ADDRESS,
            RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES, RED_LABEL_CROM0_OPERATOR_PROMPT_ADDRESS,
            RED_LABEL_CROM0_OPERATOR_PROMPT_TEXT, RED_LABEL_CROM0_RAM_FAILURE_TEXT,
            RED_LABEL_CROM0_RAM_FAILURE_TEXT_ADDRESS,
            RED_LABEL_CROM0_RAM_TEST_ACTIVE_LOOP_DELAY_MS, RED_LABEL_CROM0_RAM_TEST_COLOR,
            RED_LABEL_CROM0_RAM_TEST_DELAY_MS, RED_LABEL_CROM0_RAM_TEST_END_ADDRESS,
            RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER, RED_LABEL_CROM0_RAM_TEST_LED,
            RED_LABEL_CROM0_RAM_TEST_START_SEED, RED_LABEL_CROM0_RAM_TEST_TEXT,
            RED_LABEL_CROM0_RAM_TEST_TEXT_ADDRESS, RED_LABEL_CROM0_RAM_TEST_WORDS,
            RED_LABEL_CROM0_ROM_FAILURE_TEXT, RED_LABEL_DIAGNOSTIC_LED_FLASH_DELAY_MS,
            RED_LABEL_DIAGNOSTIC_LED_FLASH_REPETITIONS, RED_LABEL_DIAGNOSTIC_LETTER_COLOR_ADDRESS,
            RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX, RED_LABEL_DIPFLG_CELL_OFFSET,
            RED_LABEL_DIPSW_CELL_OFFSET, RED_LABEL_HIGH_SCORE_CELLS, RED_LABEL_RESET_PALETTE_BYTES,
            RED_LABEL_SCREEN_CLEAR_END, RED_LABEL_THSTAB_START, RedLabelAuditAdjustmentChange,
            RedLabelAuditAdjustmentDirection, RedLabelAuditAdjustmentValue,
            RedLabelAuditCycleState, RedLabelAuditCycleStep, RedLabelAuditDebounceState,
            RedLabelAuditDebounceStep, RedLabelAuditOperatorState, RedLabelAuditOperatorStep,
            RedLabelCrom0BadRamBitmapTextWrite, RedLabelCrom0BadRomBitmapTextWrite,
            RedLabelCrom0BadRomScreenWrite, RedLabelCrom0RamFailure,
            RedLabelCrom0RamFailureTransfer, RedLabelCrom0RamTestAbortStatus,
            RedLabelCrom0RamTestAbortTransfer, RedLabelCrom0RamTestLoopStatus,
            RedLabelCrom0RamTestLoopTarget, RedLabelCrom0RamTestPass,
            RedLabelCrom0RamTestPatternFill, RedLabelCrom0RamTestPatternVerification,
            RedLabelCrom0RamTestStartTransfer, RedLabelCrom0RamTestTarget,
            RedLabelDiagnosticBitmapTextWrite, RedLabelDiagnosticInstructionBitmapTextWrite,
            RedLabelDiagnosticInstructionWrite, RedLabelDiagnosticLedFlash,
            RedLabelDiagnosticLedOutput, RedLabelDiagnosticPaletteWrite,
            RedLabelDiagnosticTextWrite, RedLabelPowerUpAction, RedLabelPowerUpDispatchTarget,
            WATCHDOG_RESET_BYTE, cmos_4bit_write_value, cmos_sram_clear_packed_bytes,
            cmos_sram_read_byte, cmos_sram_read_word, cmos_sram_write_byte, cmos_sram_write_word,
            defender_io_window, is_main_cpu_rom_bank, main_cpu_read_target, main_cpu_write_target,
            red_label_crom0_diagnostic_screen, red_label_crom0_ram_test_next_word,
            red_label_diagnostic_led_output, video_control_cocktail, video_counter_read_value,
        },
        input::{
            CabinetInput, DEFENDER_IN0_FIRE, DEFENDER_IN0_THRUST, DEFENDER_IN1_ALTITUDE_UP,
            DEFENDER_IN2_COIN_ONE, DEFENDER_IN2_HIGH_SCORE_RESET, DefenderInputPorts,
        },
        pia::PIA_IRQ1,
        red_label_memory::{
            MemoryMapCpu, RedLabelMemoryMapEntry, red_label_audit_adjustments,
            red_label_cmos_defaults, red_label_cmos_layout, red_label_memory_map,
            red_label_ram_layout,
        },
        red_label_message::{red_label_message_glyph, red_label_score_digit_image},
        rom::{
            RED_LABEL_CROM0_ALL_ROMS_OK_TEXT_ADDRESS, RED_LABEL_CROM0_FAILURE_COLOR,
            RED_LABEL_CROM0_OK_COLOR, RED_LABEL_CROM0_ROM_FAILURE_TEXT_ADDRESS,
            RedLabelCrom0AdvanceGate, RedLabelCrom0BadRomDisplay, RedLabelCrom0RomStage,
            RedLabelCrom0RomStageStatus, RedLabelCrom0RomStageTarget, RedLabelRomImages,
            RomDescriptor, RomLoad, RomRegion, RomView, VerifiedRomFile, VerifiedRomSet,
        },
    };

    fn test_rom_images() -> RedLabelRomImages {
        let mut fixed = vec![0; 0x3000];
        fixed[0] = 0xD0;
        fixed[0x2FFF] = 0xFF;

        let mut bank_one = vec![0; 0x1000];
        bank_one[0] = 0xB1;
        bank_one[0x0FFF] = 0xC1;

        let mut bank_seven = vec![0; 0x0800];
        bank_seven[0] = 0xB7;
        bank_seven[0x07FF] = 0x77;

        let rom_set = VerifiedRomSet::from_files_for_test(vec![
            VerifiedRomFile {
                descriptor: RomDescriptor {
                    name: "fixed.rom",
                    size: fixed.len() as u64,
                    crc32: "00000000",
                },
                crc32: 0,
                bytes: fixed,
            },
            VerifiedRomFile {
                descriptor: RomDescriptor {
                    name: "bank1.rom",
                    size: bank_one.len() as u64,
                    crc32: "00000000",
                },
                crc32: 0,
                bytes: bank_one,
            },
            VerifiedRomFile {
                descriptor: RomDescriptor {
                    name: "bank7.rom",
                    size: bank_seven.len() as u64,
                    crc32: "00000000",
                },
                crc32: 0,
                bytes: bank_seven,
            },
        ]);
        let regions = [
            RomRegion {
                name: "maincpu",
                size: 0x3000,
                source: "test",
            },
            RomRegion {
                name: "banked",
                size: 0x7000,
                source: "test",
            },
        ];
        let loads = [
            RomLoad {
                name: "fixed.rom",
                region: "maincpu",
                region_offset: 0,
                size: 0x3000,
                view: RomView::Fixed,
                cpu_start: Some(0xD000),
            },
            RomLoad {
                name: "bank1.rom",
                region: "banked",
                region_offset: 0,
                size: 0x1000,
                view: RomView::Banked(1),
                cpu_start: Some(0xC000),
            },
            RomLoad {
                name: "bank7.rom",
                region: "banked",
                region_offset: 0x6000,
                size: 0x0800,
                view: RomView::Banked(7),
                cpu_start: Some(0xC000),
            },
        ];

        RedLabelRomImages::from_parts_for_test(&rom_set, &regions, &loads)
            .expect("test ROM image should build")
    }

    fn assert_message_glyph_at(
        board: &DefenderMainBoard<'_>,
        screen_address: u16,
        character: char,
    ) {
        let glyph = red_label_message_glyph(character).expect("message glyph");
        assert_image_bytes_at(
            board,
            screen_address,
            glyph.width,
            glyph.height,
            &glyph.bytes,
        );
    }

    fn assert_score_digit_at(board: &DefenderMainBoard<'_>, screen_address: u16, digit: u8) {
        let image = red_label_score_digit_image(digit).expect("score digit");
        assert_image_bytes_at(
            board,
            screen_address,
            image.width,
            image.height,
            &image.bytes,
        );
    }

    fn assert_image_bytes_at(
        board: &DefenderMainBoard<'_>,
        screen_address: u16,
        width: u8,
        height: u8,
        bytes: &[u8],
    ) {
        for column in 0..width {
            let source_column = usize::from(column) * usize::from(height);
            for row in 0..height {
                let address =
                    usize::from(screen_address + (u16::from(column) << 8) + u16::from(row));
                assert_eq!(
                    board.ram()[address],
                    bytes[source_column + usize::from(row)]
                );
            }
        }
    }

    fn main_memory_entry(handler: &str) -> RedLabelMemoryMapEntry {
        red_label_memory_map()
            .expect("memory map parses")
            .into_iter()
            .find(|entry| entry.cpu == MemoryMapCpu::Main && entry.handler == handler)
            .expect("memory map handler exists")
    }

    #[test]
    fn main_cpu_rom_bus_reads_fixed_rom_without_bank_selection() {
        let images = test_rom_images();
        let bus = DefenderMainCpuRomBus::new(&images);

        assert_eq!(bus.bank_select(), MAIN_CPU_IO_BANK);
        assert_eq!(bus.read_byte(0xD000), Some(0xD0));
        assert_eq!(bus.read(0xFFFF), Some(MainCpuRomRead::Fixed(0xFF)));
        assert_eq!(bus.read_window(0xD000), MainCpuReadWindow::FixedRom);
    }

    #[test]
    fn main_cpu_rom_bus_reads_selected_banked_rom() {
        let images = test_rom_images();
        let mut bus = DefenderMainCpuRomBus::new(&images);

        assert_eq!(bus.read_byte(0xC000), None);
        assert_eq!(bus.read_window(0xC000), MainCpuReadWindow::NonRom);

        bus.write_bank_select(1);
        assert_eq!(bus.bank_select(), 1);
        assert_eq!(
            bus.read(0xC000),
            Some(MainCpuRomRead::Banked {
                bank: 1,
                byte: 0xB1,
            })
        );
        assert_eq!(bus.read_byte(0xCFFF), Some(0xC1));

        bus.write_bank_select(0x17);
        assert_eq!(bus.bank_select(), 7);
        assert_eq!(bus.read_byte(0xC000), Some(0xB7));
        assert_eq!(bus.read_byte(0xC7FF), Some(0x77));
        assert_eq!(bus.read_byte(0xC800), None);
    }

    #[test]
    fn main_cpu_rom_bus_keeps_unknown_banks_non_rom() {
        let images = test_rom_images();
        let mut bus = DefenderMainCpuRomBus::new(&images);

        bus.write_bank_select(0x14);

        assert_eq!(bus.bank_select(), 4);
        assert_eq!(bus.read_window(0xC000), MainCpuReadWindow::NonRom);
        assert_eq!(bus.read_byte(0xC000), None);
        assert!(is_main_cpu_rom_bank(1));
        assert!(!is_main_cpu_rom_bank(MAIN_CPU_IO_BANK));
        assert_eq!(MAIN_CPU_BANK_SELECT_WRITE, 0xD000);
    }

    #[test]
    fn main_cpu_read_target_classifies_mame_map_boundaries() {
        assert_eq!(
            main_cpu_read_target(0x0000, MAIN_CPU_IO_BANK),
            MainCpuReadTarget::MainRam { offset: 0x0000 }
        );
        assert_eq!(
            main_cpu_read_target(0xBFFF, MAIN_CPU_IO_BANK),
            MainCpuReadTarget::MainRam { offset: 0xBFFF }
        );
        assert_eq!(
            main_cpu_read_target(0xC000, MAIN_CPU_IO_BANK),
            MainCpuReadTarget::BankedIo(DefenderIoWindow::PaletteRam { index: 0 })
        );
        assert_eq!(
            main_cpu_read_target(0xC000, 1),
            MainCpuReadTarget::BankedRom { bank: 1, offset: 0 }
        );
        assert_eq!(
            main_cpu_read_target(0xC000, 4),
            MainCpuReadTarget::EmptyBank { bank: 4, offset: 0 }
        );
        assert_eq!(
            main_cpu_read_target(0xD000, MAIN_CPU_IO_BANK),
            MainCpuReadTarget::FixedRom { offset: 0 }
        );
        assert_eq!(
            main_cpu_read_target(0xFFFF, MAIN_CPU_IO_BANK),
            MainCpuReadTarget::FixedRom { offset: 0x2FFF }
        );
    }

    #[test]
    fn main_cpu_write_target_keeps_bank_select_separate_from_fixed_rom_reads() {
        assert_eq!(
            main_cpu_write_target(0xD000, MAIN_CPU_IO_BANK),
            MainCpuWriteTarget::BankSelect { offset: 0 }
        );
        assert_eq!(
            main_cpu_write_target(0xDFFF, MAIN_CPU_IO_BANK),
            MainCpuWriteTarget::BankSelect { offset: 0x0FFF }
        );
        assert_eq!(
            main_cpu_write_target(0xE000, MAIN_CPU_IO_BANK),
            MainCpuWriteTarget::FixedRom { offset: 0x1000 }
        );
    }

    #[test]
    fn main_cpu_address_classifier_matches_embedded_memory_map_asset() {
        let ram = main_memory_entry("ram");
        assert_eq!(
            main_cpu_read_target(ram.start, MAIN_CPU_IO_BANK),
            MainCpuReadTarget::MainRam { offset: 0 }
        );
        assert_eq!(
            main_cpu_write_target(ram.end, MAIN_CPU_IO_BANK),
            MainCpuWriteTarget::MainRam { offset: ram.end }
        );

        let banked_rom = main_memory_entry("banked_rom");
        assert_eq!(
            main_cpu_read_target(banked_rom.start, 1),
            MainCpuReadTarget::BankedRom { bank: 1, offset: 0 }
        );
        assert_eq!(
            main_cpu_read_target(banked_rom.end, 7),
            MainCpuReadTarget::BankedRom {
                bank: 7,
                offset: banked_rom.end - banked_rom.start,
            }
        );

        let empty_bank = main_memory_entry("empty_bank");
        assert_eq!(
            main_cpu_write_target(empty_bank.start, 4),
            MainCpuWriteTarget::EmptyBank { bank: 4, offset: 0 }
        );

        let bank_select = main_memory_entry("bank_select");
        assert_eq!(
            main_cpu_write_target(bank_select.end, MAIN_CPU_IO_BANK),
            MainCpuWriteTarget::BankSelect {
                offset: bank_select.end - bank_select.start,
            }
        );

        let fixed_rom = main_memory_entry("fixed_rom");
        assert_eq!(
            main_cpu_read_target(fixed_rom.end, MAIN_CPU_IO_BANK),
            MainCpuReadTarget::FixedRom {
                offset: fixed_rom.end - fixed_rom.start,
            }
        );

        let fixed_write = main_memory_entry("fixed_rom_read_only");
        assert_eq!(
            main_cpu_write_target(fixed_write.start, MAIN_CPU_IO_BANK),
            MainCpuWriteTarget::FixedRom {
                offset: fixed_write.start - fixed_rom.start,
            }
        );
    }

    #[test]
    fn defender_io_window_classifies_mirrored_mame_handlers() {
        assert_eq!(
            defender_io_window(0xC000),
            DefenderIoWindow::PaletteRam { index: 0 }
        );
        assert_eq!(
            defender_io_window(0xC3EF),
            DefenderIoWindow::PaletteRam { index: 0x0F }
        );
        assert_eq!(
            defender_io_window(0xC010),
            DefenderIoWindow::VideoControl { register: 0 }
        );
        assert_eq!(defender_io_window(0xC3FF), DefenderIoWindow::WatchdogReset);
        assert_eq!(
            defender_io_window(0xC7FF),
            DefenderIoWindow::Cmos { offset: 0xFF }
        );
        assert_eq!(
            defender_io_window(0xC800),
            DefenderIoWindow::VideoCounter { offset: 0 }
        );
        assert_eq!(
            defender_io_window(0xCBFF),
            DefenderIoWindow::VideoCounter { offset: 0x03FF }
        );
        assert_eq!(
            defender_io_window(0xCCE3),
            DefenderIoWindow::Pia1 { register: 3 }
        );
        assert_eq!(
            defender_io_window(0xCFE4),
            DefenderIoWindow::Pia2 { register: 0 }
        );
        assert_eq!(
            defender_io_window(0xCC08),
            DefenderIoWindow::Unused { offset: 0x0C08 }
        );
    }

    #[test]
    fn defender_io_window_matches_embedded_memory_map_asset() {
        let palette = main_memory_entry("palette_ram");
        assert_eq!(
            defender_io_window(palette.start),
            DefenderIoWindow::PaletteRam { index: 0 }
        );
        assert_eq!(
            defender_io_window(palette.start | palette.mirror_mask.expect("palette mirror") | 0x0F),
            DefenderIoWindow::PaletteRam { index: 0x0F }
        );

        let video_control = main_memory_entry("video_control");
        assert_eq!(
            defender_io_window(video_control.start),
            DefenderIoWindow::VideoControl { register: 0 }
        );

        let watchdog = main_memory_entry("watchdog_reset");
        assert_eq!(
            defender_io_window(watchdog.start),
            DefenderIoWindow::WatchdogReset
        );

        let cmos = main_memory_entry("cmos");
        assert_eq!(
            defender_io_window(cmos.start | cmos.mirror_mask.expect("CMOS mirror") | 0xFF),
            DefenderIoWindow::Cmos { offset: 0xFF }
        );

        let video_counter = main_memory_entry("video_counter");
        assert_eq!(
            defender_io_window(video_counter.end),
            DefenderIoWindow::VideoCounter {
                offset: video_counter.end - video_counter.start,
            }
        );

        let pia0 = main_memory_entry("pia0");
        assert_eq!(
            defender_io_window(pia0.start | pia0.mirror_mask.expect("PIA0 mirror") | 0x03),
            DefenderIoWindow::Pia1 { register: 3 }
        );

        let pia1 = main_memory_entry("pia1");
        assert_eq!(
            defender_io_window(pia1.start | pia1.mirror_mask.expect("PIA1 mirror") | 0x03),
            DefenderIoWindow::Pia2 { register: 3 }
        );
    }

    #[test]
    fn main_board_reads_and_writes_mame_mapped_ram() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(board.ram().len(), MAIN_CPU_RAM_SIZE);
        assert_eq!(board.read_byte(0x0000), Ok(0));
        assert_eq!(board.read_byte(0xBFFF), Ok(0));

        board.write_byte(0x0000, 0x12).expect("write low RAM");
        board.write_byte(0xBFFF, 0xFE).expect("write high RAM");

        assert_eq!(board.read_byte(0x0000), Ok(0x12));
        assert_eq!(board.read_byte(0xBFFF), Ok(0xFE));
        assert_eq!(board.ram()[0], 0x12);
        assert_eq!(board.ram()[MAIN_CPU_RAM_SIZE - 1], 0xFE);
    }

    #[test]
    fn main_board_can_snapshot_source_owned_ram_layout_fields() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let fields = red_label_ram_layout().expect("RAM layout parses");
        let p2_wave = fields
            .iter()
            .find(|entry| entry.table == "player" && entry.field == "PWAV")
            .expect("player wave field");
        let object_data = fields
            .iter()
            .find(|entry| entry.table == "object" && entry.field == "ODATA")
            .expect("object data field");

        board.write_byte(0xA207, 0x05).expect("write P2 wave");
        board
            .write_byte(0xA23C + 0x15, 0x14)
            .expect("write object data timer");
        board
            .write_byte(0xA23C + 0x16, 0x01)
            .expect("write object data alive flag");

        assert_eq!(board.red_label_ram_field(p2_wave, 1), Some(&[0x05][..]));
        assert_eq!(
            board.red_label_ram_field(object_data, 0),
            Some(&[0x14, 0x01][..])
        );
        assert_eq!(board.red_label_ram_field(p2_wave, 2), None);
        assert_eq!(board.ram_range(0xC000..0xC001), None);
    }

    #[test]
    fn main_board_exposes_mame_pia_input_port_callbacks() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(board.input_ports(), DefenderInputPorts::EMPTY);
        assert_eq!(board.pia0_port_a_input(), 0);
        assert_eq!(board.pia0_port_b_input(), 0);
        assert_eq!(board.pia1_port_a_input(), 0);

        board.set_cabinet_input(CabinetInput {
            coin: true,
            altitude_up: true,
            thrust: true,
            fire: true,
            high_score_reset: true,
            ..CabinetInput::NONE
        });
        assert_eq!(
            board.pia0_port_a_input(),
            DEFENDER_IN0_FIRE | DEFENDER_IN0_THRUST
        );
        assert_eq!(board.pia0_port_b_input(), DEFENDER_IN1_ALTITUDE_UP);
        assert_eq!(
            board.pia1_port_a_input(),
            DEFENDER_IN2_COIN_ONE | DEFENDER_IN2_HIGH_SCORE_RESET
        );

        board.set_input_ports(DefenderInputPorts {
            in0: 0xAA,
            in1: 0x55,
            in2: 0xA5,
        });
        assert_eq!(board.pia0_port_a_input(), 0xAA);
        assert_eq!(board.pia0_port_b_input(), 0x55);
        assert_eq!(board.pia1_port_a_input(), 0xA5);
    }

    #[test]
    fn main_board_reads_mame_pia_inputs_after_data_register_selection() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        board.set_cabinet_input(CabinetInput {
            coin: true,
            altitude_up: true,
            thrust: true,
            fire: true,
            high_score_reset: true,
            ..CabinetInput::NONE
        });

        assert_eq!(board.read_byte(0xCC00), Ok(0x00));
        assert_eq!(board.read_byte(0xCC04), Ok(0x00));
        assert_eq!(board.read_byte(0xCC06), Ok(0x00));

        board
            .write_byte(0xCC01, 0x04)
            .expect("select PIA1 port A data");
        board
            .write_byte(0xCC05, 0x04)
            .expect("select PIA2 port A data");
        board
            .write_byte(0xCC07, 0x04)
            .expect("select PIA2 port B data");

        assert_eq!(
            board.read_byte(0xCC00),
            Ok(DEFENDER_IN2_COIN_ONE | DEFENDER_IN2_HIGH_SCORE_RESET)
        );
        assert_eq!(
            board.read_byte(0xCC04),
            Ok(DEFENDER_IN0_FIRE | DEFENDER_IN0_THRUST)
        );
        assert_eq!(board.read_byte(0xCC06), Ok(DEFENDER_IN1_ALTITUDE_UP));
    }

    #[test]
    fn main_board_red_label_reset_setup_matches_source_pia_and_palette_writes() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        board
            .write_byte(MAIN_CPU_BANK_SELECT_WRITE, 7)
            .expect("dirty bank select");
        board
            .write_byte(MAIN_CPU_BANK_SELECT_WRITE, MAIN_CPU_IO_BANK)
            .expect("select I/O bank");
        board.write_byte(0xC000, 0xAA).expect("dirty palette");

        board
            .red_label_reset_setup()
            .expect("source RESET setup writes are mapped");

        assert_eq!(board.bank_select(), MAIN_CPU_IO_BANK);
        assert_eq!(board.pia1().ddr_a(), 0xC0);
        assert_eq!(board.pia1().ddr_b(), 0xFF);
        assert_eq!(board.pia1().control_a(), 0x14);
        assert_eq!(board.pia1().control_b(), 0x04);
        assert_eq!(board.pia2().ddr_a(), 0x00);
        assert_eq!(board.pia2().ddr_b(), 0x00);
        assert_eq!(board.pia2().control_a(), 0x04);
        assert_eq!(board.pia2().control_b(), 0x04);
        assert_eq!(board.palette_ram(), &RED_LABEL_RESET_PALETTE_BYTES);
    }

    #[test]
    fn main_board_diagnostic_led_output_matches_source_leds_rotation() {
        assert_eq!(
            red_label_diagnostic_led_output(0x08),
            RedLabelDiagnosticLedOutput {
                source_value: 0x08,
                pia1_port_a: 0xC0,
                pia1_port_b: 0x3F,
            }
        );
        assert_eq!(
            red_label_diagnostic_led_output(0x04),
            RedLabelDiagnosticLedOutput {
                source_value: 0x04,
                pia1_port_a: 0x20,
                pia1_port_b: 0x3F,
            }
        );
        assert_eq!(
            red_label_diagnostic_led_output(0x02),
            RedLabelDiagnosticLedOutput {
                source_value: 0x02,
                pia1_port_a: 0x90,
                pia1_port_b: 0xBF,
            }
        );
        assert_eq!(
            red_label_diagnostic_led_output(0x01),
            RedLabelDiagnosticLedOutput {
                source_value: 0x01,
                pia1_port_a: 0x88,
                pia1_port_b: 0x7F,
            }
        );
        assert_eq!(
            red_label_diagnostic_led_output(0x00),
            RedLabelDiagnosticLedOutput {
                source_value: 0x00,
                pia1_port_a: 0x80,
                pia1_port_b: 0x3F,
            }
        );
    }

    #[test]
    fn main_board_applies_crom0_rom_stage_steady_leds() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let stage = RedLabelCrom0RomStage {
            status: RedLabelCrom0RomStageStatus::RomFailure,
            text_color: None,
            headline_address: None,
            initial_led: Some(0x08),
            final_led: Some(0x02),
            flash_led: None,
            bad_rom_displays: Vec::new(),
            advance_gates: Vec::new(),
            target: RedLabelCrom0RomStageTarget::WaitForNextSwitch,
        };

        board.red_label_apply_crom0_rom_stage(&stage);

        assert_eq!(
            board.diagnostic_led_output(),
            red_label_diagnostic_led_output(0x02)
        );
        assert!(board.diagnostic_led_flashes().is_empty());
    }

    #[test]
    fn main_board_records_crom0_rom_stage_flash_leds() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let stage = RedLabelCrom0RomStage {
            status: RedLabelCrom0RomStageStatus::AllRomsOk,
            text_color: None,
            headline_address: None,
            initial_led: None,
            final_led: None,
            flash_led: Some(0x08),
            bad_rom_displays: Vec::new(),
            advance_gates: Vec::new(),
            target: RedLabelCrom0RomStageTarget::WaitForNextSwitch,
        };

        board.red_label_apply_crom0_rom_stage(&stage);

        assert_eq!(
            board.diagnostic_led_flashes(),
            &[RedLabelDiagnosticLedFlash {
                source_value: 0x08,
                repetitions: RED_LABEL_DIAGNOSTIC_LED_FLASH_REPETITIONS,
                delay_ms: RED_LABEL_DIAGNOSTIC_LED_FLASH_DELAY_MS,
            }]
        );
        assert_eq!(
            board.diagnostic_led_output(),
            red_label_diagnostic_led_output(0)
        );
    }

    #[test]
    fn main_board_records_crom0_failure_screen_and_advance_gates() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let stage = RedLabelCrom0RomStage {
            status: RedLabelCrom0RomStageStatus::RomFailure,
            text_color: Some(RED_LABEL_CROM0_FAILURE_COLOR),
            headline_address: Some(RED_LABEL_CROM0_ROM_FAILURE_TEXT_ADDRESS),
            initial_led: Some(0x08),
            final_led: Some(12),
            flash_led: None,
            bad_rom_displays: vec![
                RedLabelCrom0BadRomDisplay {
                    rom_number: 2,
                    cursor_address: 0x4270,
                },
                RedLabelCrom0BadRomDisplay {
                    rom_number: 12,
                    cursor_address: 0x427A,
                },
            ],
            advance_gates: vec![
                RedLabelCrom0AdvanceGate::AdvanceSwitchReleaseThenPress,
                RedLabelCrom0AdvanceGate::NextTestAutoCounter,
            ],
            target: RedLabelCrom0RomStageTarget::WaitForNextSwitch,
        };
        let expected_screen = red_label_crom0_diagnostic_screen(&stage);

        board.red_label_apply_crom0_rom_stage(&stage);

        assert_eq!(
            expected_screen.letter_color,
            Some(RedLabelDiagnosticPaletteWrite {
                address: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_ADDRESS,
                index: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX,
                value: RED_LABEL_CROM0_FAILURE_COLOR,
            })
        );
        assert_eq!(
            expected_screen.headline,
            Some(RedLabelDiagnosticTextWrite {
                address: RED_LABEL_CROM0_ROM_FAILURE_TEXT_ADDRESS,
                vector_label: "VROMFL",
                text: RED_LABEL_CROM0_ROM_FAILURE_TEXT,
            })
        );
        assert_eq!(
            expected_screen.instructions.as_slice(),
            &[
                RedLabelDiagnosticInstructionWrite {
                    table_label: "IROMFL",
                    lines: RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_INSTRUCTIONS,
                },
                RedLabelDiagnosticInstructionWrite {
                    table_label: "IROMDO",
                    lines: RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_INSTRUCTIONS,
                },
            ]
        );
        assert_eq!(
            expected_screen.bad_roms.as_slice(),
            &[
                RedLabelCrom0BadRomScreenWrite {
                    row_address: 0x4270,
                    text_vector_label: "VWROM",
                    text: RED_LABEL_CROM0_BAD_ROM_LABEL_TEXT,
                    rom_number: 2,
                    rom_number_bcd: 0x02,
                },
                RedLabelCrom0BadRomScreenWrite {
                    row_address: 0x427A,
                    text_vector_label: "VWROM",
                    text: RED_LABEL_CROM0_BAD_ROM_LABEL_TEXT,
                    rom_number: 12,
                    rom_number_bcd: 0x12,
                },
            ]
        );
        assert_eq!(board.palette_ram()[1], RED_LABEL_CROM0_FAILURE_COLOR);
        assert_eq!(board.crom0_diagnostic_screen(), &expected_screen);
        assert_eq!(
            board.crom0_advance_gates(),
            &[
                RedLabelCrom0AdvanceGate::AdvanceSwitchReleaseThenPress,
                RedLabelCrom0AdvanceGate::NextTestAutoCounter,
            ]
        );
    }

    #[test]
    fn main_board_records_crom0_success_screen() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let stage = RedLabelCrom0RomStage {
            status: RedLabelCrom0RomStageStatus::AllRomsOk,
            text_color: Some(RED_LABEL_CROM0_OK_COLOR),
            headline_address: Some(RED_LABEL_CROM0_ALL_ROMS_OK_TEXT_ADDRESS),
            initial_led: None,
            final_led: None,
            flash_led: Some(0x08),
            bad_rom_displays: Vec::new(),
            advance_gates: vec![RedLabelCrom0AdvanceGate::NextTestAutoCounter],
            target: RedLabelCrom0RomStageTarget::WaitForNextSwitch,
        };

        board.red_label_apply_crom0_rom_stage(&stage);

        assert_eq!(board.palette_ram()[1], RED_LABEL_CROM0_OK_COLOR);
        assert_eq!(
            board.crom0_diagnostic_screen().headline,
            Some(RedLabelDiagnosticTextWrite {
                address: RED_LABEL_CROM0_ALL_ROMS_OK_TEXT_ADDRESS,
                vector_label: "VALROM",
                text: RED_LABEL_CROM0_ALL_ROMS_OK_TEXT,
            })
        );
        assert_eq!(
            board.crom0_diagnostic_screen().instructions.as_slice(),
            &[RedLabelDiagnosticInstructionWrite {
                table_label: "IROMDO",
                lines: RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_INSTRUCTIONS,
            }]
        );
        assert_eq!(
            board.crom0_advance_gates(),
            &[RedLabelCrom0AdvanceGate::NextTestAutoCounter]
        );
    }

    #[test]
    fn main_board_writes_crom0_failure_bitmap_text_to_video_ram() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let stage = RedLabelCrom0RomStage {
            status: RedLabelCrom0RomStageStatus::RomFailure,
            text_color: Some(RED_LABEL_CROM0_FAILURE_COLOR),
            headline_address: Some(RED_LABEL_CROM0_ROM_FAILURE_TEXT_ADDRESS),
            initial_led: Some(0x08),
            final_led: Some(12),
            flash_led: None,
            bad_rom_displays: vec![RedLabelCrom0BadRomDisplay {
                rom_number: 12,
                cursor_address: 0x427A,
            }],
            advance_gates: vec![
                RedLabelCrom0AdvanceGate::AdvanceSwitchReleaseThenPress,
                RedLabelCrom0AdvanceGate::NextTestAutoCounter,
            ],
            target: RedLabelCrom0RomStageTarget::WaitForNextSwitch,
        };

        let transfer = board
            .red_label_write_crom0_diagnostic_text(&stage)
            .expect("CROM0 text transfer");

        assert_eq!(
            transfer.headline,
            Some(RedLabelDiagnosticBitmapTextWrite {
                address: RED_LABEL_CROM0_ROM_FAILURE_TEXT_ADDRESS,
                vector_label: "VROMFL",
                text: String::from(RED_LABEL_CROM0_ROM_FAILURE_TEXT),
                cursor_after: 0x6460,
            })
        );
        assert_eq!(
            transfer.bad_roms,
            vec![RedLabelCrom0BadRomBitmapTextWrite {
                row_address: 0x427A,
                text: String::from("ROM 12"),
                rom_number: 12,
                rom_number_bcd: 0x12,
                cursor_after: 0x597A,
            }]
        );
        assert_eq!(
            transfer.instructions,
            vec![
                RedLabelDiagnosticInstructionBitmapTextWrite {
                    table_label: "IROMFL",
                    prompt: RedLabelDiagnosticBitmapTextWrite {
                        address: RED_LABEL_CROM0_OPERATOR_PROMPT_ADDRESS,
                        vector_label: "VINS1",
                        text: String::from(RED_LABEL_CROM0_OPERATOR_PROMPT_TEXT),
                        cursor_after: 0x96CE,
                    },
                    lines: vec![RedLabelDiagnosticBitmapTextWrite {
                        address: RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES[0],
                        vector_label: "VINS4",
                        text: String::from(RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_TEXT),
                        cursor_after: 0x51DA,
                    }],
                },
                RedLabelDiagnosticInstructionBitmapTextWrite {
                    table_label: "IROMDO",
                    prompt: RedLabelDiagnosticBitmapTextWrite {
                        address: RED_LABEL_CROM0_OPERATOR_PROMPT_ADDRESS,
                        vector_label: "VINS1",
                        text: String::from(RED_LABEL_CROM0_OPERATOR_PROMPT_TEXT),
                        cursor_after: 0x96CE,
                    },
                    lines: vec![RedLabelDiagnosticBitmapTextWrite {
                        address: RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES[0],
                        vector_label: "VINS4",
                        text: String::from(RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_TEXT),
                        cursor_after: 0x51DA,
                    }],
                },
            ]
        );
        assert_message_glyph_at(&board, RED_LABEL_CROM0_ROM_FAILURE_TEXT_ADDRESS, 'R');
        assert_message_glyph_at(&board, 0x4760, 'F');
        assert_score_digit_at(&board, 0x517A, 1);
        assert_score_digit_at(&board, 0x557A, 2);
        assert_message_glyph_at(&board, RED_LABEL_CROM0_OPERATOR_PROMPT_ADDRESS, 'P');
        assert_message_glyph_at(&board, 0x58CE, 'H');
        assert_message_glyph_at(&board, 0x92CE, ':');
        assert_message_glyph_at(&board, RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES[0], 'A');
    }

    #[test]
    fn main_board_writes_crom0_success_bitmap_text_to_video_ram() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let stage = RedLabelCrom0RomStage {
            status: RedLabelCrom0RomStageStatus::AllRomsOk,
            text_color: Some(RED_LABEL_CROM0_OK_COLOR),
            headline_address: Some(RED_LABEL_CROM0_ALL_ROMS_OK_TEXT_ADDRESS),
            initial_led: None,
            final_led: None,
            flash_led: Some(0x08),
            bad_rom_displays: Vec::new(),
            advance_gates: vec![RedLabelCrom0AdvanceGate::NextTestAutoCounter],
            target: RedLabelCrom0RomStageTarget::WaitForNextSwitch,
        };

        let transfer = board
            .red_label_write_crom0_diagnostic_text(&stage)
            .expect("CROM0 success text transfer");

        assert_eq!(
            transfer.headline,
            Some(RedLabelDiagnosticBitmapTextWrite {
                address: RED_LABEL_CROM0_ALL_ROMS_OK_TEXT_ADDRESS,
                vector_label: "VALROM",
                text: String::from(RED_LABEL_CROM0_ALL_ROMS_OK_TEXT),
                cursor_after: 0x6380,
            })
        );
        assert!(transfer.bad_roms.is_empty());
        assert_eq!(
            transfer.instructions,
            vec![RedLabelDiagnosticInstructionBitmapTextWrite {
                table_label: "IROMDO",
                prompt: RedLabelDiagnosticBitmapTextWrite {
                    address: RED_LABEL_CROM0_OPERATOR_PROMPT_ADDRESS,
                    vector_label: "VINS1",
                    text: String::from(RED_LABEL_CROM0_OPERATOR_PROMPT_TEXT),
                    cursor_after: 0x96CE,
                },
                lines: vec![RedLabelDiagnosticBitmapTextWrite {
                    address: RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES[0],
                    vector_label: "VINS4",
                    text: String::from(RED_LABEL_CROM0_AUTO_FOR_RAM_TEST_TEXT),
                    cursor_after: 0x51DA,
                }],
            }]
        );
        assert_message_glyph_at(&board, RED_LABEL_CROM0_ALL_ROMS_OK_TEXT_ADDRESS, 'A');
        assert_message_glyph_at(&board, 0x5980, 'O');
        assert_message_glyph_at(&board, 0x5D80, 'K');
        assert_message_glyph_at(&board, RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES[0], 'A');
    }

    #[test]
    fn main_board_writes_crom0_ram_test_start_after_rom_stage_handoff() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let stage = RedLabelCrom0RomStage {
            status: RedLabelCrom0RomStageStatus::RomFailure,
            text_color: Some(RED_LABEL_CROM0_FAILURE_COLOR),
            headline_address: Some(RED_LABEL_CROM0_ROM_FAILURE_TEXT_ADDRESS),
            initial_led: Some(0x08),
            final_led: None,
            flash_led: None,
            bad_rom_displays: vec![RedLabelCrom0BadRomDisplay {
                rom_number: 12,
                cursor_address: 0x427A,
            }],
            advance_gates: vec![RedLabelCrom0AdvanceGate::AdvanceSwitchReleaseThenPress],
            target: RedLabelCrom0RomStageTarget::RamTestStart,
        };

        board
            .red_label_write_crom0_diagnostic_text(&stage)
            .expect("CROM0 failure text transfer");
        assert_ne!(
            board.ram()[usize::from(RED_LABEL_CROM0_ROM_FAILURE_TEXT_ADDRESS)],
            0
        );

        let transfer = board
            .red_label_write_crom0_ram_test_start()
            .expect("RAM test start transfer");

        assert_eq!(
            transfer,
            RedLabelCrom0RamTestStartTransfer {
                screen_clear_end: RED_LABEL_SCREEN_CLEAR_END,
                palette_zeroed: true,
                led_output: red_label_diagnostic_led_output(0),
                letter_color: RedLabelDiagnosticPaletteWrite {
                    address: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_ADDRESS,
                    index: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX,
                    value: RED_LABEL_CROM0_RAM_TEST_COLOR,
                },
                headline: RedLabelDiagnosticBitmapTextWrite {
                    address: RED_LABEL_CROM0_RAM_TEST_TEXT_ADDRESS,
                    vector_label: "VRAMTS",
                    text: String::from(RED_LABEL_CROM0_RAM_TEST_TEXT),
                    cursor_after: 0x6180,
                },
                instructions: RedLabelDiagnosticInstructionBitmapTextWrite {
                    table_label: "IRAMTS",
                    prompt: RedLabelDiagnosticBitmapTextWrite {
                        address: RED_LABEL_CROM0_OPERATOR_PROMPT_ADDRESS,
                        vector_label: "VINS1",
                        text: String::from(RED_LABEL_CROM0_OPERATOR_PROMPT_TEXT),
                        cursor_after: 0x96CE,
                    },
                    lines: vec![RedLabelDiagnosticBitmapTextWrite {
                        address: RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES[0],
                        vector_label: "VINS5",
                        text: String::from(RED_LABEL_CROM0_AUTO_TO_EXIT_TEST_TEXT),
                        cursor_after: 0x4FDA,
                    }],
                },
                delay_ms: RED_LABEL_CROM0_RAM_TEST_DELAY_MS,
                active_loop_delay_ms: RED_LABEL_CROM0_RAM_TEST_ACTIVE_LOOP_DELAY_MS,
                test_counter: RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER,
            }
        );
        assert_eq!(
            board.ram()[usize::from(RED_LABEL_CROM0_ROM_FAILURE_TEXT_ADDRESS)],
            0
        );
        assert_eq!(board.palette_ram()[0], 0);
        assert_eq!(
            board.palette_ram()[usize::from(RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX)],
            RED_LABEL_CROM0_RAM_TEST_COLOR
        );
        assert_eq!(
            board.diagnostic_led_output(),
            red_label_diagnostic_led_output(0)
        );
        assert!(board.crom0_diagnostic_screen().headline.is_none());
        assert!(board.crom0_advance_gates().is_empty());
        assert_message_glyph_at(&board, RED_LABEL_CROM0_RAM_TEST_TEXT_ADDRESS, 'R');
        assert_message_glyph_at(&board, 0x4F80, 'T');
        assert_message_glyph_at(&board, 0x30DA, 'X');
        assert_message_glyph_at(&board, 0x34DA, 'I');
    }

    #[test]
    fn main_board_runs_crom0_ram_test_pattern_pass() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(
            red_label_crom0_ram_test_next_word(RED_LABEL_CROM0_RAM_TEST_START_SEED),
            0x8000
        );
        assert_eq!(red_label_crom0_ram_test_next_word(0x8000), 0xC000);

        let pass = board.red_label_run_crom0_ram_test_pass(
            RED_LABEL_CROM0_RAM_TEST_START_SEED,
            RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER,
        );

        assert_eq!(
            pass,
            RedLabelCrom0RamTestPass {
                test_counter: RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER,
                next_test_counter: Some(RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER - 1),
                fill: RedLabelCrom0RamTestPatternFill {
                    seed: RED_LABEL_CROM0_RAM_TEST_START_SEED,
                    next_seed: 0xCE5C,
                    start_address: 0,
                    end_address: RED_LABEL_CROM0_RAM_TEST_END_ADDRESS,
                    words_written: RED_LABEL_CROM0_RAM_TEST_WORDS,
                    watchdog_reset_count: 0xC0,
                },
                verification: RedLabelCrom0RamTestPatternVerification {
                    seed: RED_LABEL_CROM0_RAM_TEST_START_SEED,
                    next_seed: Some(0xCE5C),
                    start_address: 0,
                    end_address: RED_LABEL_CROM0_RAM_TEST_END_ADDRESS,
                    words_verified: RED_LABEL_CROM0_RAM_TEST_WORDS,
                    watchdog_reset_count: 0xC0,
                    failure: None,
                },
            }
        );
        assert_eq!(
            &board.ram()[0x0000..0x0010],
            &[
                0x80, 0x00, 0xC0, 0x00, 0xE0, 0x00, 0xF0, 0x00, 0xF8, 0x00, 0xFC, 0x00, 0xFE, 0x00,
                0xFF, 0x00,
            ]
        );
        assert_eq!(&board.ram()[0xBFFE..0xC000], &[0xCE, 0x5C]);
    }

    #[test]
    fn main_board_routes_crom0_ram_test_loop_continue_and_abort() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        let first = board.red_label_step_crom0_ram_test_loop(
            RED_LABEL_CROM0_RAM_TEST_START_SEED,
            RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER,
            false,
        );

        assert_eq!(first.status, RedLabelCrom0RamTestLoopStatus::Continue);
        assert_eq!(
            first.target,
            RedLabelCrom0RamTestLoopTarget::RamTestActiveLoop
        );
        assert_eq!(first.next_seed, Some(0xCE5C));
        assert_eq!(
            first.next_test_counter,
            Some(RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER - 1)
        );
        assert_eq!(first.abort_test_counter, None);
        assert_eq!(first.failure, None);

        let abort = board.red_label_step_crom0_ram_test_loop(
            first.next_seed.expect("next seed"),
            first.next_test_counter.expect("next counter"),
            true,
        );

        assert_eq!(abort.status, RedLabelCrom0RamTestLoopStatus::OperatorAbort);
        assert_eq!(
            abort.target,
            RedLabelCrom0RamTestLoopTarget::RamTestAbortScreen
        );
        assert_eq!(abort.next_seed, Some(0x4572));
        assert_eq!(abort.next_test_counter, None);
        assert_eq!(
            abort.abort_test_counter,
            Some(RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER - 1)
        );
        assert_eq!(abort.failure, None);

        let transfer = board
            .red_label_write_crom0_ram_test_abort(abort.abort_test_counter.expect("abort counter"))
            .expect("abort screen");
        assert_eq!(
            transfer.status,
            RedLabelCrom0RamTestAbortStatus::NoErrorsDetected
        );
    }

    #[test]
    fn main_board_routes_crom0_ram_test_loop_failure() {
        let failure = RedLabelCrom0RamFailure {
            failing_address: 0x079E,
            expected_word: 0xA97B,
            actual_word: 0xA97F,
        };
        let pass = RedLabelCrom0RamTestPass {
            test_counter: RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER,
            next_test_counter: None,
            fill: RedLabelCrom0RamTestPatternFill {
                seed: RED_LABEL_CROM0_RAM_TEST_START_SEED,
                next_seed: 0xCE5C,
                start_address: 0,
                end_address: RED_LABEL_CROM0_RAM_TEST_END_ADDRESS,
                words_written: RED_LABEL_CROM0_RAM_TEST_WORDS,
                watchdog_reset_count: 0xC0,
            },
            verification: RedLabelCrom0RamTestPatternVerification {
                seed: RED_LABEL_CROM0_RAM_TEST_START_SEED,
                next_seed: None,
                start_address: 0,
                end_address: RED_LABEL_CROM0_RAM_TEST_END_ADDRESS,
                words_verified: 0x03D0,
                watchdog_reset_count: 0x07,
                failure: Some(failure),
            },
        };

        let step = super::red_label_crom0_ram_test_loop_step(pass.clone(), false);

        assert_eq!(step.status, RedLabelCrom0RamTestLoopStatus::Failure);
        assert_eq!(
            step.target,
            RedLabelCrom0RamTestLoopTarget::RamFailureScreen
        );
        assert_eq!(step.pass, pass);
        assert_eq!(step.next_seed, None);
        assert_eq!(step.next_test_counter, None);
        assert_eq!(step.abort_test_counter, None);
        assert_eq!(step.failure, Some(failure));
    }

    #[test]
    fn main_board_detects_crom0_ram_test_pattern_failure() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        board.red_label_fill_crom0_ram_test_pattern(RED_LABEL_CROM0_RAM_TEST_START_SEED);

        let failing_address = 0x079E;
        let expected_word = u16::from_be_bytes([
            board.ram()[usize::from(failing_address)],
            board.ram()[usize::from(failing_address + 1)],
        ]);
        let actual_word = expected_word ^ 0x0004;
        board
            .write_byte(failing_address + 1, actual_word.to_be_bytes()[1])
            .expect("corrupt RAM-test word");

        let verification =
            board.red_label_verify_crom0_ram_test_pattern(RED_LABEL_CROM0_RAM_TEST_START_SEED);
        let failure = RedLabelCrom0RamFailure {
            failing_address,
            expected_word,
            actual_word,
        };

        assert_eq!(
            verification,
            RedLabelCrom0RamTestPatternVerification {
                seed: RED_LABEL_CROM0_RAM_TEST_START_SEED,
                next_seed: None,
                start_address: 0,
                end_address: RED_LABEL_CROM0_RAM_TEST_END_ADDRESS,
                words_verified: 0x03D0,
                watchdog_reset_count: 0x07,
                failure: Some(failure),
            }
        );

        let transfer = board
            .red_label_write_crom0_ram_failure(failure)
            .expect("RAM failure screen");
        assert_eq!(transfer.error_mask, 0x0004);
        assert_eq!(transfer.ram_row.block_number, 2);
        assert_eq!(transfer.ram_row.bit_number, 3);
        assert_eq!(transfer.ram_row.ram_number_bcd, 0x23);
    }

    #[test]
    fn main_board_writes_crom0_ram_failure_outcome() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        board
            .red_label_write_crom0_ram_test_start()
            .expect("RAM test start transfer");

        let failure = RedLabelCrom0RamFailure {
            failing_address: 0x079E,
            expected_word: 0xAAAA,
            actual_word: 0xAAAE,
        };
        assert_eq!(failure.error_mask(), 0x0004);
        assert_eq!(failure.bad_block_number(), 2);
        assert_eq!(failure.bad_bit_number(), Ok(3));
        assert_eq!(failure.ram_number_bcd(), Ok(0x23));

        let transfer = board
            .red_label_write_crom0_ram_failure(failure)
            .expect("RAM failure transfer");

        assert_eq!(
            transfer,
            RedLabelCrom0RamFailureTransfer {
                screen_clear_end: RED_LABEL_SCREEN_CLEAR_END,
                palette_zeroed: true,
                failing_address: 0x079E,
                expected_word: 0xAAAA,
                actual_word: 0xAAAE,
                error_mask: 0x0004,
                led_output: red_label_diagnostic_led_output(RED_LABEL_CROM0_RAM_TEST_LED),
                letter_color: RedLabelDiagnosticPaletteWrite {
                    address: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_ADDRESS,
                    index: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX,
                    value: RED_LABEL_CROM0_FAILURE_COLOR,
                },
                headline: RedLabelDiagnosticBitmapTextWrite {
                    address: RED_LABEL_CROM0_RAM_FAILURE_TEXT_ADDRESS,
                    vector_label: "VRAMFL",
                    text: String::from(RED_LABEL_CROM0_RAM_FAILURE_TEXT),
                    cursor_after: 0x6470,
                },
                instructions: RedLabelDiagnosticInstructionBitmapTextWrite {
                    table_label: "IRAMFL",
                    prompt: RedLabelDiagnosticBitmapTextWrite {
                        address: RED_LABEL_CROM0_OPERATOR_PROMPT_ADDRESS,
                        vector_label: "VINS1",
                        text: String::from(RED_LABEL_CROM0_OPERATOR_PROMPT_TEXT),
                        cursor_after: 0x96CE,
                    },
                    lines: vec![RedLabelDiagnosticBitmapTextWrite {
                        address: RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES[0],
                        vector_label: "VINS6",
                        text: String::from(RED_LABEL_CROM0_AUTO_FOR_CMOS_RAM_TEST_TEXT),
                        cursor_after: 0x64DA,
                    }],
                },
                ram_row: RedLabelCrom0BadRamBitmapTextWrite {
                    row_address: RED_LABEL_CROM0_BAD_RAM_TEXT_ADDRESS,
                    text: format!("{RED_LABEL_CROM0_BAD_RAM_LABEL_TEXT} 23"),
                    block_number: 2,
                    bit_number: 3,
                    ram_number_bcd: 0x23,
                    cursor_after: 0x5990,
                },
                block_led_output: red_label_diagnostic_led_output(0x04),
                bit_led_output: red_label_diagnostic_led_output(0x03),
                advance_gates: vec![
                    RedLabelCrom0AdvanceGate::AdvanceSwitchReleaseThenPress,
                    RedLabelCrom0AdvanceGate::AdvanceSwitchReleaseThenPress,
                    RedLabelCrom0AdvanceGate::NextTestAutoCounter,
                ],
            }
        );
        assert_eq!(
            board.palette_ram()[usize::from(RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX)],
            RED_LABEL_CROM0_FAILURE_COLOR
        );
        assert_eq!(
            board.crom0_advance_gates(),
            &[
                RedLabelCrom0AdvanceGate::AdvanceSwitchReleaseThenPress,
                RedLabelCrom0AdvanceGate::AdvanceSwitchReleaseThenPress,
                RedLabelCrom0AdvanceGate::NextTestAutoCounter,
            ]
        );
        assert_message_glyph_at(&board, RED_LABEL_CROM0_RAM_FAILURE_TEXT_ADDRESS, 'R');
        assert_message_glyph_at(&board, 0x4770, 'F');
        assert_message_glyph_at(&board, RED_LABEL_CROM0_BAD_RAM_TEXT_ADDRESS, 'R');
        assert_score_digit_at(&board, 0x5190, 2);
        assert_score_digit_at(&board, 0x5590, 3);
        assert_message_glyph_at(&board, 0x30DA, 'C');
    }

    #[test]
    fn main_board_writes_crom0_ram_abort_and_no_error_outcomes() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        board
            .red_label_write_crom0_ram_test_start()
            .expect("RAM test start transfer");

        let early = board
            .red_label_write_crom0_ram_test_abort(RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER)
            .expect("early RAM-test abort transfer");

        assert_eq!(
            early,
            RedLabelCrom0RamTestAbortTransfer {
                screen_clear_end: RED_LABEL_SCREEN_CLEAR_END,
                palette_zeroed: true,
                test_counter: RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER,
                status: RedLabelCrom0RamTestAbortStatus::EarlyAbort,
                target: RedLabelCrom0RamTestTarget::CmosRamTest,
                letter_color: None,
                headline: None,
                instructions: None,
                flash_led: None,
                advance_gates: Vec::new(),
            }
        );
        assert_eq!(
            board.ram()[usize::from(RED_LABEL_CROM0_RAM_TEST_TEXT_ADDRESS)],
            0
        );
        assert_eq!(board.palette_ram(), &[0; PALETTE_RAM_SIZE]);
        assert!(board.crom0_advance_gates().is_empty());

        let transfer = board
            .red_label_write_crom0_ram_test_abort(RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER - 1)
            .expect("no-error RAM-test abort transfer");

        assert_eq!(
            transfer,
            RedLabelCrom0RamTestAbortTransfer {
                screen_clear_end: RED_LABEL_SCREEN_CLEAR_END,
                palette_zeroed: true,
                test_counter: RED_LABEL_CROM0_RAM_TEST_INITIAL_COUNTER - 1,
                status: RedLabelCrom0RamTestAbortStatus::NoErrorsDetected,
                target: RedLabelCrom0RamTestTarget::WaitForNextSwitch,
                letter_color: Some(RedLabelDiagnosticPaletteWrite {
                    address: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_ADDRESS,
                    index: RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX,
                    value: RED_LABEL_CROM0_OK_COLOR,
                }),
                headline: Some(RedLabelDiagnosticBitmapTextWrite {
                    address: RED_LABEL_CROM0_NO_RAM_ERRORS_TEXT_ADDRESS,
                    vector_label: "VNORAM",
                    text: String::from(RED_LABEL_CROM0_NO_RAM_ERRORS_TEXT),
                    cursor_after: 0x7D80,
                }),
                instructions: Some(RedLabelDiagnosticInstructionBitmapTextWrite {
                    table_label: "IRAMDO",
                    prompt: RedLabelDiagnosticBitmapTextWrite {
                        address: RED_LABEL_CROM0_OPERATOR_PROMPT_ADDRESS,
                        vector_label: "VINS1",
                        text: String::from(RED_LABEL_CROM0_OPERATOR_PROMPT_TEXT),
                        cursor_after: 0x96CE,
                    },
                    lines: vec![RedLabelDiagnosticBitmapTextWrite {
                        address: RED_LABEL_CROM0_OPERATOR_LINE_ADDRESSES[0],
                        vector_label: "VINS6",
                        text: String::from(RED_LABEL_CROM0_AUTO_FOR_CMOS_RAM_TEST_TEXT),
                        cursor_after: 0x64DA,
                    }],
                }),
                flash_led: Some(RedLabelDiagnosticLedFlash {
                    source_value: RED_LABEL_CROM0_RAM_TEST_LED,
                    repetitions: RED_LABEL_DIAGNOSTIC_LED_FLASH_REPETITIONS,
                    delay_ms: RED_LABEL_DIAGNOSTIC_LED_FLASH_DELAY_MS,
                }),
                advance_gates: vec![RedLabelCrom0AdvanceGate::NextTestAutoCounter],
            }
        );
        assert_eq!(
            board.palette_ram()[usize::from(RED_LABEL_DIAGNOSTIC_LETTER_COLOR_INDEX)],
            RED_LABEL_CROM0_OK_COLOR
        );
        assert_eq!(
            board.diagnostic_led_flashes(),
            &[RedLabelDiagnosticLedFlash {
                source_value: RED_LABEL_CROM0_RAM_TEST_LED,
                repetitions: RED_LABEL_DIAGNOSTIC_LED_FLASH_REPETITIONS,
                delay_ms: RED_LABEL_DIAGNOSTIC_LED_FLASH_DELAY_MS,
            }]
        );
        assert_eq!(
            board.crom0_advance_gates(),
            &[RedLabelCrom0AdvanceGate::NextTestAutoCounter]
        );
        assert_message_glyph_at(&board, RED_LABEL_CROM0_NO_RAM_ERRORS_TEXT_ADDRESS, 'N');
        assert_message_glyph_at(&board, 0x7780, 'D');
        assert_message_glyph_at(&board, 0x30DA, 'C');
    }

    #[test]
    fn main_board_pia_ddr_masks_input_bits_until_data_register_is_selected() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        board.set_cabinet_input(CabinetInput {
            coin: true,
            high_score_reset: true,
            ..CabinetInput::NONE
        });

        board.write_byte(0xCC00, 0xF0).expect("write PIA1 DDRA");
        assert_eq!(board.read_byte(0xCC00), Ok(0xF0));
        assert_eq!(board.pia1().ddr_a(), 0xF0);

        board
            .write_byte(0xCC01, 0x44)
            .expect("select PIA1 port A data and mask IRQ bits");

        assert_eq!(board.pia1().control_a(), 0x04);
        assert_eq!(board.read_byte(0xCC00), Ok(DEFENDER_IN2_HIGH_SCORE_RESET));
    }

    #[test]
    fn main_board_pia1_port_b_sound_command_uses_mame_ddr_filtered_output() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        board
            .write_byte(0xCC02, 0xFF)
            .expect("PIA1 port B defaults to DDR selection");
        let ddr_latch = board
            .last_sound_command_latch()
            .expect("MAME calls output handler when DDR changes");
        assert_eq!(ddr_latch.port_b().raw(), 0xC0);
        assert_eq!(board.pia1().ddr_b(), 0xFF);

        board
            .write_byte(0xCC03, 0x04)
            .expect("select PIA1 port B data");
        board
            .write_byte(0xCC02, 0x12)
            .expect("write PIA1 port B output");

        let data_latch = board
            .last_sound_command_latch()
            .expect("PIA1 port B data write should latch sound command");
        assert_eq!(data_latch.port_b().raw(), 0xD2);
        assert!(data_latch.cb1_asserted());
        assert_eq!(board.pia1().out_b(), 0x12);
    }

    #[test]
    fn main_board_video_interrupt_lines_feed_mame_pia1_ca1_and_cb1() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        board
            .write_byte(0xCC01, 0x03)
            .expect("enable PIA1 CA1 low-to-high IRQ");
        board
            .write_byte(0xCC03, 0x03)
            .expect("enable PIA1 CB1 low-to-high IRQ");

        board.update_video_interrupt_lines(31);
        assert!(!board.main_irq_asserted());
        assert_eq!(board.read_byte(0xCC01), Ok(0x03));
        assert_eq!(board.read_byte(0xCC03), Ok(0x03));

        board.update_video_interrupt_lines(32);
        assert!(board.main_irq_asserted());
        assert_eq!(board.read_byte(0xCC03), Ok(PIA_IRQ1 | 0x03));

        board
            .write_byte(0xCC03, 0x07)
            .expect("select PIA1 port B data");
        assert_eq!(board.read_byte(0xCC02), Ok(0x00));
        assert!(!board.pia1().irq_b_asserted());

        board.update_video_interrupt_lines(239);
        assert!(!board.pia1().irq_a_asserted());
        board.update_video_interrupt_lines(240);
        assert!(board.pia1().irq_a_asserted());
        assert_eq!(board.read_byte(0xCC01), Ok(PIA_IRQ1 | 0x03));

        board
            .write_byte(0xCC01, 0x07)
            .expect("select PIA1 port A data");
        assert_eq!(board.read_byte(0xCC00), Ok(0x00));
        assert!(!board.main_irq_asserted());
    }

    #[test]
    fn main_board_bank_select_uses_low_nibble_and_reads_rom() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(board.read_byte(0xD000), Ok(0xD0));

        board
            .write_byte(0xD123, 0x11)
            .expect("bank select mirror should select bank");
        assert_eq!(board.bank_select(), 1);
        assert_eq!(board.read_byte(0xC000), Ok(0xB1));
        assert_eq!(board.read_byte(0xCFFF), Ok(0xC1));

        board
            .write_byte(0xD000, 0x17)
            .expect("bank select should use low nibble");
        assert_eq!(board.bank_select(), 7);
        assert_eq!(board.read_byte(0xC000), Ok(0xB7));
        assert_eq!(board.read_byte(0xC7FF), Ok(0x77));
        assert_eq!(
            board.read_byte(0xC800),
            Err(MainCpuReadError::UnmappedRom { address: 0xC800 })
        );
    }

    #[test]
    fn main_board_reports_unimplemented_hardware_and_rom_writes() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(
            board.read_byte(0xC000),
            Err(MainCpuReadError::Hardware(DefenderIoWindow::PaletteRam {
                index: 0,
            }))
        );
        assert_eq!(
            board.write_byte(0xE000, 0x99),
            Err(MainCpuWriteError::ReadOnlyFixedRom { offset: 0x1000 })
        );

        board.write_byte(0xD000, 0x14).expect("select empty bank");
        assert_eq!(
            board.read_byte(0xC000),
            Err(MainCpuReadError::EmptyBank { bank: 4, offset: 0 })
        );
        assert_eq!(
            board.write_byte(0xC000, 0x99),
            Err(MainCpuWriteError::EmptyBank { bank: 4, offset: 0 })
        );

        board.write_byte(0xD000, 0x11).expect("select ROM bank");
        assert_eq!(
            board.write_byte(0xC000, 0x99),
            Err(MainCpuWriteError::ReadOnlyBankedRom { bank: 1, offset: 0 })
        );
    }

    #[test]
    fn main_board_cmos_writes_store_mame_four_bit_value_and_are_readable() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(cmos_4bit_write_value(0x00), 0xF0);
        assert_eq!(cmos_4bit_write_value(0x05), 0xF5);
        assert_eq!(cmos_4bit_write_value(0xA5), 0xF5);
        assert_eq!(board.cmos_ram()[0x00], 0x00);
        assert_eq!(board.read_byte(0xC400), Ok(0x00));

        board
            .write_byte(0xC400, 0x05)
            .expect("write first CMOS byte");
        board
            .write_byte(0xC7FF, 0xA5)
            .expect("write mirrored CMOS byte");

        assert_eq!(board.cmos_ram()[0x00], 0xF5);
        assert_eq!(board.cmos_ram()[0xFF], 0xF5);
        assert_eq!(board.read_byte(0xC400), Ok(0xF5));
        assert_eq!(board.read_byte(0xC7FF), Ok(0xF5));
    }

    #[test]
    fn main_board_can_snapshot_source_owned_cmos_layout_fields() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let fields = red_label_cmos_layout().expect("CMOS layout parses");
        let replay = fields
            .iter()
            .find(|entry| entry.symbol == "REPLAY")
            .expect("REPLAY field");
        let credit_backup = fields
            .iter()
            .find(|entry| entry.symbol == "CREDST")
            .expect("CREDST marker");

        board
            .write_byte(0xC481, 0x01)
            .expect("write REPLAY high nibble");
        board
            .write_byte(0xC482, 0x00)
            .expect("write REPLAY second nibble");
        board
            .write_byte(0xC483, 0x00)
            .expect("write REPLAY third nibble");
        board
            .write_byte(0xC484, 0x00)
            .expect("write REPLAY low nibble");

        assert_eq!(
            board.red_label_cmos_field(replay),
            Some(&[0xF1, 0xF0, 0xF0, 0xF0][..])
        );
        assert_eq!(board.cmos_sram_read_word(0x81), Some(0x1000));
        assert_eq!(board.red_label_cmos_field(credit_backup), Some(&[][..]));
        assert_eq!(board.cmos_range(0x0100..0x0101), None);
    }

    #[test]
    fn main_board_can_apply_rom_derived_cmos_defaults() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let layout = red_label_cmos_layout().expect("CMOS layout parses");
        let top_high_score = layout
            .iter()
            .find(|entry| entry.symbol == "CRHSTD")
            .expect("CRHSTD layout");
        let restore_wave = layout
            .iter()
            .find(|entry| entry.symbol == "GA1+6")
            .expect("restore-wave layout");

        assert_eq!(board.apply_cmos_defaults(&defaults), Some(()));

        assert_eq!(
            board.red_label_cmos_field(top_high_score),
            Some(
                &[
                    0xF0, 0xF2, 0xF1, 0xF2, 0xF7, 0xF0, 0xF4, 0xF4, 0xF5, 0xF2, 0xF4, 0xFA
                ][..]
            )
        );
        assert_eq!(board.cmos_sram_read_word(0x81), Some(0x0100));
        assert_eq!(board.cmos_sram_read_byte(0x85), Some(0x03));
        assert_eq!(
            board.red_label_cmos_field(restore_wave),
            Some(&[0xF0, 0xF5][..])
        );
        assert_eq!(board.cmos_sram_read_byte(0x99), Some(0x15));
    }

    #[test]
    fn main_board_can_run_red_label_cmos_init_visible_effect() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");

        board.write_byte(0xC400, 0x05).expect("dirty DIPFLG");
        board.write_byte(0xC47F, 0x00).expect("dirty CMOSCK high");
        board
            .write_byte(0xC4AB, 0x09)
            .expect("dirty after defaults");
        board.write_byte(0xC4FF, 0x0C).expect("dirty final cell");

        assert_eq!(board.red_label_cmos_init(&defaults), Some(()));

        assert_eq!(board.cmos_ram()[0], 0xF0);
        assert_eq!(board.cmos_sram_read_word(0x81), Some(0x0100));
        assert_eq!(board.cmos_sram_read_byte(0x85), Some(0x03));
        assert_eq!(board.cmos_sram_read_byte(0x99), Some(0x15));
        assert_eq!(board.cmos_ram()[0xAB], 0xF0);
        assert_eq!(board.cmos_ram()[0xFF], 0xF0);
    }

    #[test]
    fn main_board_can_reset_all_time_and_todays_high_scores_from_defalt() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let expected_top_score_cells = [
            0xF0, 0xF2, 0xF1, 0xF2, 0xF7, 0xF0, 0xF4, 0xF4, 0xF5, 0xF2, 0xF4, 0xFA,
        ];

        board
            .cmos_sram_write_byte(RED_LABEL_CRHSTD_CELL_OFFSET, 0x99)
            .expect("dirty all-time top score");
        board
            .cmos_sram_write_byte(RED_LABEL_CMOSCK_CELL_OFFSET, 0x99)
            .expect("dirty non-high-score CMOS field");
        board
            .write_byte(RED_LABEL_THSTAB_START, 0x09)
            .expect("dirty today's high-score RAM");

        assert_eq!(board.red_label_reset_high_scores(&defaults), Some(()));

        assert_eq!(
            board.cmos_range(
                u16::from(RED_LABEL_CRHSTD_CELL_OFFSET)
                    ..u16::from(RED_LABEL_CRHSTD_CELL_OFFSET) + 12
            ),
            Some(&expected_top_score_cells[..])
        );
        assert_eq!(
            board.ram_range(RED_LABEL_THSTAB_START..RED_LABEL_THSTAB_START + 12),
            Some(&expected_top_score_cells[..])
        );
        assert_eq!(
            board
                .ram_range(
                    RED_LABEL_THSTAB_START
                        ..RED_LABEL_THSTAB_START + RED_LABEL_HIGH_SCORE_CELLS as u16
                )
                .expect("today's table")
                .len(),
            RED_LABEL_HIGH_SCORE_CELLS
        );
        assert_eq!(
            board.cmos_sram_read_byte(RED_LABEL_CMOSCK_CELL_OFFSET),
            Some(0x99)
        );
    }

    #[test]
    fn main_board_can_reset_only_todays_high_scores_from_defalt() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let expected_top_score_cells = [
            0xF0, 0xF2, 0xF1, 0xF2, 0xF7, 0xF0, 0xF4, 0xF4, 0xF5, 0xF2, 0xF4, 0xFA,
        ];

        board
            .cmos_sram_write_byte(RED_LABEL_CRHSTD_CELL_OFFSET, 0x99)
            .expect("dirty all-time top score");

        assert_eq!(
            board.red_label_reset_todays_high_scores(&defaults),
            Some(())
        );

        assert_eq!(
            board.ram_range(RED_LABEL_THSTAB_START..RED_LABEL_THSTAB_START + 12),
            Some(&expected_top_score_cells[..])
        );
        assert_eq!(
            board.cmos_sram_read_byte(RED_LABEL_CRHSTD_CELL_OFFSET),
            Some(0x99)
        );
    }

    #[test]
    fn main_board_power_up_accepts_valid_cmos_without_special_function() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");

        board.red_label_cmos_init(&defaults).expect("CMOS init");

        assert_eq!(
            board.red_label_power_up(&defaults),
            Some(RedLabelPowerUpAction::NoSpecialFunction)
        );
        assert_eq!(
            RedLabelPowerUpAction::NoSpecialFunction.dispatch_target(),
            RedLabelPowerUpDispatchTarget::ReturnToCaller
        );
        assert_eq!(
            board.cmos_sram_read_byte(RED_LABEL_CMOSCK_CELL_OFFSET),
            Some(0x5A)
        );
        assert_eq!(
            board.cmos_ram()[usize::from(RED_LABEL_DIPFLG_CELL_OFFSET)],
            0xF0
        );
    }

    #[test]
    fn main_board_power_up_initializes_cmos_when_check_byte_is_bad() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");

        board.write_byte(0xC47F, 0x00).expect("bad CMOSCK high");
        board.write_byte(0xC4FF, 0x0C).expect("dirty final cell");

        assert_eq!(
            board.red_label_power_up(&defaults),
            Some(RedLabelPowerUpAction::InitializeCmosAndAudit)
        );
        assert_eq!(
            RedLabelPowerUpAction::InitializeCmosAndAudit.dispatch_target(),
            RedLabelPowerUpDispatchTarget::AuditGate
        );
        assert_eq!(
            board.cmos_sram_read_byte(RED_LABEL_CMOSCK_CELL_OFFSET),
            Some(0x5A)
        );
        assert_eq!(board.cmos_sram_read_word(0x81), Some(0x0100));
        assert_eq!(board.cmos_ram()[0xFF], 0xF0);
    }

    #[test]
    fn main_board_power_up_handles_source_special_functions() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");

        board.red_label_cmos_init(&defaults).expect("CMOS init");
        board.write_byte(0xC400, 0x01).expect("set DIPFLG");
        board
            .cmos_sram_write_byte(RED_LABEL_DIPSW_CELL_OFFSET, 0x15)
            .expect("set auto-cycle function");
        assert_eq!(
            board.red_label_power_up(&defaults),
            Some(RedLabelPowerUpAction::AutoCycleRomTest)
        );
        assert_eq!(
            RedLabelPowerUpAction::AutoCycleRomTest.dispatch_target(),
            RedLabelPowerUpDispatchTarget::ComprehensiveRomTest
        );
        assert_eq!(
            board.cmos_ram()[usize::from(RED_LABEL_DIPFLG_CELL_OFFSET)],
            0xF0
        );
        assert_eq!(
            board.cmos_sram_read_byte(RED_LABEL_DIPSW_CELL_OFFSET),
            Some(0x00)
        );

        board.write_byte(0xC400, 0x01).expect("set DIPFLG");
        board
            .cmos_sram_write_byte(RED_LABEL_DIPSW_CELL_OFFSET, 0x25)
            .expect("set high-score reset function");
        board
            .cmos_sram_write_byte(RED_LABEL_CRHSTD_CELL_OFFSET, 0x99)
            .expect("dirty all-time high-score field");
        assert_eq!(
            board.red_label_power_up(&defaults),
            Some(RedLabelPowerUpAction::ResetHighScoreTables)
        );
        assert_eq!(
            RedLabelPowerUpAction::ResetHighScoreTables.dispatch_target(),
            RedLabelPowerUpDispatchTarget::ResetHighScoreTables
        );
        assert_eq!(
            board.cmos_sram_read_byte(RED_LABEL_CRHSTD_CELL_OFFSET),
            Some(0x02)
        );
        assert_eq!(
            board
                .ram_range(RED_LABEL_THSTAB_START..RED_LABEL_THSTAB_START + 2)
                .expect("today's high-score top byte"),
            &[0xF0, 0xF2]
        );

        board.write_byte(0xC400, 0x01).expect("set DIPFLG");
        board
            .cmos_sram_write_byte(RED_LABEL_DIPSW_CELL_OFFSET, 0x35)
            .expect("set audit-clear function");
        board.write_byte(0xC401, 0x09).expect("dirty audit cell");
        assert_eq!(
            board.red_label_power_up(&defaults),
            Some(RedLabelPowerUpAction::ClearAudits)
        );
        assert_eq!(
            RedLabelPowerUpAction::ClearAudits.dispatch_target(),
            RedLabelPowerUpDispatchTarget::ClearAudits
        );
        assert!(board.cmos_ram()[0..0x1C].iter().all(|cell| *cell == 0xF0));

        board.write_byte(0xC400, 0x01).expect("set DIPFLG");
        board
            .cmos_sram_write_byte(RED_LABEL_DIPSW_CELL_OFFSET, 0x55)
            .expect("set unknown function");
        assert_eq!(
            board.red_label_power_up(&defaults),
            Some(RedLabelPowerUpAction::UnknownSpecialFunction(0x55))
        );
        assert_eq!(
            RedLabelPowerUpAction::UnknownSpecialFunction(0x55).dispatch_target(),
            RedLabelPowerUpDispatchTarget::ReturnToCaller
        );
    }

    #[test]
    fn main_board_power_up_default_special_function_runs_cmos_init() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");

        board.red_label_cmos_init(&defaults).expect("CMOS init");
        board.write_byte(0xC400, 0x01).expect("set DIPFLG");
        board
            .cmos_sram_write_byte(RED_LABEL_DIPSW_CELL_OFFSET, 0x45)
            .expect("set default function");
        board.write_byte(0xC4FF, 0x0C).expect("dirty final cell");

        assert_eq!(
            board.red_label_power_up(&defaults),
            Some(RedLabelPowerUpAction::InitializeCmosAndAudit)
        );
        assert_eq!(
            RedLabelPowerUpAction::InitializeCmosAndAudit.dispatch_target(),
            RedLabelPowerUpDispatchTarget::AuditGate
        );
        assert_eq!(board.cmos_ram()[0xFF], 0xF0);
        assert_eq!(board.cmos_sram_read_word(0x81), Some(0x0100));
    }

    #[test]
    fn main_board_reads_auditg_adjustment_values_from_cmos_cells() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let adjustments = red_label_audit_adjustments().expect("audit adjustments parse");

        board.red_label_cmos_init(&defaults).expect("CMOS init");

        let left_coins = adjustments
            .iter()
            .find(|adjustment| adjustment.number == 1)
            .expect("left coin audit row");
        assert_eq!(
            board.red_label_audit_adjustment_value(left_coins),
            Some(RedLabelAuditAdjustmentValue::PackedWord(0x0000))
        );

        board
            .cmos_sram_write_word(left_coins.offset as u8, 0x1234)
            .expect("set left coin counter");
        assert_eq!(
            board.red_label_audit_adjustment_value(left_coins),
            Some(RedLabelAuditAdjustmentValue::PackedWord(0x1234))
        );

        let replay = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "REPLAY")
            .expect("replay adjustment");
        assert_eq!(
            board.red_label_audit_adjustment_value(replay),
            Some(RedLabelAuditAdjustmentValue::PackedWord(0x0100))
        );

        let ships = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "NSHIP")
            .expect("ship-count adjustment");
        assert_eq!(
            board.red_label_audit_adjustment_value(ships),
            Some(RedLabelAuditAdjustmentValue::PackedByte(0x03))
        );

        let special_function = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "DIPSW")
            .expect("special-function adjustment");
        assert_eq!(
            board.red_label_audit_adjustment_value(special_function),
            Some(RedLabelAuditAdjustmentValue::PackedByte(0x00))
        );
    }

    #[test]
    fn main_board_formats_auditg_display_line_like_disaud_buffer() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let adjustments = red_label_audit_adjustments().expect("audit adjustments parse");

        board.red_label_cmos_init(&defaults).expect("CMOS init");

        let left_coins = adjustments
            .iter()
            .find(|adjustment| adjustment.number == 1)
            .expect("left coin audit row");
        board
            .cmos_sram_write_word(left_coins.offset as u8, 0x1234)
            .expect("set left coin counter");
        let left_coins_line = board
            .red_label_audit_display_line(left_coins)
            .expect("left coin display line");
        assert_eq!(left_coins_line.row_number(), 1);
        assert_eq!(
            left_coins_line.value(),
            RedLabelAuditAdjustmentValue::PackedWord(0x1234)
        );
        let left_coins_text = left_coins_line.visible_text().as_bytes();
        assert_eq!(left_coins_text.len(), RED_LABEL_AUDIT_DISPLAY_VISIBLE_CHARS);
        assert_eq!(&left_coins_text[0..2], b"01");
        assert_eq!(&left_coins_text[7..11], b"1234");
        assert_eq!(&left_coins_text[12..22], b"COINS LEFT");

        let replay = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "REPLAY")
            .expect("replay adjustment");
        let replay_line = board
            .red_label_audit_display_line(replay)
            .expect("replay display line");
        let replay_text = replay_line.visible_text().as_bytes();
        assert_eq!(replay_line.row_number(), 8);
        assert_eq!(&replay_text[0..2], b"08");
        assert_eq!(&replay_text[5..11], b"010000");
        assert_eq!(&replay_text[12..28], b"BONUS SHIP LEVEL");

        let ships = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "NSHIP")
            .expect("ship-count adjustment");
        let ships_line = board
            .red_label_audit_display_line(ships)
            .expect("ship-count display line");
        let ships_text = ships_line.visible_text().as_bytes();
        assert_eq!(&ships_text[0..2], b"09");
        assert_eq!(&ships_text[9..11], b"03");
        assert_eq!(&ships_text[12..27], b"NUMBER OF SHIPS");

        let special_function = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "DIPSW")
            .expect("special-function adjustment");
        board
            .cmos_sram_write_byte(RED_LABEL_DIPSW_CELL_OFFSET, 0x45)
            .expect("set special function");
        let special_function_line = board
            .red_label_audit_display_line(special_function)
            .expect("special-function display line");
        let special_function_text = special_function_line.visible_text().as_bytes();
        assert_eq!(&special_function_text[0..2], b"28");
        assert_eq!(&special_function_text[9..11], b"45");
        assert_eq!(&special_function_text[12..28], b"SPECIAL FUNCTION");
    }

    #[test]
    fn main_board_auditg_debounce_uses_source_scan_delay_cadence() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let mut debounce = RedLabelAuditDebounceState::default();

        assert_eq!(debounce.scan_delay(), 0);
        assert_eq!(debounce.remaining_ticks(), 0);

        debounce.begin_after_display();
        assert_eq!(
            debounce.scan_delay(),
            RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS
        );
        assert_eq!(
            debounce.remaining_ticks(),
            RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS + 1
        );

        board.set_cabinet_input(CabinetInput {
            service_advance: true,
            ..CabinetInput::NONE
        });
        for remaining_ticks in (1..=RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS).rev() {
            assert_eq!(
                board.red_label_audit_debounce_tick(&mut debounce),
                Some(RedLabelAuditDebounceStep::Waiting {
                    remaining_ticks,
                    shift_register: 0xFF,
                })
            );
        }
        assert_eq!(
            board.red_label_audit_debounce_tick(&mut debounce),
            Some(RedLabelAuditDebounceStep::TimedOut {
                scan_delay: RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS,
            })
        );

        debounce.begin_after_display();
        assert_eq!(
            debounce.scan_delay(),
            RED_LABEL_AUDIT_REPEAT_SCAN_DELAY_TICKS
        );
        assert_eq!(
            debounce.remaining_ticks(),
            RED_LABEL_AUDIT_REPEAT_SCAN_DELAY_TICKS + 1
        );
        for remaining_ticks in (1..=RED_LABEL_AUDIT_REPEAT_SCAN_DELAY_TICKS).rev() {
            assert_eq!(
                board.red_label_audit_debounce_tick(&mut debounce),
                Some(RedLabelAuditDebounceStep::Waiting {
                    remaining_ticks,
                    shift_register: 0xFF,
                })
            );
        }
        assert_eq!(
            board.red_label_audit_debounce_tick(&mut debounce),
            Some(RedLabelAuditDebounceStep::TimedOut {
                scan_delay: RED_LABEL_AUDIT_REPEAT_SCAN_DELAY_TICKS,
            })
        );

        debounce.begin_after_display();
        assert_eq!(
            debounce.remaining_ticks(),
            RED_LABEL_AUDIT_REPEAT_SCAN_DELAY_TICKS
        );
    }

    #[test]
    fn main_board_auditg_debounce_requires_shifted_release_samples() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let mut debounce = RedLabelAuditDebounceState::default();

        debounce.begin_after_display();
        board.set_cabinet_input(CabinetInput {
            high_score_reset: true,
            ..CabinetInput::NONE
        });
        assert_eq!(
            board.red_label_audit_debounce_tick(&mut debounce),
            Some(RedLabelAuditDebounceStep::Waiting {
                remaining_ticks: RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS,
                shift_register: 0xFF,
            })
        );

        board.set_cabinet_input(CabinetInput::NONE);
        for (remaining_ticks, shift_register) in [
            (99, 0x7F),
            (98, 0x3F),
            (97, 0x1F),
            (96, 0x0F),
            (95, 0x07),
            (94, 0x03),
            (93, 0x01),
        ] {
            assert_eq!(
                board.red_label_audit_debounce_tick(&mut debounce),
                Some(RedLabelAuditDebounceStep::Waiting {
                    remaining_ticks,
                    shift_register,
                })
            );
        }
        assert_eq!(
            board.red_label_audit_debounce_tick(&mut debounce),
            Some(RedLabelAuditDebounceStep::Released { shift_register: 0 })
        );
        assert_eq!(debounce.scan_delay(), 0);
        assert_eq!(debounce.remaining_ticks(), 0);
        assert_eq!(debounce.shift_register(), 0);
        assert_eq!(board.red_label_audit_debounce_tick(&mut debounce), None);
    }

    #[test]
    fn main_board_auditg_cycle_displays_line_and_gates_navigation_until_release() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let adjustments = red_label_audit_adjustments().expect("audit adjustments parse");
        let mut cycle = RedLabelAuditCycleState::default();

        board.red_label_cmos_init(&defaults).expect("CMOS init");
        board.set_cabinet_input(CabinetInput::NONE);
        match board
            .red_label_audit_cycle_step(&mut cycle, &adjustments)
            .expect("initial cycle")
        {
            RedLabelAuditCycleStep::Display { line, change } => {
                assert_eq!(line.row_number(), 1);
                assert_eq!(&line.visible_text().as_bytes()[0..2], b"01");
                assert_eq!(change, None);
            }
            other => panic!("unexpected initial audit cycle step {other:?}"),
        }
        assert_eq!(cycle.operator().row_number(), 1);
        assert_eq!(
            cycle.debounce().remaining_ticks(),
            RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS + 1
        );

        board.set_cabinet_input(CabinetInput {
            service_advance: true,
            ..CabinetInput::NONE
        });
        assert_eq!(
            board.red_label_audit_cycle_step(&mut cycle, &adjustments),
            Some(RedLabelAuditCycleStep::Debounce(
                RedLabelAuditDebounceStep::Waiting {
                    remaining_ticks: RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS,
                    shift_register: 0xFF,
                }
            ))
        );
        assert_eq!(cycle.operator().row_number(), 1);

        board.set_cabinet_input(CabinetInput::NONE);
        for _ in 0..7 {
            assert!(matches!(
                board.red_label_audit_cycle_step(&mut cycle, &adjustments),
                Some(RedLabelAuditCycleStep::Debounce(
                    RedLabelAuditDebounceStep::Waiting { .. }
                ))
            ));
        }
        assert_eq!(
            board.red_label_audit_cycle_step(&mut cycle, &adjustments),
            Some(RedLabelAuditCycleStep::Debounce(
                RedLabelAuditDebounceStep::Released { shift_register: 0 }
            ))
        );

        board.set_cabinet_input(CabinetInput {
            service_advance: true,
            ..CabinetInput::NONE
        });
        match board
            .red_label_audit_cycle_step(&mut cycle, &adjustments)
            .expect("advance cycle")
        {
            RedLabelAuditCycleStep::Display { line, change } => {
                assert_eq!(line.row_number(), 28);
                assert_eq!(&line.visible_text().as_bytes()[0..2], b"28");
                assert_eq!(change, None);
            }
            other => panic!("unexpected manual advance audit cycle step {other:?}"),
        }
        assert_eq!(cycle.operator().row_number(), 28);
    }

    #[test]
    fn main_board_auditg_cycle_applies_change_before_display_line() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let adjustments = red_label_audit_adjustments().expect("audit adjustments parse");
        let mut cycle = RedLabelAuditCycleState::for_displayed_row_number(9).expect("row 9 cycle");

        board.red_label_cmos_init(&defaults).expect("CMOS init");
        board.set_cabinet_input(CabinetInput {
            high_score_reset: true,
            auto_up_manual_down: true,
            ..CabinetInput::NONE
        });

        match board
            .red_label_audit_cycle_step(&mut cycle, &adjustments)
            .expect("adjustment cycle")
        {
            RedLabelAuditCycleStep::Display { line, change } => {
                assert_eq!(line.row_number(), 9);
                assert_eq!(
                    change,
                    Some(RedLabelAuditAdjustmentChange::Changed(
                        RedLabelAuditAdjustmentValue::PackedByte(0x04)
                    ))
                );
                assert_eq!(line.value(), RedLabelAuditAdjustmentValue::PackedByte(0x04));
                assert_eq!(&line.visible_text().as_bytes()[9..11], b"04");
            }
            other => panic!("unexpected adjustment audit cycle step {other:?}"),
        }
        assert_eq!(
            cycle.debounce().remaining_ticks(),
            RED_LABEL_AUDIT_FIRST_SCAN_DELAY_TICKS + 1
        );
    }

    #[test]
    fn main_board_auditg_cycle_exits_after_last_row_auto_advance() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let adjustments = red_label_audit_adjustments().expect("audit adjustments parse");
        let mut cycle =
            RedLabelAuditCycleState::for_displayed_row_number(28).expect("row 28 cycle");

        board.red_label_cmos_init(&defaults).expect("CMOS init");
        board.set_cabinet_input(CabinetInput {
            service_advance: true,
            auto_up_manual_down: true,
            ..CabinetInput::NONE
        });

        assert_eq!(
            board.red_label_audit_cycle_step(&mut cycle, &adjustments),
            Some(RedLabelAuditCycleStep::ReturnToGame)
        );
    }

    #[test]
    fn main_board_alters_auditg_adjustments_like_source_buttons() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let adjustments = red_label_audit_adjustments().expect("audit adjustments parse");

        board.red_label_cmos_init(&defaults).expect("CMOS init");

        let left_coins = adjustments
            .iter()
            .find(|adjustment| adjustment.number == 1)
            .expect("left coin audit row");
        board
            .cmos_sram_write_word(left_coins.offset as u8, 0x1234)
            .expect("dirty audit counter");
        assert_eq!(
            board
                .red_label_alter_audit_adjustment(left_coins, RedLabelAuditAdjustmentDirection::Up),
            Some(RedLabelAuditAdjustmentChange::ReadOnly)
        );
        assert_eq!(
            board.red_label_audit_adjustment_value(left_coins),
            Some(RedLabelAuditAdjustmentValue::PackedWord(0x1234))
        );

        let ships = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "NSHIP")
            .expect("ship-count adjustment");
        assert_eq!(
            board.red_label_alter_audit_adjustment(ships, RedLabelAuditAdjustmentDirection::Up),
            Some(RedLabelAuditAdjustmentChange::Changed(
                RedLabelAuditAdjustmentValue::PackedByte(0x04)
            ))
        );
        assert_eq!(
            board.red_label_alter_audit_adjustment(ships, RedLabelAuditAdjustmentDirection::Down),
            Some(RedLabelAuditAdjustmentChange::Changed(
                RedLabelAuditAdjustmentValue::PackedByte(0x03)
            ))
        );

        let replay = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "REPLAY")
            .expect("replay adjustment");
        board
            .cmos_sram_write_word(replay.offset as u8, 0x0190)
            .expect("set replay level near low-byte carry");
        assert_eq!(
            board.red_label_alter_audit_adjustment(replay, RedLabelAuditAdjustmentDirection::Up),
            Some(RedLabelAuditAdjustmentChange::Changed(
                RedLabelAuditAdjustmentValue::PackedWord(0x0200)
            ))
        );
        assert_eq!(
            board.red_label_alter_audit_adjustment(replay, RedLabelAuditAdjustmentDirection::Down),
            Some(RedLabelAuditAdjustmentChange::Changed(
                RedLabelAuditAdjustmentValue::PackedWord(0x0190)
            ))
        );

        let left_multiplier = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "SLOT1M")
            .expect("left coin multiplier adjustment");
        assert_eq!(
            board.red_label_alter_audit_adjustment(
                left_multiplier,
                RedLabelAuditAdjustmentDirection::Up
            ),
            Some(RedLabelAuditAdjustmentChange::CoinageLocked)
        );
        assert_eq!(
            board.red_label_audit_adjustment_value(left_multiplier),
            Some(RedLabelAuditAdjustmentValue::PackedByte(0x01))
        );

        let coin_select = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "COINSL")
            .expect("coin select adjustment");
        board
            .cmos_sram_write_byte(coin_select.offset as u8, 0x00)
            .expect("unlock coin multiplier rows");
        assert_eq!(
            board.red_label_alter_audit_adjustment(
                left_multiplier,
                RedLabelAuditAdjustmentDirection::Up
            ),
            Some(RedLabelAuditAdjustmentChange::Changed(
                RedLabelAuditAdjustmentValue::PackedByte(0x02)
            ))
        );

        let special_function = adjustments
            .iter()
            .find(|adjustment| adjustment.symbol == "DIPSW")
            .expect("special-function adjustment");
        assert_eq!(
            board.red_label_alter_audit_adjustment(
                special_function,
                RedLabelAuditAdjustmentDirection::Up
            ),
            Some(RedLabelAuditAdjustmentChange::Changed(
                RedLabelAuditAdjustmentValue::PackedByte(0x01)
            ))
        );
        assert_eq!(
            board.cmos_ram()[usize::from(RED_LABEL_DIPFLG_CELL_OFFSET)],
            0xF1
        );
    }

    #[test]
    fn main_board_auditg_operator_step_follows_service_switch_navigation() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let adjustments = red_label_audit_adjustments().expect("audit adjustments parse");
        let mut state = RedLabelAuditOperatorState::default();

        board.red_label_cmos_init(&defaults).expect("CMOS init");

        assert_eq!(state.row_index(), 0);
        assert_eq!(state.row_number(), 1);
        assert!(state.display_pending());

        board.set_cabinet_input(CabinetInput::NONE);
        assert_eq!(
            board.red_label_audit_operator_step(&mut state, &adjustments),
            Some(RedLabelAuditOperatorStep::Display {
                row_number: 1,
                change: None,
            })
        );
        assert!(!state.display_pending());

        assert_eq!(
            board.red_label_audit_operator_step(&mut state, &adjustments),
            Some(RedLabelAuditOperatorStep::Idle {
                row_number: 1,
                change: None,
            })
        );

        board.set_cabinet_input(CabinetInput {
            service_advance: true,
            ..CabinetInput::NONE
        });
        assert_eq!(
            board.red_label_audit_operator_step(&mut state, &adjustments),
            Some(RedLabelAuditOperatorStep::Display {
                row_number: 28,
                change: None,
            })
        );
        assert_eq!(state.row_number(), 28);

        board.set_cabinet_input(CabinetInput {
            service_advance: true,
            auto_up_manual_down: true,
            ..CabinetInput::NONE
        });
        assert_eq!(
            board.red_label_audit_operator_step(&mut state, &adjustments),
            Some(RedLabelAuditOperatorStep::ReturnToGame)
        );
    }

    #[test]
    fn main_board_auditg_operator_step_applies_high_score_reset_adjustments() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");
        let adjustments = red_label_audit_adjustments().expect("audit adjustments parse");

        board.red_label_cmos_init(&defaults).expect("CMOS init");

        let mut read_only_state =
            RedLabelAuditOperatorState::for_displayed_row_number(1).expect("row 1 state");
        board.set_cabinet_input(CabinetInput {
            high_score_reset: true,
            auto_up_manual_down: true,
            ..CabinetInput::NONE
        });
        assert_eq!(
            board.red_label_audit_operator_step(&mut read_only_state, &adjustments),
            Some(RedLabelAuditOperatorStep::Idle {
                row_number: 1,
                change: Some(RedLabelAuditAdjustmentChange::ReadOnly),
            })
        );

        let mut ship_state =
            RedLabelAuditOperatorState::for_displayed_row_number(9).expect("row 9 state");
        assert_eq!(
            board.red_label_audit_operator_step(&mut ship_state, &adjustments),
            Some(RedLabelAuditOperatorStep::Display {
                row_number: 9,
                change: Some(RedLabelAuditAdjustmentChange::Changed(
                    RedLabelAuditAdjustmentValue::PackedByte(0x04)
                )),
            })
        );
        assert!(!ship_state.display_pending());

        board.set_cabinet_input(CabinetInput {
            service_advance: true,
            high_score_reset: true,
            ..CabinetInput::NONE
        });
        assert_eq!(
            board.red_label_audit_operator_step(&mut ship_state, &adjustments),
            Some(RedLabelAuditOperatorStep::Display {
                row_number: 8,
                change: Some(RedLabelAuditAdjustmentChange::Changed(
                    RedLabelAuditAdjustmentValue::PackedWord(0x0090)
                )),
            })
        );
        assert_eq!(ship_state.row_number(), 8);
    }

    #[test]
    fn main_board_cmos_sram_helpers_pack_four_bit_cells_like_red_label_routines() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(cmos_sram_read_byte(board.cmos_ram(), 0), Some(0x00));
        assert_eq!(cmos_sram_read_word(board.cmos_ram(), 0), Some(0x0000));

        assert_eq!(board.cmos_sram_write_byte(0x10, 0xA5), Some(()));
        assert_eq!(board.cmos_ram()[0x10], 0xFA);
        assert_eq!(board.cmos_ram()[0x11], 0xF5);
        assert_eq!(board.cmos_sram_read_byte(0x10), Some(0xA5));
        assert_eq!(cmos_sram_read_byte(board.cmos_ram(), 0x10), Some(0xA5));

        assert_eq!(board.cmos_sram_write_word(0x20, 0x1234), Some(()));
        assert_eq!(&board.cmos_ram()[0x20..=0x23], &[0xF1, 0xF2, 0xF3, 0xF4]);
        assert_eq!(board.cmos_sram_read_word(0x20), Some(0x1234));
        assert_eq!(cmos_sram_read_word(board.cmos_ram(), 0x20), Some(0x1234));

        let mut raw = [0; CMOS_RAM_SIZE];
        assert_eq!(cmos_sram_write_byte(&mut raw, 0xFE, 0x6C), Some(()));
        assert_eq!(cmos_sram_read_byte(&raw, 0xFE), Some(0x6C));
        assert_eq!(cmos_sram_write_byte(&mut raw, 0xFF, 0x6C), None);
        assert_eq!(cmos_sram_write_word(&mut raw, 0xFC, 0x9876), Some(()));
        assert_eq!(cmos_sram_read_word(&raw, 0xFC), Some(0x9876));
        assert_eq!(cmos_sram_write_word(&mut raw, 0xFD, 0x9876), None);
    }

    #[test]
    fn main_board_clraud_matches_source_visible_cell_range() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        board.write_byte(0xC400, 0x01).expect("dirty DIPFLG");
        board
            .write_byte(0xC41B, 0x02)
            .expect("dirty last CLRAUD cell");
        board
            .write_byte(0xC41C, 0x03)
            .expect("dirty cell just after CLRAUD range");

        assert_eq!(board.red_label_clear_cmos_audit_cells(), Some(()));

        assert_eq!(RED_LABEL_CLRAUD_PACKED_BYTE_WRITES, 0x0E);
        assert!(board.cmos_ram()[0..0x1C].iter().all(|cell| *cell == 0xF0));
        assert_eq!(board.cmos_ram()[0x1C], 0xF3);
    }

    #[test]
    fn cmos_clear_helpers_write_zero_nibbles_through_sram_cell_format() {
        let mut raw = [0xA5; CMOS_RAM_SIZE];

        assert_eq!(RED_LABEL_CLRALL_PACKED_BYTE_WRITES, 0x80);
        assert_eq!(cmos_sram_clear_packed_bytes(&mut raw, 0x10, 2), Some(()));
        assert_eq!(&raw[0x0F..=0x14], &[0xA5, 0xF0, 0xF0, 0xF0, 0xF0, 0xA5]);
        assert_eq!(cmos_sram_clear_packed_bytes(&mut raw, 0xFF, 1), None);
    }

    #[test]
    fn main_board_video_control_tracks_mame_cocktail_bit_only() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert!(!board.cocktail());
        assert!(!video_control_cocktail(0x00));
        assert!(video_control_cocktail(0x01));
        assert!(!video_control_cocktail(0x02));

        board.write_byte(0xC010, 0x01).expect("enable cocktail");
        assert!(board.cocktail());

        board
            .write_byte(0xC3FE, 0x02)
            .expect("mirrored video control write");
        assert!(!board.cocktail());
        assert_eq!(
            board.read_byte(0xC010),
            Err(MainCpuReadError::Hardware(DefenderIoWindow::VideoControl {
                register: 0,
            }))
        );
    }

    #[test]
    fn main_board_watchdog_only_counts_mame_reset_byte() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(WATCHDOG_RESET_BYTE, 0x39);
        assert_eq!(board.watchdog_reset_count(), 0);

        board
            .write_byte(0xC3FF, 0x38)
            .expect("non-reset watchdog write is handled");
        assert_eq!(board.watchdog_reset_count(), 0);

        board
            .write_byte(0xC3FF, WATCHDOG_RESET_BYTE)
            .expect("reset watchdog write is handled");
        assert_eq!(board.watchdog_reset_count(), 1);
        assert_eq!(
            board.read_byte(0xC3FF),
            Err(MainCpuReadError::Hardware(DefenderIoWindow::WatchdogReset))
        );
    }

    #[test]
    fn main_board_video_counter_reads_mame_vpos_bits_two_through_seven() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(board.video_counter_vpos(), 0);
        assert_eq!(board.video_counter_read(), 0x00);
        assert_eq!(video_counter_read_value(0x000), 0x00);
        assert_eq!(video_counter_read_value(0x003), 0x00);
        assert_eq!(video_counter_read_value(0x004), 0x04);
        assert_eq!(video_counter_read_value(0x0FE), 0xFC);
        assert_eq!(video_counter_read_value(0x0FF), 0xFC);
        assert_eq!(video_counter_read_value(0x100), 0xFC);
        assert_eq!(video_counter_read_value(0x123), 0xFC);

        board.set_video_counter_vpos(0x2A);
        assert_eq!(board.video_counter_vpos(), 0x2A);
        assert_eq!(board.video_counter_read(), 0x28);
        assert_eq!(board.read_byte(0xC800), Ok(0x28));
        assert_eq!(board.read_byte(0xCBFF), Ok(0x28));

        board.set_video_counter_vpos(0x100);
        assert_eq!(board.read_byte(0xC800), Ok(0xFC));
        assert_eq!(
            board.write_byte(0xC800, 0x00),
            Err(MainCpuWriteError::Hardware(
                DefenderIoWindow::VideoCounter { offset: 0 }
            ))
        );
    }

    #[test]
    fn main_board_tracks_mame_pia1_port_b_sound_output_callback() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(board.last_sound_command_latch(), None);

        let active = board.write_pia1_port_b_output(0x12);
        assert_eq!(active.port_b().raw(), 0xD2);
        assert!(active.cb1_asserted());
        assert_eq!(board.last_sound_command_latch(), Some(active));

        let idle = board.write_pia1_port_b_output(0x3F);
        assert_eq!(idle.port_b().raw(), 0xFF);
        assert!(!idle.cb1_asserted());
        assert_eq!(board.last_sound_command_latch(), Some(idle));
    }

    #[test]
    fn main_board_does_not_treat_pia1_ddr_write_as_raw_sound_command() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        board
            .write_byte(0xCC02, 0x12)
            .expect("default PIA1 port B write selects DDRB");

        let latch = board
            .last_sound_command_latch()
            .expect("MAME emits filtered output when DDR changes");
        assert_eq!(board.pia1().ddr_b(), 0x12);
        assert_eq!(board.pia1().out_b(), 0x00);
        assert_eq!(latch.port_b().raw(), 0xC0);
    }

    #[test]
    fn main_board_stores_raw_palette_ram_writes_only() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        assert_eq!(board.palette_ram().len(), PALETTE_RAM_SIZE);
        assert!(board.palette_ram().iter().all(|entry| *entry == 0));

        board
            .write_byte(0xC000, 0b1101_0110)
            .expect("write palette register 0");
        board
            .write_byte(0xC3EF, 0b0010_1001)
            .expect("write mirrored palette register 15");

        assert_eq!(board.palette_ram()[0], 0b1101_0110);
        assert_eq!(board.palette_ram()[0x0F], 0b0010_1001);
        assert_eq!(
            board.read_byte(0xC000),
            Err(MainCpuReadError::Hardware(DefenderIoWindow::PaletteRam {
                index: 0,
            }))
        );
    }

    #[test]
    fn main_board_exposes_native_visible_palette_indices_from_video_ram() {
        let images = test_rom_images();
        let mut board = DefenderMainBoard::with_cleared_ram(&images);

        board.write_byte(0xC00A, 0x5A).expect("write palette A");
        board.write_byte(0xC00B, 0xB5).expect("write palette B");

        let offset = crate::video::defender_visible_byte_offset(0, 0)
            .expect("visible origin should map to RAM");
        board
            .write_byte(offset as u16, 0xAB)
            .expect("write visible video byte");

        assert_eq!(board.visible_palette_index(0, 0), Some(0x5A));
        assert_eq!(board.visible_palette_index(1, 0), Some(0xB5));
        assert_eq!(
            board.visible_palette_index(crate::machine::VISIBLE_WIDTH, 0),
            None
        );

        let pixels = board
            .visible_palette_indices()
            .expect("main RAM covers the visible screen format");
        assert_eq!(
            pixels.len(),
            usize::from(crate::machine::VISIBLE_WIDTH)
                * usize::from(crate::machine::VISIBLE_HEIGHT)
        );
        assert_eq!(pixels[0], 0x5A);
        assert_eq!(pixels[1], 0xB5);

        let image = board
            .visible_rgba_image()
            .expect("main RAM covers the visible RGBA frame");
        assert_eq!(image.width, u32::from(crate::machine::VISIBLE_WIDTH));
        assert_eq!(image.height, u32::from(crate::machine::VISIBLE_HEIGHT));
        assert_eq!(
            &image.pixels[0..4],
            &crate::video::williams_palette_rgba(0x5A)
        );
        assert_eq!(
            &image.pixels[4..8],
            &crate::video::williams_palette_rgba(0xB5)
        );
    }
}
