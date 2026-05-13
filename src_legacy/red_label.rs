//! Direct Rust translations of small red-label machine routines and constants.
//!
//! This module is the start of the assembly-to-Rust rewrite. Each item here is
//! either a red-label data table or a tiny routine that can be unit-tested
//! independently before larger `defa7.src` / `defb6.src` process translation.
//! Static data tables are embedded from `assets/red-label/`.

use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fixed16(pub i32);

impl Fixed16 {
    pub const ZERO: Self = Self(0);

    pub const fn from_parts(integer: i16, fraction: u16) -> Self {
        Self(((integer as i32) << 16) | fraction as i32)
    }

    pub const fn raw(self) -> i32 {
        self.0
    }

    pub const fn integer(self) -> i16 {
        (self.0 >> 16) as i16
    }

    pub fn wrapping_add(self, delta: Self) -> Self {
        Self(self.0.wrapping_add(delta.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    Left,
    Right,
}

impl Facing {
    pub const fn reversed(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandState {
    pub seed: u8,
    pub hseed: u8,
    pub lseed: u8,
}

impl Default for RandState {
    fn default() -> Self {
        Self {
            seed: 0,
            hseed: 0xA5,
            lseed: 0x5A,
        }
    }
}

impl RandState {
    /// Red-label `RAND` translation.
    ///
    /// Source: `defa7.src` `RAND`, as cited in `README.md` source materials
    /// and preserved in the archived prototype notes under `oldsrc/src/game.rs`.
    pub fn advance(&mut self) {
        let product_low = self.seed.wrapping_mul(3).wrapping_add(17);
        let mut a = self.lseed >> 3;
        a ^= self.lseed;
        let carry_into_hseed = (a & 0x01) != 0;
        let old_hseed = self.hseed;
        self.hseed = (u8::from(carry_into_hseed) << 7) | (self.hseed >> 1);
        let carry_into_lseed = (old_hseed & 0x01) != 0;
        self.lseed = (u8::from(carry_into_lseed) << 7) | (self.lseed >> 1);
        let (with_lseed, carry) = adc8(product_low, self.lseed, false);
        let (new_seed, _) = adc8(with_lseed, self.hseed, carry);
        self.seed = new_seed;
    }

    pub fn advanced(mut self) -> Self {
        self.advance();
        self
    }
}

/// Red-label `RMAX` helper translation.
///
/// Source: random bounded-count helper used by pod/swarmer setup paths in
/// `defb6.src`, preserved in archived prototype extraction notes.
pub fn rmax(max: u8, mut seed: u8) -> u8 {
    while seed > max {
        seed >>= 1;
    }
    seed.wrapping_add(1)
}

fn adc8(lhs: u8, rhs: u8, carry: bool) -> (u8, bool) {
    let sum = u16::from(lhs) + u16::from(rhs) + u16::from(u8::from(carry));
    ((sum & 0xFF) as u8, sum > 0xFF)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelDefaults {
    pub lives: u8,
    pub smart_bombs: u8,
    pub wave: u8,
    pub human_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreTable {
    pub lander: u32,
    pub mutant: u32,
    pub baiter: u32,
    pub bomber: u32,
    pub pod: u32,
    pub swarmer: u32,
    pub hazard_collision: u32,
    pub human_catch: u32,
    pub human_return: u32,
    pub safe_fall: u32,
    pub bonus_stock: u32,
}

impl ScoreTable {
    pub const fn for_enemy(self, kind: EnemyKind) -> u32 {
        match kind {
            EnemyKind::Lander => self.lander,
            EnemyKind::Mutant => self.mutant,
            EnemyKind::Baiter => self.baiter,
            EnemyKind::Bomber => self.bomber,
            EnemyKind::Pod => self.pod,
            EnemyKind::Swarmer => self.swarmer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreSeed {
    pub initials: &'static str,
    pub score: u32,
}

pub fn defaults() -> &'static RedLabelDefaults {
    static DEFAULTS: OnceLock<RedLabelDefaults> = OnceLock::new();
    DEFAULTS.get_or_init(|| {
        parse_defaults(crate::assets::RED_LABEL_DEFAULTS_TSV)
            .expect("embedded red-label defaults should parse")
    })
}

pub fn score_table() -> &'static ScoreTable {
    static SCORES: OnceLock<ScoreTable> = OnceLock::new();
    SCORES.get_or_init(|| {
        parse_score_table(crate::assets::RED_LABEL_SCORES_TSV)
            .expect("embedded red-label scores should parse")
    })
}

pub fn default_high_scores() -> &'static [HighScoreSeed] {
    static HIGH_SCORES: OnceLock<Vec<HighScoreSeed>> = OnceLock::new();
    HIGH_SCORES
        .get_or_init(|| {
            parse_high_scores(crate::assets::RED_LABEL_HIGH_SCORES_TSV)
                .expect("embedded red-label high-score seeds should parse")
        })
        .as_slice()
}

pub fn bonus_stock_score() -> u32 {
    score_table().bonus_stock
}

pub fn human_count() -> usize {
    defaults().human_count
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Lander,
    Mutant,
    Baiter,
    Bomber,
    Pod,
    Swarmer,
}

pub fn score_for_enemy(kind: EnemyKind) -> u32 {
    score_table().for_enemy(kind)
}

fn parse_defaults(text: &'static str) -> Result<RedLabelDefaults, String> {
    let rows = key_value_rows(text, "key\tvalue")?;
    let mut lives = None;
    let mut smart_bombs = None;
    let mut wave = None;
    let mut human_count = None;

    for (line_number, key, value) in rows {
        match key {
            "lives" => lives = Some(parse_u8(value, line_number, key)?),
            "smart_bombs" => smart_bombs = Some(parse_u8(value, line_number, key)?),
            "wave" => wave = Some(parse_u8(value, line_number, key)?),
            "human_count" => human_count = Some(parse_usize(value, line_number, key)?),
            other => return Err(format!("unknown default key {other} on line {line_number}")),
        }
    }

    Ok(RedLabelDefaults {
        lives: require_value(lives, "lives")?,
        smart_bombs: require_value(smart_bombs, "smart_bombs")?,
        wave: require_value(wave, "wave")?,
        human_count: require_value(human_count, "human_count")?,
    })
}

fn parse_score_table(text: &'static str) -> Result<ScoreTable, String> {
    let rows = key_value_rows(text, "kind\tscore")?;
    let mut lander = None;
    let mut mutant = None;
    let mut baiter = None;
    let mut bomber = None;
    let mut pod = None;
    let mut swarmer = None;
    let mut hazard_collision = None;
    let mut human_catch = None;
    let mut human_return = None;
    let mut safe_fall = None;
    let mut bonus_stock = None;

    for (line_number, key, value) in rows {
        let score = parse_u32(value, line_number, key)?;
        match key {
            "lander" => lander = Some(score),
            "mutant" => mutant = Some(score),
            "baiter" => baiter = Some(score),
            "bomber" => bomber = Some(score),
            "pod" => pod = Some(score),
            "swarmer" => swarmer = Some(score),
            "hazard_collision" => hazard_collision = Some(score),
            "human_catch" => human_catch = Some(score),
            "human_return" => human_return = Some(score),
            "safe_fall" => safe_fall = Some(score),
            "bonus_stock" => bonus_stock = Some(score),
            other => return Err(format!("unknown score key {other} on line {line_number}")),
        }
    }

    Ok(ScoreTable {
        lander: require_value(lander, "lander")?,
        mutant: require_value(mutant, "mutant")?,
        baiter: require_value(baiter, "baiter")?,
        bomber: require_value(bomber, "bomber")?,
        pod: require_value(pod, "pod")?,
        swarmer: require_value(swarmer, "swarmer")?,
        hazard_collision: require_value(hazard_collision, "hazard_collision")?,
        human_catch: require_value(human_catch, "human_catch")?,
        human_return: require_value(human_return, "human_return")?,
        safe_fall: require_value(safe_fall, "safe_fall")?,
        bonus_stock: require_value(bonus_stock, "bonus_stock")?,
    })
}

fn parse_high_scores(text: &'static str) -> Result<Vec<HighScoreSeed>, String> {
    let mut lines = text.lines().enumerate();
    let Some((_, header)) = lines.next() else {
        return Err(String::from("high-score asset is empty"));
    };
    if header != "initials\tscore" {
        return Err(format!("unexpected high-score header: {header}"));
    }

    let mut seeds = Vec::new();
    for (line_index, line) in lines {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let fields = trimmed.split('\t').collect::<Vec<_>>();
        if fields.len() != 2 {
            return Err(format!(
                "high-score line {line_number} has wrong field count"
            ));
        }
        if fields[0].len() != 3 || !fields[0].bytes().all(|byte| byte.is_ascii_uppercase()) {
            return Err(format!(
                "high-score line {line_number} has invalid initials"
            ));
        }

        seeds.push(HighScoreSeed {
            initials: fields[0],
            score: parse_u32(fields[1], line_number, "score")?,
        });
    }

    if seeds.is_empty() {
        return Err(String::from("high-score asset has no rows"));
    }

    Ok(seeds)
}

fn key_value_rows<'a>(
    text: &'a str,
    expected_header: &str,
) -> Result<Vec<(usize, &'a str, &'a str)>, String> {
    let mut lines = text.lines().enumerate();
    let Some((_, header)) = lines.next() else {
        return Err(String::from("TSV asset is empty"));
    };
    if header != expected_header {
        return Err(format!("unexpected TSV header: {header}"));
    }

    let mut rows = Vec::new();
    for (line_index, line) in lines {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let fields = trimmed.split('\t').collect::<Vec<_>>();
        if fields.len() != 2 {
            return Err(format!("TSV line {line_number} has wrong field count"));
        }
        rows.push((line_number, fields[0], fields[1]));
    }

    Ok(rows)
}

fn parse_u8(value: &str, line_number: usize, key: &str) -> Result<u8, String> {
    value
        .parse::<u8>()
        .map_err(|error| format!("{key} on line {line_number} is not a u8: {error}"))
}

fn parse_u32(value: &str, line_number: usize, key: &str) -> Result<u32, String> {
    value
        .parse::<u32>()
        .map_err(|error| format!("{key} on line {line_number} is not a u32: {error}"))
}

fn parse_usize(value: &str, line_number: usize, key: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|error| format!("{key} on line {line_number} is not a usize: {error}"))
}

fn require_value<T>(value: Option<T>, key: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("missing required key {key}"))
}

#[cfg(test)]
mod tests {
    use super::{
        EnemyKind, Facing, RandState, default_high_scores, defaults, human_count, parse_defaults,
        parse_high_scores, parse_score_table, rmax, score_for_enemy, score_table,
    };

    #[test]
    fn rand_sequence_matches_translated_red_label_path() {
        let mut state = RandState::default();
        let mut observed = Vec::new();
        for _ in 0..5 {
            state.advance();
            observed.push((state.seed, state.hseed, state.lseed));
        }

        assert_eq!(
            observed,
            vec![
                (0x90, 0xD2, 0xAD),
                (0x81, 0x69, 0x56),
                (0x74, 0x34, 0xAB),
                (0xDC, 0x1A, 0x55),
                (0x5C, 0x8D, 0x2A),
            ]
        );
    }

    #[test]
    fn rmax_uses_the_cabinet_halving_rule() {
        assert_eq!(rmax(6, 0x80), 5);
        assert_eq!(rmax(6, 6), 7);
        assert_eq!(rmax(6, 0), 1);
    }

    #[test]
    fn mutant_score_matches_red_label_score_card() {
        assert_eq!(score_table().mutant, 150);
        assert_eq!(score_for_enemy(EnemyKind::Mutant), 150);
    }

    #[test]
    fn default_high_score_seed_starts_with_drj() {
        assert_eq!(default_high_scores()[0].initials, "DRJ");
        assert_eq!(default_high_scores()[0].score, 21_270);
    }

    #[test]
    fn fixed16_exposes_raw_and_integer_parts() {
        let value = super::Fixed16::from_parts(12, 0x8000);

        assert_eq!(value.integer(), 12);
        assert_eq!(value.raw(), 0x000C_8000);
        assert_eq!(value.wrapping_add(super::Fixed16(1)).raw(), 0x000C_8001);
    }

    #[test]
    fn enemy_score_table_matches_score_card_values() {
        assert_eq!(score_for_enemy(EnemyKind::Lander), 150);
        assert_eq!(score_for_enemy(EnemyKind::Baiter), 200);
        assert_eq!(score_for_enemy(EnemyKind::Bomber), 250);
        assert_eq!(score_for_enemy(EnemyKind::Pod), 1_000);
        assert_eq!(score_for_enemy(EnemyKind::Swarmer), 150);
    }

    #[test]
    fn facing_reversal_round_trips() {
        assert_eq!(Facing::Right.reversed(), Facing::Left);
        assert_eq!(Facing::Right.reversed().reversed(), Facing::Right);
    }

    #[test]
    fn defaults_are_loaded_from_embedded_asset() {
        assert_eq!(defaults().lives, 3);
        assert_eq!(defaults().smart_bombs, 3);
        assert_eq!(defaults().wave, 1);
        assert_eq!(human_count(), 10);
    }

    #[test]
    fn score_table_is_loaded_from_embedded_asset() {
        assert_eq!(score_table().bonus_stock, 10_000);
        assert_eq!(score_table().hazard_collision, 25);
        assert_eq!(score_table().human_catch, 500);
        assert_eq!(score_table().human_return, 500);
        assert_eq!(score_table().safe_fall, 250);
    }

    #[test]
    fn default_parser_rejects_missing_required_key() {
        let error = parse_defaults("key\tvalue\nlives\t3\nsmart_bombs\t3\nwave\t1\n")
            .expect_err("defaults should fail");

        assert!(error.contains("human_count"));
    }

    #[test]
    fn score_parser_rejects_unknown_key() {
        let error = parse_score_table("kind\tscore\nlander\t150\nwat\t1\n")
            .expect_err("scores should fail");

        assert!(error.contains("unknown score key"));
    }

    #[test]
    fn high_score_parser_rejects_invalid_initials() {
        let error =
            parse_high_scores("initials\tscore\nxy\t100\n").expect_err("high scores should fail");

        assert!(error.contains("invalid initials"));
    }
}
