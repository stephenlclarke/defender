const SPAWN_WORLD_FRACTION_MASK: u16 = 0x00FF;
const BOMBER_RESTORE_SPACING_WORD: u16 = 0x0180;
const WORLD_HALF_WRAP_OFFSET: u16 = 0x8000;
const POD_RESTORE_X_SEED_MASK: u8 = 0x3F;
const POD_RESTORE_X_BASE: u8 = 0x10;
const POD_RESTORE_X_VELOCITY_CENTER: u8 = 0x20;
const POD_RESTORE_Y_VELOCITY_MASK: u8 = 0x7F;
const POD_RESTORE_Y_VELOCITY_CENTER: u8 = 0x40;
const SPAWN_BYTE_SIGN_BIT: u8 = 0x80;
const POD_RESTORE_Y_VELOCITY_MAGNITUDE_BIT: u8 = 0x20;
const POD_RESTORE_Y_VELOCITY_CLEAR_MAGNITUDE_MASK: u8 = 0xDF;
const SWARMER_RELEASE_X_VELOCITY_MASK: u8 = 0x3F;
const SWARMER_RELEASE_X_VELOCITY_CENTER: u8 = 0x20;
const SWARMER_RELEASE_SLEEP_MASK: u8 = 0x1F;
const MINI_SWARMER_RESTORE_X_SEED_MASK: u8 = 0x3F;
const MINI_SWARMER_RESTORE_X_BASE: u8 = 0x80;
const HUMAN_RESTORE_QUADRANT_BANKS: [u8; 4] = [0x00, 0x40, 0x80, 0xC0];
const HUMAN_RESTORE_X_OFFSET_MASK: u8 = 0x1F;
const HUMAN_RESTORE_ANIMATION_SEED_BIT: u8 = 0x01;
const BAITER_RIGHT_SPAWN_PARITY_X_THRESHOLD: i16 = 128;
const BAITER_RIGHT_SPAWN_X: i16 = 228;
const BAITER_LEFT_SPAWN_X: i16 = 28;
const BAITER_VERTICAL_SPACING: i16 = 24;
const BAITER_VERTICAL_SPAWN_PHASES: usize = 3;
const BAITER_PLAYER_BOUNDS_TOP_CLEARANCE: i16 = 8;

impl ActorLanderSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            runtime_state: None,
        }
    }

    const fn from_first_wave_record(start: FirstWaveLanderSpawnRecord) -> Self {
        Self {
            position: Point::new((start.world_x >> 8) as i16, (start.world_y >> 8) as i16),
            runtime_state: Some(LanderRuntimeState {
                x_fraction: (start.world_x & SPAWN_WORLD_FRACTION_MASK) as u8,
                y_fraction: (start.world_y & SPAWN_WORLD_FRACTION_MASK) as u8,
                x_velocity: start.x_velocity,
                y_velocity: start.y_velocity,
                shot_timer: start.shot_timer,
                sleep_ticks: start.sleep_ticks,
                animation_frame: start.animation_frame,
                target_human_index: start.target_human_index,
            }),
        }
    }

    fn from_wave_restore(
        actor_rng: &mut ActorRng,
        profile: ActorWaveTuning,
        target_human_index: Option<usize>,
    ) -> Self {
        let placement_state = actor_rng.advance();
        let x = placement_state.hseed;
        let x_fraction = placement_state.lseed;
        let y = PLAYFIELD_TOP_EDGE_Y.wrapping_add(2);
        let y_velocity =
            u16::from_be_bytes([profile.lander_y_velocity_msb, profile.lander_y_velocity_lsb]);
        let shot_timer =
            actor_rng.advance_rmax(profile.lander_shot_time.min(u32::from(u8::MAX)) as u8);
        let x_velocity_byte = actor_rng.advance_rmax(profile.lander_x_velocity);
        let x_velocity = if x_velocity_byte & 1 == 0 {
            u16::from(x_velocity_byte)
        } else {
            !u16::from(x_velocity_byte)
        };

        Self {
            position: Point::new(i16::from(x), i16::from(y)),
            runtime_state: Some(LanderRuntimeState {
                x_fraction,
                y_fraction: 0,
                x_velocity,
                y_velocity,
                shot_timer,
                sleep_ticks: 0,
                animation_frame: SpriteFrameIndex::new(0),
                target_human_index,
            }),
        }
    }
}

