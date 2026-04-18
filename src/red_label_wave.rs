//! Holds the red-label Defender wave data extracted from `blk71.src` `WVTAB`.

use std::sync::OnceLock;

use crate::customization;

const DEFAULT_WAVE_TABLE: &str = include_str!("../assets/arcade/red-label-wave-table.txt");

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
    baiter_time: WaveRecord,
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
    pub baiter_delay: u32,
}

pub fn red_label_wave_table() -> &'static RedLabelWaveTable {
    static TABLE: OnceLock<RedLabelWaveTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        let text = customization::load_arcade_text("red-label-wave-table.txt", DEFAULT_WAVE_TABLE);
        parse_wave_table(&text)
    })
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
            baiter_delay: self.baiter_time.value_for_wave(wave) as u32,
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

fn parse_wave_table(text: &str) -> RedLabelWaveTable {
    let mut landers = None;
    let mut bombers = None;
    let mut pods = None;
    let mut mutants = None;
    let mut swarmers = None;
    let mut wave_time = None;
    let mut wave_size = None;
    let mut baiter_time = None;

    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .expect("red-label wave table should use key=value lines");
        let record = parse_record(value);
        match key {
            "landers" => landers = Some(record),
            "bombers" => bombers = Some(record),
            "pods" => pods = Some(record),
            "mutants" => mutants = Some(record),
            "swarmers" => swarmers = Some(record),
            "wave_time" => wave_time = Some(record),
            "wave_size" => wave_size = Some(record),
            "baiter_time" => baiter_time = Some(record),
            _ => {}
        }
    }

    RedLabelWaveTable {
        landers: landers.expect("landers record should exist"),
        bombers: bombers.expect("bombers record should exist"),
        pods: pods.expect("pods record should exist"),
        mutants: mutants.expect("mutants record should exist"),
        swarmers: swarmers.expect("swarmers record should exist"),
        wave_time: wave_time.expect("wave_time record should exist"),
        wave_size: wave_size.expect("wave_size record should exist"),
        baiter_time: baiter_time.expect("baiter_time record should exist"),
    }
}

fn parse_record(value: &str) -> WaveRecord {
    let (limits, waves) = value
        .split_once('|')
        .expect("wave records should use limits|waves format");
    let limits = parse_i32_list(limits);
    let waves = parse_i32_list(waves);
    WaveRecord {
        ceiling: limits[0],
        floor: limits[1],
        _intra_delta: limits[2],
        inter_delta: limits[3],
        waves: [waves[0], waves[1], waves[2], waves[3]],
    }
}

fn parse_i32_list(value: &str) -> Vec<i32> {
    value.split(',').map(parse_i32).collect()
}

fn parse_i32(value: &str) -> i32 {
    value.trim().parse().expect("expected signed decimal value")
}

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
        assert_eq!(wave_two.landers, 20);
        assert_eq!(wave_two.bombers, 3);
        assert_eq!(wave_two.pods, 1);
        assert_eq!(wave_two.baiter_delay, 196);
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
    }
}
