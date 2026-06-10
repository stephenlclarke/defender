const SAMPLE_RATE_HZ: u32 = 11_025;
const LOOP_SCALE: usize = 2;
const TURBO_NOISE_DAC_REPEATS: usize = LOOP_SCALE;
const SAMPLE_GAIN: f32 = 0.33;
const VARI_SAMPLE_GAIN: f32 = SAMPLE_GAIN * 0.75;
const BACKGROUND_NOISE_GAIN_SCALE: f32 = 1.25;
const TONAL_HIT_PERIOD_DIVISOR: u16 = 16;
const APPEAR_INITIAL_FREQUENCY: u8 = 0xC0;
const APPEAR_FREQUENCY_DELTA: u8 = 0xFE;
const APPEAR_CYCLE_COUNT: usize = 0x10;
const APPEAR_DELAY_DIVISOR: usize = 12;
const LIGHTNING_MIN_SAMPLES: usize = SAMPLE_RATE_HZ as usize * 5 / 2;
const QUARTER_NOTE_SAMPLES: usize = 2_048;
const WHOLE_NOTE_SAMPLES: usize = QUARTER_NOTE_SAMPLES * 4;

const RADIO_WAVEFORM: [u8; 16] = [
    0x8C, 0x5B, 0xB6, 0x40, 0xBF, 0x49, 0xA4, 0x73, 0x73, 0xA4, 0x49, 0xBF, 0x40, 0xB6, 0x5B, 0x8C,
];

const EIGHT_SAMPLE_SINE_WAVEFORM: &[u8] = &[127, 217, 255, 217, 127, 36, 0, 36];

const ASTRONAUT_CATCH_WAVEFORM: &[u8] = &[127, 196, 225, 196, 127, 57, 30, 57];

const EIGHT_SAMPLE_SQUARE_WAVEFORM: &[u8] = &[0, 64, 128, 0, 255, 0, 128, 64];

const SIXTEEN_SAMPLE_SINE_WAVEFORM: &[u8] = &[
    127, 176, 217, 245, 255, 245, 217, 176, 127, 78, 36, 9, 0, 9, 36, 78,
];

const DISTORTED_HIT_WAVEFORM: &[u8] = &[
    127, 197, 236, 231, 191, 141, 109, 106, 127, 148, 146, 113, 64, 23, 18, 57,
];

const LONG_SQUARE_WAVEFORM: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0,
];

const RAMPED_SINE_WAVEFORM: &[u8] = &[
    138, 149, 160, 171, 181, 191, 200, 209, 218, 225, 232, 238, 243, 247, 251, 253, 254, 255, 254,
    253, 251, 247, 243, 238, 232, 225, 218, 209, 200, 191, 181, 171, 160, 149, 138, 127, 117, 106,
    95, 84, 74, 64, 55, 46, 37, 30, 23, 17, 12, 8, 4, 2, 1, 0, 1, 2, 4, 8, 12, 17, 23, 30, 37, 46,
    55, 64, 74, 84, 95, 106, 117, 127,
];

const TURBINE_SINE_WAVEFORM: &[u8] = &[
    89, 123, 152, 172, 179, 172, 152, 123, 89, 55, 25, 6, 0, 6, 25, 55,
];

const BONUS_PATTERN: &[u8] = &[
    0xA0, 0x98, 0x90, 0x88, 0x80, 0x78, 0x70, 0x68, 0x60, 0x58, 0x50, 0x44, 0x40,
];

const HIT_BELL_DECAY_PATTERN: &[u8] = &[
    1, 1, 2, 2, 4, 4, 8, 8, 0x10, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x60, 0x70, 0x80, 0xA0,
    0xB0, 0xC0,
];

const SPINNER_PATTERN: &[u8] = &[1, 1, 2, 2, 3, 4, 5, 6, 7, 8, 9, 0x0A, 0x0C];

const TURBINE_PATTERN: &[u8] = &[0x80, 0x7C, 0x78, 0x74, 0x70, 0x74, 0x78, 0x7C, 0x80];

const BIG_BELL_PATTERN: &[u8] = &[
    8, 64, 8, 64, 8, 64, 8, 64, 8, 64, 8, 64, 8, 64, 8, 64, 8, 64, 8, 64,
];

const HEARTBEAT_ECHO_PATTERN: &[u8] = &[
    1, 2, 4, 8, 9, 0x0A, 0x0B, 0x0C, 0x0E, 0x0F, 0x10, 0x12, 0x14, 0x40, 0x10,
];

const SPINNER_DRIP_PATTERN: &[u8] = &[0x40, 0x10, 8, 1];

const ASTRONAUT_CATCH_PATTERN: &[u8] = &[0x40];

const COOL_DOWN_PATTERN: &[u8] = &[0x10, 8, 1];

const START_DISTORTO_PATTERN: &[u8] = &[
    1, 1, 1, 1, 2, 2, 3, 3, 4, 4, 5, 6, 8, 0x0A, 0x0C, 0x10, 0x14, 0x18, 0x20, 0x30, 0x40, 0x50,
    0x40, 0x30, 0x20, 0x10, 0x0C, 0x0A, 8, 7, 6, 5, 4, 3, 2, 2, 1, 1, 1,
];

const SHORT_ECHO_DECAY_PATTERN: &[u8] = &[7, 8, 9, 0x0A, 0x0C, 8];

const LONG_ECHO_DECAY_PATTERN: &[u8] = &[0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C];

const ORGAN_NOTE_DELAYS: [u8; 12] = [
    0x47, 0x3F, 0x37, 0x30, 0x29, 0x23, 0x1D, 0x17, 0x12, 0x0D, 0x08, 0x04,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SoundAction {
    Special(SpecialSound),
    GWave(GWaveSound),
    OrganTune(OrganTune),
    OrganNote(OrganNote),
    Vari(VariSound),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpecialSound {
    Spinner,
    BackgroundNoise,
    BackgroundRumbleStep,
    Lightning,
    BonusChime,
    BackgroundEnd,
    Turbo,
    Materialize,
    Thrust,
    Cannon,
    Radio,
    Hyper,
    Scream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GWaveSound {
    Vector(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrganTune {
    Phantom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrganNote {
    Cs2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VariSound {
    Saw,
    Falling,
    Quasar,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RenderedSound {
    pub(crate) sample_rate_hz: u32,
    pub(crate) samples: Vec<f32>,
}