impl ActorBomberSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            runtime_state: None,
        }
    }

    const fn from_wave_slot(position: Point, x_velocity_word: u8, spawn_index: usize) -> Self {
        let velocity_low = if spawn_index < 2 {
            0u8.wrapping_sub(x_velocity_word)
        } else {
            x_velocity_word
        };
        Self {
            position,
            runtime_state: Some(BomberRuntimeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: actor_sign_extend_u8_to_u16(velocity_low),
                y_velocity: 0,
                animation_frame: SpriteFrameIndex::new(0),
                cruise_altitude: BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                slot: (spawn_index % 4) as u8,
            }),
        }
    }

    fn wave_restore_batch(
        profile: ActorWaveTuning,
        player_absolute_x: u16,
        count: usize,
    ) -> Vec<Self> {
        let mut bombers = Vec::with_capacity(count);
        let mut remaining = count;
        let mut positive_x_velocity = true;

        while remaining > 0 {
            let squad_count = remaining.min(BOMBER_SQUAD_SIZE);
            let velocity_low = if positive_x_velocity {
                profile.bomber_x_velocity
            } else {
                0u8.wrapping_sub(profile.bomber_x_velocity)
            };
            positive_x_velocity = !positive_x_velocity;
            let x_velocity = actor_sign_extend_u8_to_u16(velocity_low);

            for squad_remaining in (1..=squad_count).rev() {
                let world_x_word = player_absolute_x
                    .wrapping_add((squad_remaining as u16).wrapping_mul(BOMBER_RESTORE_SPACING_WORD))
                    .wrapping_add(WORLD_HALF_WRAP_OFFSET);
                let [x, x_fraction] = world_x_word.to_be_bytes();
                bombers.push(Self {
                    position: Point::new(i16::from(x), BOMBER_CRUISE_ALTITUDE),
                    runtime_state: Some(BomberRuntimeState {
                        x_fraction,
                        y_fraction: 0,
                        x_velocity,
                        y_velocity: 0,
                        animation_frame: SpriteFrameIndex::new(0),
                        cruise_altitude: BOMBER_CRUISE_ALTITUDE,
                        sleep_ticks: 0,
                        slot: (squad_remaining - 1) as u8,
                    }),
                });
            }

            remaining -= squad_count;
        }

        bombers
    }
}

impl ActorPodSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            runtime_state: None,
        }
    }

    const fn from_wave_slot(position: Point, spawn_index: usize) -> Self {
        let velocity_low = if spawn_index < 2 {
            0u8.wrapping_sub(INITIAL_POD_X_SPEED)
        } else {
            INITIAL_POD_X_SPEED
        };
        Self {
            position,
            runtime_state: Some(PodRuntimeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: actor_sign_extend_u8_to_u16(velocity_low),
                y_velocity: 0,
            }),
        }
    }

    fn from_wave_restore(actor_rng: &mut ActorRng) -> Self {
        let state = actor_rng.advance();
        let [x, x_fraction] =
            u16::from_be_bytes([
                (state.hseed & POD_RESTORE_X_SEED_MASK).wrapping_add(POD_RESTORE_X_BASE),
                state.lseed,
            ])
            .to_be_bytes();
        let y = state
            .lseed
            .wrapping_shr(1)
            .wrapping_add(PLAYFIELD_TOP_EDGE_Y);
        let x_velocity = actor_sign_extend_u8_to_u16(
            (state.seed & POD_RESTORE_X_SEED_MASK).wrapping_sub(POD_RESTORE_X_VELOCITY_CENTER),
        );
        let mut y_velocity_low =
            (state.lseed & POD_RESTORE_Y_VELOCITY_MASK).wrapping_sub(POD_RESTORE_Y_VELOCITY_CENTER);
        if y_velocity_low & SPAWN_BYTE_SIGN_BIT == 0 {
            y_velocity_low |= POD_RESTORE_Y_VELOCITY_MAGNITUDE_BIT;
        } else {
            y_velocity_low &= POD_RESTORE_Y_VELOCITY_CLEAR_MAGNITUDE_MASK;
        }

        Self {
            position: Point::new(i16::from(x), i16::from(y)),
            runtime_state: Some(PodRuntimeState {
                x_fraction,
                y_fraction: 0,
                x_velocity,
                y_velocity: actor_sign_extend_u8_to_u16(y_velocity_low),
            }),
        }
    }
}

