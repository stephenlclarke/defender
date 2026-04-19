//! Holds the red-label Defender wave data extracted from `blk71.src` `WVTAB`.
//!
//! These records stay compiled into the runtime so the default game path uses immutable
//! red-label data instead of parsing `arcade-rules.txt` or accepting local gameplay overrides.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WaveRecord {
    ceiling: i32,
    floor: i32,
    _intra_delta: i32,
    inter_delta: i32,
    waves: [i32; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedLabelWaveTable {
    landers: WaveRecord,
    bombers: WaveRecord,
    pods: WaveRecord,
    mutants: WaveRecord,
    swarmers: WaveRecord,
    wave_time: WaveRecord,
    wave_size: WaveRecord,
    lander_shot_time: WaveRecord,
    mutant_shot_time: WaveRecord,
    swarmer_shot_time: WaveRecord,
    baiter_time: WaveRecord,
    baiter_shot_time: WaveRecord,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveProfile {
    pub landers: u8,
    pub bombers: u8,
    pub pods: u8,
    pub mutants: u8,
    pub swarmers: u8,
    pub wave_time: u32,
    pub wave_size: u8,
    pub lander_shot_time: u32,
    pub mutant_shot_time: u32,
    pub swarmer_shot_time: u32,
    pub baiter_delay: u32,
    pub baiter_shot_time: u32,
}

pub fn red_label_wave_table() -> &'static RedLabelWaveTable {
    &RED_LABEL_WAVE_TABLE
}

impl RedLabelWaveTable {
    pub fn profile_for_wave(&self, wave: u8) -> WaveProfile {
        WaveProfile {
            landers: self.landers.value_for_wave(wave) as u8,
            bombers: self.bombers.value_for_wave(wave) as u8,
            pods: self.pods.value_for_wave(wave) as u8,
            mutants: self.mutants.value_for_wave(wave) as u8,
            swarmers: self.swarmers.value_for_wave(wave) as u8,
            wave_time: self.wave_time.value_for_wave(wave) as u32,
            wave_size: self.wave_size.value_for_wave(wave) as u8,
            lander_shot_time: self.lander_shot_time.value_for_wave(wave) as u32,
            mutant_shot_time: self.mutant_shot_time.value_for_wave(wave) as u32,
            swarmer_shot_time: self.swarmer_shot_time.value_for_wave(wave) as u32,
            baiter_delay: self.baiter_time.value_for_wave(wave) as u32,
            baiter_shot_time: self.baiter_shot_time.value_for_wave(wave) as u32,
        }
    }
}

impl WaveRecord {
    fn value_for_wave(self, wave: u8) -> i32 {
        let wave = wave.max(1);
        if wave <= 4 {
            return self.waves[wave as usize - 1];
        }

        let mut value = self.waves[3];
        for _ in 0..wave.saturating_sub(4) {
            value = apply_delta(value, self.inter_delta, self.floor, self.ceiling);
        }
        value
    }
}

fn apply_delta(value: i32, delta: i32, floor: i32, ceiling: i32) -> i32 {
    if delta > 0 {
        (value + delta).min(ceiling)
    } else if delta < 0 {
        (value + delta).max(floor)
    } else {
        value
    }
}

const fn wave_record(
    ceiling: i32,
    floor: i32,
    intra_delta: i32,
    inter_delta: i32,
    waves: [i32; 4],
) -> WaveRecord {
    WaveRecord {
        ceiling,
        floor,
        _intra_delta: intra_delta,
        inter_delta,
        waves,
    }
}

static RED_LABEL_WAVE_TABLE: RedLabelWaveTable = RedLabelWaveTable {
    landers: wave_record(20, 0, 0, 0, [15, 20, 20, 20]),
    bombers: wave_record(3, 0, 0, 0, [0, 3, 4, 5]),
    pods: wave_record(6, 0, 0, 0, [0, 1, 3, 4]),
    mutants: wave_record(10, 0, 0, 0, [0, 0, 0, 0]),
    swarmers: wave_record(10, 0, 0, 0, [0, 0, 0, 0]),
    wave_time: wave_record(30, 0, 0, 0, [30, 25, 20, 16]),
    wave_size: wave_record(5, 0, 0, 0, [5, 5, 5, 5]),
    lander_shot_time: wave_record(128, 16, -4, -2, [74, 58, 42, 42]),
    mutant_shot_time: wave_record(255, 8, -2, -2, [42, 34, 30, 28]),
    swarmer_shot_time: wave_record(40, 10, -2, -1, [25, 25, 25, 25]),
    baiter_time: wave_record(192, 24, -12, -4, [212, 196, 164, 148]),
    baiter_shot_time: wave_record(10, 3, -1, -1, [15, 13, 12, 10]),
};

#[cfg(test)]
mod tests {
    use super::red_label_wave_table;

    #[test]
    fn red_label_defaults_match_known_wave_one_and_two_counts() {
        let table = red_label_wave_table();
        let wave_one = table.profile_for_wave(1);
        let wave_two = table.profile_for_wave(2);

        assert_eq!(wave_one.landers, 15);
        assert_eq!(wave_one.bombers, 0);
        assert_eq!(wave_one.pods, 0);
        assert_eq!(wave_one.wave_time, 30);
        assert_eq!(wave_one.wave_size, 5);
        assert_eq!(wave_one.lander_shot_time, 74);
        assert_eq!(wave_one.mutant_shot_time, 42);
        assert_eq!(wave_one.swarmer_shot_time, 25);
        assert_eq!(wave_one.baiter_shot_time, 15);
        assert_eq!(wave_two.landers, 20);
        assert_eq!(wave_two.bombers, 3);
        assert_eq!(wave_two.pods, 1);
        assert_eq!(wave_two.baiter_delay, 196);
        assert_eq!(wave_two.lander_shot_time, 58);
        assert_eq!(wave_two.mutant_shot_time, 34);
        assert_eq!(wave_two.baiter_shot_time, 13);
    }

    #[test]
    fn later_waves_follow_the_recorded_inter_delta() {
        let table = red_label_wave_table();
        let wave_four = table.profile_for_wave(4);
        let wave_five = table.profile_for_wave(5);

        assert_eq!(wave_four.wave_time, 16);
        assert_eq!(wave_five.wave_time, 16);
        assert_eq!(wave_four.baiter_delay, 148);
        assert_eq!(wave_five.baiter_delay, 144);
        assert_eq!(wave_four.lander_shot_time, 42);
        assert_eq!(wave_five.lander_shot_time, 40);
        assert_eq!(wave_four.baiter_shot_time, 10);
        assert_eq!(wave_five.baiter_shot_time, 9);
    }
}
