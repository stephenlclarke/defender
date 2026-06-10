#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorWaveTuning {
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ActorRuntimeState {
    #[default]
    None,
    Lander(LanderRuntimeState),
    Bomber(BomberRuntimeState),
    Pod(PodRuntimeState),
    Swarmer(SwarmerRuntimeState),
    Baiter(BaiterRuntimeState),
    Mutant(MutantRuntimeState),
    Human(HumanRuntimeState),
    EnemyProjectile(EnemyProjectileRuntimeState),
}

impl ActorRuntimeState {
    pub(crate) const NONE: Self = Self::None;

    pub(crate) const fn lander(runtime_state: Option<LanderRuntimeState>) -> Self {
        match runtime_state {
            Some(runtime_state) => Self::Lander(runtime_state),
            None => Self::None,
        }
    }

    pub(crate) const fn bomber(runtime_state: Option<BomberRuntimeState>) -> Self {
        match runtime_state {
            Some(runtime_state) => Self::Bomber(runtime_state),
            None => Self::None,
        }
    }

    pub(crate) const fn pod(runtime_state: Option<PodRuntimeState>) -> Self {
        match runtime_state {
            Some(runtime_state) => Self::Pod(runtime_state),
            None => Self::None,
        }
    }

    pub(crate) const fn swarmer(runtime_state: Option<SwarmerRuntimeState>) -> Self {
        match runtime_state {
            Some(runtime_state) => Self::Swarmer(runtime_state),
            None => Self::None,
        }
    }

    pub(crate) const fn baiter(runtime_state: Option<BaiterRuntimeState>) -> Self {
        match runtime_state {
            Some(runtime_state) => Self::Baiter(runtime_state),
            None => Self::None,
        }
    }

    pub(crate) const fn mutant(runtime_state: Option<MutantRuntimeState>) -> Self {
        match runtime_state {
            Some(runtime_state) => Self::Mutant(runtime_state),
            None => Self::None,
        }
    }

    pub(crate) const fn human(runtime_state: Option<HumanRuntimeState>) -> Self {
        match runtime_state {
            Some(runtime_state) => Self::Human(runtime_state),
            None => Self::None,
        }
    }

    pub(crate) const fn enemy_projectile(runtime_state: Option<EnemyProjectileRuntimeState>) -> Self {
        match runtime_state {
            Some(runtime_state) => Self::EnemyProjectile(runtime_state),
            None => Self::None,
        }
    }

    pub(crate) const fn as_lander(self) -> Option<LanderRuntimeState> {
        match self {
            Self::Lander(runtime_state) => Some(runtime_state),
            _ => None,
        }
    }

    pub(crate) const fn as_bomber(self) -> Option<BomberRuntimeState> {
        match self {
            Self::Bomber(runtime_state) => Some(runtime_state),
            _ => None,
        }
    }

    pub(crate) const fn as_pod(self) -> Option<PodRuntimeState> {
        match self {
            Self::Pod(runtime_state) => Some(runtime_state),
            _ => None,
        }
    }

    pub(crate) const fn as_swarmer(self) -> Option<SwarmerRuntimeState> {
        match self {
            Self::Swarmer(runtime_state) => Some(runtime_state),
            _ => None,
        }
    }

    pub(crate) const fn as_baiter(self) -> Option<BaiterRuntimeState> {
        match self {
            Self::Baiter(runtime_state) => Some(runtime_state),
            _ => None,
        }
    }

    pub(crate) const fn as_mutant(self) -> Option<MutantRuntimeState> {
        match self {
            Self::Mutant(runtime_state) => Some(runtime_state),
            _ => None,
        }
    }

    pub(crate) const fn as_human(self) -> Option<HumanRuntimeState> {
        match self {
            Self::Human(runtime_state) => Some(runtime_state),
            _ => None,
        }
    }

    pub(crate) const fn as_enemy_projectile(self) -> Option<EnemyProjectileRuntimeState> {
        match self {
            Self::EnemyProjectile(runtime_state) => Some(runtime_state),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanderRuntimeState {
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
pub struct BomberRuntimeState {
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
pub struct PodRuntimeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmerRuntimeState {
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
pub struct BaiterRuntimeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub animation_frame: SpriteFrameIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutantRuntimeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub hop_rng: ActorRngSnapshot,
    pub render_x_correction: u16,
    pub dive_entry_shot_deferred: bool,
}

impl MutantRuntimeState {
    fn from_lander_conversion(
        lander_runtime: LanderRuntimeState,
        profile: ActorWaveTuning,
        hop_rng: ActorRngSnapshot,
    ) -> Self {
        Self {
            x_fraction: lander_runtime.x_fraction,
            y_fraction: lander_runtime.y_fraction,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8,
            sleep_ticks: 0,
            hop_rng,
            render_x_correction: mutant_dive_conversion_x_correction(lander_runtime)
                .unwrap_or(0),
            dive_entry_shot_deferred: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyProjectileRuntimeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub lifetime_ticks: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorLanderSpawn {
    pub position: Point,
    pub runtime_state: Option<LanderRuntimeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBomberSpawn {
    pub position: Point,
    pub runtime_state: Option<BomberRuntimeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorPodSpawn {
    pub position: Point,
    pub runtime_state: Option<PodRuntimeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSwarmerSpawn {
    pub position: Point,
    pub runtime_state: Option<SwarmerRuntimeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBaiterSpawn {
    pub position: Point,
    pub runtime_state: Option<BaiterRuntimeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorMutantSpawn {
    pub position: Point,
    pub runtime_state: Option<MutantRuntimeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HumanRuntimeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub animation_frame: SpriteFrameIndex,
    pub target_slot_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorHumanSpawn {
    pub position: Point,
    pub mode: HumanMode,
    pub runtime_state: Option<HumanRuntimeState>,
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