impl ActorSwarmerSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            runtime_state: None,
        }
    }

    fn from_pod_release(
        actor_rng: &mut ActorRng,
        profile: ActorWaveTuning,
        position: Point,
    ) -> Self {
        let velocity_rand = actor_rng.advance();
        let y_velocity = actor_sign_extend_u8_to_u16(velocity_rand.seed).wrapping_shl(1);
        let x_velocity = actor_sign_extend_u8_to_u16(
            (velocity_rand.lseed & SWARMER_RELEASE_X_VELOCITY_MASK)
                .wrapping_sub(SWARMER_RELEASE_X_VELOCITY_CENTER),
        );
        let acceleration = velocity_rand.lseed & profile.swarmer_acceleration_mask;
        let sleep_ticks = velocity_rand.hseed & SWARMER_RELEASE_SLEEP_MASK;
        let shot_timer =
            actor_rng.advance_rmax(profile.swarmer_shot_time.min(u32::from(u8::MAX)) as u8);

        Self {
            position,
            runtime_state: Some(SwarmerRuntimeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity,
                y_velocity,
                acceleration,
                sleep_ticks,
                shot_timer,
                horizontal_seek_pending: true,
            }),
        }
    }

    fn wave_restore_batch(
        actor_rng: &mut ActorRng,
        profile: ActorWaveTuning,
        count: usize,
    ) -> Vec<Self> {
        if count == 0 {
            return Vec::new();
        }

        let world_y_word = u16::from_be_bytes([
            actor_rng
                .seed
                .wrapping_shr(1)
                .wrapping_add(PLAYFIELD_TOP_EDGE_Y),
            0,
        ]);
        let placement_rand = actor_rng.advance();
        let world_x_word = u16::from_be_bytes([
            (placement_rand.seed & MINI_SWARMER_RESTORE_X_SEED_MASK)
                .wrapping_add(MINI_SWARMER_RESTORE_X_BASE),
            MINI_SWARMER_RESTORE_X_LOW,
        ]);
        let [x, x_fraction] = world_x_word.to_be_bytes();
        let [y, y_fraction] = world_y_word.to_be_bytes();
        let position = Point::new(i16::from(x), i16::from(y));

        (0..count)
            .map(|_| {
                let mut spawn = Self::from_pod_release(actor_rng, profile, position);
                if let Some(runtime_state) = &mut spawn.runtime_state {
                    runtime_state.x_fraction = x_fraction;
                    runtime_state.y_fraction = y_fraction;
                }
                spawn
            })
            .collect()
    }
}

impl ActorBaiterSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            runtime_state: None,
        }
    }

    fn from_player_position(
        profile: ActorWaveTuning,
        player_position: Point,
        active_baiters: usize,
    ) -> Self {
        let spawn_x = if (active_baiters
            + usize::from(player_position.x >= BAITER_RIGHT_SPAWN_PARITY_X_THRESHOLD))
        .is_multiple_of(2)
        {
            BAITER_RIGHT_SPAWN_X
        } else {
            BAITER_LEFT_SPAWN_X
        };
        let spawn_y = (player_position.y + BAITER_VERTICAL_SPACING
            - (i16::try_from(active_baiters % BAITER_VERTICAL_SPAWN_PHASES).unwrap_or(0)
                * BAITER_VERTICAL_SPACING))
        .clamp(
            PLAYER_BOUNDS.top + BAITER_PLAYER_BOUNDS_TOP_CLEARANCE,
            HUMAN_GROUND_Y - BAITER_VERTICAL_SPACING,
        );
        let position = Point::new(spawn_x, spawn_y);
        let mut runtime_state = BaiterRuntimeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: BAITER_INITIAL_SHOT_TIMER,
            sleep_ticks: 0,
            animation_frame: SpriteFrameIndex::new(0),
        };
        update_baiter_velocity(
            &mut runtime_state,
            position,
            profile,
            player_position,
            Velocity::default(),
            false,
            u8::MAX,
        );
        Self {
            position,
            runtime_state: Some(runtime_state),
        }
    }
}

impl ActorMutantSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            runtime_state: None,
        }
    }

    fn from_wave_slot(
        position: Point,
        profile: ActorWaveTuning,
        spawn_index: usize,
    ) -> Self {
        let mut actor_rng = DEFAULT_RNG;
        for _ in 0..=spawn_index {
            actor_rng.advance();
        }
        let shot_timer =
            actor_rng.advance_rmax(profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8);
        Self {
            position,
            runtime_state: Some(MutantRuntimeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer,
                sleep_ticks: 0,
                hop_rng: actor_rng.snapshot(),
                render_x_correction: 0,
                dive_entry_shot_deferred: false,
            }),
        }
    }

    fn from_wave_restore(
        actor_rng: &mut ActorRng,
        profile: ActorWaveTuning,
        background_absolute_x: u16,
    ) -> Self {
        let placement_state = actor_rng.advance();
        let avoid_left = background_absolute_x.wrapping_sub(MUTANT_RESTORE_AVOID_HALF_WIDTH);
        let mut relative = u16::from_be_bytes([placement_state.hseed, placement_state.lseed])
            .wrapping_sub(avoid_left);
        if relative < MUTANT_RESTORE_AVOID_WIDTH {
            relative = relative.wrapping_add(WORLD_HALF_WRAP_OFFSET);
        }
        let world_x_word = relative.wrapping_add(avoid_left);
        let [x, x_fraction] = world_x_word.to_be_bytes();
        let y = placement_state
            .seed
            .wrapping_shr(1)
            .wrapping_add(PLAYFIELD_TOP_EDGE_Y);
        let shot_timer =
            actor_rng.advance_rmax(profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8);

        Self {
            position: Point::new(i16::from(x), i16::from(y)),
            runtime_state: Some(MutantRuntimeState {
                x_fraction,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer,
                sleep_ticks: 0,
                hop_rng: actor_rng.snapshot(),
                render_x_correction: 0,
                dive_entry_shot_deferred: false,
            }),
        }
    }
}

