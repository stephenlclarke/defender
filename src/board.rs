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
//! `romc0.src` power-up branch. CMOS persistence, post-power-up routine
//! dispatch, LED segment side effects, and full video timing remain explicit
//! fidelity gaps.

use crate::{
    input::{CabinetInput, DefenderInputPorts},
    pia::{Pia6821, PiaOutputEvent},
    red_label_memory::{
        RedLabelCmosDefault, RedLabelCmosLayoutEntry, RedLabelRamLayoutEntry, pack_sram_byte,
        pack_sram_word, unpack_sram_byte, unpack_sram_word,
    },
    rom::RedLabelRomImages,
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
pub const RED_LABEL_HIGH_SCORE_DEFAULT_BYTES: usize = 48;
pub const RED_LABEL_HIGH_SCORE_CELLS: usize = RED_LABEL_HIGH_SCORE_DEFAULT_BYTES * 2;
pub const RED_LABEL_THSTAB_START: u16 = 0xB260;
pub const RED_LABEL_CLRAUD_PACKED_BYTE_WRITES: usize = 0x0E;
pub const RED_LABEL_CLRALL_PACKED_BYTE_WRITES: usize = CMOS_RAM_SIZE / 2;
pub const RED_LABEL_RESET_PALETTE_BYTES: [u8; PALETTE_RAM_SIZE] = [
    0xC0, 0x87, 0x5F, 0x43, 0x2F, 0x21, 0x17, 0x10, 0x0B, 0x07, 0x04, 0x02, 0x01, 0x00, 0x00, 0x00,
];
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
            MainCpuWriteTarget, PALETTE_RAM_SIZE, RED_LABEL_CLRALL_PACKED_BYTE_WRITES,
            RED_LABEL_CLRAUD_PACKED_BYTE_WRITES, RED_LABEL_CMOSCK_CELL_OFFSET,
            RED_LABEL_CRHSTD_CELL_OFFSET, RED_LABEL_DIPFLG_CELL_OFFSET,
            RED_LABEL_DIPSW_CELL_OFFSET, RED_LABEL_HIGH_SCORE_CELLS, RED_LABEL_RESET_PALETTE_BYTES,
            RED_LABEL_THSTAB_START, RedLabelPowerUpAction, RedLabelPowerUpDispatchTarget,
            WATCHDOG_RESET_BYTE, cmos_4bit_write_value, cmos_sram_clear_packed_bytes,
            cmos_sram_read_byte, cmos_sram_read_word, cmos_sram_write_byte, cmos_sram_write_word,
            defender_io_window, is_main_cpu_rom_bank, main_cpu_read_target, main_cpu_write_target,
            video_control_cocktail, video_counter_read_value,
        },
        input::{
            CabinetInput, DEFENDER_IN0_FIRE, DEFENDER_IN0_THRUST, DEFENDER_IN1_ALTITUDE_UP,
            DEFENDER_IN2_COIN_ONE, DEFENDER_IN2_HIGH_SCORE_RESET, DefenderInputPorts,
        },
        pia::PIA_IRQ1,
        red_label_memory::{
            MemoryMapCpu, RedLabelMemoryMapEntry, red_label_cmos_defaults, red_label_cmos_layout,
            red_label_memory_map, red_label_ram_layout,
        },
        rom::{
            RedLabelRomImages, RomDescriptor, RomLoad, RomRegion, RomView, VerifiedRomFile,
            VerifiedRomSet,
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
