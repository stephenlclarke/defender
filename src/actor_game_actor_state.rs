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
pub(crate) struct ActorSubpixels {
    x: u8,
    y: u8,
}

impl ActorSubpixels {
    pub(crate) const fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    pub(crate) const fn x(self) -> u8 {
        self.x
    }

    pub(crate) const fn y(self) -> u8 {
        self.y
    }
}

mod actor_motion {
    use super::{
        ActorSubpixels, Point, Velocity, screen_velocity_from_motion_words, step_motion_axis,
        step_wrapping_motion_y,
    };

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub(super) struct ActorMotion {
        x_fraction: u8,
        y_fraction: u8,
        x_velocity: u16,
        y_velocity: u16,
    }

    impl ActorMotion {
        pub(super) const fn new(
            x_fraction: u8,
            y_fraction: u8,
            x_velocity: u16,
            y_velocity: u16,
        ) -> Self {
            Self {
                x_fraction,
                y_fraction,
                x_velocity,
                y_velocity,
            }
        }

        pub(super) const fn stationary(x_fraction: u8, y_fraction: u8) -> Self {
            Self::new(x_fraction, y_fraction, 0, 0)
        }

        pub(super) const fn at_rest() -> Self {
            Self::new(0, 0, 0, 0)
        }

        pub(super) const fn from_world_words(
            world_x: u16,
            world_y: u16,
            x_velocity: u16,
            y_velocity: u16,
        ) -> (Point, Self) {
            let [x, x_fraction] = world_x.to_be_bytes();
            let [y, y_fraction] = world_y.to_be_bytes();
            (
                Point::new(x as i16, y as i16),
                Self::new(x_fraction, y_fraction, x_velocity, y_velocity),
            )
        }

        pub(super) const fn subpixels(self) -> ActorSubpixels {
            ActorSubpixels::new(self.x_fraction, self.y_fraction)
        }

        pub(super) const fn x_fraction(self) -> u8 {
            self.x_fraction
        }

        pub(super) const fn y_fraction(self) -> u8 {
            self.y_fraction
        }

        pub(super) const fn x_velocity(self) -> u16 {
            self.x_velocity
        }

        pub(super) const fn y_velocity(self) -> u16 {
            self.y_velocity
        }

        pub(super) const fn is_stationary(self) -> bool {
            self.x_velocity == 0 && self.y_velocity == 0
        }

        pub(super) fn set_x_velocity(&mut self, x_velocity: u16) {
            self.x_velocity = x_velocity;
        }

        pub(super) fn set_y_velocity(&mut self, y_velocity: u16) {
            self.y_velocity = y_velocity;
        }

        pub(super) fn set_velocity(&mut self, x_velocity: u16, y_velocity: u16) {
            self.x_velocity = x_velocity;
            self.y_velocity = y_velocity;
        }

        pub(super) fn set_subpixels(&mut self, x_fraction: u8, y_fraction: u8) {
            self.x_fraction = x_fraction;
            self.y_fraction = y_fraction;
        }

        pub(super) fn advance(&mut self, position: Point) -> Point {
            self.advance_with_x_velocity(position, self.x_velocity)
        }

        pub(super) fn advance_with_x_velocity(
            &mut self,
            position: Point,
            x_velocity: u16,
        ) -> Point {
            let (x, x_fraction) = step_motion_axis(position.x, self.x_fraction, x_velocity);
            let (y, y_fraction) =
                step_wrapping_motion_y(position.y, self.y_fraction, self.y_velocity);
            self.x_fraction = x_fraction;
            self.y_fraction = y_fraction;
            Point::new(x, y)
        }

        pub(super) fn screen_velocity(self) -> Velocity {
            screen_velocity_from_motion_words(self.x_velocity, self.y_velocity)
        }
    }
}

use actor_motion::ActorMotion;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ActorInternalState {
    #[default]
    None,
    Lander(LanderActorState),
    Bomber(BomberActorState),
    Pod(PodActorState),
    Swarmer(SwarmerActorState),
    Baiter(BaiterActorState),
    Mutant(MutantActorState),
    Human(HumanActorState),
    EnemyProjectile(EnemyProjectileActorState),
}

