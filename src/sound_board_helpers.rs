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

pub(crate) fn sound_actions_for_command(command: crate::SoundCommand) -> Vec<SoundAction> {
    let sound_number = (!command.byte()) & 0x1F;
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
            waveform: LONG_SQUARE_WAVEFORM,
            pre_decay: 0,
            delta_frequency_increment: 0,
            delta_frequency_count: 0,
            pattern: HIT_BELL_DECAY_PATTERN,
            period_divisor: TONAL_HIT_PERIOD_DIVISOR,
        }),
        2 => Some(GWaveVector {
            echo_count: 1,
            cycle_count: 2,
            echo_decay: 0,
            waveform: RAMPED_SINE_WAVEFORM,
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
            waveform: RAMPED_SINE_WAVEFORM,
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
            waveform: EIGHT_SAMPLE_SQUARE_WAVEFORM,
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
            waveform: SIXTEEN_SAMPLE_SINE_WAVEFORM,
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
            waveform: RAMPED_SINE_WAVEFORM,
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
            waveform: RAMPED_SINE_WAVEFORM,
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
            waveform: ASTRONAUT_CATCH_WAVEFORM,
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
            waveform: EIGHT_SAMPLE_SQUARE_WAVEFORM,
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
            waveform: RAMPED_SINE_WAVEFORM,
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
            waveform: DISTORTED_HIT_WAVEFORM,
            pre_decay: 3,
            delta_frequency_increment: 0,
            delta_frequency_count: 2,
            pattern: SHORT_ECHO_DECAY_PATTERN,
            period_divisor: 8,
        }),
        12 => Some(GWaveVector {
            echo_count: 6,
            cycle_count: 10,
            echo_decay: 1,
            waveform: EIGHT_SAMPLE_SINE_WAVEFORM,
            pre_decay: 2,
            delta_frequency_increment: 0,
            delta_frequency_count: 2,
            pattern: LONG_ECHO_DECAY_PATTERN,
            period_divisor: 8,
        }),
        13 => Some(GWaveVector {
            echo_count: 1,
            cycle_count: 15,
            echo_decay: 1,
            waveform: SIXTEEN_SAMPLE_SINE_WAVEFORM,
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
        waveform: EIGHT_SAMPLE_SQUARE_WAVEFORM,
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
        waveform: TURBINE_SINE_WAVEFORM,
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

fn decay_waveform(waveform: &mut [u8], base_waveform: &[u8], decay_factor: u8) {
    if decay_factor == 0 {
        return;
    }
    for (sample, base_sample) in waveform.iter_mut().zip(base_waveform) {
        let decay_step = base_sample >> 4;
        for _ in 0..decay_factor {
            *sample = sample.wrapping_sub(decay_step);
        }
    }
}

fn adjusted_pattern_range(pattern: &[u8], frequency_offset: i16) -> Option<(usize, usize)> {
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

fn vari_repeat_count(ticks: u16) -> usize {
    (usize::from(ticks) / 8).max(1)
}

fn materialize_repeat_count(frequency: u8) -> usize {
    (usize::from(frequency.max(1)) / APPEAR_DELAY_DIVISOR).max(1)
}
