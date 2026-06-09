impl ActorLanderSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            arcade_state: None,
        }
    }

    const fn from_first_wave_record(start: FirstWaveLanderSpawnRecord) -> Self {
        Self {
            position: Point::new((start.world_x >> 8) as i16, (start.world_y >> 8) as i16),
            arcade_state: Some(LanderArcadeState {
                x_fraction: (start.world_x & 0x00FF) as u8,
                y_fraction: (start.world_y & 0x00FF) as u8,
                x_velocity: start.x_velocity,
                y_velocity: start.y_velocity,
                shot_timer: start.shot_timer,
                sleep_ticks: start.sleep_ticks,
                animation_frame: start.animation_frame,
                target_human_index: start.target_human_index,
            }),
        }
    }

    fn from_arcade_restore(
        arcade_rng: &mut ActorArcadeRng,
        profile: ArcadeWaveProfile,
        target_human_index: Option<usize>,
    ) -> Self {
        let placement_state = arcade_rng.advance();
        let x = placement_state.hseed;
        let x_fraction = placement_state.lseed;
        let y = PLAYFIELD_TOP_EDGE_Y.wrapping_add(2);
        let y_velocity =
            u16::from_be_bytes([profile.lander_y_velocity_msb, profile.lander_y_velocity_lsb]);
        let shot_timer =
            arcade_rng.advance_rmax(profile.lander_shot_time.min(u32::from(u8::MAX)) as u8);
        let x_velocity_byte = arcade_rng.advance_rmax(profile.lander_x_velocity);
        let x_velocity = if x_velocity_byte & 1 == 0 {
            u16::from(x_velocity_byte)
        } else {
            !u16::from(x_velocity_byte)
        };

        Self {
            position: Point::new(i16::from(x), i16::from(y)),
            arcade_state: Some(LanderArcadeState {
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
            arcade_state: None,
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
            arcade_state: Some(BomberArcadeState {
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

    fn arcade_restore_batch(
        profile: ArcadeWaveProfile,
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
                    .wrapping_add((squad_remaining as u16).wrapping_mul(0x0180))
                    .wrapping_add(0x8000);
                let [x, x_fraction] = world_x_word.to_be_bytes();
                bombers.push(Self {
                    position: Point::new(i16::from(x), BOMBER_CRUISE_ALTITUDE),
                    arcade_state: Some(BomberArcadeState {
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
            arcade_state: None,
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
            arcade_state: Some(PodArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: actor_sign_extend_u8_to_u16(velocity_low),
                y_velocity: 0,
            }),
        }
    }

    fn from_arcade_restore(arcade_rng: &mut ActorArcadeRng) -> Self {
        let state = arcade_rng.advance();
        let [x, x_fraction] =
            u16::from_be_bytes([(state.hseed & 0x3F).wrapping_add(0x10), state.lseed])
                .to_be_bytes();
        let y = state
            .lseed
            .wrapping_shr(1)
            .wrapping_add(PLAYFIELD_TOP_EDGE_Y);
        let x_velocity = actor_sign_extend_u8_to_u16((state.seed & 0x3F).wrapping_sub(0x20));
        let mut y_velocity_low = (state.lseed & 0x7F).wrapping_sub(0x40);
        if y_velocity_low & 0x80 == 0 {
            y_velocity_low |= 0x20;
        } else {
            y_velocity_low &= 0xDF;
        }

        Self {
            position: Point::new(i16::from(x), i16::from(y)),
            arcade_state: Some(PodArcadeState {
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
            arcade_state: None,
        }
    }

    fn from_pod_release(
        arcade_rng: &mut ActorArcadeRng,
        profile: ArcadeWaveProfile,
        position: Point,
    ) -> Self {
        let velocity_rand = arcade_rng.advance();
        let y_velocity = actor_sign_extend_u8_to_u16(velocity_rand.seed).wrapping_shl(1);
        let x_velocity =
            actor_sign_extend_u8_to_u16((velocity_rand.lseed & 0x3F).wrapping_sub(0x20));
        let acceleration = velocity_rand.lseed & profile.swarmer_acceleration_mask;
        let sleep_ticks = velocity_rand.hseed & 0x1F;
        let shot_timer =
            arcade_rng.advance_rmax(profile.swarmer_shot_time.min(u32::from(u8::MAX)) as u8);

        Self {
            position,
            arcade_state: Some(SwarmerArcadeState {
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

    fn arcade_restore_batch(
        arcade_rng: &mut ActorArcadeRng,
        profile: ArcadeWaveProfile,
        count: usize,
    ) -> Vec<Self> {
        if count == 0 {
            return Vec::new();
        }

        let world_y_word = u16::from_be_bytes([
            arcade_rng
                .seed
                .wrapping_shr(1)
                .wrapping_add(PLAYFIELD_TOP_EDGE_Y),
            0,
        ]);
        let placement_rand = arcade_rng.advance();
        let world_x_word = u16::from_be_bytes([
            (placement_rand.seed & 0x3F).wrapping_add(0x80),
            MINI_SWARMER_RESTORE_X_LOW,
        ]);
        let [x, x_fraction] = world_x_word.to_be_bytes();
        let [y, y_fraction] = world_y_word.to_be_bytes();
        let position = Point::new(i16::from(x), i16::from(y));

        (0..count)
            .map(|_| {
                let mut spawn = Self::from_pod_release(arcade_rng, profile, position);
                if let Some(arcade_state) = &mut spawn.arcade_state {
                    arcade_state.x_fraction = x_fraction;
                    arcade_state.y_fraction = y_fraction;
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
            arcade_state: None,
        }
    }

    fn from_player_position(
        profile: ArcadeWaveProfile,
        player_position: Point,
        active_baiters: usize,
    ) -> Self {
        let spawn_x = if (active_baiters + usize::from(player_position.x >= 128)).is_multiple_of(2)
        {
            228
        } else {
            28
        };
        let spawn_y = (player_position.y + 24
            - (i16::try_from(active_baiters % 3).unwrap_or(0) * 24))
            .clamp(PLAYER_BOUNDS.top + 8, HUMAN_GROUND_Y - 24);
        let position = Point::new(spawn_x, spawn_y);
        let mut arcade_state = BaiterArcadeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: BAITER_INITIAL_SHOT_TIMER,
            sleep_ticks: 0,
            picture_frame: 0,
        };
        update_baiter_arcade_velocity(
            &mut arcade_state,
            position,
            profile,
            player_position,
            Velocity::default(),
            false,
            u8::MAX,
        );
        Self {
            position,
            arcade_state: Some(arcade_state),
        }
    }
}

impl ActorMutantSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            arcade_state: None,
        }
    }

    fn from_wave_slot(
        position: Point,
        profile: ArcadeWaveProfile,
        spawn_index: usize,
    ) -> Self {
        let mut arcade_rng = DEFAULT_RNG;
        for _ in 0..=spawn_index {
            arcade_rng.advance();
        }
        let shot_timer =
            arcade_rng.advance_rmax(profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8);
        Self {
            position,
            arcade_state: Some(MutantArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer,
                sleep_ticks: 0,
                hop_rng: arcade_rng.snapshot(),
                render_x_correction: 0,
                dive_entry_shot_deferred: false,
            }),
        }
    }

    fn from_arcade_restore(
        arcade_rng: &mut ActorArcadeRng,
        profile: ArcadeWaveProfile,
        background_absolute_x: u16,
    ) -> Self {
        let placement_state = arcade_rng.advance();
        let avoid_left = background_absolute_x.wrapping_sub(MUTANT_RESTORE_AVOID_HALF_WIDTH);
        let mut relative = u16::from_be_bytes([placement_state.hseed, placement_state.lseed])
            .wrapping_sub(avoid_left);
        if relative < MUTANT_RESTORE_AVOID_WIDTH {
            relative = relative.wrapping_add(0x8000);
        }
        let world_x_word = relative.wrapping_add(avoid_left);
        let [x, x_fraction] = world_x_word.to_be_bytes();
        let y = placement_state
            .seed
            .wrapping_shr(1)
            .wrapping_add(PLAYFIELD_TOP_EDGE_Y);
        let shot_timer =
            arcade_rng.advance_rmax(profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8);

        Self {
            position: Point::new(i16::from(x), i16::from(y)),
            arcade_state: Some(MutantArcadeState {
                x_fraction,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer,
                sleep_ticks: 0,
                hop_rng: arcade_rng.snapshot(),
                render_x_correction: 0,
                dive_entry_shot_deferred: false,
            }),
        }
    }
}

fn initial_target_list_humans() -> Vec<ActorHumanSpawn> {
    let mut arcade_rng = DEFAULT_RNG;
    arcade_target_list_restore_humans(&mut arcade_rng, START_HUMAN_COUNT)
}

fn arcade_target_list_restore_humans(
    arcade_rng: &mut ActorArcadeRng,
    target_count: u8,
) -> Vec<ActorHumanSpawn> {
    let mut humans = Vec::with_capacity(usize::from(target_count));
    let mut slot_index = 0usize;
    let mut remainder = target_count;

    if target_count > 7 {
        let quadrant_count = target_count >> 2;
        for x_bank in [0x00, 0x40, 0x80, 0xC0] {
            slot_index = push_arcade_target_list_restore_human_group(
                &mut humans,
                arcade_rng,
                quadrant_count,
                x_bank,
                slot_index,
            );
        }
        remainder = target_count.wrapping_sub(quadrant_count << 2);
    }

    for _ in 0..remainder {
        let x_bank = arcade_rng.hseed;
        slot_index = push_arcade_target_list_restore_human_group(
            &mut humans,
            arcade_rng,
            1,
            x_bank,
            slot_index,
        );
    }

    humans
}

fn push_arcade_target_list_restore_human_group(
    humans: &mut Vec<ActorHumanSpawn>,
    arcade_rng: &mut ActorArcadeRng,
    count: u8,
    x_bank: u8,
    mut slot_index: usize,
) -> usize {
    for _ in 0..count {
        let state = arcade_rng.advance();
        let spawn_x = (state.hseed & 0x1F).wrapping_add(x_bank);
        let picture_frame = if state.lseed & 0x01 != 0 { 2 } else { 0 };
        humans.push(ActorHumanSpawn {
            position: Point::new(i16::from(spawn_x), i16::from(ASTRONAUT_RESTORE_Y)),
            mode: HumanMode::Grounded,
            arcade_state: Some(HumanArcadeState {
                x_fraction: state.lseed,
                y_fraction: 0,
                picture_frame,
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
    if !humans.iter().any(|human| human.arcade_state.is_some()) {
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
                .arcade_state
                .is_some_and(|arcade_state| arcade_state.target_slot_index == probe)
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
            arcade_state: None,
        }
    }

    const fn from_first_wave_record(
        target_slot_index: usize,
        start: FirstWaveHumanSpawnRecord,
    ) -> Self {
        Self {
            position: Point::new((start.world_x >> 8) as i16, (start.world_y >> 8) as i16),
            mode: HumanMode::Grounded,
            arcade_state: Some(HumanArcadeState {
                x_fraction: (start.world_x & 0x00FF) as u8,
                y_fraction: (start.world_y & 0x00FF) as u8,
                picture_frame: start.picture_frame,
                target_slot_index,
            }),
        }
    }
}