impl ActorInternalState {
    pub(crate) const NONE: Self = Self::None;

    pub(crate) const fn lander(actor_state: Option<LanderActorState>) -> Self {
        match actor_state {
            Some(actor_state) => Self::Lander(actor_state),
            None => Self::None,
        }
    }

    pub(crate) const fn bomber(actor_state: Option<BomberActorState>) -> Self {
        match actor_state {
            Some(actor_state) => Self::Bomber(actor_state),
            None => Self::None,
        }
    }

    pub(crate) const fn pod(actor_state: Option<PodActorState>) -> Self {
        match actor_state {
            Some(actor_state) => Self::Pod(actor_state),
            None => Self::None,
        }
    }

    pub(crate) const fn swarmer(actor_state: Option<SwarmerActorState>) -> Self {
        match actor_state {
            Some(actor_state) => Self::Swarmer(actor_state),
            None => Self::None,
        }
    }

    pub(crate) const fn baiter(actor_state: Option<BaiterActorState>) -> Self {
        match actor_state {
            Some(actor_state) => Self::Baiter(actor_state),
            None => Self::None,
        }
    }

    pub(crate) const fn mutant(actor_state: Option<MutantActorState>) -> Self {
        match actor_state {
            Some(actor_state) => Self::Mutant(actor_state),
            None => Self::None,
        }
    }

    pub(crate) const fn human(actor_state: Option<HumanActorState>) -> Self {
        match actor_state {
            Some(actor_state) => Self::Human(actor_state),
            None => Self::None,
        }
    }

    pub(crate) const fn enemy_projectile(actor_state: Option<EnemyProjectileActorState>) -> Self {
        match actor_state {
            Some(actor_state) => Self::EnemyProjectile(actor_state),
            None => Self::None,
        }
    }

    pub(crate) const fn as_lander(self) -> Option<LanderActorState> {
        match self {
            Self::Lander(actor_state) => Some(actor_state),
            _ => None,
        }
    }

    pub(crate) const fn as_bomber(self) -> Option<BomberActorState> {
        match self {
            Self::Bomber(actor_state) => Some(actor_state),
            _ => None,
        }
    }

    pub(crate) const fn as_pod(self) -> Option<PodActorState> {
        match self {
            Self::Pod(actor_state) => Some(actor_state),
            _ => None,
        }
    }

    pub(crate) const fn as_swarmer(self) -> Option<SwarmerActorState> {
        match self {
            Self::Swarmer(actor_state) => Some(actor_state),
            _ => None,
        }
    }

    pub(crate) const fn as_baiter(self) -> Option<BaiterActorState> {
        match self {
            Self::Baiter(actor_state) => Some(actor_state),
            _ => None,
        }
    }

    pub(crate) const fn as_mutant(self) -> Option<MutantActorState> {
        match self {
            Self::Mutant(actor_state) => Some(actor_state),
            _ => None,
        }
    }

    pub(crate) const fn as_human(self) -> Option<HumanActorState> {
        match self {
            Self::Human(actor_state) => Some(actor_state),
            _ => None,
        }
    }

    pub(crate) const fn as_enemy_projectile(self) -> Option<EnemyProjectileActorState> {
        match self {
            Self::EnemyProjectile(actor_state) => Some(actor_state),
            _ => None,
        }
    }

