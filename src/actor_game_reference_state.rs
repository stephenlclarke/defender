#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorWaveTuning {
    pub landers: u8,
    pub bombers: u8,
    pub pods: u8,
    pub mutants: u8,
    pub swarmers: u8,
    pub wave_size: u8,
    pub lander_x_velocity: u8,
    pub lander_y_velocity: u16,
    pub bomber_x_velocity: u8,
    pub swarmer_x_velocity: u8,
    pub swarmer_shot_time: u32,
    pub swarmer_acceleration_mask: u8,
    pub baiter_delay: u32,
    pub baiter_shot_time: u32,
    pub baiter_seek_probability: u8,
    pub lander_shot_time: u32,
    pub mutant_random_y: u8,
    pub mutant_y_velocity: u16,
    pub mutant_x_velocity: u8,
    pub mutant_shot_time: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActorReferenceSubpixels {
    pub(crate) x: u8,
    pub(crate) y: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ActorReferenceState {
    #[default]
    None,
    Lander(LanderReferenceState),
    Bomber(BomberReferenceState),
    Pod(PodReferenceState),
    Swarmer(SwarmerReferenceState),
    Baiter(BaiterReferenceState),
    Mutant(MutantReferenceState),
    Human(HumanReferenceState),
    EnemyProjectile(EnemyProjectileReferenceState),
}

impl ActorReferenceState {
    pub(crate) const NONE: Self = Self::None;

    pub(crate) const fn lander(reference_state: Option<LanderReferenceState>) -> Self {
        match reference_state {
            Some(reference_state) => Self::Lander(reference_state),
            None => Self::None,
        }
    }

    pub(crate) const fn bomber(reference_state: Option<BomberReferenceState>) -> Self {
        match reference_state {
            Some(reference_state) => Self::Bomber(reference_state),
            None => Self::None,
        }
    }

    pub(crate) const fn pod(reference_state: Option<PodReferenceState>) -> Self {
        match reference_state {
            Some(reference_state) => Self::Pod(reference_state),
            None => Self::None,
        }
    }

    pub(crate) const fn swarmer(reference_state: Option<SwarmerReferenceState>) -> Self {
        match reference_state {
            Some(reference_state) => Self::Swarmer(reference_state),
            None => Self::None,
        }
    }

    pub(crate) const fn baiter(reference_state: Option<BaiterReferenceState>) -> Self {
        match reference_state {
            Some(reference_state) => Self::Baiter(reference_state),
            None => Self::None,
        }
    }

    pub(crate) const fn mutant(reference_state: Option<MutantReferenceState>) -> Self {
        match reference_state {
            Some(reference_state) => Self::Mutant(reference_state),
            None => Self::None,
        }
    }

    pub(crate) const fn human(reference_state: Option<HumanReferenceState>) -> Self {
        match reference_state {
            Some(reference_state) => Self::Human(reference_state),
            None => Self::None,
        }
    }

    pub(crate) const fn enemy_projectile(reference_state: Option<EnemyProjectileReferenceState>) -> Self {
        match reference_state {
            Some(reference_state) => Self::EnemyProjectile(reference_state),
            None => Self::None,
        }
    }

    pub(crate) const fn as_lander(self) -> Option<LanderReferenceState> {
        match self {
            Self::Lander(reference_state) => Some(reference_state),
            _ => None,
        }
    }

    pub(crate) const fn as_bomber(self) -> Option<BomberReferenceState> {
        match self {
            Self::Bomber(reference_state) => Some(reference_state),
            _ => None,
        }
    }

    pub(crate) const fn as_pod(self) -> Option<PodReferenceState> {
        match self {
            Self::Pod(reference_state) => Some(reference_state),
            _ => None,
        }
    }

    pub(crate) const fn as_swarmer(self) -> Option<SwarmerReferenceState> {
        match self {
            Self::Swarmer(reference_state) => Some(reference_state),
            _ => None,
        }
    }

    pub(crate) const fn as_baiter(self) -> Option<BaiterReferenceState> {
        match self {
            Self::Baiter(reference_state) => Some(reference_state),
            _ => None,
        }
    }

    pub(crate) const fn as_mutant(self) -> Option<MutantReferenceState> {
        match self {
            Self::Mutant(reference_state) => Some(reference_state),
            _ => None,
        }
    }

    pub(crate) const fn as_human(self) -> Option<HumanReferenceState> {
        match self {
            Self::Human(reference_state) => Some(reference_state),
            _ => None,
        }
    }

    pub(crate) const fn as_enemy_projectile(self) -> Option<EnemyProjectileReferenceState> {
        match self {
            Self::EnemyProjectile(reference_state) => Some(reference_state),
            _ => None,
        }
    }

    pub(crate) const fn subpixels(self) -> Option<ActorReferenceSubpixels> {
        match self {
            Self::Lander(reference_state) => Some(reference_state.subpixels()),
            Self::Bomber(reference_state) => Some(reference_state.subpixels()),
            Self::Pod(reference_state) => Some(reference_state.subpixels()),
            Self::Swarmer(reference_state) => Some(reference_state.subpixels()),
            Self::Baiter(reference_state) => Some(reference_state.subpixels()),
            Self::Mutant(reference_state) => Some(reference_state.subpixels()),
            Self::Human(reference_state) => Some(reference_state.subpixels()),
            Self::EnemyProjectile(reference_state) => Some(reference_state.subpixels()),
            Self::None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LanderReferenceState {
    pub(crate) x_fraction: u8,
    pub(crate) y_fraction: u8,
    pub(crate) x_velocity: u16,
    pub(crate) y_velocity: u16,
    pub(crate) shot_timer: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) animation_frame: SpriteFrameIndex,
    pub(crate) target_human_index: Option<usize>,
}

impl LanderReferenceState {
    const fn subpixels(self) -> ActorReferenceSubpixels {
        ActorReferenceSubpixels {
            x: self.x_fraction,
            y: self.y_fraction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BomberReferenceState {
    pub(crate) x_fraction: u8,
    pub(crate) y_fraction: u8,
    pub(crate) x_velocity: u16,
    pub(crate) y_velocity: u16,
    pub(crate) animation_frame: SpriteFrameIndex,
    pub(crate) cruise_altitude: i16,
    pub(crate) sleep_ticks: u8,
    pub(crate) slot: u8,
}

impl BomberReferenceState {
    const fn subpixels(self) -> ActorReferenceSubpixels {
        ActorReferenceSubpixels {
            x: self.x_fraction,
            y: self.y_fraction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PodReferenceState {
    pub(crate) x_fraction: u8,
    pub(crate) y_fraction: u8,
    pub(crate) x_velocity: u16,
    pub(crate) y_velocity: u16,
}

impl PodReferenceState {
    const fn subpixels(self) -> ActorReferenceSubpixels {
        ActorReferenceSubpixels {
            x: self.x_fraction,
            y: self.y_fraction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SwarmerReferenceState {
    pub(crate) x_fraction: u8,
    pub(crate) y_fraction: u8,
    pub(crate) x_velocity: u16,
    pub(crate) y_velocity: u16,
    pub(crate) acceleration: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) shot_timer: u8,
    pub(crate) horizontal_seek_pending: bool,
}

impl SwarmerReferenceState {
    const fn subpixels(self) -> ActorReferenceSubpixels {
        ActorReferenceSubpixels {
            x: self.x_fraction,
            y: self.y_fraction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BaiterReferenceState {
    pub(crate) x_fraction: u8,
    pub(crate) y_fraction: u8,
    pub(crate) x_velocity: u16,
    pub(crate) y_velocity: u16,
    pub(crate) shot_timer: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) animation_frame: SpriteFrameIndex,
}

impl BaiterReferenceState {
    const fn subpixels(self) -> ActorReferenceSubpixels {
        ActorReferenceSubpixels {
            x: self.x_fraction,
            y: self.y_fraction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MutantReferenceState {
    pub(crate) x_fraction: u8,
    pub(crate) y_fraction: u8,
    pub(crate) x_velocity: u16,
    pub(crate) y_velocity: u16,
    pub(crate) shot_timer: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) hop_rng: ActorRngSnapshot,
    pub(crate) render_x_correction: u16,
    pub(crate) dive_entry_shot_deferred: bool,
}

impl MutantReferenceState {
    fn from_lander_conversion(
        lander_reference_state: LanderReferenceState,
        profile: ActorWaveTuning,
        hop_rng: ActorRngSnapshot,
    ) -> Self {
        Self {
            x_fraction: lander_reference_state.x_fraction,
            y_fraction: lander_reference_state.y_fraction,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8,
            sleep_ticks: 0,
            hop_rng,
            render_x_correction: mutant_dive_conversion_x_correction(lander_reference_state)
                .unwrap_or(0),
            dive_entry_shot_deferred: false,
        }
    }

    const fn subpixels(self) -> ActorReferenceSubpixels {
        ActorReferenceSubpixels {
            x: self.x_fraction,
            y: self.y_fraction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyProjectileReferenceState {
    pub(crate) x_fraction: u8,
    pub(crate) y_fraction: u8,
    pub(crate) x_velocity: u16,
    pub(crate) y_velocity: u16,
    pub(crate) lifetime_ticks: u8,
}

impl EnemyProjectileReferenceState {
    const fn subpixels(self) -> ActorReferenceSubpixels {
        ActorReferenceSubpixels {
            x: self.x_fraction,
            y: self.y_fraction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorLanderSpawn {
    pub position: Point,
    reference_state: Option<LanderReferenceState>,
    spawn_visibility: LanderSpawnVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBomberSpawn {
    pub position: Point,
    reference_state: Option<BomberReferenceState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorPodSpawn {
    pub position: Point,
    reference_state: Option<PodReferenceState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSwarmerSpawn {
    pub position: Point,
    reference_state: Option<SwarmerReferenceState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBaiterSpawn {
    pub position: Point,
    reference_state: Option<BaiterReferenceState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorMutantSpawn {
    pub position: Point,
    reference_state: Option<MutantReferenceState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HumanReferenceState {
    pub(crate) x_fraction: u8,
    pub(crate) y_fraction: u8,
    pub(crate) animation_frame: SpriteFrameIndex,
    pub(crate) target_slot_index: usize,
}

impl HumanReferenceState {
    const fn subpixels(self) -> ActorReferenceSubpixels {
        ActorReferenceSubpixels {
            x: self.x_fraction,
            y: self.y_fraction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorHumanSpawn {
    pub position: Point,
    pub mode: HumanMode,
    reference_state: Option<HumanReferenceState>,
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
