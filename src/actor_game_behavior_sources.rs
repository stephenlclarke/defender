fn parse_behavior_actor_kind(
    line_number: usize,
    token: &str,
) -> Result<ActorKind, ActorBehaviorScriptParseError> {
    match normalize_script_token(token).as_str() {
        "attract_director" => Ok(ActorKind::AttractDirector),
        "attract_script" => Ok(ActorKind::AttractScript),
        "status_display" => Ok(ActorKind::StatusDisplay),
        "williams_logo" => Ok(ActorKind::WilliamsLogo),
        "defender_wordmark" => Ok(ActorKind::DefenderWordmark),
        "player" => Ok(ActorKind::Player),
        "lander" => Ok(ActorKind::Lander),
        "mutant" => Ok(ActorKind::Mutant),
        "bomber" => Ok(ActorKind::Bomber),
        "bomb" => Ok(ActorKind::Bomb),
        "pod" => Ok(ActorKind::Pod),
        "swarmer" => Ok(ActorKind::Swarmer),
        "baiter" => Ok(ActorKind::Baiter),
        "human" => Ok(ActorKind::Human),
        "laser" => Ok(ActorKind::Laser),
        "enemy_laser" => Ok(ActorKind::EnemyLaser),
        "explosion" => Ok(ActorKind::Explosion),
        "score_popup" => Ok(ActorKind::ScorePopup),
        "text" => Ok(ActorKind::Text),
        _ => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("unknown actor kind `{token}`"),
        )),
    }
}

fn parse_behavior_i16_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<i16, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    i16::try_from(parse_behavior_i64(line_number, value, field)?).map_err(|error| {
        ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_behavior_u8_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<u8, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    u8::try_from(parse_behavior_u64(line_number, value, field)?).map_err(|error| {
        ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_behavior_u16_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<u16, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    u16::try_from(parse_behavior_u64(line_number, value, field)?).map_err(|error| {
        ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_behavior_u64_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<u64, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    parse_behavior_u64(line_number, value, field)
}

fn parse_behavior_bool_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<bool, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    match normalize_script_token(value).as_str() {
        "true" | "yes" | "on" | "1" => Ok(true),
        "false" | "no" | "off" | "0" => Ok(false),
        _ => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is not a boolean"),
        )),
    }
}

fn parse_behavior_hyperspace_arcade_seed_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<Option<ActorHyperspaceArcadeSeed>, ActorBehaviorScriptParseError> {
    if values.len() == 1 && normalize_script_token(values[0]) == "none" {
        return Ok(None);
    }
    if values.len() != 3 {
        return Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} needs `none` or three seed bytes"),
        ));
    }
    Ok(Some(ActorHyperspaceArcadeSeed {
        seed: parse_behavior_u8_value(line_number, &values[0..1], "seed")?,
        hseed: parse_behavior_u8_value(line_number, &values[1..2], "hseed")?,
        lseed: parse_behavior_u8_value(line_number, &values[2..3], "lseed")?,
    }))
}

fn parse_lander_behavior_mode_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<LanderBehaviorMode, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    match normalize_script_token(value).as_str() {
        "seek_nearest_human" | "seek_human" | "human" => Ok(LanderBehaviorMode::SeekNearestHuman),
        "chase_player" | "chase" | "player" => Ok(LanderBehaviorMode::ChasePlayer),
        "drift" => Ok(LanderBehaviorMode::Drift),
        _ => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is not a lander behavior mode"),
        )),
    }
}

fn parse_hostile_movement_mode_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<HostileMovementMode, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    match normalize_script_token(value).as_str() {
        "drift" => Ok(HostileMovementMode::Drift),
        "chase_player" | "chase" | "player" => Ok(HostileMovementMode::ChasePlayer),
        _ => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is not a hostile movement mode"),
        )),
    }
}