    pub(crate) const fn subpixels(self) -> Option<ActorSubpixels> {
        match self {
            Self::Lander(actor_state) => Some(actor_state.subpixels()),
            Self::Bomber(actor_state) => Some(actor_state.subpixels()),
            Self::Pod(actor_state) => Some(actor_state.subpixels()),
            Self::Swarmer(actor_state) => Some(actor_state.subpixels()),
            Self::Baiter(actor_state) => Some(actor_state.subpixels()),
            Self::Mutant(actor_state) => Some(actor_state.subpixels()),
            Self::Human(actor_state) => Some(actor_state.subpixels()),
            Self::EnemyProjectile(actor_state) => Some(actor_state.subpixels()),
            Self::None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LanderActorState {
    motion: ActorMotion,
    pub(crate) shot_timer: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) animation_frame: SpriteFrameIndex,
    pub(crate) target_human_index: Option<usize>,
}

impl LanderActorState {
    const fn new(
        motion: ActorMotion,
        shot_timer: u8,
        sleep_ticks: u8,
        animation_frame: SpriteFrameIndex,
        target_human_index: Option<usize>,
    ) -> Self {
        Self {
            motion,
            shot_timer,
            sleep_ticks,
            animation_frame,
            target_human_index,
        }
    }

    const fn subpixels(self) -> ActorSubpixels {
        self.motion.subpixels()
    }

    const fn x_fraction(self) -> u8 {
        self.motion.x_fraction()
    }

    const fn y_fraction(self) -> u8 {
        self.motion.y_fraction()
    }

    const fn x_velocity(self) -> u16 {
        self.motion.x_velocity()
    }

    const fn y_velocity(self) -> u16 {
        self.motion.y_velocity()
    }

    const fn is_stationary(self) -> bool {
        self.motion.is_stationary()
    }

    #[cfg(test)]
    fn set_x_velocity(&mut self, x_velocity: u16) {
        self.motion.set_x_velocity(x_velocity);
    }

    fn advance_motion(&mut self, position: Point) -> Point {
        self.motion.advance(position)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BomberActorState {
    motion: ActorMotion,
    pub(crate) animation_frame: SpriteFrameIndex,
    pub(crate) cruise_altitude: i16,
    pub(crate) sleep_ticks: u8,
    pub(crate) slot: u8,
}

impl BomberActorState {
    const fn new(
        motion: ActorMotion,
        animation_frame: SpriteFrameIndex,
        cruise_altitude: i16,
        sleep_ticks: u8,
        slot: u8,
    ) -> Self {
        Self {
            motion,
            animation_frame,
            cruise_altitude,
            sleep_ticks,
            slot,
        }
    }

    const fn subpixels(self) -> ActorSubpixels {
        self.motion.subpixels()
    }

    const fn x_fraction(self) -> u8 {
        self.motion.x_fraction()
    }

    const fn y_fraction(self) -> u8 {
        self.motion.y_fraction()
    }

    const fn x_velocity(self) -> u16 {
        self.motion.x_velocity()
    }

    const fn y_velocity(self) -> u16 {
        self.motion.y_velocity()
    }

    fn set_y_velocity(&mut self, y_velocity: u16) {
        self.motion.set_y_velocity(y_velocity);
    }

    fn advance_motion(&mut self, position: Point) -> Point {
        self.motion.advance(position)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PodActorState {
    motion: ActorMotion,
}

impl PodActorState {
    const fn new(motion: ActorMotion) -> Self {
        Self { motion }
    }

    const fn subpixels(self) -> ActorSubpixels {
        self.motion.subpixels()
    }

    const fn x_velocity(self) -> u16 {
        self.motion.x_velocity()
    }

    const fn x_fraction(self) -> u8 {
        self.motion.x_fraction()
    }

    const fn y_fraction(self) -> u8 {
        self.motion.y_fraction()
    }

    const fn y_velocity(self) -> u16 {
        self.motion.y_velocity()
    }

    #[cfg(test)]
    fn set_subpixels(&mut self, x_fraction: u8, y_fraction: u8) {
        self.motion.set_subpixels(x_fraction, y_fraction);
    }

    fn advance_motion(&mut self, position: Point) -> Point {
        self.motion.advance(position)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SwarmerActorState {
    motion: ActorMotion,
    pub(crate) acceleration: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) shot_timer: u8,
    pub(crate) horizontal_seek_pending: bool,
}

impl SwarmerActorState {
    const fn new(
        motion: ActorMotion,
        acceleration: u8,
        sleep_ticks: u8,
        shot_timer: u8,
        horizontal_seek_pending: bool,
    ) -> Self {
        Self {
            motion,
            acceleration,
            sleep_ticks,
            shot_timer,
            horizontal_seek_pending,
        }
    }

    const fn subpixels(self) -> ActorSubpixels {
        self.motion.subpixels()
    }

    const fn x_fraction(self) -> u8 {
        self.motion.x_fraction()
    }

    const fn y_fraction(self) -> u8 {
        self.motion.y_fraction()
    }

    const fn x_velocity(self) -> u16 {
        self.motion.x_velocity()
    }

    const fn y_velocity(self) -> u16 {
        self.motion.y_velocity()
    }

    fn set_subpixels(&mut self, x_fraction: u8, y_fraction: u8) {
        self.motion.set_subpixels(x_fraction, y_fraction);
    }

    fn set_x_velocity(&mut self, x_velocity: u16) {
        self.motion.set_x_velocity(x_velocity);
    }

    fn set_y_velocity(&mut self, y_velocity: u16) {
        self.motion.set_y_velocity(y_velocity);
    }

    fn advance_motion(&mut self, position: Point) -> Point {
        self.motion.advance(position)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BaiterActorState {
    motion: ActorMotion,
    pub(crate) shot_timer: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) animation_frame: SpriteFrameIndex,
}

impl BaiterActorState {
    const fn new(
        motion: ActorMotion,
        shot_timer: u8,
        sleep_ticks: u8,
        animation_frame: SpriteFrameIndex,
    ) -> Self {
        Self {
            motion,
            shot_timer,
            sleep_ticks,
            animation_frame,
        }
    }

    const fn subpixels(self) -> ActorSubpixels {
        self.motion.subpixels()
    }

    const fn x_fraction(self) -> u8 {
        self.motion.x_fraction()
    }

    const fn y_fraction(self) -> u8 {
        self.motion.y_fraction()
    }

    const fn x_velocity(self) -> u16 {
        self.motion.x_velocity()
    }

    const fn y_velocity(self) -> u16 {
        self.motion.y_velocity()
    }

    fn set_x_velocity(&mut self, x_velocity: u16) {
        self.motion.set_x_velocity(x_velocity);
    }

    fn set_y_velocity(&mut self, y_velocity: u16) {
        self.motion.set_y_velocity(y_velocity);
    }

    #[cfg(test)]
    fn set_subpixels(&mut self, x_fraction: u8, y_fraction: u8) {
        self.motion.set_subpixels(x_fraction, y_fraction);
    }

    fn advance_motion_with_x_velocity(&mut self, position: Point, x_velocity: u16) -> Point {
        self.motion.advance_with_x_velocity(position, x_velocity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MutantActorState {
    motion: ActorMotion,
    pub(crate) shot_timer: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) hop_rng: ActorRngSnapshot,
    pub(crate) render_x_correction: u16,
    pub(crate) dive_entry_shot_deferred: bool,
}

impl MutantActorState {
    fn from_lander_conversion(
        lander_actor_state: LanderActorState,
        profile: ActorWaveTuning,
        hop_rng: ActorRngSnapshot,
    ) -> Self {
        Self {
            motion: ActorMotion::stationary(
                lander_actor_state.x_fraction(),
                lander_actor_state.y_fraction(),
            ),
            shot_timer: profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8,
            sleep_ticks: 0,
            hop_rng,
            render_x_correction: mutant_dive_conversion_x_correction(lander_actor_state)
                .unwrap_or(0),
            dive_entry_shot_deferred: false,
        }
    }

    const fn subpixels(self) -> ActorSubpixels {
        self.motion.subpixels()
    }

    const fn x_fraction(self) -> u8 {
        self.motion.x_fraction()
    }

    const fn y_fraction(self) -> u8 {
        self.motion.y_fraction()
    }

    const fn x_velocity(self) -> u16 {
        self.motion.x_velocity()
    }

    const fn y_velocity(self) -> u16 {
        self.motion.y_velocity()
    }

    #[cfg(test)]
    fn set_subpixels(&mut self, x_fraction: u8, y_fraction: u8) {
        self.motion.set_subpixels(x_fraction, y_fraction);
    }

    fn set_velocity(&mut self, x_velocity: u16, y_velocity: u16) {
        self.motion.set_velocity(x_velocity, y_velocity);
    }

    fn advance_motion(&mut self, position: Point) -> Point {
        self.motion.advance(position)
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyProjectileActorState {
    motion: ActorMotion,
    pub(crate) lifetime_ticks: u8,
}

impl EnemyProjectileActorState {
    const fn new(motion: ActorMotion, lifetime_ticks: u8) -> Self {
        Self {
            motion,
            lifetime_ticks,
        }
    }

    fn from_velocity(
        x_fraction: u8,
        y_fraction: u8,
        velocity: Velocity,
        lifetime_ticks: u8,
    ) -> Self {
        Self::new(
            ActorMotion::new(
                x_fraction,
                y_fraction,
                projectile_velocity_word(velocity.dx),
                projectile_velocity_word(velocity.dy),
            ),
            lifetime_ticks,
        )
    }

    const fn subpixels(self) -> ActorSubpixels {
        self.motion.subpixels()
    }

    pub(crate) const fn x_fraction(self) -> u8 {
        self.motion.x_fraction()
    }

    pub(crate) const fn y_fraction(self) -> u8 {
        self.motion.y_fraction()
    }

    pub(crate) const fn x_velocity(self) -> u16 {
        self.motion.x_velocity()
    }

    pub(crate) const fn y_velocity(self) -> u16 {
        self.motion.y_velocity()
    }

    #[cfg(test)]
    fn set_velocity(&mut self, x_velocity: u16, y_velocity: u16) {
        self.motion.set_velocity(x_velocity, y_velocity);
    }

    fn set_subpixels(&mut self, x_fraction: u8, y_fraction: u8) {
        self.motion.set_subpixels(x_fraction, y_fraction);
    }

    fn advance_projectile_motion(&mut self, position: Point) -> Point {
        let (x, x_fraction) =
            step_projectile_axis(position.x, self.x_fraction(), self.x_velocity());
        let (y, y_fraction) =
            step_projectile_axis(position.y, self.y_fraction(), self.y_velocity());
        self.set_subpixels(x_fraction, y_fraction);
        Point::new(x, y)
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorLanderSpawn {
    pub position: Point,
    actor_state: Option<LanderActorState>,
    spawn_visibility: LanderSpawnVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBomberSpawn {
    pub position: Point,
    actor_state: Option<BomberActorState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorPodSpawn {
    pub position: Point,
    actor_state: Option<PodActorState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSwarmerSpawn {
    pub position: Point,
    actor_state: Option<SwarmerActorState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBaiterSpawn {
    pub position: Point,
    actor_state: Option<BaiterActorState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorMutantSpawn {
    pub position: Point,
    actor_state: Option<MutantActorState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HumanActorState {
    motion: ActorMotion,
    pub(crate) animation_frame: SpriteFrameIndex,
    pub(crate) target_slot_index: usize,
}

impl HumanActorState {
    const fn new(
        x_fraction: u8,
        y_fraction: u8,
        animation_frame: SpriteFrameIndex,
        target_slot_index: usize,
    ) -> Self {
        Self {
            motion: ActorMotion::stationary(x_fraction, y_fraction),
            animation_frame,
            target_slot_index,
        }
    }

    const fn subpixels(self) -> ActorSubpixels {
        self.motion.subpixels()
    }

    const fn x_fraction(self) -> u8 {
        self.motion.x_fraction()
    }

    const fn y_fraction(self) -> u8 {
        self.motion.y_fraction()
    }

    fn set_subpixels(&mut self, x_fraction: u8, y_fraction: u8) {
        self.motion.set_subpixels(x_fraction, y_fraction);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorHumanSpawn {
    pub position: Point,
    pub mode: HumanMode,
    actor_state: Option<HumanActorState>,
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
