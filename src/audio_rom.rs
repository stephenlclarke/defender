//! ROM-derived sound synthesis helpers translated from `VSNDRM1.SRC`.

const SAMPLE_RATE: u32 = 11_025;
const LOOP_SCALE: usize = 2;
const SAMPLE_GAIN: f32 = 0.55;
const QUARTER_NOTE_SAMPLES: usize = 2_048;
const WHOLE_NOTE_SAMPLES: usize = QUARTER_NOTE_SAMPLES * 4;

const RADSND: [u8; 16] = [
    0x8C, 0x5B, 0xB6, 0x40, 0xBF, 0x49, 0xA4, 0x73, 0x73, 0xA4, 0x49, 0xBF, 0x40, 0xB6, 0x5B, 0x8C,
];

const GWS_GS1: &[u8] = &[
    127, 176, 217, 245, 255, 245, 217, 176, 127, 78, 36, 9, 0, 9, 36, 78,
];

const BONSND: &[u8] = &[
    0xA0, 0x98, 0x90, 0x88, 0x80, 0x78, 0x70, 0x68, 0x60, 0x58, 0x50, 0x44, 0x40,
];

const NOTTAB: [u8; 12] = [
    0x47, 0x3F, 0x37, 0x30, 0x29, 0x23, 0x1D, 0x17, 0x12, 0x0D, 0x08, 0x04,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RomAction {
    Special(SpecialRoutine),
    GWave(GWaveSound),
    OrganTune(OrganTune),
    OrganNote(OrganNote),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpecialRoutine {
    Thrust,
    Cannon,
    Radio,
    Scream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GWaveSound {
    Bonus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrganTune {
    Phantom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrganNote {
    Cs2,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RenderedSound {
    pub(crate) sample_rate: u32,
    pub(crate) samples: Vec<f32>,
}

#[derive(Debug, Clone)]
pub(crate) struct RomSoundBoard {
    _bg1flg: u8,
    _bg2flg: u8,
    _sp1flg: u8,
    _b2flg: u8,
    _orgflg: i8,
    hi: u8,
    lo: u8,
}

impl Default for RomSoundBoard {
    fn default() -> Self {
        Self {
            _bg1flg: 0,
            _bg2flg: 0,
            _sp1flg: 0,
            _b2flg: 0,
            _orgflg: 0,
            hi: 0x3C,
            lo: 0,
        }
    }
}

impl RomSoundBoard {
    pub(crate) fn render_actions(&mut self, actions: &[RomAction]) -> RenderedSound {
        let mut samples = Vec::new();
        for action in actions {
            match action {
                RomAction::Special(routine) => self.render_special(*routine, &mut samples),
                RomAction::GWave(sound) => self.render_gwave(*sound, &mut samples),
                RomAction::OrganTune(tune) => self.render_organ_tune(*tune, &mut samples),
                RomAction::OrganNote(note) => self.render_organ_note(*note, &mut samples),
            }
        }

        RenderedSound {
            sample_rate: SAMPLE_RATE,
            samples,
        }
    }

    fn render_special(&mut self, routine: SpecialRoutine, out: &mut Vec<f32>) {
        match routine {
            SpecialRoutine::Thrust => {
                self._bg1flg = 1;
                self.render_filtered_noise(false, false, 3, 900, out);
            }
            SpecialRoutine::Cannon => self.render_filtered_noise(true, true, 0xFF, 1_000, out),
            SpecialRoutine::Radio => self.render_radio(out),
            SpecialRoutine::Scream => self.render_scream(out),
        }
    }

    fn render_gwave(&mut self, sound: GWaveSound, out: &mut Vec<f32>) {
        let vector = match sound {
            GWaveSound::Bonus => GWaveVector {
                gecho: 3,
                gccnt: 1,
                gecdec: 1,
                waveform: GWS_GS1,
                pre_decay: 0,
                delta_freq_inc: -1,
                delta_freq_count: 0,
                pattern: BONSND,
            },
        };

        let mut waveform = vector.waveform.to_vec();
        if vector.pre_decay != 0 {
            decay_waveform(&mut waveform, vector.pre_decay);
        }

        let mut echo_count = vector.gecho.max(1);
        let mut frequency_offset: i16 = 0;
        let mut delta_counter = vector.delta_freq_count;
        loop {
            let mut pattern_end = vector.pattern.len();
            for (index, period) in vector.pattern.iter().enumerate() {
                let adjusted = i16::from(*period) + frequency_offset;
                if adjusted <= 0 {
                    if pattern_end == vector.pattern.len() {
                        pattern_end = index;
                    }
                    continue;
                }

                for _ in 0..vector.gccnt.max(1) {
                    self.append_wave(out, &waveform, adjusted as u16);
                }

                if delta_counter > 0 {
                    delta_counter -= 1;
                    if delta_counter > 0 {
                        frequency_offset += i16::from(vector.delta_freq_inc);
                    }
                }
            }

            if echo_count <= 1 {
                break;
            }
            echo_count -= 1;
            decay_waveform(&mut waveform, vector.gecdec);
        }
    }

    fn render_organ_tune(&mut self, tune: OrganTune, out: &mut Vec<f32>) {
        match tune {
            OrganTune::Phantom => {
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

    fn render_organ_note(&mut self, note: OrganNote, out: &mut Vec<f32>) {
        let note_index = match note {
            OrganNote::Cs2 => 5,
        };
        let delay = NOTTAB[note_index];
        self.append_organ_step(
            out,
            OrganStep {
                oscillator_mask: 0x7C,
                delay,
                samples: QUARTER_NOTE_SAMPLES / 2,
            },
        );
    }

    fn render_filtered_noise(
        &mut self,
        decay_frequency: bool,
        distortion: bool,
        initial_max_freq: u8,
        sample_count: usize,
        out: &mut Vec<f32>,
    ) {
        let mut fmax = initial_max_freq.max(1);
        let mut current = 0u8;
        let mut samples_left = sample_count.max(1);
        while samples_left > 0 {
            let target = self.next_random_byte();
            let mut step = if distortion {
                (fmax & self.hi).max(1)
            } else {
                fmax.max(1)
            };
            step = step.max(1);
            current = move_toward(current, target, step);
            append_level(out, current, LOOP_SCALE);
            samples_left -= 1;

            if decay_frequency && samples_left.is_multiple_of(64) {
                let decay = (fmax / 8).max(1);
                fmax = fmax.saturating_sub(decay).max(7);
                if fmax == 7 && samples_left < 64 {
                    break;
                }
            }
        }
    }

    fn render_radio(&mut self, out: &mut Vec<f32>) {
        let mut phase = 0u16;
        let mut freq = 0x0100u16;
        loop {
            let (next_phase, carry) = phase.overflowing_add(freq);
            phase = next_phase;
            if carry {
                freq = freq.wrapping_add(1);
                if freq == 0 {
                    break;
                }
            }
            let index = usize::from(((phase >> 8) & 0x0F) as u8);
            append_level(out, RADSND[index], LOOP_SCALE);
            if out.len() > SAMPLE_RATE as usize {
                break;
            }
        }
    }

    fn render_scream(&mut self, out: &mut Vec<f32>) {
        let mut table = [(0u8, 0u8); 4];
        table[0].0 = 0x40;
        let mut timer = 0u8;
        loop {
            let mut amplitude = 0x80u8;
            let mut output = 0u8;
            for entry in &mut table {
                let (freq, counter) = entry;
                *counter = counter.wrapping_add(*freq);
                if (*counter as i8).is_negative() {
                    output = output.wrapping_add(amplitude);
                }
                amplitude >>= 1;
            }
            append_level(out, output, LOOP_SCALE);
            timer = timer.wrapping_add(1);
            if timer == 0 {
                let mut any = false;
                for index in 0..table.len() {
                    let mut freq = table[index].0;
                    if freq == 0 {
                        continue;
                    }
                    if freq == 0x37 && index + 1 < table.len() {
                        table[index + 1].0 = 0x41;
                    }
                    freq = freq.wrapping_sub(1);
                    table[index].0 = freq;
                    any = any || freq != 0;
                }
                if !any {
                    break;
                }
            }
        }
    }

    fn append_wave(&mut self, out: &mut Vec<f32>, waveform: &[u8], period: u16) {
        let repeats = (usize::from(period) / 8).max(1);
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
        let mut a = self.lo;
        a >>= 1;
        a >>= 1;
        a >>= 1;
        a ^= self.lo;
        let carry = a & 1;
        let hi_carry = self.hi & 1;
        self.hi = (self.hi >> 1) | (carry << 7);
        self.lo = (self.lo >> 1) | (hi_carry << 7);
        (self.lo & 1) != 0
    }

    fn next_random_byte(&mut self) -> u8 {
        for _ in 0..8 {
            self.next_random_bit();
        }
        self.lo
    }
}

#[derive(Debug, Clone, Copy)]
struct GWaveVector {
    gecho: u8,
    gccnt: u8,
    gecdec: u8,
    waveform: &'static [u8],
    pre_decay: u8,
    delta_freq_inc: i8,
    delta_freq_count: u8,
    pattern: &'static [u8],
}

#[derive(Debug, Clone, Copy)]
struct OrganStep {
    oscillator_mask: u8,
    delay: u8,
    samples: usize,
}

fn append_level(out: &mut Vec<f32>, level: u8, repeats: usize) {
    let sample = u8_to_sample(level);
    out.extend(std::iter::repeat_n(sample, repeats.max(1)));
}

fn u8_to_sample(level: u8) -> f32 {
    (((level as f32 / 255.0) * 2.0) - 1.0) * SAMPLE_GAIN
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

fn decay_waveform(waveform: &mut [u8], decay_factor: u8) {
    if decay_factor == 0 {
        return;
    }
    for sample in waveform {
        let decay = (sample.saturating_sub(127) / 16).saturating_mul(decay_factor.max(1));
        *sample = sample.saturating_sub(decay);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GWaveSound, OrganNote, OrganTune, RomAction, RomSoundBoard, SAMPLE_RATE, SpecialRoutine,
    };

    #[test]
    fn special_and_gwave_routines_generate_samples() {
        let mut board = RomSoundBoard::default();
        let rendered = board.render_actions(&[
            RomAction::Special(SpecialRoutine::Radio),
            RomAction::Special(SpecialRoutine::Cannon),
            RomAction::GWave(GWaveSound::Bonus),
        ]);

        assert_eq!(rendered.sample_rate, SAMPLE_RATE);
        assert!(!rendered.samples.is_empty());
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn organ_tune_and_note_generate_non_silent_audio() {
        let mut board = RomSoundBoard::default();
        let rendered = board.render_actions(&[
            RomAction::OrganTune(OrganTune::Phantom),
            RomAction::OrganNote(OrganNote::Cs2),
        ]);

        assert!(rendered.samples.len() > SAMPLE_RATE as usize / 4);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn scream_generates_a_decaying_tail() {
        let mut board = RomSoundBoard::default();
        let rendered = board.render_actions(&[RomAction::Special(SpecialRoutine::Scream)]);
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
}
