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
//! CB1 IRQ input, the DAC output callback boundary, source-shaped IRQ
//! GWAVE/VARI command flow, and source-shaped GWAVE period, VARI sweep, LITEN
//! random-complement, TURBO noise-decay, BG1 / THRUST / CANNON filtered-noise,
//! RADIO timer-table, HYPER phase-edge, SCREAM echo-cascade, and ORGAN
//! tune/note byte extraction. It exposes source-visible IRQ DAC write order and
//! ticks; it does not emulate 6808-cycle-accurate sample spacing or independent
//! sound CPU IRQ scheduling yet.

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
pub const VSNDRM1_STACK_TOP: u8 = 0x7F;
/// One source-visible DAC write in the translated sound-board timing model.
/// This is deliberately not a 6808 CPU-cycle duration.
pub const VSOUND_SOURCE_DAC_TICK_STEP: u64 = 1;
pub const VSNDRM1_SPINNER_SOUND_CODE: u8 = 0x0E;
pub const VSNDRM1_BONUS2_SOUND_CODE: u8 = 0x12;
pub const VSNDRM1_BG2_MAX: u8 = 29;
pub const VSNDRM1_SPINNER_MAX: u8 = 32;
pub const VSNDRM1_GWAVE_VECTOR_COUNT: u8 = 15;
pub const VSNDRM1_VARI_VECTOR_COUNT: u8 = 4;
const VSNDRM1_TALK_ENTRY_ADDRESS: u16 = 0xEFFD;
const VSNDRM1_TALKD_ENTRY_ADDRESS: u16 = 0xEFFA;
const VSNDRM1_NMI_CHECKSUM_ADDRESS: u16 = 0xF800;
const VSNDRM1_NMI_FIRST_SUMMED_ADDRESS: u16 = 0xF801;
const VSNDRM1_NMI_LAST_SUMMED_ADDRESS: u16 = 0xFFFF;
const VSNDRM1_NMI_VARI_VECTOR_INDEX: u8 = 1;
const VSNDRM1_BG1_ENTRY_ADDRESS: u16 = 0xF913;
const VSNDRM1_THRUST_ENTRY_ADDRESS: u16 = 0xF91C;
const VSNDRM1_VVECT_ADDRESS: u16 = 0xFD76;
const VSNDRM1_RADSND_ADDRESS: u16 = 0xFD9A;
const VSNDRM1_ORGTAB_ADDRESS: u16 = 0xFDAA;
const VSNDRM1_ORGAN1_ADDRESS: u16 = 0xFADD;
const VSNDRM1_NOTTAB_ADDRESS: u16 = 0xFE41;
const VSNDRM1_GWVTAB_ADDRESS: u16 = 0xFE4D;
const VSNDRM1_SVTAB_ADDRESS: u16 = 0xFEEC;
const VSNDRM1_GFRTAB_ADDRESS: u16 = 0xFF55;
const VSNDRM1_WVELEN: u8 = 72;
const VSNDRM1_ORGAN_RDELAY_LEN: usize = 60;
const VSNDRM1_SPINNER_VARI_VECTOR_INDEX: u8 = 3;
const VSNDRM1_BONUS2_GWAVE_VECTOR_INDEX: u8 = 13;
const VSNDRM1_BACKGROUND2_GWAVE_VECTOR_INDEX: u8 = 14;
const VSNDRM1_BG1FLG_OFFSET: usize = 0x04;
const VSNDRM1_BG2FLG_OFFSET: usize = 0x05;
const VSNDRM1_SP1FLG_OFFSET: usize = 0x06;
const VSNDRM1_B2FLG_OFFSET: usize = 0x07;
const VSNDRM1_ORGFLG_OFFSET: usize = 0x08;
const VSNDRM1_HI_OFFSET: usize = 0x09;
const VSNDRM1_LO_OFFSET: usize = 0x0A;
const VSNDRM1_TEMPX_OFFSET: usize = 0x0B;
const VSNDRM1_XPLAY_OFFSET: usize = 0x0D;
const VSNDRM1_XPTR_OFFSET: usize = 0x0F;
const VSNDRM1_TEMPA_OFFSET: usize = 0x11;
const VSNDRM1_TEMPB_OFFSET: usize = 0x12;
const VSNDRM1_LOPER_OFFSET: usize = 0x13;
const VSNDRM1_HIPER_OFFSET: usize = 0x14;
const VSNDRM1_LODT_OFFSET: usize = 0x15;
const VSNDRM1_HIDT_OFFSET: usize = 0x16;
const VSNDRM1_HIEN_OFFSET: usize = 0x17;
const VSNDRM1_SWPDT_OFFSET: usize = 0x18;
const VSNDRM1_LOMOD_OFFSET: usize = 0x1A;
const VSNDRM1_VAMP_OFFSET: usize = 0x1B;
const VSNDRM1_LOCNT_OFFSET: usize = 0x1C;
const VSNDRM1_HICNT_OFFSET: usize = 0x1D;
const VSNDRM1_GECHO_OFFSET: usize = 0x13;
const VSNDRM1_GCCNT_OFFSET: usize = 0x14;
const VSNDRM1_GECDEC_OFFSET: usize = 0x15;
const VSNDRM1_GDFINC_OFFSET: usize = 0x16;
const VSNDRM1_GDCNT_OFFSET: usize = 0x17;
const VSNDRM1_GWFRM_OFFSET: usize = 0x18;
const VSNDRM1_PRDECA_OFFSET: usize = 0x1A;
const VSNDRM1_GWFRQ_OFFSET: usize = 0x1B;
const VSNDRM1_FRQEND_OFFSET: usize = 0x1D;
const VSNDRM1_WVEND_OFFSET: usize = 0x1F;
const VSNDRM1_GPER_OFFSET: usize = 0x21;
const VSNDRM1_GECNT_OFFSET: usize = 0x22;
const VSNDRM1_FOFSET_OFFSET: usize = 0x23;
const VSNDRM1_GWTAB_OFFSET: usize = 0x24;
const VSNDRM1_DECAY_OFFSET: usize = 0x13;
const VSNDRM1_NAMP_OFFSET: usize = 0x14;
const VSNDRM1_CYCNT_OFFSET: usize = 0x15;
const VSNDRM1_NFRQ1_OFFSET: usize = 0x16;
const VSNDRM1_NFFLG_OFFSET: usize = 0x18;
const VSNDRM1_LFREQ_OFFSET: usize = 0x19;
const VSNDRM1_DFREQ_OFFSET: usize = 0x1A;
const VSNDRM1_FMAX_OFFSET: usize = 0x13;
const VSNDRM1_FHI_OFFSET: usize = 0x14;
const VSNDRM1_FLO_OFFSET: usize = 0x15;
const VSNDRM1_SAMPC_OFFSET: usize = 0x16;
const VSNDRM1_FDFLG_OFFSET: usize = 0x18;
const VSNDRM1_DSFLG_OFFSET: usize = 0x19;
const VSNDRM1_SCREAM_ECHO_COUNT: usize = 4;
const VSNDRM1_STABLE_OFFSET: usize = 0x13;
const VSNDRM1_SRMEND_OFFSET: usize = VSNDRM1_STABLE_OFFSET + 2 * VSNDRM1_SCREAM_ECHO_COUNT;
const VSNDRM1_SCREAM_INITIAL_FREQUENCY: u8 = 0x40;
const VSNDRM1_SCREAM_NEXT_ECHO_FREQUENCY: u8 = 0x41;
const VSNDRM1_SCREAM_SPAWN_FREQUENCY: u8 = 0x37;
const VSNDRM1_DUR_OFFSET: usize = 0x13;
const VSNDRM1_OSCIL_OFFSET: usize = 0x15;
const VSNDRM1_RDELAY_OFFSET: usize = 0x16;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundSetup {
    pub stack_top: u8,
    pub pia_ddr_a: u8,
    pub pia_ddr_b: u8,
    pub pia_control_a: u8,
    pub pia_control_b: u8,
    pub random_seed_hi: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundIrqPrelude {
    pub stack_top: u8,
    pub latched_port_b: u8,
    pub command_code: u8,
    pub irq_asserted_before_read: bool,
    pub irq_asserted_after_read: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundOrganIrqStep {
    Inactive,
    Note,
    TuneAndNote,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundIrqOrganFlow {
    Inactive {
        latched_port_b: u8,
        command_code: u8,
    },
    Tune {
        latched_port_b: u8,
        command_code: u8,
        tune: VSoundOrganTuneStep,
    },
    Note {
        latched_port_b: u8,
        command_code: u8,
        parameter_code: u8,
        step: VSoundOrganNoteStep,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundIrqFlow {
    Command(VSoundIrqCommandAndBackgroundFlow),
    OrganTune {
        organ: VSoundIrqOrganFlow,
        irq3: VSoundIrq3AfterCommand,
    },
    OrganNote {
        organ: VSoundIrqOrganFlow,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundIrqCycle {
    pub prelude: VSoundIrqPrelude,
    pub flow: VSoundIrqFlow,
}

impl VSoundIrqCycle {
    pub fn dac_samples(&self) -> Vec<u8> {
        self.flow.dac_samples()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundDacSampleWindow {
    pub first_tick: Option<u64>,
    pub tick_step: u64,
    pub dac_samples: Vec<u8>,
    pub last_dac_value: Option<u8>,
}

impl VSoundDacSampleWindow {
    pub fn sample_count(&self) -> usize {
        self.dac_samples.len()
    }

    pub fn sample_tick(&self, sample_index: usize) -> Option<u64> {
        if sample_index >= self.dac_samples.len() {
            return None;
        }

        self.first_tick.map(|first_tick| {
            first_tick
                + u64::try_from(sample_index).expect("sample index fits in u64") * self.tick_step
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundIrqTimedCycle {
    pub irq_tick: u64,
    pub cycle: VSoundIrqCycle,
    pub dac: VSoundDacSampleWindow,
    pub irq_asserted_after_cycle: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundNmiChecksum {
    pub stack_top: u8,
    pub checksum_address: u16,
    pub first_summed_address: u16,
    pub last_summed_address: u16,
    pub computed_checksum: u8,
    pub expected_checksum: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundNmiDiagnosticCycle {
    ChecksumMismatch(VSoundNmiChecksum),
    ChecksumMatched {
        checksum: VSoundNmiChecksum,
        vari_load: VSoundVariLoad,
        sweep: VSoundVariSweep,
        talking_diagnostic_present: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundNmiDiagnosticError {
    MissingSoundRomByte { address: u16 },
    Vari(VSoundVariLoadError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundSpecialRoutine {
    Spinner1,
    Background1,
    Background2Increment,
    Lite,
    Bonus2,
    BackgroundEnd,
    Turbo,
    Appear,
    Thrust,
    Cannon,
    Radio,
    Hyper,
    Scream,
    OrganTune,
    OrganNote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundIrqRoutine {
    Invalid,
    GWave {
        sound_code: u8,
        table_index: u8,
    },
    Special {
        sound_code: u8,
        table_index: u8,
        routine: VSoundSpecialRoutine,
    },
    Vari {
        sound_code: u8,
        table_index: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundBackgroundContinuation {
    WaitingForBackground,
    Background1,
    Background2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundIrq3BackgroundFlow {
    WaitingForBackground,
    Background1(VSoundFilteredNoiseWindow),
    Background2(VSoundBackground2Setup),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundIrq3Handoff {
    Ready,
    Running(VSoundIrqRunningRoutine),
    Deferred { routine: VSoundSpecialRoutine },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundIrqRunningRoutine {
    GWave,
    Vari,
    Spinner1,
    Background1,
    Background2,
    Bonus2,
    Thrust,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundBackgroundFlags {
    pub background1_flag: u8,
    pub background2_flag: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundIrqDispatch {
    pub latched_port_b: u8,
    pub command_code: u8,
    pub effective_code: u8,
    pub organ_step: VSoundOrganIrqStep,
    pub talking_program_present: bool,
    pub routine: VSoundIrqRoutine,
    pub spinner_flag: u8,
    pub bonus2_flag: u8,
    pub background1_flag: u8,
    pub background2_flag: u8,
    pub background: VSoundBackgroundContinuation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundIrqCommandFlow {
    Invalid {
        dispatch: VSoundIrqDispatch,
    },
    GWave {
        dispatch: VSoundIrqDispatch,
        load: VSoundGWaveLoad,
        step: VSoundGWaveStep,
    },
    Special {
        dispatch: VSoundIrqDispatch,
        step: VSoundIrqSpecialFlow,
    },
    Vari {
        dispatch: VSoundIrqDispatch,
        load: VSoundVariLoad,
        sweep: VSoundVariSweep,
    },
}

impl VSoundIrqCommandFlow {
    /// Source-shaped return classification for deciding when it is legal to
    /// enter `IRQ3`. A `Ready` result means the translated slice has reached
    /// the source branch that falls into `IRQ3`; `Running` means the source
    /// routine is still looping or has jumped back into playback.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L939-L967>.
    pub fn irq3_handoff(&self) -> VSoundIrq3Handoff {
        match self {
            VSoundIrqCommandFlow::Invalid { .. } => VSoundIrq3Handoff::Ready,
            VSoundIrqCommandFlow::GWave { step, .. } => step.irq3_handoff(),
            VSoundIrqCommandFlow::Special { step, .. } => step.irq3_handoff(),
            VSoundIrqCommandFlow::Vari { sweep, .. } => sweep.result.irq3_handoff(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundIrq3AfterCommand {
    Entered(VSoundIrq3BackgroundFlow),
    Skipped(VSoundIrq3Handoff),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundIrqCommandAndBackgroundFlow {
    pub command: VSoundIrqCommandFlow,
    pub irq3: VSoundIrq3AfterCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundIrqSpecialFlow {
    Deferred { routine: VSoundSpecialRoutine },
    Spinner1(VSoundSpinner1Command),
    Background1(VSoundFilteredNoiseWindow),
    Background2Increment(VSoundBackground2Setup),
    Lite(VSoundLiteNoise),
    Bonus2(VSoundBonus2Command),
    BackgroundEnd(VSoundBackgroundFlags),
    Turbo(VSoundTurboNoise),
    Appear(VSoundLiteNoise),
    Thrust(VSoundFilteredNoiseWindow),
    Cannon(VSoundFilteredNoise),
    Radio(VSoundRadioWave),
    Hyper(VSoundHyperSweep),
    Scream(VSoundScream),
    OrganTune(VSoundOrganTuneStart),
    OrganNote(VSoundOrganNoteStart),
}

impl VSoundIrqSpecialFlow {
    /// Source-shaped return classification for `JMPTBL` special routines.
    /// Continuous branches such as `SP1`, `BG1`, `BG2`, and `THRUST` must not
    /// enter the separate `IRQ3` handoff until a later interrupt preempts them.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L1003-L1006>.
    pub fn irq3_handoff(&self) -> VSoundIrq3Handoff {
        match self {
            VSoundIrqSpecialFlow::Deferred { routine } => {
                VSoundIrq3Handoff::Deferred { routine: *routine }
            }
            VSoundIrqSpecialFlow::Spinner1(_) => {
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Spinner1)
            }
            VSoundIrqSpecialFlow::Background1(_) => {
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Background1)
            }
            VSoundIrqSpecialFlow::Background2Increment(_) => {
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Background2)
            }
            VSoundIrqSpecialFlow::Bonus2(command) => command.irq3_handoff(),
            VSoundIrqSpecialFlow::Thrust(_) => {
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Thrust)
            }
            VSoundIrqSpecialFlow::Lite(_)
            | VSoundIrqSpecialFlow::BackgroundEnd(_)
            | VSoundIrqSpecialFlow::Turbo(_)
            | VSoundIrqSpecialFlow::Appear(_)
            | VSoundIrqSpecialFlow::Cannon(_)
            | VSoundIrqSpecialFlow::Radio(_)
            | VSoundIrqSpecialFlow::Hyper(_)
            | VSoundIrqSpecialFlow::Scream(_)
            | VSoundIrqSpecialFlow::OrganTune(_)
            | VSoundIrqSpecialFlow::OrganNote(_) => VSoundIrq3Handoff::Ready,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundIrqCommandFlowError {
    GWave(VSoundGWaveLoadError),
    Spinner1(VSoundVariLoadError),
    Background2(VSoundGWaveLoadError),
    Bonus2(VSoundGWaveLoadError),
    Radio(VSoundGWaveLoadError),
    Vari(VSoundVariLoadError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundIrqCommandAndBackgroundFlowError {
    Command(VSoundIrqCommandFlowError),
    Background(VSoundGWaveLoadError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundIrqFlowError {
    Organ(VSoundOrganLoadError),
    Command(VSoundIrqCommandAndBackgroundFlowError),
    Background(VSoundGWaveLoadError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundGWaveLoad {
    pub table_index: u8,
    pub vector_address: u16,
    pub echo_count: u8,
    pub cycle_count: u8,
    pub echo_decay: u8,
    pub waveform_index: u8,
    pub waveform_address: u16,
    pub waveform_length: u8,
    pub predecay_factor: u8,
    pub frequency_delta: u8,
    pub frequency_delta_count: u8,
    pub frequency_pattern_address: u16,
    pub frequency_pattern_length: u8,
    pub frequency_end_address: u16,
    pub wave_ram_start: u16,
    pub wave_ram_end: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundGWaveLoadError {
    InvalidSoundCode { sound_code: u8 },
    InvalidVectorIndex { table_index: u8 },
    MissingSoundRomByte { address: u16 },
    InvalidWaveRamEnd { wave_ram_end: u16 },
    WaveformTooLong { address: u16, length: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundVariLoad {
    pub table_index: u8,
    pub vector_address: u16,
    pub low_period: u8,
    pub high_period: u8,
    pub low_period_delta: u8,
    pub high_period_delta: u8,
    pub high_period_end: u8,
    pub sweep_period: u16,
    pub low_period_mod: u8,
    pub amplitude: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundVariLoadError {
    InvalidSoundCode { sound_code: u8 },
    InvalidVectorIndex { table_index: u8 },
    MissingSoundRomByte { address: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundSpinner1Setup {
    pub vari_load: VSoundVariLoad,
    pub spinner_flag: u8,
    pub low_period: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundSpinner1Command {
    pub setup: VSoundSpinner1Setup,
    pub sweep: VSoundVariSweep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundBonus2Setup {
    Started {
        gwave_load: VSoundGWaveLoad,
        bonus2_flag: u8,
    },
    Continued {
        bonus2_flag: u8,
        gend50_step: VSoundGEnd50Step,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundBonus2Command {
    Started {
        gwave_load: VSoundGWaveLoad,
        bonus2_flag: u8,
        step: VSoundGWaveStep,
    },
    Continued {
        bonus2_flag: u8,
        gend50_step: VSoundGEnd50Step,
    },
}

impl VSoundBonus2Command {
    /// Source-shaped `BON2` return classification. `BON2` starts in `GWAVE` or
    /// jumps into `GEND50`; restarted playback keeps the bonus sound running,
    /// while a terminating `GEND50` / `GWAVE` result returns to `IRQ3`.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L719-L727>.
    pub fn irq3_handoff(&self) -> VSoundIrq3Handoff {
        let handoff = match self {
            VSoundBonus2Command::Started { step, .. } => step.irq3_handoff(),
            VSoundBonus2Command::Continued { gend50_step, .. } => gend50_step.irq3_handoff(),
        };

        match handoff {
            VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::GWave) => {
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Bonus2)
            }
            other => other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundGEnd50Step {
    NoFrequencyDelta { frequency_delta: u8 },
    DeltaCountExpired { frequency_delta_count: u8 },
    Updated(VSoundGEndUpdate),
}

impl VSoundGEnd50Step {
    /// Source-shaped `GEND50` / `GEND61` return classification.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L831-L867>.
    pub fn irq3_handoff(self) -> VSoundIrq3Handoff {
        match self {
            VSoundGEnd50Step::NoFrequencyDelta { .. }
            | VSoundGEnd50Step::DeltaCountExpired { .. } => VSoundIrq3Handoff::Ready,
            VSoundGEnd50Step::Updated(update) => match update.result {
                VSoundGEnd61Result::RestartGWave { .. } => {
                    VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::GWave)
                }
                VSoundGEnd61Result::AllOver => VSoundIrq3Handoff::Ready,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundGEnd61Result {
    RestartGWave { waveform_reloaded: bool },
    AllOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundGEndUpdate {
    pub frequency_offset: u8,
    pub frequency_pattern_address: u16,
    pub frequency_end_address: u16,
    pub result: VSoundGEnd61Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundBackground2Setup {
    pub gwave_load: VSoundGWaveLoad,
    pub background2_flag: u8,
    pub frequency_update: VSoundGEndUpdate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundGEndStep {
    EchoRestart { echo_count: u8 },
    BonusStopped { bonus2_flag: u8 },
    Frequency(VSoundGEnd50Step),
}

impl VSoundGEndStep {
    /// Source-shaped `GEND` return classification.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L825-L868>.
    pub fn irq3_handoff(self) -> VSoundIrq3Handoff {
        match self {
            VSoundGEndStep::EchoRestart { .. } => {
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::GWave)
            }
            VSoundGEndStep::BonusStopped { .. } => VSoundIrq3Handoff::Ready,
            VSoundGEndStep::Frequency(step) => step.irq3_handoff(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundGWavePeriod {
    pub frequency_address: u16,
    pub next_frequency_address: u16,
    pub period: u8,
    pub cycle_count: u8,
    pub waveform_cycles: u16,
    pub wave_ram_start: u16,
    pub wave_ram_end: u16,
    pub dac_samples: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundGWaveStep {
    Playing(VSoundGWavePeriod),
    Ended(VSoundGEndStep),
}

impl VSoundGWaveStep {
    /// Source-shaped `GWAVE` / `GPLAY` return classification.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L790-L868>.
    pub fn irq3_handoff(&self) -> VSoundIrq3Handoff {
        match self {
            VSoundGWaveStep::Playing(_) => {
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::GWave)
            }
            VSoundGWaveStep::Ended(step) => step.irq3_handoff(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundVariSweepResult {
    Continue,
    Restarted { low_period: u8 },
    Terminated { low_period: u8, low_period_mod: u8 },
}

impl VSoundVariSweepResult {
    /// Source-shaped `VARI` / `VSWEEP` return classification.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L213-L251>.
    pub fn irq3_handoff(self) -> VSoundIrq3Handoff {
        match self {
            VSoundVariSweepResult::Continue | VSoundVariSweepResult::Restarted { .. } => {
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Vari)
            }
            VSoundVariSweepResult::Terminated { .. } => VSoundIrq3Handoff::Ready,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundVariSweep {
    pub sweep_period: u16,
    pub low_count: u8,
    pub high_count: u8,
    pub low_count_after_sweep: u8,
    pub high_count_after_sweep: u8,
    pub dac_samples: Vec<u8>,
    pub result: VSoundVariSweepResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundLiteNoise {
    pub initial_frequency: u8,
    pub final_frequency: u8,
    pub frequency_delta: u8,
    pub cycle_count: u8,
    pub frequency_passes: u16,
    pub random_steps: u16,
    pub random_seed_hi: u8,
    pub random_seed_lo: u8,
    pub dac_samples: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundTurboNoise {
    pub initial_period: u16,
    pub final_period: u16,
    pub initial_amplitude: u8,
    pub final_amplitude: u8,
    pub decay: u8,
    pub cycle_count: u8,
    pub amplitude_passes: u16,
    pub random_steps: u16,
    pub random_seed_hi: u8,
    pub random_seed_lo: u8,
    pub dac_samples: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundRadioWave {
    pub table_address: u16,
    pub table: [u8; 16],
    pub initial_frequency: u16,
    pub final_frequency: u16,
    pub initial_timer_high: u8,
    pub final_timer_high: u8,
    pub final_timer_low: u8,
    pub successful_frequency_increments: u16,
    pub dac_samples: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundFilteredNoise {
    pub initial_sample_count: u16,
    pub initial_max_frequency: u8,
    pub final_max_frequency: u8,
    pub final_frequency_high: u8,
    pub final_frequency_low: u8,
    pub frequency_decay_enabled: bool,
    pub distortion_enabled: bool,
    pub decay_passes: u16,
    pub random_steps: u16,
    pub random_seed_hi: u8,
    pub random_seed_lo: u8,
    pub dac_samples: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundFilteredNoiseWindow {
    pub entry_address: u16,
    pub initial_sample_count: u16,
    pub initial_max_frequency: u8,
    pub final_frequency_high: u8,
    pub final_frequency_low: u8,
    pub frequency_decay_enabled: bool,
    pub distortion_enabled: bool,
    pub background1_flag: u8,
    pub continues_after_window: bool,
    pub random_steps: u16,
    pub random_seed_hi: u8,
    pub random_seed_lo: u8,
    pub dac_samples: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundHyperSweep {
    pub phase_count: u16,
    pub final_phase: u8,
    pub dac_samples: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundScream {
    pub echo_count: u8,
    pub initial_frequency: u8,
    pub next_echo_frequency: u8,
    pub spawn_frequency: u8,
    pub initial_timer: u8,
    pub final_timer: u8,
    pub decay_passes: u8,
    pub echo_starts: u8,
    pub final_echo_table: [u8; 2 * VSNDRM1_SCREAM_ECHO_COUNT],
    pub srmend_byte: u8,
    pub dac_samples: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundOrganNoteStart {
    pub organ_flag: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundOrganTuneStart {
    pub organ_flag: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VSoundOrganMaskByte {
    pub parameter_code: u8,
    pub organ_flag: u8,
    pub oscillator_mask: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundOrganDelayPatch {
    pub requested_delay: u8,
    pub rdelay_start: u16,
    pub rdelay_end: u16,
    pub nop_count: u8,
    pub cmp_zero_patch: bool,
    pub jump_address: u16,
    pub patch_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundOrganWindow {
    pub oscillator_mask: u8,
    pub duration: u16,
    pub initial_timer: u8,
    pub final_timer: u8,
    pub delay_patch: VSoundOrganDelayPatch,
    pub dac_samples: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundOrganTuneNote {
    pub note_address: u16,
    pub oscillator_mask: u8,
    pub note_delay: u8,
    pub duration: u16,
    pub window: VSoundOrganWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundOrganTune {
    pub tune_number: u8,
    pub table_address: u16,
    pub tune_length: u8,
    pub tune_start_address: u16,
    pub tune_end_address: u16,
    pub notes: Vec<VSoundOrganTuneNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundOrganTuneStep {
    Played(VSoundOrganTune),
    InvalidTune { tune_number: u8, table_address: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSoundOrganNoteWindow {
    pub note_parameter: u8,
    pub note_delay: u8,
    pub organ_flag: u8,
    pub window: VSoundOrganWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VSoundOrganNoteStep {
    MaskByte(VSoundOrganMaskByte),
    NoteStarted(VSoundOrganNoteWindow),
}

impl VSoundIrqFlow {
    pub fn dac_samples(&self) -> Vec<u8> {
        let mut samples = Vec::new();
        self.append_dac_samples(&mut samples);
        samples
    }

    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        match self {
            VSoundIrqFlow::Command(flow) => flow.append_dac_samples(samples),
            VSoundIrqFlow::OrganTune { organ, irq3 } => {
                organ.append_dac_samples(samples);
                irq3.append_dac_samples(samples);
            }
            VSoundIrqFlow::OrganNote { organ } => organ.append_dac_samples(samples),
        }
    }
}

impl VSoundIrqOrganFlow {
    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        match self {
            VSoundIrqOrganFlow::Inactive { .. } => {}
            VSoundIrqOrganFlow::Tune { tune, .. } => tune.append_dac_samples(samples),
            VSoundIrqOrganFlow::Note { step, .. } => step.append_dac_samples(samples),
        }
    }
}

impl VSoundIrqCommandAndBackgroundFlow {
    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        self.command.append_dac_samples(samples);
        self.irq3.append_dac_samples(samples);
    }
}

impl VSoundIrqCommandFlow {
    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        match self {
            VSoundIrqCommandFlow::Invalid { .. } => {}
            VSoundIrqCommandFlow::GWave { step, .. } => step.append_dac_samples(samples),
            VSoundIrqCommandFlow::Special { step, .. } => step.append_dac_samples(samples),
            VSoundIrqCommandFlow::Vari { sweep, .. } => {
                samples.extend_from_slice(&sweep.dac_samples);
            }
        }
    }
}

impl VSoundIrq3AfterCommand {
    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        match self {
            VSoundIrq3AfterCommand::Entered(flow) => flow.append_dac_samples(samples),
            VSoundIrq3AfterCommand::Skipped(_) => {}
        }
    }
}

impl VSoundIrq3BackgroundFlow {
    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        match self {
            VSoundIrq3BackgroundFlow::WaitingForBackground
            | VSoundIrq3BackgroundFlow::Background2(_) => {}
            VSoundIrq3BackgroundFlow::Background1(window) => {
                samples.extend_from_slice(&window.dac_samples);
            }
        }
    }
}

impl VSoundIrqSpecialFlow {
    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        match self {
            VSoundIrqSpecialFlow::Deferred { .. }
            | VSoundIrqSpecialFlow::Background2Increment(_)
            | VSoundIrqSpecialFlow::BackgroundEnd(_)
            | VSoundIrqSpecialFlow::OrganTune(_)
            | VSoundIrqSpecialFlow::OrganNote(_) => {}
            VSoundIrqSpecialFlow::Spinner1(command) => {
                samples.extend_from_slice(&command.sweep.dac_samples);
            }
            VSoundIrqSpecialFlow::Background1(window) | VSoundIrqSpecialFlow::Thrust(window) => {
                samples.extend_from_slice(&window.dac_samples);
            }
            VSoundIrqSpecialFlow::Lite(noise) | VSoundIrqSpecialFlow::Appear(noise) => {
                samples.extend_from_slice(&noise.dac_samples);
            }
            VSoundIrqSpecialFlow::Bonus2(command) => command.append_dac_samples(samples),
            VSoundIrqSpecialFlow::Turbo(noise) => samples.extend_from_slice(&noise.dac_samples),
            VSoundIrqSpecialFlow::Cannon(noise) => samples.extend_from_slice(&noise.dac_samples),
            VSoundIrqSpecialFlow::Radio(wave) => samples.extend_from_slice(&wave.dac_samples),
            VSoundIrqSpecialFlow::Hyper(sweep) => samples.extend_from_slice(&sweep.dac_samples),
            VSoundIrqSpecialFlow::Scream(scream) => {
                samples.extend_from_slice(&scream.dac_samples);
            }
        }
    }
}

impl VSoundBonus2Command {
    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        match self {
            VSoundBonus2Command::Started { step, .. } => step.append_dac_samples(samples),
            VSoundBonus2Command::Continued { .. } => {}
        }
    }
}

impl VSoundGWaveStep {
    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        match self {
            VSoundGWaveStep::Playing(period) => samples.extend_from_slice(&period.dac_samples),
            VSoundGWaveStep::Ended(_) => {}
        }
    }
}

impl VSoundOrganTuneStep {
    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        match self {
            VSoundOrganTuneStep::Played(tune) => {
                for note in &tune.notes {
                    samples.extend_from_slice(&note.window.dac_samples);
                }
            }
            VSoundOrganTuneStep::InvalidTune { .. } => {}
        }
    }
}

impl VSoundOrganNoteStep {
    fn append_dac_samples(&self, samples: &mut Vec<u8>) {
        match self {
            VSoundOrganNoteStep::MaskByte(_) => {}
            VSoundOrganNoteStep::NoteStarted(note) => {
                samples.extend_from_slice(&note.window.dac_samples);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSoundOrganLoadError {
    MissingSoundRomByte { address: u16 },
    InvalidTuneLength { table_address: u16, length: u8 },
    DelayPatchTooLong { delay: u8 },
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

    /// Source-shaped `VSNDRM1.SRC` `SETUP` visible initialization: configure
    /// the sound PIA data direction/control registers, seed `HI`, and clear the
    /// source global flags.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L176-L193>.
    pub fn run_vsnd_setup(&mut self) -> Result<VSoundSetup, SoundCpuWriteError> {
        self.write_byte(SOUND_CPU_PIA_START + 1, 0x00)?;
        self.write_byte(SOUND_CPU_PIA_START + 3, 0x00)?;
        self.write_byte(SOUND_CPU_PIA_START, 0xFF)?;
        self.write_byte(SOUND_CPU_PIA_START + 2, 0x00)?;
        self.write_byte(SOUND_CPU_PIA_START + 3, 0x37)?;
        self.write_byte(SOUND_CPU_PIA_START + 1, 0x3C)?;
        self.ram[VSNDRM1_HI_OFFSET] = 0x3C;
        self.ram[VSNDRM1_B2FLG_OFFSET] = 0;
        self.ram[VSNDRM1_BG1FLG_OFFSET] = 0;
        self.ram[VSNDRM1_BG2FLG_OFFSET] = 0;
        self.ram[VSNDRM1_SP1FLG_OFFSET] = 0;
        self.ram[VSNDRM1_ORGFLG_OFFSET] = 0;

        Ok(VSoundSetup {
            stack_top: VSNDRM1_STACK_TOP,
            pia_ddr_a: self.pia.ddr_a(),
            pia_ddr_b: self.pia.ddr_b(),
            pia_control_a: self.pia.control_a(),
            pia_control_b: self.pia.control_b(),
            random_seed_hi: self.ram[VSNDRM1_HI_OFFSET],
        })
    }

    /// Source-shaped `VSNDRM1.SRC` `IRQ` prelude: reset the interrupt stack,
    /// read the sound command through PIA port B at `SOUND+2`, then invert and
    /// mask the low five command bits. In the configured `SETUP` state, the PIA
    /// read also clears the CB1 command IRQ flag.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L914-L920>.
    pub fn run_vsnd_irq_prelude(&mut self) -> VSoundIrqPrelude {
        let irq_asserted_before_read = self.sound_irq_asserted();
        let latched_port_b = self.pia.read(
            2,
            SOUND_PIA_UNCONNECTED_PORT_A_INPUT,
            self.sound_pia_port_b_input,
        );
        VSoundIrqPrelude {
            stack_top: VSNDRM1_STACK_TOP,
            latched_port_b,
            command_code: (!latched_port_b) & 0x1F,
            irq_asserted_before_read,
            irq_asserted_after_read: self.sound_irq_asserted(),
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `IRQ` command decode and source-visible flag
    /// mutations around the dispatch boundary. This decodes the latched PIA
    /// port-B command, runs the simple flag effects that surround the routine
    /// jump table, and reports which waveform/special routine would execute.
    /// Full waveform loop scheduling remains separate work.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L914-L967>.
    pub fn step_vsnd_irq_dispatch(&mut self) -> VSoundIrqDispatch {
        let latched_port_b = self.sound_pia_port_b_input;
        let command_code = (!latched_port_b) & 0x1F;
        let orgflg = self.ram[VSNDRM1_ORGFLG_OFFSET];
        let organ_step = if orgflg == 0 {
            VSoundOrganIrqStep::Inactive
        } else if orgflg & 0x80 == 0 {
            VSoundOrganIrqStep::Note
        } else {
            VSoundOrganIrqStep::TuneAndNote
        };
        let effective_code = if orgflg == 0 {
            command_code
        } else {
            command_code.wrapping_sub(1)
        };

        if effective_code != VSNDRM1_SPINNER_SOUND_CODE {
            self.ram[VSNDRM1_SP1FLG_OFFSET] = 0;
        }
        if effective_code != VSNDRM1_BONUS2_SOUND_CODE {
            self.ram[VSNDRM1_B2FLG_OFFSET] = 0;
        }

        let talking_program_present =
            self.roms.sound_cpu_byte(VSNDRM1_TALK_ENTRY_ADDRESS) == Some(0x7E);
        let routine = vsnd_irq_routine(effective_code);
        self.apply_vsound_special_flag_effects(routine);
        let background = self.vsound_background_continuation();

        VSoundIrqDispatch {
            latched_port_b,
            command_code,
            effective_code,
            organ_step,
            talking_program_present,
            routine,
            spinner_flag: self.ram[VSNDRM1_SP1FLG_OFFSET],
            bonus2_flag: self.ram[VSNDRM1_B2FLG_OFFSET],
            background1_flag: self.ram[VSNDRM1_BG1FLG_OFFSET],
            background2_flag: self.ram[VSNDRM1_BG2FLG_OFFSET],
            background,
        }
    }

    /// Source-shaped organ branch from `VSNDRM1.SRC` `IRQ`: when `ORGFLG` is
    /// negative, run `ORGNT1` with the raw decoded command as the tune number;
    /// when it is positive, apply the source `IRQ0`/`ORGNN1` parameter
    /// decrements and consume an organ-note byte. A zero `ORGFLG` leaves normal
    /// IRQ dispatch to the caller.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L914-L924>.
    pub fn step_vsnd_irq_organ_flow(&mut self) -> Result<VSoundIrqOrganFlow, VSoundOrganLoadError> {
        let latched_port_b = self.sound_pia_port_b_input;
        let command_code = (!latched_port_b) & 0x1F;
        let orgflg = self.ram[VSNDRM1_ORGFLG_OFFSET];

        if orgflg == 0 {
            return Ok(VSoundIrqOrganFlow::Inactive {
                latched_port_b,
                command_code,
            });
        }

        if orgflg & 0x80 != 0 {
            return Ok(VSoundIrqOrganFlow::Tune {
                latched_port_b,
                command_code,
                tune: self.run_vsnd_organ_tune(command_code)?,
            });
        }

        let parameter_code = if orgflg == 1 {
            command_code.wrapping_sub(2)
        } else {
            command_code.wrapping_sub(1)
        };
        Ok(VSoundIrqOrganFlow::Note {
            latched_port_b,
            command_code,
            parameter_code,
            step: self.step_vsnd_organ_note_parameter(parameter_code)?,
        })
    }

    /// Source-shaped top-level `VSNDRM1.SRC` `IRQ` gate. Active `ORGFLG`
    /// branches run the organ code before normal command dispatch can mutate
    /// spinner/bonus/background flags. `ORGNT1` jumps to `IRQ3`, so tune
    /// branches may enter the background handoff; `ORGNN1` loops inside the
    /// organ-note path, so note branches stop there.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L914-L967>.
    pub fn step_vsnd_irq_flow(&mut self) -> Result<VSoundIrqFlow, VSoundIrqFlowError> {
        let orgflg = self.ram[VSNDRM1_ORGFLG_OFFSET];
        if orgflg == 0 {
            return self
                .step_vsnd_irq_command_and_background_flow()
                .map(VSoundIrqFlow::Command)
                .map_err(VSoundIrqFlowError::Command);
        }

        let organ = self
            .step_vsnd_irq_organ_flow()
            .map_err(VSoundIrqFlowError::Organ)?;

        if orgflg & 0x80 != 0 {
            let irq3 = VSoundIrq3AfterCommand::Entered(
                self.step_vsnd_irq3_background_flow()
                    .map_err(VSoundIrqFlowError::Background)?,
            );
            Ok(VSoundIrqFlow::OrganTune { organ, irq3 })
        } else {
            Ok(VSoundIrqFlow::OrganNote { organ })
        }
    }

    /// Source-shaped top-level `VSNDRM1.SRC` `IRQ` entry from the PIA command
    /// read through the translated command/organ/background flow. The prelude
    /// consumes the CB1 command edge before the source branches inspect
    /// `ORGFLG` and dispatch the decoded sound command.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L914-L967>.
    pub fn step_vsnd_irq_cycle(&mut self) -> Result<VSoundIrqCycle, VSoundIrqFlowError> {
        let prelude = self.run_vsnd_irq_prelude();
        match self.step_vsnd_irq_flow() {
            Ok(flow) => Ok(VSoundIrqCycle { prelude, flow }),
            Err(error) => Err(error),
        }
    }

    /// Source-visible timed IRQ cycle: run the translated IRQ entry, collect
    /// the generated DAC write bytes in source order, and attach monotonically
    /// increasing DAC-write ticks. The tick unit is one translated DAC write,
    /// not a 6808 CPU cycle.
    pub fn step_vsnd_irq_timed_cycle(
        &mut self,
        irq_tick: u64,
    ) -> Result<VSoundIrqTimedCycle, VSoundIrqFlowError> {
        let cycle = self.step_vsnd_irq_cycle()?;
        let dac_samples = cycle.dac_samples();
        let last_dac_value = self.last_dac_value();
        let dac = VSoundDacSampleWindow {
            first_tick: if dac_samples.is_empty() {
                None
            } else {
                Some(irq_tick)
            },
            tick_step: VSOUND_SOURCE_DAC_TICK_STEP,
            dac_samples,
            last_dac_value,
        };

        Ok(VSoundIrqTimedCycle {
            irq_tick,
            cycle,
            dac,
            irq_asserted_after_cycle: self.sound_irq_asserted(),
        })
    }

    /// Source-shaped `VSNDRM1.SRC` `NMI` diagnostic cycle: reset the stack,
    /// checksum the sound ROM span with the source `ADCB` carry chain, wait on
    /// checksum failure, or run `VARILD #1` / `VARI` before checking the
    /// diagnostic talking-program entry.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L981-L999>.
    pub fn run_vsnd_nmi_diagnostic_cycle(
        &mut self,
    ) -> Result<VSoundNmiDiagnosticCycle, VSoundNmiDiagnosticError> {
        let checksum = self.vsnd_nmi_checksum()?;
        if checksum.computed_checksum != checksum.expected_checksum {
            return Ok(VSoundNmiDiagnosticCycle::ChecksumMismatch(checksum));
        }

        let vari_load = self
            .load_vsnd_vari_vector(VSNDRM1_NMI_VARI_VECTOR_INDEX)
            .map_err(VSoundNmiDiagnosticError::Vari)?;
        let sweep = self.run_vsnd_vari_start();
        Ok(VSoundNmiDiagnosticCycle::ChecksumMatched {
            checksum,
            vari_load,
            sweep,
            talking_diagnostic_present: self.roms.sound_cpu_byte(VSNDRM1_TALKD_ENTRY_ADDRESS)
                == Some(0x7E),
        })
    }

    /// Source-shaped `VSNDRM1.SRC` `IRQ1` executable branches for normal
    /// command flow. `GWAVE` and `VARI` commands run their source loaders and
    /// first translated waveform window; special jump-table commands are
    /// returned after `step_vsnd_irq_dispatch` applies their source-visible flag
    /// effects so callers do not double-run the special setup helpers.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L939-L958>.
    pub fn step_vsnd_irq_command_flow(
        &mut self,
    ) -> Result<VSoundIrqCommandFlow, VSoundIrqCommandFlowError> {
        let previous_bonus2_flag = self.ram[VSNDRM1_B2FLG_OFFSET];
        let dispatch = self.step_vsnd_irq_dispatch();
        match dispatch.routine {
            VSoundIrqRoutine::Invalid => Ok(VSoundIrqCommandFlow::Invalid { dispatch }),
            VSoundIrqRoutine::GWave { sound_code, .. } => {
                let load = self
                    .load_vsnd_gwave_sound_code(sound_code)
                    .map_err(VSoundIrqCommandFlowError::GWave)?;
                let step = self
                    .run_vsnd_gwave_start()
                    .map_err(VSoundIrqCommandFlowError::GWave)?;
                Ok(VSoundIrqCommandFlow::GWave {
                    dispatch,
                    load,
                    step,
                })
            }
            VSoundIrqRoutine::Special { routine, .. } => {
                let step = self.step_vsnd_irq_special_flow(routine, previous_bonus2_flag)?;
                Ok(VSoundIrqCommandFlow::Special { dispatch, step })
            }
            VSoundIrqRoutine::Vari { sound_code, .. } => {
                let load = self
                    .load_vsnd_vari_sound_code(sound_code)
                    .map_err(VSoundIrqCommandFlowError::Vari)?;
                let sweep = self.run_vsnd_vari_start();
                Ok(VSoundIrqCommandFlow::Vari {
                    dispatch,
                    load,
                    sweep,
                })
            }
        }
    }

    /// Source-shaped `IRQ1` command slice followed by the `IRQ3` background
    /// handoff only when the command routine has returned to the source branch
    /// that enters `IRQ3`. Continuous routines report the running handoff and
    /// leave background continuation for a later interrupt.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L939-L967>.
    pub fn step_vsnd_irq_command_and_background_flow(
        &mut self,
    ) -> Result<VSoundIrqCommandAndBackgroundFlow, VSoundIrqCommandAndBackgroundFlowError> {
        let command = self
            .step_vsnd_irq_command_flow()
            .map_err(VSoundIrqCommandAndBackgroundFlowError::Command)?;
        let irq3 = match command.irq3_handoff() {
            VSoundIrq3Handoff::Ready => VSoundIrq3AfterCommand::Entered(
                self.step_vsnd_irq3_background_flow()
                    .map_err(VSoundIrqCommandAndBackgroundFlowError::Background)?,
            ),
            handoff => VSoundIrq3AfterCommand::Skipped(handoff),
        };

        Ok(VSoundIrqCommandAndBackgroundFlow { command, irq3 })
    }

    fn step_vsnd_irq_special_flow(
        &mut self,
        routine: VSoundSpecialRoutine,
        previous_bonus2_flag: u8,
    ) -> Result<VSoundIrqSpecialFlow, VSoundIrqCommandFlowError> {
        let step = match routine {
            VSoundSpecialRoutine::Spinner1 => VSoundIrqSpecialFlow::Spinner1(
                self.run_vsnd_spinner1_command()
                    .map_err(VSoundIrqCommandFlowError::Spinner1)?,
            ),
            VSoundSpecialRoutine::Background1 => {
                VSoundIrqSpecialFlow::Background1(self.run_vsnd_background1_fnoise_window())
            }
            VSoundSpecialRoutine::Background2Increment => {
                VSoundIrqSpecialFlow::Background2Increment(
                    self.run_vsnd_background2_setup()
                        .map_err(VSoundIrqCommandFlowError::Background2)?,
                )
            }
            VSoundSpecialRoutine::Lite => VSoundIrqSpecialFlow::Lite(self.run_vsnd_lite()),
            VSoundSpecialRoutine::Bonus2 => VSoundIrqSpecialFlow::Bonus2(
                self.run_vsnd_bonus2_command(previous_bonus2_flag)
                    .map_err(VSoundIrqCommandFlowError::Bonus2)?,
            ),
            VSoundSpecialRoutine::BackgroundEnd => {
                VSoundIrqSpecialFlow::BackgroundEnd(self.vsound_background_flags())
            }
            VSoundSpecialRoutine::Turbo => VSoundIrqSpecialFlow::Turbo(self.run_vsnd_turbo()),
            VSoundSpecialRoutine::Appear => VSoundIrqSpecialFlow::Appear(self.run_vsnd_appear()),
            VSoundSpecialRoutine::Thrust => {
                VSoundIrqSpecialFlow::Thrust(self.run_vsnd_thrust_fnoise_window())
            }
            VSoundSpecialRoutine::Cannon => VSoundIrqSpecialFlow::Cannon(self.run_vsnd_cannon()),
            VSoundSpecialRoutine::Radio => VSoundIrqSpecialFlow::Radio(
                self.run_vsnd_radio()
                    .map_err(VSoundIrqCommandFlowError::Radio)?,
            ),
            VSoundSpecialRoutine::Hyper => VSoundIrqSpecialFlow::Hyper(self.run_vsnd_hyper()),
            VSoundSpecialRoutine::Scream => VSoundIrqSpecialFlow::Scream(self.run_vsnd_scream()),
            VSoundSpecialRoutine::OrganTune => {
                VSoundIrqSpecialFlow::OrganTune(self.vsound_organ_tune_start())
            }
            VSoundSpecialRoutine::OrganNote => {
                VSoundIrqSpecialFlow::OrganNote(self.vsound_organ_note_start())
            }
        };
        Ok(step)
    }

    fn vsound_background_flags(&self) -> VSoundBackgroundFlags {
        VSoundBackgroundFlags {
            background1_flag: self.ram[VSNDRM1_BG1FLG_OFFSET],
            background2_flag: self.ram[VSNDRM1_BG2FLG_OFFSET],
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `IRQ3` background handoff. When either
    /// background flag is active, the source kills `B2FLG` and jumps to `BG1`
    /// if `BG1FLG` is set, otherwise to `BG2`. Callers decide when the current
    /// sound routine has returned far enough to enter this handoff.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L958-L967>.
    pub fn step_vsnd_irq3_background_flow(
        &mut self,
    ) -> Result<VSoundIrq3BackgroundFlow, VSoundGWaveLoadError> {
        if self.ram[VSNDRM1_BG1FLG_OFFSET] | self.ram[VSNDRM1_BG2FLG_OFFSET] == 0 {
            return Ok(VSoundIrq3BackgroundFlow::WaitingForBackground);
        }

        self.ram[VSNDRM1_B2FLG_OFFSET] = 0;
        if self.ram[VSNDRM1_BG1FLG_OFFSET] != 0 {
            Ok(VSoundIrq3BackgroundFlow::Background1(
                self.run_vsnd_background1_fnoise_window(),
            ))
        } else {
            Ok(VSoundIrq3BackgroundFlow::Background2(
                self.run_vsnd_background2_setup()?,
            ))
        }
    }

    fn vsound_organ_tune_start(&self) -> VSoundOrganTuneStart {
        VSoundOrganTuneStart {
            organ_flag: self.ram[VSNDRM1_ORGFLG_OFFSET],
        }
    }

    fn vsound_organ_note_start(&self) -> VSoundOrganNoteStart {
        VSoundOrganNoteStart {
            organ_flag: self.ram[VSNDRM1_ORGFLG_OFFSET],
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `GWLD`: load one GWAVE vector from `SVTAB`,
    /// copy the selected `GWVTAB` waveform to direct-page `GWTAB`, apply
    /// `WVDECA` predecay, and write the direct-page GWAVE parameters.
    /// Cycle-accurate `GWAVE` DAC scheduling remains separate work.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L736-L789>.
    pub fn load_vsnd_gwave_sound_code(
        &mut self,
        sound_code: u8,
    ) -> Result<VSoundGWaveLoad, VSoundGWaveLoadError> {
        match vsnd_irq_routine(sound_code) {
            VSoundIrqRoutine::GWave { table_index, .. } => self.load_vsnd_gwave_vector(table_index),
            VSoundIrqRoutine::Invalid
            | VSoundIrqRoutine::Special { .. }
            | VSoundIrqRoutine::Vari { .. } => {
                Err(VSoundGWaveLoadError::InvalidSoundCode { sound_code })
            }
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `GWLD` entry by zero-based `SVTAB` vector
    /// index. Valid indexes are 0..=14; the first 13 are normal IRQ GWAVE
    /// commands, while 13 and 14 are used by `BON2` and `BG2`.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L736-L789>.
    pub fn load_vsnd_gwave_vector(
        &mut self,
        table_index: u8,
    ) -> Result<VSoundGWaveLoad, VSoundGWaveLoadError> {
        if table_index >= VSNDRM1_GWAVE_VECTOR_COUNT {
            return Err(VSoundGWaveLoadError::InvalidVectorIndex { table_index });
        }

        let vector_address = VSNDRM1_SVTAB_ADDRESS + u16::from(table_index) * 7;
        let vector = self.vsnd_sound_rom_bytes::<7>(vector_address)?;
        let cycle_count = vector[0] & 0x0F;
        let echo_count = vector[0] >> 4;
        let echo_decay = vector[1] >> 4;
        let waveform_index = vector[1] & 0x0F;
        let waveform_address = self.vsnd_waveform_address(waveform_index)?;
        let waveform_length = self.vsnd_sound_rom_byte(waveform_address)?;
        let wave_ram_start = VSNDRM1_GWTAB_OFFSET as u16;
        let wave_ram_end = self.ensure_vsnd_waveform_length(waveform_address, waveform_length)?;

        self.ram[VSNDRM1_GCCNT_OFFSET] = cycle_count;
        self.ram[VSNDRM1_GECHO_OFFSET] = echo_count;
        self.ram[VSNDRM1_GECDEC_OFFSET] = echo_decay;
        self.write_direct_word(VSNDRM1_GWFRM_OFFSET, waveform_address);
        self.copy_vsnd_waveform_to_ram(waveform_address, waveform_length)?;

        let predecay_factor = vector[2];
        self.ram[VSNDRM1_PRDECA_OFFSET] = predecay_factor;
        self.apply_vsnd_wave_decay(waveform_address, waveform_length, predecay_factor)?;

        let frequency_delta = vector[3];
        let frequency_delta_count = vector[4];
        let frequency_pattern_length = vector[5];
        let frequency_pattern_address = VSNDRM1_GFRTAB_ADDRESS.wrapping_add(u16::from(vector[6]));
        let frequency_end_address =
            frequency_pattern_address.wrapping_add(u16::from(frequency_pattern_length));

        self.ram[VSNDRM1_GDFINC_OFFSET] = frequency_delta;
        self.ram[VSNDRM1_GDCNT_OFFSET] = frequency_delta_count;
        self.write_direct_word(VSNDRM1_GWFRQ_OFFSET, frequency_pattern_address);
        self.ram[VSNDRM1_FOFSET_OFFSET] = 0;
        self.write_direct_word(VSNDRM1_FRQEND_OFFSET, frequency_end_address);

        Ok(VSoundGWaveLoad {
            table_index,
            vector_address,
            echo_count,
            cycle_count,
            echo_decay,
            waveform_index,
            waveform_address,
            waveform_length,
            predecay_factor,
            frequency_delta,
            frequency_delta_count,
            frequency_pattern_address,
            frequency_pattern_length,
            frequency_end_address,
            wave_ram_start,
            wave_ram_end,
        })
    }

    /// Source-shaped `VSNDRM1.SRC` `VARILD`: load one nine-byte VARI vector
    /// from `VVECT` into direct-page VARI parameters. Cycle-accurate `VARI`
    /// output scheduling remains separate work.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L198-L209>.
    pub fn load_vsnd_vari_sound_code(
        &mut self,
        sound_code: u8,
    ) -> Result<VSoundVariLoad, VSoundVariLoadError> {
        match vsnd_irq_routine(sound_code) {
            VSoundIrqRoutine::Vari { table_index, .. } => self.load_vsnd_vari_vector(table_index),
            VSoundIrqRoutine::Invalid
            | VSoundIrqRoutine::GWave { .. }
            | VSoundIrqRoutine::Special { .. } => {
                Err(VSoundVariLoadError::InvalidSoundCode { sound_code })
            }
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `VARILD` entry by zero-based `VVECT` vector
    /// index. Defender's table has SAW, FOSHIT, QUASAR, and CABSHK.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L1008-L1014>.
    pub fn load_vsnd_vari_vector(
        &mut self,
        table_index: u8,
    ) -> Result<VSoundVariLoad, VSoundVariLoadError> {
        if table_index >= VSNDRM1_VARI_VECTOR_COUNT {
            return Err(VSoundVariLoadError::InvalidVectorIndex { table_index });
        }

        let vector_address = VSNDRM1_VVECT_ADDRESS + u16::from(table_index) * 9;
        let vector = self.vsnd_vari_rom_bytes::<9>(vector_address)?;
        self.ram[VSNDRM1_LOPER_OFFSET] = vector[0];
        self.ram[VSNDRM1_HIPER_OFFSET] = vector[1];
        self.ram[VSNDRM1_LODT_OFFSET] = vector[2];
        self.ram[VSNDRM1_HIDT_OFFSET] = vector[3];
        self.ram[VSNDRM1_HIEN_OFFSET] = vector[4];
        self.ram[VSNDRM1_SWPDT_OFFSET] = vector[5];
        self.ram[VSNDRM1_SWPDT_OFFSET + 1] = vector[6];
        self.ram[VSNDRM1_LOMOD_OFFSET] = vector[7];
        self.ram[VSNDRM1_VAMP_OFFSET] = vector[8];

        Ok(VSoundVariLoad {
            table_index,
            vector_address,
            low_period: vector[0],
            high_period: vector[1],
            low_period_delta: vector[2],
            high_period_delta: vector[3],
            high_period_end: vector[4],
            sweep_period: u16::from_be_bytes([vector[5], vector[6]]),
            low_period_mod: vector[7],
            amplitude: vector[8],
        })
    }

    /// Source-shaped `VSNDRM1.SRC` `VARI` / `VAR0` entry: output `VAMP`, seed
    /// `LOCNT` / `HICNT` from `LOPER` / `HIPER`, then run one `V0` sweep.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L213-L251>.
    pub fn run_vsnd_vari_start(&mut self) -> VSoundVariSweep {
        let mut dac_samples = Vec::new();
        self.emit_vsnd_dac_sample(&mut dac_samples, self.ram[VSNDRM1_VAMP_OFFSET]);
        self.ram[VSNDRM1_LOCNT_OFFSET] = self.ram[VSNDRM1_LOPER_OFFSET];
        self.ram[VSNDRM1_HICNT_OFFSET] = self.ram[VSNDRM1_HIPER_OFFSET];
        self.run_vsnd_vari_sweep_with_samples(dac_samples)
    }

    /// Source-shaped `VSNDRM1.SRC` `V0` / `VSWEEP`: run one variable-duty
    /// square-wave sweep using the existing direct-page counters and DAC latch.
    /// The returned bytes are the source `STAA SOUND` / `COM SOUND` transitions;
    /// exact CPU-cycle spacing remains separate work.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L219-L251>.
    pub fn run_vsnd_vari_sweep(&mut self) -> VSoundVariSweep {
        self.run_vsnd_vari_sweep_with_samples(Vec::new())
    }

    fn run_vsnd_vari_sweep_with_samples(&mut self, mut dac_samples: Vec<u8>) -> VSoundVariSweep {
        let sweep_period = self.read_direct_word(VSNDRM1_SWPDT_OFFSET);
        let low_count = self.ram[VSNDRM1_LOCNT_OFFSET];
        let high_count = self.ram[VSNDRM1_HICNT_OFFSET];
        let mut sweep_counter = sweep_period;

        'sweep: loop {
            let mut low_counter = low_count;
            self.complement_vsnd_dac_sample(&mut dac_samples);
            loop {
                sweep_counter = sweep_counter.wrapping_sub(1);
                if sweep_counter == 0 {
                    break 'sweep;
                }
                low_counter = low_counter.wrapping_sub(1);
                if low_counter == 0 {
                    break;
                }
            }

            self.complement_vsnd_dac_sample(&mut dac_samples);
            let mut high_counter = high_count;
            loop {
                sweep_counter = sweep_counter.wrapping_sub(1);
                if sweep_counter == 0 {
                    break 'sweep;
                }
                high_counter = high_counter.wrapping_sub(1);
                if high_counter == 0 {
                    break;
                }
            }
        }

        let current_sample = self.current_vsnd_dac_sample();
        let sweep_sample = if current_sample & 0x80 == 0 {
            !current_sample
        } else {
            current_sample
        };
        self.emit_vsnd_dac_sample(&mut dac_samples, sweep_sample);

        let low_count_after_sweep = low_count.wrapping_add(self.ram[VSNDRM1_LODT_OFFSET]);
        self.ram[VSNDRM1_LOCNT_OFFSET] = low_count_after_sweep;
        let high_count_after_sweep = high_count.wrapping_add(self.ram[VSNDRM1_HIDT_OFFSET]);
        self.ram[VSNDRM1_HICNT_OFFSET] = high_count_after_sweep;

        let result = if high_count_after_sweep != self.ram[VSNDRM1_HIEN_OFFSET] {
            VSoundVariSweepResult::Continue
        } else {
            self.finish_vsnd_vari_sweep_at_high_end()
        };

        VSoundVariSweep {
            sweep_period,
            low_count,
            high_count,
            low_count_after_sweep,
            high_count_after_sweep,
            dac_samples,
            result,
        }
    }

    fn finish_vsnd_vari_sweep_at_high_end(&mut self) -> VSoundVariSweepResult {
        let low_period_mod = self.ram[VSNDRM1_LOMOD_OFFSET];
        if low_period_mod == 0 {
            return VSoundVariSweepResult::Terminated {
                low_period: self.ram[VSNDRM1_LOPER_OFFSET],
                low_period_mod,
            };
        }

        let low_period = self.ram[VSNDRM1_LOPER_OFFSET].wrapping_add(low_period_mod);
        self.ram[VSNDRM1_LOPER_OFFSET] = low_period;
        if low_period == 0 {
            return VSoundVariSweepResult::Terminated {
                low_period,
                low_period_mod,
            };
        }

        self.ram[VSNDRM1_LOCNT_OFFSET] = low_period;
        self.ram[VSNDRM1_HICNT_OFFSET] = self.ram[VSNDRM1_HIPER_OFFSET];
        VSoundVariSweepResult::Restarted { low_period }
    }

    /// Source-shaped `VSNDRM1.SRC` `LITE`: seed the lightning frequency delta,
    /// then enter the shared `LITEN` random complement loop.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L253-L258>.
    pub fn run_vsnd_lite(&mut self) -> VSoundLiteNoise {
        self.ram[VSNDRM1_DFREQ_OFFSET] = 1;
        self.run_vsnd_liten(1, 3)
    }

    /// Source-shaped `VSNDRM1.SRC` `APPEAR`: seed the appearance frequency
    /// delta and start frequency, then enter the shared `LITEN` loop.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L260-L266>.
    pub fn run_vsnd_appear(&mut self) -> VSoundLiteNoise {
        self.ram[VSNDRM1_DFREQ_OFFSET] = 0xFE;
        self.run_vsnd_liten(0xC0, 0x10)
    }

    fn run_vsnd_liten(&mut self, initial_frequency: u8, cycle_count: u8) -> VSoundLiteNoise {
        self.ram[VSNDRM1_LFREQ_OFFSET] = initial_frequency;
        let mut dac_samples = Vec::new();
        self.emit_vsnd_dac_sample(&mut dac_samples, 0xFF);
        self.ram[VSNDRM1_CYCNT_OFFSET] = cycle_count;

        let mut frequency_passes = 0;
        let mut random_steps = 0;
        loop {
            let mut cycles = self.ram[VSNDRM1_CYCNT_OFFSET];
            loop {
                let carry = self.advance_vsnd_random_seed();
                random_steps += 1;
                if carry {
                    self.complement_vsnd_dac_sample(&mut dac_samples);
                }
                cycles = cycles.wrapping_sub(1);
                if cycles == 0 {
                    break;
                }
            }

            frequency_passes += 1;
            let next_frequency =
                self.ram[VSNDRM1_LFREQ_OFFSET].wrapping_add(self.ram[VSNDRM1_DFREQ_OFFSET]);
            self.ram[VSNDRM1_LFREQ_OFFSET] = next_frequency;
            if next_frequency == 0 {
                break;
            }
        }

        VSoundLiteNoise {
            initial_frequency,
            final_frequency: self.ram[VSNDRM1_LFREQ_OFFSET],
            frequency_delta: self.ram[VSNDRM1_DFREQ_OFFSET],
            cycle_count: self.ram[VSNDRM1_CYCNT_OFFSET],
            frequency_passes,
            random_steps,
            random_seed_hi: self.ram[VSNDRM1_HI_OFFSET],
            random_seed_lo: self.ram[VSNDRM1_LO_OFFSET],
            dac_samples,
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `TURBO`: seed the white-noise parameters,
    /// then run the `NOISE` path with frequency decay enabled.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L296-L339>.
    pub fn run_vsnd_turbo(&mut self) -> VSoundTurboNoise {
        self.ram[VSNDRM1_CYCNT_OFFSET] = 0x20;
        self.ram[VSNDRM1_NFFLG_OFFSET] = 0x20;
        self.run_vsnd_turbo_noise(1, 0xFF, 1)
    }

    fn run_vsnd_turbo_noise(
        &mut self,
        initial_period: u16,
        initial_amplitude: u8,
        decay: u8,
    ) -> VSoundTurboNoise {
        self.ram[VSNDRM1_DECAY_OFFSET] = decay;
        let mut period = initial_period;
        let mut amplitude = initial_amplitude;
        let mut amplitude_passes = 0;
        let mut random_steps = 0;
        let mut dac_samples = Vec::new();

        loop {
            self.write_direct_word(VSNDRM1_NFRQ1_OFFSET, period);
            self.ram[VSNDRM1_NAMP_OFFSET] = amplitude;
            let mut cycles = self.ram[VSNDRM1_CYCNT_OFFSET];
            loop {
                let sample = if self.advance_vsnd_random_seed() {
                    self.ram[VSNDRM1_NAMP_OFFSET]
                } else {
                    0
                };
                self.emit_vsnd_dac_sample(&mut dac_samples, sample);
                random_steps += 1;
                cycles = cycles.wrapping_sub(1);
                if cycles == 0 {
                    break;
                }
            }

            amplitude_passes += 1;
            let next_amplitude = amplitude.wrapping_sub(self.ram[VSNDRM1_DECAY_OFFSET]);
            if next_amplitude == 0 {
                break;
            }
            period = self.read_direct_word(VSNDRM1_NFRQ1_OFFSET).wrapping_add(1);
            amplitude = next_amplitude;
        }

        VSoundTurboNoise {
            initial_period,
            final_period: self.read_direct_word(VSNDRM1_NFRQ1_OFFSET),
            initial_amplitude,
            final_amplitude: self.ram[VSNDRM1_NAMP_OFFSET],
            decay: self.ram[VSNDRM1_DECAY_OFFSET],
            cycle_count: self.ram[VSNDRM1_CYCNT_OFFSET],
            amplitude_passes,
            random_steps,
            random_seed_hi: self.ram[VSNDRM1_HI_OFFSET],
            random_seed_lo: self.ram[VSNDRM1_LO_OFFSET],
            dac_samples,
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `CANNON`: seed the filtered-noise distortion
    /// parameters, then run the terminating frequency-decay `FNOISE` path.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L355-L431>.
    pub fn run_vsnd_cannon(&mut self) -> VSoundFilteredNoise {
        self.ram[VSNDRM1_DSFLG_OFFSET] = 1;
        self.run_vsnd_filtered_noise_decay(1000, 0xFF)
    }

    /// Source-shaped `VSNDRM1.SRC` `BG1`: keep `BG1FLG` set, enter `FNOISE`
    /// with the jump-table entry address still in X, and extract one continuous
    /// filtered-noise `SAMPC` window.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L341-L347>.
    pub fn run_vsnd_background1_fnoise_window(&mut self) -> VSoundFilteredNoiseWindow {
        self.ram[VSNDRM1_BG1FLG_OFFSET] = 1;
        self.ram[VSNDRM1_DSFLG_OFFSET] = 0;
        self.run_vsnd_filtered_noise_continuous_window(VSNDRM1_BG1_ENTRY_ADDRESS, 1)
    }

    /// Source-shaped `VSNDRM1.SRC` `THRUST`: enter `FNOISE` with the jump-table
    /// entry address still in X and extract one continuous filtered-noise
    /// `SAMPC` window.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L349-L354>.
    pub fn run_vsnd_thrust_fnoise_window(&mut self) -> VSoundFilteredNoiseWindow {
        self.ram[VSNDRM1_DSFLG_OFFSET] = 0;
        self.run_vsnd_filtered_noise_continuous_window(VSNDRM1_THRUST_ENTRY_ADDRESS, 3)
    }

    fn run_vsnd_filtered_noise_continuous_window(
        &mut self,
        entry_address: u16,
        initial_max_frequency: u8,
    ) -> VSoundFilteredNoiseWindow {
        self.ram[VSNDRM1_FDFLG_OFFSET] = 0;
        self.ram[VSNDRM1_FMAX_OFFSET] = initial_max_frequency;
        self.write_direct_word(VSNDRM1_SAMPC_OFFSET, entry_address);
        self.ram[VSNDRM1_FLO_OFFSET] = 0;
        let mut sample_count = self.read_direct_word(VSNDRM1_SAMPC_OFFSET);
        let mut dac_samples = Vec::new();
        let random_steps = self.run_vsnd_filtered_noise_window(&mut sample_count, &mut dac_samples);

        VSoundFilteredNoiseWindow {
            entry_address,
            initial_sample_count: entry_address,
            initial_max_frequency,
            final_frequency_high: self.ram[VSNDRM1_FHI_OFFSET],
            final_frequency_low: self.ram[VSNDRM1_FLO_OFFSET],
            frequency_decay_enabled: self.ram[VSNDRM1_FDFLG_OFFSET] != 0,
            distortion_enabled: self.ram[VSNDRM1_DSFLG_OFFSET] != 0,
            background1_flag: self.ram[VSNDRM1_BG1FLG_OFFSET],
            continues_after_window: self.ram[VSNDRM1_FDFLG_OFFSET] == 0,
            random_steps,
            random_seed_hi: self.ram[VSNDRM1_HI_OFFSET],
            random_seed_lo: self.ram[VSNDRM1_LO_OFFSET],
            dac_samples,
        }
    }

    fn run_vsnd_filtered_noise_decay(
        &mut self,
        initial_sample_count: u16,
        initial_max_frequency: u8,
    ) -> VSoundFilteredNoise {
        self.ram[VSNDRM1_FDFLG_OFFSET] = 1;
        self.ram[VSNDRM1_FMAX_OFFSET] = initial_max_frequency;
        self.write_direct_word(VSNDRM1_SAMPC_OFFSET, initial_sample_count);
        self.ram[VSNDRM1_FLO_OFFSET] = 0;
        let mut decay_passes = 0;
        let mut random_steps = 0;
        let mut dac_samples = Vec::new();

        loop {
            let mut sample_count = self.read_direct_word(VSNDRM1_SAMPC_OFFSET);
            random_steps +=
                self.run_vsnd_filtered_noise_window(&mut sample_count, &mut dac_samples);

            self.decay_vsnd_filtered_noise_frequency();
            decay_passes += 1;
            if self.ram[VSNDRM1_FMAX_OFFSET] == 0 && self.ram[VSNDRM1_FLO_OFFSET] == 7 {
                break;
            }
        }

        VSoundFilteredNoise {
            initial_sample_count,
            initial_max_frequency,
            final_max_frequency: self.ram[VSNDRM1_FMAX_OFFSET],
            final_frequency_high: self.ram[VSNDRM1_FHI_OFFSET],
            final_frequency_low: self.ram[VSNDRM1_FLO_OFFSET],
            frequency_decay_enabled: self.ram[VSNDRM1_FDFLG_OFFSET] != 0,
            distortion_enabled: self.ram[VSNDRM1_DSFLG_OFFSET] != 0,
            decay_passes,
            random_steps,
            random_seed_hi: self.ram[VSNDRM1_HI_OFFSET],
            random_seed_lo: self.ram[VSNDRM1_LO_OFFSET],
            dac_samples,
        }
    }

    fn run_vsnd_filtered_noise_window(
        &mut self,
        sample_count: &mut u16,
        dac_samples: &mut Vec<u8>,
    ) -> u16 {
        let mut sample = self.current_vsnd_dac_sample();
        let mut random_steps = 0;

        loop {
            self.advance_vsnd_filtered_noise_seed(sample);
            random_steps += 1;

            let mut frequency_high = self.ram[VSNDRM1_FMAX_OFFSET];
            if self.ram[VSNDRM1_DSFLG_OFFSET] != 0 {
                frequency_high &= self.ram[VSNDRM1_HI_OFFSET];
            }
            self.ram[VSNDRM1_FHI_OFFSET] = frequency_high;
            let mut frequency_low = self.ram[VSNDRM1_FLO_OFFSET];

            if sample > self.ram[VSNDRM1_LO_OFFSET] {
                loop {
                    *sample_count = sample_count.wrapping_sub(1);
                    if *sample_count == 0 {
                        break;
                    }
                    self.emit_vsnd_dac_sample(dac_samples, sample);

                    let (next_frequency_low, low_borrow) =
                        frequency_low.overflowing_sub(self.ram[VSNDRM1_FLO_OFFSET]);
                    let (next_sample_without_borrow, high_borrow) =
                        sample.overflowing_sub(self.ram[VSNDRM1_FHI_OFFSET]);
                    let (next_sample, carry_borrow) =
                        next_sample_without_borrow.overflowing_sub(u8::from(low_borrow));
                    frequency_low = next_frequency_low;
                    sample = next_sample;

                    if high_borrow || carry_borrow || sample <= self.ram[VSNDRM1_LO_OFFSET] {
                        break;
                    }
                }
            } else {
                loop {
                    *sample_count = sample_count.wrapping_sub(1);
                    if *sample_count == 0 {
                        break;
                    }
                    self.emit_vsnd_dac_sample(dac_samples, sample);

                    let (next_frequency_low, low_carry) =
                        frequency_low.overflowing_add(self.ram[VSNDRM1_FLO_OFFSET]);
                    let (next_sample_without_carry, high_carry) =
                        sample.overflowing_add(self.ram[VSNDRM1_FHI_OFFSET]);
                    let (next_sample, carry_carry) =
                        next_sample_without_carry.overflowing_add(u8::from(low_carry));
                    frequency_low = next_frequency_low;
                    sample = next_sample;

                    if high_carry || carry_carry || sample > self.ram[VSNDRM1_LO_OFFSET] {
                        break;
                    }
                }
            }

            if *sample_count == 0 {
                break;
            }
            sample = self.ram[VSNDRM1_LO_OFFSET];
            self.emit_vsnd_dac_sample(dac_samples, sample);
        }

        random_steps
    }

    /// Source-shaped `VSNDRM1.SRC` `RADIO`: add the current frequency to the
    /// 16-bit timer, raise frequency on timer overflow, and use the timer high
    /// nibble to index the `RADSND` waveform table.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L433-L457>.
    pub fn run_vsnd_radio(&mut self) -> Result<VSoundRadioWave, VSoundGWaveLoadError> {
        let table = self.vsnd_sound_rom_bytes::<16>(VSNDRM1_RADSND_ADDRESS)?;
        self.ram[VSNDRM1_XPTR_OFFSET] = (VSNDRM1_RADSND_ADDRESS >> 8) as u8;
        self.write_direct_word(VSNDRM1_TEMPX_OFFSET, 100);
        let initial_timer_high = self.ram[VSNDRM1_TEMPA_OFFSET];
        let mut timer_low = 0u8;
        let mut successful_frequency_increments = 0;
        let mut dac_samples = Vec::new();

        loop {
            let current_frequency = self.read_direct_word(VSNDRM1_TEMPX_OFFSET);
            let (next_timer_low, low_carry) =
                timer_low.overflowing_add(self.ram[VSNDRM1_TEMPX_OFFSET + 1]);
            timer_low = next_timer_low;

            let (timer_high_without_carry, high_byte_carry) =
                self.ram[VSNDRM1_TEMPA_OFFSET].overflowing_add(self.ram[VSNDRM1_TEMPX_OFFSET]);
            let (timer_high, low_byte_carry) =
                timer_high_without_carry.overflowing_add(u8::from(low_carry));
            self.ram[VSNDRM1_TEMPA_OFFSET] = timer_high;

            if high_byte_carry || low_byte_carry {
                let next_frequency = current_frequency.wrapping_add(1);
                if next_frequency == 0 {
                    break;
                }
                self.write_direct_word(VSNDRM1_TEMPX_OFFSET, next_frequency);
                successful_frequency_increments += 1;
            }

            let table_offset = self.ram[VSNDRM1_TEMPA_OFFSET] & 0x0F;
            self.ram[VSNDRM1_XPTR_OFFSET + 1] = (VSNDRM1_RADSND_ADDRESS as u8) + table_offset;
            self.emit_vsnd_dac_sample(&mut dac_samples, table[usize::from(table_offset)]);
        }

        Ok(VSoundRadioWave {
            table_address: VSNDRM1_RADSND_ADDRESS,
            table,
            initial_frequency: 100,
            final_frequency: self.read_direct_word(VSNDRM1_TEMPX_OFFSET),
            initial_timer_high,
            final_timer_high: self.ram[VSNDRM1_TEMPA_OFFSET],
            final_timer_low: timer_low,
            successful_frequency_increments,
            dac_samples,
        })
    }

    /// Source-shaped `VSNDRM1.SRC` `HYPER`: output silence, then complement the
    /// DAC latch at each phase edge and cycle edge while `TEMPA` advances.
    /// Exact CPU-cycle spacing remains separate work.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L459-L476>.
    pub fn run_vsnd_hyper(&mut self) -> VSoundHyperSweep {
        let mut dac_samples = Vec::new();
        self.emit_vsnd_dac_sample(&mut dac_samples, 0);
        self.ram[VSNDRM1_TEMPA_OFFSET] = 0;
        let mut phase_count = 0;

        loop {
            let phase = self.ram[VSNDRM1_TEMPA_OFFSET];
            let mut time_counter = 0u8;
            loop {
                if time_counter == phase {
                    self.complement_vsnd_dac_sample(&mut dac_samples);
                }
                time_counter = time_counter.wrapping_add(1);
                if time_counter & 0x80 != 0 {
                    break;
                }
            }

            self.complement_vsnd_dac_sample(&mut dac_samples);
            phase_count += 1;
            let next_phase = phase.wrapping_add(1);
            self.ram[VSNDRM1_TEMPA_OFFSET] = next_phase;
            if next_phase & 0x80 != 0 {
                break;
            }
        }

        VSoundHyperSweep {
            phase_count,
            final_phase: self.ram[VSNDRM1_TEMPA_OFFSET],
            dac_samples,
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `SCREAM`: run the four-echo frequency table
    /// cascade and output each mixed echo byte. `TEMPB` is deliberately used as
    /// the source timer in its current direct-page state; the routine does not
    /// clear it before the first 256-count decay window.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L478-L519>.
    pub fn run_vsnd_scream(&mut self) -> VSoundScream {
        for offset in VSNDRM1_STABLE_OFFSET..VSNDRM1_SRMEND_OFFSET {
            self.ram[offset] = 0;
        }
        self.ram[VSNDRM1_STABLE_OFFSET] = VSNDRM1_SCREAM_INITIAL_FREQUENCY;

        let initial_timer = self.ram[VSNDRM1_TEMPB_OFFSET];
        let mut decay_passes = 0;
        let mut echo_starts = 0;
        let mut dac_samples = Vec::new();

        loop {
            loop {
                self.ram[VSNDRM1_TEMPA_OFFSET] = 0x80;
                let mut output = 0u8;

                for echo in 0..VSNDRM1_SCREAM_ECHO_COUNT {
                    let frequency_offset = VSNDRM1_STABLE_OFFSET + 2 * echo;
                    let timer_offset = frequency_offset + 1;
                    let timer = self.ram[timer_offset].wrapping_add(self.ram[frequency_offset]);
                    self.ram[timer_offset] = timer;
                    if timer & 0x80 != 0 {
                        output = output.wrapping_add(self.ram[VSNDRM1_TEMPA_OFFSET]);
                    }
                    self.ram[VSNDRM1_TEMPA_OFFSET] >>= 1;
                }

                self.emit_vsnd_dac_sample(&mut dac_samples, output);
                self.ram[VSNDRM1_TEMPB_OFFSET] = self.ram[VSNDRM1_TEMPB_OFFSET].wrapping_add(1);
                if self.ram[VSNDRM1_TEMPB_OFFSET] == 0 {
                    break;
                }
            }

            let mut active_frequency_flag = 0u8;
            for echo in 0..VSNDRM1_SCREAM_ECHO_COUNT {
                let frequency_offset = VSNDRM1_STABLE_OFFSET + 2 * echo;
                let frequency = self.ram[frequency_offset];
                if frequency == 0 {
                    continue;
                }

                if frequency == VSNDRM1_SCREAM_SPAWN_FREQUENCY {
                    self.ram[frequency_offset + 2] = VSNDRM1_SCREAM_NEXT_ECHO_FREQUENCY;
                    active_frequency_flag = VSNDRM1_SCREAM_NEXT_ECHO_FREQUENCY;
                    echo_starts += 1;
                }

                self.ram[frequency_offset] = frequency.wrapping_sub(1);
                active_frequency_flag = active_frequency_flag.wrapping_add(1);
            }

            decay_passes += 1;
            if active_frequency_flag == 0 {
                break;
            }
        }

        let mut final_echo_table = [0u8; 2 * VSNDRM1_SCREAM_ECHO_COUNT];
        final_echo_table.copy_from_slice(&self.ram[VSNDRM1_STABLE_OFFSET..VSNDRM1_SRMEND_OFFSET]);

        VSoundScream {
            echo_count: VSNDRM1_SCREAM_ECHO_COUNT as u8,
            initial_frequency: VSNDRM1_SCREAM_INITIAL_FREQUENCY,
            next_echo_frequency: VSNDRM1_SCREAM_NEXT_ECHO_FREQUENCY,
            spawn_frequency: VSNDRM1_SCREAM_SPAWN_FREQUENCY,
            initial_timer,
            final_timer: self.ram[VSNDRM1_TEMPB_OFFSET],
            decay_passes,
            echo_starts,
            final_echo_table,
            srmend_byte: self.ram[VSNDRM1_SRMEND_OFFSET],
            dac_samples,
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `ORGANN`: set the three-step organ-note
    /// parameter gate. The following IRQs deliver source `ACCA` values after
    /// the IRQ routine's `DECA`.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L557-L560>.
    pub fn run_vsnd_organ_note_start(&mut self) -> VSoundOrganNoteStart {
        self.ram[VSNDRM1_ORGFLG_OFFSET] = 3;

        VSoundOrganNoteStart {
            organ_flag: self.ram[VSNDRM1_ORGFLG_OFFSET],
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `ORGNT1`: select a tune from `ORGTAB`,
    /// then run each four-byte `(OSCIL, delay, DUR)` note through `ORGANL` and
    /// one finite `ORGAN` duration window. Source IRQ scheduling and the
    /// preceding `ORGANT` flag cadence remain separate work.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L524-L555>.
    pub fn run_vsnd_organ_tune(
        &mut self,
        tune_number: u8,
    ) -> Result<VSoundOrganTuneStep, VSoundOrganLoadError> {
        self.ram[VSNDRM1_ORGFLG_OFFSET] = 0;
        self.ram[VSNDRM1_TEMPA_OFFSET] = tune_number;

        let mut table_address = VSNDRM1_ORGTAB_ADDRESS;
        loop {
            let tune_length = self.vsnd_organ_sound_rom_byte(table_address)?;
            if tune_length == 0 {
                return Ok(VSoundOrganTuneStep::InvalidTune {
                    tune_number,
                    table_address,
                });
            }

            self.ram[VSNDRM1_TEMPA_OFFSET] = self.ram[VSNDRM1_TEMPA_OFFSET].wrapping_sub(1);
            if self.ram[VSNDRM1_TEMPA_OFFSET] == 0 {
                return self.run_vsnd_selected_organ_tune(tune_number, table_address, tune_length);
            }

            table_address = table_address.wrapping_add(1 + u16::from(tune_length));
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `ORGNN1`: consume one organ-note parameter
    /// byte after the IRQ routine has decremented the latched command code.
    /// The first two parameters assemble `OSCIL`; the third looks up `NOTTAB`,
    /// patches `RDELAY` through `ORGANL`, and extracts one finite `ORGAN`
    /// duration window. The source then loops forever through `ORGNN4`; this
    /// method intentionally stops at the first source `DUR` window.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L562-L584>.
    pub fn step_vsnd_organ_note_parameter(
        &mut self,
        parameter_code: u8,
    ) -> Result<VSoundOrganNoteStep, VSoundOrganLoadError> {
        self.ram[VSNDRM1_ORGFLG_OFFSET] = self.ram[VSNDRM1_ORGFLG_OFFSET].wrapping_sub(1);
        let organ_flag = self.ram[VSNDRM1_ORGFLG_OFFSET];
        if organ_flag != 0 {
            let oscillator_mask = self.ram[VSNDRM1_OSCIL_OFFSET]
                .wrapping_shl(4)
                .wrapping_add(parameter_code);
            self.ram[VSNDRM1_OSCIL_OFFSET] = oscillator_mask;
            return Ok(VSoundOrganNoteStep::MaskByte(VSoundOrganMaskByte {
                parameter_code,
                organ_flag,
                oscillator_mask,
            }));
        }

        let note_delay = if parameter_code > 11 {
            0
        } else {
            self.vsnd_organ_sound_rom_byte(VSNDRM1_NOTTAB_ADDRESS + u16::from(parameter_code))?
        };
        self.write_direct_word(VSNDRM1_DUR_OFFSET, 0xFFFF);
        let delay_patch = self.load_vsnd_organ_delay(note_delay)?;
        let window = self.run_vsnd_organ_window(delay_patch);

        Ok(VSoundOrganNoteStep::NoteStarted(VSoundOrganNoteWindow {
            note_parameter: parameter_code,
            note_delay,
            organ_flag,
            window,
        }))
    }

    fn run_vsnd_selected_organ_tune(
        &mut self,
        tune_number: u8,
        table_address: u16,
        tune_length: u8,
    ) -> Result<VSoundOrganTuneStep, VSoundOrganLoadError> {
        if !tune_length.is_multiple_of(4) {
            return Err(VSoundOrganLoadError::InvalidTuneLength {
                table_address,
                length: tune_length,
            });
        }

        let tune_start_address = table_address.wrapping_add(1);
        let tune_end_address = tune_start_address.wrapping_add(u16::from(tune_length));
        self.write_direct_word(VSNDRM1_XPTR_OFFSET, tune_start_address);
        self.write_direct_word(VSNDRM1_XPLAY_OFFSET, tune_end_address);

        let mut note_address = tune_start_address;
        let mut notes = Vec::with_capacity(usize::from(tune_length / 4));
        while note_address != tune_end_address {
            let note = self.run_vsnd_organ_tune_note(note_address)?;
            notes.push(note);
            note_address = note_address.wrapping_add(4);
            self.write_direct_word(VSNDRM1_XPTR_OFFSET, note_address);
        }

        Ok(VSoundOrganTuneStep::Played(VSoundOrganTune {
            tune_number,
            table_address,
            tune_length,
            tune_start_address,
            tune_end_address,
            notes,
        }))
    }

    fn run_vsnd_organ_tune_note(
        &mut self,
        note_address: u16,
    ) -> Result<VSoundOrganTuneNote, VSoundOrganLoadError> {
        let oscillator_mask = self.vsnd_organ_sound_rom_byte(note_address)?;
        let note_delay = self.vsnd_organ_sound_rom_byte(note_address.wrapping_add(1))?;
        let duration = self.vsnd_organ_sound_rom_word(note_address.wrapping_add(2))?;

        self.ram[VSNDRM1_OSCIL_OFFSET] = oscillator_mask;
        self.write_direct_word(VSNDRM1_DUR_OFFSET, duration);
        let delay_patch = self.load_vsnd_organ_delay(note_delay)?;
        let window = self.run_vsnd_organ_window(delay_patch);

        Ok(VSoundOrganTuneNote {
            note_address,
            oscillator_mask,
            note_delay,
            duration,
            window,
        })
    }

    fn load_vsnd_organ_delay(
        &mut self,
        requested_delay: u8,
    ) -> Result<VSoundOrganDelayPatch, VSoundOrganLoadError> {
        let mut delay = requested_delay;
        let mut cursor = VSNDRM1_RDELAY_OFFSET;
        let rdelay_end = VSNDRM1_RDELAY_OFFSET + VSNDRM1_ORGAN_RDELAY_LEN;
        let mut nop_count = 0u8;
        let mut cmp_zero_patch = false;

        loop {
            if delay == 0 {
                break;
            }
            if delay == 3 {
                if cursor + 2 > rdelay_end {
                    return Err(VSoundOrganLoadError::DelayPatchTooLong {
                        delay: requested_delay,
                    });
                }
                self.ram[cursor] = 0x91;
                self.ram[cursor + 1] = 0;
                cursor += 2;
                cmp_zero_patch = true;
                break;
            }
            if cursor + 1 > rdelay_end {
                return Err(VSoundOrganLoadError::DelayPatchTooLong {
                    delay: requested_delay,
                });
            }
            self.ram[cursor] = 1;
            cursor += 1;
            nop_count = nop_count.wrapping_add(1);
            delay = delay.wrapping_sub(2);
        }

        if cursor + 3 > rdelay_end {
            return Err(VSoundOrganLoadError::DelayPatchTooLong {
                delay: requested_delay,
            });
        }
        self.ram[cursor] = 0x7E;
        self.ram[cursor + 1] = (VSNDRM1_ORGAN1_ADDRESS >> 8) as u8;
        self.ram[cursor + 2] = VSNDRM1_ORGAN1_ADDRESS as u8;
        cursor += 3;

        Ok(VSoundOrganDelayPatch {
            requested_delay,
            rdelay_start: VSNDRM1_RDELAY_OFFSET as u16,
            rdelay_end: cursor as u16,
            nop_count,
            cmp_zero_patch,
            jump_address: VSNDRM1_ORGAN1_ADDRESS,
            patch_bytes: self.ram[VSNDRM1_RDELAY_OFFSET..cursor].to_vec(),
        })
    }

    fn run_vsnd_organ_window(&mut self, delay_patch: VSoundOrganDelayPatch) -> VSoundOrganWindow {
        let oscillator_mask = self.ram[VSNDRM1_OSCIL_OFFSET];
        let duration = self.read_direct_word(VSNDRM1_DUR_OFFSET);
        let initial_timer = self.ram[VSNDRM1_TEMPB_OFFSET];
        let mut remaining = duration;
        let mut dac_samples = Vec::with_capacity(usize::from(duration));

        loop {
            let timer = self.ram[VSNDRM1_TEMPB_OFFSET].wrapping_add(1);
            self.ram[VSNDRM1_TEMPB_OFFSET] = timer;
            let sample = ((timer & oscillator_mask).count_ones() as u8) << 4;
            self.emit_vsnd_dac_sample(&mut dac_samples, sample);
            remaining = remaining.wrapping_sub(1);
            if remaining == 0 {
                break;
            }
        }

        VSoundOrganWindow {
            oscillator_mask,
            duration,
            initial_timer,
            final_timer: self.ram[VSNDRM1_TEMPB_OFFSET],
            delay_patch,
            dac_samples,
        }
    }

    /// Source-shaped `VSNDRM1.SRC` `SP1` setup before the infinite `VARI`
    /// playback loop: load `CABSHK`, advance `SP1FLG`, and derive `LOPER`.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L696-L716>.
    pub fn run_vsnd_spinner1_setup(&mut self) -> Result<VSoundSpinner1Setup, VSoundVariLoadError> {
        let spinner_flag = self.advance_vsnd_spinner1_flag();
        self.load_vsnd_spinner1_setup_for_flag(spinner_flag)
    }

    fn run_vsnd_spinner1_command(&mut self) -> Result<VSoundSpinner1Command, VSoundVariLoadError> {
        let setup = self.load_vsnd_spinner1_setup_for_flag(self.ram[VSNDRM1_SP1FLG_OFFSET])?;
        let sweep = self.run_vsnd_vari_start();
        Ok(VSoundSpinner1Command { setup, sweep })
    }

    fn load_vsnd_spinner1_setup_for_flag(
        &mut self,
        spinner_flag: u8,
    ) -> Result<VSoundSpinner1Setup, VSoundVariLoadError> {
        let vari_load = self.load_vsnd_vari_vector(VSNDRM1_SPINNER_VARI_VECTOR_INDEX)?;

        let mut remaining = VSNDRM1_SPINNER_MAX.wrapping_sub(spinner_flag);
        let mut low_period = 0u8;
        while remaining > 20 {
            low_period = low_period.wrapping_add(14);
            remaining = remaining.wrapping_sub(1);
        }
        while remaining != 0 {
            low_period = low_period.wrapping_add(5);
            remaining = remaining.wrapping_sub(1);
        }
        self.ram[VSNDRM1_LOPER_OFFSET] = low_period;

        Ok(VSoundSpinner1Setup {
            vari_load,
            spinner_flag,
            low_period,
        })
    }

    fn advance_vsnd_spinner1_flag(&mut self) -> u8 {
        let mut spinner_flag = self.ram[VSNDRM1_SP1FLG_OFFSET];
        if spinner_flag == VSNDRM1_SPINNER_MAX - 1 {
            spinner_flag = 0;
        }
        spinner_flag = spinner_flag.wrapping_add(1);
        self.ram[VSNDRM1_SP1FLG_OFFSET] = spinner_flag;
        spinner_flag
    }

    /// Source-shaped `VSNDRM1.SRC` `BON2` setup before entering `GWAVE` or the
    /// `GEND50` continuation path.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L721-L727>.
    pub fn run_vsnd_bonus2_setup(&mut self) -> Result<VSoundBonus2Setup, VSoundGWaveLoadError> {
        let bonus2_flag = self.ram[VSNDRM1_B2FLG_OFFSET];
        if bonus2_flag != 0 {
            let gend50_step = self.run_vsnd_gend50()?;
            return Ok(VSoundBonus2Setup::Continued {
                bonus2_flag,
                gend50_step,
            });
        }

        self.ram[VSNDRM1_B2FLG_OFFSET] = 1;
        let gwave_load = self.load_vsnd_gwave_vector(VSNDRM1_BONUS2_GWAVE_VECTOR_INDEX)?;
        Ok(VSoundBonus2Setup::Started {
            gwave_load,
            bonus2_flag: self.ram[VSNDRM1_B2FLG_OFFSET],
        })
    }

    fn run_vsnd_bonus2_command(
        &mut self,
        previous_bonus2_flag: u8,
    ) -> Result<VSoundBonus2Command, VSoundGWaveLoadError> {
        if previous_bonus2_flag != 0 {
            let gend50_step = self.run_vsnd_gend50()?;
            return Ok(VSoundBonus2Command::Continued {
                bonus2_flag: self.ram[VSNDRM1_B2FLG_OFFSET],
                gend50_step,
            });
        }

        let gwave_load = self.load_vsnd_gwave_vector(VSNDRM1_BONUS2_GWAVE_VECTOR_INDEX)?;
        let step = self.run_vsnd_gwave_start()?;
        Ok(VSoundBonus2Command::Started {
            gwave_load,
            bonus2_flag: self.ram[VSNDRM1_B2FLG_OFFSET],
            step,
        })
    }

    /// Source-shaped `VSNDRM1.SRC` `GWAVE` / `GWT4` entry: seed the echo counter,
    /// point `XPLAY` at the current frequency pattern start, and run one
    /// `GPLAY` period. The returned DAC bytes preserve the source waveform byte
    /// order; exact CPU-cycle audio scheduling remains separate work.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L790-L824>.
    pub fn run_vsnd_gwave_start(&mut self) -> Result<VSoundGWaveStep, VSoundGWaveLoadError> {
        let echo_count = self.ram[VSNDRM1_GECHO_OFFSET];
        self.ram[VSNDRM1_GECNT_OFFSET] = echo_count;
        let frequency_address = self.read_direct_word(VSNDRM1_GWFRQ_OFFSET);
        self.write_direct_word(VSNDRM1_XPLAY_OFFSET, frequency_address);
        self.run_vsnd_gwave_period()
    }

    /// Source-shaped `VSNDRM1.SRC` `GPLAY` / `GOUT`: consume the current
    /// frequency byte, write `GPER`, advance `XPLAY`, and expose the waveform
    /// bytes that `STAA SOUND` would output for this period. When `XPLAY` is
    /// already at `FRQEND`, the source branches to `GEND` after writing `GPER`.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L794-L824>.
    pub fn run_vsnd_gwave_period(&mut self) -> Result<VSoundGWaveStep, VSoundGWaveLoadError> {
        let frequency_address = self.read_direct_word(VSNDRM1_XPLAY_OFFSET);
        let frequency = self.vsnd_sound_rom_byte(frequency_address)?;
        let period = frequency.wrapping_add(self.ram[VSNDRM1_FOFSET_OFFSET]);
        self.ram[VSNDRM1_GPER_OFFSET] = period;

        if frequency_address == self.read_direct_word(VSNDRM1_FRQEND_OFFSET) {
            return self.run_vsnd_gend().map(VSoundGWaveStep::Ended);
        }

        let cycle_count = self.ram[VSNDRM1_GCCNT_OFFSET];
        let waveform_cycles = if cycle_count == 0 {
            256
        } else {
            u16::from(cycle_count)
        };
        let next_frequency_address = frequency_address.wrapping_add(1);
        self.write_direct_word(VSNDRM1_XPLAY_OFFSET, next_frequency_address);

        let waveform_address = self.read_direct_word(VSNDRM1_GWFRM_OFFSET);
        let waveform_length = self.current_vsnd_waveform_length(waveform_address)?;
        let wave_ram_start = VSNDRM1_GWTAB_OFFSET;
        let wave_ram_end = wave_ram_start + usize::from(waveform_length);
        let waveform = self.ram[wave_ram_start..wave_ram_end].to_vec();
        let mut dac_samples =
            Vec::with_capacity(usize::from(waveform_length) * usize::from(waveform_cycles));
        for _ in 0..waveform_cycles {
            for sample in &waveform {
                self.emit_vsnd_dac_sample(&mut dac_samples, *sample);
            }
        }

        Ok(VSoundGWaveStep::Playing(VSoundGWavePeriod {
            frequency_address,
            next_frequency_address,
            period,
            cycle_count,
            waveform_cycles,
            wave_ram_start: VSNDRM1_GWTAB_OFFSET as u16,
            wave_ram_end: wave_ram_end as u16,
            dac_samples,
        }))
    }

    /// Source-shaped `VSNDRM1.SRC` `GEND`: apply echo decay with `GECDEC`,
    /// then enter `GEND40` to decide whether the GWAVE echo restarts,
    /// terminates, or continues into frequency modulation.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L825-L830>.
    pub fn run_vsnd_gend(&mut self) -> Result<VSoundGEndStep, VSoundGWaveLoadError> {
        self.apply_vsnd_current_wave_decay(self.ram[VSNDRM1_GECDEC_OFFSET])?;
        self.run_vsnd_gend40()
    }

    /// Source-shaped `VSNDRM1.SRC` `GEND40`: decrement `GECNT`; non-zero
    /// restarts `GWT4`, zero with active `B2FLG` terminates bonus playback, and
    /// zero without bonus falls into `GEND50`.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L827-L831>.
    pub fn run_vsnd_gend40(&mut self) -> Result<VSoundGEndStep, VSoundGWaveLoadError> {
        let echo_count = self.ram[VSNDRM1_GECNT_OFFSET].wrapping_sub(1);
        self.ram[VSNDRM1_GECNT_OFFSET] = echo_count;
        if echo_count != 0 {
            return Ok(VSoundGEndStep::EchoRestart { echo_count });
        }

        let bonus2_flag = self.ram[VSNDRM1_B2FLG_OFFSET];
        if bonus2_flag != 0 {
            return Ok(VSoundGEndStep::BonusStopped { bonus2_flag });
        }

        Ok(VSoundGEndStep::Frequency(self.run_vsnd_gend50()?))
    }

    /// Source-shaped `VSNDRM1.SRC` `GEND50`: continue frequency-modulated
    /// sounds by decrementing `GDCNT`, adding `GDFINC` into `FOFSET`, and
    /// falling through the shared `GEND60` / `GEND61` frequency-window update.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L831-L867>.
    pub fn run_vsnd_gend50(&mut self) -> Result<VSoundGEnd50Step, VSoundGWaveLoadError> {
        let frequency_delta = self.ram[VSNDRM1_GDFINC_OFFSET];
        if frequency_delta == 0 {
            return Ok(VSoundGEnd50Step::NoFrequencyDelta { frequency_delta });
        }

        let frequency_delta_count = self.ram[VSNDRM1_GDCNT_OFFSET].wrapping_sub(1);
        self.ram[VSNDRM1_GDCNT_OFFSET] = frequency_delta_count;
        if frequency_delta_count == 0 {
            return Ok(VSoundGEnd50Step::DeltaCountExpired {
                frequency_delta_count,
            });
        }

        let frequency_offset = frequency_delta.wrapping_add(self.ram[VSNDRM1_FOFSET_OFFSET]);
        Ok(VSoundGEnd50Step::Updated(
            self.run_vsnd_gend60(frequency_offset)?,
        ))
    }

    /// Source-shaped `VSNDRM1.SRC` `BG2` setup: load `TRBV`, derive `FOFSET`
    /// from `BG2FLG`, and run the shared `GEND60` / `GEND61` frequency-window
    /// update before the repeating background loop. Full `GWAVE` DAC scheduling
    /// remains separate work.
    /// Source: <https://github.com/historicalsource/williams-soundroms/blob/main/VSNDRM1.SRC#L681-L692>.
    pub fn run_vsnd_background2_setup(
        &mut self,
    ) -> Result<VSoundBackground2Setup, VSoundGWaveLoadError> {
        let gwave_load = self.load_vsnd_gwave_vector(VSNDRM1_BACKGROUND2_GWAVE_VECTOR_INDEX)?;
        let background2_flag = self.ram[VSNDRM1_BG2FLG_OFFSET];
        let frequency_offset = !background2_flag.wrapping_shl(2);
        let frequency_update = self.run_vsnd_gend60(frequency_offset)?;

        Ok(VSoundBackground2Setup {
            gwave_load,
            background2_flag,
            frequency_update,
        })
    }

    fn run_vsnd_gend60(
        &mut self,
        frequency_offset: u8,
    ) -> Result<VSoundGEndUpdate, VSoundGWaveLoadError> {
        self.ram[VSNDRM1_FOFSET_OFFSET] = frequency_offset;
        self.run_vsnd_gend61()
    }

    fn run_vsnd_gend61(&mut self) -> Result<VSoundGEndUpdate, VSoundGWaveLoadError> {
        let original_frequency_end = self.read_direct_word(VSNDRM1_FRQEND_OFFSET);
        let mut frequency_address = self.read_direct_word(VSNDRM1_GWFRQ_OFFSET);
        let mut found_start = false;

        while frequency_address != original_frequency_end {
            let frequency_offset = self.ram[VSNDRM1_FOFSET_OFFSET];
            let pattern_byte = self.vsnd_sound_rom_byte(frequency_address)?;
            let (wrapped_frequency, carry) = frequency_offset.overflowing_add(pattern_byte);
            let overflow = if self.ram[VSNDRM1_GDFINC_OFFSET] & 0x80 == 0 {
                carry
            } else {
                wrapped_frequency == 0 || !carry
            };

            if overflow {
                if found_start {
                    return self.finish_vsnd_gend61(frequency_address);
                }
            } else if !found_start {
                self.write_direct_word(VSNDRM1_GWFRQ_OFFSET, frequency_address);
                found_start = true;
            }

            frequency_address = frequency_address.wrapping_add(1);
        }

        if found_start {
            self.finish_vsnd_gend61(frequency_address)
        } else {
            Ok(VSoundGEndUpdate {
                frequency_offset: self.ram[VSNDRM1_FOFSET_OFFSET],
                frequency_pattern_address: self.read_direct_word(VSNDRM1_GWFRQ_OFFSET),
                frequency_end_address: self.read_direct_word(VSNDRM1_FRQEND_OFFSET),
                result: VSoundGEnd61Result::AllOver,
            })
        }
    }

    fn finish_vsnd_gend61(
        &mut self,
        frequency_end_address: u16,
    ) -> Result<VSoundGEndUpdate, VSoundGWaveLoadError> {
        self.write_direct_word(VSNDRM1_FRQEND_OFFSET, frequency_end_address);
        let waveform_reloaded = self.ram[VSNDRM1_GECDEC_OFFSET] != 0;
        if waveform_reloaded {
            let waveform_address = self.read_direct_word(VSNDRM1_GWFRM_OFFSET);
            let waveform_length = self.vsnd_sound_rom_byte(waveform_address)?;
            self.ensure_vsnd_waveform_length(waveform_address, waveform_length)?;
            self.copy_vsnd_waveform_to_ram(waveform_address, waveform_length)?;
            self.apply_vsnd_wave_decay(
                waveform_address,
                waveform_length,
                self.ram[VSNDRM1_PRDECA_OFFSET],
            )?;
        }

        Ok(VSoundGEndUpdate {
            frequency_offset: self.ram[VSNDRM1_FOFSET_OFFSET],
            frequency_pattern_address: self.read_direct_word(VSNDRM1_GWFRQ_OFFSET),
            frequency_end_address: self.read_direct_word(VSNDRM1_FRQEND_OFFSET),
            result: VSoundGEnd61Result::RestartGWave { waveform_reloaded },
        })
    }

    fn apply_vsound_special_flag_effects(&mut self, routine: VSoundIrqRoutine) {
        let VSoundIrqRoutine::Special { routine, .. } = routine else {
            return;
        };

        match routine {
            VSoundSpecialRoutine::Spinner1 => {
                self.advance_vsnd_spinner1_flag();
            }
            VSoundSpecialRoutine::Background1 => {
                self.ram[VSNDRM1_BG1FLG_OFFSET] = 1;
            }
            VSoundSpecialRoutine::Background2Increment => {
                self.ram[VSNDRM1_BG1FLG_OFFSET] = 0;
                let mut background2 = self.ram[VSNDRM1_BG2FLG_OFFSET] & 0x7F;
                if background2 == VSNDRM1_BG2_MAX {
                    background2 = 0;
                }
                self.ram[VSNDRM1_BG2FLG_OFFSET] = background2.wrapping_add(1);
            }
            VSoundSpecialRoutine::Bonus2 => {
                if self.ram[VSNDRM1_B2FLG_OFFSET] == 0 {
                    self.ram[VSNDRM1_B2FLG_OFFSET] = 1;
                }
            }
            VSoundSpecialRoutine::BackgroundEnd => {
                self.ram[VSNDRM1_BG1FLG_OFFSET] = 0;
                self.ram[VSNDRM1_BG2FLG_OFFSET] = 0;
            }
            VSoundSpecialRoutine::OrganTune => {
                self.ram[VSNDRM1_ORGFLG_OFFSET] = self.ram[VSNDRM1_ORGFLG_OFFSET].wrapping_sub(1);
            }
            VSoundSpecialRoutine::OrganNote => {
                self.ram[VSNDRM1_ORGFLG_OFFSET] = 3;
            }
            VSoundSpecialRoutine::Lite
            | VSoundSpecialRoutine::Turbo
            | VSoundSpecialRoutine::Appear
            | VSoundSpecialRoutine::Thrust
            | VSoundSpecialRoutine::Cannon
            | VSoundSpecialRoutine::Radio
            | VSoundSpecialRoutine::Hyper
            | VSoundSpecialRoutine::Scream => {}
        }
    }

    fn vsound_background_continuation(&mut self) -> VSoundBackgroundContinuation {
        if self.ram[VSNDRM1_BG1FLG_OFFSET] | self.ram[VSNDRM1_BG2FLG_OFFSET] == 0 {
            return VSoundBackgroundContinuation::WaitingForBackground;
        }

        self.ram[VSNDRM1_B2FLG_OFFSET] = 0;
        if self.ram[VSNDRM1_BG1FLG_OFFSET] != 0 {
            VSoundBackgroundContinuation::Background1
        } else {
            VSoundBackgroundContinuation::Background2
        }
    }

    fn vsnd_nmi_checksum(&self) -> Result<VSoundNmiChecksum, VSoundNmiDiagnosticError> {
        let mut accumulator = 0;
        let mut carry = false;
        let mut address = VSNDRM1_NMI_LAST_SUMMED_ADDRESS;
        loop {
            let byte = self.vsnd_nmi_sound_rom_byte(address)?;
            (accumulator, carry) = add_with_carry(accumulator, byte, carry);
            if address == VSNDRM1_NMI_FIRST_SUMMED_ADDRESS {
                break;
            }
            address = address.wrapping_sub(1);
        }

        Ok(VSoundNmiChecksum {
            stack_top: VSNDRM1_STACK_TOP,
            checksum_address: VSNDRM1_NMI_CHECKSUM_ADDRESS,
            first_summed_address: VSNDRM1_NMI_FIRST_SUMMED_ADDRESS,
            last_summed_address: VSNDRM1_NMI_LAST_SUMMED_ADDRESS,
            computed_checksum: accumulator,
            expected_checksum: self.vsnd_nmi_sound_rom_byte(VSNDRM1_NMI_CHECKSUM_ADDRESS)?,
        })
    }

    fn vsnd_nmi_sound_rom_byte(&self, address: u16) -> Result<u8, VSoundNmiDiagnosticError> {
        self.roms
            .sound_cpu_byte(address)
            .ok_or(VSoundNmiDiagnosticError::MissingSoundRomByte { address })
    }

    fn copy_vsnd_waveform_to_ram(
        &mut self,
        waveform_address: u16,
        waveform_length: u8,
    ) -> Result<(), VSoundGWaveLoadError> {
        let wave_ram_end = VSNDRM1_GWTAB_OFFSET + usize::from(waveform_length);
        for ram_offset in VSNDRM1_GWTAB_OFFSET..wave_ram_end {
            let waveform_offset = u16::try_from(ram_offset - VSNDRM1_GWTAB_OFFSET)
                .expect("waveform RAM offset fits in u16");
            self.ram[ram_offset] =
                self.vsnd_sound_rom_byte(waveform_address + 1 + waveform_offset)?;
        }
        self.write_direct_word(VSNDRM1_WVEND_OFFSET, wave_ram_end as u16);
        Ok(())
    }

    fn apply_vsnd_current_wave_decay(
        &mut self,
        decay_factor: u8,
    ) -> Result<(), VSoundGWaveLoadError> {
        if decay_factor == 0 {
            return Ok(());
        }

        let waveform_address = self.read_direct_word(VSNDRM1_GWFRM_OFFSET);
        let waveform_length = self.current_vsnd_waveform_length(waveform_address)?;
        self.apply_vsnd_wave_decay(waveform_address, waveform_length, decay_factor)
    }

    fn current_vsnd_waveform_length(
        &self,
        waveform_address: u16,
    ) -> Result<u8, VSoundGWaveLoadError> {
        let wave_ram_end = self.read_direct_word(VSNDRM1_WVEND_OFFSET);
        if wave_ram_end < VSNDRM1_GWTAB_OFFSET as u16 || usize::from(wave_ram_end) > self.ram.len()
        {
            return Err(VSoundGWaveLoadError::InvalidWaveRamEnd { wave_ram_end });
        }

        let waveform_length = u8::try_from(wave_ram_end - VSNDRM1_GWTAB_OFFSET as u16)
            .expect("sound CPU direct-page RAM length fits in u8");
        self.ensure_vsnd_waveform_length(waveform_address, waveform_length)?;
        Ok(waveform_length)
    }

    fn ensure_vsnd_waveform_length(
        &self,
        waveform_address: u16,
        waveform_length: u8,
    ) -> Result<u16, VSoundGWaveLoadError> {
        if waveform_length > VSNDRM1_WVELEN {
            return Err(VSoundGWaveLoadError::WaveformTooLong {
                address: waveform_address,
                length: waveform_length,
            });
        }

        let wave_ram_end = VSNDRM1_GWTAB_OFFSET as u16 + u16::from(waveform_length);
        Ok(wave_ram_end)
    }

    fn apply_vsnd_wave_decay(
        &mut self,
        waveform_address: u16,
        waveform_length: u8,
        decay_factor: u8,
    ) -> Result<(), VSoundGWaveLoadError> {
        if decay_factor == 0 {
            return Ok(());
        }

        for wave_index in 0..usize::from(waveform_length) {
            let source_sample = self.vsnd_sound_rom_byte(
                waveform_address + 1 + u16::try_from(wave_index).expect("wave index fits in u16"),
            )?;
            let decay_amount = source_sample >> 4;
            let sample = &mut self.ram[VSNDRM1_GWTAB_OFFSET + wave_index];
            for _ in 0..decay_factor {
                *sample = sample.wrapping_sub(decay_amount);
            }
        }
        Ok(())
    }

    fn current_vsnd_dac_sample(&self) -> u8 {
        self.last_dac_value.unwrap_or(0)
    }

    fn decay_vsnd_filtered_noise_frequency(&mut self) {
        let frequency =
            u16::from_be_bytes([self.ram[VSNDRM1_FMAX_OFFSET], self.ram[VSNDRM1_FLO_OFFSET]]);
        let decayed = frequency.wrapping_sub(frequency >> 3);
        let [frequency_high, frequency_low] = decayed.to_be_bytes();
        self.ram[VSNDRM1_FMAX_OFFSET] = frequency_high;
        self.ram[VSNDRM1_FLO_OFFSET] = frequency_low;
    }

    fn complement_vsnd_dac_sample(&mut self, dac_samples: &mut Vec<u8>) {
        self.emit_vsnd_dac_sample(dac_samples, !self.current_vsnd_dac_sample());
    }

    fn emit_vsnd_dac_sample(&mut self, dac_samples: &mut Vec<u8>, sample: u8) {
        self.last_dac_value = Some(sample);
        dac_samples.push(sample);
    }

    fn advance_vsnd_random_seed(&mut self) -> bool {
        let low_seed = self.ram[VSNDRM1_LO_OFFSET];
        let mut accumulator = low_seed;
        accumulator >>= 1;
        accumulator >>= 1;
        accumulator >>= 1;
        accumulator ^= low_seed;
        let feedback = accumulator & 0x01 != 0;

        let (high_seed, carry) = rotate_right_through_carry(self.ram[VSNDRM1_HI_OFFSET], feedback);
        self.ram[VSNDRM1_HI_OFFSET] = high_seed;
        let (low_seed, carry) = rotate_right_through_carry(self.ram[VSNDRM1_LO_OFFSET], carry);
        self.ram[VSNDRM1_LO_OFFSET] = low_seed;
        carry
    }

    fn advance_vsnd_filtered_noise_seed(&mut self, sample: u8) {
        let feedback = ((sample >> 3) ^ self.ram[VSNDRM1_LO_OFFSET]) & 0x01 != 0;

        let (high_seed, carry) = rotate_right_through_carry(self.ram[VSNDRM1_HI_OFFSET], feedback);
        self.ram[VSNDRM1_HI_OFFSET] = high_seed;
        let (low_seed, _) = rotate_right_through_carry(self.ram[VSNDRM1_LO_OFFSET], carry);
        self.ram[VSNDRM1_LO_OFFSET] = low_seed;
    }

    fn vsnd_waveform_address(&self, waveform_index: u8) -> Result<u16, VSoundGWaveLoadError> {
        let mut address = VSNDRM1_GWVTAB_ADDRESS;
        for _ in 0..waveform_index {
            let waveform_length = self.vsnd_sound_rom_byte(address)?;
            address = address.wrapping_add(1 + u16::from(waveform_length));
        }
        Ok(address)
    }

    fn vsnd_sound_rom_bytes<const N: usize>(
        &self,
        address: u16,
    ) -> Result<[u8; N], VSoundGWaveLoadError> {
        let mut bytes = [0; N];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = self.vsnd_sound_rom_byte(
                address + u16::try_from(index).expect("small ROM byte index fits in u16"),
            )?;
        }
        Ok(bytes)
    }

    fn vsnd_sound_rom_byte(&self, address: u16) -> Result<u8, VSoundGWaveLoadError> {
        self.roms
            .sound_cpu_byte(address)
            .ok_or(VSoundGWaveLoadError::MissingSoundRomByte { address })
    }

    fn vsnd_organ_sound_rom_byte(&self, address: u16) -> Result<u8, VSoundOrganLoadError> {
        self.roms
            .sound_cpu_byte(address)
            .ok_or(VSoundOrganLoadError::MissingSoundRomByte { address })
    }

    fn vsnd_organ_sound_rom_word(&self, address: u16) -> Result<u16, VSoundOrganLoadError> {
        Ok(u16::from_be_bytes([
            self.vsnd_organ_sound_rom_byte(address)?,
            self.vsnd_organ_sound_rom_byte(address.wrapping_add(1))?,
        ]))
    }

    fn vsnd_vari_rom_bytes<const N: usize>(
        &self,
        address: u16,
    ) -> Result<[u8; N], VSoundVariLoadError> {
        let mut bytes = [0; N];
        for (index, byte) in bytes.iter_mut().enumerate() {
            let byte_address =
                address + u16::try_from(index).expect("small ROM byte index fits in u16");
            *byte = self.roms.sound_cpu_byte(byte_address).ok_or(
                VSoundVariLoadError::MissingSoundRomByte {
                    address: byte_address,
                },
            )?;
        }
        Ok(bytes)
    }

    fn write_direct_word(&mut self, offset: usize, value: u16) {
        let [high, low] = value.to_be_bytes();
        self.ram[offset] = high;
        self.ram[offset + 1] = low;
    }

    fn read_direct_word(&self, offset: usize) -> u16 {
        u16::from_be_bytes([self.ram[offset], self.ram[offset + 1]])
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

fn add_with_carry(accumulator: u8, value: u8, carry: bool) -> (u8, bool) {
    let sum = u16::from(accumulator) + u16::from(value) + u16::from(carry);
    (sum as u8, sum > 0xFF)
}

pub fn vsnd_irq_routine(sound_code: u8) -> VSoundIrqRoutine {
    if sound_code == 0 {
        return VSoundIrqRoutine::Invalid;
    }

    let table_index = sound_code.wrapping_sub(1);
    if table_index <= 0x0C {
        return VSoundIrqRoutine::GWave {
            sound_code,
            table_index,
        };
    }

    if table_index <= 0x1B {
        return VSoundIrqRoutine::Special {
            sound_code,
            table_index: table_index - 0x0D,
            routine: VSNDRM1_SPECIAL_ROUTINES[usize::from(table_index - 0x0D)],
        };
    }

    VSoundIrqRoutine::Vari {
        sound_code,
        table_index: table_index - 0x1C,
    }
}

fn rotate_right_through_carry(value: u8, carry_in: bool) -> (u8, bool) {
    let carry_out = value & 0x01 != 0;
    ((value >> 1) | (u8::from(carry_in) << 7), carry_out)
}

const VSNDRM1_SPECIAL_ROUTINES: [VSoundSpecialRoutine; 15] = [
    VSoundSpecialRoutine::Spinner1,
    VSoundSpecialRoutine::Background1,
    VSoundSpecialRoutine::Background2Increment,
    VSoundSpecialRoutine::Lite,
    VSoundSpecialRoutine::Bonus2,
    VSoundSpecialRoutine::BackgroundEnd,
    VSoundSpecialRoutine::Turbo,
    VSoundSpecialRoutine::Appear,
    VSoundSpecialRoutine::Thrust,
    VSoundSpecialRoutine::Cannon,
    VSoundSpecialRoutine::Radio,
    VSoundSpecialRoutine::Hyper,
    VSoundSpecialRoutine::Scream,
    VSoundSpecialRoutine::OrganTune,
    VSoundSpecialRoutine::OrganNote,
];

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
            VerifiedRomSet, crc32,
        },
        sound::{
            DefenderSoundBoard, SOUND_CPU_INTERNAL_RAM_SIZE, SOUND_PIA_UNCONNECTED_PORT_A_INPUT,
            SoundCommand, SoundCommandLatch, SoundCpuAddressTarget, SoundCpuReadError,
            SoundCpuWriteError, VSNDRM1_BG2_MAX, VSNDRM1_BONUS2_SOUND_CODE, VSNDRM1_SPINNER_MAX,
            VSNDRM1_SPINNER_SOUND_CODE, VSOUND_SOURCE_DAC_TICK_STEP, VSoundBackground2Setup,
            VSoundBackgroundContinuation, VSoundBackgroundFlags, VSoundBonus2Command,
            VSoundBonus2Setup, VSoundFilteredNoise, VSoundFilteredNoiseWindow, VSoundGEnd50Step,
            VSoundGEnd61Result, VSoundGEndStep, VSoundGWaveLoad, VSoundGWaveLoadError,
            VSoundGWavePeriod, VSoundGWaveStep, VSoundHyperSweep, VSoundIrq3AfterCommand,
            VSoundIrq3BackgroundFlow, VSoundIrq3Handoff, VSoundIrqCommandAndBackgroundFlowError,
            VSoundIrqCommandFlow, VSoundIrqCommandFlowError, VSoundIrqDispatch, VSoundIrqFlow,
            VSoundIrqFlowError, VSoundIrqOrganFlow, VSoundIrqPrelude, VSoundIrqRoutine,
            VSoundIrqRunningRoutine, VSoundIrqSpecialFlow, VSoundLiteNoise, VSoundNmiChecksum,
            VSoundNmiDiagnosticCycle, VSoundNmiDiagnosticError, VSoundOrganIrqStep,
            VSoundOrganLoadError, VSoundOrganNoteStart, VSoundOrganNoteStep, VSoundOrganTuneStart,
            VSoundOrganTuneStep, VSoundRadioWave, VSoundScream, VSoundSpecialRoutine,
            VSoundSpinner1Command, VSoundTurboNoise, VSoundVariLoad, VSoundVariLoadError,
            VSoundVariSweep, VSoundVariSweepResult, format_sound_command_list,
            sound_cpu_address_target, vsnd_irq_routine,
        },
    };

    fn test_rom_images() -> RedLabelRomImages {
        let mut sound = vec![0; 0x0800];
        sound[0] = 0xF8;
        sound[1] = 0x01;
        sound[0x07FF] = 0xFF;

        test_rom_images_from_sound(sound)
    }

    fn test_rom_images_without_sound_load() -> RedLabelRomImages {
        let rom_set = VerifiedRomSet::from_files_for_test(Vec::new());
        let regions = [RomRegion {
            name: "soundcpu",
            size: 0x10000,
            source: "test",
        }];

        RedLabelRomImages::from_parts_for_test(&rom_set, &regions, &[])
            .expect("test sound ROM image without loads should build")
    }

    fn test_rom_images_with_vsnd_tables() -> RedLabelRomImages {
        let mut sound = vec![0; 0x0800];
        sound[0] = 0xF8;
        sound[1] = 0x01;
        sound[0x07FF] = 0xFF;

        write_test_sound_rom(
            &mut sound,
            0xFD76,
            &[
                0x40, 0x01, 0x00, 0x10, 0xE1, 0x00, 0x80, 0xFF, 0xFF, 0x28, 0x01, 0x00, 0x08, 0x81,
                0x02, 0x00, 0xFF, 0xFF, 0x28, 0x81, 0x00, 0xFC, 0x01, 0x02, 0x00, 0xFC, 0xFF, 0xFF,
                0x01, 0x00, 0x18, 0x41, 0x04, 0x80, 0x00, 0xFF,
            ],
        );
        write_test_sound_rom(
            &mut sound,
            0xFD9A,
            &[
                0x8C, 0x5B, 0xB6, 0x40, 0xBF, 0x49, 0xA4, 0x73, 0x73, 0xA4, 0x49, 0xBF, 0x40, 0xB6,
                0x5B, 0x8C,
            ],
        );
        write_test_sound_rom(
            &mut sound,
            0xFDAA,
            &[
                0x0C, 0x7F, 0x1D, 0x0F, 0xFB, 0x7F, 0x23, 0x0F, 0x15, 0xFE, 0x08, 0x50, 0x8B, 0x88,
                0x3E, 0x3F, 0x02, 0x3E, 0x7C, 0x04, 0x03, 0xFF, 0x3E, 0x3F, 0x2C, 0xE2, 0x7C, 0x12,
                0x0D, 0x74, 0x7C, 0x0D, 0x0E, 0x41, 0x7C, 0x23, 0x0B, 0x50, 0x7C, 0x1D, 0x29, 0xF2,
                0x7C, 0x3F, 0x02, 0x3E, 0xF8, 0x04, 0x03, 0xFF, 0x7C, 0x3F, 0x2C, 0xE2, 0xF8, 0x12,
                0x0D, 0x74, 0xF8, 0x0D, 0x0E, 0x41, 0xF8, 0x23, 0x0B, 0x50, 0xF8, 0x1D, 0x2F, 0xF2,
                0xF8, 0x23, 0x05, 0xA8, 0xF8, 0x12, 0x06, 0xBA, 0xF8, 0x04, 0x07, 0xFF, 0x7C, 0x37,
                0x04, 0xC1, 0x7C, 0x23, 0x05, 0xA8, 0x7C, 0x12, 0x06, 0xBA, 0x3E, 0x04, 0x07, 0xFF,
                0x3E, 0x37, 0x04, 0xC1, 0x3E, 0x23, 0x05, 0xA8, 0x1F, 0x12, 0x06, 0xBA, 0x1F, 0x04,
                0x07, 0xFF, 0x1F, 0x37, 0x04, 0xC1, 0x1F, 0x23, 0x16, 0xA0, 0xFE, 0x1D, 0x17, 0xF9,
                0x7F, 0x37, 0x13, 0x06, 0x7F, 0x3F, 0x08, 0xFA, 0xFE, 0x04, 0x0F, 0xFF, 0xFE, 0x0D,
                0x0E, 0x41, 0xFE, 0x23, 0x0B, 0x50, 0xFE, 0x1D, 0x5F, 0xE4, 0x00,
            ],
        );
        write_test_sound_rom(
            &mut sound,
            0xFE41,
            &[
                0x47, 0x3F, 0x37, 0x30, 0x29, 0x23, 0x1D, 0x17, 0x12, 0x0D, 0x08, 0x04,
            ],
        );
        write_test_sound_rom(&mut sound, 0xFE4D, &[8, 127, 217, 255, 217, 127, 36, 0, 36]);
        write_test_sound_rom(&mut sound, 0xFE56, &[8, 0, 64, 128, 0, 255, 0, 128, 64]);
        write_test_sound_rom(
            &mut sound,
            0xFE5F,
            &[
                16, 127, 176, 217, 245, 255, 245, 217, 176, 127, 78, 36, 9, 0, 9, 36, 78,
            ],
        );
        write_test_sound_rom(
            &mut sound,
            0xFE70,
            &[
                16, 127, 197, 236, 231, 191, 141, 109, 106, 127, 148, 146, 113, 64, 23, 18, 57,
            ],
        );
        write_test_sound_rom(
            &mut sound,
            0xFE81,
            &[
                16, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0,
            ],
        );
        write_test_sound_rom(&mut sound, 0xFE92, &[72]);
        write_test_sound_rom(
            &mut sound,
            0xFEDB,
            &[
                16, 89, 123, 152, 172, 179, 172, 152, 123, 89, 55, 25, 6, 0, 6, 25, 55,
            ],
        );

        write_test_sound_rom(&mut sound, 0xFEEC, &[0x81, 0x24, 0, 0, 0, 22, 0x31]);
        write_test_sound_rom(&mut sound, 0xFF32, &[0xF6, 0x53, 3, 0, 2, 6, 0x94]);
        write_test_sound_rom(&mut sound, 0xFF47, &[0x31, 0x11, 0, 0xFF, 0, 13, 0]);
        write_test_sound_rom(&mut sound, 0xFF4E, &[0x12, 0x06, 0, 0xFF, 1, 9, 0x28]);
        write_test_sound_rom(
            &mut sound,
            0xFF55,
            &[
                0xA0, 0x98, 0x90, 0x88, 0x80, 0x78, 0x70, 0x68, 0x60, 0x58, 0x50, 0x44, 0x40,
            ],
        );
        write_test_sound_rom(
            &mut sound,
            0xFF7D,
            &[0x80, 0x7C, 0x78, 0x74, 0x70, 0x74, 0x78, 0x7C, 0x80],
        );

        test_rom_images_from_sound(sound)
    }

    fn test_rom_images_with_organ_note_delay(note_delay: u8) -> RedLabelRomImages {
        let mut sound = vec![0; 0x0800];
        sound[0] = 0xF8;
        sound[1] = 0x01;
        sound[0x07FF] = 0xFF;
        write_test_sound_rom(&mut sound, 0xFE41, &[note_delay]);

        test_rom_images_from_sound(sound)
    }

    fn test_rom_images_with_organ_tune_table(table: &[u8]) -> RedLabelRomImages {
        let mut sound = vec![0; 0x0800];
        sound[0] = 0xF8;
        sound[1] = 0x01;
        sound[0x07FF] = 0xFF;
        write_test_sound_rom(&mut sound, 0xFDAA, table);

        test_rom_images_from_sound(sound)
    }

    fn test_rom_images_with_nmi_diagnostic_checksum(checksum: u8) -> RedLabelRomImages {
        let mut sound = vec![0; 0x0800];
        sound[0] = checksum;
        write_test_sound_rom(
            &mut sound,
            0xFD7F,
            &[0x28, 0x01, 0x00, 0x08, 0x81, 0x02, 0x00, 0xFF, 0xFF],
        );

        test_rom_images_from_sound(sound)
    }

    fn write_test_sound_rom(sound: &mut [u8], address: u16, bytes: &[u8]) {
        let start = usize::from(address - 0xF800);
        let end = start + bytes.len();
        sound[start..end].copy_from_slice(bytes);
    }

    fn test_rom_images_from_sound(sound: Vec<u8>) -> RedLabelRomImages {
        let sound_size = sound.len();
        let rom_set = VerifiedRomSet::from_files_for_test(vec![VerifiedRomFile {
            descriptor: RomDescriptor {
                name: "sound.rom",
                size: sound_size as u64,
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
            size: sound_size,
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

    fn raw_for_vsound_code(sound_code: u8) -> u8 {
        (!sound_code) & 0x1F
    }

    fn organ_note_started(
        step: VSoundOrganNoteStep,
    ) -> Option<crate::sound::VSoundOrganNoteWindow> {
        match step {
            VSoundOrganNoteStep::NoteStarted(note) => Some(note),
            VSoundOrganNoteStep::MaskByte(_) => None,
        }
    }

    fn organ_tune_played(step: VSoundOrganTuneStep) -> Option<crate::sound::VSoundOrganTune> {
        match step {
            VSoundOrganTuneStep::Played(tune) => Some(tune),
            VSoundOrganTuneStep::InvalidTune { .. } => None,
        }
    }

    fn irq_organ_tune_started(flow: VSoundIrqOrganFlow) -> Option<(u8, u8, VSoundOrganTuneStep)> {
        match flow {
            VSoundIrqOrganFlow::Tune {
                latched_port_b,
                command_code,
                tune,
            } => Some((latched_port_b, command_code, tune)),
            VSoundIrqOrganFlow::Inactive { .. } | VSoundIrqOrganFlow::Note { .. } => None,
        }
    }

    fn irq_organ_note_step(flow: VSoundIrqOrganFlow) -> Option<(u8, u8, u8, VSoundOrganNoteStep)> {
        match flow {
            VSoundIrqOrganFlow::Note {
                latched_port_b,
                command_code,
                parameter_code,
                step,
            } => Some((latched_port_b, command_code, parameter_code, step)),
            VSoundIrqOrganFlow::Inactive { .. } | VSoundIrqOrganFlow::Tune { .. } => None,
        }
    }

    fn irq_command_special_step(
        flow: VSoundIrqCommandFlow,
    ) -> Option<(VSoundIrqDispatch, VSoundIrqSpecialFlow)> {
        match flow {
            VSoundIrqCommandFlow::Special { dispatch, step } => Some((dispatch, step)),
            VSoundIrqCommandFlow::Invalid { .. }
            | VSoundIrqCommandFlow::GWave { .. }
            | VSoundIrqCommandFlow::Vari { .. } => None,
        }
    }

    fn spinner1_special_command(step: VSoundIrqSpecialFlow) -> Option<VSoundSpinner1Command> {
        match step {
            VSoundIrqSpecialFlow::Spinner1(command) => Some(command),
            _ => None,
        }
    }

    fn bonus2_special_command(step: VSoundIrqSpecialFlow) -> Option<VSoundBonus2Command> {
        match step {
            VSoundIrqSpecialFlow::Bonus2(command) => Some(command),
            _ => None,
        }
    }

    fn organ_tune_special_start(step: VSoundIrqSpecialFlow) -> Option<VSoundOrganTuneStart> {
        match step {
            VSoundIrqSpecialFlow::OrganTune(start) => Some(start),
            _ => None,
        }
    }

    fn organ_note_special_start(step: VSoundIrqSpecialFlow) -> Option<VSoundOrganNoteStart> {
        match step {
            VSoundIrqSpecialFlow::OrganNote(start) => Some(start),
            _ => None,
        }
    }

    fn lite_special_noise(step: VSoundIrqSpecialFlow) -> Option<VSoundLiteNoise> {
        match step {
            VSoundIrqSpecialFlow::Lite(noise) => Some(noise),
            _ => None,
        }
    }

    fn background1_special_window(step: VSoundIrqSpecialFlow) -> Option<VSoundFilteredNoiseWindow> {
        match step {
            VSoundIrqSpecialFlow::Background1(noise) => Some(noise),
            _ => None,
        }
    }

    fn background2_increment_special_setup(
        step: VSoundIrqSpecialFlow,
    ) -> Option<VSoundBackground2Setup> {
        match step {
            VSoundIrqSpecialFlow::Background2Increment(setup) => Some(setup),
            _ => None,
        }
    }

    fn turbo_special_noise(step: VSoundIrqSpecialFlow) -> Option<VSoundTurboNoise> {
        match step {
            VSoundIrqSpecialFlow::Turbo(noise) => Some(noise),
            _ => None,
        }
    }

    fn appear_special_noise(step: VSoundIrqSpecialFlow) -> Option<VSoundLiteNoise> {
        match step {
            VSoundIrqSpecialFlow::Appear(noise) => Some(noise),
            _ => None,
        }
    }

    fn background_end_special_flags(step: VSoundIrqSpecialFlow) -> Option<VSoundBackgroundFlags> {
        match step {
            VSoundIrqSpecialFlow::BackgroundEnd(flags) => Some(flags),
            _ => None,
        }
    }

    fn thrust_special_window(step: VSoundIrqSpecialFlow) -> Option<VSoundFilteredNoiseWindow> {
        match step {
            VSoundIrqSpecialFlow::Thrust(noise) => Some(noise),
            _ => None,
        }
    }

    fn cannon_special_noise(step: VSoundIrqSpecialFlow) -> Option<VSoundFilteredNoise> {
        match step {
            VSoundIrqSpecialFlow::Cannon(noise) => Some(noise),
            _ => None,
        }
    }

    fn radio_special_wave(step: VSoundIrqSpecialFlow) -> Option<VSoundRadioWave> {
        match step {
            VSoundIrqSpecialFlow::Radio(wave) => Some(wave),
            _ => None,
        }
    }

    fn hyper_special_sweep(step: VSoundIrqSpecialFlow) -> Option<VSoundHyperSweep> {
        match step {
            VSoundIrqSpecialFlow::Hyper(sweep) => Some(sweep),
            _ => None,
        }
    }

    fn scream_special(step: VSoundIrqSpecialFlow) -> Option<VSoundScream> {
        match step {
            VSoundIrqSpecialFlow::Scream(scream) => Some(scream),
            _ => None,
        }
    }

    fn minimal_gwave_update(result: VSoundGEnd61Result) -> crate::sound::VSoundGEndUpdate {
        crate::sound::VSoundGEndUpdate {
            frequency_offset: 0,
            frequency_pattern_address: 0,
            frequency_end_address: 0,
            result,
        }
    }

    fn minimal_gwave_load() -> VSoundGWaveLoad {
        VSoundGWaveLoad {
            table_index: 0,
            vector_address: 0,
            echo_count: 0,
            cycle_count: 0,
            echo_decay: 0,
            waveform_index: 0,
            waveform_address: 0,
            waveform_length: 0,
            predecay_factor: 0,
            frequency_delta: 0,
            frequency_delta_count: 0,
            frequency_pattern_address: 0,
            frequency_pattern_length: 0,
            frequency_end_address: 0,
            wave_ram_start: 0,
            wave_ram_end: 0,
        }
    }

    fn minimal_gwave_period() -> VSoundGWavePeriod {
        VSoundGWavePeriod {
            frequency_address: 0,
            next_frequency_address: 0,
            period: 0,
            cycle_count: 0,
            waveform_cycles: 0,
            wave_ram_start: 0,
            wave_ram_end: 0,
            dac_samples: Vec::new(),
        }
    }

    fn minimal_vari_sweep(result: VSoundVariSweepResult) -> VSoundVariSweep {
        VSoundVariSweep {
            sweep_period: 0,
            low_count: 0,
            high_count: 0,
            low_count_after_sweep: 0,
            high_count_after_sweep: 0,
            dac_samples: Vec::new(),
            result,
        }
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
    fn vsnd_setup_initializes_pia_and_source_flags() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0xAA).expect("seed BG1FLG");
        board.write_byte(0x0005, 0xBB).expect("seed BG2FLG");
        board.write_byte(0x0006, 0xCC).expect("seed SP1FLG");
        board.write_byte(0x0007, 0xDD).expect("seed B2FLG");
        board.write_byte(0x0008, 0xEE).expect("seed ORGFLG");

        let setup = board.run_vsnd_setup().expect("VSNDRM1 SETUP");

        assert_eq!(setup.stack_top, 0x7F);
        assert_eq!(setup.pia_ddr_a, 0xFF);
        assert_eq!(setup.pia_ddr_b, 0x00);
        assert_eq!(setup.pia_control_a, 0x3C);
        assert_eq!(setup.pia_control_b, 0x37);
        assert_eq!(setup.random_seed_hi, 0x3C);
        assert_eq!(&board.ram()[0x04..0x09], &[0, 0, 0, 0, 0]);
    }

    #[test]
    fn vsnd_irq_prelude_reads_pia_command_and_clears_cb1_irq() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.run_vsnd_setup().expect("VSNDRM1 SETUP");
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        assert!(board.sound_irq_asserted());
        assert_eq!(board.read_byte(0x0403), Ok(PIA_IRQ1 | 0x37));

        assert_eq!(
            board.run_vsnd_irq_prelude(),
            VSoundIrqPrelude {
                stack_top: 0x7F,
                latched_port_b: 0xDE,
                command_code: 1,
                irq_asserted_before_read: true,
                irq_asserted_after_read: false,
            }
        );
        assert!(!board.sound_irq_asserted());
        assert_eq!(board.read_byte(0x0403), Ok(0x37));
    }

    #[test]
    fn vsnd_irq_prelude_reports_idle_command_without_irq_edge() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.run_vsnd_setup().expect("VSNDRM1 SETUP");
        board.latch_main_board_sound_command(0x3F);

        assert_eq!(
            board.run_vsnd_irq_prelude(),
            VSoundIrqPrelude {
                stack_top: 0x7F,
                latched_port_b: 0xFF,
                command_code: 0,
                irq_asserted_before_read: false,
                irq_asserted_after_read: false,
            }
        );
        assert!(!board.sound_irq_asserted());
    }

    #[test]
    fn vsnd_irq_cycle_runs_pia_prelude_before_command_flow() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.run_vsnd_setup().expect("VSNDRM1 SETUP");
        board.latch_main_board_sound_command(raw_for_vsound_code(0));

        assert!(board.sound_irq_asserted());
        assert_eq!(board.read_byte(0x0403), Ok(PIA_IRQ1 | 0x37));

        let cycle = board.step_vsnd_irq_cycle().expect("invalid IRQ cycle");

        assert_eq!(
            cycle.prelude,
            VSoundIrqPrelude {
                stack_top: 0x7F,
                latched_port_b: 0xDF,
                command_code: 0,
                irq_asserted_before_read: true,
                irq_asserted_after_read: false,
            }
        );
        assert!(matches!(
            cycle.flow,
            VSoundIrqFlow::Command(crate::sound::VSoundIrqCommandAndBackgroundFlow {
                command: VSoundIrqCommandFlow::Invalid {
                    dispatch: VSoundIrqDispatch {
                        latched_port_b: 0xDF,
                        command_code: 0,
                        effective_code: 0,
                        organ_step: VSoundOrganIrqStep::Inactive,
                        talking_program_present: false,
                        routine: VSoundIrqRoutine::Invalid,
                        spinner_flag: 0,
                        bonus2_flag: 0,
                        background1_flag: 0,
                        background2_flag: 0,
                        background: VSoundBackgroundContinuation::WaitingForBackground,
                    },
                },
                irq3: VSoundIrq3AfterCommand::Entered(
                    VSoundIrq3BackgroundFlow::WaitingForBackground
                ),
            })
        ));
        assert!(!board.sound_irq_asserted());
        assert_eq!(board.read_byte(0x0403), Ok(0x37));
    }

    #[test]
    fn vsnd_irq_cycle_clears_pia_irq_before_reporting_command_error() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.run_vsnd_setup().expect("VSNDRM1 SETUP");
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        assert!(board.sound_irq_asserted());

        assert_eq!(
            board.step_vsnd_irq_cycle(),
            Err(VSoundIrqFlowError::Command(
                VSoundIrqCommandAndBackgroundFlowError::Command(VSoundIrqCommandFlowError::GWave(
                    VSoundGWaveLoadError::MissingSoundRomByte { address: 0xFEEC },
                ),),
            ))
        );
        assert!(!board.sound_irq_asserted());
        assert_eq!(board.read_byte(0x0403), Ok(0x37));
    }

    #[test]
    fn vsnd_irq_timed_cycle_consumes_latch_and_reports_dac_window() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.run_vsnd_setup().expect("VSNDRM1 SETUP");
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        assert!(board.sound_irq_asserted());

        let timed = board
            .step_vsnd_irq_timed_cycle(512)
            .expect("timed GWAVE IRQ cycle");

        assert_eq!(timed.irq_tick, 512);
        assert_eq!(
            timed.cycle.prelude,
            VSoundIrqPrelude {
                stack_top: 0x7F,
                latched_port_b: 0xDE,
                command_code: 1,
                irq_asserted_before_read: true,
                irq_asserted_after_read: false,
            }
        );
        assert_eq!(timed.dac.first_tick, Some(512));
        assert_eq!(timed.dac.tick_step, VSOUND_SOURCE_DAC_TICK_STEP);
        assert_eq!(timed.dac.sample_tick(0), Some(512));
        assert_eq!(timed.dac.sample_tick(15), Some(527));
        assert_eq!(timed.dac.sample_tick(16), None);
        assert_eq!(timed.dac.sample_count(), 16);
        assert_eq!(
            timed.dac.dac_samples.as_slice(),
            &[
                0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0,
            ]
        );
        assert_eq!(timed.dac.last_dac_value, Some(0));
        assert_eq!(board.last_dac_value(), Some(0));
        assert!(!timed.irq_asserted_after_cycle);
        assert!(!board.sound_irq_asserted());
        assert_eq!(&board.ram()[0x0D..0x0F], &[0xFF, 0x87]);
        assert_eq!(board.ram()[0x21], 0);
    }

    #[test]
    fn vsnd_nmi_diagnostic_cycle_runs_vari_vector_after_matching_checksum() {
        let images = test_rom_images_with_nmi_diagnostic_checksum(0xB4);
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let cycle = board
            .run_vsnd_nmi_diagnostic_cycle()
            .expect("matching NMI diagnostic checksum");

        assert!(matches!(
            cycle,
            VSoundNmiDiagnosticCycle::ChecksumMatched {
                checksum: VSoundNmiChecksum {
                    stack_top: 0x7F,
                    checksum_address: 0xF800,
                    first_summed_address: 0xF801,
                    last_summed_address: 0xFFFF,
                    computed_checksum: 0xB4,
                    expected_checksum: 0xB4,
                },
                vari_load: VSoundVariLoad {
                    table_index: 1,
                    vector_address: 0xFD7F,
                    low_period: 0x28,
                    high_period: 0x01,
                    low_period_delta: 0x00,
                    high_period_delta: 0x08,
                    high_period_end: 0x81,
                    sweep_period: 0x0200,
                    low_period_mod: 0xFF,
                    amplitude: 0xFF,
                },
                sweep: VSoundVariSweep {
                    sweep_period: 0x0200,
                    low_count: 0x28,
                    high_count: 0x01,
                    low_count_after_sweep: 0x28,
                    high_count_after_sweep: 0x09,
                    ref dac_samples,
                    result: VSoundVariSweepResult::Continue,
                },
                talking_diagnostic_present: false,
            } if dac_samples.len() == 27
                && dac_samples.first() == Some(&0xFF)
                && dac_samples.last() == Some(&0xFF)
        ));
        assert_eq!(board.last_dac_value(), Some(0xFF));
    }

    #[test]
    fn vsnd_nmi_diagnostic_cycle_waits_without_vari_when_checksum_mismatches() {
        let images = test_rom_images_with_nmi_diagnostic_checksum(0x00);
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(
            board.run_vsnd_nmi_diagnostic_cycle(),
            Ok(VSoundNmiDiagnosticCycle::ChecksumMismatch(
                VSoundNmiChecksum {
                    stack_top: 0x7F,
                    checksum_address: 0xF800,
                    first_summed_address: 0xF801,
                    last_summed_address: 0xFFFF,
                    computed_checksum: 0xB4,
                    expected_checksum: 0x00,
                },
            ))
        );
        assert_eq!(&board.ram()[0x13..0x1C], &[0; 9]);
        assert_eq!(board.last_dac_value(), None);
    }

    #[test]
    fn vsnd_nmi_diagnostic_cycle_reports_missing_checksum_rom_byte() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(
            board.run_vsnd_nmi_diagnostic_cycle(),
            Err(VSoundNmiDiagnosticError::MissingSoundRomByte { address: 0xFFFF })
        );
    }

    #[test]
    fn vsnd_irq_routine_decoder_matches_source_ranges() {
        assert_eq!(vsnd_irq_routine(0), VSoundIrqRoutine::Invalid);
        assert_eq!(
            vsnd_irq_routine(1),
            VSoundIrqRoutine::GWave {
                sound_code: 1,
                table_index: 0
            }
        );
        assert_eq!(
            vsnd_irq_routine(13),
            VSoundIrqRoutine::GWave {
                sound_code: 13,
                table_index: 12
            }
        );
        assert_eq!(
            vsnd_irq_routine(14),
            VSoundIrqRoutine::Special {
                sound_code: 14,
                table_index: 0,
                routine: VSoundSpecialRoutine::Spinner1,
            }
        );
        assert_eq!(
            vsnd_irq_routine(28),
            VSoundIrqRoutine::Special {
                sound_code: 28,
                table_index: 14,
                routine: VSoundSpecialRoutine::OrganNote,
            }
        );
        assert_eq!(
            vsnd_irq_routine(29),
            VSoundIrqRoutine::Vari {
                sound_code: 29,
                table_index: 0
            }
        );
    }

    #[test]
    fn vsnd_irq3_handoff_classifies_gwave_and_gend_source_returns() {
        assert_eq!(
            VSoundGWaveStep::Playing(minimal_gwave_period()).irq3_handoff(),
            VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::GWave)
        );
        assert_eq!(
            VSoundGWaveStep::Ended(VSoundGEndStep::EchoRestart { echo_count: 1 }).irq3_handoff(),
            VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::GWave)
        );
        assert_eq!(
            VSoundGWaveStep::Ended(VSoundGEndStep::BonusStopped { bonus2_flag: 1 }).irq3_handoff(),
            VSoundIrq3Handoff::Ready
        );
        assert_eq!(
            VSoundGWaveStep::Ended(VSoundGEndStep::Frequency(
                VSoundGEnd50Step::NoFrequencyDelta { frequency_delta: 0 }
            ))
            .irq3_handoff(),
            VSoundIrq3Handoff::Ready
        );
        assert_eq!(
            VSoundGEnd50Step::DeltaCountExpired {
                frequency_delta_count: 0,
            }
            .irq3_handoff(),
            VSoundIrq3Handoff::Ready
        );
        assert_eq!(
            VSoundGEnd50Step::Updated(minimal_gwave_update(VSoundGEnd61Result::AllOver))
                .irq3_handoff(),
            VSoundIrq3Handoff::Ready
        );
        assert_eq!(
            VSoundGEnd50Step::Updated(minimal_gwave_update(VSoundGEnd61Result::RestartGWave {
                waveform_reloaded: false,
            },))
            .irq3_handoff(),
            VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::GWave)
        );
    }

    #[test]
    fn vsnd_irq3_handoff_classifies_vari_source_returns() {
        assert_eq!(
            VSoundVariSweepResult::Continue.irq3_handoff(),
            VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Vari)
        );
        assert_eq!(
            VSoundVariSweepResult::Restarted { low_period: 0x40 }.irq3_handoff(),
            VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Vari)
        );
        assert_eq!(
            VSoundVariSweepResult::Terminated {
                low_period: 0,
                low_period_mod: 0xFF,
            }
            .irq3_handoff(),
            VSoundIrq3Handoff::Ready
        );
    }

    #[test]
    fn vsnd_irq3_handoff_classifies_bonus2_source_returns() {
        let running_start = VSoundBonus2Command::Started {
            gwave_load: minimal_gwave_load(),
            bonus2_flag: 1,
            step: VSoundGWaveStep::Playing(minimal_gwave_period()),
        };
        assert_eq!(
            running_start.irq3_handoff(),
            VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Bonus2)
        );

        let returned_start = VSoundBonus2Command::Started {
            gwave_load: minimal_gwave_load(),
            bonus2_flag: 1,
            step: VSoundGWaveStep::Ended(VSoundGEndStep::BonusStopped { bonus2_flag: 1 }),
        };
        assert_eq!(returned_start.irq3_handoff(), VSoundIrq3Handoff::Ready);

        let running_continuation = VSoundBonus2Command::Continued {
            bonus2_flag: 1,
            gend50_step: VSoundGEnd50Step::Updated(minimal_gwave_update(
                VSoundGEnd61Result::RestartGWave {
                    waveform_reloaded: true,
                },
            )),
        };
        assert_eq!(
            running_continuation.irq3_handoff(),
            VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Bonus2)
        );

        let returned_continuation = VSoundBonus2Command::Continued {
            bonus2_flag: 1,
            gend50_step: VSoundGEnd50Step::NoFrequencyDelta { frequency_delta: 0 },
        };
        assert_eq!(
            returned_continuation.irq3_handoff(),
            VSoundIrq3Handoff::Ready
        );
    }

    #[test]
    fn vsnd_irq3_handoff_classifies_special_command_source_returns() {
        let images = test_rom_images_with_vsnd_tables();
        let cases = [
            (
                VSNDRM1_SPINNER_SOUND_CODE,
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Spinner1),
            ),
            (
                15,
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Background1),
            ),
            (
                16,
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Background2),
            ),
            (17, VSoundIrq3Handoff::Ready),
            (
                VSNDRM1_BONUS2_SOUND_CODE,
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Bonus2),
            ),
            (19, VSoundIrq3Handoff::Ready),
            (20, VSoundIrq3Handoff::Ready),
            (21, VSoundIrq3Handoff::Ready),
            (
                22,
                VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Thrust),
            ),
            (23, VSoundIrq3Handoff::Ready),
            (24, VSoundIrq3Handoff::Ready),
            (25, VSoundIrq3Handoff::Ready),
            (26, VSoundIrq3Handoff::Ready),
            (27, VSoundIrq3Handoff::Ready),
            (28, VSoundIrq3Handoff::Ready),
        ];

        for (sound_code, expected) in cases {
            let mut board = DefenderSoundBoard::with_cleared_ram(&images);
            board.latch_main_board_sound_command(raw_for_vsound_code(sound_code));
            let (_, step) = irq_command_special_step(
                board
                    .step_vsnd_irq_command_flow()
                    .expect("special flow should run"),
            )
            .expect("special flow should extract");
            assert_eq!(step.irq3_handoff(), expected, "sound code {sound_code}");
        }

        assert_eq!(
            VSoundIrqSpecialFlow::Deferred {
                routine: VSoundSpecialRoutine::Spinner1,
            }
            .irq3_handoff(),
            VSoundIrq3Handoff::Deferred {
                routine: VSoundSpecialRoutine::Spinner1,
            }
        );
    }

    #[test]
    fn vsnd_irq_command_flow_reports_source_irq3_handoff_readiness() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(0));
        assert_eq!(
            board
                .step_vsnd_irq_command_flow()
                .expect("invalid flow should classify")
                .irq3_handoff(),
            VSoundIrq3Handoff::Ready
        );

        board.latch_main_board_sound_command(raw_for_vsound_code(1));
        assert_eq!(
            board
                .step_vsnd_irq_command_flow()
                .expect("GWAVE flow should run")
                .irq3_handoff(),
            VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::GWave)
        );

        board.latch_main_board_sound_command(raw_for_vsound_code(29));
        assert_eq!(
            board
                .step_vsnd_irq_command_flow()
                .expect("VARI flow should run")
                .irq3_handoff(),
            VSoundIrq3Handoff::Running(VSoundIrqRunningRoutine::Vari)
        );

        board.latch_main_board_sound_command(raw_for_vsound_code(17));
        assert_eq!(
            board
                .step_vsnd_irq_command_flow()
                .expect("special LITE flow should run")
                .irq3_handoff(),
            VSoundIrq3Handoff::Ready
        );

        assert_eq!(
            VSoundIrqCommandFlow::Vari {
                dispatch: VSoundIrqDispatch {
                    latched_port_b: 0,
                    command_code: 0,
                    effective_code: 0,
                    organ_step: VSoundOrganIrqStep::Inactive,
                    talking_program_present: false,
                    routine: VSoundIrqRoutine::Invalid,
                    spinner_flag: 0,
                    bonus2_flag: 0,
                    background1_flag: 0,
                    background2_flag: 0,
                    background: VSoundBackgroundContinuation::WaitingForBackground,
                },
                load: VSoundVariLoad {
                    table_index: 0,
                    vector_address: 0,
                    low_period: 0,
                    high_period: 0,
                    low_period_delta: 0,
                    high_period_delta: 0,
                    high_period_end: 0,
                    sweep_period: 0,
                    low_period_mod: 0,
                    amplitude: 0,
                },
                sweep: minimal_vari_sweep(VSoundVariSweepResult::Terminated {
                    low_period: 0,
                    low_period_mod: 0,
                }),
            }
            .irq3_handoff(),
            VSoundIrq3Handoff::Ready
        );
    }

    #[test]
    fn vsnd_irq_command_and_background_flow_enters_irq3_after_ready_command() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0x44).expect("seed BG1FLG");
        board.write_byte(0x0005, 0x55).expect("seed BG2FLG");
        board.write_byte(0x0007, 0x66).expect("seed B2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(0));

        let flow = board
            .step_vsnd_irq_command_and_background_flow()
            .expect("invalid command should reach IRQ3");

        assert!(matches!(
            &flow.command,
            VSoundIrqCommandFlow::Invalid {
                dispatch: VSoundIrqDispatch {
                    command_code: 0,
                    background1_flag: 0x44,
                    background2_flag: 0x55,
                    background: VSoundBackgroundContinuation::Background1,
                    ..
                }
            }
        ));
        assert!(matches!(
            &flow.irq3,
            VSoundIrq3AfterCommand::Entered(VSoundIrq3BackgroundFlow::Background1(_))
        ));
        assert_eq!(board.ram()[0x04], 1);
        assert_eq!(board.ram()[0x07], 0);
        assert_eq!(board.last_dac_value(), Some(0));
    }

    #[test]
    fn vsnd_irq_command_and_background_flow_skips_irq3_while_command_runs() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0x44).expect("seed BG1FLG");
        board.write_byte(0x0007, 0x66).expect("seed B2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        let flow = board
            .step_vsnd_irq_command_and_background_flow()
            .expect("GWAVE command should run");

        assert!(matches!(&flow.command, VSoundIrqCommandFlow::GWave { .. }));
        assert_eq!(
            flow.irq3,
            VSoundIrq3AfterCommand::Skipped(VSoundIrq3Handoff::Running(
                VSoundIrqRunningRoutine::GWave
            ))
        );
        assert_eq!(board.ram()[0x04], 0x44);
        assert_eq!(board.ram()[0x07], 0);
        assert_eq!(board.last_dac_value(), Some(0));
    }

    #[test]
    fn vsnd_irq_command_and_background_flow_honors_bgend_before_irq3() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0x44).expect("seed BG1FLG");
        board.write_byte(0x0005, 0x55).expect("seed BG2FLG");
        board.write_byte(0x0007, 0x66).expect("seed B2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(19));

        let flow = board
            .step_vsnd_irq_command_and_background_flow()
            .expect("BGEND should reach IRQ3 with flags cleared");

        assert!(matches!(
            &flow.command,
            VSoundIrqCommandFlow::Special {
                step: VSoundIrqSpecialFlow::BackgroundEnd(VSoundBackgroundFlags {
                    background1_flag: 0,
                    background2_flag: 0,
                }),
                ..
            }
        ));
        assert_eq!(
            flow.irq3,
            VSoundIrq3AfterCommand::Entered(VSoundIrq3BackgroundFlow::WaitingForBackground)
        );
        assert_eq!(&board.ram()[0x04..0x08], &[0, 0, 0, 0]);
    }

    #[test]
    fn vsnd_irq_command_and_background_flow_reports_background_errors() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0005, 0x55).expect("seed BG2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(0));

        assert_eq!(
            board.step_vsnd_irq_command_and_background_flow(),
            Err(VSoundIrqCommandAndBackgroundFlowError::Background(
                VSoundGWaveLoadError::MissingSoundRomByte { address: 0xFF4E },
            ))
        );
        assert_eq!(board.ram()[0x05], 0x55);
        assert_eq!(board.ram()[0x07], 0);
    }

    #[test]
    fn vsnd_irq_command_and_background_flow_wraps_command_errors() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0x44).expect("seed BG1FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        assert_eq!(
            board.step_vsnd_irq_command_and_background_flow(),
            Err(VSoundIrqCommandAndBackgroundFlowError::Command(
                VSoundIrqCommandFlowError::GWave(VSoundGWaveLoadError::MissingSoundRomByte {
                    address: 0xFEEC,
                }),
            ))
        );
        assert_eq!(board.ram()[0x04], 0x44);
        assert_eq!(board.last_dac_value(), None);
    }

    #[test]
    fn vsnd_gwave_loader_copies_source_waveform_and_parameters() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let load = board
            .load_vsnd_gwave_sound_code(1)
            .expect("load HBDV GWAVE");

        assert_eq!(load.table_index, 0);
        assert_eq!(load.vector_address, 0xFEEC);
        assert_eq!(load.echo_count, 8);
        assert_eq!(load.cycle_count, 1);
        assert_eq!(load.echo_decay, 2);
        assert_eq!(load.waveform_index, 4);
        assert_eq!(load.waveform_address, 0xFE81);
        assert_eq!(load.waveform_length, 16);
        assert_eq!(load.predecay_factor, 0);
        assert_eq!(load.frequency_delta, 0);
        assert_eq!(load.frequency_delta_count, 0);
        assert_eq!(load.frequency_pattern_address, 0xFF86);
        assert_eq!(load.frequency_pattern_length, 22);
        assert_eq!(load.frequency_end_address, 0xFF9C);
        assert_eq!(load.wave_ram_start, 0x0024);
        assert_eq!(load.wave_ram_end, 0x0034);
        assert_eq!(
            &board.ram()[0x13..0x21],
            &[
                8, 1, 2, 0, 0, 0xFE, 0x81, 0, 0xFF, 0x86, 0xFF, 0x9C, 0, 0x34
            ]
        );
        assert_eq!(
            &board.ram()[0x24..0x34],
            &[
                0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn vsnd_gwave_loader_applies_source_predecay() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let load = board.load_vsnd_gwave_vector(10).expect("load ED10 GWAVE");

        assert_eq!(load.vector_address, 0xFF32);
        assert_eq!(load.echo_count, 15);
        assert_eq!(load.cycle_count, 6);
        assert_eq!(load.echo_decay, 5);
        assert_eq!(load.waveform_index, 3);
        assert_eq!(load.waveform_address, 0xFE70);
        assert_eq!(load.predecay_factor, 3);
        assert_eq!(load.frequency_pattern_address, 0xFFE9);
        assert_eq!(load.frequency_end_address, 0xFFEF);
        assert_eq!(
            &board.ram()[0x24..0x34],
            &[
                106, 161, 194, 189, 158, 117, 91, 88, 106, 121, 119, 92, 52, 20, 15, 48
            ]
        );
    }

    #[test]
    fn vsnd_gwave_loader_rejects_non_gwave_inputs_and_oversized_waveforms() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(
            board.load_vsnd_gwave_sound_code(VSNDRM1_SPINNER_SOUND_CODE),
            Err(VSoundGWaveLoadError::InvalidSoundCode {
                sound_code: VSNDRM1_SPINNER_SOUND_CODE,
            })
        );
        assert_eq!(
            board.load_vsnd_gwave_vector(15),
            Err(VSoundGWaveLoadError::InvalidVectorIndex { table_index: 15 })
        );

        let mut sound = vec![0; 0x0800];
        write_test_sound_rom(&mut sound, 0xFE4D, &[73]);
        write_test_sound_rom(&mut sound, 0xFEEC, &[0x01, 0x00, 0, 0, 0, 1, 0]);
        let images = test_rom_images_from_sound(sound);
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(
            board.load_vsnd_gwave_vector(0),
            Err(VSoundGWaveLoadError::WaveformTooLong {
                address: 0xFE4D,
                length: 73,
            })
        );
    }

    #[test]
    fn vsnd_vari_loader_copies_source_vectors_to_direct_page() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let saw = board.load_vsnd_vari_sound_code(29).expect("load SAW VARI");

        assert_eq!(saw.table_index, 0);
        assert_eq!(saw.vector_address, 0xFD76);
        assert_eq!(saw.low_period, 0x40);
        assert_eq!(saw.high_period, 0x01);
        assert_eq!(saw.low_period_delta, 0x00);
        assert_eq!(saw.high_period_delta, 0x10);
        assert_eq!(saw.high_period_end, 0xE1);
        assert_eq!(saw.sweep_period, 0x0080);
        assert_eq!(saw.low_period_mod, 0xFF);
        assert_eq!(saw.amplitude, 0xFF);
        assert_eq!(
            &board.ram()[0x13..0x1C],
            &[0x40, 0x01, 0x00, 0x10, 0xE1, 0x00, 0x80, 0xFF, 0xFF]
        );

        let cabinet_shake = board.load_vsnd_vari_vector(3).expect("load CABSHK VARI");

        assert_eq!(cabinet_shake.vector_address, 0xFD91);
        assert_eq!(cabinet_shake.low_period, 0xFF);
        assert_eq!(cabinet_shake.high_period, 0x01);
        assert_eq!(cabinet_shake.high_period_delta, 0x18);
        assert_eq!(cabinet_shake.high_period_end, 0x41);
        assert_eq!(cabinet_shake.sweep_period, 0x0480);
        assert_eq!(cabinet_shake.low_period_mod, 0x00);
        assert_eq!(cabinet_shake.amplitude, 0xFF);
        assert_eq!(
            &board.ram()[0x13..0x1C],
            &[0xFF, 0x01, 0x00, 0x18, 0x41, 0x04, 0x80, 0x00, 0xFF]
        );
    }

    #[test]
    fn vsnd_vari_loader_rejects_non_vari_inputs_and_out_of_range_vectors() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(
            board.load_vsnd_vari_sound_code(1),
            Err(VSoundVariLoadError::InvalidSoundCode { sound_code: 1 })
        );
        assert_eq!(
            board.load_vsnd_vari_vector(4),
            Err(VSoundVariLoadError::InvalidVectorIndex { table_index: 4 })
        );
    }

    #[test]
    fn vsnd_vari_start_outputs_saw_sweep_and_updates_direct_page_counts() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.load_vsnd_vari_sound_code(29).expect("load SAW VARI");

        assert_eq!(
            board.run_vsnd_vari_start(),
            VSoundVariSweep {
                sweep_period: 0x0080,
                low_count: 0x40,
                high_count: 0x01,
                low_count_after_sweep: 0x40,
                high_count_after_sweep: 0x11,
                dac_samples: vec![0xFF, 0x00, 0xFF, 0x00, 0xFF],
                result: VSoundVariSweepResult::Continue,
            }
        );
        assert_eq!(board.last_dac_value(), Some(0xFF));
        assert_eq!(&board.ram()[0x1C..0x1E], &[0x40, 0x11]);
    }

    #[test]
    fn vsnd_vari_sweep_restarts_when_high_count_reaches_end_with_mod() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.load_vsnd_vari_sound_code(29).expect("load SAW VARI");
        board.write_byte(0x0018, 0x00).expect("seed SWPDT high");
        board.write_byte(0x0019, 0x02).expect("seed SWPDT low");
        board.write_byte(0x001C, 0x01).expect("seed LOCNT");
        board.write_byte(0x001D, 0xD1).expect("seed HICNT");

        assert_eq!(
            board.run_vsnd_vari_sweep(),
            VSoundVariSweep {
                sweep_period: 0x0002,
                low_count: 0x01,
                high_count: 0xD1,
                low_count_after_sweep: 0x01,
                high_count_after_sweep: 0xE1,
                dac_samples: vec![0xFF, 0x00, 0xFF],
                result: VSoundVariSweepResult::Restarted { low_period: 0x3F },
            }
        );
        assert_eq!(board.ram()[0x13], 0x3F);
        assert_eq!(&board.ram()[0x1C..0x1E], &[0x3F, 0x01]);
    }

    #[test]
    fn vsnd_vari_sweep_counts_down_multi_count_high_half_cycle() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.load_vsnd_vari_sound_code(29).expect("load SAW VARI");
        board.write_byte(0x0018, 0x00).expect("seed SWPDT high");
        board.write_byte(0x0019, 0x04).expect("seed SWPDT low");
        board.write_byte(0x001C, 0x01).expect("seed LOCNT");
        board.write_byte(0x001D, 0x02).expect("seed HICNT");

        assert_eq!(
            board.run_vsnd_vari_sweep(),
            VSoundVariSweep {
                sweep_period: 0x0004,
                low_count: 0x01,
                high_count: 0x02,
                low_count_after_sweep: 0x01,
                high_count_after_sweep: 0x12,
                dac_samples: vec![0xFF, 0x00, 0xFF, 0xFF],
                result: VSoundVariSweepResult::Continue,
            }
        );
        assert_eq!(&board.ram()[0x1C..0x1E], &[0x01, 0x12]);
    }

    #[test]
    fn vsnd_vari_sweep_terminates_when_low_period_mod_is_zero() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.load_vsnd_vari_sound_code(29).expect("load SAW VARI");
        board.write_byte(0x0018, 0x00).expect("seed SWPDT high");
        board.write_byte(0x0019, 0x01).expect("seed SWPDT low");
        board.write_byte(0x001A, 0x00).expect("clear LOMOD");
        board.write_byte(0x001C, 0x01).expect("seed LOCNT");
        board.write_byte(0x001D, 0xD1).expect("seed HICNT");

        assert_eq!(
            board.run_vsnd_vari_sweep(),
            VSoundVariSweep {
                sweep_period: 0x0001,
                low_count: 0x01,
                high_count: 0xD1,
                low_count_after_sweep: 0x01,
                high_count_after_sweep: 0xE1,
                dac_samples: vec![0xFF, 0xFF],
                result: VSoundVariSweepResult::Terminated {
                    low_period: 0x40,
                    low_period_mod: 0,
                },
            }
        );
        assert_eq!(board.last_dac_value(), Some(0xFF));
        assert_eq!(&board.ram()[0x1C..0x1E], &[0x01, 0xE1]);
    }

    #[test]
    fn vsnd_vari_sweep_terminates_when_low_period_mod_wraps_to_zero() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.load_vsnd_vari_sound_code(29).expect("load SAW VARI");
        board.write_byte(0x0013, 0x01).expect("seed LOPER");
        board.write_byte(0x0018, 0x00).expect("seed SWPDT high");
        board.write_byte(0x0019, 0x01).expect("seed SWPDT low");
        board.write_byte(0x001C, 0x01).expect("seed LOCNT");
        board.write_byte(0x001D, 0xD1).expect("seed HICNT");

        assert_eq!(
            board.run_vsnd_vari_sweep(),
            VSoundVariSweep {
                sweep_period: 0x0001,
                low_count: 0x01,
                high_count: 0xD1,
                low_count_after_sweep: 0x01,
                high_count_after_sweep: 0xE1,
                dac_samples: vec![0xFF, 0xFF],
                result: VSoundVariSweepResult::Terminated {
                    low_period: 0,
                    low_period_mod: 0xFF,
                },
            }
        );
        assert_eq!(board.ram()[0x13], 0);
        assert_eq!(&board.ram()[0x1C..0x1E], &[0x01, 0xE1]);
    }

    #[test]
    fn vsnd_lite_outputs_source_liten_random_complement_stream() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x3C).expect("seed random HI");
        board.write_byte(0x000A, 0x00).expect("seed random LO");

        let noise = board.run_vsnd_lite();

        assert_eq!(noise.initial_frequency, 1);
        assert_eq!(noise.final_frequency, 0);
        assert_eq!(noise.frequency_delta, 1);
        assert_eq!(noise.cycle_count, 3);
        assert_eq!(noise.frequency_passes, 255);
        assert_eq!(noise.random_steps, 765);
        assert_eq!(noise.random_seed_hi, 0xC6);
        assert_eq!(noise.random_seed_lo, 0x09);
        assert_eq!(noise.dac_samples.len(), 386);
        assert_alternating_dac_transitions(&noise.dac_samples);
        assert_eq!(board.last_dac_value(), Some(0));
        assert_eq!(&board.ram()[0x15..0x1B], &[3, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn vsnd_appear_outputs_source_liten_descending_frequency_stream() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x12).expect("seed random HI");
        board.write_byte(0x000A, 0x34).expect("seed random LO");

        let noise = board.run_vsnd_appear();

        assert_eq!(noise.initial_frequency, 0xC0);
        assert_eq!(noise.final_frequency, 0);
        assert_eq!(noise.frequency_delta, 0xFE);
        assert_eq!(noise.cycle_count, 0x10);
        assert_eq!(noise.frequency_passes, 96);
        assert_eq!(noise.random_steps, 1536);
        assert_eq!(noise.random_seed_hi, 0xA2);
        assert_eq!(noise.random_seed_lo, 0x5C);
        assert_eq!(noise.dac_samples.len(), 765);
        assert_alternating_dac_transitions(&noise.dac_samples);
        assert_eq!(board.last_dac_value(), Some(0xFF));
        assert_eq!(&board.ram()[0x15..0x1B], &[0x10, 0, 0, 0, 0, 0xFE]);
    }

    #[test]
    fn vsnd_turbo_outputs_source_noise_decay_stream() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x3C).expect("seed random HI");
        board.write_byte(0x000A, 0x00).expect("seed random LO");

        let noise = board.run_vsnd_turbo();

        assert_eq!(noise.initial_period, 1);
        assert_eq!(noise.final_period, 255);
        assert_eq!(noise.initial_amplitude, 0xFF);
        assert_eq!(noise.final_amplitude, 1);
        assert_eq!(noise.decay, 1);
        assert_eq!(noise.cycle_count, 0x20);
        assert_eq!(noise.amplitude_passes, 255);
        assert_eq!(noise.random_steps, 8160);
        assert_eq!(noise.random_seed_hi, 0x7C);
        assert_eq!(noise.random_seed_lo, 0x66);
        assert_eq!(noise.dac_samples.len(), 8160);
        assert_eq!(
            &noise.dac_samples[..32],
            &[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255,
                255, 255, 0, 255, 255, 255, 0, 0,
            ]
        );
        assert_eq!(
            &noise.dac_samples[noise.dac_samples.len() - 32..],
            &[
                1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 1,
                0, 1, 0, 1,
            ]
        );
        assert_eq!(
            noise
                .dac_samples
                .iter()
                .filter(|&&sample| sample == 0)
                .count(),
            4189
        );
        assert_eq!(
            noise
                .dac_samples
                .iter()
                .filter(|&&sample| sample != 0)
                .count(),
            3971
        );
        assert_eq!(board.last_dac_value(), Some(1));
        assert_eq!(&board.ram()[0x13..0x19], &[1, 1, 0x20, 0, 0xFF, 0x20]);
    }

    #[test]
    fn vsnd_cannon_outputs_source_filtered_noise_decay_stream() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x3C).expect("seed random HI");
        board.write_byte(0x000A, 0x00).expect("seed random LO");

        let noise = board.run_vsnd_cannon();

        assert_eq!(noise.initial_sample_count, 1000);
        assert_eq!(noise.initial_max_frequency, 0xFF);
        assert_eq!(noise.final_max_frequency, 0);
        assert_eq!(noise.final_frequency_high, 0);
        assert_eq!(noise.final_frequency_low, 7);
        assert!(noise.frequency_decay_enabled);
        assert!(noise.distortion_enabled);
        assert_eq!(noise.decay_passes, 72);
        assert_eq!(noise.random_steps, 1817);
        assert_eq!(noise.random_seed_hi, 0xE2);
        assert_eq!(noise.random_seed_lo, 0x56);
        assert_eq!(noise.dac_samples.len(), 73673);
        assert_eq!(
            &noise.dac_samples[..32],
            &[
                0, 0, 0, 0, 0, 7, 14, 21, 28, 35, 42, 49, 56, 63, 70, 77, 84, 91, 98, 105, 112,
                119, 126, 128, 128, 131, 134, 137, 140, 143, 146, 149,
            ]
        );
        assert_eq!(
            &noise.dac_samples[noise.dac_samples.len() - 32..],
            &[
                145, 145, 145, 145, 145, 145, 145, 145, 145, 145, 145, 145, 145, 145, 145, 145,
                145, 145, 145, 145, 145, 145, 145, 145, 145, 145, 145, 145, 145, 144, 144, 144,
            ]
        );
        assert_eq!(count_dac_samples(&noise.dac_samples, 0), 35);
        assert_eq!(count_dac_samples(&noise.dac_samples, 1), 15);
        assert_eq!(count_dac_samples(&noise.dac_samples, 7), 42);
        assert_eq!(count_dac_samples(&noise.dac_samples, 0xFF), 41);
        assert_eq!(board.last_dac_value(), Some(0x90));
        assert_eq!(&board.ram()[0x13..0x1A], &[0, 0, 7, 0x03, 0xE8, 1, 1]);
    }

    #[test]
    fn vsnd_background1_outputs_first_source_fnoise_window() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x3C).expect("seed random HI");
        board.write_byte(0x000A, 0x00).expect("seed random LO");

        let noise = board.run_vsnd_background1_fnoise_window();

        assert_eq!(noise.entry_address, 0xF913);
        assert_eq!(noise.initial_sample_count, 0xF913);
        assert_eq!(noise.initial_max_frequency, 1);
        assert_eq!(noise.final_frequency_high, 1);
        assert_eq!(noise.final_frequency_low, 0);
        assert!(!noise.frequency_decay_enabled);
        assert!(!noise.distortion_enabled);
        assert_eq!(noise.background1_flag, 1);
        assert!(noise.continues_after_window);
        assert_eq!(noise.random_steps, 1000);
        assert_eq!(noise.random_seed_hi, 0x87);
        assert_eq!(noise.random_seed_lo, 0xB5);
        assert_eq!(noise.dac_samples.len(), 64761);
        assert_eq!(
            &noise.dac_samples[..32],
            &[
                0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
                20, 21, 22, 23, 24, 25, 26, 27,
            ]
        );
        assert_eq!(
            &noise.dac_samples[noise.dac_samples.len() - 32..],
            &[
                136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151,
                152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167,
            ]
        );
        assert_eq!(count_dac_samples(&noise.dac_samples, 0), 11);
        assert_eq!(count_dac_samples(&noise.dac_samples, 1), 11);
        assert_eq!(count_dac_samples(&noise.dac_samples, 7), 36);
        assert_eq!(count_dac_samples(&noise.dac_samples, 0xFF), 0);
        assert_eq!(board.last_dac_value(), Some(0xA7));
        assert_eq!(&board.ram()[0x13..0x1A], &[1, 1, 0, 0xF9, 0x13, 0, 0]);
    }

    #[test]
    fn vsnd_thrust_outputs_first_source_fnoise_window() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0xAA).expect("seed BG1FLG");
        board.write_byte(0x0009, 0x3C).expect("seed random HI");
        board.write_byte(0x000A, 0x00).expect("seed random LO");

        let noise = board.run_vsnd_thrust_fnoise_window();

        assert_eq!(noise.entry_address, 0xF91C);
        assert_eq!(noise.initial_sample_count, 0xF91C);
        assert_eq!(noise.initial_max_frequency, 3);
        assert_eq!(noise.final_frequency_high, 3);
        assert_eq!(noise.final_frequency_low, 0);
        assert!(!noise.frequency_decay_enabled);
        assert!(!noise.distortion_enabled);
        assert_eq!(noise.background1_flag, 0xAA);
        assert!(noise.continues_after_window);
        assert_eq!(noise.random_steps, 2903);
        assert_eq!(noise.random_seed_hi, 0xDF);
        assert_eq!(noise.random_seed_lo, 0x2D);
        assert_eq!(noise.dac_samples.len(), 66673);
        assert_eq!(
            &noise.dac_samples[..32],
            &[
                0, 0, 0, 0, 0, 3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45, 48, 51, 54,
                57, 60, 63, 66, 69, 72, 75, 78, 81,
            ]
        );
        assert_eq!(
            &noise.dac_samples[noise.dac_samples.len() - 32..],
            &[
                178, 175, 172, 169, 166, 163, 160, 157, 154, 151, 148, 145, 142, 139, 136, 133,
                130, 127, 124, 121, 118, 115, 112, 109, 106, 103, 100, 97, 94, 91, 90, 90,
            ]
        );
        assert_eq!(count_dac_samples(&noise.dac_samples, 0), 15);
        assert_eq!(count_dac_samples(&noise.dac_samples, 3), 28);
        assert_eq!(count_dac_samples(&noise.dac_samples, 7), 55);
        assert_eq!(count_dac_samples(&noise.dac_samples, 0xFF), 6);
        assert_eq!(board.last_dac_value(), Some(0x5A));
        assert_eq!(&board.ram()[0x13..0x1A], &[3, 3, 0, 0xF9, 0x1C, 0, 0]);
    }

    #[test]
    fn vsnd_radio_outputs_source_radsnd_frequency_table_stream() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let radio = board.run_vsnd_radio().expect("RADIO");

        assert_eq!(radio.table_address, 0xFD9A);
        assert_eq!(
            radio.table,
            [
                0x8C, 0x5B, 0xB6, 0x40, 0xBF, 0x49, 0xA4, 0x73, 0x73, 0xA4, 0x49, 0xBF, 0x40, 0xB6,
                0x5B, 0x8C,
            ]
        );
        assert_eq!(radio.initial_frequency, 100);
        assert_eq!(radio.final_frequency, 0xFFFF);
        assert_eq!(radio.initial_timer_high, 0);
        assert_eq!(radio.final_timer_high, 0xF7);
        assert_eq!(radio.final_timer_low, 0x2C);
        assert_eq!(radio.successful_frequency_increments, 65435);
        assert_eq!(radio.dac_samples.len(), 425344);
        assert_eq!(
            &radio.dac_samples[..32],
            &[
                0x8C, 0x8C, 0x5B, 0x5B, 0x5B, 0xB6, 0xB6, 0x40, 0x40, 0x40, 0xBF, 0xBF, 0x49, 0x49,
                0x49, 0xA4, 0xA4, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0xA4, 0xA4, 0x49, 0x49, 0x49,
                0xBF, 0xBF, 0x40, 0x40,
            ]
        );
        assert_eq!(
            &radio.dac_samples[radio.dac_samples.len() - 32..],
            &[
                0xA4, 0xA4, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73,
                0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73, 0x73,
                0x73, 0x73, 0x73, 0x73,
            ]
        );
        assert_eq!(count_dac_samples(&radio.dac_samples, 0x40), 53165);
        assert_eq!(count_dac_samples(&radio.dac_samples, 0x49), 53494);
        assert_eq!(count_dac_samples(&radio.dac_samples, 0x5B), 52989);
        assert_eq!(count_dac_samples(&radio.dac_samples, 0x73), 53297);
        assert_eq!(count_dac_samples(&radio.dac_samples, 0x8C), 52922);
        assert_eq!(count_dac_samples(&radio.dac_samples, 0xA4), 53142);
        assert_eq!(count_dac_samples(&radio.dac_samples, 0xB6), 53076);
        assert_eq!(count_dac_samples(&radio.dac_samples, 0xBF), 53259);
        assert_eq!(board.last_dac_value(), Some(0x73));
        assert_eq!(
            &board.ram()[0x0B..0x12],
            &[0xFF, 0xFF, 0, 0, 0xFD, 0xA1, 0xF7]
        );
    }

    #[test]
    fn vsnd_hyper_outputs_source_phase_edges_and_final_phase() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let sweep = board.run_vsnd_hyper();

        assert_eq!(sweep.phase_count, 128);
        assert_eq!(sweep.final_phase, 0x80);
        assert_eq!(sweep.dac_samples.len(), 257);
        assert_eq!(sweep.dac_samples[0], 0);
        for pair in sweep.dac_samples[1..].chunks_exact(2) {
            assert_eq!(pair, &[0xFF, 0x00]);
        }
        assert_eq!(board.last_dac_value(), Some(0));
        assert_eq!(board.ram()[0x11], 0x80);
    }

    #[test]
    fn vsnd_scream_outputs_source_echo_cascade_stream() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let scream = board.run_vsnd_scream();

        assert_eq!(scream.echo_count, 4);
        assert_eq!(scream.initial_frequency, 0x40);
        assert_eq!(scream.next_echo_frequency, 0x41);
        assert_eq!(scream.spawn_frequency, 0x37);
        assert_eq!(scream.initial_timer, 0);
        assert_eq!(scream.final_timer, 0);
        assert_eq!(scream.decay_passes, 95);
        assert_eq!(scream.echo_starts, 4);
        assert_eq!(scream.final_echo_table, [0; 8]);
        assert_eq!(scream.srmend_byte, 0x41);
        assert_eq!(scream.dac_samples.len(), 24320);
        assert_eq!(
            &scream.dac_samples[..32],
            &[
                0, 128, 128, 0, 0, 128, 128, 0, 0, 128, 128, 0, 0, 128, 128, 0, 0, 128, 128, 0, 0,
                128, 128, 0, 0, 128, 128, 0, 0, 128, 128, 0,
            ]
        );
        assert_eq!(
            &scream.dac_samples[scream.dac_samples.len() - 32..],
            &[0; 32]
        );
        assert_eq!(count_dac_samples(&scream.dac_samples, 0), 5609);
        assert_eq!(count_dac_samples(&scream.dac_samples, 0x10), 2555);
        assert_eq!(count_dac_samples(&scream.dac_samples, 0x20), 1592);
        assert_eq!(count_dac_samples(&scream.dac_samples, 0x40), 1600);
        assert_eq!(count_dac_samples(&scream.dac_samples, 0x80), 2555);
        assert_eq!(count_dac_samples(&scream.dac_samples, 0xC0), 1667);
        assert_eq!(count_dac_samples(&scream.dac_samples, 0xF0), 777);
        assert_eq!(
            scream
                .dac_samples
                .iter()
                .filter(|&&sample| sample != 0)
                .count(),
            18711
        );
        assert_eq!(board.last_dac_value(), Some(0));
        assert_eq!(
            &board.ram()[0x11..0x1C],
            &[0x08, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x41]
        );
    }

    #[test]
    fn vsnd_scream_uses_existing_tempb_timer_before_first_decay() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0012, 0xFF).expect("seed TEMPB");

        let scream = board.run_vsnd_scream();

        assert_eq!(scream.initial_timer, 0xFF);
        assert_eq!(scream.final_timer, 0);
        assert_eq!(scream.decay_passes, 95);
        assert_eq!(scream.echo_starts, 4);
        assert_eq!(scream.dac_samples.len(), 24065);
        assert_eq!(
            &scream.dac_samples[..32],
            &[
                0, 0, 128, 128, 0, 0, 128, 128, 0, 0, 128, 128, 0, 0, 128, 128, 0, 0, 128, 128, 0,
                0, 128, 128, 0, 0, 128, 128, 0, 0, 128, 128,
            ]
        );
        assert_eq!(
            &scream.dac_samples[scream.dac_samples.len() - 32..],
            &[0; 32]
        );
        assert_eq!(scream.final_echo_table, [0, 0x40, 0, 0, 0, 0, 0, 0]);
        assert_eq!(scream.srmend_byte, 0x41);
        assert_eq!(board.last_dac_value(), Some(0));
        assert_eq!(board.ram()[0x14], 0x40);
    }

    #[test]
    fn vsnd_organ_tune_outputs_phantom_rom_table_windows() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let tune = board.run_vsnd_organ_tune(1).expect("PHANTOM tune");
        let tune = organ_tune_played(tune).expect("PHANTOM tune should play");

        assert_eq!(tune.tune_number, 1);
        assert_eq!(tune.table_address, 0xFDAA);
        assert_eq!(tune.tune_length, 0x0C);
        assert_eq!(tune.tune_start_address, 0xFDAB);
        assert_eq!(tune.tune_end_address, 0xFDB7);
        assert_eq!(tune.notes.len(), 3);
        assert_eq!(board.ram()[0x08], 0);
        assert_eq!(board.ram()[0x11], 0);
        assert_eq!(&board.ram()[0x0D..0x11], &[0xFD, 0xB7, 0xFD, 0xB7]);

        let first = &tune.notes[0];
        assert_eq!(first.note_address, 0xFDAB);
        assert_eq!(first.oscillator_mask, 0x7F);
        assert_eq!(first.note_delay, 0x1D);
        assert_eq!(first.duration, 0x0FFB);
        assert_eq!(first.window.duration, 0x0FFB);
        assert_eq!(first.window.delay_patch.requested_delay, 0x1D);
        assert_eq!(first.window.delay_patch.rdelay_end, 0x28);
        assert_eq!(first.window.delay_patch.nop_count, 13);
        assert!(first.window.delay_patch.cmp_zero_patch);
        assert_eq!(first.window.dac_samples.len(), 4091);
        assert_eq!(first.window.final_timer, 0xFB);
        assert_eq!(
            &first.window.dac_samples[..16],
            &[
                16, 16, 32, 16, 32, 32, 48, 16, 32, 32, 48, 32, 48, 48, 64, 16
            ]
        );
        assert_eq!(
            &first.window.dac_samples[first.window.dac_samples.len() - 16..],
            &[
                64, 80, 80, 96, 48, 64, 64, 80, 64, 80, 80, 96, 64, 80, 80, 96
            ]
        );
        assert_eq!(count_dac_samples(&first.window.dac_samples, 0), 31);
        assert_eq!(count_dac_samples(&first.window.dac_samples, 112), 31);

        assert_eq!(tune.notes[1].note_address, 0xFDAF);
        assert_eq!(tune.notes[1].window.initial_timer, 0xFB);
        assert_eq!(tune.notes[1].window.final_timer, 0x10);
        assert_eq!(tune.notes[2].note_address, 0xFDB3);
        assert_eq!(tune.notes[2].duration, 0x508B);
        assert_eq!(tune.notes[2].window.delay_patch.requested_delay, 0x08);
        assert_eq!(tune.notes[2].window.final_timer, 0x9B);
        assert_eq!(board.last_dac_value(), Some(64));
    }

    #[test]
    fn vsnd_organ_tune_skips_to_taccata_rom_table() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let tune = board.run_vsnd_organ_tune(2).expect("TACCATA tune");
        let tune = organ_tune_played(tune).expect("TACCATA tune should play");

        assert_eq!(tune.tune_number, 2);
        assert_eq!(tune.table_address, 0xFDB7);
        assert_eq!(tune.tune_length, 0x88);
        assert_eq!(tune.tune_start_address, 0xFDB8);
        assert_eq!(tune.tune_end_address, 0xFE40);
        assert_eq!(tune.notes.len(), 34);
        assert_eq!(
            tune.notes
                .iter()
                .map(|note| usize::from(note.duration))
                .sum::<usize>(),
            142751
        );

        let first = &tune.notes[0];
        assert_eq!(first.note_address, 0xFDB8);
        assert_eq!(first.oscillator_mask, 0x3E);
        assert_eq!(first.note_delay, 0x3F);
        assert_eq!(first.duration, 0x023E);
        assert_eq!(first.window.final_timer, 0x3E);

        let last = tune.notes.last().expect("last TACCATA note");
        assert_eq!(last.note_address, 0xFE3C);
        assert_eq!(last.oscillator_mask, 0xFE);
        assert_eq!(last.note_delay, 0x1D);
        assert_eq!(last.duration, 0x5FE4);
        assert_eq!(last.window.final_timer, 0x9F);
        assert_eq!(&board.ram()[0x0D..0x11], &[0xFE, 0x40, 0xFE, 0x40]);
        assert_eq!(board.last_dac_value(), Some(80));
    }

    #[test]
    fn vsnd_organ_tune_reports_invalid_source_tune_number() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(
            board.run_vsnd_organ_tune(3),
            Ok(VSoundOrganTuneStep::InvalidTune {
                tune_number: 3,
                table_address: 0xFE40,
            })
        );
        assert_eq!(
            organ_tune_played(VSoundOrganTuneStep::InvalidTune {
                tune_number: 3,
                table_address: 0xFE40,
            }),
            None
        );
        assert_eq!(board.ram()[0x08], 0);
        assert_eq!(board.ram()[0x11], 1);
    }

    #[test]
    fn vsnd_organ_tune_rejects_corrupt_non_note_length() {
        let images = test_rom_images_with_organ_tune_table(&[0x03, 0x7F, 0x1D, 0x0F]);
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        assert_eq!(
            board.run_vsnd_organ_tune(1),
            Err(VSoundOrganLoadError::InvalidTuneLength {
                table_address: 0xFDAA,
                length: 3,
            })
        );
    }

    #[test]
    fn vsnd_organ_note_parameters_accumulate_source_oscillator_mask() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let start = board.run_vsnd_organ_note_start();

        assert_eq!(start.organ_flag, 3);

        let high = board
            .step_vsnd_organ_note_parameter(0x07)
            .expect("first organ parameter");
        assert_eq!(organ_note_started(high.clone()), None);
        assert_eq!(
            high,
            VSoundOrganNoteStep::MaskByte(crate::sound::VSoundOrganMaskByte {
                parameter_code: 0x07,
                organ_flag: 2,
                oscillator_mask: 0x07,
            })
        );

        let low = board
            .step_vsnd_organ_note_parameter(0x0C)
            .expect("second organ parameter");
        assert_eq!(
            low,
            VSoundOrganNoteStep::MaskByte(crate::sound::VSoundOrganMaskByte {
                parameter_code: 0x0C,
                organ_flag: 1,
                oscillator_mask: 0x7C,
            })
        );
        assert_eq!(board.ram()[0x08], 1);
        assert_eq!(board.ram()[0x15], 0x7C);
    }

    #[test]
    fn vsnd_organ_note_outputs_first_source_organ_window() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.run_vsnd_organ_note_start();
        board
            .step_vsnd_organ_note_parameter(0x07)
            .expect("first organ parameter");
        board
            .step_vsnd_organ_note_parameter(0x0C)
            .expect("second organ parameter");

        let note = board
            .step_vsnd_organ_note_parameter(0x06)
            .expect("D organ note");
        let note = organ_note_started(note).expect("third organ parameter should start note");

        assert_eq!(note.note_parameter, 0x06);
        assert_eq!(note.note_delay, 0x1D);
        assert_eq!(note.organ_flag, 0);
        assert_eq!(note.window.oscillator_mask, 0x7C);
        assert_eq!(note.window.duration, 0xFFFF);
        assert_eq!(note.window.initial_timer, 0);
        assert_eq!(note.window.final_timer, 0xFF);
        assert_eq!(note.window.delay_patch.requested_delay, 0x1D);
        assert_eq!(note.window.delay_patch.rdelay_start, 0x16);
        assert_eq!(note.window.delay_patch.rdelay_end, 0x28);
        assert_eq!(note.window.delay_patch.nop_count, 13);
        assert!(note.window.delay_patch.cmp_zero_patch);
        assert_eq!(note.window.delay_patch.jump_address, 0xFADD);
        assert_eq!(
            note.window.delay_patch.patch_bytes,
            [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0x91, 0, 0x7E, 0xFA, 0xDD,
            ]
        );
        assert_eq!(note.window.dac_samples.len(), 65535);
        assert_eq!(
            &note.window.dac_samples[..32],
            &[
                0, 0, 0, 16, 16, 16, 16, 16, 16, 16, 16, 32, 32, 32, 32, 16, 16, 16, 16, 32, 32,
                32, 32, 32, 32, 32, 32, 48, 48, 48, 48, 16,
            ]
        );
        assert_eq!(
            &note.window.dac_samples[note.window.dac_samples.len() - 32..],
            &[
                32, 32, 32, 32, 48, 48, 48, 48, 48, 48, 48, 48, 64, 64, 64, 64, 48, 48, 48, 48, 64,
                64, 64, 64, 64, 64, 64, 64, 80, 80, 80, 80,
            ]
        );
        assert_eq!(count_dac_samples(&note.window.dac_samples, 0), 2047);
        assert_eq!(count_dac_samples(&note.window.dac_samples, 16), 10240);
        assert_eq!(count_dac_samples(&note.window.dac_samples, 32), 20480);
        assert_eq!(count_dac_samples(&note.window.dac_samples, 48), 20480);
        assert_eq!(count_dac_samples(&note.window.dac_samples, 64), 10240);
        assert_eq!(count_dac_samples(&note.window.dac_samples, 80), 2048);
        assert_eq!(board.last_dac_value(), Some(80));
        assert_eq!(
            &board.ram()[0x13..0x1C],
            &[0xFF, 0xFF, 0x7C, 1, 1, 1, 1, 1, 1]
        );
    }

    #[test]
    fn vsnd_organ_note_invalid_note_parameter_uses_zero_delay() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0008, 1).expect("seed ORGFLG");
        board.write_byte(0x0015, 1).expect("seed OSCIL");

        let note = board
            .step_vsnd_organ_note_parameter(0x1F)
            .expect("invalid note parameter");
        let note = organ_note_started(note).expect("final organ parameter should start note");

        assert_eq!(note.note_parameter, 0x1F);
        assert_eq!(note.note_delay, 0);
        assert_eq!(note.window.delay_patch.nop_count, 0);
        assert!(!note.window.delay_patch.cmp_zero_patch);
        assert_eq!(note.window.delay_patch.patch_bytes, [0x7E, 0xFA, 0xDD]);
        assert_eq!(note.window.dac_samples.len(), 65535);
        assert_eq!(count_dac_samples(&note.window.dac_samples, 0), 32767);
        assert_eq!(count_dac_samples(&note.window.dac_samples, 16), 32768);
        assert_eq!(board.last_dac_value(), Some(16));
        assert_eq!(board.ram()[0x12], 0xFF);
    }

    #[test]
    fn vsnd_organ_note_rejects_delay_patch_that_exceeds_rdelay() {
        for note_delay in [0xFF, 0x79, 0x74] {
            let images = test_rom_images_with_organ_note_delay(note_delay);
            let mut board = DefenderSoundBoard::with_cleared_ram(&images);
            board.write_byte(0x0008, 1).expect("seed ORGFLG");

            assert_eq!(
                board.step_vsnd_organ_note_parameter(0),
                Err(VSoundOrganLoadError::DelayPatchTooLong { delay: note_delay })
            );
        }
    }

    fn assert_alternating_dac_transitions(samples: &[u8]) {
        assert_eq!(samples[0], 0xFF);
        for (index, sample) in samples.iter().copied().enumerate().skip(1) {
            let expected = if index % 2 == 0 { 0xFF } else { 0x00 };
            assert_eq!(sample, expected, "sample {index}");
        }
    }

    fn count_dac_samples(samples: &[u8], expected: u8) -> usize {
        samples.iter().filter(|&&sample| sample == expected).count()
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct VSoundDacSignature {
        sample_count: usize,
        crc32: u32,
        first_sample: Option<u8>,
        last_sample: Option<u8>,
        last_dac_value: Option<u8>,
        direct_page_crc32: u32,
    }

    #[test]
    fn vsnd_waveform_signature_matrix_locks_deterministic_dac_buffers() {
        let images = test_rom_images_with_vsnd_tables();
        let mut signatures = Vec::new();

        let mut gwave = DefenderSoundBoard::with_cleared_ram(&images);
        gwave
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        let gwave_step = gwave.run_vsnd_gwave_start().expect("GWAVE period");
        let VSoundGWaveStep::Playing(gwave_period) = &gwave_step else {
            panic!("expected GWAVE playback");
        };
        signatures.push(dac_signature(
            "gwave_bonus2",
            &gwave,
            &gwave_period.dac_samples,
        ));

        let mut vari = DefenderSoundBoard::with_cleared_ram(&images);
        vari.load_vsnd_vari_vector(0).expect("load VARI vector");
        let vari_sweep = vari.run_vsnd_vari_start();
        signatures.push(dac_signature(
            "vari_vector0",
            &vari,
            &vari_sweep.dac_samples,
        ));

        let mut lite = DefenderSoundBoard::with_cleared_ram(&images);
        lite.write_byte(0x0009, 0x3C).expect("seed LITE HI");
        lite.write_byte(0x000A, 0x00).expect("seed LITE LO");
        let lite_noise = lite.run_vsnd_lite();
        signatures.push(dac_signature("lite", &lite, &lite_noise.dac_samples));

        let mut turbo = DefenderSoundBoard::with_cleared_ram(&images);
        turbo.write_byte(0x0009, 0x3C).expect("seed TURBO HI");
        turbo.write_byte(0x000A, 0x00).expect("seed TURBO LO");
        let turbo_noise = turbo.run_vsnd_turbo();
        signatures.push(dac_signature("turbo", &turbo, &turbo_noise.dac_samples));

        let mut cannon = DefenderSoundBoard::with_cleared_ram(&images);
        cannon.write_byte(0x0009, 0x3C).expect("seed CANNON HI");
        cannon.write_byte(0x000A, 0x00).expect("seed CANNON LO");
        let cannon_noise = cannon.run_vsnd_cannon();
        signatures.push(dac_signature("cannon", &cannon, &cannon_noise.dac_samples));

        let mut radio = DefenderSoundBoard::with_cleared_ram(&images);
        let radio_wave = radio.run_vsnd_radio().expect("RADIO");
        signatures.push(dac_signature("radio", &radio, &radio_wave.dac_samples));

        let mut hyper = DefenderSoundBoard::with_cleared_ram(&images);
        let hyper_sweep = hyper.run_vsnd_hyper();
        signatures.push(dac_signature("hyper", &hyper, &hyper_sweep.dac_samples));

        let mut scream = DefenderSoundBoard::with_cleared_ram(&images);
        let scream_wave = scream.run_vsnd_scream();
        signatures.push(dac_signature("scream", &scream, &scream_wave.dac_samples));

        let mut organ = DefenderSoundBoard::with_cleared_ram(&images);
        organ.run_vsnd_organ_note_start();
        organ
            .step_vsnd_organ_note_parameter(0x07)
            .expect("first organ parameter");
        organ
            .step_vsnd_organ_note_parameter(0x0C)
            .expect("second organ parameter");
        let organ_note = organ
            .step_vsnd_organ_note_parameter(0x06)
            .expect("D organ note");
        let VSoundOrganNoteStep::NoteStarted(organ_window) = &organ_note else {
            panic!("expected organ note window");
        };
        signatures.push(dac_signature(
            "organ_note_d",
            &organ,
            &organ_window.window.dac_samples,
        ));

        assert_eq!(
            signatures,
            vec![
                (
                    "gwave_bonus2",
                    VSoundDacSignature {
                        sample_count: 8,
                        crc32: 0xACC3_61E2,
                        first_sample: Some(0),
                        last_sample: Some(64),
                        last_dac_value: Some(64),
                        direct_page_crc32: 0xB1F0_43F9,
                    },
                ),
                (
                    "vari_vector0",
                    VSoundDacSignature {
                        sample_count: 5,
                        crc32: 0x9FDC_EBF1,
                        first_sample: Some(0xFF),
                        last_sample: Some(0xFF),
                        last_dac_value: Some(0xFF),
                        direct_page_crc32: 0x8CB8_76B7,
                    },
                ),
                (
                    "lite",
                    VSoundDacSignature {
                        sample_count: 386,
                        crc32: 0x39D3_9D89,
                        first_sample: Some(0xFF),
                        last_sample: Some(0),
                        last_dac_value: Some(0),
                        direct_page_crc32: 0x726C_8C1F,
                    },
                ),
                (
                    "turbo",
                    VSoundDacSignature {
                        sample_count: 8160,
                        crc32: 0x9E91_31E4,
                        first_sample: Some(0),
                        last_sample: Some(1),
                        last_dac_value: Some(1),
                        direct_page_crc32: 0xFB8E_EC11,
                    },
                ),
                (
                    "cannon",
                    VSoundDacSignature {
                        sample_count: 73673,
                        crc32: 0x13A2_622B,
                        first_sample: Some(0),
                        last_sample: Some(0x90),
                        last_dac_value: Some(0x90),
                        direct_page_crc32: 0xA9DE_95F1,
                    },
                ),
                (
                    "radio",
                    VSoundDacSignature {
                        sample_count: 425344,
                        crc32: 0x6BB4_AAA3,
                        first_sample: Some(0x8C),
                        last_sample: Some(0x73),
                        last_dac_value: Some(0x73),
                        direct_page_crc32: 0x883E_D08C,
                    },
                ),
                (
                    "hyper",
                    VSoundDacSignature {
                        sample_count: 257,
                        crc32: 0x250C_A37B,
                        first_sample: Some(0),
                        last_sample: Some(0),
                        last_dac_value: Some(0),
                        direct_page_crc32: 0xE0C9_9B0E,
                    },
                ),
                (
                    "scream",
                    VSoundDacSignature {
                        sample_count: 24320,
                        crc32: 0x98DC_C563,
                        first_sample: Some(0),
                        last_sample: Some(0),
                        last_dac_value: Some(0),
                        direct_page_crc32: 0x266B_1967,
                    },
                ),
                (
                    "organ_note_d",
                    VSoundDacSignature {
                        sample_count: 65535,
                        crc32: 0x7103_0F73,
                        first_sample: Some(0),
                        last_sample: Some(80),
                        last_dac_value: Some(80),
                        direct_page_crc32: 0x6816_4827,
                    },
                ),
            ]
        );
    }

    fn dac_signature(
        name: &'static str,
        board: &DefenderSoundBoard<'_>,
        samples: &[u8],
    ) -> (&'static str, VSoundDacSignature) {
        (
            name,
            VSoundDacSignature {
                sample_count: samples.len(),
                crc32: crc32(samples),
                first_sample: samples.first().copied(),
                last_sample: samples.last().copied(),
                last_dac_value: board.last_dac_value(),
                direct_page_crc32: crc32(&board.ram()[0x04..0x34]),
            },
        )
    }

    #[test]
    fn vsnd_spinner1_setup_matches_source_pre_loop_state() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .write_byte(0x0006, VSNDRM1_SPINNER_MAX - 1)
            .expect("seed SP1FLG");

        let wrapped = board.run_vsnd_spinner1_setup().expect("SP1 setup");

        assert_eq!(wrapped.vari_load.table_index, 3);
        assert_eq!(wrapped.vari_load.vector_address, 0xFD91);
        assert_eq!(wrapped.spinner_flag, 1);
        assert_eq!(wrapped.low_period, 254);
        assert_eq!(
            &board.ram()[0x13..0x1C],
            &[0xFE, 0x01, 0x00, 0x18, 0x41, 0x04, 0x80, 0x00, 0xFF]
        );

        let next = board.run_vsnd_spinner1_setup().expect("second SP1 setup");

        assert_eq!(next.spinner_flag, 2);
        assert_eq!(next.low_period, 240);
        assert_eq!(board.ram()[0x06], 2);
        assert_eq!(board.ram()[0x13], 240);
    }

    #[test]
    fn vsnd_bonus2_setup_starts_once_then_continues_source_path() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);

        let started = board.run_vsnd_bonus2_setup().expect("BON2 start setup");

        let VSoundBonus2Setup::Started {
            gwave_load,
            bonus2_flag,
        } = started
        else {
            panic!("expected BON2 start");
        };
        assert_eq!(bonus2_flag, 1);
        assert_eq!(gwave_load.table_index, 13);
        assert_eq!(gwave_load.vector_address, 0xFF47);
        assert_eq!(gwave_load.echo_count, 3);
        assert_eq!(gwave_load.cycle_count, 1);
        assert_eq!(gwave_load.echo_decay, 1);
        assert_eq!(gwave_load.waveform_index, 1);
        assert_eq!(gwave_load.waveform_address, 0xFE56);
        assert_eq!(gwave_load.frequency_delta, 0xFF);
        assert_eq!(gwave_load.frequency_pattern_address, 0xFF55);
        assert_eq!(gwave_load.frequency_end_address, 0xFF62);
        assert_eq!(&board.ram()[0x24..0x2C], &[0, 64, 128, 0, 255, 0, 128, 64]);

        let continued = board
            .run_vsnd_bonus2_setup()
            .expect("BON2 continuation should run GEND50");
        let VSoundBonus2Setup::Continued {
            bonus2_flag,
            gend50_step,
        } = continued
        else {
            panic!("expected BON2 continuation");
        };
        assert_eq!(bonus2_flag, 1);
        assert_eq!(
            gend50_step,
            VSoundGEnd50Step::Updated(crate::sound::VSoundGEndUpdate {
                frequency_offset: 0xFF,
                frequency_pattern_address: 0xFF55,
                frequency_end_address: 0xFF62,
                result: VSoundGEnd61Result::RestartGWave {
                    waveform_reloaded: true,
                },
            })
        );
        assert_eq!(board.ram()[0x17], 0xFF);
        assert_eq!(&board.ram()[0x1B..0x1F], &[0xFF, 0x55, 0xFF, 0x62]);
    }

    #[test]
    fn vsnd_background2_setup_loads_trbv_and_filters_frequency_window() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .write_byte(0x0005, VSNDRM1_BG2_MAX)
            .expect("seed BG2FLG");

        let setup = board
            .run_vsnd_background2_setup()
            .expect("BG2 setup should load TRBV");

        let VSoundBackground2Setup {
            gwave_load,
            background2_flag,
            frequency_update,
        } = setup;
        assert_eq!(background2_flag, VSNDRM1_BG2_MAX);
        assert_eq!(gwave_load.table_index, 14);
        assert_eq!(gwave_load.vector_address, 0xFF4E);
        assert_eq!(gwave_load.echo_count, 1);
        assert_eq!(gwave_load.cycle_count, 2);
        assert_eq!(gwave_load.echo_decay, 0);
        assert_eq!(gwave_load.waveform_index, 6);
        assert_eq!(gwave_load.waveform_address, 0xFEDB);
        assert_eq!(gwave_load.frequency_delta, 0xFF);
        assert_eq!(gwave_load.frequency_delta_count, 1);
        assert_eq!(gwave_load.frequency_pattern_address, 0xFF7D);
        assert_eq!(gwave_load.frequency_end_address, 0xFF86);

        assert_eq!(frequency_update.frequency_offset, 0x8B);
        assert_eq!(frequency_update.frequency_pattern_address, 0xFF7D);
        assert_eq!(frequency_update.frequency_end_address, 0xFF80);
        assert_eq!(
            frequency_update.result,
            VSoundGEnd61Result::RestartGWave {
                waveform_reloaded: false,
            }
        );
        assert_eq!(&board.ram()[0x1B..0x1F], &[0xFF, 0x7D, 0xFF, 0x80]);
        assert_eq!(board.ram()[0x23], 0x8B);
        assert_eq!(
            &board.ram()[0x24..0x34],
            &[
                89, 123, 152, 172, 179, 172, 152, 123, 89, 55, 25, 6, 0, 6, 25, 55
            ]
        );
    }

    #[test]
    fn vsnd_irq3_background_flow_waits_without_clearing_bonus_when_inactive() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0007, 0x55).expect("seed B2FLG");

        assert_eq!(
            board.step_vsnd_irq3_background_flow(),
            Ok(VSoundIrq3BackgroundFlow::WaitingForBackground)
        );
        assert_eq!(board.ram()[0x07], 0x55);
        assert_eq!(board.last_dac_value(), None);
    }

    #[test]
    fn vsnd_irq3_background_flow_runs_bg1_before_bg2_and_kills_bonus() {
        let images = test_rom_images();
        let mut expected = DefenderSoundBoard::with_cleared_ram(&images);
        expected.write_byte(0x0009, 0x3C).expect("seed expected HI");
        expected.write_byte(0x000A, 0x00).expect("seed expected LO");
        let expected_window = expected.run_vsnd_background1_fnoise_window();

        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0x44).expect("seed BG1FLG");
        board.write_byte(0x0005, 0x55).expect("seed BG2FLG");
        board.write_byte(0x0007, 0xAA).expect("seed B2FLG");
        board.write_byte(0x0009, 0x3C).expect("seed HI");
        board.write_byte(0x000A, 0x00).expect("seed LO");

        assert_eq!(
            board.step_vsnd_irq3_background_flow(),
            Ok(VSoundIrq3BackgroundFlow::Background1(expected_window))
        );
        assert_eq!(board.ram()[0x04], 1);
        assert_eq!(board.ram()[0x05], 0x55);
        assert_eq!(board.ram()[0x07], 0);
        assert_eq!(board.last_dac_value(), expected.last_dac_value());
    }

    #[test]
    fn vsnd_irq3_background_flow_runs_bg2_setup_and_kills_bonus() {
        let images = test_rom_images_with_vsnd_tables();
        let mut expected = DefenderSoundBoard::with_cleared_ram(&images);
        expected
            .write_byte(0x0005, VSNDRM1_BG2_MAX)
            .expect("seed expected BG2FLG");
        let expected_setup = expected
            .run_vsnd_background2_setup()
            .expect("expected BG2 setup");

        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .write_byte(0x0005, VSNDRM1_BG2_MAX)
            .expect("seed BG2FLG");
        board.write_byte(0x0007, 0xAA).expect("seed B2FLG");

        assert_eq!(
            board.step_vsnd_irq3_background_flow(),
            Ok(VSoundIrq3BackgroundFlow::Background2(expected_setup))
        );
        assert_eq!(board.ram()[0x05], VSNDRM1_BG2_MAX);
        assert_eq!(board.ram()[0x07], 0);
        assert_eq!(&board.ram()[0x1B..0x1F], &[0xFF, 0x7D, 0xFF, 0x80]);
    }

    #[test]
    fn vsnd_irq3_background_flow_reports_bg2_rom_errors_after_killing_bonus() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0005, 1).expect("seed BG2FLG");
        board.write_byte(0x0007, 0xAA).expect("seed B2FLG");

        assert_eq!(
            board.step_vsnd_irq3_background_flow(),
            Err(VSoundGWaveLoadError::MissingSoundRomByte { address: 0xFF4E })
        );
        assert_eq!(board.ram()[0x07], 0);
        assert_eq!(board.last_dac_value(), None);
    }

    #[test]
    fn vsnd_gwave_start_outputs_first_source_period_and_sets_xplay() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");

        assert_eq!(
            board.run_vsnd_gwave_start(),
            Ok(VSoundGWaveStep::Playing(VSoundGWavePeriod {
                frequency_address: 0xFF55,
                next_frequency_address: 0xFF56,
                period: 0xA0,
                cycle_count: 1,
                waveform_cycles: 1,
                wave_ram_start: 0x24,
                wave_ram_end: 0x2C,
                dac_samples: vec![0, 64, 128, 0, 255, 0, 128, 64],
            }))
        );
        assert_eq!(&board.ram()[0x0D..0x0F], &[0xFF, 0x56]);
        assert_eq!(board.ram()[0x21], 0xA0);
        assert_eq!(board.ram()[0x22], 3);
        assert_eq!(board.last_dac_value(), Some(64));
    }

    #[test]
    fn vsnd_gwave_period_repeats_zero_cycle_count_as_wrapped_counter() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        board.write_byte(0x0014, 0).expect("seed GCCNT");

        let step = board
            .run_vsnd_gwave_start()
            .expect("GWAVE start should output period");
        let VSoundGWaveStep::Playing(period) = step else {
            panic!("expected playing GWAVE period");
        };

        assert_eq!(period.cycle_count, 0);
        assert_eq!(period.waveform_cycles, 256);
        assert_eq!(period.dac_samples.len(), 8 * 256);
        assert_eq!(&period.dac_samples[..8], &[0, 64, 128, 0, 255, 0, 128, 64]);
        assert_eq!(
            &period.dac_samples[period.dac_samples.len() - 8..],
            &[0, 64, 128, 0, 255, 0, 128, 64]
        );
        assert_eq!(board.last_dac_value(), Some(64));
    }

    #[test]
    fn vsnd_gwave_period_at_frequency_end_enters_gend_after_setting_period() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        board.write_byte(0x000D, 0xFF).expect("seed XPLAY high");
        board.write_byte(0x000E, 0x62).expect("seed XPLAY low");
        board.write_byte(0x0015, 0).expect("clear GECDEC");
        board.write_byte(0x0016, 0).expect("clear GDFINC");
        board.write_byte(0x0022, 1).expect("seed GECNT");
        board.write_byte(0x0023, 5).expect("seed FOFSET");

        assert_eq!(
            board.run_vsnd_gwave_period(),
            Ok(VSoundGWaveStep::Ended(VSoundGEndStep::Frequency(
                VSoundGEnd50Step::NoFrequencyDelta { frequency_delta: 0 },
            )))
        );
        assert_eq!(&board.ram()[0x0D..0x0F], &[0xFF, 0x62]);
        assert_eq!(board.ram()[0x21], 5);
        assert_eq!(board.ram()[0x22], 0);
    }

    #[test]
    fn vsnd_gwave_period_reports_missing_frequency_byte_before_state_writes() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x000D, 0xF7).expect("seed XPLAY high");
        board.write_byte(0x000E, 0xFF).expect("seed XPLAY low");

        assert_eq!(
            board.run_vsnd_gwave_period(),
            Err(VSoundGWaveLoadError::MissingSoundRomByte { address: 0xF7FF })
        );
        assert_eq!(board.ram()[0x21], 0);
    }

    #[test]
    fn vsnd_gwave_period_reports_invalid_wave_window_after_xplay_advance() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        board.write_byte(0x000D, 0xFF).expect("seed XPLAY high");
        board.write_byte(0x000E, 0x55).expect("seed XPLAY low");
        board.write_byte(0x001F, 0).expect("seed WVEND high");
        board.write_byte(0x0020, 0x23).expect("seed WVEND low");

        assert_eq!(
            board.run_vsnd_gwave_period(),
            Err(VSoundGWaveLoadError::InvalidWaveRamEnd { wave_ram_end: 0x23 })
        );
        assert_eq!(&board.ram()[0x0D..0x0F], &[0xFF, 0x56]);
        assert_eq!(board.ram()[0x21], 0xA0);
    }

    #[test]
    fn vsnd_gend_decays_wave_and_restarts_when_echoes_remain() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        board.write_byte(0x0022, 2).expect("seed GECNT");

        assert_eq!(
            board.run_vsnd_gend(),
            Ok(VSoundGEndStep::EchoRestart { echo_count: 1 })
        );

        assert_eq!(board.ram()[0x22], 1);
        assert_eq!(&board.ram()[0x24..0x2C], &[0, 60, 120, 0, 240, 0, 120, 60]);
    }

    #[test]
    fn vsnd_gend_stops_bonus_when_echoes_finish_with_bonus_flag() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        board.write_byte(0x0015, 0).expect("clear GECDEC");
        board.write_byte(0x0022, 1).expect("seed GECNT");
        board.write_byte(0x0007, 1).expect("seed B2FLG");

        assert_eq!(
            board.run_vsnd_gend(),
            Ok(VSoundGEndStep::BonusStopped { bonus2_flag: 1 })
        );
        assert_eq!(board.ram()[0x22], 0);
        assert_eq!(board.ram()[0x17], 0);
    }

    #[test]
    fn vsnd_gend_falls_through_to_gend50_terminal_branches() {
        let images = test_rom_images_with_vsnd_tables();
        let mut no_delta = DefenderSoundBoard::with_cleared_ram(&images);
        no_delta
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        no_delta.write_byte(0x0015, 0).expect("clear GECDEC");
        no_delta.write_byte(0x0016, 0).expect("clear GDFINC");
        no_delta.write_byte(0x0022, 1).expect("seed GECNT");

        assert_eq!(
            no_delta.run_vsnd_gend(),
            Ok(VSoundGEndStep::Frequency(
                VSoundGEnd50Step::NoFrequencyDelta { frequency_delta: 0 },
            ))
        );

        let mut expired = DefenderSoundBoard::with_cleared_ram(&images);
        expired
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        expired.write_byte(0x0015, 0).expect("clear GECDEC");
        expired.write_byte(0x0017, 1).expect("seed GDCNT");
        expired.write_byte(0x0022, 1).expect("seed GECNT");

        assert_eq!(
            expired.run_vsnd_gend(),
            Ok(VSoundGEndStep::Frequency(
                VSoundGEnd50Step::DeltaCountExpired {
                    frequency_delta_count: 0,
                },
            ))
        );
        assert_eq!(expired.ram()[0x17], 0);
    }

    #[test]
    fn vsnd_gend_reports_invalid_current_wave_window() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        board.write_byte(0x001F, 0).expect("seed WVEND high");
        board.write_byte(0x0020, 0x23).expect("seed WVEND low");

        assert_eq!(
            board.run_vsnd_gend(),
            Err(VSoundGWaveLoadError::InvalidWaveRamEnd { wave_ram_end: 0x23 })
        );
    }

    #[test]
    fn vsnd_gend40_propagates_gend50_frequency_rom_errors() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0016, 1).expect("seed GDFINC");
        board.write_byte(0x0017, 2).expect("seed GDCNT");
        board.write_byte(0x001B, 0xF7).expect("seed GWFRQ high");
        board.write_byte(0x001C, 0xFF).expect("seed GWFRQ low");
        board.write_byte(0x001D, 0xF8).expect("seed FRQEND high");
        board.write_byte(0x001E, 0x00).expect("seed FRQEND low");
        board.write_byte(0x0022, 1).expect("seed GECNT");

        assert_eq!(
            board.run_vsnd_gend40(),
            Err(VSoundGWaveLoadError::MissingSoundRomByte { address: 0xF7FF })
        );
        assert_eq!(board.ram()[0x17], 1);
        assert_eq!(board.ram()[0x22], 0);
        assert_eq!(board.ram()[0x23], 1);
    }

    #[test]
    fn vsnd_gend50_reports_all_over_when_frequency_window_has_no_start() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        board.write_byte(0x0017, 2).expect("seed GDCNT");
        board.write_byte(0x0023, 1).expect("seed FOFSET");

        assert_eq!(
            board.run_vsnd_gend50(),
            Ok(VSoundGEnd50Step::Updated(crate::sound::VSoundGEndUpdate {
                frequency_offset: 0,
                frequency_pattern_address: 0xFF55,
                frequency_end_address: 0xFF62,
                result: VSoundGEnd61Result::AllOver,
            },))
        );
        assert_eq!(board.ram()[0x17], 1);
        assert_eq!(board.ram()[0x23], 0);
    }

    #[test]
    fn vsnd_gend50_positive_delta_uses_carry_for_frequency_window() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .load_vsnd_gwave_vector(13)
            .expect("load BONV GWAVE state");
        board.write_byte(0x0016, 1).expect("seed positive GDFINC");
        board.write_byte(0x0017, 2).expect("seed GDCNT");
        board.write_byte(0x0023, 0xFF).expect("seed FOFSET");

        assert_eq!(
            board.run_vsnd_gend50(),
            Ok(VSoundGEnd50Step::Updated(crate::sound::VSoundGEndUpdate {
                frequency_offset: 0,
                frequency_pattern_address: 0xFF55,
                frequency_end_address: 0xFF62,
                result: VSoundGEnd61Result::RestartGWave {
                    waveform_reloaded: true,
                },
            },))
        );
        assert_eq!(board.ram()[0x17], 1);
        assert_eq!(board.ram()[0x23], 0);
    }

    #[test]
    fn vsnd_irq_dispatch_decodes_gwave_and_clears_non_matching_flags() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0006, 0x33).expect("seed SP1FLG");
        board.write_byte(0x0007, 0x44).expect("seed B2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        let dispatch = board.step_vsnd_irq_dispatch();

        assert_eq!(dispatch.latched_port_b, 0xDE);
        assert_eq!(dispatch.command_code, 1);
        assert_eq!(dispatch.effective_code, 1);
        assert_eq!(dispatch.organ_step, VSoundOrganIrqStep::Inactive);
        assert!(!dispatch.talking_program_present);
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::GWave {
                sound_code: 1,
                table_index: 0,
            }
        );
        assert_eq!(dispatch.spinner_flag, 0);
        assert_eq!(dispatch.bonus2_flag, 0);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );
    }

    #[test]
    fn vsnd_irq_dispatch_applies_source_special_flag_effects() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .write_byte(0x0006, VSNDRM1_SPINNER_MAX - 1)
            .expect("seed SP1FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(VSNDRM1_SPINNER_SOUND_CODE));

        let spinner = board.step_vsnd_irq_dispatch();

        assert_eq!(
            spinner.routine,
            VSoundIrqRoutine::Special {
                sound_code: VSNDRM1_SPINNER_SOUND_CODE,
                table_index: 0,
                routine: VSoundSpecialRoutine::Spinner1,
            }
        );
        assert_eq!(spinner.spinner_flag, 1);

        board
            .write_byte(0x0005, VSNDRM1_BG2_MAX)
            .expect("seed BG2FLG");
        board.write_byte(0x0007, 0xAA).expect("seed B2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(16));

        let background2 = board.step_vsnd_irq_dispatch();

        assert_eq!(
            background2.routine,
            VSoundIrqRoutine::Special {
                sound_code: 16,
                table_index: 2,
                routine: VSoundSpecialRoutine::Background2Increment,
            }
        );
        assert_eq!(background2.background1_flag, 0);
        assert_eq!(background2.background2_flag, 1);
        assert_eq!(background2.bonus2_flag, 0);
        assert_eq!(
            background2.background,
            VSoundBackgroundContinuation::Background2
        );
    }

    #[test]
    fn vsnd_irq_dispatch_preserves_bonus2_code_and_tracks_organ_shift() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(VSNDRM1_BONUS2_SOUND_CODE));

        let bonus = board.step_vsnd_irq_dispatch();

        assert_eq!(
            bonus.routine,
            VSoundIrqRoutine::Special {
                sound_code: VSNDRM1_BONUS2_SOUND_CODE,
                table_index: 4,
                routine: VSoundSpecialRoutine::Bonus2,
            }
        );
        assert_eq!(bonus.bonus2_flag, 1);
        assert_eq!(
            bonus.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        board.write_byte(0x0008, 0x03).expect("seed ORGFLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(2));
        let organ = board.step_vsnd_irq_dispatch();

        assert_eq!(organ.command_code, 2);
        assert_eq!(organ.effective_code, 1);
        assert_eq!(organ.organ_step, VSoundOrganIrqStep::Note);
        assert_eq!(
            organ.routine,
            VSoundIrqRoutine::GWave {
                sound_code: 1,
                table_index: 0,
            }
        );

        board.write_byte(0x0008, 0xFF).expect("seed tune ORGFLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(1));
        let tune_marker = board.step_vsnd_irq_dispatch();

        assert_eq!(tune_marker.command_code, 1);
        assert_eq!(tune_marker.effective_code, 0);
        assert_eq!(tune_marker.organ_step, VSoundOrganIrqStep::TuneAndNote);
    }

    #[test]
    fn vsnd_irq_organ_flow_reports_inactive_without_normal_dispatch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .write_byte(0x0006, 0x33)
            .expect("seed SP1FLG for no-op check");
        board.latch_main_board_sound_command(raw_for_vsound_code(27));

        let flow = board
            .step_vsnd_irq_organ_flow()
            .expect("inactive organ flow");
        assert_eq!(
            flow,
            VSoundIrqOrganFlow::Inactive {
                latched_port_b: 0xC4,
                command_code: 27,
            }
        );
        assert_eq!(irq_organ_tune_started(flow.clone()), None);
        assert_eq!(irq_organ_note_step(flow), None);
        assert_eq!(board.ram()[0x06], 0x33);
        assert_eq!(board.ram()[0x08], 0);
    }

    #[test]
    fn vsnd_irq_organ_flow_runs_source_tune_after_organt_flag() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(27));

        let dispatch = board.step_vsnd_irq_dispatch();

        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 27,
                table_index: 13,
                routine: VSoundSpecialRoutine::OrganTune,
            }
        );
        assert_eq!(board.ram()[0x08], 0xFF);

        board.latch_main_board_sound_command(raw_for_vsound_code(1));
        let flow = board
            .step_vsnd_irq_organ_flow()
            .expect("negative ORGFLG should run ORGNT1");
        let (latched_port_b, command_code, tune) =
            irq_organ_tune_started(flow).expect("expected organ tune branch");
        let tune = organ_tune_played(tune).expect("tune command 1 should play PHANTOM");

        assert_eq!(latched_port_b, 0xDE);
        assert_eq!(command_code, 1);
        assert_eq!(tune.table_address, 0xFDAA);
        assert_eq!(tune.notes.len(), 3);
        assert_eq!(board.ram()[0x08], 0);
        assert_eq!(&board.ram()[0x0D..0x11], &[0xFD, 0xB7, 0xFD, 0xB7]);
        assert_eq!(board.last_dac_value(), Some(64));
    }

    #[test]
    fn vsnd_irq_organ_flow_applies_source_note_parameter_decrements() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.run_vsnd_organ_note_start();
        board.write_byte(0x0007, 0x44).expect("seed B2FLG");

        board.latch_main_board_sound_command(raw_for_vsound_code(8));
        let high = board
            .step_vsnd_irq_organ_flow()
            .expect("first organ note parameter");
        assert_eq!(
            high,
            VSoundIrqOrganFlow::Note {
                latched_port_b: 0xD7,
                command_code: 8,
                parameter_code: 7,
                step: VSoundOrganNoteStep::MaskByte(crate::sound::VSoundOrganMaskByte {
                    parameter_code: 7,
                    organ_flag: 2,
                    oscillator_mask: 7,
                }),
            }
        );
        assert_eq!(board.ram()[0x07], 0x44);

        board.latch_main_board_sound_command(raw_for_vsound_code(13));
        let low = board
            .step_vsnd_irq_organ_flow()
            .expect("second organ note parameter");
        assert_eq!(
            low,
            VSoundIrqOrganFlow::Note {
                latched_port_b: 0xD2,
                command_code: 13,
                parameter_code: 12,
                step: VSoundOrganNoteStep::MaskByte(crate::sound::VSoundOrganMaskByte {
                    parameter_code: 12,
                    organ_flag: 1,
                    oscillator_mask: 0x7C,
                }),
            }
        );

        board.latch_main_board_sound_command(raw_for_vsound_code(8));
        let final_note = board
            .step_vsnd_irq_organ_flow()
            .expect("final organ note parameter");
        let (latched_port_b, command_code, parameter_code, step) =
            irq_organ_note_step(final_note).expect("expected organ note branch");
        let note = organ_note_started(step).expect("final parameter should start note");

        assert_eq!(latched_port_b, 0xD7);
        assert_eq!(command_code, 8);
        assert_eq!(parameter_code, 6);
        assert_eq!(note.note_delay, 0x1D);
        assert_eq!(note.window.duration, 0xFFFF);
        assert_eq!(note.window.oscillator_mask, 0x7C);
        assert_eq!(board.ram()[0x08], 0);
        assert_eq!(board.last_dac_value(), Some(80));
    }

    #[test]
    fn vsnd_irq_flow_runs_command_and_background_when_organ_inactive() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0x44).expect("seed BG1FLG");
        board.write_byte(0x0007, 0x66).expect("seed B2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(0));

        let flow = board.step_vsnd_irq_flow().expect("inactive organ IRQ");

        assert!(matches!(
            flow,
            VSoundIrqFlow::Command(crate::sound::VSoundIrqCommandAndBackgroundFlow {
                command: VSoundIrqCommandFlow::Invalid {
                    dispatch: VSoundIrqDispatch {
                        command_code: 0,
                        organ_step: VSoundOrganIrqStep::Inactive,
                        ..
                    },
                },
                irq3: VSoundIrq3AfterCommand::Entered(VSoundIrq3BackgroundFlow::Background1(_)),
            })
        ));
        assert_eq!(board.ram()[0x04], 1);
        assert_eq!(board.ram()[0x07], 0);
    }

    #[test]
    fn vsnd_irq_flow_runs_organ_note_without_normal_dispatch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.run_vsnd_organ_note_start();
        board.write_byte(0x0004, 0x44).expect("seed BG1FLG");
        board.write_byte(0x0006, 0x55).expect("seed SP1FLG");
        board.write_byte(0x0007, 0x66).expect("seed B2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(8));

        assert_eq!(
            board.step_vsnd_irq_flow(),
            Ok(VSoundIrqFlow::OrganNote {
                organ: VSoundIrqOrganFlow::Note {
                    latched_port_b: 0xD7,
                    command_code: 8,
                    parameter_code: 7,
                    step: VSoundOrganNoteStep::MaskByte(crate::sound::VSoundOrganMaskByte {
                        parameter_code: 7,
                        organ_flag: 2,
                        oscillator_mask: 7,
                    }),
                },
            })
        );
        assert_eq!(&board.ram()[0x04..0x08], &[0x44, 0, 0x55, 0x66]);
        assert_eq!(board.ram()[0x08], 2);
        assert_eq!(board.last_dac_value(), None);
    }

    #[test]
    fn vsnd_irq_flow_runs_organ_tune_then_irq3_background() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0x44).expect("seed BG1FLG");
        board.write_byte(0x0007, 0x66).expect("seed B2FLG");
        board.write_byte(0x0008, 0xFF).expect("seed ORGFLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        let flow = board.step_vsnd_irq_flow().expect("organ tune IRQ");
        assert!(matches!(
            flow,
            VSoundIrqFlow::OrganTune {
                organ: VSoundIrqOrganFlow::Tune {
                    latched_port_b: 0xDE,
                    command_code: 1,
                    tune: VSoundOrganTuneStep::Played(crate::sound::VSoundOrganTune {
                        table_address: 0xFDAA,
                        notes,
                        ..
                    }),
                },
                irq3: VSoundIrq3AfterCommand::Entered(VSoundIrq3BackgroundFlow::Background1(_)),
            } if notes.len() == 3
        ));
        assert_eq!(board.ram()[0x04], 1);
        assert_eq!(board.ram()[0x07], 0);
        assert_eq!(board.ram()[0x08], 0);
        assert_eq!(board.last_dac_value(), Some(0));
    }

    #[test]
    fn vsnd_irq_flow_wraps_command_errors_when_organ_inactive() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        assert_eq!(
            board.step_vsnd_irq_flow(),
            Err(VSoundIrqFlowError::Command(
                VSoundIrqCommandAndBackgroundFlowError::Command(VSoundIrqCommandFlowError::GWave(
                    VSoundGWaveLoadError::MissingSoundRomByte { address: 0xFEEC },
                ),),
            ))
        );
    }

    #[test]
    fn vsnd_irq_flow_wraps_organ_errors_before_command_dispatch() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0x44).expect("seed BG1FLG");
        board.write_byte(0x0007, 0x66).expect("seed B2FLG");
        board.write_byte(0x0008, 0xFF).expect("seed ORGFLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        assert_eq!(
            board.step_vsnd_irq_flow(),
            Err(VSoundIrqFlowError::Organ(
                VSoundOrganLoadError::MissingSoundRomByte { address: 0xFDAA },
            ))
        );
        assert_eq!(board.ram()[0x04], 0x44);
        assert_eq!(board.ram()[0x07], 0x66);
        assert_eq!(board.ram()[0x08], 0);
    }

    #[test]
    fn vsnd_irq_flow_reports_irq3_background_errors_after_organ_tune() {
        let mut sound = vec![0; 0x5AB];
        write_test_sound_rom(&mut sound, 0xFDAA, &[0]);
        let images = test_rom_images_from_sound(sound);
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0005, 0x44).expect("seed BG2FLG");
        board.write_byte(0x0007, 0x66).expect("seed B2FLG");
        board.write_byte(0x0008, 0xFF).expect("seed ORGFLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        assert_eq!(
            board.step_vsnd_irq_flow(),
            Err(VSoundIrqFlowError::Background(
                VSoundGWaveLoadError::MissingSoundRomByte { address: 0xFF4E },
            ))
        );
        assert_eq!(board.ram()[0x07], 0);
        assert_eq!(board.ram()[0x08], 0);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_source_gwave_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(1));

        assert_eq!(
            board
                .step_vsnd_irq_command_flow()
                .expect("GWAVE IRQ command should run"),
            VSoundIrqCommandFlow::GWave {
                dispatch: VSoundIrqDispatch {
                    latched_port_b: 0xDE,
                    command_code: 1,
                    effective_code: 1,
                    organ_step: VSoundOrganIrqStep::Inactive,
                    talking_program_present: false,
                    routine: VSoundIrqRoutine::GWave {
                        sound_code: 1,
                        table_index: 0,
                    },
                    spinner_flag: 0,
                    bonus2_flag: 0,
                    background1_flag: 0,
                    background2_flag: 0,
                    background: VSoundBackgroundContinuation::WaitingForBackground,
                },
                load: VSoundGWaveLoad {
                    table_index: 0,
                    vector_address: 0xFEEC,
                    echo_count: 8,
                    cycle_count: 1,
                    echo_decay: 2,
                    waveform_index: 4,
                    waveform_address: 0xFE81,
                    waveform_length: 16,
                    predecay_factor: 0,
                    frequency_delta: 0,
                    frequency_delta_count: 0,
                    frequency_pattern_address: 0xFF86,
                    frequency_pattern_length: 22,
                    frequency_end_address: 0xFF9C,
                    wave_ram_start: 0x24,
                    wave_ram_end: 0x34,
                },
                step: VSoundGWaveStep::Playing(VSoundGWavePeriod {
                    frequency_address: 0xFF86,
                    next_frequency_address: 0xFF87,
                    period: 0,
                    cycle_count: 1,
                    waveform_cycles: 1,
                    wave_ram_start: 0x24,
                    wave_ram_end: 0x34,
                    dac_samples: vec![
                        0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0,
                    ],
                }),
            }
        );
        assert_eq!(&board.ram()[0x0D..0x0F], &[0xFF, 0x87]);
        assert_eq!(board.last_dac_value(), Some(0));
    }

    #[test]
    fn vsnd_irq_command_flow_runs_source_vari_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(29));

        assert_eq!(
            board
                .step_vsnd_irq_command_flow()
                .expect("VARI IRQ command should run"),
            VSoundIrqCommandFlow::Vari {
                dispatch: VSoundIrqDispatch {
                    latched_port_b: 0xC2,
                    command_code: 29,
                    effective_code: 29,
                    organ_step: VSoundOrganIrqStep::Inactive,
                    talking_program_present: false,
                    routine: VSoundIrqRoutine::Vari {
                        sound_code: 29,
                        table_index: 0,
                    },
                    spinner_flag: 0,
                    bonus2_flag: 0,
                    background1_flag: 0,
                    background2_flag: 0,
                    background: VSoundBackgroundContinuation::WaitingForBackground,
                },
                load: VSoundVariLoad {
                    table_index: 0,
                    vector_address: 0xFD76,
                    low_period: 0x40,
                    high_period: 0x01,
                    low_period_delta: 0,
                    high_period_delta: 0x10,
                    high_period_end: 0xE1,
                    sweep_period: 0x0080,
                    low_period_mod: 0xFF,
                    amplitude: 0xFF,
                },
                sweep: VSoundVariSweep {
                    sweep_period: 0x0080,
                    low_count: 0x40,
                    high_count: 0x01,
                    low_count_after_sweep: 0x40,
                    high_count_after_sweep: 0x11,
                    dac_samples: vec![0xFF, 0, 0xFF, 0, 0xFF],
                    result: VSoundVariSweepResult::Continue,
                },
            }
        );
        assert_eq!(&board.ram()[0x1C..0x1E], &[0x40, 0x11]);
        assert_eq!(board.last_dac_value(), Some(0xFF));
    }

    #[test]
    fn vsnd_irq_command_flow_reports_invalid_and_runs_organ_note_special_flag_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(0));

        let invalid_flow = board
            .step_vsnd_irq_command_flow()
            .expect("invalid IRQ command should classify");
        assert_eq!(
            invalid_flow,
            VSoundIrqCommandFlow::Invalid {
                dispatch: VSoundIrqDispatch {
                    latched_port_b: 0xDF,
                    command_code: 0,
                    effective_code: 0,
                    organ_step: VSoundOrganIrqStep::Inactive,
                    talking_program_present: false,
                    routine: VSoundIrqRoutine::Invalid,
                    spinner_flag: 0,
                    bonus2_flag: 0,
                    background1_flag: 0,
                    background2_flag: 0,
                    background: VSoundBackgroundContinuation::WaitingForBackground,
                },
            }
        );
        assert_eq!(irq_command_special_step(invalid_flow), None);

        board.latch_main_board_sound_command(raw_for_vsound_code(28));

        let special_flow = board
            .step_vsnd_irq_command_flow()
            .expect("special IRQ command should classify");
        assert_eq!(
            special_flow,
            VSoundIrqCommandFlow::Special {
                dispatch: VSoundIrqDispatch {
                    latched_port_b: 0xC3,
                    command_code: 28,
                    effective_code: 28,
                    organ_step: VSoundOrganIrqStep::Inactive,
                    talking_program_present: false,
                    routine: VSoundIrqRoutine::Special {
                        sound_code: 28,
                        table_index: 14,
                        routine: VSoundSpecialRoutine::OrganNote,
                    },
                    spinner_flag: 0,
                    bonus2_flag: 0,
                    background1_flag: 0,
                    background2_flag: 0,
                    background: VSoundBackgroundContinuation::WaitingForBackground,
                },
                step: VSoundIrqSpecialFlow::OrganNote(VSoundOrganNoteStart { organ_flag: 3 }),
            }
        );
        let (_, organ_step) =
            irq_command_special_step(special_flow).expect("special step should extract");
        assert_eq!(
            organ_note_special_start(organ_step.clone()),
            Some(VSoundOrganNoteStart { organ_flag: 3 })
        );
        assert_eq!(board.ram()[0x08], 3);
        assert_eq!(spinner1_special_command(organ_step.clone()), None);
        assert_eq!(bonus2_special_command(organ_step.clone()), None);
        assert_eq!(organ_tune_special_start(organ_step.clone()), None);
        assert_eq!(lite_special_noise(organ_step.clone()), None);
        assert_eq!(background1_special_window(organ_step.clone()), None);
        assert_eq!(
            background2_increment_special_setup(organ_step.clone()),
            None
        );
        assert_eq!(turbo_special_noise(organ_step.clone()), None);
        assert_eq!(appear_special_noise(organ_step.clone()), None);
        assert_eq!(background_end_special_flags(organ_step.clone()), None);
        assert_eq!(thrust_special_window(organ_step), None);
        assert_eq!(
            cannon_special_noise(VSoundIrqSpecialFlow::Deferred {
                routine: VSoundSpecialRoutine::Spinner1,
            }),
            None
        );
        assert_eq!(
            hyper_special_sweep(VSoundIrqSpecialFlow::Deferred {
                routine: VSoundSpecialRoutine::Spinner1,
            }),
            None
        );
        assert_eq!(
            radio_special_wave(VSoundIrqSpecialFlow::Deferred {
                routine: VSoundSpecialRoutine::Spinner1,
            }),
            None
        );
        assert_eq!(
            scream_special(VSoundIrqSpecialFlow::Deferred {
                routine: VSoundSpecialRoutine::Spinner1,
            }),
            None
        );
        assert_eq!(&board.ram()[0x13..0x1E], &[0; 11]);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_organ_tune_special_flag_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(27));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("ORGANT IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("ORGANT special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 27,
                table_index: 13,
                routine: VSoundSpecialRoutine::OrganTune,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xC4);
        assert_eq!(dispatch.command_code, 27);
        assert_eq!(dispatch.effective_code, 27);
        assert_eq!(dispatch.organ_step, VSoundOrganIrqStep::Inactive);
        assert_eq!(board.ram()[0x08], 0xFF);
        assert_eq!(
            organ_tune_special_start(step.clone()),
            Some(VSoundOrganTuneStart { organ_flag: 0xFF })
        );
        assert_eq!(organ_note_special_start(step), None);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_bonus2_special_start_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(VSNDRM1_BONUS2_SOUND_CODE));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("BON2 IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("BON2 special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: VSNDRM1_BONUS2_SOUND_CODE,
                table_index: 4,
                routine: VSoundSpecialRoutine::Bonus2,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xCD);
        assert_eq!(dispatch.command_code, VSNDRM1_BONUS2_SOUND_CODE);
        assert_eq!(dispatch.effective_code, VSNDRM1_BONUS2_SOUND_CODE);
        assert_eq!(dispatch.bonus2_flag, 1);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        let command = bonus2_special_command(step).expect("BON2 should execute BONV GWAVE");
        assert_eq!(
            command,
            VSoundBonus2Command::Started {
                gwave_load: VSoundGWaveLoad {
                    table_index: 13,
                    vector_address: 0xFF47,
                    echo_count: 3,
                    cycle_count: 1,
                    echo_decay: 1,
                    waveform_index: 1,
                    waveform_address: 0xFE56,
                    waveform_length: 8,
                    predecay_factor: 0,
                    frequency_delta: 0xFF,
                    frequency_delta_count: 0,
                    frequency_pattern_address: 0xFF55,
                    frequency_pattern_length: 0x0D,
                    frequency_end_address: 0xFF62,
                    wave_ram_start: 0x24,
                    wave_ram_end: 0x2C,
                },
                bonus2_flag: 1,
                step: VSoundGWaveStep::Playing(VSoundGWavePeriod {
                    frequency_address: 0xFF55,
                    next_frequency_address: 0xFF56,
                    period: 0xA0,
                    cycle_count: 1,
                    waveform_cycles: 1,
                    wave_ram_start: 0x24,
                    wave_ram_end: 0x2C,
                    dac_samples: vec![0, 64, 128, 0, 255, 0, 128, 64],
                }),
            }
        );
        assert_eq!(board.ram()[0x07], 1);
        assert_eq!(&board.ram()[0x0D..0x0F], &[0xFF, 0x56]);
        assert_eq!(board.ram()[0x21], 0xA0);
        assert_eq!(board.ram()[0x22], 3);
        assert_eq!(board.last_dac_value(), Some(64));
    }

    #[test]
    fn vsnd_irq_command_flow_runs_bonus2_special_continuation_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .run_vsnd_bonus2_setup()
            .expect("seed active BON2 GWAVE state");
        board.latch_main_board_sound_command(raw_for_vsound_code(VSNDRM1_BONUS2_SOUND_CODE));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("active BON2 IRQ command should continue");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("BON2 special command should extract");
        assert_eq!(dispatch.bonus2_flag, 1);
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: VSNDRM1_BONUS2_SOUND_CODE,
                table_index: 4,
                routine: VSoundSpecialRoutine::Bonus2,
            }
        );

        assert_eq!(
            bonus2_special_command(step).expect("BON2 should run GEND50 continuation"),
            VSoundBonus2Command::Continued {
                bonus2_flag: 1,
                gend50_step: VSoundGEnd50Step::Updated(crate::sound::VSoundGEndUpdate {
                    frequency_offset: 0xFF,
                    frequency_pattern_address: 0xFF55,
                    frequency_end_address: 0xFF62,
                    result: VSoundGEnd61Result::RestartGWave {
                        waveform_reloaded: true,
                    },
                }),
            }
        );
        assert_eq!(board.ram()[0x17], 0xFF);
        assert_eq!(&board.ram()[0x1B..0x1F], &[0xFF, 0x55, 0xFF, 0x62]);
    }

    #[test]
    fn vsnd_irq_command_flow_reports_bonus2_gwave_rom_errors_after_source_flag_effect() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(VSNDRM1_BONUS2_SOUND_CODE));

        assert_eq!(
            board.step_vsnd_irq_command_flow(),
            Err(VSoundIrqCommandFlowError::Bonus2(
                VSoundGWaveLoadError::MissingSoundRomByte { address: 0xFF47 },
            ))
        );
        assert_eq!(board.ram()[0x07], 1);
        assert_eq!(board.last_dac_value(), None);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_spinner1_special_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board
            .write_byte(0x0006, VSNDRM1_SPINNER_MAX - 1)
            .expect("seed SP1FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(VSNDRM1_SPINNER_SOUND_CODE));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("SP1 IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("SP1 special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: VSNDRM1_SPINNER_SOUND_CODE,
                table_index: 0,
                routine: VSoundSpecialRoutine::Spinner1,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xD1);
        assert_eq!(dispatch.command_code, VSNDRM1_SPINNER_SOUND_CODE);
        assert_eq!(dispatch.effective_code, VSNDRM1_SPINNER_SOUND_CODE);
        assert_eq!(dispatch.spinner_flag, 1);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        let command = spinner1_special_command(step).expect("SP1 should execute CABSHK VARI");
        assert_eq!(command.setup.vari_load.table_index, 3);
        assert_eq!(command.setup.vari_load.vector_address, 0xFD91);
        assert_eq!(command.setup.spinner_flag, 1);
        assert_eq!(command.setup.low_period, 254);
        assert_eq!(command.sweep.sweep_period, 0x0480);
        assert_eq!(command.sweep.low_count, 0xFE);
        assert_eq!(command.sweep.high_count, 0x01);
        assert_eq!(command.sweep.low_count_after_sweep, 0xFE);
        assert_eq!(command.sweep.high_count_after_sweep, 0x19);
        assert_eq!(
            command.sweep.dac_samples,
            vec![0xFF, 0, 0xFF, 0, 0xFF, 0, 0xFF, 0, 0xFF, 0, 0xFF]
        );
        assert_eq!(command.sweep.result, VSoundVariSweepResult::Continue);
        assert_eq!(
            &board.ram()[0x13..0x1E],
            &[0xFE, 0x01, 0, 0x18, 0x41, 0x04, 0x80, 0, 0xFF, 0xFE, 0x19]
        );
        assert_eq!(board.ram()[0x06], 1);
        assert_eq!(board.last_dac_value(), Some(0xFF));
    }

    #[test]
    fn vsnd_irq_command_flow_reports_spinner1_vari_rom_errors_after_source_flag_effect() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(VSNDRM1_SPINNER_SOUND_CODE));

        assert_eq!(
            board.step_vsnd_irq_command_flow(),
            Err(VSoundIrqCommandFlowError::Spinner1(
                VSoundVariLoadError::MissingSoundRomByte { address: 0xFD91 },
            ))
        );
        assert_eq!(board.ram()[0x06], 1);
        assert_eq!(board.last_dac_value(), None);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_background1_special_branch() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x3C).expect("seed random HI");
        board.write_byte(0x000A, 0x00).expect("seed random LO");
        board.latch_main_board_sound_command(raw_for_vsound_code(15));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("BG1 IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("BG1 special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 15,
                table_index: 1,
                routine: VSoundSpecialRoutine::Background1,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xD0);
        assert_eq!(dispatch.command_code, 15);
        assert_eq!(dispatch.effective_code, 15);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::Background1
        );

        let noise = background1_special_window(step).expect("BG1 should execute FNOISE window");
        assert_eq!(noise.entry_address, 0xF913);
        assert_eq!(noise.initial_sample_count, 0xF913);
        assert_eq!(noise.initial_max_frequency, 1);
        assert_eq!(noise.final_frequency_high, 1);
        assert_eq!(noise.final_frequency_low, 0);
        assert!(!noise.frequency_decay_enabled);
        assert!(!noise.distortion_enabled);
        assert_eq!(noise.background1_flag, 1);
        assert!(noise.continues_after_window);
        assert_eq!(noise.random_steps, 1000);
        assert_eq!(noise.random_seed_hi, 0x87);
        assert_eq!(noise.random_seed_lo, 0xB5);
        assert_eq!(noise.dac_samples.len(), 64761);
        assert_eq!(count_dac_samples(&noise.dac_samples, 0), 11);
        assert_eq!(count_dac_samples(&noise.dac_samples, 1), 11);
        assert_eq!(count_dac_samples(&noise.dac_samples, 7), 36);
        assert_eq!(count_dac_samples(&noise.dac_samples, 0xFF), 0);
        assert_eq!(board.last_dac_value(), Some(0xA7));
        assert_eq!(&board.ram()[0x13..0x1A], &[1, 1, 0, 0xF9, 0x13, 0, 0]);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_background2_increment_special_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0xAA).expect("seed BG1FLG");
        board
            .write_byte(0x0005, VSNDRM1_BG2_MAX)
            .expect("seed BG2FLG");
        board.write_byte(0x0007, 0x55).expect("seed B2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(16));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("BG2INC IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("BG2INC special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 16,
                table_index: 2,
                routine: VSoundSpecialRoutine::Background2Increment,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xCF);
        assert_eq!(dispatch.command_code, 16);
        assert_eq!(dispatch.effective_code, 16);
        assert_eq!(dispatch.spinner_flag, 0);
        assert_eq!(dispatch.bonus2_flag, 0);
        assert_eq!(dispatch.background1_flag, 0);
        assert_eq!(dispatch.background2_flag, 1);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::Background2
        );

        assert_eq!(
            background2_increment_special_setup(step),
            Some(VSoundBackground2Setup {
                gwave_load: VSoundGWaveLoad {
                    table_index: 14,
                    vector_address: 0xFF4E,
                    echo_count: 1,
                    cycle_count: 2,
                    echo_decay: 0,
                    waveform_index: 6,
                    waveform_address: 0xFEDB,
                    waveform_length: 16,
                    predecay_factor: 0,
                    frequency_delta: 0xFF,
                    frequency_delta_count: 1,
                    frequency_pattern_address: 0xFF7D,
                    frequency_pattern_length: 9,
                    frequency_end_address: 0xFF86,
                    wave_ram_start: 0x24,
                    wave_ram_end: 0x34,
                },
                background2_flag: 1,
                frequency_update: crate::sound::VSoundGEndUpdate {
                    frequency_offset: 0xFB,
                    frequency_pattern_address: 0xFF7D,
                    frequency_end_address: 0xFF86,
                    result: VSoundGEnd61Result::RestartGWave {
                        waveform_reloaded: false,
                    },
                },
            })
        );
        assert_eq!(&board.ram()[0x04..0x08], &[0, 1, 0, 0]);
        assert_eq!(board.ram()[0x23], 0xFB);
        assert_eq!(&board.ram()[0x1B..0x1F], &[0xFF, 0x7D, 0xFF, 0x86]);
    }

    #[test]
    fn vsnd_irq_command_flow_reports_background2_gwave_rom_errors_after_source_flag_effect() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0xAA).expect("seed BG1FLG");
        board
            .write_byte(0x0005, VSNDRM1_BG2_MAX)
            .expect("seed BG2FLG");
        board.write_byte(0x0007, 0x55).expect("seed B2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(16));

        assert_eq!(
            board.step_vsnd_irq_command_flow(),
            Err(VSoundIrqCommandFlowError::Background2(
                VSoundGWaveLoadError::MissingSoundRomByte { address: 0xFF4E },
            ))
        );
        assert_eq!(&board.ram()[0x04..0x08], &[0, 1, 0, 0]);
        assert_eq!(board.last_dac_value(), None);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_lite_special_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x3C).expect("seed random HI");
        board.write_byte(0x000A, 0x00).expect("seed random LO");
        board.latch_main_board_sound_command(raw_for_vsound_code(17));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("LITE IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("LITE special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 17,
                table_index: 3,
                routine: VSoundSpecialRoutine::Lite,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xCE);
        assert_eq!(dispatch.command_code, 17);
        assert_eq!(dispatch.effective_code, 17);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        let noise = lite_special_noise(step).expect("LITE should execute LITEN");
        assert_eq!(noise.initial_frequency, 1);
        assert_eq!(noise.final_frequency, 0);
        assert_eq!(noise.frequency_delta, 1);
        assert_eq!(noise.cycle_count, 3);
        assert_eq!(noise.frequency_passes, 255);
        assert_eq!(noise.random_steps, 765);
        assert_eq!(noise.random_seed_hi, 0xC6);
        assert_eq!(noise.random_seed_lo, 0x09);
        assert_eq!(noise.dac_samples.len(), 386);
        assert_alternating_dac_transitions(&noise.dac_samples);
        assert_eq!(board.last_dac_value(), Some(0));
        assert_eq!(&board.ram()[0x15..0x1B], &[3, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_background_end_special_branch() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0004, 0x77).expect("seed BG1FLG");
        board.write_byte(0x0005, 0x88).expect("seed BG2FLG");
        board.write_byte(0x0007, 0x55).expect("seed B2FLG");
        board.latch_main_board_sound_command(raw_for_vsound_code(19));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("BGEND IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("BGEND special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 19,
                table_index: 5,
                routine: VSoundSpecialRoutine::BackgroundEnd,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xCC);
        assert_eq!(dispatch.command_code, 19);
        assert_eq!(dispatch.effective_code, 19);
        assert_eq!(dispatch.spinner_flag, 0);
        assert_eq!(dispatch.bonus2_flag, 0);
        assert_eq!(dispatch.background1_flag, 0);
        assert_eq!(dispatch.background2_flag, 0);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        assert_eq!(
            background_end_special_flags(step),
            Some(VSoundBackgroundFlags {
                background1_flag: 0,
                background2_flag: 0,
            })
        );
        assert_eq!(&board.ram()[0x04..0x08], &[0, 0, 0, 0]);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_appear_special_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x12).expect("seed random HI");
        board.write_byte(0x000A, 0x34).expect("seed random LO");
        board.latch_main_board_sound_command(raw_for_vsound_code(21));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("APPEAR IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("APPEAR special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 21,
                table_index: 7,
                routine: VSoundSpecialRoutine::Appear,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xCA);
        assert_eq!(dispatch.command_code, 21);
        assert_eq!(dispatch.effective_code, 21);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        let noise = appear_special_noise(step).expect("APPEAR should execute LITEN");
        assert_eq!(noise.initial_frequency, 0xC0);
        assert_eq!(noise.final_frequency, 0);
        assert_eq!(noise.frequency_delta, 0xFE);
        assert_eq!(noise.cycle_count, 0x10);
        assert_eq!(noise.frequency_passes, 96);
        assert_eq!(noise.random_steps, 1536);
        assert_eq!(noise.random_seed_hi, 0xA2);
        assert_eq!(noise.random_seed_lo, 0x5C);
        assert_eq!(noise.dac_samples.len(), 765);
        assert_alternating_dac_transitions(&noise.dac_samples);
        assert_eq!(board.last_dac_value(), Some(0xFF));
        assert_eq!(&board.ram()[0x15..0x1B], &[0x10, 0, 0, 0, 0, 0xFE]);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_thrust_special_branch() {
        let images = test_rom_images();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x3C).expect("seed random HI");
        board.write_byte(0x000A, 0x00).expect("seed random LO");
        board.latch_main_board_sound_command(raw_for_vsound_code(22));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("THRUST IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("THRUST special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 22,
                table_index: 8,
                routine: VSoundSpecialRoutine::Thrust,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xC9);
        assert_eq!(dispatch.command_code, 22);
        assert_eq!(dispatch.effective_code, 22);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        let noise = thrust_special_window(step).expect("THRUST should execute FNOISE window");
        assert_eq!(noise.entry_address, 0xF91C);
        assert_eq!(noise.initial_sample_count, 0xF91C);
        assert_eq!(noise.initial_max_frequency, 3);
        assert_eq!(noise.final_frequency_high, 3);
        assert_eq!(noise.final_frequency_low, 0);
        assert!(!noise.frequency_decay_enabled);
        assert!(!noise.distortion_enabled);
        assert_eq!(noise.background1_flag, 0);
        assert!(noise.continues_after_window);
        assert_eq!(noise.random_steps, 2903);
        assert_eq!(noise.random_seed_hi, 0xDF);
        assert_eq!(noise.random_seed_lo, 0x2D);
        assert_eq!(noise.dac_samples.len(), 66673);
        assert_eq!(count_dac_samples(&noise.dac_samples, 0), 15);
        assert_eq!(count_dac_samples(&noise.dac_samples, 3), 28);
        assert_eq!(count_dac_samples(&noise.dac_samples, 7), 55);
        assert_eq!(count_dac_samples(&noise.dac_samples, 0xFF), 6);
        assert_eq!(board.last_dac_value(), Some(0x5A));
        assert_eq!(&board.ram()[0x13..0x1A], &[3, 3, 0, 0xF9, 0x1C, 0, 0]);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_turbo_special_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x3C).expect("seed random HI");
        board.write_byte(0x000A, 0x00).expect("seed random LO");
        board.latch_main_board_sound_command(raw_for_vsound_code(20));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("TURBO IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("TURBO special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 20,
                table_index: 6,
                routine: VSoundSpecialRoutine::Turbo,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xCB);
        assert_eq!(dispatch.command_code, 20);
        assert_eq!(dispatch.effective_code, 20);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        let noise = turbo_special_noise(step).expect("TURBO should execute NOISE");
        assert_eq!(noise.initial_period, 1);
        assert_eq!(noise.final_period, 255);
        assert_eq!(noise.initial_amplitude, 0xFF);
        assert_eq!(noise.final_amplitude, 1);
        assert_eq!(noise.decay, 1);
        assert_eq!(noise.cycle_count, 0x20);
        assert_eq!(noise.amplitude_passes, 255);
        assert_eq!(noise.random_steps, 8160);
        assert_eq!(noise.random_seed_hi, 0x7C);
        assert_eq!(noise.random_seed_lo, 0x66);
        assert_eq!(noise.dac_samples.len(), 8160);
        assert_eq!(count_dac_samples(&noise.dac_samples, 0), 4189);
        assert_eq!(
            noise
                .dac_samples
                .iter()
                .filter(|&&sample| sample != 0)
                .count(),
            3971
        );
        assert_eq!(board.last_dac_value(), Some(1));
        assert_eq!(&board.ram()[0x13..0x19], &[1, 1, 0x20, 0, 0xFF, 0x20]);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_cannon_special_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.write_byte(0x0009, 0x3C).expect("seed random HI");
        board.write_byte(0x000A, 0x00).expect("seed random LO");
        board.latch_main_board_sound_command(raw_for_vsound_code(23));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("CANNON IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("CANNON special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 23,
                table_index: 9,
                routine: VSoundSpecialRoutine::Cannon,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xC8);
        assert_eq!(dispatch.command_code, 23);
        assert_eq!(dispatch.effective_code, 23);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        let noise = cannon_special_noise(step).expect("CANNON should execute FNOISE");
        assert_eq!(noise.initial_sample_count, 1000);
        assert_eq!(noise.initial_max_frequency, 0xFF);
        assert_eq!(noise.final_max_frequency, 0);
        assert_eq!(noise.final_frequency_high, 0);
        assert_eq!(noise.final_frequency_low, 7);
        assert!(noise.frequency_decay_enabled);
        assert!(noise.distortion_enabled);
        assert_eq!(noise.decay_passes, 72);
        assert_eq!(noise.random_steps, 1817);
        assert_eq!(noise.random_seed_hi, 0xE2);
        assert_eq!(noise.random_seed_lo, 0x56);
        assert_eq!(noise.dac_samples.len(), 73673);
        assert_eq!(count_dac_samples(&noise.dac_samples, 0), 35);
        assert_eq!(count_dac_samples(&noise.dac_samples, 0xFF), 41);
        assert_eq!(board.last_dac_value(), Some(0x90));
        assert_eq!(&board.ram()[0x13..0x1A], &[0, 0, 7, 0x03, 0xE8, 1, 1]);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_radio_special_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(24));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("RADIO IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("RADIO special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 24,
                table_index: 10,
                routine: VSoundSpecialRoutine::Radio,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xC7);
        assert_eq!(dispatch.command_code, 24);
        assert_eq!(dispatch.effective_code, 24);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        let radio = radio_special_wave(step).expect("RADIO should execute RADSND");
        assert_eq!(radio.table_address, 0xFD9A);
        assert_eq!(
            radio.table,
            [
                0x8C, 0x5B, 0xB6, 0x40, 0xBF, 0x49, 0xA4, 0x73, 0x73, 0xA4, 0x49, 0xBF, 0x40, 0xB6,
                0x5B, 0x8C,
            ]
        );
        assert_eq!(radio.initial_frequency, 100);
        assert_eq!(radio.final_frequency, 0xFFFF);
        assert_eq!(radio.initial_timer_high, 0);
        assert_eq!(radio.final_timer_high, 0xF7);
        assert_eq!(radio.final_timer_low, 0x2C);
        assert_eq!(radio.successful_frequency_increments, 65435);
        assert_eq!(radio.dac_samples.len(), 425344);
        assert_eq!(count_dac_samples(&radio.dac_samples, 0x40), 53165);
        assert_eq!(count_dac_samples(&radio.dac_samples, 0x73), 53297);
        assert_eq!(count_dac_samples(&radio.dac_samples, 0xBF), 53259);
        assert_eq!(board.last_dac_value(), Some(0x73));
        assert_eq!(
            &board.ram()[0x0B..0x12],
            &[0xFF, 0xFF, 0, 0, 0xFD, 0xA1, 0xF7]
        );
    }

    #[test]
    fn vsnd_irq_command_flow_reports_radio_table_rom_errors() {
        let images = test_rom_images_without_sound_load();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(24));

        assert_eq!(
            board.step_vsnd_irq_command_flow(),
            Err(VSoundIrqCommandFlowError::Radio(
                VSoundGWaveLoadError::MissingSoundRomByte { address: 0xFD9A },
            ))
        );
        assert_eq!(board.last_dac_value(), None);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_hyper_special_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(25));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("HYPER IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("HYPER special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 25,
                table_index: 11,
                routine: VSoundSpecialRoutine::Hyper,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xC6);
        assert_eq!(dispatch.command_code, 25);
        assert_eq!(dispatch.effective_code, 25);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        let sweep = hyper_special_sweep(step).expect("HYPER should execute phase sweep");
        assert_eq!(sweep.phase_count, 128);
        assert_eq!(sweep.final_phase, 0x80);
        assert_eq!(sweep.dac_samples.len(), 257);
        assert_eq!(sweep.dac_samples[0], 0);
        for pair in sweep.dac_samples[1..].chunks_exact(2) {
            assert_eq!(pair, &[0xFF, 0x00]);
        }
        assert_eq!(board.last_dac_value(), Some(0));
        assert_eq!(board.ram()[0x11], 0x80);
    }

    #[test]
    fn vsnd_irq_command_flow_runs_scream_special_branch() {
        let images = test_rom_images_with_vsnd_tables();
        let mut board = DefenderSoundBoard::with_cleared_ram(&images);
        board.latch_main_board_sound_command(raw_for_vsound_code(26));

        let flow = board
            .step_vsnd_irq_command_flow()
            .expect("SCREAM IRQ command should run");
        let (dispatch, step) =
            irq_command_special_step(flow).expect("SCREAM special command should extract");
        assert_eq!(
            dispatch.routine,
            VSoundIrqRoutine::Special {
                sound_code: 26,
                table_index: 12,
                routine: VSoundSpecialRoutine::Scream,
            }
        );
        assert_eq!(dispatch.latched_port_b, 0xC5);
        assert_eq!(dispatch.command_code, 26);
        assert_eq!(dispatch.effective_code, 26);
        assert_eq!(
            dispatch.background,
            VSoundBackgroundContinuation::WaitingForBackground
        );

        let scream = scream_special(step).expect("SCREAM should execute echo cascade");
        assert_eq!(scream.echo_count, 4);
        assert_eq!(scream.initial_frequency, 0x40);
        assert_eq!(scream.next_echo_frequency, 0x41);
        assert_eq!(scream.spawn_frequency, 0x37);
        assert_eq!(scream.initial_timer, 0);
        assert_eq!(scream.final_timer, 0);
        assert_eq!(scream.decay_passes, 95);
        assert_eq!(scream.echo_starts, 4);
        assert_eq!(scream.final_echo_table, [0; 8]);
        assert_eq!(scream.srmend_byte, 0x41);
        assert_eq!(scream.dac_samples.len(), 24320);
        assert_eq!(count_dac_samples(&scream.dac_samples, 0), 5609);
        assert_eq!(count_dac_samples(&scream.dac_samples, 0x80), 2555);
        assert_eq!(
            scream
                .dac_samples
                .iter()
                .filter(|&&sample| sample != 0)
                .count(),
            18711
        );
        assert_eq!(board.last_dac_value(), Some(0));
        assert_eq!(
            &board.ram()[0x11..0x1C],
            &[0x08, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x41]
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
