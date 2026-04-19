//! Holds the compiled Defender gameplay defaults used by the native Rust implementation.
//!
//! These values remain compiled into the runtime so the default game path no longer parses
//! editable text or merges `~/.xyzzy/defender/arcade-rules.txt` overrides into cabinet
//! behavior. The red-label `WVTAB` records now live separately in `red_label_wave.rs`.

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
    pub player_shot_limit: usize,
    pub player_shot_speed: i32,
    pub enemy_shot_limit: usize,
    pub swarmer_fire_delay: u32,
    pub swarmer_fire_lead_divisor: u32,
    pub swarmer_speed: i32,
    pub baiter_speed: i32,
    pub max_baiters: usize,
    pub baiter_repeat_delay: u32,
    pub pod_swarmer_burst_min: usize,
    pub pod_swarmer_burst_range: usize,
    pub max_swarmers: usize,
    pub bomber_mine_drop_delay: u32,
    pub max_mines: usize,
    pub default_human_world_xs: &'static [i32; 10],
}

pub fn arcade_tables() -> &'static ArcadeTables {
    &ARCADE_TABLES
}

const DEFAULT_HUMAN_WORLD_XS: [i32; 10] = [8, 26, 44, 62, 80, 98, 116, 134, 152, 170];

static ARCADE_TABLES: ArcadeTables = ArcadeTables {
    safe_fall_height: 2,
    safe_fall_score: 250,
    human_catch_score: 500,
    human_landing_score: 500,
    hazard_collision_score: 25,
    bonus_stock_score: 10_000,
    max_wave_humanoid_bonus: 500,
    player_max_speed: 1,
    player_shot_limit: 4,
    player_shot_speed: 2,
    enemy_shot_limit: 6,
    swarmer_fire_delay: 3,
    swarmer_fire_lead_divisor: 4,
    swarmer_speed: 2,
    baiter_speed: 2,
    max_baiters: 4,
    baiter_repeat_delay: 20,
    pod_swarmer_burst_min: 5,
    pod_swarmer_burst_range: 3,
    max_swarmers: 20,
    bomber_mine_drop_delay: 3,
    max_mines: 24,
    default_human_world_xs: &DEFAULT_HUMAN_WORLD_XS,
};

#[cfg(test)]
mod tests {
    use super::arcade_tables;

    #[test]
    fn defender_arcade_tables_match_expected_defaults() {
        let tables = arcade_tables();

        assert_eq!(tables.safe_fall_height, 2);
        assert_eq!(tables.player_shot_limit, 4);
        assert_eq!(tables.player_shot_speed, 2);
        assert_eq!(tables.enemy_shot_limit, 6);
        assert_eq!(tables.swarmer_fire_delay, 3);
        assert_eq!(tables.default_human_world_xs.len(), 10);
        assert_eq!(tables.default_human_world_xs[0], 8);
        assert_eq!(tables.default_human_world_xs[9], 170);
    }
}
