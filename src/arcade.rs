//! Holds the extracted Defender arcade tuning tables used by the native Rust implementation.

use std::sync::OnceLock;

use crate::customization;

const ARCADE_RULES: &str = include_str!("../assets/arcade/arcade-rules.txt");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArcadeTables {
    pub safe_fall_height: i32,
    pub safe_fall_score: u16,
    pub human_catch_score: u32,
    pub human_landing_score: u16,
    pub hazard_collision_score: u32,
    pub bonus_stock_score: u32,
    pub max_wave_humanoid_bonus: u32,
    pub player_max_speed: i32,
    pub swarmer_speed: i32,
    pub baiter_speed: i32,
    pub bomber_base_speed: i32,
    pub bomber_evasive_speed: i32,
    pub max_baiters: usize,
    pub baiter_base_delay: u32,
    pub baiter_repeat_delay: u32,
    pub pod_swarmer_burst_min: usize,
    pub pod_swarmer_burst_range: usize,
    pub max_swarmers: usize,
    pub bomber_mine_drop_delay: u32,
    pub max_mines: usize,
    pub attack_wave_group_size: u8,
    pub attack_wave_total_openers: u8,
    pub attack_wave_reinforcement_delay: u32,
    pub default_human_world_xs: Vec<i32>,
}

pub fn arcade_tables() -> &'static ArcadeTables {
    static TABLES: OnceLock<ArcadeTables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let rules = customization::load_arcade_text("arcade-rules.txt", ARCADE_RULES);
        parse_arcade_tables(&rules)
    })
}