fn initial_target_list_humans() -> Vec<ActorHumanSpawn> {
    let mut actor_rng = DEFAULT_RNG;
    restore_target_list_humans(&mut actor_rng, START_HUMAN_COUNT)
}

fn restore_target_list_humans(
    actor_rng: &mut ActorRng,
    target_count: u8,
) -> Vec<ActorHumanSpawn> {
    let mut humans = Vec::with_capacity(usize::from(target_count));
    let mut slot_index = 0usize;
    let mut remainder = target_count;

    if target_count > 7 {
        let quadrant_count = target_count >> 2;
        for x_bank in HUMAN_RESTORE_QUADRANT_BANKS {
            slot_index = push_target_list_restore_human_group(
                &mut humans,
                actor_rng,
                quadrant_count,
                x_bank,
                slot_index,
            );
        }
        remainder = target_count.wrapping_sub(quadrant_count << 2);
    }

    for _ in 0..remainder {
        let x_bank = actor_rng.hseed;
        slot_index = push_target_list_restore_human_group(
            &mut humans,
            actor_rng,
            1,
            x_bank,
            slot_index,
        );
    }

    humans
}

fn push_target_list_restore_human_group(
    humans: &mut Vec<ActorHumanSpawn>,
    actor_rng: &mut ActorRng,
    count: u8,
    x_bank: u8,
    mut slot_index: usize,
) -> usize {
    for _ in 0..count {
        let state = actor_rng.advance();
        let spawn_x = (state.hseed & HUMAN_RESTORE_X_OFFSET_MASK).wrapping_add(x_bank);
        let animation_frame = SpriteFrameIndex::new(if state.lseed & HUMAN_RESTORE_ANIMATION_SEED_BIT != 0 {
            2
        } else {
            0
        });
        humans.push(ActorHumanSpawn {
            position: Point::new(i16::from(spawn_x), i16::from(ASTRONAUT_RESTORE_Y)),
            mode: HumanMode::Grounded,
            runtime_state: Some(HumanRuntimeState {
                x_fraction: state.lseed,
                y_fraction: 0,
                animation_frame,
                target_slot_index: slot_index,
            }),
        });
        slot_index += 1;
    }
    slot_index
}

fn select_next_lander_target_slot_index(
    cursor: &mut Option<usize>,
    humans: &[ActorHumanSpawn],
) -> Option<usize> {
    if !humans.iter().any(|human| human.runtime_state.is_some()) {
        return None;
    }

    let original_cursor = cursor
        .filter(|slot| *slot < TARGET_LIST_ENTRY_COUNT)
        .unwrap_or(0);
    let mut probe = original_cursor;
    for _ in 0..TARGET_LIST_ENTRY_COUNT {
        probe = next_target_list_slot_index(probe);
        if humans.iter().any(|human| {
            human
                .runtime_state
                .is_some_and(|runtime_state| runtime_state.target_slot_index == probe)
        }) {
            *cursor = Some(probe);
            return Some(probe);
        }
        if probe == original_cursor {
            break;
        }
    }

    None
}

const fn next_target_list_slot_index(slot_index: usize) -> usize {
    if slot_index + 1 < TARGET_LIST_ENTRY_COUNT {
        slot_index + 1
    } else {
        0
    }
}

const fn next_astronaut_target_slot_index(slot_index: usize) -> usize {
    if slot_index + 1 < ASTRONAUT_TARGET_CURSOR_ENTRY_COUNT {
        slot_index + 1
    } else {
        0
    }
}

impl ActorHumanSpawn {
    pub const fn new(position: Point, mode: HumanMode) -> Self {
        Self {
            position,
            mode,
            runtime_state: None,
        }
    }

    const fn from_first_wave_record(
        target_slot_index: usize,
        start: FirstWaveHumanSpawnRecord,
    ) -> Self {
        Self {
            position: Point::new((start.world_x >> 8) as i16, (start.world_y >> 8) as i16),
            mode: HumanMode::Grounded,
            runtime_state: Some(HumanRuntimeState {
                x_fraction: (start.world_x & SPAWN_WORLD_FRACTION_MASK) as u8,
                y_fraction: (start.world_y & SPAWN_WORLD_FRACTION_MASK) as u8,
                animation_frame: start.animation_frame,
                target_slot_index,
            }),
        }
    }
}
