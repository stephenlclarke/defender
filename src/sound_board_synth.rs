const DEFAULT_RANDOM_HIGH_SEED: u8 = 0x3C;
const MAX_SOUND_LEVEL: u8 = 0xFF;
const TURBO_NOISE_CYCLE_COUNT: usize = 0x20;
const PHANTOM_ORGAN_STEPS: [OrganStep; 3] = [
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
const SAW_VARI_VECTOR: VariVector = VariVector {
    low_period: 0x40,
    high_period: 0x01,
    low_delta: 0,
    high_delta: 0x10,
    high_end: 0xE1,
    sweep_period: 0x0080,
    low_mod: MAX_SOUND_LEVEL,
    amplitude: MAX_SOUND_LEVEL,
};
const FALLING_VARI_VECTOR: VariVector = VariVector {
    low_period: 0x28,
    high_period: 0x01,
    low_delta: 0,
    high_delta: 0x08,
    high_end: 0x81,
    sweep_period: 0x0200,
    low_mod: MAX_SOUND_LEVEL,
    amplitude: MAX_SOUND_LEVEL,
};
const QUASAR_VARI_VECTOR: VariVector = VariVector {
    low_period: 0x28,
    high_period: 0x81,
    low_delta: 0,
    high_delta: 0xFC,
    high_end: 0x01,
    sweep_period: 0x0200,
    low_mod: 0xFC,
    amplitude: MAX_SOUND_LEVEL,
};
const ORGAN_NOTE_CS2_OSCILLATOR_MASK: u8 = 0x7C;
const VARI_LEVEL_SIGN_BIT: u8 = 0x80;
const RADIO_INITIAL_FREQUENCY: u16 = 0x0100;
const RADIO_WAVEFORM_INDEX_MASK: u16 = 0x0F;
const HYPER_PHASE_MAX: u8 = 0x7F;
const HYPER_COUNTER_MAX: u8 = 0x7F;
const SCREAM_INITIAL_FREQUENCY: u8 = 0x40;
const SCREAM_INITIAL_AMPLITUDE: u8 = 0x80;
const SCREAM_CASCADE_TRIGGER_FREQUENCY: u8 = 0x37;
const SCREAM_CASCADE_START_FREQUENCY: u8 = 0x41;

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
            random_hi: DEFAULT_RANDOM_HIGH_SEED,
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
            SpecialSound::Turbo => self.render_white_noise(
                1,
                TURBO_NOISE_CYCLE_COUNT,
                1,
                MAX_SOUND_LEVEL,
                true,
                out,
            ),
            SpecialSound::Materialize => self.render_materialize_noise(out),
            SpecialSound::Thrust => {
                self.background_1_active = 1;
                self.render_filtered_noise(false, false, 4, 900, out);
            }
            SpecialSound::Cannon => {
                self.render_filtered_noise(true, true, MAX_SOUND_LEVEL, 16_000, out)
            }
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
        let base_waveform = vector.waveform;
        let mut waveform = base_waveform.to_vec();
        decay_waveform(&mut waveform, base_waveform, vector.pre_decay);
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
                decay_waveform(&mut waveform, base_waveform, vector.echo_decay);
            }

            if vector.delta_frequency_increment == 0 {
                break;
            }
            delta_counter = delta_counter.wrapping_sub(1);
            if delta_counter == 0 {
                break;
            }
            frequency_offset += i16::from(vector.delta_frequency_increment);
            let Some((next_start, next_end)) = adjusted_pattern_range(
                &vector.pattern[pattern_start..pattern_end],
                frequency_offset,
            ) else {
                break;
            };
            pattern_start += next_start;
            pattern_end = pattern_start + (next_end - next_start);
            if vector.echo_decay != 0 {
                waveform.clear();
                waveform.extend_from_slice(base_waveform);
                decay_waveform(&mut waveform, base_waveform, vector.pre_decay);
            }
        }
    }

    fn render_organ_tune(&mut self, tune: OrganTune, out: &mut Vec<f32>) {
        match tune {
            OrganTune::Phantom => {
                self.organ_active = 1;
                for step in PHANTOM_ORGAN_STEPS {
                    self.append_organ_step(out, step);
                }
            }
        }
    }

    fn render_vari(&mut self, sound: VariSound, out: &mut Vec<f32>) {
        let vector = match sound {
            VariSound::Saw => SAW_VARI_VECTOR,
            VariSound::Falling => FALLING_VARI_VECTOR,
            VariSound::Quasar => QUASAR_VARI_VECTOR,
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
                oscillator_mask: ORGAN_NOTE_CS2_OSCILLATOR_MASK,
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
        let mut level = MAX_SOUND_LEVEL;
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
        let mut level = MAX_SOUND_LEVEL;
        let mut frequency = APPEAR_INITIAL_FREQUENCY;
        append_level(out, level, materialize_repeat_count(frequency));

        loop {
            for _ in 0..APPEAR_CYCLE_COUNT {
                if self.next_noise_random_carry_bit() {
                    level = !level;
                }
                append_level(out, level, materialize_repeat_count(frequency));
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
                    append_vari_level(out, level, vari_repeat_count(low_ticks));
                    sweep_counter = sweep_counter.saturating_sub(low_ticks);
                    if sweep_counter == 0 {
                        break;
                    }

                    level = !level;
                    let high_ticks = u16::from(high_count.max(1)).min(sweep_counter);
                    append_vari_level(out, level, vari_repeat_count(high_ticks));
                    sweep_counter = sweep_counter.saturating_sub(high_ticks);
                }

                level = if level & VARI_LEVEL_SIGN_BIT == 0 {
                    !level
                } else {
                    level
                };
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
        let mut frequency = RADIO_INITIAL_FREQUENCY;
        loop {
            let (next_phase, carry) = phase.overflowing_add(frequency);
            phase = next_phase;
            if carry {
                frequency = frequency.wrapping_add(1);
                if frequency == 0 {
                    break;
                }
            }
            let index = usize::from(((phase >> 8) & RADIO_WAVEFORM_INDEX_MASK) as u8);
            append_level(out, RADIO_WAVEFORM[index], LOOP_SCALE);
            if out.len() > SAMPLE_RATE_HZ as usize {
                break;
            }
        }
    }

    fn render_hyper(&mut self, out: &mut Vec<f32>) {
        let mut sound = 0u8;
        for phase in 0u8..=HYPER_PHASE_MAX {
            for counter in 0u8..=HYPER_COUNTER_MAX {
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
        table[0].0 = SCREAM_INITIAL_FREQUENCY;
        let mut timer = 0u8;
        loop {
            let mut amplitude = SCREAM_INITIAL_AMPLITUDE;
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
                    if frequency == SCREAM_CASCADE_TRIGGER_FREQUENCY && index + 1 < table.len() {
                        table[index + 1].0 = SCREAM_CASCADE_START_FREQUENCY;
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

    fn next_noise_random_carry_bit(&mut self) -> bool {
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
