#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArcadeWaveProfile {
    pub landers: u8,
    pub bombers: u8,
    pub pods: u8,
    pub mutants: u8,
    pub swarmers: u8,
    pub wave_size: u8,
    pub lander_x_velocity: u8,
    pub lander_y_velocity_msb: u8,
    pub lander_y_velocity_lsb: u8,
    pub bomber_x_velocity: u8,
    pub swarmer_x_velocity: u8,
    pub swarmer_shot_time: u32,
    pub swarmer_acceleration_mask: u8,
    pub baiter_delay: u32,
    pub baiter_shot_time: u32,
    pub baiter_seek_probability: u8,
    pub lander_shot_time: u32,
    pub mutant_random_y: u8,
    pub mutant_y_velocity_msb: u8,
    pub mutant_y_velocity_lsb: u8,
    pub mutant_x_velocity: u8,
    pub mutant_shot_time: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanderArcadeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub animation_frame: SpriteFrameIndex,
    pub target_human_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BomberArcadeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub animation_frame: SpriteFrameIndex,
    pub cruise_altitude: i16,
    pub sleep_ticks: u8,
    pub slot: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PodArcadeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmerArcadeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub acceleration: u8,
    pub sleep_ticks: u8,
    pub shot_timer: u8,
    pub horizontal_seek_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaiterArcadeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub animation_frame: SpriteFrameIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutantArcadeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub hop_rng: ActorArcadeRngSnapshot,
    pub render_x_correction: u16,
    pub dive_entry_shot_deferred: bool,
}

impl MutantArcadeState {
    fn from_lander_conversion(
        lander_runtime: LanderArcadeState,
        profile: ArcadeWaveProfile,
        hop_rng: ActorArcadeRngSnapshot,
    ) -> Self {
        Self {
            x_fraction: lander_runtime.x_fraction,
            y_fraction: lander_runtime.y_fraction,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8,
            sleep_ticks: 0,
            hop_rng,
            render_x_correction: mutant_dive_arcade_conversion_x_correction(lander_runtime)
                .unwrap_or(0),
            dive_entry_shot_deferred: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyProjectileArcadeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub lifetime_ticks: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorLanderSpawn {
    pub position: Point,
    pub arcade_state: Option<LanderArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBomberSpawn {
    pub position: Point,
    pub arcade_state: Option<BomberArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorPodSpawn {
    pub position: Point,
    pub arcade_state: Option<PodArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSwarmerSpawn {
    pub position: Point,
    pub arcade_state: Option<SwarmerArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBaiterSpawn {
    pub position: Point,
    pub arcade_state: Option<BaiterArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorMutantSpawn {
    pub position: Point,
    pub arcade_state: Option<MutantArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HumanArcadeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub animation_frame: SpriteFrameIndex,
    pub target_slot_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorHumanSpawn {
    pub position: Point,
    pub mode: HumanMode,
    pub arcade_state: Option<HumanArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FirstWaveLanderSpawnRecord {
    world_x: u16,
    world_y: u16,
    x_velocity: u16,
    y_velocity: u16,
    shot_timer: u8,
    sleep_ticks: u8,
    animation_frame: SpriteFrameIndex,
    target_human_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FirstWaveHumanSpawnRecord {
    world_x: u16,
    world_y: u16,
    animation_frame: SpriteFrameIndex,
}
