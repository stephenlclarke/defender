use super::*;

pub(in crate::actor_game) const FIRST_WAVE_HUMAN_GROUND_WORLD_Y: u16 = 0xE000;
pub(in crate::actor_game) const FIRST_WAVE_HUMAN_RAISED_WORLD_Y: u16 = 0xE100;
pub(in crate::actor_game) const FIRST_WAVE_LANDER_UPPER_PATROL_WORLD_Y: u16 = 0x2CE0;
pub(in crate::actor_game) const FIRST_WAVE_LANDER_LOWER_PATROL_WORLD_Y: u16 = 0x2C70;
pub(in crate::actor_game) const FIRST_WAVE_LANDER_DRIFT_Y_VELOCITY: u16 = 0x0070;
pub(in crate::actor_game) const FIRST_WAVE_REFILL_LANDER_DIVE_Y_VELOCITY: u16 = 0x0090;
pub(in crate::actor_game) const FIRST_WAVE_LANDER_ENTRY_SLEEP_TICKS: u8 = 0x04;
pub(in crate::actor_game) const FIRST_WAVE_LANDER_ASLEEP_ON_ENTRY: u8 = 0;
pub(in crate::actor_game) const FIRST_WAVE_LANDER_VISIBLE_ON_NEXT_TICK: u8 = 1;
pub(in crate::actor_game) const FIRST_WAVE_STATIONARY_LANDER_VELOCITY: u16 = 0;
pub(in crate::actor_game) const FIRST_WAVE_REFILL_HIDDEN_ENTRY_SLEEP_TICKS: u8 = 6;
pub(in crate::actor_game) const FIRST_WAVE_HUMAN_STANDING_FRAME: SpriteFrameIndex =
    SpriteFrameIndex::new(0);
pub(in crate::actor_game) const FIRST_WAVE_HUMAN_WALKING_FRAME_A: SpriteFrameIndex =
    SpriteFrameIndex::new(2);
pub(in crate::actor_game) const FIRST_WAVE_HUMAN_WALKING_FRAME_B: SpriteFrameIndex =
    SpriteFrameIndex::new(3);
pub(in crate::actor_game) const FIRST_WAVE_LANDER_FRAME_A: SpriteFrameIndex =
    SpriteFrameIndex::new(0);
pub(in crate::actor_game) const FIRST_WAVE_LANDER_FRAME_B: SpriteFrameIndex =
    SpriteFrameIndex::new(1);

