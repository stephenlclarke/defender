//! Source-cited sound-command and sound-board helpers for the red-label model.
//!
//! The main CPU writes byte commands to the Williams sound board. This module
//! deliberately models only that raw command boundary; command values must come
//! from translated red-label routines or verified traces before gameplay emits
//! them.
//!
//! The MAME Williams driver documents Defender's 6808 sound-board map as
//! internal RAM at `0x0000..=0x007f`, PIA IC4 at `0x0400..=0x0403` mirrored at
//! `0x8400..=0x8403`, and ROM at `0xb000..=0xffff`.
//! It also documents the main-board sound command path:
//! `williams_state::snd_cmd_w` ORs command bytes with `0xc0`, then
//! `deferred_snd_cmd_w<2>` writes the value to the sound PIA port B and sets
//! CB1 high unless the byte is `0xff`.
//! Source: <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams.cpp>.
//! Source: <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams_m.cpp>.
//! Source: <https://github.com/mamedev/mame/blob/master/src/devices/machine/6821pia.cpp>.
//! This module models the sound PIA data/control register surface, command
//! CB1 IRQ input, and the DAC output callback boundary. It does not emulate
//! sample generation, CPU IRQ scheduling, or `VSNDRM1.SRC` routines yet.

use crate::{
    pia::{Pia6821, PiaOutputEvent},
    rom::RedLabelRomImages,
};

pub const FRAME_SOUND_COMMAND_CAPACITY: usize = 8;
pub const SOUND_CPU_INTERNAL_RAM_START: u16 = 0x0000;
pub const SOUND_CPU_INTERNAL_RAM_END: u16 = 0x007F;
pub const SOUND_CPU_INTERNAL_RAM_SIZE: usize =
    (SOUND_CPU_INTERNAL_RAM_END - SOUND_CPU_INTERNAL_RAM_START + 1) as usize;
pub const SOUND_CPU_PIA_START: u16 = 0x0400;
pub const SOUND_CPU_PIA_END: u16 = 0x0403;
pub const SOUND_CPU_PIA_MIRROR_MASK: u16 = 0x8000;
pub const SOUND_CPU_ROM_START: u16 = 0xB000;
pub const SOUND_CPU_ROM_END: u16 = 0xFFFF;
pub const MAIN_BOARD_SOUND_COMMAND_HIGH_BITS: u8 = 0xC0;
pub const SOUND_COMMAND_IDLE_BYTE: u8 = 0xFF;
pub const SOUND_PIA_UNCONNECTED_PORT_A_INPUT: u8 = 0xFF;