fn parse_arcade_tables(text: &str) -> ArcadeTables {
    let mut safe_fall_height = None;
    let mut safe_fall_score = None;
    let mut human_catch_score = None;
    let mut human_landing_score = None;
    let mut hazard_collision_score = None;
    let mut bonus_stock_score = None;
    let mut max_wave_humanoid_bonus = None;
    let mut player_max_speed = None;
    let mut swarmer_speed = None;
    let mut baiter_speed = None;
    let mut bomber_base_speed = None;
    let mut bomber_evasive_speed = None;
    let mut max_baiters = None;
    let mut baiter_base_delay = None;
    let mut baiter_repeat_delay = None;
    let mut pod_swarmer_burst_min = None;
    let mut pod_swarmer_burst_range = None;
    let mut max_swarmers = None;
    let mut bomber_mine_drop_delay = None;
    let mut max_mines = None;
    let mut attack_wave_group_size = None;
    let mut attack_wave_total_openers = None;
    let mut attack_wave_reinforcement_delay = None;
    let mut default_human_world_xs = None;

    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .expect("arcade rules should use key=value lines");
        match key {
            "safe_fall_height" => safe_fall_height = Some(parse_i32(value)),
            "safe_fall_score" => safe_fall_score = Some(parse_u16(value)),
            "human_catch_score" => human_catch_score = Some(parse_u32(value)),
            "human_landing_score" => human_landing_score = Some(parse_u16(value)),
            "hazard_collision_score" => hazard_collision_score = Some(parse_u32(value)),
            "bonus_stock_score" => bonus_stock_score = Some(parse_u32(value)),
            "max_wave_humanoid_bonus" => max_wave_humanoid_bonus = Some(parse_u32(value)),
            "player_max_speed" => player_max_speed = Some(parse_i32(value)),
            "swarmer_speed" => swarmer_speed = Some(parse_i32(value)),
            "baiter_speed" => baiter_speed = Some(parse_i32(value)),
            "bomber_base_speed" => bomber_base_speed = Some(parse_i32(value)),
            "bomber_evasive_speed" => bomber_evasive_speed = Some(parse_i32(value)),
            "max_baiters" => max_baiters = Some(parse_usize(value)),
            "baiter_base_delay" => baiter_base_delay = Some(parse_u32(value)),
            "baiter_repeat_delay" => baiter_repeat_delay = Some(parse_u32(value)),
            "pod_swarmer_burst_min" => pod_swarmer_burst_min = Some(parse_usize(value)),
            "pod_swarmer_burst_range" => pod_swarmer_burst_range = Some(parse_usize(value)),
            "max_swarmers" => max_swarmers = Some(parse_usize(value)),
            "bomber_mine_drop_delay" => bomber_mine_drop_delay = Some(parse_u32(value)),
            "max_mines" => max_mines = Some(parse_usize(value)),
            "attack_wave_group_size" => attack_wave_group_size = Some(parse_u8(value)),
            "attack_wave_total_openers" => attack_wave_total_openers = Some(parse_u8(value)),
            "attack_wave_reinforcement_delay" => {
                attack_wave_reinforcement_delay = Some(parse_u32(value))
            }
            "default_human_world_xs" => default_human_world_xs = Some(parse_i32_list(value)),
            _ => panic!("unknown arcade rule key {key}"),
        }
    }

    ArcadeTables {
        safe_fall_height: safe_fall_height.expect("safe_fall_height should be defined"),
        safe_fall_score: safe_fall_score.expect("safe_fall_score should be defined"),
        human_catch_score: human_catch_score.expect("human_catch_score should be defined"),
        human_landing_score: human_landing_score.expect("human_landing_score should be defined"),
        hazard_collision_score: hazard_collision_score
            .expect("hazard_collision_score should be defined"),
        bonus_stock_score: bonus_stock_score.expect("bonus_stock_score should be defined"),
        max_wave_humanoid_bonus: max_wave_humanoid_bonus
            .expect("max_wave_humanoid_bonus should be defined"),
        player_max_speed: player_max_speed.expect("player_max_speed should be defined"),
        swarmer_speed: swarmer_speed.expect("swarmer_speed should be defined"),
        baiter_speed: baiter_speed.expect("baiter_speed should be defined"),
        bomber_base_speed: bomber_base_speed.expect("bomber_base_speed should be defined"),
        bomber_evasive_speed: bomber_evasive_speed.expect("bomber_evasive_speed should be defined"),
        max_baiters: max_baiters.expect("max_baiters should be defined"),
        baiter_base_delay: baiter_base_delay.expect("baiter_base_delay should be defined"),
        baiter_repeat_delay: baiter_repeat_delay.expect("baiter_repeat_delay should be defined"),
        pod_swarmer_burst_min: pod_swarmer_burst_min
            .expect("pod_swarmer_burst_min should be defined"),
        pod_swarmer_burst_range: pod_swarmer_burst_range
            .expect("pod_swarmer_burst_range should be defined"),
        max_swarmers: max_swarmers.expect("max_swarmers should be defined"),
        bomber_mine_drop_delay: bomber_mine_drop_delay
            .expect("bomber_mine_drop_delay should be defined"),
        max_mines: max_mines.expect("max_mines should be defined"),
        attack_wave_group_size: attack_wave_group_size
            .expect("attack_wave_group_size should be defined"),
        attack_wave_total_openers: attack_wave_total_openers
            .expect("attack_wave_total_openers should be defined"),
        attack_wave_reinforcement_delay: attack_wave_reinforcement_delay
            .expect("attack_wave_reinforcement_delay should be defined"),
        default_human_world_xs: default_human_world_xs
            .expect("default_human_world_xs should be defined"),
    }
}

fn parse_i32(value: &str) -> i32 {
    value.parse().expect("expected signed integer arcade value")
}

fn parse_u16(value: &str) -> u16 {
    value.parse().expect("expected u16 arcade value")
}

fn parse_u32(value: &str) -> u32 {
    value.parse().expect("expected u32 arcade value")
}

fn parse_u8(value: &str) -> u8 {
    value.parse().expect("expected u8 arcade value")
}

fn parse_usize(value: &str) -> usize {
    value.parse().expect("expected usize arcade value")
}

fn parse_i32_list(value: &str) -> Vec<i32> {
    value.split(',').map(parse_i32).collect()
}

#[cfg(test)]
mod tests {
    use super::arcade_tables;

    #[test]
    fn defender_arcade_tables_match_expected_defaults() {
        let tables = arcade_tables();

        assert_eq!(tables.safe_fall_height, 2);
        assert_eq!(tables.attack_wave_total_openers, 15);
        assert_eq!(tables.attack_wave_group_size, 5);
        assert_eq!(tables.default_human_world_xs.len(), 10);
        assert_eq!(tables.default_human_world_xs[0], 8);
        assert_eq!(tables.default_human_world_xs[9], 170);
    }
}
