//! Holds the red-label Defender wave data extracted from `blk71.src` `WVTAB`.
//!
//! The checked-in source data lives in `assets/red-label/wave-table.tsv`; this
//! module parses that embedded asset once so gameplay code consumes immutable
//! red-label data instead of accepting local gameplay overrides.

use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const RED_LABEL_WDELT_RECORD_COUNT: usize = 23;

const WDELT_KEY_ORDER: [&str; RED_LABEL_WDELT_RECORD_COUNT] = [
    "landers",
    "bombers",
    "pods",
    "mutants",
    "swarmers",
    "wave_time",
    "wave_size",
    "lander_x_velocity",
    "lander_y_velocity_msb",
    "lander_y_velocity_lsb",
    "lander_shot_time",
    "bomber_x_velocity",
    "mutant_random_y",
    "mutant_y_velocity_msb",
    "mutant_y_velocity_lsb",
    "mutant_x_velocity",
    "mutant_shot_time",
    "swarmer_x_velocity",
    "swarmer_shot_time",
    "swarmer_acceleration_mask",
    "baiter_time",
    "baiter_shot_time",
    "baiter_seek_probability",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WaveRecord {
    ceiling: i32,
    floor: i32,
    intra_delta: i32,
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
    lander_x_velocity: WaveRecord,
    lander_y_velocity_msb: WaveRecord,
    lander_y_velocity_lsb: WaveRecord,
    mutant_random_y: WaveRecord,
    mutant_y_velocity_msb: WaveRecord,
    mutant_y_velocity_lsb: WaveRecord,
    mutant_x_velocity: WaveRecord,
    swarmer_x_velocity: WaveRecord,
    wave_time: WaveRecord,
    wave_size: WaveRecord,
    lander_shot_time: WaveRecord,
    bomber_x_velocity: WaveRecord,
    mutant_shot_time: WaveRecord,
    swarmer_shot_time: WaveRecord,
    swarmer_acceleration_mask: WaveRecord,
    baiter_time: WaveRecord,
    baiter_shot_time: WaveRecord,
    baiter_seek_probability: WaveRecord,
    wdelt_source_order: [WaveRecord; RED_LABEL_WDELT_RECORD_COUNT],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveProfile {
    pub landers: u8,
    pub bombers: u8,
    pub pods: u8,
    pub mutants: u8,
    pub swarmers: u8,
    pub lander_x_velocity: u8,
    pub lander_y_velocity_msb: u8,
    pub lander_y_velocity_lsb: u8,
    pub mutant_random_y: u8,
    pub mutant_y_velocity_msb: u8,
    pub mutant_y_velocity_lsb: u8,
    pub mutant_x_velocity: u8,
    pub swarmer_x_velocity: u8,
    pub wave_time: u32,
    pub wave_size: u8,
    pub lander_shot_time: u32,
    pub bomber_x_velocity: u8,
    pub mutant_shot_time: u32,
    pub swarmer_shot_time: u32,
    pub swarmer_acceleration_mask: u8,
    pub baiter_delay: u32,
    pub baiter_shot_time: u32,
    pub baiter_seek_probability: u8,
}

pub fn red_label_wave_table() -> &'static RedLabelWaveTable {
    static TABLE: OnceLock<RedLabelWaveTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        parse_wave_table(crate::assets::RED_LABEL_WAVE_TABLE_TSV)
            .expect("embedded red-label wave table should parse")
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
            lander_x_velocity: self.lander_x_velocity.value_for_wave(wave) as u8,
            lander_y_velocity_msb: self.lander_y_velocity_msb.value_for_wave(wave) as u8,
            lander_y_velocity_lsb: self.lander_y_velocity_lsb.value_for_wave(wave) as u8,
            mutant_random_y: self.mutant_random_y.value_for_wave(wave) as u8,
            mutant_y_velocity_msb: self.mutant_y_velocity_msb.value_for_wave(wave) as u8,
            mutant_y_velocity_lsb: self.mutant_y_velocity_lsb.value_for_wave(wave) as u8,
            mutant_x_velocity: self.mutant_x_velocity.value_for_wave(wave) as u8,
            swarmer_x_velocity: self.swarmer_x_velocity.value_for_wave(wave) as u8,
            wave_time: self.wave_time.value_for_wave(wave) as u32,
            wave_size: self.wave_size.value_for_wave(wave) as u8,
            lander_shot_time: self.lander_shot_time.value_for_wave(wave) as u32,
            bomber_x_velocity: self.bomber_x_velocity.value_for_wave(wave) as u8,
            mutant_shot_time: self.mutant_shot_time.value_for_wave(wave) as u32,
            swarmer_shot_time: self.swarmer_shot_time.value_for_wave(wave) as u32,
            swarmer_acceleration_mask: self.swarmer_acceleration_mask.value_for_wave(wave) as u8,
            baiter_delay: self.baiter_time.value_for_wave(wave) as u32,
            baiter_shot_time: self.baiter_shot_time.value_for_wave(wave) as u32,
            baiter_seek_probability: self.baiter_seek_probability.value_for_wave(wave) as u8,
        }
    }

    /// Copy the same source-order `WVTAB` value bytes that red-label `GETWV`
    /// copies into `PENEMY` before applying difficulty deltas.
    ///
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1865-L1876>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L674-L723>.
    pub fn getwv_base_values(
        &self,
        wave: u8,
    ) -> Result<[u8; RED_LABEL_WDELT_RECORD_COUNT], String> {
        let source_column = if wave > 4 { 7 } else { usize::from(wave) + 3 };
        let mut values = [0; RED_LABEL_WDELT_RECORD_COUNT];
        for (value, record) in values.iter_mut().zip(self.wdelt_source_order) {
            *value = record.source_byte(source_column)?;
        }
        Ok(values)
    }

    /// Red-label `WDELT` with `B=2`: apply intra-wall deltas from source-order
    /// `WVTAB` records to the caller's `ELIST`/`PENEMY` byte slice.
    ///
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1908-L1924>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L674-L723>.
    pub fn apply_intra_wall_deltas(&self, values: &mut [u8]) -> Result<usize, String> {
        self.apply_wall_deltas(values, RedLabelWallDeltaColumn::Intra)
    }

    /// Red-label `WDELT` with `B=3`: apply inter-wall deltas from source-order
    /// `WVTAB` records to the caller's `PENEMY` byte slice.
    ///
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1884-L1899>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1908-L1924>.
    pub fn apply_inter_wall_deltas(&self, values: &mut [u8]) -> Result<usize, String> {
        self.apply_wall_deltas(values, RedLabelWallDeltaColumn::Inter)
    }

    fn apply_wall_deltas(
        &self,
        values: &mut [u8],
        delta_column: RedLabelWallDeltaColumn,
    ) -> Result<usize, String> {
        if values.len() < RED_LABEL_WDELT_RECORD_COUNT {
            return Err(format!(
                "red-label WDELT requires {RED_LABEL_WDELT_RECORD_COUNT} value byte(s), got {}",
                values.len()
            ));
        }

        let mut changed = 0;
        for (value, record) in values
            .iter_mut()
            .take(RED_LABEL_WDELT_RECORD_COUNT)
            .zip(self.wdelt_source_order)
        {
            let previous = *value;
            *value = record.apply_delta(previous, delta_column.delta(record))?;
            if *value != previous {
                changed += 1;
            }
        }
        Ok(changed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RedLabelWallDeltaColumn {
    Intra,
    Inter,
}

impl RedLabelWallDeltaColumn {
    fn delta(self, record: WaveRecord) -> i32 {
        match self {
            Self::Intra => record.intra_delta,
            Self::Inter => record.inter_delta,
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

    fn apply_delta(self, value: u8, delta: i32) -> Result<u8, String> {
        let ceiling = u8::try_from(self.ceiling)
            .map_err(|_| format!("red-label WVTAB ceiling {} is not a byte", self.ceiling))?;
        let floor = u8::try_from(self.floor)
            .map_err(|_| format!("red-label WVTAB floor {} is not a byte", self.floor))?;
        if delta >= 0 {
            let delta = u16::try_from(delta)
                .map_err(|_| format!("red-label WVTAB positive delta {delta} is invalid"))?;
            let sum = u16::from(value) + delta;
            if sum > 0xFF {
                return Ok(value);
            }
            let candidate = sum as u8;
            if candidate > ceiling {
                Ok(value)
            } else {
                Ok(candidate)
            }
        } else {
            let delta = i16::try_from(delta)
                .map_err(|_| format!("red-label WVTAB negative delta {delta} is invalid"))?;
            let candidate = i16::from(value) + delta;
            if candidate < 0 {
                return Ok(value);
            }
            let candidate = candidate as u8;
            if candidate < floor {
                Ok(value)
            } else {
                Ok(candidate)
            }
        }
    }

    fn source_byte(self, source_column: usize) -> Result<u8, String> {
        let value = match source_column {
            0 => self.ceiling,
            1 => self.floor,
            2 => self.intra_delta,
            3 => self.inter_delta,
            4..=7 => self.waves[source_column - 4],
            _ => {
                return Err(format!(
                    "red-label WVTAB source column {source_column} is out of range"
                ));
            }
        };
        if !(-128..=255).contains(&value) {
            return Err(format!(
                "red-label WVTAB source value {value} is outside an 8-bit FCB"
            ));
        }
        Ok(value as u8)
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

fn parse_wave_table(text: &'static str) -> Result<RedLabelWaveTable, String> {
    let mut lines = text.lines().enumerate();
    let Some((_, header)) = lines.next() else {
        return Err(String::from("wave table asset is empty"));
    };
    if header != "key\tceiling\tfloor\tintra_delta\tinter_delta\twave1\twave2\twave3\twave4" {
        return Err(format!("unexpected wave table header: {header}"));
    }

    let mut records = BTreeMap::new();
    for (line_index, line) in lines {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let fields = trimmed.split('\t').collect::<Vec<_>>();
        if fields.len() != 9 {
            return Err(format!(
                "wave table line {line_number} has wrong field count"
            ));
        }
        if records
            .insert(fields[0], parse_wave_record(&fields, line_number)?)
            .is_some()
        {
            return Err(format!("duplicate wave table key {}", fields[0]));
        }
    }

    let wdelt_source_order = wdelt_source_records(&records)?;
    let table = RedLabelWaveTable {
        landers: take_record(&mut records, "landers")?,
        bombers: take_record(&mut records, "bombers")?,
        pods: take_record(&mut records, "pods")?,
        mutants: take_record(&mut records, "mutants")?,
        swarmers: take_record(&mut records, "swarmers")?,
        lander_x_velocity: take_record(&mut records, "lander_x_velocity")?,
        lander_y_velocity_msb: take_record(&mut records, "lander_y_velocity_msb")?,
        lander_y_velocity_lsb: take_record(&mut records, "lander_y_velocity_lsb")?,
        mutant_random_y: take_record(&mut records, "mutant_random_y")?,
        mutant_y_velocity_msb: take_record(&mut records, "mutant_y_velocity_msb")?,
        mutant_y_velocity_lsb: take_record(&mut records, "mutant_y_velocity_lsb")?,
        mutant_x_velocity: take_record(&mut records, "mutant_x_velocity")?,
        swarmer_x_velocity: take_record(&mut records, "swarmer_x_velocity")?,
        wave_time: take_record(&mut records, "wave_time")?,
        wave_size: take_record(&mut records, "wave_size")?,
        lander_shot_time: take_record(&mut records, "lander_shot_time")?,
        bomber_x_velocity: take_record(&mut records, "bomber_x_velocity")?,
        mutant_shot_time: take_record(&mut records, "mutant_shot_time")?,
        swarmer_shot_time: take_record(&mut records, "swarmer_shot_time")?,
        swarmer_acceleration_mask: take_record(&mut records, "swarmer_acceleration_mask")?,
        baiter_time: take_record(&mut records, "baiter_time")?,
        baiter_shot_time: take_record(&mut records, "baiter_shot_time")?,
        baiter_seek_probability: take_record(&mut records, "baiter_seek_probability")?,
        wdelt_source_order,
    };

    if let Some(unknown) = records.keys().next() {
        return Err(format!("unknown wave table key {unknown}"));
    }

    Ok(table)
}

fn parse_wave_record(fields: &[&str], line_number: usize) -> Result<WaveRecord, String> {
    Ok(wave_record(
        parse_i32(fields[1], line_number, "ceiling")?,
        parse_i32(fields[2], line_number, "floor")?,
        parse_i32(fields[3], line_number, "intra_delta")?,
        parse_i32(fields[4], line_number, "inter_delta")?,
        [
            parse_i32(fields[5], line_number, "wave1")?,
            parse_i32(fields[6], line_number, "wave2")?,
            parse_i32(fields[7], line_number, "wave3")?,
            parse_i32(fields[8], line_number, "wave4")?,
        ],
    ))
}

fn parse_i32(value: &str, line_number: usize, field: &str) -> Result<i32, String> {
    value
        .parse::<i32>()
        .map_err(|error| format!("{field} on line {line_number} is not an i32: {error}"))
}

fn take_record(
    records: &mut BTreeMap<&'static str, WaveRecord>,
    key: &str,
) -> Result<WaveRecord, String> {
    records
        .remove(key)
        .ok_or_else(|| format!("missing wave table key {key}"))
}

fn wdelt_source_records(
    records: &BTreeMap<&'static str, WaveRecord>,
) -> Result<[WaveRecord; RED_LABEL_WDELT_RECORD_COUNT], String> {
    let mut source_records = [wave_record(0, 0, 0, 0, [0; 4]); RED_LABEL_WDELT_RECORD_COUNT];
    for (index, key) in WDELT_KEY_ORDER.iter().enumerate() {
        source_records[index] = *records
            .get(key)
            .ok_or_else(|| format!("missing wave table key {key}"))?;
    }
    Ok(source_records)
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
        intra_delta,
        inter_delta,
        waves,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_wave_table, red_label_wave_table};

    #[test]
    fn red_label_defaults_match_known_wave_one_and_two_counts() {
        let table = red_label_wave_table();
        let wave_one = table.profile_for_wave(1);
        let wave_two = table.profile_for_wave(2);

        assert_eq!(wave_one.landers, 15);
        assert_eq!(wave_one.bombers, 0);
        assert_eq!(wave_one.pods, 0);
        assert_eq!(wave_one.lander_x_velocity, 22);
        assert_eq!(wave_one.lander_y_velocity_msb, 0);
        assert_eq!(wave_one.lander_y_velocity_lsb, 112);
        assert_eq!(wave_one.mutant_random_y, 1);
        assert_eq!(wave_one.mutant_y_velocity_msb, 0);
        assert_eq!(wave_one.mutant_y_velocity_lsb, 98);
        assert_eq!(wave_one.mutant_x_velocity, 12);
        assert_eq!(wave_one.swarmer_x_velocity, 22);
        assert_eq!(wave_one.wave_time, 30);
        assert_eq!(wave_one.wave_size, 5);
        assert_eq!(wave_one.lander_shot_time, 74);
        assert_eq!(wave_one.bomber_x_velocity, 32);
        assert_eq!(wave_one.mutant_shot_time, 42);
        assert_eq!(wave_one.swarmer_shot_time, 25);
        assert_eq!(wave_one.swarmer_acceleration_mask, 31);
        assert_eq!(wave_one.baiter_shot_time, 15);
        assert_eq!(wave_one.baiter_seek_probability, 240);
        assert_eq!(wave_two.landers, 20);
        assert_eq!(wave_two.bombers, 3);
        assert_eq!(wave_two.pods, 1);
        assert_eq!(wave_two.lander_x_velocity, 30);
        assert_eq!(wave_two.lander_y_velocity_msb, 0);
        assert_eq!(wave_two.lander_y_velocity_lsb, 176);
        assert_eq!(wave_two.mutant_random_y, 1);
        assert_eq!(wave_two.mutant_y_velocity_msb, 0);
        assert_eq!(wave_two.mutant_y_velocity_lsb, 224);
        assert_eq!(wave_two.mutant_x_velocity, 28);
        assert_eq!(wave_two.swarmer_x_velocity, 30);
        assert_eq!(wave_two.baiter_delay, 196);
        assert_eq!(wave_two.lander_shot_time, 58);
        assert_eq!(wave_two.bomber_x_velocity, 40);
        assert_eq!(wave_two.mutant_shot_time, 34);
        assert_eq!(wave_two.swarmer_acceleration_mask, 31);
        assert_eq!(wave_two.baiter_shot_time, 13);
        assert_eq!(wave_two.baiter_seek_probability, 220);
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
        assert_eq!(wave_four.baiter_seek_probability, 200);
        assert_eq!(wave_five.baiter_seek_probability, 192);
    }

    #[test]
    fn getwv_base_values_follow_source_wave_columns() {
        let table = red_label_wave_table();

        assert_eq!(
            table.getwv_base_values(1).expect("wave one GETWV base"),
            [
                15, 0, 0, 0, 0, 30, 5, 22, 0, 112, 74, 32, 1, 0, 98, 12, 42, 22, 25, 31, 212, 15,
                240,
            ]
        );
        assert_eq!(
            table.getwv_base_values(4).expect("wave four GETWV base"),
            [
                20, 5, 4, 0, 0, 16, 5, 46, 1, 0, 42, 48, 2, 1, 18, 40, 28, 34, 25, 63, 148, 10,
                200,
            ]
        );
        assert_eq!(
            table.getwv_base_values(5).expect("wave five GETWV base"),
            table.getwv_base_values(4).expect("wave four repeat")
        );
    }

    #[test]
    fn getwv_wave_zero_matches_source_column_selection() {
        let table = red_label_wave_table();

        assert_eq!(
            table.getwv_base_values(0).expect("wrapped GETWV base"),
            [
                0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 254, 0, 0, 0, 6, 4, 254, 2, 255, 0, 252, 255, 248,
            ]
        );
    }

    #[test]
    fn wdelt_inter_wall_deltas_follow_source_order_and_bounds() {
        let table = red_label_wave_table();
        let mut values = table.getwv_base_values(1).expect("wave one GETWV base");

        let changed = table
            .apply_inter_wall_deltas(&mut values)
            .expect("WDELT inter-wall deltas");

        assert_eq!(changed, 10);
        assert_eq!(
            values,
            [
                15, 0, 0, 0, 0, 30, 5, 24, 0, 112, 72, 32, 1, 0, 104, 16, 40, 24, 24, 31, 208, 14,
                232,
            ]
        );
    }

    #[test]
    fn wdelt_intra_wall_deltas_follow_source_order_and_bounds() {
        let table = red_label_wave_table();
        let mut values = [
            1, 2, 3, 4, 5, 30, 5, 94, 0, 240, 20, 32, 1, 0, 250, 88, 9, 92, 11, 31, 30, 4, 60,
        ];

        let changed = table
            .apply_intra_wall_deltas(&mut values)
            .expect("WDELT intra-wall deltas");

        assert_eq!(changed, 4);
        assert_eq!(
            values,
            [
                1, 2, 3, 4, 5, 30, 5, 94, 0, 240, 16, 32, 1, 0, 250, 96, 9, 92, 11, 31, 30, 3, 48,
            ]
        );
    }

    #[test]
    fn wdelt_rejects_short_target_slices() {
        let table = red_label_wave_table();
        let mut values = [0; 22];

        let error = table
            .apply_intra_wall_deltas(&mut values)
            .expect_err("short WDELT target");

        assert!(error.contains("23 value byte"));
    }

    #[test]
    fn wave_table_parser_rejects_missing_records() {
        let error = parse_wave_table(
            "key\tceiling\tfloor\tintra_delta\tinter_delta\twave1\twave2\twave3\twave4\n\
             landers\t20\t0\t0\t0\t15\t20\t20\t20\n",
        )
        .expect_err("wave table should fail");

        assert!(error.contains("missing wave table key bombers"));
    }

    #[test]
    fn wave_table_parser_rejects_malformed_numbers() {
        let error = parse_wave_table(
            "key\tceiling\tfloor\tintra_delta\tinter_delta\twave1\twave2\twave3\twave4\n\
             landers\twat\t0\t0\t0\t15\t20\t20\t20\n",
        )
        .expect_err("wave table should fail");

        assert!(error.contains("ceiling"));
    }
}