pub type SoundCpuRam = [u8; SOUND_CPU_INTERNAL_RAM_SIZE];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenderSoundIoWindow {
    Pia { register: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCpuAddressTarget {
    InternalRam { offset: u8 },
    Pia { register: u8 },
    Rom { offset: u16 },
    Unmapped { address: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCpuReadError {
    Hardware(DefenderSoundIoWindow),
    UnmappedAddress { address: u16 },
    UnmappedRom { address: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCpuWriteError {
    Hardware(DefenderSoundIoWindow),
    ReadOnlyRom { offset: u16 },
    UnmappedAddress { address: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundCommand {
    raw: u8,
}

impl SoundCommand {
    pub const fn new(raw: u8) -> Self {
        Self { raw }
    }

    pub const fn from_main_board_pia_port_b(raw: u8) -> Self {
        Self {
            raw: raw | MAIN_BOARD_SOUND_COMMAND_HIGH_BITS,
        }
    }

    pub const fn raw(self) -> u8 {
        self.raw
    }

    pub fn hex(self) -> String {
        format!("0x{:02X}", self.raw)
    }
}

pub fn format_sound_command_list(commands: &[SoundCommand]) -> String {
    if commands.is_empty() {
        return String::from("-");
    }

    commands
        .iter()
        .copied()
        .map(SoundCommand::hex)
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundCommandLatch {
    port_b: SoundCommand,
    cb1_asserted: bool,
}

impl SoundCommandLatch {
    pub fn from_main_board_pia_port_b(raw: u8) -> Self {
        let port_b = SoundCommand::from_main_board_pia_port_b(raw);

        Self {
            port_b,
            cb1_asserted: port_b.raw() != SOUND_COMMAND_IDLE_BYTE,
        }
    }

    pub fn port_b(self) -> SoundCommand {
        self.port_b
    }

    pub fn cb1_asserted(self) -> bool {
        self.cb1_asserted
    }
}

#[derive(Debug)]
pub struct DefenderSoundBoard<'a> {
    roms: &'a RedLabelRomImages,
    ram: SoundCpuRam,
    pia: Pia6821,
    sound_pia_port_b_input: u8,
    last_command_latch: Option<SoundCommandLatch>,
    last_dac_value: Option<u8>,
}

impl<'a> DefenderSoundBoard<'a> {
    pub fn from_ram(roms: &'a RedLabelRomImages, ram: SoundCpuRam) -> Self {
        Self {
            roms,
            ram,
            pia: Pia6821::default(),
            sound_pia_port_b_input: 0,
            last_command_latch: None,
            last_dac_value: None,
        }
    }

    /// Deterministic harness constructor. The exact sound-board boot RAM
    /// contents are not modeled until `VSNDRM1.SRC` reset behavior is ported.
    pub fn with_cleared_ram(roms: &'a RedLabelRomImages) -> Self {
        Self::from_ram(roms, cleared_sound_cpu_ram())
    }

    pub fn ram(&self) -> &SoundCpuRam {
        &self.ram
    }

    pub fn last_command_latch(&self) -> Option<SoundCommandLatch> {
        self.last_command_latch
    }

    pub fn pia(&self) -> Pia6821 {
        self.pia
    }

    pub fn sound_pia_port_b_input(&self) -> u8 {
        self.sound_pia_port_b_input
    }

    pub fn last_dac_value(&self) -> Option<u8> {
        self.last_dac_value
    }

    pub fn sound_irq_asserted(&self) -> bool {
        self.pia.irq_a_asserted() || self.pia.irq_b_asserted()
    }

    pub fn latch_main_board_sound_command(&mut self, raw: u8) -> SoundCommandLatch {
        let latch = SoundCommandLatch::from_main_board_pia_port_b(raw);
        self.sound_pia_port_b_input = latch.port_b().raw();
        self.pia.cb1_w(latch.cb1_asserted());
        self.last_command_latch = Some(latch);
        latch
    }

    pub fn read_byte(&mut self, address: u16) -> Result<u8, SoundCpuReadError> {
        match sound_cpu_address_target(address) {
            SoundCpuAddressTarget::InternalRam { offset } => Ok(self.ram[usize::from(offset)]),
            SoundCpuAddressTarget::Pia { register } => Ok(self.pia.read(
                register,
                SOUND_PIA_UNCONNECTED_PORT_A_INPUT,
                self.sound_pia_port_b_input,
            )),
            SoundCpuAddressTarget::Rom { .. } => self
                .roms
                .sound_cpu_byte(address)
                .ok_or(SoundCpuReadError::UnmappedRom { address }),
            SoundCpuAddressTarget::Unmapped { address } => {
                Err(SoundCpuReadError::UnmappedAddress { address })
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) -> Result<(), SoundCpuWriteError> {
        match sound_cpu_address_target(address) {
            SoundCpuAddressTarget::InternalRam { offset } => {
                self.ram[usize::from(offset)] = value;
                Ok(())
            }
            SoundCpuAddressTarget::Pia { register } => {
                if let Some(PiaOutputEvent::PortA { data, .. }) = self.pia.write(
                    register,
                    value,
                    SOUND_PIA_UNCONNECTED_PORT_A_INPUT,
                    self.sound_pia_port_b_input,
                ) {
                    self.last_dac_value = Some(data);
                }
                Ok(())
            }
            SoundCpuAddressTarget::Rom { offset } => {
                Err(SoundCpuWriteError::ReadOnlyRom { offset })
            }
            SoundCpuAddressTarget::Unmapped { address } => {
                Err(SoundCpuWriteError::UnmappedAddress { address })
            }
        }
    }
}

pub fn cleared_sound_cpu_ram() -> SoundCpuRam {
    [0; SOUND_CPU_INTERNAL_RAM_SIZE]
}

pub fn sound_cpu_address_target(address: u16) -> SoundCpuAddressTarget {
    if (SOUND_CPU_INTERNAL_RAM_START..=SOUND_CPU_INTERNAL_RAM_END).contains(&address) {
        return SoundCpuAddressTarget::InternalRam {
            offset: (address - SOUND_CPU_INTERNAL_RAM_START) as u8,
        };
    }

    if let Some(register) = sound_pia_register(address) {
        return SoundCpuAddressTarget::Pia { register };
    }

    if (SOUND_CPU_ROM_START..=SOUND_CPU_ROM_END).contains(&address) {
        return SoundCpuAddressTarget::Rom {
            offset: address - SOUND_CPU_ROM_START,
        };
    }

    SoundCpuAddressTarget::Unmapped { address }
}

fn sound_pia_register(address: u16) -> Option<u8> {
    let canonical = address & !SOUND_CPU_PIA_MIRROR_MASK;
    if (SOUND_CPU_PIA_START..=SOUND_CPU_PIA_END).contains(&canonical) {
        Some((canonical - SOUND_CPU_PIA_START) as u8)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        pia::PIA_IRQ1,
        red_label_memory::{MemoryMapCpu, RedLabelMemoryMapEntry, red_label_memory_map},
        rom::{
            RedLabelRomImages, RomDescriptor, RomLoad, RomRegion, RomView, VerifiedRomFile,
            VerifiedRomSet,
        },
        sound::{
            DefenderSoundBoard, SOUND_CPU_INTERNAL_RAM_SIZE, SOUND_PIA_UNCONNECTED_PORT_A_INPUT,
            SoundCommand, SoundCommandLatch, SoundCpuAddressTarget, SoundCpuReadError,
            SoundCpuWriteError, format_sound_command_list, sound_cpu_address_target,
        },
    };

    fn test_rom_images() -> RedLabelRomImages {
        let mut sound = vec![0; 0x0800];
        sound[0] = 0xF8;
        sound[1] = 0x01;
        sound[0x07FF] = 0xFF;

        let rom_set = VerifiedRomSet::from_files_for_test(vec![VerifiedRomFile {
            descriptor: RomDescriptor {
                name: "sound.rom",
                size: sound.len() as u64,
                crc32: "00000000",
            },
            crc32: 0,
            bytes: sound,
        }]);
        let regions = [RomRegion {
            name: "soundcpu",
            size: 0x10000,
            source: "test",
        }];
        let loads = [RomLoad {
            name: "sound.rom",
            region: "soundcpu",
            region_offset: 0xF800,
            size: 0x0800,
            view: RomView::SoundCpu,
            cpu_start: Some(0xF800),
        }];

        RedLabelRomImages::from_parts_for_test(&rom_set, &regions, &loads)
            .expect("test sound ROM image should build")
    }

    fn sound_memory_entry(handler: &str) -> RedLabelMemoryMapEntry {
        red_label_memory_map()
            .expect("memory map parses")
            .into_iter()
            .find(|entry| entry.cpu == MemoryMapCpu::Sound && entry.handler == handler)
            .expect("sound memory map handler exists")
    }

    #[test]
    fn command_preserves_raw_sound_latch_byte() {
        let command = SoundCommand::new(0x8F);

        assert_eq!(command.raw(), 0x8F);
        assert_eq!(command.hex(), "0x8F");
    }

    #[test]
    fn command_from_main_board_pia_sets_mame_external_high_bits() {
        assert_eq!(SoundCommand::from_main_board_pia_port_b(0x00).raw(), 0xC0);
        assert_eq!(SoundCommand::from_main_board_pia_port_b(0x12).raw(), 0xD2);
        assert_eq!(SoundCommand::from_main_board_pia_port_b(0x3F).raw(), 0xFF);
        assert_eq!(SoundCommand::from_main_board_pia_port_b(0xFF).raw(), 0xFF);
    }

    #[test]
    fn command_latch_tracks_sound_pia_port_b_and_cb1_line() {
        let active = SoundCommandLatch::from_main_board_pia_port_b(0x01);
        assert_eq!(active.port_b().raw(), 0xC1);
        assert!(active.cb1_asserted());

        let idle_from_low_six_bits = SoundCommandLatch::from_main_board_pia_port_b(0x3F);
        assert_eq!(idle_from_low_six_bits.port_b().raw(), 0xFF);
        assert!(!idle_from_low_six_bits.cb1_asserted());

        let idle_from_all_bits = SoundCommandLatch::from_main_board_pia_port_b(0xFF);
        assert_eq!(idle_from_all_bits.port_b().raw(), 0xFF);
        assert!(!idle_from_all_bits.cb1_asserted());
    }

    #[test]
    fn command_list_uses_trace_friendly_hex_or_empty_marker() {
        assert_eq!(format_sound_command_list(&[]), "-");
        assert_eq!(
            format_sound_command_list(&[SoundCommand::new(0x01), SoundCommand::new(0xA0)]),
            "0x01,0xA0"
        );
    }

    #[test]
    fn sound_cpu_address_target_classifies_mame_defender_sound_map() {
        assert_eq!(
            sound_cpu_address_target(0x0000),
            SoundCpuAddressTarget::InternalRam { offset: 0 }
        );
        assert_eq!(
            sound_cpu_address_target(0x007F),
            SoundCpuAddressTarget::InternalRam { offset: 0x7F }
        );
        assert_eq!(
            sound_cpu_address_target(0x0080),
            SoundCpuAddressTarget::Unmapped { address: 0x0080 }
        );
        assert_eq!(
            sound_cpu_address_target(0x0400),
            SoundCpuAddressTarget::Pia { register: 0 }
        );
        assert_eq!(
            sound_cpu_address_target(0x0403),
            SoundCpuAddressTarget::Pia { register: 3 }
        );
        assert_eq!(
            sound_cpu_address_target(0x8400),
            SoundCpuAddressTarget::Pia { register: 0 }
        );
        assert_eq!(
            sound_cpu_address_target(0x8403),
            SoundCpuAddressTarget::Pia { register: 3 }
        );
        assert_eq!(
            sound_cpu_address_target(0x8404),
            SoundCpuAddressTarget::Unmapped { address: 0x8404 }
        );
        assert_eq!(
            sound_cpu_address_target(0xB000),
            SoundCpuAddressTarget::Rom { offset: 0 }
        );
        assert_eq!(
            sound_cpu_address_target(0xFFFF),
            SoundCpuAddressTarget::Rom { offset: 0x4FFF }
        );
    }

    #[test]
    fn sound_cpu_address_classifier_matches_embedded_memory_map_asset() {
        let ram = sound_memory_entry("internal_ram");
        assert_eq!(
            sound_cpu_address_target(ram.start),
            SoundCpuAddressTarget::InternalRam { offset: 0 }
        );
        assert_eq!(
            sound_cpu_address_target(ram.end),
            SoundCpuAddressTarget::InternalRam {
                offset: (ram.end - ram.start) as u8,
            }
        );

        let pia = sound_memory_entry("pia_ic4");
        assert_eq!(
            sound_cpu_address_target(pia.start),
            SoundCpuAddressTarget::Pia { register: 0 }
        );
        assert_eq!(
            sound_cpu_address_target(pia.start | pia.mirror_mask.expect("PIA mirror") | 0x03),
            SoundCpuAddressTarget::Pia { register: 3 }
        );

        let rom = sound_memory_entry("rom");
        assert_eq!(
            sound_cpu_address_target(rom.start),
            SoundCpuAddressTarget::Rom { offset: 0 }
        );
        assert_eq!(
            sound_cpu_address_target(rom.end),
            SoundCpuAddressTarget::Rom {
                offset: rom.end - rom.start,
            }
        );
    }

    #[test]
    fn sound_board_reads_and_writes_internal_ram() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(board.ram().len(), SOUND_CPU_INTERNAL_RAM_SIZE);
        assert_eq!(board.read_byte(0x0000), Ok(0));
        assert_eq!(board.read_byte(0x007F), Ok(0));

        board.write_byte(0x0000, 0x12).expect("write low RAM");
        board.write_byte(0x007F, 0xFE).expect("write high RAM");

        assert_eq!(board.read_byte(0x0000), Ok(0x12));
        assert_eq!(board.read_byte(0x007F), Ok(0xFE));
        assert_eq!(board.ram()[0], 0x12);
        assert_eq!(board.ram()[SOUND_CPU_INTERNAL_RAM_SIZE - 1], 0xFE);
    }

    #[test]
    fn sound_board_keeps_last_main_board_command_latch_without_running_audio() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(board.last_command_latch(), None);

        let active = board.latch_main_board_sound_command(0x22);
        assert_eq!(active.port_b().raw(), 0xE2);
        assert!(active.cb1_asserted());
        assert_eq!(board.last_command_latch(), Some(active));
        assert_eq!(board.sound_pia_port_b_input(), 0xE2);

        let idle = board.latch_main_board_sound_command(0x3F);
        assert_eq!(idle.port_b().raw(), 0xFF);
        assert!(!idle.cb1_asserted());
        assert_eq!(board.last_command_latch(), Some(idle));
        assert_eq!(board.sound_pia_port_b_input(), 0xFF);
    }

    #[test]
    fn sound_board_reads_latched_command_through_sound_pia_port_b() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(board.read_byte(0x0402), Ok(0x00));

        board.latch_main_board_sound_command(0x12);
        assert_eq!(board.sound_pia_port_b_input(), 0xD2);
        assert_eq!(board.read_byte(0x0402), Ok(0x00));

        board
            .write_byte(0x0403, 0x04)
            .expect("select sound PIA port B data");
        assert_eq!(board.read_byte(0x0402), Ok(0xD2));

        board
            .write_byte(0x0403, 0x00)
            .expect("select sound PIA DDRB");
        board
            .write_byte(0x0402, 0xF0)
            .expect("write sound PIA DDRB");
        board
            .write_byte(0x0403, 0x04)
            .expect("select sound PIA port B data again");

        assert_eq!(board.read_byte(0x0402), Ok(0x02));
        assert_eq!(board.pia().ddr_b(), 0xF0);
    }

    #[test]
    fn sound_board_command_cb1_sets_and_clears_sound_pia_irq() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        board
            .write_byte(0x0403, 0x03)
            .expect("enable sound PIA CB1 low-to-high IRQ");
        board.latch_main_board_sound_command(0x12);

        assert!(board.sound_irq_asserted());
        assert!(board.pia().irq_b_asserted());
        assert_eq!(board.read_byte(0x0403), Ok(PIA_IRQ1 | 0x03));

        board
            .write_byte(0x0403, 0x07)
            .expect("select sound PIA port B data");
        assert_eq!(board.read_byte(0x0402), Ok(0xD2));
        assert!(!board.sound_irq_asserted());

        board.latch_main_board_sound_command(0x3F);
        assert!(!board.pia().in_cb1());
        assert!(!board.sound_irq_asserted());
    }

    #[test]
    fn sound_board_pia_port_a_output_tracks_dac_callback_boundary() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(board.last_dac_value(), None);
        assert_eq!(board.read_byte(0x0400), Ok(0x00));
        assert_eq!(
            board.pia().read(0, SOUND_PIA_UNCONNECTED_PORT_A_INPUT, 0),
            0x00
        );

        board
            .write_byte(0x0400, 0xFF)
            .expect("default sound PIA port A write selects DDRA");
        assert_eq!(board.last_dac_value(), Some(0x00));
        assert_eq!(board.pia().ddr_a(), 0xFF);

        board
            .write_byte(0x0401, 0x04)
            .expect("select sound PIA port A data");
        board
            .write_byte(0x0400, 0xA5)
            .expect("write DAC output byte");
        assert_eq!(board.last_dac_value(), Some(0xA5));
        assert_eq!(board.read_byte(0x0400), Ok(0xA5));
    }

    #[test]
    fn sound_board_reads_loaded_sound_rom_without_filling_unknown_rom_bytes() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(board.read_byte(0xF800), Ok(0xF8));
        assert_eq!(board.read_byte(0xF801), Ok(0x01));
        assert_eq!(board.read_byte(0xFFFF), Ok(0xFF));
        assert_eq!(
            board.read_byte(0xB000),
            Err(SoundCpuReadError::UnmappedRom { address: 0xB000 })
        );
    }

    #[test]
    fn sound_board_reports_read_only_rom_and_unmapped_writes() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(board.read_byte(0x0400), Ok(0x00));
        board
            .write_byte(0x8403, 0x80)
            .expect("mirrored PIA control writes are handled");
        assert_eq!(
            board.write_byte(0xF800, 0x99),
            Err(SoundCpuWriteError::ReadOnlyRom { offset: 0x4800 })
        );
        assert_eq!(
            board.read_byte(0x0100),
            Err(SoundCpuReadError::UnmappedAddress { address: 0x0100 })
        );
        assert_eq!(
            board.write_byte(0x0100, 0x99),
            Err(SoundCpuWriteError::UnmappedAddress { address: 0x0100 })
        );
    }
}
