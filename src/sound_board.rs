//! Source-derived Williams sound-board synthesis helpers.

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

const GWAVE_GS2: &[u8] = &[127, 217, 255, 217, 127, 36, 0, 36];

const GWAVE_GS2_CATCH: &[u8] = &[127, 196, 225, 196, 127, 57, 30, 57];

const GWAVE_GSSQ2: &[u8] = &[0, 64, 128, 0, 255, 0, 128, 64];

const GWAVE_GS1: &[u8] = &[
    127, 176, 217, 245, 255, 245, 217, 176, 127, 78, 36, 9, 0, 9, 36, 78,
];

const GWAVE_GS12: &[u8] = &[
    127, 197, 236, 231, 191, 141, 109, 106, 127, 148, 146, 113, 64, 23, 18, 57,
];

const GWAVE_GSQ22: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0,
];

const GWAVE_GS72: &[u8] = &[
    138, 149, 160, 171, 181, 191, 200, 209, 218, 225, 232, 238, 243, 247, 251, 253, 254, 255, 254,
    253, 251, 247, 243, 238, 232, 225, 218, 209, 200, 191, 181, 171, 160, 149, 138, 127, 117, 106,
    95, 84, 74, 64, 55, 46, 37, 30, 23, 17, 12, 8, 4, 2, 1, 0, 1, 2, 4, 8, 12, 17, 23, 30, 37, 46,
    55, 64, 74, 84, 95, 106, 117, 127,
];

const GWAVE_GS1_7: &[u8] = &[
    89, 123, 152, 172, 179, 172, 152, 123, 89, 55, 25, 6, 0, 6, 25, 55,
];

const BONUS_PATTERN: &[u8] = &[
    0xA0, 0x98, 0x90, 0x88, 0x80, 0x78, 0x70, 0x68, 0x60, 0x58, 0x50, 0x44, 0x40,
];

