const ACTOR_FIRST_WAVE_HUMAN_SPAWNS: [ActorHumanSpawn; 10] = [
    ActorHumanSpawn::from_first_wave_record(
        0,
        FirstWaveHumanSpawnRecord {
            world_x: 0x18C3,
            world_y: 0xE000,
            animation_frame: SpriteFrameIndex::new(2),
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        1,
        FirstWaveHumanSpawnRecord {
            world_x: 0x1C81,
            world_y: 0xE100,
            animation_frame: SpriteFrameIndex::new(3),
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        2,
        FirstWaveHumanSpawnRecord {
            world_x: 0x4E30,
            world_y: 0xE000,
            animation_frame: SpriteFrameIndex::new(0),
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        3,
        FirstWaveHumanSpawnRecord {
            world_x: 0x5718,
            world_y: 0xE000,
            animation_frame: SpriteFrameIndex::new(0),
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        4,
        FirstWaveHumanSpawnRecord {
            world_x: 0x9B8C,
            world_y: 0xE000,
            animation_frame: SpriteFrameIndex::new(0),
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        5,
        FirstWaveHumanSpawnRecord {
            world_x: 0x9DC6,
            world_y: 0xE000,
            animation_frame: SpriteFrameIndex::new(0),
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        6,
        FirstWaveHumanSpawnRecord {
            world_x: 0xCEE3,
            world_y: 0xE000,
            animation_frame: SpriteFrameIndex::new(2),
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        7,
        FirstWaveHumanSpawnRecord {
            world_x: 0xD771,
            world_y: 0xE000,
            animation_frame: SpriteFrameIndex::new(2),
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        8,
        FirstWaveHumanSpawnRecord {
            world_x: 0xD2B8,
            world_y: 0xE000,
            animation_frame: SpriteFrameIndex::new(0),
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        9,
        FirstWaveHumanSpawnRecord {
            world_x: 0xE8DC,
            world_y: 0xE000,
            animation_frame: SpriteFrameIndex::new(0),
        },
    ),
];

const ACTOR_FIRST_WAVE_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xFB33,
        world_y: 0x2CE0,
        x_velocity: 0xFFDE,
        y_velocity: 0x0070,
        shot_timer: 0x27,
        sleep_ticks: 0x04,
        animation_frame: SpriteFrameIndex::new(1),
        target_human_index: Some(1),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x3F4A,
        world_y: 0x2CE0,
        x_velocity: FIRST_WAVE_EARLY_RESERVE_TARGET2_X_VELOCITY,
        y_velocity: 0x0070,
        shot_timer: 0x3B,
        sleep_ticks: 0x04,
        animation_frame: SpriteFrameIndex::new(1),
        target_human_index: Some(2),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x67FF,
        world_y: 0x2C70,
        x_velocity: 0x0012,
        y_velocity: 0x0070,
        shot_timer: 0x23,
        sleep_ticks: 0x04,
        animation_frame: SpriteFrameIndex::new(1),
        target_human_index: Some(3),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x0D11,
        world_y: 0x2C70,
        x_velocity: 0x0014,
        y_velocity: 0x0070,
        shot_timer: 0x3C,
        sleep_ticks: 0x04,
        animation_frame: SpriteFrameIndex::new(0),
        target_human_index: Some(4),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x41B9,
        world_y: 0x2C70,
        x_velocity: 0x001A,
        y_velocity: 0x0070,
        shot_timer: 0x25,
        sleep_ticks: 0x04,
        animation_frame: SpriteFrameIndex::new(1),
        target_human_index: Some(5),
    }),
];

const ACTOR_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x689A,
        world_y: 0x2C70,
        x_velocity: 0x001E,
        y_velocity: 0x0070,
        shot_timer: 0x10,
        sleep_ticks: 0,
        animation_frame: SpriteFrameIndex::new(1),
        target_human_index: Some(7),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x43D3,
        world_y: 0x2C70,
        x_velocity: 0xFFEC,
        y_velocity: 0x0070,
        shot_timer: 0x3A,
        sleep_ticks: 0,
        animation_frame: SpriteFrameIndex::new(1),
        target_human_index: Some(9),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x1F51,
        world_y: 0x2C70,
        x_velocity: 0x0014,
        y_velocity: 0x0070,
        shot_timer: 0x13,
        sleep_ticks: 0,
        animation_frame: SpriteFrameIndex::new(0),
        target_human_index: Some(8),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xFA03,
        world_y: 0x2C70,
        x_velocity: 0x0016,
        y_velocity: 0x0070,
        shot_timer: 0x26,
        sleep_ticks: 0,
        animation_frame: SpriteFrameIndex::new(1),
        target_human_index: Some(7),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xCF34,
        world_y: 0x2CE0,
        x_velocity: 0,
        y_velocity: 0,
        shot_timer: 0x34,
        sleep_ticks: 1,
        animation_frame: SpriteFrameIndex::new(0),
        target_human_index: Some(6),
    }),
];

const ACTOR_FIRST_WAVE_REFILL_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xBC29,
        world_y: 0x2CFD,
        x_velocity: 0x001C,
        y_velocity: 0x0090,
        shot_timer: 0x36,
        sleep_ticks: 6,
        animation_frame: SpriteFrameIndex::new(1),
        target_human_index: Some(7),
    })
    .with_spawn_visibility(LanderSpawnVisibility::HiddenFirstWaveRefill),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xE14C,
        world_y: 0x2CAE,
        x_velocity: 0x000E,
        y_velocity: 0x0090,
        shot_timer: 0x2F,
        sleep_ticks: 0,
        animation_frame: SpriteFrameIndex::new(0),
        target_human_index: Some(4),
    })
    .with_spawn_visibility(LanderSpawnVisibility::HiddenFirstWaveRefill),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x0A63,
        world_y: 0x2CF0,
        x_velocity: 0xFFF4,
        y_velocity: 0x0090,
        shot_timer: 0x23,
        sleep_ticks: 1,
        animation_frame: SpriteFrameIndex::new(0),
        target_human_index: Some(3),
    })
    .with_spawn_visibility(LanderSpawnVisibility::VisibleFirstWaveRefill),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x531B,
        world_y: 0x2CC0,
        x_velocity: 0xFFF6,
        y_velocity: 0x0090,
        shot_timer: 0x30,
        sleep_ticks: 1,
        animation_frame: SpriteFrameIndex::new(0),
        target_human_index: Some(2),
    })
    .with_spawn_visibility(LanderSpawnVisibility::HiddenFirstWaveRefill),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x98D9,
        world_y: 0x2CB8,
        x_velocity: 0x001A,
        y_velocity: 0x0090,
        shot_timer: 0x1F,
        sleep_ticks: 1,
        animation_frame: SpriteFrameIndex::new(0),
        target_human_index: Some(1),
    })
    .with_spawn_visibility(LanderSpawnVisibility::HiddenFirstWaveRefill),
];
