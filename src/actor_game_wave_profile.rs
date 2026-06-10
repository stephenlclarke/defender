const ADC8_RESULT_MASK: u16 = 0x00FF;
const ADC8_CARRY_THRESHOLD: u16 = 0x00FF;
const ACTOR_U8_SIGN_BIT: u8 = 0x80;
const ACTOR_POSITIVE_SIGN_EXTENSION: u8 = 0x00;
const ACTOR_NEGATIVE_SIGN_EXTENSION: u8 = 0xFF;

impl ActorWaveTuning {
    pub fn for_wave(wave: u16) -> Self {
        let wave = u8::try_from(wave.min(u16::from(u8::MAX))).unwrap_or(u8::MAX);
        Self {
            landers: actor_wave_tuning_u8(crate::reference_assets::WaveMetric::Landers, wave),
            bombers: actor_wave_tuning_u8(crate::reference_assets::WaveMetric::Bombers, wave),
            pods: actor_wave_tuning_u8(crate::reference_assets::WaveMetric::Pods, wave),
            mutants: actor_wave_tuning_u8(crate::reference_assets::WaveMetric::Mutants, wave),
            swarmers: actor_wave_tuning_u8(crate::reference_assets::WaveMetric::Swarmers, wave),
            wave_size: actor_wave_tuning_u8(crate::reference_assets::WaveMetric::WaveSize, wave),
            lander_x_velocity: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::LanderXVelocity,
                wave,
            ),
            lander_y_velocity_msb: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::LanderYVelocityHigh,
                wave,
            ),
            lander_y_velocity_lsb: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::LanderYVelocityLow,
                wave,
            ),
            bomber_x_velocity: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::BomberXVelocity,
                wave,
            ),
            swarmer_x_velocity: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::SwarmerXVelocity,
                wave,
            ),
            swarmer_shot_time: actor_wave_tuning_u32(
                crate::reference_assets::WaveMetric::SwarmerShotTime,
                wave,
            ),
            swarmer_acceleration_mask: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::SwarmerAccelerationMask,
                wave,
            ),
            baiter_delay: actor_wave_tuning_u32(
                crate::reference_assets::WaveMetric::BaiterDelay,
                wave,
            ),
            baiter_shot_time: actor_wave_tuning_u32(
                crate::reference_assets::WaveMetric::BaiterShotTime,
                wave,
            ),
            baiter_seek_probability: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::BaiterSeekProbability,
                wave,
            ),
            lander_shot_time: actor_wave_tuning_u32(
                crate::reference_assets::WaveMetric::LanderShotTime,
                wave,
            ),
            mutant_random_y: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::MutantRandomY,
                wave,
            ),
            mutant_y_velocity_msb: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::MutantYVelocityHigh,
                wave,
            ),
            mutant_y_velocity_lsb: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::MutantYVelocityLow,
                wave,
            ),
            mutant_x_velocity: actor_wave_tuning_u8(
                crate::reference_assets::WaveMetric::MutantXVelocity,
                wave,
            ),
            mutant_shot_time: actor_wave_tuning_u32(
                crate::reference_assets::WaveMetric::MutantShotTime,
                wave,
            ),
        }
    }

    fn lander_behavior(self) -> ActorBehaviorProfile {
        ActorBehaviorProfile {
            lander_seek_speed: lander_speed_from_velocity_byte(self.lander_x_velocity),
            lander_drift_speed: lander_speed_from_velocity_byte(self.lander_x_velocity),
            lander_fire_period_steps: u64::from(self.lander_shot_time.max(1)),
            ..ActorBehaviorProfile::default()
        }
    }

    fn lander_spawns(self, wave: u16, humans: &[ActorHumanSpawn]) -> Vec<ActorLanderSpawn> {
        let mut first_wave_lander_index = 0;
        let mut actor_rng = DEFAULT_RNG;
        let mut target_cursor = Some(0usize);
        self.active_family_slots()
            .into_iter()
            .filter_map(|slot| {
                if slot.kind != WaveEnemyKind::Lander {
                    return None;
                }
                let spawn = if wave == 1 {
                    ACTOR_FIRST_WAVE_LANDER_SPAWNS
                        .get(first_wave_lander_index)
                        .copied()
                        .unwrap_or_else(|| ActorLanderSpawn::new(slot.position))
                } else {
                    ActorLanderSpawn::from_wave_restore(
                        &mut actor_rng,
                        self,
                        select_next_lander_target_slot_index(&mut target_cursor, humans),
                    )
                };
                first_wave_lander_index += 1;
                Some(spawn)
            })
            .collect()
    }

    fn human_spawns(self, wave: u16) -> Vec<ActorHumanSpawn> {
        if wave == 1 {
            ACTOR_FIRST_WAVE_HUMAN_SPAWNS.to_vec()
        } else {
            initial_target_list_humans()
        }
    }

    fn bomber_spawns(self) -> Vec<ActorBomberSpawn> {
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == WaveEnemyKind::Bomber)
            .map(|slot| {
                ActorBomberSpawn::from_wave_slot(slot.position, self.bomber_x_velocity, slot.index)
            })
            .collect()
    }

    fn pod_spawns(self) -> Vec<ActorPodSpawn> {
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == WaveEnemyKind::Pod)
            .map(|slot| ActorPodSpawn::from_wave_slot(slot.position, slot.index))
            .collect()
    }

    fn mutant_spawns(self) -> Vec<ActorMutantSpawn> {
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == WaveEnemyKind::Mutant)
            .map(|slot| ActorMutantSpawn::from_wave_slot(slot.position, self, slot.index))
            .collect()
    }

    fn swarmer_spawns(self) -> Vec<ActorSwarmerSpawn> {
        let mut actor_rng = DEFAULT_RNG;
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == WaveEnemyKind::Swarmer)
            .map(|slot| ActorSwarmerSpawn::from_pod_release(&mut actor_rng, self, slot.position))
            .collect()
    }

    fn enemy_reserve_after_active_batch(self) -> EnemyReserveSnapshot {
        let mut reserve = EnemyReserveSnapshot {
            landers: self.landers,
            bombers: self.bombers,
            pods: self.pods,
            mutants: self.mutants,
            swarmers: self.swarmers,
        };
        for slot in self.active_family_slots() {
            actor_enemy_reserve_take(&mut reserve, slot.kind);
        }
        reserve
    }

    fn active_family_slots(self) -> Vec<WaveEnemySlot> {
        let mut counts = WaveEnemyCounts {
            landers: self.landers,
            bombers: self.bombers,
            pods: self.pods,
            mutants: self.mutants,
            swarmers: self.swarmers,
        };
        let target = usize::from(self.wave_size)
            .min(MAX_ACTIVE_WAVE_ENEMIES)
            .min(usize::from(counts.total()));
        let mut kinds = Vec::with_capacity(target);

        for kind in [
            WaveEnemyKind::Lander,
            WaveEnemyKind::Bomber,
            WaveEnemyKind::Pod,
            WaveEnemyKind::Mutant,
            WaveEnemyKind::Swarmer,
        ] {
            push_wave_enemy_kind(&mut kinds, &mut counts, target, kind);
        }
        for kind in [
            WaveEnemyKind::Lander,
            WaveEnemyKind::Bomber,
            WaveEnemyKind::Pod,
            WaveEnemyKind::Mutant,
            WaveEnemyKind::Swarmer,
        ] {
            while kinds.len() < target && counts.take(kind) {
                kinds.push(kind);
            }
        }

        kinds
            .into_iter()
            .enumerate()
            .map(|(index, kind)| WaveEnemySlot {
                kind,
                index,
                position: ACTOR_WAVE_ACTIVE_SPAWN_SLOTS[index],
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WaveEnemySlot {
    kind: WaveEnemyKind,
    index: usize,
    position: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaveEnemyKind {
    Lander,
    Bomber,
    Pod,
    Mutant,
    Swarmer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WaveEnemyCounts {
    landers: u8,
    bombers: u8,
    pods: u8,
    mutants: u8,
    swarmers: u8,
}

impl WaveEnemyCounts {
    const fn total(self) -> u8 {
        self.landers
            .saturating_add(self.bombers)
            .saturating_add(self.pods)
            .saturating_add(self.mutants)
            .saturating_add(self.swarmers)
    }

    fn take(&mut self, kind: WaveEnemyKind) -> bool {
        let count = match kind {
            WaveEnemyKind::Lander => &mut self.landers,
            WaveEnemyKind::Bomber => &mut self.bombers,
            WaveEnemyKind::Pod => &mut self.pods,
            WaveEnemyKind::Mutant => &mut self.mutants,
            WaveEnemyKind::Swarmer => &mut self.swarmers,
        };
        if *count == 0 {
            return false;
        }
        *count = count.saturating_sub(1);
        true
    }
}

fn push_wave_enemy_kind(
    kinds: &mut Vec<WaveEnemyKind>,
    counts: &mut WaveEnemyCounts,
    target: usize,
    kind: WaveEnemyKind,
) {
    if kinds.len() < target && counts.take(kind) {
        kinds.push(kind);
    }
}

fn actor_enemy_reserve_total(reserve: EnemyReserveSnapshot) -> u8 {
    reserve
        .landers
        .saturating_add(reserve.bombers)
        .saturating_add(reserve.pods)
        .saturating_add(reserve.mutants)
        .saturating_add(reserve.swarmers)
}

fn actor_enemy_reserve_is_empty(reserve: EnemyReserveSnapshot) -> bool {
    actor_enemy_reserve_total(reserve) == 0
}

fn actor_enemy_reserve_take(
    reserve: &mut EnemyReserveSnapshot,
    kind: WaveEnemyKind,
) -> bool {
    let count = match kind {
        WaveEnemyKind::Lander => &mut reserve.landers,
        WaveEnemyKind::Bomber => &mut reserve.bombers,
        WaveEnemyKind::Pod => &mut reserve.pods,
        WaveEnemyKind::Mutant => &mut reserve.mutants,
        WaveEnemyKind::Swarmer => &mut reserve.swarmers,
    };
    if *count == 0 {
        return false;
    }
    *count = count.saturating_sub(1);
    true
}

fn reserve_wave_enemy_kinds(
    reserve: &mut EnemyReserveSnapshot,
    profile: ActorWaveTuning,
) -> Vec<WaveEnemyKind> {
    if reserve.landers > 0 {
        let target = MAX_ACTIVE_WAVE_ENEMIES.min(usize::from(reserve.landers));
        let mut kinds = Vec::with_capacity(target);
        while kinds.len() < target
            && actor_enemy_reserve_take(reserve, WaveEnemyKind::Lander)
        {
            kinds.push(WaveEnemyKind::Lander);
        }
        return kinds;
    }

    let target = usize::from(profile.wave_size)
        .min(MAX_ACTIVE_WAVE_ENEMIES)
        .min(usize::from(actor_enemy_reserve_total(*reserve)));
    let mut kinds = Vec::with_capacity(target);

    for kind in [
        WaveEnemyKind::Lander,
        WaveEnemyKind::Bomber,
        WaveEnemyKind::Pod,
        WaveEnemyKind::Mutant,
        WaveEnemyKind::Swarmer,
    ] {
        push_actor_reserve_kind(&mut kinds, reserve, target, kind);
    }

    for kind in [
        WaveEnemyKind::Lander,
        WaveEnemyKind::Bomber,
        WaveEnemyKind::Pod,
        WaveEnemyKind::Mutant,
        WaveEnemyKind::Swarmer,
    ] {
        while kinds.len() < target && actor_enemy_reserve_take(reserve, kind) {
            kinds.push(kind);
        }
    }

    kinds
}

fn push_actor_reserve_kind(
    kinds: &mut Vec<WaveEnemyKind>,
    reserve: &mut EnemyReserveSnapshot,
    target: usize,
    kind: WaveEnemyKind,
) {
    if kinds.len() < target && actor_enemy_reserve_take(reserve, kind) {
        kinds.push(kind);
    }
}

fn lander_speed_from_velocity_byte(velocity: u8) -> i16 {
    i16::from((velocity / 16).max(1))
}

fn speed_pixels_from_velocity_byte(velocity: u8) -> i16 {
    i16::from((velocity / 32).max(1))
}

fn adc8(lhs: u8, rhs: u8, carry: bool) -> (u8, bool) {
    let sum = u16::from(lhs) + u16::from(rhs) + u16::from(u8::from(carry));
    ((sum & ADC8_RESULT_MASK) as u8, sum > ADC8_CARRY_THRESHOLD)
}

fn bounded_actor_rng_value(max: u8, mut seed: u8) -> u8 {
    while seed > max {
        seed >>= 1;
    }
    seed.wrapping_add(1)
}

const fn actor_sign_extend_u8_to_u16(value: u8) -> u16 {
    let sign = if value & ACTOR_U8_SIGN_BIT == 0 {
        ACTOR_POSITIVE_SIGN_EXTENSION
    } else {
        ACTOR_NEGATIVE_SIGN_EXTENSION
    };
    u16::from_be_bytes([sign, value])
}

fn actor_wave_tuning_u8(metric: crate::reference_assets::WaveMetric, wave: u8) -> u8 {
    u8::try_from(actor_wave_tuning_value(metric, wave))
        .unwrap_or_else(|_| panic!("actor wave metric should fit u8"))
}

fn actor_wave_tuning_u32(metric: crate::reference_assets::WaveMetric, wave: u8) -> u32 {
    u32::try_from(actor_wave_tuning_value(metric, wave))
        .unwrap_or_else(|_| panic!("actor wave metric should be non-negative"))
}

fn actor_wave_tuning_value(metric: crate::reference_assets::WaveMetric, wave: u8) -> i32 {
    crate::reference_assets::wave_metric_value(
        metric,
        wave,
        actor_wave_inter_delta_iterations(wave.max(1)),
    )
}

fn actor_wave_inter_delta_iterations(wave: u8) -> u16 {
    let wave_delta = wave.saturating_sub(4);
    let pre_ceiling = ACTOR_DEFAULT_DIFFICULTY_INITIAL.saturating_add(wave_delta);
    u16::from(pre_ceiling.min(ACTOR_DEFAULT_DIFFICULTY_CEILING))
}