fn parse_behavior_single_value<'a>(
    line_number: usize,
    values: &'a [&'a str],
    field: &str,
) -> Result<&'a str, ActorBehaviorScriptParseError> {
    match values {
        [value] => Ok(value),
        [] => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} needs a value"),
        )),
        _ => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} accepts one value"),
        )),
    }
}

fn parse_behavior_u64(
    line_number: usize,
    value: &str,
    field: &str,
) -> Result<u64, ActorBehaviorScriptParseError> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16).map_err(|error| {
            ActorBehaviorScriptParseError::new(
                line_number,
                format!("{field} `{value}` is invalid: {error}"),
            )
        });
    }
    value.parse::<u64>().map_err(|error| {
        ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_behavior_i64(
    line_number: usize,
    value: &str,
    field: &str,
) -> Result<i64, ActorBehaviorScriptParseError> {
    if let Some(hex) = value
        .strip_prefix("-0x")
        .or_else(|| value.strip_prefix("-0X"))
    {
        return i64::from_str_radix(hex, 16)
            .map(|value| -value)
            .map_err(|error| {
                ActorBehaviorScriptParseError::new(
                    line_number,
                    format!("{field} `{value}` is invalid: {error}"),
                )
            });
    }
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return i64::from_str_radix(hex, 16).map_err(|error| {
            ActorBehaviorScriptParseError::new(
                line_number,
                format!("{field} `{value}` is invalid: {error}"),
            )
        });
    }
    value.parse::<i64>().map_err(|error| {
        ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

impl Default for ActorBehaviorScript {
    fn default() -> Self {
        Self::red_label_default()
    }
}

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
    pub picture_frame: u8,
    pub target_human_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BomberArcadeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub picture_frame: u8,
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
    pub picture_frame: u8,
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
    pub target6_first_shot_deferred: bool,
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
            render_x_correction: actor_source_target6_mutant_conversion_x_correction(lander_runtime)
                .unwrap_or(0),
            target6_first_shot_deferred: false,
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
    pub source: Option<LanderArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBomberSpawn {
    pub position: Point,
    pub source: Option<BomberArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorPodSpawn {
    pub position: Point,
    pub source: Option<PodArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSwarmerSpawn {
    pub position: Point,
    pub source: Option<SwarmerArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBaiterSpawn {
    pub position: Point,
    pub source: Option<BaiterArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorMutantSpawn {
    pub position: Point,
    pub source: Option<MutantArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HumanArcadeState {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub picture_frame: u8,
    pub target_slot_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorHumanSpawn {
    pub position: Point,
    pub mode: HumanMode,
    pub source: Option<HumanArcadeState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FirstWaveLanderSpawnRecord {
    world_x: u16,
    world_y: u16,
    x_velocity: u16,
    y_velocity: u16,
    shot_timer: u8,
    sleep_ticks: u8,
    picture_frame: u8,
    target_human_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FirstWaveHumanSpawnRecord {
    world_x: u16,
    world_y: u16,
    picture_frame: u8,
}

impl ActorLanderSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            source: None,
        }
    }

    const fn from_first_wave_record(start: FirstWaveLanderSpawnRecord) -> Self {
        Self {
            position: Point::new((start.world_x >> 8) as i16, (start.world_y >> 8) as i16),
            source: Some(LanderArcadeState {
                x_fraction: (start.world_x & 0x00FF) as u8,
                y_fraction: (start.world_y & 0x00FF) as u8,
                x_velocity: start.x_velocity,
                y_velocity: start.y_velocity,
                shot_timer: start.shot_timer,
                sleep_ticks: start.sleep_ticks,
                picture_frame: start.picture_frame,
                target_human_index: start.target_human_index,
            }),
        }
    }

    fn source_restore(
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
            source: Some(LanderArcadeState {
                x_fraction,
                y_fraction: 0,
                x_velocity,
                y_velocity,
                shot_timer,
                sleep_ticks: 0,
                picture_frame: 0,
                target_human_index,
            }),
        }
    }
}

impl ActorBomberSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            source: None,
        }
    }

    const fn source_initial(position: Point, x_velocity_word: u8, spawn_index: usize) -> Self {
        let velocity_low = if spawn_index < 2 {
            0u8.wrapping_sub(x_velocity_word)
        } else {
            x_velocity_word
        };
        Self {
            position,
            source: Some(BomberArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: actor_sign_extend_u8_to_u16(velocity_low),
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                slot: (spawn_index % 4) as u8,
            }),
        }
    }

    fn source_restore_batch(
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
                let x16 = player_absolute_x
                    .wrapping_add((squad_remaining as u16).wrapping_mul(0x0180))
                    .wrapping_add(0x8000);
                let [x, x_fraction] = x16.to_be_bytes();
                bombers.push(Self {
                    position: Point::new(i16::from(x), BOMBER_CRUISE_ALTITUDE),
                    source: Some(BomberArcadeState {
                        x_fraction,
                        y_fraction: 0,
                        x_velocity,
                        y_velocity: 0,
                        picture_frame: 0,
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
            source: None,
        }
    }

    const fn source_initial(position: Point, spawn_index: usize) -> Self {
        let velocity_low = if spawn_index < 2 {
            0u8.wrapping_sub(INITIAL_POD_X_SPEED)
        } else {
            INITIAL_POD_X_SPEED
        };
        Self {
            position,
            source: Some(PodArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: actor_sign_extend_u8_to_u16(velocity_low),
                y_velocity: 0,
            }),
        }
    }

    fn source_restore(arcade_rng: &mut ActorArcadeRng) -> Self {
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
            source: Some(PodArcadeState {
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
            source: None,
        }
    }

    fn source_from_pod(
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
            source: Some(SwarmerArcadeState {
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

    fn source_restore_batch(
        arcade_rng: &mut ActorArcadeRng,
        profile: ArcadeWaveProfile,
        count: usize,
    ) -> Vec<Self> {
        if count == 0 {
            return Vec::new();
        }

        let y16 = u16::from_be_bytes([
            arcade_rng
                .seed
                .wrapping_shr(1)
                .wrapping_add(PLAYFIELD_TOP_EDGE_Y),
            0,
        ]);
        let placement_rand = arcade_rng.advance();
        let x16 = u16::from_be_bytes([
            (placement_rand.seed & 0x3F).wrapping_add(0x80),
            MINI_SWARMER_RESTORE_X_LOW,
        ]);
        let [x, x_fraction] = x16.to_be_bytes();
        let [y, y_fraction] = y16.to_be_bytes();
        let position = Point::new(i16::from(x), i16::from(y));

        (0..count)
            .map(|_| {
                let mut spawn = Self::source_from_pod(arcade_rng, profile, position);
                if let Some(source) = &mut spawn.source {
                    source.x_fraction = x_fraction;
                    source.y_fraction = y_fraction;
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
            source: None,
        }
    }

    fn source_from_player(
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
        let mut source = BaiterArcadeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: BAITER_INITIAL_SHOT_TIMER,
            sleep_ticks: 0,
            picture_frame: 0,
        };
        update_baiter_arcade_velocity(
            &mut source,
            position,
            profile,
            player_position,
            Velocity::default(),
            false,
            u8::MAX,
        );
        Self {
            position,
            source: Some(source),
        }
    }
}

impl ActorMutantSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            source: None,
        }
    }

    fn source_initial(
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
            source: Some(MutantArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer,
                sleep_ticks: 0,
                hop_rng: arcade_rng.snapshot(),
                render_x_correction: 0,
                target6_first_shot_deferred: false,
            }),
        }
    }

    fn source_restore(
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
        let x16 = relative.wrapping_add(avoid_left);
        let [x, x_fraction] = x16.to_be_bytes();
        let y = placement_state
            .seed
            .wrapping_shr(1)
            .wrapping_add(PLAYFIELD_TOP_EDGE_Y);
        let shot_timer =
            arcade_rng.advance_rmax(profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8);

        Self {
            position: Point::new(i16::from(x), i16::from(y)),
            source: Some(MutantArcadeState {
                x_fraction,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer,
                sleep_ticks: 0,
                hop_rng: arcade_rng.snapshot(),
                render_x_correction: 0,
                target6_first_shot_deferred: false,
            }),
        }
    }
}

fn actor_source_initial_target_list_humans() -> Vec<ActorHumanSpawn> {
    let mut arcade_rng = DEFAULT_RNG;
    actor_source_target_list_restore_humans(&mut arcade_rng, START_HUMAN_COUNT)
}

fn actor_source_target_list_restore_humans(
    arcade_rng: &mut ActorArcadeRng,
    target_count: u8,
) -> Vec<ActorHumanSpawn> {
    let mut humans = Vec::with_capacity(usize::from(target_count));
    let mut slot_index = 0usize;
    let mut remainder = target_count;

    if target_count > 7 {
        let quadrant_count = target_count >> 2;
        for x_bank in [0x00, 0x40, 0x80, 0xC0] {
            slot_index = actor_source_target_list_restore_human_group(
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
        slot_index = actor_source_target_list_restore_human_group(
            &mut humans,
            arcade_rng,
            1,
            x_bank,
            slot_index,
        );
    }

    humans
}

fn actor_source_target_list_restore_human_group(
    humans: &mut Vec<ActorHumanSpawn>,
    arcade_rng: &mut ActorArcadeRng,
    count: u8,
    x_bank: u8,
    mut slot_index: usize,
) -> usize {
    for _ in 0..count {
        let state = arcade_rng.advance();
        let source_x = (state.hseed & 0x1F).wrapping_add(x_bank);
        let picture_frame = if state.lseed & 0x01 != 0 { 2 } else { 0 };
        humans.push(ActorHumanSpawn {
            position: Point::new(i16::from(source_x), i16::from(ASTRONAUT_RESTORE_Y)),
            mode: HumanMode::Grounded,
            source: Some(HumanArcadeState {
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

fn actor_source_select_lander_target_index(
    cursor: &mut Option<usize>,
    humans: &[ActorHumanSpawn],
) -> Option<usize> {
    if !humans.iter().any(|human| human.source.is_some()) {
        return None;
    }

    let original_cursor = cursor
        .filter(|slot| *slot < TARGET_LIST_ENTRY_COUNT)
        .unwrap_or(0);
    let mut probe = original_cursor;
    for _ in 0..TARGET_LIST_ENTRY_COUNT {
        probe = actor_source_target_list_next_slot_index(probe);
        if humans.iter().any(|human| {
            human
                .source
                .is_some_and(|source| source.target_slot_index == probe)
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

const fn actor_source_target_list_next_slot_index(slot_index: usize) -> usize {
    if slot_index + 1 < TARGET_LIST_ENTRY_COUNT {
        slot_index + 1
    } else {
        0
    }
}

const fn actor_source_astronaut_next_slot_index(slot_index: usize) -> usize {
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
            source: None,
        }
    }

    const fn from_first_wave_record(
        target_slot_index: usize,
        start: FirstWaveHumanSpawnRecord,
    ) -> Self {
        Self {
            position: Point::new((start.world_x >> 8) as i16, (start.world_y >> 8) as i16),
            mode: HumanMode::Grounded,
            source: Some(HumanArcadeState {
                x_fraction: (start.world_x & 0x00FF) as u8,
                y_fraction: (start.world_y & 0x00FF) as u8,
                picture_frame: start.picture_frame,
                target_slot_index,
            }),
        }
    }
}

impl ArcadeWaveProfile {
    pub fn for_wave(wave: u16) -> Self {
        let wave = u8::try_from(wave.min(u16::from(u8::MAX))).unwrap_or(u8::MAX);
        Self {
            landers: actor_arcade_wave_u8(crate::arcade_assets::WaveMetric::Landers, wave),
            bombers: actor_arcade_wave_u8(crate::arcade_assets::WaveMetric::Bombers, wave),
            pods: actor_arcade_wave_u8(crate::arcade_assets::WaveMetric::Pods, wave),
            mutants: actor_arcade_wave_u8(crate::arcade_assets::WaveMetric::Mutants, wave),
            swarmers: actor_arcade_wave_u8(crate::arcade_assets::WaveMetric::Swarmers, wave),
            wave_size: actor_arcade_wave_u8(crate::arcade_assets::WaveMetric::WaveSize, wave),
            lander_x_velocity: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::LanderXVelocity,
                wave,
            ),
            lander_y_velocity_msb: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::LanderYVelocityHigh,
                wave,
            ),
            lander_y_velocity_lsb: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::LanderYVelocityLow,
                wave,
            ),
            bomber_x_velocity: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::BomberXVelocity,
                wave,
            ),
            swarmer_x_velocity: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::SwarmerXVelocity,
                wave,
            ),
            swarmer_shot_time: actor_arcade_wave_u32(
                crate::arcade_assets::WaveMetric::SwarmerShotTime,
                wave,
            ),
            swarmer_acceleration_mask: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::SwarmerAccelerationMask,
                wave,
            ),
            baiter_delay: actor_arcade_wave_u32(
                crate::arcade_assets::WaveMetric::BaiterDelay,
                wave,
            ),
            baiter_shot_time: actor_arcade_wave_u32(
                crate::arcade_assets::WaveMetric::BaiterShotTime,
                wave,
            ),
            baiter_seek_probability: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::BaiterSeekProbability,
                wave,
            ),
            lander_shot_time: actor_arcade_wave_u32(
                crate::arcade_assets::WaveMetric::LanderShotTime,
                wave,
            ),
            mutant_random_y: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::MutantRandomY,
                wave,
            ),
            mutant_y_velocity_msb: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::MutantYVelocityHigh,
                wave,
            ),
            mutant_y_velocity_lsb: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::MutantYVelocityLow,
                wave,
            ),
            mutant_x_velocity: actor_arcade_wave_u8(
                crate::arcade_assets::WaveMetric::MutantXVelocity,
                wave,
            ),
            mutant_shot_time: actor_arcade_wave_u32(
                crate::arcade_assets::WaveMetric::MutantShotTime,
                wave,
            ),
        }
    }

    fn lander_behavior(self) -> ActorBehaviorProfile {
        ActorBehaviorProfile {
            lander_seek_speed: actor_lander_speed_from_source(self.lander_x_velocity),
            lander_drift_speed: actor_lander_speed_from_source(self.lander_x_velocity),
            lander_fire_period_steps: u64::from(self.lander_shot_time.max(1)),
            ..ActorBehaviorProfile::default()
        }
    }

    fn lander_spawns(self, wave: u16, humans: &[ActorHumanSpawn]) -> Vec<ActorLanderSpawn> {
        let mut source_lander_index = 0;
        let mut arcade_rng = DEFAULT_RNG;
        let mut target_cursor = Some(0usize);
        self.active_family_slots()
            .into_iter()
            .filter_map(|slot| {
                if slot.kind != ActorSourceEnemyKind::Lander {
                    return None;
                }
                let spawn = if wave == 1 {
                    ACTOR_FIRST_WAVE_LANDER_SPAWNS
                        .get(source_lander_index)
                        .copied()
                        .unwrap_or_else(|| ActorLanderSpawn::new(slot.position))
                } else {
                    ActorLanderSpawn::source_restore(
                        &mut arcade_rng,
                        self,
                        actor_source_select_lander_target_index(&mut target_cursor, humans),
                    )
                };
                source_lander_index += 1;
                Some(spawn)
            })
            .collect()
    }

    fn human_spawns(self, wave: u16) -> Vec<ActorHumanSpawn> {
        if wave == 1 {
            ACTOR_FIRST_WAVE_HUMAN_SPAWNS.to_vec()
        } else {
            actor_source_initial_target_list_humans()
        }
    }

    fn bomber_spawns(self) -> Vec<ActorBomberSpawn> {
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == ActorSourceEnemyKind::Bomber)
            .map(|slot| {
                ActorBomberSpawn::source_initial(slot.position, self.bomber_x_velocity, slot.index)
            })
            .collect()
    }

    fn pod_spawns(self) -> Vec<ActorPodSpawn> {
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == ActorSourceEnemyKind::Pod)
            .map(|slot| ActorPodSpawn::source_initial(slot.position, slot.index))
            .collect()
    }

    fn mutant_spawns(self) -> Vec<ActorMutantSpawn> {
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == ActorSourceEnemyKind::Mutant)
            .map(|slot| ActorMutantSpawn::source_initial(slot.position, self, slot.index))
            .collect()
    }

    fn swarmer_spawns(self) -> Vec<ActorSwarmerSpawn> {
        let mut arcade_rng = DEFAULT_RNG;
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == ActorSourceEnemyKind::Swarmer)
            .map(|slot| ActorSwarmerSpawn::source_from_pod(&mut arcade_rng, self, slot.position))
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

    fn active_family_slots(self) -> Vec<ActorSourceEnemySlot> {
        let mut counts = ActorSourceEnemyCounts {
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
            ActorSourceEnemyKind::Lander,
            ActorSourceEnemyKind::Bomber,
            ActorSourceEnemyKind::Pod,
            ActorSourceEnemyKind::Mutant,
            ActorSourceEnemyKind::Swarmer,
        ] {
            push_actor_source_kind(&mut kinds, &mut counts, target, kind);
        }
        for kind in [
            ActorSourceEnemyKind::Lander,
            ActorSourceEnemyKind::Bomber,
            ActorSourceEnemyKind::Pod,
            ActorSourceEnemyKind::Mutant,
            ActorSourceEnemyKind::Swarmer,
        ] {
            while kinds.len() < target && counts.take(kind) {
                kinds.push(kind);
            }
        }

        kinds
            .into_iter()
            .enumerate()
            .map(|(index, kind)| ActorSourceEnemySlot {
                kind,
                index,
                position: ACTOR_WAVE_ACTIVE_SPAWN_SLOTS[index],
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceEnemySlot {
    kind: ActorSourceEnemyKind,
    index: usize,
    position: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorSourceEnemyKind {
    Lander,
    Bomber,
    Pod,
    Mutant,
    Swarmer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceEnemyCounts {
    landers: u8,
    bombers: u8,
    pods: u8,
    mutants: u8,
    swarmers: u8,
}

impl ActorSourceEnemyCounts {
    const fn total(self) -> u8 {
        self.landers
            .saturating_add(self.bombers)
            .saturating_add(self.pods)
            .saturating_add(self.mutants)
            .saturating_add(self.swarmers)
    }

    fn take(&mut self, kind: ActorSourceEnemyKind) -> bool {
        let count = match kind {
            ActorSourceEnemyKind::Lander => &mut self.landers,
            ActorSourceEnemyKind::Bomber => &mut self.bombers,
            ActorSourceEnemyKind::Pod => &mut self.pods,
            ActorSourceEnemyKind::Mutant => &mut self.mutants,
            ActorSourceEnemyKind::Swarmer => &mut self.swarmers,
        };
        if *count == 0 {
            return false;
        }
        *count = count.saturating_sub(1);
        true
    }
}

fn push_actor_source_kind(
    kinds: &mut Vec<ActorSourceEnemyKind>,
    counts: &mut ActorSourceEnemyCounts,
    target: usize,
    kind: ActorSourceEnemyKind,
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
    kind: ActorSourceEnemyKind,
) -> bool {
    let count = match kind {
        ActorSourceEnemyKind::Lander => &mut reserve.landers,
        ActorSourceEnemyKind::Bomber => &mut reserve.bombers,
        ActorSourceEnemyKind::Pod => &mut reserve.pods,
        ActorSourceEnemyKind::Mutant => &mut reserve.mutants,
        ActorSourceEnemyKind::Swarmer => &mut reserve.swarmers,
    };
    if *count == 0 {
        return false;
    }
    *count = count.saturating_sub(1);
    true
}

fn actor_source_reserve_enemy_kinds(
    reserve: &mut EnemyReserveSnapshot,
    profile: ArcadeWaveProfile,
) -> Vec<ActorSourceEnemyKind> {
    if reserve.landers > 0 {
        let target = MAX_ACTIVE_WAVE_ENEMIES.min(usize::from(reserve.landers));
        let mut kinds = Vec::with_capacity(target);
        while kinds.len() < target
            && actor_enemy_reserve_take(reserve, ActorSourceEnemyKind::Lander)
        {
            kinds.push(ActorSourceEnemyKind::Lander);
        }
        return kinds;
    }

    let target = usize::from(profile.wave_size)
        .min(MAX_ACTIVE_WAVE_ENEMIES)
        .min(usize::from(actor_enemy_reserve_total(*reserve)));
    let mut kinds = Vec::with_capacity(target);

    for kind in [
        ActorSourceEnemyKind::Lander,
        ActorSourceEnemyKind::Bomber,
        ActorSourceEnemyKind::Pod,
        ActorSourceEnemyKind::Mutant,
        ActorSourceEnemyKind::Swarmer,
    ] {
        push_actor_reserve_kind(&mut kinds, reserve, target, kind);
    }

    for kind in [
        ActorSourceEnemyKind::Lander,
        ActorSourceEnemyKind::Bomber,
        ActorSourceEnemyKind::Pod,
        ActorSourceEnemyKind::Mutant,
        ActorSourceEnemyKind::Swarmer,
    ] {
        while kinds.len() < target && actor_enemy_reserve_take(reserve, kind) {
            kinds.push(kind);
        }
    }

    kinds
}

fn push_actor_reserve_kind(
    kinds: &mut Vec<ActorSourceEnemyKind>,
    reserve: &mut EnemyReserveSnapshot,
    target: usize,
    kind: ActorSourceEnemyKind,
) {
    if kinds.len() < target && actor_enemy_reserve_take(reserve, kind) {
        kinds.push(kind);
    }
}

fn actor_lander_speed_from_source(velocity: u8) -> i16 {
    i16::from((velocity / 16).max(1))
}

fn actor_velocity_pixels_from_source(velocity: u8) -> i16 {
    i16::from((velocity / 32).max(1))
}

fn adc8(lhs: u8, rhs: u8, carry: bool) -> (u8, bool) {
    let sum = u16::from(lhs) + u16::from(rhs) + u16::from(u8::from(carry));
    ((sum & 0xFF) as u8, sum > 0xFF)
}

fn source_rmax(max: u8, mut seed: u8) -> u8 {
    while seed > max {
        seed >>= 1;
    }
    seed.wrapping_add(1)
}

const fn actor_sign_extend_u8_to_u16(value: u8) -> u16 {
    let sign = if value & 0x80 == 0 { 0x00 } else { 0xFF };
    u16::from_be_bytes([sign, value])
}

fn actor_arcade_wave_u8(metric: crate::arcade_assets::WaveMetric, wave: u8) -> u8 {
    u8::try_from(actor_arcade_wave_value(metric, wave))
        .unwrap_or_else(|_| panic!("actor wave metric should fit u8"))
}

fn actor_arcade_wave_u32(metric: crate::arcade_assets::WaveMetric, wave: u8) -> u32 {
    u32::try_from(actor_arcade_wave_value(metric, wave))
        .unwrap_or_else(|_| panic!("actor wave metric should be non-negative"))
}

fn actor_arcade_wave_value(metric: crate::arcade_assets::WaveMetric, wave: u8) -> i32 {
    crate::arcade_assets::wave_metric_value(
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