pub(in crate::actor_game) const ACTOR_FIRST_WAVE_HUMAN_SPAWNS: [ActorHumanSpawn; 10] = [
    ActorHumanSpawn::from_first_wave_record(
        0,
        FirstWaveHumanSpawnRecord {
            world_x: 0x18C3,
            world_y: FIRST_WAVE_HUMAN_GROUND_WORLD_Y,
            animation_frame: FIRST_WAVE_HUMAN_WALKING_FRAME_A,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        1,
        FirstWaveHumanSpawnRecord {
            world_x: 0x1C81,
            world_y: FIRST_WAVE_HUMAN_RAISED_WORLD_Y,
            animation_frame: FIRST_WAVE_HUMAN_WALKING_FRAME_B,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        2,
        FirstWaveHumanSpawnRecord {
            world_x: 0x4E30,
            world_y: FIRST_WAVE_HUMAN_GROUND_WORLD_Y,
            animation_frame: FIRST_WAVE_HUMAN_STANDING_FRAME,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        3,
        FirstWaveHumanSpawnRecord {
            world_x: 0x5718,
            world_y: FIRST_WAVE_HUMAN_GROUND_WORLD_Y,
            animation_frame: FIRST_WAVE_HUMAN_STANDING_FRAME,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        4,
        FirstWaveHumanSpawnRecord {
            world_x: 0x9B8C,
            world_y: FIRST_WAVE_HUMAN_GROUND_WORLD_Y,
            animation_frame: FIRST_WAVE_HUMAN_STANDING_FRAME,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        5,
        FirstWaveHumanSpawnRecord {
            world_x: 0x9DC6,
            world_y: FIRST_WAVE_HUMAN_GROUND_WORLD_Y,
            animation_frame: FIRST_WAVE_HUMAN_STANDING_FRAME,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        6,
        FirstWaveHumanSpawnRecord {
            world_x: 0xCEE3,
            world_y: FIRST_WAVE_HUMAN_GROUND_WORLD_Y,
            animation_frame: FIRST_WAVE_HUMAN_WALKING_FRAME_A,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        7,
        FirstWaveHumanSpawnRecord {
            world_x: 0xD771,
            world_y: FIRST_WAVE_HUMAN_GROUND_WORLD_Y,
            animation_frame: FIRST_WAVE_HUMAN_WALKING_FRAME_A,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        8,
        FirstWaveHumanSpawnRecord {
            world_x: 0xD2B8,
            world_y: FIRST_WAVE_HUMAN_GROUND_WORLD_Y,
            animation_frame: FIRST_WAVE_HUMAN_STANDING_FRAME,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        9,
        FirstWaveHumanSpawnRecord {
            world_x: 0xE8DC,
            world_y: FIRST_WAVE_HUMAN_GROUND_WORLD_Y,
            animation_frame: FIRST_WAVE_HUMAN_STANDING_FRAME,
        },
    ),
];

pub(in crate::actor_game) const ACTOR_FIRST_WAVE_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xFB33,
        world_y: FIRST_WAVE_LANDER_UPPER_PATROL_WORLD_Y,
        x_velocity: 0xFFDE,
        y_velocity: FIRST_WAVE_LANDER_DRIFT_Y_VELOCITY,
        shot_timer: 0x27,
        sleep_ticks: FIRST_WAVE_LANDER_ENTRY_SLEEP_TICKS,
        animation_frame: FIRST_WAVE_LANDER_FRAME_B,
        target_human_index: Some(1),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x3F4A,
        world_y: FIRST_WAVE_LANDER_UPPER_PATROL_WORLD_Y,
        x_velocity: FIRST_WAVE_EARLY_RESERVE_TARGET2_X_VELOCITY,
        y_velocity: FIRST_WAVE_LANDER_DRIFT_Y_VELOCITY,
        shot_timer: 0x3B,
        sleep_ticks: FIRST_WAVE_LANDER_ENTRY_SLEEP_TICKS,
        animation_frame: FIRST_WAVE_LANDER_FRAME_B,
        target_human_index: Some(2),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x67FF,
        world_y: FIRST_WAVE_LANDER_LOWER_PATROL_WORLD_Y,
        x_velocity: 0x0012,
        y_velocity: FIRST_WAVE_LANDER_DRIFT_Y_VELOCITY,
        shot_timer: 0x23,
        sleep_ticks: FIRST_WAVE_LANDER_ENTRY_SLEEP_TICKS,
        animation_frame: FIRST_WAVE_LANDER_FRAME_B,
        target_human_index: Some(3),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x0D11,
        world_y: FIRST_WAVE_LANDER_LOWER_PATROL_WORLD_Y,
        x_velocity: 0x0014,
        y_velocity: FIRST_WAVE_LANDER_DRIFT_Y_VELOCITY,
        shot_timer: 0x3C,
        sleep_ticks: FIRST_WAVE_LANDER_ENTRY_SLEEP_TICKS,
        animation_frame: FIRST_WAVE_LANDER_FRAME_A,
        target_human_index: Some(4),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x41B9,
        world_y: FIRST_WAVE_LANDER_LOWER_PATROL_WORLD_Y,
        x_velocity: 0x001A,
        y_velocity: FIRST_WAVE_LANDER_DRIFT_Y_VELOCITY,
        shot_timer: 0x25,
        sleep_ticks: FIRST_WAVE_LANDER_ENTRY_SLEEP_TICKS,
        animation_frame: FIRST_WAVE_LANDER_FRAME_B,
        target_human_index: Some(5),
    }),
];

pub(in crate::actor_game) const ACTOR_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS: [ActorLanderSpawn;
    5] = [
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x689A,
        world_y: FIRST_WAVE_LANDER_LOWER_PATROL_WORLD_Y,
        x_velocity: 0x001E,
        y_velocity: FIRST_WAVE_LANDER_DRIFT_Y_VELOCITY,
        shot_timer: 0x10,
        sleep_ticks: FIRST_WAVE_LANDER_ASLEEP_ON_ENTRY,
        animation_frame: FIRST_WAVE_LANDER_FRAME_B,
        target_human_index: Some(7),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x43D3,
        world_y: FIRST_WAVE_LANDER_LOWER_PATROL_WORLD_Y,
        x_velocity: 0xFFEC,
        y_velocity: FIRST_WAVE_LANDER_DRIFT_Y_VELOCITY,
        shot_timer: 0x3A,
        sleep_ticks: FIRST_WAVE_LANDER_ASLEEP_ON_ENTRY,
        animation_frame: FIRST_WAVE_LANDER_FRAME_B,
        target_human_index: Some(9),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x1F51,
        world_y: FIRST_WAVE_LANDER_LOWER_PATROL_WORLD_Y,
        x_velocity: 0x0014,
        y_velocity: FIRST_WAVE_LANDER_DRIFT_Y_VELOCITY,
        shot_timer: 0x13,
        sleep_ticks: FIRST_WAVE_LANDER_ASLEEP_ON_ENTRY,
        animation_frame: FIRST_WAVE_LANDER_FRAME_A,
        target_human_index: Some(8),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xFA03,
        world_y: FIRST_WAVE_LANDER_LOWER_PATROL_WORLD_Y,
        x_velocity: 0x0016,
        y_velocity: FIRST_WAVE_LANDER_DRIFT_Y_VELOCITY,
        shot_timer: 0x26,
        sleep_ticks: FIRST_WAVE_LANDER_ASLEEP_ON_ENTRY,
        animation_frame: FIRST_WAVE_LANDER_FRAME_B,
        target_human_index: Some(7),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xCF34,
        world_y: FIRST_WAVE_LANDER_UPPER_PATROL_WORLD_Y,
        x_velocity: FIRST_WAVE_STATIONARY_LANDER_VELOCITY,
        y_velocity: FIRST_WAVE_STATIONARY_LANDER_VELOCITY,
        shot_timer: 0x34,
        sleep_ticks: FIRST_WAVE_LANDER_VISIBLE_ON_NEXT_TICK,
        animation_frame: FIRST_WAVE_LANDER_FRAME_A,
        target_human_index: Some(6),
    }),
];

pub(in crate::actor_game) const ACTOR_FIRST_WAVE_REFILL_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xBC29,
        world_y: 0x2CFD,
        x_velocity: 0x001C,
        y_velocity: FIRST_WAVE_REFILL_LANDER_DIVE_Y_VELOCITY,
        shot_timer: 0x36,
        sleep_ticks: FIRST_WAVE_REFILL_HIDDEN_ENTRY_SLEEP_TICKS,
        animation_frame: FIRST_WAVE_LANDER_FRAME_B,
        target_human_index: Some(7),
    })
    .with_spawn_visibility(LanderSpawnVisibility::HiddenFirstWaveRefill),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xE14C,
        world_y: 0x2CAE,
        x_velocity: 0x000E,
        y_velocity: FIRST_WAVE_REFILL_LANDER_DIVE_Y_VELOCITY,
        shot_timer: 0x2F,
        sleep_ticks: FIRST_WAVE_LANDER_ASLEEP_ON_ENTRY,
        animation_frame: FIRST_WAVE_LANDER_FRAME_A,
        target_human_index: Some(4),
    })
    .with_spawn_visibility(LanderSpawnVisibility::HiddenFirstWaveRefill),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x0A63,
        world_y: 0x2CF0,
        x_velocity: 0xFFF4,
        y_velocity: FIRST_WAVE_REFILL_LANDER_DIVE_Y_VELOCITY,
        shot_timer: 0x23,
        sleep_ticks: FIRST_WAVE_LANDER_VISIBLE_ON_NEXT_TICK,
        animation_frame: FIRST_WAVE_LANDER_FRAME_A,
        target_human_index: Some(3),
    })
    .with_spawn_visibility(LanderSpawnVisibility::VisibleFirstWaveRefill),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x531B,
        world_y: 0x2CC0,
        x_velocity: 0xFFF6,
        y_velocity: FIRST_WAVE_REFILL_LANDER_DIVE_Y_VELOCITY,
        shot_timer: 0x30,
        sleep_ticks: FIRST_WAVE_LANDER_VISIBLE_ON_NEXT_TICK,
        animation_frame: FIRST_WAVE_LANDER_FRAME_A,
        target_human_index: Some(2),
    })
    .with_spawn_visibility(LanderSpawnVisibility::HiddenFirstWaveRefill),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x98D9,
        world_y: 0x2CB8,
        x_velocity: 0x001A,
        y_velocity: FIRST_WAVE_REFILL_LANDER_DIVE_Y_VELOCITY,
        shot_timer: 0x1F,
        sleep_ticks: FIRST_WAVE_LANDER_VISIBLE_ON_NEXT_TICK,
        animation_frame: FIRST_WAVE_LANDER_FRAME_A,
        target_human_index: Some(1),
    })
    .with_spawn_visibility(LanderSpawnVisibility::HiddenFirstWaveRefill),
];