const HBD_PATTERN: &[u8] = &[
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

const ED10_PATTERN: &[u8] = &[7, 8, 9, 0x0A, 0x0C, 8];

const ED13_PATTERN: &[u8] = &[0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C];

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

#[derive(Debug, Clone)]
pub(crate) struct SoundBoardSynth {
    background_1_active: u8,
    background_2_active: u8,
    spinner_active: u8,
    bonus_active: u8,
    organ_active: i8,
    random_hi: u8,
    random_lo: u8,
}

impl Default for SoundBoardSynth {
    fn default() -> Self {
        Self {
            background_1_active: 0,
            background_2_active: 0,
            spinner_active: 0,
            bonus_active: 0,
            organ_active: 0,
            random_hi: 0x3C,
            random_lo: 0,
        }
    }
}

impl SoundBoardSynth {
    pub(crate) fn render_actions(&mut self, actions: &[SoundAction]) -> RenderedSound {
        let mut samples = Vec::new();
        for action in actions {
            match action {
                SoundAction::Special(sound) => self.render_special(*sound, &mut samples),
                SoundAction::GWave(sound) => self.render_gwave(*sound, &mut samples),
                SoundAction::OrganTune(tune) => self.render_organ_tune(*tune, &mut samples),
                SoundAction::OrganNote(note) => self.render_organ_note(*note, &mut samples),
                SoundAction::Vari(sound) => self.render_vari(*sound, &mut samples),
            }
        }

        RenderedSound {
            sample_rate_hz: SAMPLE_RATE_HZ,
            samples,
        }
    }

    fn render_special(&mut self, sound: SpecialSound, out: &mut Vec<f32>) {
        match sound {
            SpecialSound::Spinner => self.render_vari(VariSound::Quasar, out),
            SpecialSound::BackgroundNoise => {
                self.background_1_active = 1;
                let start = out.len();
                self.render_filtered_noise(false, false, 3, 600, out);
                for sample in &mut out[start..] {
                    *sample = (*sample * BACKGROUND_NOISE_GAIN_SCALE).clamp(-0.85, 0.85);
                }
            }
            SpecialSound::BackgroundRumbleStep => {
                self.background_2_active = self.background_2_active.wrapping_add(1).max(1);
                self.render_gwave_vector(turbine_vector(), out);
            }
            SpecialSound::Lightning => self.render_lightning_noise(1, 1, 3, 8, out),
            SpecialSound::BonusChime => {
                self.bonus_active = 1;
                self.render_gwave_vector(bonus_vector(), out);
            }
            SpecialSound::BackgroundEnd => {
                self.background_1_active = 0;
                self.background_2_active = 0;
            }
            SpecialSound::Turbo => self.render_white_noise(1, 0x20, 1, 0xFF, true, out),
            SpecialSound::Materialize => self.render_materialize_noise(out),
            SpecialSound::Thrust => {
                self.background_1_active = 1;
                self.render_filtered_noise(false, false, 4, 900, out);
            }
            SpecialSound::Cannon => self.render_filtered_noise(true, true, 0xFF, 16_000, out),
            SpecialSound::Radio => self.render_radio(out),
            SpecialSound::Hyper => self.render_hyper(out),
            SpecialSound::Scream => self.render_scream(out),
        }
    }

    fn render_gwave(&mut self, sound: GWaveSound, out: &mut Vec<f32>) {
        let Some(vector) = gwave_vector(sound) else {
            return;
        };
        self.render_gwave_vector(vector, out);
    }

    fn render_gwave_vector(&mut self, vector: GWaveVector, out: &mut Vec<f32>) {
        let source_waveform = vector.waveform;
        let mut waveform = source_waveform.to_vec();
        source_decay_waveform(&mut waveform, source_waveform, vector.pre_decay);
        let mut frequency_offset: i16 = 0;
        let mut delta_counter = vector.delta_frequency_count;
        let mut pattern_start = 0;
        let mut pattern_end = vector.pattern.len();
        loop {
            let mut echo_count = vector.echo_count.max(1);
            loop {
                for period in &vector.pattern[pattern_start..pattern_end] {
                    let adjusted = i16::from(*period) + frequency_offset;
                    if !(1..=i16::from(u8::MAX)).contains(&adjusted) {
                        continue;
                    }

                    for _ in 0..vector.cycle_count.max(1) {
                        self.append_wave(out, &waveform, adjusted as u16, vector.period_divisor);
                    }
                }

                if echo_count <= 1 {
                    break;
                }
                echo_count -= 1;
                source_decay_waveform(&mut waveform, source_waveform, vector.echo_decay);
            }

            if vector.delta_frequency_increment == 0 {
                break;
            }
            delta_counter = delta_counter.wrapping_sub(1);
            if delta_counter == 0 {
                break;
            }
            frequency_offset += i16::from(vector.delta_frequency_increment);
            let Some((next_start, next_end)) = source_adjusted_pattern_range(
                &vector.pattern[pattern_start..pattern_end],
                frequency_offset,
            ) else {
                break;
            };
            pattern_start += next_start;
            pattern_end = pattern_start + (next_end - next_start);
            if vector.echo_decay != 0 {
                waveform.clear();
                waveform.extend_from_slice(source_waveform);
                source_decay_waveform(&mut waveform, source_waveform, vector.pre_decay);
            }
        }
    }

    fn render_organ_tune(&mut self, tune: OrganTune, out: &mut Vec<f32>) {
        match tune {
            OrganTune::Phantom => {
                self.organ_active = 1;
                let steps = [
                    OrganStep {
                        oscillator_mask: 0x7F,
                        delay: 0x1D,
                        samples: QUARTER_NOTE_SAMPLES,
                    },
                    OrganStep {
                        oscillator_mask: 0x7F,
                        delay: 0x23,
                        samples: QUARTER_NOTE_SAMPLES,
                    },
                    OrganStep {
                        oscillator_mask: 0xFE,
                        delay: 0x08,
                        samples: WHOLE_NOTE_SAMPLES,
                    },
                ];
                for step in steps {
                    self.append_organ_step(out, step);
                }
            }
        }
    }

    fn render_vari(&mut self, sound: VariSound, out: &mut Vec<f32>) {
        let vector = match sound {
            VariSound::Saw => VariVector {
                low_period: 0x40,
                high_period: 0x01,
                low_delta: 0,
                high_delta: 0x10,
                high_end: 0xE1,
                sweep_period: 0x0080,
                low_mod: 0xFF,
                amplitude: 0xFF,
            },
            VariSound::Falling => VariVector {
                low_period: 0x28,
                high_period: 0x01,
                low_delta: 0,
                high_delta: 0x08,
                high_end: 0x81,
                sweep_period: 0x0200,
                low_mod: 0xFF,
                amplitude: 0xFF,
            },
            VariSound::Quasar => VariVector {
                low_period: 0x28,
                high_period: 0x81,
                low_delta: 0,
                high_delta: 0xFC,
                high_end: 0x01,
                sweep_period: 0x0200,
                low_mod: 0xFC,
                amplitude: 0xFF,
            },
        };
        self.render_vari_vector(vector, out);
    }

    fn render_organ_note(&mut self, note: OrganNote, out: &mut Vec<f32>) {
        let note_index = match note {
            OrganNote::Cs2 => 5,
        };
        self.append_organ_step(
            out,
            OrganStep {
                oscillator_mask: 0x7C,
                delay: ORGAN_NOTE_DELAYS[note_index],
                samples: QUARTER_NOTE_SAMPLES / 2,
            },
        );
    }

    fn render_filtered_noise(
        &mut self,
        decay_frequency: bool,
        distortion: bool,
        initial_max_frequency: u8,
        sample_count: usize,
        out: &mut Vec<f32>,
    ) {
        let mut max_frequency = initial_max_frequency.max(1);
        let mut current = 0u8;
        let mut samples_left = sample_count.max(1);
        while samples_left > 0 {
            let target = self.next_random_byte();
            let mut step = if distortion {
                (max_frequency & self.random_hi).max(1)
            } else {
                max_frequency.max(1)
            };
            step = step.max(1);
            current = move_toward(current, target, step);
            append_level(out, current, LOOP_SCALE);
            samples_left -= 1;

            if decay_frequency && samples_left.is_multiple_of(64) {
                let decay = (max_frequency / 8).max(1);
                max_frequency = max_frequency.saturating_sub(decay).max(7);
                if max_frequency == 7 && samples_left < 64 {
                    break;
                }
            }
        }
    }

    fn render_white_noise(
        &mut self,
        decay_rate: u8,
        cycle_count: usize,
        initial_period: u16,
        initial_amplitude: u8,
        decay_frequency: bool,
        out: &mut Vec<f32>,
    ) {
        let mut amplitude = initial_amplitude;
        let mut period = initial_period.max(1);
        while amplitude > decay_rate {
            for _ in 0..cycle_count.max(1) {
                let level = if self.next_random_bit() { amplitude } else { 0 };
                let repeats = if decay_frequency {
                    TURBO_NOISE_DAC_REPEATS
                } else {
                    usize::from(period.max(1))
                };
                append_level(out, level, repeats);
            }
            amplitude = amplitude.saturating_sub(decay_rate.max(1));
            if decay_frequency {
                period = period.saturating_add(1);
            }
            if out.len() > SAMPLE_RATE_HZ as usize {
                break;
            }
        }
    }

    fn render_lightning_noise(
        &mut self,
        frequency_delta: u8,
        initial_frequency: u8,
        cycle_count: usize,
        repeat_divisor: u8,
        out: &mut Vec<f32>,
    ) {
        let repeat_divisor = usize::from(repeat_divisor.max(1));
        let mut level = 0xFF;
        while out.len() < LIGHTNING_MIN_SAMPLES {
            let mut frequency = initial_frequency.max(1);
            loop {
                for _ in 0..cycle_count.max(1) {
                    if self.next_random_bit() {
                        level = !level;
                    }
                    append_level(
                        out,
                        level,
                        usize::from(frequency.max(1)) / repeat_divisor + 1,
                    );
                }
                frequency = frequency.wrapping_add(frequency_delta);
                if frequency == 0 {
                    break;
                }
            }
        }
    }

    fn render_materialize_noise(&mut self, out: &mut Vec<f32>) {
        let mut level = 0xFF;
        let mut frequency = APPEAR_INITIAL_FREQUENCY;
        append_level(out, level, source_appear_repeat_count(frequency));

        loop {
            for _ in 0..APPEAR_CYCLE_COUNT {
                if self.next_source_random_carry_bit() {
                    level = !level;
                }
                append_level(out, level, source_appear_repeat_count(frequency));
            }

            frequency = frequency.wrapping_add(APPEAR_FREQUENCY_DELTA);
            if frequency == 0 {
                break;
            }
        }
    }

    fn render_vari_vector(&mut self, vector: VariVector, out: &mut Vec<f32>) {
        let max_samples = SAMPLE_RATE_HZ as usize * 2;
        let mut low_period_seed = vector.low_period;
        let mut level = vector.amplitude;
        append_vari_level(out, level, LOOP_SCALE);

        while out.len() < max_samples {
            let mut low_count = low_period_seed;
            let mut high_count = vector.high_period;
            loop {
                let mut sweep_counter = vector.sweep_period.max(1);
                while sweep_counter > 0 && out.len() < max_samples {
                    level = !level;
                    let low_ticks = u16::from(low_count.max(1)).min(sweep_counter);
                    append_vari_level(out, level, source_vari_repeat_count(low_ticks));
                    sweep_counter = sweep_counter.saturating_sub(low_ticks);
                    if sweep_counter == 0 {
                        break;
                    }

                    level = !level;
                    let high_ticks = u16::from(high_count.max(1)).min(sweep_counter);
                    append_vari_level(out, level, source_vari_repeat_count(high_ticks));
                    sweep_counter = sweep_counter.saturating_sub(high_ticks);
                }

                level = if level & 0x80 == 0 { !level } else { level };
                append_vari_level(out, level, LOOP_SCALE);

                low_count = low_count.wrapping_add(vector.low_delta);
                high_count = high_count.wrapping_add(vector.high_delta);
                if high_count != vector.high_end && out.len() < max_samples {
                    continue;
                }

                if vector.low_mod == 0 {
                    return;
                }
                low_period_seed = low_period_seed.wrapping_add(vector.low_mod);
                if low_period_seed == 0 {
                    return;
                }
                break;
            }
        }
    }

    fn render_radio(&mut self, out: &mut Vec<f32>) {
        self.spinner_active = 1;
        let mut phase = 0u16;
        let mut frequency = 0x0100u16;
        loop {
            let (next_phase, carry) = phase.overflowing_add(frequency);
            phase = next_phase;
            if carry {
                frequency = frequency.wrapping_add(1);
                if frequency == 0 {
                    break;
                }
            }
            let index = usize::from(((phase >> 8) & 0x0F) as u8);
            append_level(out, RADIO_WAVEFORM[index], LOOP_SCALE);
            if out.len() > SAMPLE_RATE_HZ as usize {
                break;
            }
        }
    }

    fn render_hyper(&mut self, out: &mut Vec<f32>) {
        let mut sound = 0u8;
        for phase in 0u8..=0x7F {
            for counter in 0u8..=0x7F {
                if counter == phase {
                    sound = !sound;
                }
                append_level(out, sound, 18);
            }
            sound = !sound;
        }
    }

    fn render_scream(&mut self, out: &mut Vec<f32>) {
        self.background_2_active = 1;
        let mut table = [(0u8, 0u8); 4];
        table[0].0 = 0x40;
        let mut timer = 0u8;
        loop {
            let mut amplitude = 0x80u8;
            let mut output = 0u8;
            for entry in &mut table {
                let (frequency, counter) = entry;
                *counter = counter.wrapping_add(*frequency);
                if (*counter as i8).is_negative() {
                    output = output.wrapping_add(amplitude);
                }
                amplitude >>= 1;
            }
            append_level(out, output, LOOP_SCALE);
            timer = timer.wrapping_add(1);
            if timer == 0 {
                let mut any_active = false;
                for index in 0..table.len() {
                    let mut frequency = table[index].0;
                    if frequency == 0 {
                        continue;
                    }
                    if frequency == 0x37 && index + 1 < table.len() {
                        table[index + 1].0 = 0x41;
                    }
                    frequency = frequency.wrapping_sub(1);
                    table[index].0 = frequency;
                    any_active = any_active || frequency != 0;
                }
                if !any_active {
                    break;
                }
            }
        }
    }

    fn append_wave(
        &mut self,
        out: &mut Vec<f32>,
        waveform: &[u8],
        period: u16,
        period_divisor: u16,
    ) {
        let repeats = (usize::from(period) / usize::from(period_divisor.max(1))).max(1);
        for sample in waveform {
            append_level(out, *sample, repeats);
        }
    }

    fn append_organ_step(&mut self, out: &mut Vec<f32>, step: OrganStep) {
        let period = usize::from(step.delay.max(1));
        let mut counter = 0u8;
        for _ in 0..step.samples {
            counter = counter.wrapping_add(1);
            let masked = counter & step.oscillator_mask;
            let harmonic_count = masked.count_ones() as f32 / 8.0;
            let level = (harmonic_count * 255.0).round().clamp(0.0, 255.0) as u8;
            append_level(out, level, period / 3 + 1);
        }
    }

    fn next_random_bit(&mut self) -> bool {
        let mut value = self.random_lo;
        value >>= 1;
        value >>= 1;
        value >>= 1;
        value ^= self.random_lo;
        let carry = value & 1;
        let high_carry = self.random_hi & 1;
        self.random_hi = (self.random_hi >> 1) | (carry << 7);
        self.random_lo = (self.random_lo >> 1) | (high_carry << 7);
        (self.random_lo & 1) != 0
    }

    fn next_source_random_carry_bit(&mut self) -> bool {
        let feedback = ((self.random_lo >> 3) ^ self.random_lo) & 1;
        let high_carry = self.random_hi & 1;
        let low_carry = self.random_lo & 1;
        self.random_hi = (self.random_hi >> 1) | (feedback << 7);
        self.random_lo = (self.random_lo >> 1) | (high_carry << 7);
        low_carry != 0
    }

    fn next_random_byte(&mut self) -> u8 {
        for _ in 0..8 {
            self.next_random_bit();
        }
        self.random_lo
    }
}

#[derive(Debug, Clone, Copy)]
struct GWaveVector {
    echo_count: u8,
    cycle_count: u8,
    echo_decay: u8,
    waveform: &'static [u8],
    pre_decay: u8,
    delta_frequency_increment: i8,
    delta_frequency_count: u8,
    pattern: &'static [u8],
    period_divisor: u16,
}

#[derive(Debug, Clone, Copy)]
struct VariVector {
    low_period: u8,
    high_period: u8,
    low_delta: u8,
    high_delta: u8,
    high_end: u8,
    sweep_period: u16,
    low_mod: u8,
    amplitude: u8,
}

#[derive(Debug, Clone, Copy)]
struct OrganStep {
    oscillator_mask: u8,
    delay: u8,
    samples: usize,
}

pub(crate) fn sound_actions_for_command(command: u8) -> Vec<SoundAction> {
    let sound_number = (!command) & 0x1F;
    let action = match sound_number {
        0 => return Vec::new(),
        1..=13 => SoundAction::GWave(GWaveSound::Vector(sound_number)),
        14 => SoundAction::Special(SpecialSound::Spinner),
        15 => SoundAction::Special(SpecialSound::BackgroundNoise),
        16 => SoundAction::Special(SpecialSound::BackgroundRumbleStep),
        17 => SoundAction::Special(SpecialSound::Lightning),
        18 => SoundAction::Special(SpecialSound::BonusChime),
        19 => SoundAction::Special(SpecialSound::BackgroundEnd),
        20 => SoundAction::Special(SpecialSound::Turbo),
        21 => SoundAction::Special(SpecialSound::Materialize),
        22 => SoundAction::Special(SpecialSound::Thrust),
        23 => SoundAction::Special(SpecialSound::Cannon),
        24 => SoundAction::Special(SpecialSound::Radio),
        25 => SoundAction::Special(SpecialSound::Hyper),
        26 => SoundAction::Special(SpecialSound::Scream),
        27 => SoundAction::OrganTune(OrganTune::Phantom),
        28 => SoundAction::OrganNote(OrganNote::Cs2),
        29 => SoundAction::Vari(VariSound::Saw),
        30 => SoundAction::Vari(VariSound::Falling),
        31 => SoundAction::Vari(VariSound::Quasar),
        _ => unreachable!("five-bit sound command is in range"),
    };
    vec![action]
}

fn gwave_vector(sound: GWaveSound) -> Option<GWaveVector> {
    let GWaveSound::Vector(number) = sound;
    match number {
        1 => Some(GWaveVector {
            echo_count: 8,
            cycle_count: 1,
            echo_decay: 2,
            waveform: GWAVE_GSQ22,
            pre_decay: 0,
            delta_frequency_increment: 0,
            delta_frequency_count: 0,
            pattern: HBD_PATTERN,
            period_divisor: TONAL_HIT_PERIOD_DIVISOR,
        }),
        2 => Some(GWaveVector {
            echo_count: 1,
            cycle_count: 2,
            echo_decay: 0,
            waveform: GWAVE_GS72,
            pre_decay: 0x1A,
            delta_frequency_increment: -1,
            delta_frequency_count: 0,
            pattern: START_DISTORTO_PATTERN,
            period_divisor: 8,
        }),
        3 => Some(GWaveVector {
            echo_count: 1,
            cycle_count: 1,
            echo_decay: 0,
            waveform: GWAVE_GS72,
            pre_decay: 0x11,
            delta_frequency_increment: 1,
            delta_frequency_count: 15,
            pattern: &BIG_BELL_PATTERN[..1],
            period_divisor: 8,
        }),
        4 => Some(GWaveVector {
            echo_count: 1,
            cycle_count: 1,
            echo_decay: 3,
            waveform: GWAVE_GSSQ2,
            pre_decay: 0,
            delta_frequency_increment: 1,
            delta_frequency_count: 0,
            pattern: SPINNER_PATTERN,
            period_divisor: 8,
        }),
        5 => Some(GWaveVector {
            echo_count: 15,
            cycle_count: 4,
            echo_decay: 1,
            waveform: GWAVE_GS1,
            pre_decay: 0,
            delta_frequency_increment: 0,
            delta_frequency_count: 0,
            pattern: BIG_BELL_PATTERN,
            period_divisor: TONAL_HIT_PERIOD_DIVISOR,
        }),
        6 => Some(GWaveVector {
            echo_count: 4,
            cycle_count: 1,
            echo_decay: 4,
            waveform: GWAVE_GS72,
            pre_decay: 0,
            delta_frequency_increment: 0,
            delta_frequency_count: 0,
            pattern: HEARTBEAT_ECHO_PATTERN,
            period_divisor: 8,
        }),
        7 => Some(GWaveVector {
            echo_count: 2,
            cycle_count: 1,
            echo_decay: 3,
            waveform: GWAVE_GS72,
            pre_decay: 0x11,
            delta_frequency_increment: -1,
            delta_frequency_count: 0,
            pattern: SPINNER_PATTERN,
            period_divisor: 8,
        }),
        8 => Some(GWaveVector {
            echo_count: 1,
            cycle_count: 5,
            echo_decay: 0,
            waveform: GWAVE_GS2_CATCH,
            pre_decay: 0,
            delta_frequency_increment: -3,
            delta_frequency_count: 0,
            pattern: ASTRONAUT_CATCH_PATTERN,
            period_divisor: 12,
        }),
        9 => Some(GWaveVector {
            echo_count: 3,
            cycle_count: 1,
            echo_decay: 1,
            waveform: GWAVE_GSSQ2,
            pre_decay: 0,
            delta_frequency_increment: 1,
            delta_frequency_count: 0,
            pattern: COOL_DOWN_PATTERN,
            period_divisor: 8,
        }),
        10 => Some(GWaveVector {
            echo_count: 1,
            cycle_count: 1,
            echo_decay: 1,
            waveform: GWAVE_GS72,
            pre_decay: 1,
            delta_frequency_increment: 1,
            delta_frequency_count: 1,
            pattern: &BIG_BELL_PATTERN[..1],
            period_divisor: 8,
        }),
        11 => Some(GWaveVector {
            echo_count: 15,
            cycle_count: 6,
            echo_decay: 5,
            waveform: GWAVE_GS12,
            pre_decay: 3,
            delta_frequency_increment: 0,
            delta_frequency_count: 2,
            pattern: ED10_PATTERN,
            period_divisor: 8,
        }),
        12 => Some(GWaveVector {
            echo_count: 6,
            cycle_count: 10,
            echo_decay: 1,
            waveform: GWAVE_GS2,
            pre_decay: 2,
            delta_frequency_increment: 0,
            delta_frequency_count: 2,
            pattern: ED13_PATTERN,
            period_divisor: 8,
        }),
        13 => Some(GWaveVector {
            echo_count: 1,
            cycle_count: 15,
            echo_decay: 1,
            waveform: GWAVE_GS1,
            pre_decay: 0,
            delta_frequency_increment: -1,
            delta_frequency_count: 0x10,
            pattern: SPINNER_DRIP_PATTERN,
            period_divisor: 8,
        }),
        _ => None,
    }
}

fn bonus_vector() -> GWaveVector {
    GWaveVector {
        echo_count: 3,
        cycle_count: 1,
        echo_decay: 1,
        waveform: GWAVE_GSSQ2,
        pre_decay: 0,
        delta_frequency_increment: -1,
        delta_frequency_count: 0,
        pattern: BONUS_PATTERN,
        period_divisor: 8,
    }
}

fn turbine_vector() -> GWaveVector {
    GWaveVector {
        echo_count: 1,
        cycle_count: 2,
        echo_decay: 0,
        waveform: GWAVE_GS1_7,
        pre_decay: 0,
        delta_frequency_increment: -1,
        delta_frequency_count: 1,
        pattern: TURBINE_PATTERN,
        period_divisor: 8,
    }
}

fn append_level(out: &mut Vec<f32>, level: u8, repeats: usize) {
    let sample = u8_to_sample(level);
    out.extend(std::iter::repeat_n(sample, repeats.max(1)));
}

fn append_vari_level(out: &mut Vec<f32>, level: u8, repeats: usize) {
    let sample = u8_to_sample_with_gain(level, VARI_SAMPLE_GAIN);
    out.extend(std::iter::repeat_n(sample, repeats.max(1)));
}

fn u8_to_sample(level: u8) -> f32 {
    u8_to_sample_with_gain(level, SAMPLE_GAIN)
}

fn u8_to_sample_with_gain(level: u8, gain: f32) -> f32 {
    (((level as f32 / 255.0) * 2.0) - 1.0) * gain
}

fn move_toward(current: u8, target: u8, step: u8) -> u8 {
    if current < target {
        current.saturating_add(step).min(target)
    } else if current > target {
        current.saturating_sub(step).max(target)
    } else {
        current
    }
}

fn source_decay_waveform(waveform: &mut [u8], source_waveform: &[u8], decay_factor: u8) {
    if decay_factor == 0 {
        return;
    }
    for (sample, source_sample) in waveform.iter_mut().zip(source_waveform) {
        let decay_step = source_sample >> 4;
        for _ in 0..decay_factor {
            *sample = sample.wrapping_sub(decay_step);
        }
    }
}

fn source_adjusted_pattern_range(pattern: &[u8], frequency_offset: i16) -> Option<(usize, usize)> {
    let mut start = None;
    for (index, period) in pattern.iter().enumerate() {
        let adjusted = i16::from(*period) + frequency_offset;
        let valid = (1..=i16::from(u8::MAX)).contains(&adjusted);
        match (start, valid) {
            (None, true) => start = Some(index),
            (Some(start), false) => return Some((start, index)),
            _ => {}
        }
    }
    start.map(|start| (start, pattern.len()))
}

fn source_vari_repeat_count(ticks: u16) -> usize {
    (usize::from(ticks) / 8).max(1)
}

fn source_appear_repeat_count(frequency: u8) -> usize {
    (usize::from(frequency.max(1)) / APPEAR_DELAY_DIVISOR).max(1)
}

#[cfg(test)]
mod tests {
    use super::{
        GWaveSound, LIGHTNING_MIN_SAMPLES, OrganNote, OrganTune, SAMPLE_RATE_HZ, SoundAction,
        SoundBoardSynth, SpecialSound, VariSound, gwave_vector, sound_actions_for_command,
        source_adjusted_pattern_range, source_decay_waveform,
    };

    #[test]
    fn special_and_gwave_sounds_generate_samples() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[
            SoundAction::Special(SpecialSound::Radio),
            SoundAction::Special(SpecialSound::Hyper),
            SoundAction::Special(SpecialSound::Cannon),
            SoundAction::Special(SpecialSound::Turbo),
            SoundAction::Special(SpecialSound::Lightning),
            SoundAction::Special(SpecialSound::Materialize),
            SoundAction::GWave(GWaveSound::Vector(10)),
            SoundAction::Vari(VariSound::Falling),
        ]);

        assert_eq!(rendered.sample_rate_hz, SAMPLE_RATE_HZ);
        assert!(!rendered.samples.is_empty());
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn quasar_vari_uses_source_restart_tail() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Vari(VariSound::Quasar)]);

        assert!(rendered.samples.len() > SAMPLE_RATE_HZ as usize / 2);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.1));
    }

    #[test]
    fn source_waveform_decay_uses_wrapping_rom_sample_subtraction() {
        let source_waveform = [0, 16, 127, 255];
        let mut waveform = source_waveform;

        source_decay_waveform(&mut waveform, &source_waveform, 17);

        assert_eq!(waveform, [0, 255, 8, 0]);
    }

    #[test]
    fn source_frequency_delta_keeps_contiguous_valid_period_range() {
        assert_eq!(
            source_adjusted_pattern_range(&[1, 2, 3, 4], -1),
            Some((1, 4))
        );
        assert_eq!(
            source_adjusted_pattern_range(&[252, 253, 254, 255], 2),
            Some((0, 2))
        );
        assert_eq!(source_adjusted_pattern_range(&[1, 2], -2), None);
    }

    #[test]
    fn tonal_hit_gwave_vectors_use_mame_calibrated_period_density() {
        let bomber_hit = gwave_vector(GWaveSound::Vector(1)).expect("bomber hit GWAVE vector");
        let pod_hit = gwave_vector(GWaveSound::Vector(5)).expect("pod hit GWAVE vector");

        assert_eq!(bomber_hit.period_divisor, 16);
        assert_eq!(pod_hit.period_divisor, 16);
    }

    #[test]
    fn cannon_generates_player_death_length_noise_tail() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Special(SpecialSound::Cannon)]);

        assert!(rendered.samples.len() > SAMPLE_RATE_HZ as usize);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.1));
    }

    #[test]
    fn lightning_generates_human_loss_length_tail() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Special(SpecialSound::Lightning)]);

        assert!(rendered.samples.len() >= LIGHTNING_MIN_SAMPLES);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.1));
    }

    #[test]
    fn materialize_uses_source_appear_sweep_cadence() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Special(SpecialSound::Materialize)]);

        assert!(
            (SAMPLE_RATE_HZ as usize..SAMPLE_RATE_HZ as usize * 6 / 5)
                .contains(&rendered.samples.len())
        );
        let sign_transitions = rendered
            .samples
            .windows(2)
            .filter(|pair| pair[0].is_sign_positive() != pair[1].is_sign_positive())
            .count();
        assert!((600..900).contains(&sign_transitions));
    }

    #[test]
    fn organ_sounds_generate_non_silent_samples() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[
            SoundAction::OrganTune(OrganTune::Phantom),
            SoundAction::OrganNote(OrganNote::Cs2),
        ]);

        assert!(rendered.samples.len() > SAMPLE_RATE_HZ as usize / 4);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn scream_generates_a_decaying_tail() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Special(SpecialSound::Scream)]);
        let midpoint = rendered.samples.len() / 2;
        let first_half_peak = rendered.samples[..midpoint]
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0, f32::max);
        let second_half_peak = rendered.samples[midpoint..]
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0, f32::max);

        assert!(first_half_peak >= second_half_peak);
    }

    #[test]
    fn hyper_generates_a_toggling_square_wave() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Special(SpecialSound::Hyper)]);

        assert!(rendered.samples.len() > SAMPLE_RATE_HZ as usize / 2);
        let transitions = rendered
            .samples
            .windows(2)
            .filter(|pair| (pair[0] - pair[1]).abs() > 0.5)
            .count();
        assert!(transitions > 32);
    }

    #[test]
    fn source_commands_decode_to_sound_board_actions() {
        assert_eq!(
            sound_actions_for_command(0xF5),
            vec![SoundAction::GWave(GWaveSound::Vector(10))]
        );
        assert_eq!(
            sound_actions_for_command(0xEB),
            vec![SoundAction::Special(SpecialSound::Turbo)]
        );
        assert_eq!(
            sound_actions_for_command(0xE8),
            vec![SoundAction::Special(SpecialSound::Cannon)]
        );
        assert_eq!(
            sound_actions_for_command(0xEE),
            vec![SoundAction::Special(SpecialSound::Lightning)]
        );
        assert_eq!(
            sound_actions_for_command(0xE5),
            vec![SoundAction::Special(SpecialSound::Scream)]
        );
        assert!(sound_actions_for_command(0xFF).is_empty());
    }
}
