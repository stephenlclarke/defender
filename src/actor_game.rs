//! Actor-oriented Defender rewrite prototype.
//!
//! This module is intentionally independent from the current MAME-shaped
//! `Game` implementation. It models the game as driver-owned actor threads:
//! the driver prompts every asset once per step, gathers commands, resolves
//! world rules in a stable order, and publishes a step description.

use crate::{
    game::{
        ATTRACT_SCORING_SEQUENCE_START_FRAME, AttractPresentationSnapshot,
        Direction as CleanDirection, EnemyKind as CleanEnemyKind,
        EnemyProjectileSnapshot as CleanEnemyProjectileSnapshot, EnemyProjectileSourceKind,
        EnemyReserveSnapshot, EnemySnapshot as CleanEnemySnapshot,
        ExplosionKind as CleanExplosionKind, ExplosionSnapshot as CleanExplosionSnapshot,
        GameEvent, GameEvents, GameFrame, GameInput as CleanGameInput, GameOverSnapshot, GamePhase,
        GameState, HIGH_SCORE_TABLE_ENTRIES, HighScoreEntrySnapshot, HighScoreTableEntrySnapshot,
        HighScoreTablesSnapshot, HumanSnapshot as CleanHumanSnapshot, PlayerSnapshot,
        PlayerStockSnapshot, ProjectileSnapshot as CleanProjectileSnapshot,
        SOURCE_TERRAIN_BLOW_COMPLETE_FRAME, SOURCE_TERRAIN_BLOW_FLASH_COLOR_BYTES,
        SOURCE_TERRAIN_BLOW_OVERLOAD_COUNTER, SOURCE_TERRAIN_BLOW_START_SOUND_FRAMES,
        SOURCE_TERRAIN_EXPLOSION_LIFETIME_FRAMES, ScorePopupKind as CleanScorePopupKind,
        ScorePopupSnapshot as CleanScorePopupSnapshot, ScoreSnapshot, SoundEvent,
        SourceBaiterSnapshot, SourceBomberSnapshot, SourceLanderSnapshot, SourceMutantSnapshot,
        SourcePodSnapshot, SourceRandSnapshot, SourceSwarmerSnapshot, TerrainBlowSnapshot,
        TerrainBlowStage, TerrainSegment, WaveProfileSnapshot, WorldSnapshot, WorldVector,
        push_source_bgout_terrain_sprites, push_source_explosion_cloud_pixels,
        source_explosion_render_scale, source_explosion_size_for_age,
        source_terrain_blow_flash_tint, source_terrain_explosion_size_for_age,
    },
    renderer::{
        Color, RenderLayer, RenderScene, SceneSprite, SpriteId, SurfaceSize,
        push_source_controlled_message_sprites, push_source_text_bytes_sprites,
        source_attract_defender_appearance_pixels,
        source_attract_williams_logo_operation_pixel_counts,
        source_attract_williams_logo_pixel_path, source_message_text, source_screen_position,
        source_screen_position_with_offset,
    },
    systems::{
        HighScoreEntrySystem, HighScoreInitialsState, PlayerStock, ScoreSystem, ScreenPosition,
        ScreenVelocity,
    },
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt,
    str::FromStr,
    sync::{
        OnceLock,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

const PLAYER_SPEED: i16 = 2;
const ACTOR_RENDER_SURFACE: SurfaceSize = SurfaceSize::new(292, 240);
const INITIAL_PLAYER_LIVES: u8 = 3;
const INITIAL_SMART_BOMBS: u8 = 3;
const PLAYER_LASER_COOLDOWN_STEPS: u8 = 8;
const PLAYER_HYPERSPACE_HIDDEN_STEPS: u8 = 33;
const PLAYER_HYPERSPACE_REMATERIALIZE_X: i16 = 128;
const PLAYER_HYPERSPACE_REMATERIALIZE_Y: i16 = 120;
const PLAYER_HYPERSPACE_DEATH_DELAY_STEPS: u8 = 39;
const PLAYER_HYPERSPACE_DEATH_LSEED: u8 = 0x0C;
const SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD: u8 = 0xC0;
const SOURCE_PLAYFIELD_Y_MIN: u8 = 42;
const SOURCE_PLAYFIELD_Y_MAX: u8 = 240;
const SOURCE_MUTANT_RESTORE_AVOID_HALF_WIDTH: u16 = 300 * 32;
const SOURCE_MUTANT_RESTORE_AVOID_WIDTH: u16 = 600 * 32;
const SOURCE_SHELL_SCAN_INITIAL_DELAY_STEPS: u8 = 6;
const SOURCE_SHELL_SCAN_CADENCE_STEPS: u8 = 8;
const SOURCE_SHELL_LIMIT: usize = 20;
const SOURCE_SHELL_LIFETIME_TICKS: u8 = 20;
const SOURCE_SMART_BOMB_DETONATION_DELAY_STEPS: u8 = 3;
const SOURCE_SMART_BOMB_FLASH_STEPS: u8 = 5;
const SOURCE_SMART_BOMB_RESERVE_DELAY_STEPS: u16 = 240;
const SOURCE_SBSND_SOUND_COMMAND: u8 = 0xEE;
const SOURCE_CANNON_SOUND_COMMAND: u8 = 0xE8;
const SOURCE_TBSND_SOUND_COMMAND: u8 = 0xEB;
const SOURCE_ACSND_SOUND_COMMAND: u8 = 0xF7;
const SOURCE_ASCSND_SOUND_COMMAND: u8 = 0xE5;
const SOURCE_APPEAR_SOUND_COMMAND: u8 = 0xEA;
const SOURCE_SMART_BOMB_SOUND_SEQUENCE: [(u8, u8); 7] = [
    (4, SOURCE_SBSND_SOUND_COMMAND),
    (8, SOURCE_SBSND_SOUND_COMMAND),
    (12, SOURCE_SBSND_SOUND_COMMAND),
    (16, SOURCE_SBSND_SOUND_COMMAND),
    (20, SOURCE_SBSND_SOUND_COMMAND),
    (24, SOURCE_SBSND_SOUND_COMMAND),
    (28, SOURCE_CANNON_SOUND_COMMAND),
];
const SOURCE_TERRAIN_BLOW_SOUND_TAIL_SEQUENCE: [(u8, u8); 4] = [
    (4, SOURCE_SBSND_SOUND_COMMAND),
    (10, SOURCE_SBSND_SOUND_COMMAND),
    (16, SOURCE_CANNON_SOUND_COMMAND),
    (26, SOURCE_CANNON_SOUND_COMMAND),
];
const SOURCE_ACSND_SOUND_TAIL_SEQUENCE: [(u8, u8); 2] = [
    (10, SOURCE_ACSND_SOUND_COMMAND),
    (20, SOURCE_ACSND_SOUND_COMMAND),
];
const SOURCE_PLAYER_SWITCH_SLEEP_STEPS: u8 = 0x60;
const SOURCE_START_SOUND_DELAY_STEPS: u8 = 1;
const SOURCE_START_PLAYFIELD_DELAY_STEPS: u8 = 138;
const SOURCE_SHELL_X_MAX: i16 = 0x98;
const SOURCE_PLAYFIELD_START_RNG: ActorSourceRng = ActorSourceRng {
    seed: 0x52,
    hseed: 0x62,
    lseed: 0x0C,
};
const SOURCE_DEFAULT_RNG: ActorSourceRng = ActorSourceRng {
    seed: 0,
    hseed: 0xA5,
    lseed: 0x5A,
};
const SOURCE_FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS: u16 = 449;
const SOURCE_FIRST_WAVE_EARLY_RESERVE_ACTIVE_LIMIT: usize = 10;
const SOURCE_FIRST_WAVE_EARLY_RESERVE_TARGET_CURSOR_SLOT: usize = 6;
const SOURCE_FIRST_WAVE_EARLY_RESERVE_TARGET2_SHOT_PHASE_DELAY: u8 = 2;
const SOURCE_FIRST_WAVE_EARLY_RESERVE_RNG: ActorSourceRng = ActorSourceRng {
    seed: 0x3A,
    hseed: 0xDA,
    lseed: 0x1F,
};
const SOURCE_FIRST_WAVE_LANDER_REFILL_ACTIVE_THRESHOLD: usize = 8;
const SOURCE_FIRST_WAVE_LANDER_REFILL_DELAY_STEPS: u8 = 47;
const SOURCE_FIRST_WAVE_LANDER_REFILL_APPEAR_SOUND_DELAY_STEPS: u8 = 1;
const PLAYER_BOUNDS: Rect = Rect::new(0, 18, 255, 220);
const LASER_SPEED: i16 = 8;
const LASER_LIFETIME: u16 = 34;
const LANDER_FIRE_PERIOD: u64 = 96;
const LANDER_SHOT_SPEED: i16 = 3;
const LANDER_SHOT_LIFETIME: u16 = 90;
const EXPLOSION_LIFETIME: u16 = 20;
const SCORE_POPUP_LIFETIME: u16 = 50;
const SOURCE_ATTRACT_PRESENTS_START_STEP: u64 = 236;
const SOURCE_ATTRACT_DEFENDER_WORDMARK_START_STEP: u64 = 365;
const SOURCE_ATTRACT_HALL_OF_FAME_START_STEP: u64 = 488;
const SOURCE_ATTRACT_SCORING_SEQUENCE_START_STEP: u64 = ATTRACT_SCORING_SEQUENCE_START_FRAME as u64;
const SOURCE_ATTRACT_CYCLE_STEPS: u64 =
    SOURCE_ATTRACT_SCORING_SEQUENCE_START_STEP + ATTRACT_SCORING_DEMO_TOTAL_STEPS as u64;
const SOURCE_HIGH_SCORE_HALL_STALL_STEPS: u8 = 60;
const SOURCE_ATTRACT_WILLIAMS_LOGO_DURATION_STEPS: u64 = SOURCE_ATTRACT_HALL_OF_FAME_START_STEP - 1;
const SOURCE_ATTRACT_PRESENTS_DURATION_STEPS: u64 =
    SOURCE_ATTRACT_HALL_OF_FAME_START_STEP - SOURCE_ATTRACT_PRESENTS_START_STEP;
const SOURCE_REPLAY_SCORE: u32 = 10_000;
const SOURCE_WAVE_COMPLETION_STATUS_LINES: &[(&str, u16)] =
    &[("ATWV", 0x3850), ("COMPV", 0x3D60), ("BONSX", 0x3C90)];
const SOURCE_WAVE_COMPLETION_WAVE_NUMBER_SCREEN: u16 = 0x6550;
const SOURCE_WAVE_COMPLETION_MULTIPLIER_NUMBER_SCREEN: u16 = 0x5890;
const SOURCE_SURVIVOR_BONUS_FIRST_HUMAN_SCREEN: u16 = 0x3CA0;
const SOURCE_SURVIVOR_BONUS_HUMAN_STEP: u8 = 0x04;
const SOURCE_SURVIVOR_BONUS_HUMAN_LIMIT: usize = 10;
const SOURCE_SURVIVOR_BONUS_HUMAN_SIZE: [f32; 2] = [4.0, 8.0];
const SOURCE_SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS: u8 = 4;
const SOURCE_SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS: u8 = 0x80;
const SOURCE_ATTRACT_DEFENDER_WORDMARK_DURATION_STEPS: u64 =
    SOURCE_ATTRACT_HALL_OF_FAME_START_STEP - SOURCE_ATTRACT_DEFENDER_WORDMARK_START_STEP;
const SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS: u64 =
    SOURCE_ATTRACT_SCORING_SEQUENCE_START_STEP - SOURCE_ATTRACT_HALL_OF_FAME_START_STEP;
const WILLIAMS_REVEAL_STEPS: u16 = SOURCE_ATTRACT_PRESENTS_START_STEP as u16;
const WILLIAMS_COLOR_PERIOD: u16 = 8;
const SOURCE_ATTRACT_WILLIAMS_LOGO_POSITION: Point = Point::new(108, 60);
const SOURCE_ATTRACT_DEFENDER_WORDMARK_POSITION: Point = Point::new(96, 144);
const SOURCE_ATTRACT_CREDIT_LABEL_POSITION: Point = Point::new(176, 226);
const SOURCE_ATTRACT_CREDIT_COUNT_POSITION: Point = Point::new(248, 226);
const SOURCE_ATTRACT_HALL_TITLE_LABEL: &str = "HALLD_TITLE";
const SOURCE_ATTRACT_HALL_TODAYS_LABEL: &str = "HALLD_TODAYS";
const SOURCE_ATTRACT_HALL_ALL_TIME_LABEL: &str = "HALLD_ALL_TIME";
const SOURCE_ATTRACT_HALL_GREATEST_LABEL: &str = "HALLD_GREATEST";
const SOURCE_ATTRACT_HALL_DEFENDER_LOGO_POSITION: Point = Point::new(85, 50);
const SOURCE_ATTRACT_HALL_TODAYS_TABLE_SCREEN: u16 = 0x1886;
const SOURCE_ATTRACT_HALL_ALL_TIME_TABLE_SCREEN: u16 = 0x5986;
const SOURCE_ATTRACT_HALL_TABLE_ROW_STEP: u8 = 0x0A;
const SOURCE_ATTRACT_HALL_TABLE_INITIALS_OFFSET: u8 = 0x05;
const SOURCE_ATTRACT_HALL_TABLE_SCORE_OFFSET: u8 = 0x13;
const SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET: Point = Point::new(-11, -6);
const SOURCE_ATTRACT_HALL_SCORE_TEXT_LEN: usize = 6;
const SOURCE_ATTRACT_SCORING_VISUAL_OFFSET: Point = Point::new(-11, -7);
const SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES: [(&str, u16); 7] = [
    ("SCANV", 0x4330),
    ("LANDV", 0x1C70),
    ("MUTV", 0x3C70),
    ("BAITV", 0x5F70),
    ("BOMBV", 0x1CA8),
    ("SWRMPV", 0x40A8),
    ("SWARMV", 0x5CA8),
];
const SOURCE_PLAYER_START_PROMPT_SCREEN: u16 = 0x3C80;
const SOURCE_PLAYER_SWITCH_LABEL_SCREEN: u16 = 0x3C78;
const SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN: u16 = 0x3E88;
const SOURCE_TOP_DISPLAY_BORDER_SEGMENTS: [(u16, [f32; 2]); 6] = [
    (0x0028, [312.0, 2.0]),
    (0x2F08, [2.0, 32.0]),
    (0x7008, [2.0, 32.0]),
    (0x2F07, [130.0, 1.0]),
    (0x4C07, [16.0, 2.0]),
    (0x4C28, [16.0, 2.0]),
];
const ATTRACT_SCORING_SCANNER_TERRAIN_PIXEL_SIZE: [f32; 2] = [1.0, 1.0];
const ATTRACT_SCORING_SCANNER_TERRAIN_TINT: Color = Color::from_rgba(174, 81, 0, 255);
const ATTRACT_SCORING_SCANNER_BORDER_TINT: Color = Color::from_rgba(38, 0, 160, 255);
const ATTRACT_SCORING_RESCUE_DESCENT_STEPS: u16 = 0xE6;
const ATTRACT_SCORING_RESCUE_ASCENT_STEPS: u16 = 0xA0;
const ATTRACT_SCORING_RESCUE_LASER_STEPS: u16 = 0x15;
const ATTRACT_SCORING_RESCUE_FALL_STEPS: u16 = 0x2D * 2;
const ATTRACT_SCORING_RESCUE_SCORE_STEPS: u16 = 0x50;
const ATTRACT_SCORING_RESCUE_RETURN_STEPS: u16 = 0x60;
const ATTRACT_SCORING_LEGEND_APPROACH_STEPS: u16 = 0x5F;
const ATTRACT_SCORING_LEGEND_LASER_STEPS: u16 = 0x17;
const ATTRACT_SCORING_LEGEND_TRANSFER_STEPS: u16 = 0x20;
const ATTRACT_SCORING_LEGEND_REVEAL_STEPS: u16 = 0x20;
const ATTRACT_SCORING_LEGEND_ENTRY_STEPS: u16 = ATTRACT_SCORING_LEGEND_APPROACH_STEPS
    + ATTRACT_SCORING_LEGEND_LASER_STEPS
    + ATTRACT_SCORING_LEGEND_TRANSFER_STEPS
    + ATTRACT_SCORING_LEGEND_REVEAL_STEPS;
const ATTRACT_SCORING_LEGEND_HOLD_STEPS: u16 = 0xFF + 0xFF;
const ATTRACT_SCORING_LEGEND_ENTRIES: u16 = 6;
const ATTRACT_SCORING_RESCUE_SEQUENCE_STEPS: u16 = ATTRACT_SCORING_RESCUE_DESCENT_STEPS
    + ATTRACT_SCORING_RESCUE_ASCENT_STEPS
    + ATTRACT_SCORING_RESCUE_LASER_STEPS
    + ATTRACT_SCORING_RESCUE_FALL_STEPS
    + ATTRACT_SCORING_RESCUE_SCORE_STEPS
    + ATTRACT_SCORING_RESCUE_RETURN_STEPS;
const ATTRACT_SCORING_DEMO_TOTAL_STEPS: u16 = ATTRACT_SCORING_RESCUE_SEQUENCE_STEPS
    + ATTRACT_SCORING_LEGEND_ENTRY_STEPS * ATTRACT_SCORING_LEGEND_ENTRIES
    + ATTRACT_SCORING_LEGEND_HOLD_STEPS;
const ATTRACT_SCORING_PROTECTED_DEMO_STEP_OFFSET: u16 = ATTRACT_SCORING_RESCUE_DESCENT_STEPS
    + ATTRACT_SCORING_RESCUE_ASCENT_STEPS
    + ATTRACT_SCORING_RESCUE_LASER_STEPS;
const ATTRACT_SCORING_PLAYER_X16: i32 = 0x0800;
const ATTRACT_SCORING_PLAYER_Y16: i32 = 0x5000;
const ATTRACT_SCORING_HUMAN_X16: i32 = 0x1E00;
const ATTRACT_SCORING_HUMAN_Y16: i32 = 0xDB00;
const ATTRACT_SCORING_LANDER_X16: i32 = 0x1DA0;
const ATTRACT_SCORING_LANDER_Y16: i32 = 0x4000;
const ATTRACT_SCORING_SCORE_500_X16: i32 = 0x1DFF;
const ATTRACT_SCORING_SCORE_500_Y16: i32 = 0x9000;
const ATTRACT_SCORING_SCORE_500_DROP_X16: i32 = 0x1C00;
const ATTRACT_SCORING_SCORE_500_DROP_Y16: i32 = 0xE000;
const ATTRACT_SCORING_CAUGHT_HUMAN_X16: i32 = 0x1E80;
const ATTRACT_SCORING_CAUGHT_HUMAN_Y16: i32 = 0xA2E0;
const ATTRACT_SCORING_GROUNDED_HUMAN_Y16: i32 = 0xDEE0;
const ATTRACT_SCORING_RESCUE_SHIP_XV16: i32 = 0x0040;
const ATTRACT_SCORING_RESCUE_SHIP_YV16: i32 = 0x00D4;
const ATTRACT_SCORING_RESCUE_HUMAN_ACCEL16: i32 = 0x0008;
const ATTRACT_SCORING_RESCUE_DROP_YV16: i32 = 0x00C0;
const ATTRACT_SCORING_RESCUE_RETURN_XV16: i32 = -0x0040;
const ATTRACT_SCORING_RESCUE_RETURN_YV16: i32 = -0x0180;
const ATTRACT_SCORING_OBJECT_REFERENCE_OFFSET: [f32; 2] = [-15.0, -10.0];
const ATTRACT_SCORING_PLAYFIELD_SIZE: [f32; 2] = [320.0, 256.0];
const ATTRACT_SCORING_SCANNER_ORIGIN: [f32; 2] = [84.0, 0.0];
const ATTRACT_SCORING_SCANNER_SIZE: [f32; 2] = [128.0, 32.0];
const ATTRACT_SCORING_PLAYER_SCANNER_SIZE: [f32; 2] = [3.0, 2.0];
const ATTRACT_SCORING_OBJECT_SCANNER_SIZE: [f32; 2] = [2.0, 2.0];
const ATTRACT_SCORING_PLAYER_SCANNER_COLOR_WORD: u16 = 0x9099;
const ATTRACT_SCORING_HUMAN_SCANNER_COLOR_WORD: u16 = 0x6666;
const ATTRACT_SCORING_LANDER_SCANNER_COLOR_WORD: u16 = 0x4433;
const ATTRACT_SCORING_LEGEND_SOURCE_X16: i32 = 0x1F00;
const ATTRACT_SCORING_LEGEND_SOURCE_START_Y16: i32 = 0xA000;
const SOURCE_OBJECT_IMAGES_TSV: &str = include_str!("../assets/red-label/object-images.tsv");
const SOURCE_NORMAL_PALETTE_BYTES: [u8; 16] = [
    0x00, 0x00, 0x07, 0x28, 0x2F, 0x81, 0xA4, 0x15, 0xC7, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const SOURCE_LASER_BYTE_PIXELS: i32 = 2;
const SOURCE_LASER_BODY_BYTE: u8 = 0x11;
const SOURCE_LASER_TIP_BYTE: u8 = 0x99;
const SOURCE_LASER_BODY_CELLS: i32 = 4;
const SOURCE_LASER_BODY_TINT: Color = Color::from_rgba(0x00, 0xB8, 0xFF, 0xFF);
const SOURCE_LASER_TIP_TINT: Color = Color::from_rgba(0x8F, 0xE8, 0xFF, 0xFF);
const SOURCE_LASER_FIZZLE_TINT: Color = Color::from_rgba(0x00, 0x78, 0xD8, 0xFF);
const SOURCE_WILLIAMS_RED_GREEN_LEVELS: [u8; 8] = [0, 38, 81, 118, 137, 174, 217, 255];
const SOURCE_WILLIAMS_BLUE_LEVELS: [u8; 4] = [0, 95, 160, 255];
const SOURCE_TERRAIN_DATA_TSV: &str = include_str!("../assets/red-label/terrain-data.tsv");
const SOURCE_TERRAIN_MTERR_LABEL: &str = "MTERR";
const SOURCE_TERRAIN_MTERR_ADDRESS: u16 = 0xCD67;
const SOURCE_TERRAIN_MTERR_BYTES: usize = 0x180;
const SOURCE_SCANNER_TERRAIN_RECORDS: usize = 0x40;
const SOURCE_SCANNER_MINI_TERRAIN_RECORDS: usize = SOURCE_TERRAIN_MTERR_BYTES / 3;
const SOURCE_SCANNER_OBJECT_BASE_SCREEN: u16 = 0x3008;
const SOURCE_SCANNER_SCAN_CENTER_OFFSET: u16 = 0x6D40;
const DEFENDER_WORDMARK_START_STEP: u64 = SOURCE_ATTRACT_DEFENDER_WORDMARK_START_STEP;
const DEFENDER_WORDMARK_SLOTS: u16 = 15;
const DEFENDER_WORDMARK_ROW_PAIRS: u16 = 6;
const SOURCE_ATTRACT_DEFENDER_APPEARANCE_FINAL_TICK: u8 = 0x2E;
const WILLIAMS_LOGO_SCENE_SIZE: [f32; 2] = [92.0, 19.0];
const DEFENDER_WORDMARK_SCENE_SIZE: [f32; 2] = [120.0, 24.0];
const PLAYER_SHIP_SCENE_SIZE: [f32; 2] = [16.0, 6.0];
const PLAYER_PROJECTILE_SCENE_SIZE: [f32; 2] = [16.0, 1.0];
const LANDER_SCENE_SIZE: [f32; 2] = [10.0, 8.0];
const HUMAN_SCENE_SIZE: [f32; 2] = [4.0, 8.0];
const MUTANT_SCENE_SIZE: [f32; 2] = [10.0, 8.0];
const BAITER_SCENE_SIZE: [f32; 2] = [12.0, 4.0];
const BOMBER_SCENE_SIZE: [f32; 2] = [8.0, 8.0];
const POD_SCENE_SIZE: [f32; 2] = [8.0, 8.0];
const SWARMER_SCENE_SIZE: [f32; 2] = [6.0, 4.0];
const ENEMY_BOMB_SCENE_SIZE: [f32; 2] = [4.0, 3.0];
const EXPLOSION_SCENE_SIZE: [f32; 2] = [8.0, 8.0];
const PLAYER_EXPLOSION_PIXEL_SCENE_SIZE: [f32; 2] = [1.0, 1.0];
const SCORE_POPUP_SCENE_SIZE: [f32; 2] = [12.0, 6.0];
const HUMAN_GROUND_Y: i16 = 214;
const HUMAN_FALL_ACCELERATION: i16 = 1;
const HUMAN_MAX_FALL_SPEED: i16 = 8;
const HUMAN_SAFE_LANDING_SPEED: i16 = 3;
const HUMAN_CARRIED_OFFSET_Y: i16 = 8;
const SOURCE_ASTRO_RESTORE_Y: u8 = 0xE0;
const SOURCE_ASTRONAUT_TARGET_CURSOR_ENTRY_COUNT: usize = 16;
const SOURCE_ASTRONAUT_PROCESS_SLEEP_TICKS: u8 = 2;
const SOURCE_HUMAN_TURN_SEED_MAX: u8 = 8;
const SOURCE_HUMAN_LEFT_TARGET_Y_OFFSET: u8 = 4;
const SOURCE_HUMAN_RIGHT_TARGET_Y_OFFSET: u8 = 15;
const SOURCE_HUMAN_MAX_TARGET_Y: u8 = 0xE8;
const SOURCE_HUMAN_LEFT_X_VELOCITY: u16 = 0xFFE0;
const SOURCE_HUMAN_RIGHT_X_VELOCITY: u16 = 0x0020;
const SOURCE_INITIAL_POD_X_SPEED: u8 = 0x20;
const SOURCE_BOMBER_SQUAD_SIZE: usize = 4;
const SOURCE_POD_SWARMER_REQUEST_LIMIT: usize = 6;
const SOURCE_ACTIVE_SWARMER_LIMIT: usize = 20;
const SOURCE_ACTIVE_BAITER_LIMIT: usize = 12;
const SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS: u8 = 3;
const SOURCE_MINI_SWARMER_MAX_Y_VELOCITY: u16 = 0x0200;
const SOURCE_MINI_SWARMER_MIN_Y_VELOCITY: u16 = 0xFE00;
const SOURCE_MINI_SWARMER_TURN_WINDOW: u16 = 300 * 32;
const SOURCE_MINI_SWARMER_TURN_WINDOW_HALF: u16 = 150 * 32;
const SOURCE_MINI_SWARMER_RESTORE_X_LOW: u8 = 0x07;
const SOURCE_BAITER_INITIAL_SHOT_TIMER: u8 = 8;
const SOURCE_BAITER_LOOP_SLEEP_TICKS: u8 = 6;
const SOURCE_BAITER_X_SEEK_SPEED: u8 = 0x40;
const SOURCE_BAITER_Y_SEEK_BYTE: u8 = 0x01;
const SOURCE_BAITER_X_SEEK_WINDOW_HALF_PIXELS: i16 = 20;
const SOURCE_BAITER_Y_SEEK_WINDOW_HALF_PIXELS: i16 = 10;
const SOURCE_BAITER_PICTURE_FRAME_COUNT: u8 = 3;
const ACTOR_BAITER_TIMER_PACING_STEPS: u8 = 15;
const SOURCE_BOMBER_LOOP_SLEEP_TICKS: u8 = 1;
const SOURCE_BOMBER_PICTURE_FRAME_COUNT: u8 = 4;
const SOURCE_BOMBER_CRUISE_ALTITUDE: i16 = 0x50;
const SOURCE_BOMBER_MIN_CRUISE_ALTITUDE: i16 = 0x40;
const SOURCE_BOMBER_MAX_CRUISE_ALTITUDE: i16 = 0x68;
const SOURCE_BOMBER_CRUISE_WINDOW_HALF_PIXELS: i16 = 0x10;
const SOURCE_ACTIVE_BOMBER_BOMB_LIMIT: usize = 10;
const SOURCE_MAX_ACTIVE_WAVE_ENEMIES: usize = 5;
const SOURCE_START_HUMAN_COUNT: u8 = 10;
const SOURCE_TARGET_LIST_ENTRY_COUNT: usize = 32;
const STATUS_SCORE_POSITION: Point = Point::new(8, 6);
const STATUS_HIGH_SCORE_POSITION: Point = Point::new(94, 6);
const STATUS_PLAYER_TWO_SCORE_POSITION: Point = Point::new(208, 6);
const STATUS_WAVE_POSITION: Point = Point::new(8, 18);
const STATUS_LIVES_POSITION: Point = Point::new(86, 18);
const STATUS_SMART_BOMBS_POSITION: Point = Point::new(140, 18);
const STATUS_CREDITS_POSITION: Point = Point::new(176, 226);
const SOURCE_CREDITS_MESSAGE_LABEL: &str = "CREDV";
const SOURCE_PRESENTS_MESSAGE_LABEL: &str = "ELECV";
const SOURCE_ATTRACT_PRESENTS_ELECTRONICS_SCREEN: u16 = 0x3258;
const STATUS_FINAL_SCORE_POSITION: Point = Point::new(56, 92);
const STATUS_HIGH_SCORE_TABLE_TITLE_POSITION: Point = Point::new(78, 112);
const STATUS_HIGH_SCORE_TABLE_START_Y: i16 = 128;
const STATUS_HIGH_SCORE_TABLE_ROW_HEIGHT: i16 = 12;
const LANDER_SEEK_SPEED: i16 = 1;
const LANDER_DRIFT_SPEED: i16 = 1;
const LANDER_CARRY_SPEED: i16 = 2;
const LANDER_PICKUP_RADIUS_X: i16 = 6;
const LANDER_PICKUP_RADIUS_Y: i16 = 8;
const LANDER_CONVERSION_Y: i16 = 24;
const MUTANT_SEEK_SPEED: i16 = 1;
const MUTANT_SHOT_LIFETIME: u16 = 90;
const SOURCE_MUTANT_LOOP_SLEEP_TICKS: u8 = 2;
const SOURCE_MUTANT_X_DISTANCE_OFFSET: u16 = 380;
const SOURCE_MUTANT_CLOSE_X_WINDOW: u16 = 0x0700;
const SOURCE_MUTANT_VERTICAL_WINDOW: u8 = 8;
const SOURCE_FIRST_WAVE_RESCUE_AIM_PLAYER_MIN_Y: i16 = 0xA0;
const SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION: u16 = 0x0120;
const SOURCE_TARGET6_MUTANT_DEFERRED_SHOT_TIMER: u8 = 5;
const SOURCE_TARGET6_MUTANT_POST_SHOT_TIMER: u8 = 0x2C;
const SOURCE_TARGET6_MUTANT_FIRE2524_PENDING_SHOT_TIMER: u8 = 0xFE;
const SOURCE_TARGET6_MUTANT_DIVE_ENTRY_RAW: (u16, u16) = (0x037C, 0x3380);
const SOURCE_TARGET6_MUTANT_DIVE_FIRST_SHOT_RAW: (u16, u16) = (0x088C, 0x61B0);
const SOURCE_TARGET6_MUTANT_DIVE_SECOND_SHOT_RAW: (u16, u16) = (0x07FC, 0x7800);
const SOURCE_TARGET6_MUTANT_FIRE2524_FIRST_SHOT_RAW: (u16, u16) = (0x082C, 0x5160);
const SOURCE_TARGET6_MUTANT_FIRE2524_SECOND_SHOT_RAW: (u16, u16) = (0x07FC, 0x8150);
const SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MIN: u16 = 0xA400;
const SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MAX: u16 = 0xA600;
const SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_TOP_LEFT: Point = Point::new(0x20, 0xA2);
const SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_CENTER: Point = Point::new(0x21, 0xA9);
const SOURCE_TARGET6_MUTANT_VISUAL_X_CORRECTION: u16 = 0x0168;
const SOURCE_OBJECT_SCREEN_X_SHIFT: u8 = 6;
const SOURCE_OBJECT_VISIBLE_WIDTH: u16 = 292;
const BOMBER_DRIFT_SPEED: i16 = 1;
const BOMBER_BOMB_PERIOD: u64 = 64;
const POD_DRIFT_SPEED: i16 = 1;
const SWARMER_SEEK_SPEED: i16 = 2;
const SWARMER_FIRE_PERIOD: u64 = 58;
const SWARMER_SHOT_SPEED: i16 = 3;
const BAITER_SEEK_SPEED: i16 = 3;
const BAITER_FIRE_PERIOD: u64 = 42;
const BAITER_SHOT_SPEED: i16 = 4;
const BOMB_LIFETIME: u16 = 96;
const LANDER_SCORE: u32 = 150;
const MUTANT_SCORE: u32 = 150;
const BOMBER_SCORE: u32 = 250;
const POD_SCORE: u32 = 1000;
const SWARMER_SCORE: u32 = 150;
const BAITER_SCORE: u32 = 200;
const HUMAN_RESCUE_SCORE: u32 = 500;
const HUMAN_SAFE_LANDING_SCORE: u32 = 250;
const ACTOR_RED_LABEL_ATTRACT_SCRIPT: &str =
    include_str!("../assets/red-label/actor-attract.script");
const ACTOR_RED_LABEL_BEHAVIOR_SCRIPT: &str =
    include_str!("../assets/red-label/actor-behavior.script");
const ACTOR_RED_LABEL_WAVE_SCRIPT: &str = include_str!("../assets/red-label/actor-waves.script");
const ACTOR_SOURCE_WAVE_TABLE_TSV: &str = include_str!("../assets/red-label/wave-table.tsv");
const ACTOR_SOURCE_HIGH_SCORES_TSV: &str = include_str!("../assets/red-label/high-scores.tsv");
const ACTOR_SOURCE_WAVE_TABLE_HEADER: &str =
    "key\tceiling\tfloor\tintra_delta\tinter_delta\twave1\twave2\twave3\twave4";
const ACTOR_SOURCE_DEFAULT_DIFFICULTY_INITIAL: u8 = 5;
const ACTOR_SOURCE_DEFAULT_DIFFICULTY_CEILING: u8 = 15;
const ACTOR_SOURCE_BACKED_WAVES: u16 = 16;
const ACTOR_SOURCE_FIRST_WAVE_HUMAN_SPAWNS: [ActorHumanSpawn; 10] = [
    ActorHumanSpawn::source_first_wave(
        0,
        ActorSourceFirstWaveHumanStart {
            x16: 0x18C3,
            y16: 0xE000,
            picture_frame: 2,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        1,
        ActorSourceFirstWaveHumanStart {
            x16: 0x1C81,
            y16: 0xE100,
            picture_frame: 3,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        2,
        ActorSourceFirstWaveHumanStart {
            x16: 0x4E30,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        3,
        ActorSourceFirstWaveHumanStart {
            x16: 0x5718,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        4,
        ActorSourceFirstWaveHumanStart {
            x16: 0x9B8C,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        5,
        ActorSourceFirstWaveHumanStart {
            x16: 0x9DC6,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        6,
        ActorSourceFirstWaveHumanStart {
            x16: 0xCEE3,
            y16: 0xE000,
            picture_frame: 2,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        7,
        ActorSourceFirstWaveHumanStart {
            x16: 0xD771,
            y16: 0xE000,
            picture_frame: 2,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        8,
        ActorSourceFirstWaveHumanStart {
            x16: 0xD2B8,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::source_first_wave(
        9,
        ActorSourceFirstWaveHumanStart {
            x16: 0xE8DC,
            y16: 0xE000,
            picture_frame: 0,
        },
    ),
];
const ACTOR_SOURCE_FIRST_WAVE_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0xFB33,
        y16: 0x2CE0,
        x_velocity: 0xFFDE,
        y_velocity: 0x0070,
        shot_timer: 0x27,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(1),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x3F4A,
        y16: 0x2CE0,
        x_velocity: 0xFFEE,
        y_velocity: 0x0070,
        shot_timer: 0x3B,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(2),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x67FF,
        y16: 0x2C70,
        x_velocity: 0x0012,
        y_velocity: 0x0070,
        shot_timer: 0x23,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(3),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x0D11,
        y16: 0x2C70,
        x_velocity: 0x0014,
        y_velocity: 0x0070,
        shot_timer: 0x3C,
        sleep_ticks: 0x04,
        picture_frame: 0,
        target_human_index: Some(4),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x41B9,
        y16: 0x2C70,
        x_velocity: 0x001A,
        y_velocity: 0x0070,
        shot_timer: 0x25,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(5),
    }),
];
const ACTOR_SOURCE_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x689A,
        y16: 0x2C70,
        x_velocity: 0x001E,
        y_velocity: 0x0070,
        shot_timer: 0x10,
        sleep_ticks: 0,
        picture_frame: 1,
        target_human_index: Some(7),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x43D3,
        y16: 0x2C70,
        x_velocity: 0xFFEC,
        y_velocity: 0x0070,
        shot_timer: 0x3A,
        sleep_ticks: 0,
        picture_frame: 1,
        target_human_index: Some(9),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x1F51,
        y16: 0x2C70,
        x_velocity: 0x0014,
        y_velocity: 0x0070,
        shot_timer: 0x13,
        sleep_ticks: 0,
        picture_frame: 0,
        target_human_index: Some(8),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0xFA03,
        y16: 0x2C70,
        x_velocity: 0x0016,
        y_velocity: 0x0070,
        shot_timer: 0x26,
        sleep_ticks: 0,
        picture_frame: 1,
        target_human_index: Some(7),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0xCF34,
        y16: 0x2CE0,
        x_velocity: 0,
        y_velocity: 0,
        shot_timer: 0x34,
        sleep_ticks: 1,
        picture_frame: 0,
        target_human_index: Some(6),
    }),
];
const ACTOR_SOURCE_FIRST_WAVE_REFILL_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0xBC29,
        y16: 0x2CFD,
        x_velocity: 0x001C,
        y_velocity: 0x0090,
        shot_timer: 0x36,
        sleep_ticks: 6,
        picture_frame: 1,
        target_human_index: Some(7),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0xE14C,
        y16: 0x2CAE,
        x_velocity: 0x000E,
        y_velocity: 0x0090,
        shot_timer: 0x2F,
        sleep_ticks: 0,
        picture_frame: 0,
        target_human_index: Some(4),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x0A63,
        y16: 0x2CF0,
        x_velocity: 0xFFF4,
        y_velocity: 0x0090,
        shot_timer: 0x23,
        sleep_ticks: 1,
        picture_frame: 0,
        target_human_index: Some(3),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x531B,
        y16: 0x2CC0,
        x_velocity: 0xFFF6,
        y_velocity: 0x0090,
        shot_timer: 0x30,
        sleep_ticks: 1,
        picture_frame: 0,
        target_human_index: Some(2),
    }),
    ActorLanderSpawn::source_first_wave(ActorSourceFirstWaveLanderStart {
        x16: 0x98D9,
        y16: 0x2CB8,
        x_velocity: 0x001A,
        y_velocity: 0x0090,
        shot_timer: 0x1F,
        sleep_ticks: 1,
        picture_frame: 0,
        target_human_index: Some(1),
    }),
];
const ACTOR_WAVE_ACTIVE_SPAWN_SLOTS: [Point; SOURCE_MAX_ACTIVE_WAVE_ENEMIES] = [
    Point::new(0xE4, 0x2A),
    Point::new(228, 104),
    Point::new(184, 72),
    Point::new(148, 96),
    Point::new(236, 66),
];
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActorId(u64);

impl ActorId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct XyzzyMode {
    pub active: bool,
    pub auto_fire: bool,
    pub invincible: bool,
    pub overlay_smart_bomb: bool,
}

impl XyzzyMode {
    pub const INACTIVE: Self = Self {
        active: false,
        auto_fire: false,
        invincible: false,
        overlay_smart_bomb: false,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameInput {
    pub coin: bool,
    pub coin_two: bool,
    pub coin_three: bool,
    pub start_one: bool,
    pub start_two: bool,
    pub altitude_up: bool,
    pub altitude_down: bool,
    pub thrust: bool,
    pub reverse: bool,
    pub fire: bool,
    pub smart_bomb: bool,
    pub hyperspace: bool,
    pub service_advance: bool,
    pub high_score_reset: bool,
    pub high_score_initial: Option<char>,
    pub high_score_backspace: bool,
    pub auto_up_manual_down: bool,
    pub tilt: bool,
    pub xyzzy: XyzzyMode,
}

impl GameInput {
    pub const NONE: Self = Self {
        coin: false,
        coin_two: false,
        coin_three: false,
        start_one: false,
        start_two: false,
        altitude_up: false,
        altitude_down: false,
        thrust: false,
        reverse: false,
        fire: false,
        smart_bomb: false,
        hyperspace: false,
        service_advance: false,
        high_score_reset: false,
        high_score_initial: None,
        high_score_backspace: false,
        auto_up_manual_down: false,
        tilt: false,
        xyzzy: XyzzyMode::INACTIVE,
    };

    pub const fn from_clean_input(input: CleanGameInput, xyzzy: XyzzyMode) -> Self {
        Self {
            coin: input.coin,
            coin_two: input.coin_two,
            coin_three: input.coin_three,
            start_one: input.start_one,
            start_two: input.start_two,
            altitude_up: input.altitude_up,
            altitude_down: input.altitude_down,
            thrust: input.thrust,
            reverse: input.reverse,
            fire: input.fire,
            smart_bomb: input.smart_bomb,
            hyperspace: input.hyperspace,
            service_advance: input.service_advance,
            high_score_reset: input.high_score_reset,
            high_score_initial: input.high_score_initial,
            high_score_backspace: input.high_score_backspace,
            auto_up_manual_down: input.service_auto_up,
            tilt: input.tilt,
            xyzzy,
        }
    }

    fn coin_insertions(self) -> u8 {
        u8::from(self.coin) + u8::from(self.coin_two) + u8::from(self.coin_three)
    }

    fn wants_fire(self) -> bool {
        self.fire || self.xyzzy.auto_fire
    }

    fn wants_stock_smart_bomb(self) -> bool {
        self.smart_bomb && !self.xyzzy.overlay_smart_bomb
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KeyboardProfile {
    #[default]
    Planetoid,
    Cabinet,
}

impl KeyboardProfile {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "planetoid" => Some(Self::Planetoid),
            "cabinet" => Some(Self::Cabinet),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Planetoid => "planetoid",
            Self::Cabinet => "cabinet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardKey {
    Character(char),
    Enter,
    Backspace,
    Escape,
    Tab,
    ArrowUp,
    ArrowDown,
    Function(u8),
    LeftShift,
    RightShift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyTransition {
    Press,
    Repeat,
    Release,
}

impl KeyTransition {
    const fn contributes_input(self) -> bool {
        matches!(self, Self::Press | Self::Repeat)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardEvent {
    pub key: KeyboardKey,
    pub transition: KeyTransition,
}

impl KeyboardEvent {
    pub const fn press(key: KeyboardKey) -> Self {
        Self {
            key,
            transition: KeyTransition::Press,
        }
    }

    pub const fn release(key: KeyboardKey) -> Self {
        Self {
            key,
            transition: KeyTransition::Release,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyboardPoll {
    pub input: GameInput,
    pub typed_chars: Vec<char>,
    pub quit_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct HeldControls {
    altitude_up: bool,
    altitude_down: bool,
    thrust: bool,
    auto_up_manual_down: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardMapper {
    profile: KeyboardProfile,
    held: HeldControls,
    xyzzy: XyzzyController,
}

impl KeyboardMapper {
    pub fn new(profile: KeyboardProfile) -> Self {
        Self {
            profile,
            held: HeldControls::default(),
            xyzzy: XyzzyController::default(),
        }
    }

    pub fn profile(&self) -> KeyboardProfile {
        self.profile
    }

    pub fn xyzzy_mode(&self) -> XyzzyMode {
        self.xyzzy.mode(false)
    }

    pub fn map_event(&mut self, event: KeyboardEvent, step: &mut KeyboardPoll) {
        let active = event.transition.contributes_input();
        if event.transition == KeyTransition::Press {
            match event.key {
                KeyboardKey::Character(character) => {
                    let typed = character.to_ascii_lowercase();
                    step.typed_chars.push(typed);
                    self.xyzzy.ingest(typed);
                }
                KeyboardKey::Backspace => step.typed_chars.push('\u{8}'),
                _ => {}
            }
        }

        match event.key {
            KeyboardKey::Escape if active => step.quit_requested = true,
            KeyboardKey::Character('q' | 'Q') if active => step.quit_requested = true,
            _ => self.map_profile_event(event, step),
        }

        if self.xyzzy.active() && active {
            match event.key {
                KeyboardKey::Character('f' | 'F') => self.xyzzy.toggle_auto_fire(),
                KeyboardKey::Character('g' | 'G') => self.xyzzy.toggle_invincible(),
                KeyboardKey::Tab => step.input.xyzzy.overlay_smart_bomb = true,
                _ => {}
            }
        }
    }

    pub fn finish_poll(&self, step: &mut KeyboardPoll) {
        step.input.altitude_up |= self.held.altitude_up;
        step.input.altitude_down |= self.held.altitude_down;
        step.input.thrust |= self.held.thrust;
        step.input.auto_up_manual_down |= self.held.auto_up_manual_down;
        let overlay_smart_bomb = step.input.xyzzy.overlay_smart_bomb;
        step.input.xyzzy = self.xyzzy.mode(overlay_smart_bomb);
        if step.input.xyzzy.auto_fire {
            step.input.fire = true;
        }
    }

    fn map_profile_event(&mut self, event: KeyboardEvent, step: &mut KeyboardPoll) {
        match self.profile {
            KeyboardProfile::Planetoid => self.map_planetoid_event(event, step),
            KeyboardProfile::Cabinet => self.map_cabinet_event(event, step),
        }
    }

    fn map_planetoid_event(&mut self, event: KeyboardEvent, step: &mut KeyboardPoll) {
        let active = event.transition.contributes_input();
        match event.key {
            KeyboardKey::Enter if active => {
                step.input.start_one = true;
                step.input.fire = true;
            }
            KeyboardKey::Character('1') if active => step.input.start_one = true,
            KeyboardKey::Character('5') if active => step.input.coin = true,
            KeyboardKey::Character('6') if active => step.input.coin_two = true,
            KeyboardKey::Character('7') if active => step.input.coin_three = true,
            KeyboardKey::Character('a' | 'A') => set_held_control(
                &mut self.held.altitude_up,
                event.transition,
                &mut step.input.altitude_up,
            ),
            KeyboardKey::Character('z' | 'Z') => set_held_control(
                &mut self.held.altitude_down,
                event.transition,
                &mut step.input.altitude_down,
            ),
            KeyboardKey::LeftShift | KeyboardKey::RightShift => set_held_control(
                &mut self.held.thrust,
                event.transition,
                &mut step.input.thrust,
            ),
            KeyboardKey::Character(' ') if active => step.input.reverse = true,
            KeyboardKey::Tab if active => step.input.smart_bomb = true,
            KeyboardKey::Character('h' | 'H') if active => step.input.hyperspace = true,
            KeyboardKey::Function(2) if active => step.input.service_advance = true,
            KeyboardKey::Function(3) if active => step.input.high_score_reset = true,
            KeyboardKey::Function(4) => set_held_control(
                &mut self.held.auto_up_manual_down,
                event.transition,
                &mut step.input.auto_up_manual_down,
            ),
            KeyboardKey::Function(5) if active => step.input.tilt = true,
            _ => {}
        }
    }

    fn map_cabinet_event(&mut self, event: KeyboardEvent, step: &mut KeyboardPoll) {
        let active = event.transition.contributes_input();
        match event.key {
            KeyboardKey::Character('5') if active => step.input.coin = true,
            KeyboardKey::Character('6') if active => step.input.coin_two = true,
            KeyboardKey::Character('7') if active => step.input.coin_three = true,
            KeyboardKey::Character('1') if active => step.input.start_one = true,
            KeyboardKey::Character('2') if active => step.input.start_two = true,
            KeyboardKey::ArrowUp => set_held_control(
                &mut self.held.altitude_up,
                event.transition,
                &mut step.input.altitude_up,
            ),
            KeyboardKey::ArrowDown => set_held_control(
                &mut self.held.altitude_down,
                event.transition,
                &mut step.input.altitude_down,
            ),
            KeyboardKey::Character('r' | 'R') if active => step.input.reverse = true,
            KeyboardKey::Character('t' | 'T') => set_held_control(
                &mut self.held.thrust,
                event.transition,
                &mut step.input.thrust,
            ),
            KeyboardKey::Character('f' | 'F') if active => step.input.fire = true,
            KeyboardKey::Character('b' | 'B') if active => step.input.smart_bomb = true,
            KeyboardKey::Character('h' | 'H') if active => step.input.hyperspace = true,
            KeyboardKey::Function(2) if active => step.input.service_advance = true,
            KeyboardKey::Function(3) if active => step.input.high_score_reset = true,
            KeyboardKey::Function(4) => set_held_control(
                &mut self.held.auto_up_manual_down,
                event.transition,
                &mut step.input.auto_up_manual_down,
            ),
            KeyboardKey::Function(5) if active => step.input.tilt = true,
            _ => {}
        }
    }
}

impl Default for KeyboardMapper {
    fn default() -> Self {
        Self::new(KeyboardProfile::default())
    }
}

fn set_held_control(held: &mut bool, transition: KeyTransition, output: &mut bool) {
    match transition {
        KeyTransition::Press | KeyTransition::Repeat => {
            *held = true;
            *output = true;
        }
        KeyTransition::Release => *held = false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XyzzyController {
    active: bool,
    sequence_index: usize,
    auto_fire: bool,
    invincible: bool,
}

impl XyzzyController {
    const CODE: [char; 5] = ['x', 'y', 'z', 'z', 'y'];

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn auto_fire(&self) -> bool {
        self.auto_fire
    }

    pub fn invincible(&self) -> bool {
        self.invincible
    }

    pub fn ingest(&mut self, character: char) {
        let character = character.to_ascii_lowercase();
        if character == Self::CODE[self.sequence_index] {
            self.sequence_index += 1;
            if self.sequence_index == Self::CODE.len() {
                self.active = !self.active;
                self.sequence_index = 0;
                if !self.active {
                    self.auto_fire = false;
                    self.invincible = false;
                }
            }
        } else {
            self.sequence_index = usize::from(character == Self::CODE[0]);
        }
    }

    pub fn toggle_auto_fire(&mut self) {
        if self.active {
            self.auto_fire = !self.auto_fire;
        }
    }

    pub fn toggle_invincible(&mut self) {
        if self.active {
            self.invincible = !self.invincible;
        }
    }

    pub fn mode(&self, overlay_smart_bomb: bool) -> XyzzyMode {
        XyzzyMode {
            active: self.active,
            auto_fire: self.active && self.auto_fire,
            invincible: self.active && self.invincible,
            overlay_smart_bomb: self.active && overlay_smart_bomb,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub const fn offset(self, velocity: Velocity) -> Self {
        Self {
            x: self.x + velocity.dx,
            y: self.y + velocity.dy,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Velocity {
    pub dx: i16,
    pub dy: i16,
}

impl Velocity {
    pub const fn new(dx: i16, dy: i16) -> Self {
        Self { dx, dy }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

impl Rect {
    pub const fn new(left: i16, top: i16, right: i16, bottom: i16) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn from_center(center: Point, width: i16, height: i16) -> Self {
        let half_width = width / 2;
        let half_height = height / 2;
        Self {
            left: center.x - half_width,
            top: center.y - half_height,
            right: center.x + half_width,
            bottom: center.y + half_height,
        }
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.left <= other.right
            && self.right >= other.left
            && self.top <= other.bottom
            && self.bottom >= other.top
    }

    pub const fn clamp_point(self, point: Point) -> Point {
        Point {
            x: clamp_i16(point.x, self.left, self.right),
            y: clamp_i16(point.y, self.top, self.bottom),
        }
    }
}

const fn clamp_i16(value: i16, min: i16, max: i16) -> i16 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    const fn sign(self) -> i16 {
        match self {
            Self::Left => -1,
            Self::Right => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Attract,
    Playing,
    GameOver,
    HighScoreEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActorKind {
    AttractDirector,
    AttractScript,
    StatusDisplay,
    WilliamsLogo,
    DefenderWordmark,
    Player,
    Lander,
    Mutant,
    Bomber,
    Bomb,
    Pod,
    Swarmer,
    Baiter,
    Human,
    Laser,
    EnemyLaser,
    Explosion,
    ScorePopup,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanderBehaviorMode {
    SeekNearestHuman,
    ChasePlayer,
    Drift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostileMovementMode {
    Drift,
    ChasePlayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorHyperspaceSourceSeed {
    pub seed: u8,
    pub hseed: u8,
    pub lseed: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceRngSnapshot {
    pub seed: u8,
    pub hseed: u8,
    pub lseed: u8,
}

impl ActorSourceRngSnapshot {
    const fn hyperspace_seed(self) -> ActorHyperspaceSourceSeed {
        ActorHyperspaceSourceSeed {
            seed: self.seed,
            hseed: self.hseed,
            lseed: self.lseed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceRng {
    seed: u8,
    hseed: u8,
    lseed: u8,
}

impl ActorSourceRng {
    fn advance(&mut self) -> Self {
        let product_low = self.seed.wrapping_mul(3).wrapping_add(17);
        let mut a = self.lseed >> 3;
        a ^= self.lseed;
        let carry_into_hseed = (a & 0x01) != 0;
        let old_hseed = self.hseed;
        self.hseed = (u8::from(carry_into_hseed) << 7) | (self.hseed >> 1);
        let carry_into_lseed = (old_hseed & 0x01) != 0;
        self.lseed = (u8::from(carry_into_lseed) << 7) | (self.lseed >> 1);
        let (with_lseed, carry) = adc8(product_low, self.lseed, false);
        let (new_seed, _) = adc8(with_lseed, self.hseed, carry);
        self.seed = new_seed;
        *self
    }

    fn advance_rmax(&mut self, max: u8) -> u8 {
        let state = self.advance();
        source_rmax(max, state.seed)
    }

    const fn snapshot(self) -> ActorSourceRngSnapshot {
        ActorSourceRngSnapshot {
            seed: self.seed,
            hseed: self.hseed,
            lseed: self.lseed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBehaviorProfile {
    pub player_speed: i16,
    pub player_laser_cooldown_steps: u8,
    pub player_hyperspace_hidden_steps: u8,
    pub player_hyperspace_rematerialize_x: i16,
    pub player_hyperspace_rematerialize_y: i16,
    pub player_hyperspace_source_seed: Option<ActorHyperspaceSourceSeed>,
    pub player_hyperspace_death_delay_steps: u8,
    pub player_hyperspace_death_lseed: u8,
    pub player_takes_enemy_collision_damage: bool,
    pub laser_speed: i16,
    pub laser_lifetime_steps: u16,
    pub lander_seek_speed: i16,
    pub lander_drift_speed: i16,
    pub lander_carry_speed: i16,
    pub lander_pickup_radius_x: i16,
    pub lander_pickup_radius_y: i16,
    pub lander_conversion_y: i16,
    pub lander_fire_period_steps: u64,
    pub lander_shot_speed: i16,
    pub lander_shot_lifetime_steps: u16,
    pub lander_mode: LanderBehaviorMode,
    pub mutant_seek_speed: i16,
    pub mutant_shot_lifetime_steps: u16,
    pub mutant_mode: HostileMovementMode,
    pub bomber_drift_speed: i16,
    pub bomber_bomb_period_steps: u64,
    pub bomber_mode: HostileMovementMode,
    pub pod_drift_speed: i16,
    pub pod_mode: HostileMovementMode,
    pub swarmer_seek_speed: i16,
    pub swarmer_fire_period_steps: u64,
    pub swarmer_shot_speed: i16,
    pub swarmer_mode: HostileMovementMode,
    pub baiter_seek_speed: i16,
    pub baiter_fire_period_steps: u64,
    pub baiter_shot_speed: i16,
    pub baiter_mode: HostileMovementMode,
    pub bomb_lifetime_steps: u16,
    pub human_ground_y: i16,
    pub human_fall_acceleration: i16,
    pub human_max_fall_speed: i16,
    pub human_safe_landing_speed: i16,
    pub human_carried_offset_y: i16,
    pub explosion_lifetime_steps: u16,
    pub score_popup_lifetime_steps: u16,
}

impl ActorBehaviorProfile {
    pub const DEFAULT: Self = Self {
        player_speed: PLAYER_SPEED,
        player_laser_cooldown_steps: PLAYER_LASER_COOLDOWN_STEPS,
        player_hyperspace_hidden_steps: PLAYER_HYPERSPACE_HIDDEN_STEPS,
        player_hyperspace_rematerialize_x: PLAYER_HYPERSPACE_REMATERIALIZE_X,
        player_hyperspace_rematerialize_y: PLAYER_HYPERSPACE_REMATERIALIZE_Y,
        player_hyperspace_source_seed: None,
        player_hyperspace_death_delay_steps: PLAYER_HYPERSPACE_DEATH_DELAY_STEPS,
        player_hyperspace_death_lseed: PLAYER_HYPERSPACE_DEATH_LSEED,
        player_takes_enemy_collision_damage: true,
        laser_speed: LASER_SPEED,
        laser_lifetime_steps: LASER_LIFETIME,
        lander_seek_speed: LANDER_SEEK_SPEED,
        lander_drift_speed: LANDER_DRIFT_SPEED,
        lander_carry_speed: LANDER_CARRY_SPEED,
        lander_pickup_radius_x: LANDER_PICKUP_RADIUS_X,
        lander_pickup_radius_y: LANDER_PICKUP_RADIUS_Y,
        lander_conversion_y: LANDER_CONVERSION_Y,
        lander_fire_period_steps: LANDER_FIRE_PERIOD,
        lander_shot_speed: LANDER_SHOT_SPEED,
        lander_shot_lifetime_steps: LANDER_SHOT_LIFETIME,
        lander_mode: LanderBehaviorMode::SeekNearestHuman,
        mutant_seek_speed: MUTANT_SEEK_SPEED,
        mutant_shot_lifetime_steps: MUTANT_SHOT_LIFETIME,
        mutant_mode: HostileMovementMode::ChasePlayer,
        bomber_drift_speed: BOMBER_DRIFT_SPEED,
        bomber_bomb_period_steps: BOMBER_BOMB_PERIOD,
        bomber_mode: HostileMovementMode::Drift,
        pod_drift_speed: POD_DRIFT_SPEED,
        pod_mode: HostileMovementMode::Drift,
        swarmer_seek_speed: SWARMER_SEEK_SPEED,
        swarmer_fire_period_steps: SWARMER_FIRE_PERIOD,
        swarmer_shot_speed: SWARMER_SHOT_SPEED,
        swarmer_mode: HostileMovementMode::ChasePlayer,
        baiter_seek_speed: BAITER_SEEK_SPEED,
        baiter_fire_period_steps: BAITER_FIRE_PERIOD,
        baiter_shot_speed: BAITER_SHOT_SPEED,
        baiter_mode: HostileMovementMode::ChasePlayer,
        bomb_lifetime_steps: BOMB_LIFETIME,
        human_ground_y: HUMAN_GROUND_Y,
        human_fall_acceleration: HUMAN_FALL_ACCELERATION,
        human_max_fall_speed: HUMAN_MAX_FALL_SPEED,
        human_safe_landing_speed: HUMAN_SAFE_LANDING_SPEED,
        human_carried_offset_y: HUMAN_CARRIED_OFFSET_Y,
        explosion_lifetime_steps: EXPLOSION_LIFETIME,
        score_popup_lifetime_steps: SCORE_POPUP_LIFETIME,
    };

    pub const fn arcade_default() -> Self {
        Self::DEFAULT
    }
}

impl Default for ActorBehaviorProfile {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorBehaviorScript {
    default_profile: ActorBehaviorProfile,
    kind_profiles: BTreeMap<ActorKind, ActorBehaviorProfile>,
    actor_profiles: BTreeMap<ActorId, ActorBehaviorProfile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorKindBehaviorProfile {
    pub kind: ActorKind,
    pub profile: ActorBehaviorProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorInstanceBehaviorProfile {
    pub actor: ActorId,
    pub profile: ActorBehaviorProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorBehaviorScriptManifest {
    pub default_profile: ActorBehaviorProfile,
    pub kind_profiles: Vec<ActorKindBehaviorProfile>,
    pub actor_profiles: Vec<ActorInstanceBehaviorProfile>,
}

impl ActorBehaviorScriptManifest {
    pub fn kind_profile(&self, kind: ActorKind) -> Option<ActorBehaviorProfile> {
        self.kind_profiles
            .iter()
            .find(|entry| entry.kind == kind)
            .map(|entry| entry.profile)
    }

    pub fn actor_profile(&self, actor: ActorId) -> Option<ActorBehaviorProfile> {
        self.actor_profiles
            .iter()
            .find(|entry| entry.actor == actor)
            .map(|entry| entry.profile)
    }

    pub fn behavior_for(&self, actor: ActorId, kind: ActorKind) -> ActorBehaviorProfile {
        self.actor_profile(actor)
            .or_else(|| self.kind_profile(kind))
            .unwrap_or(self.default_profile)
    }
}

impl ActorBehaviorScript {
    pub fn new(default_profile: ActorBehaviorProfile) -> Self {
        Self {
            default_profile,
            kind_profiles: BTreeMap::new(),
            actor_profiles: BTreeMap::new(),
        }
    }

    pub fn from_arcade_profile() -> Self {
        Self::new(ActorBehaviorProfile::DEFAULT)
    }

    pub fn red_label_default() -> Self {
        Self::parse_text(ACTOR_RED_LABEL_BEHAVIOR_SCRIPT)
            .unwrap_or_else(|error| panic!("embedded actor behavior script is invalid: {error}"))
    }

    pub fn default_profile(&self) -> ActorBehaviorProfile {
        self.default_profile
    }

    pub fn set_default_profile(&mut self, profile: ActorBehaviorProfile) {
        self.default_profile = profile;
    }

    pub fn set_kind_behavior(&mut self, kind: ActorKind, profile: ActorBehaviorProfile) {
        self.kind_profiles.insert(kind, profile);
    }

    pub fn set_actor_behavior(&mut self, actor: ActorId, profile: ActorBehaviorProfile) {
        self.actor_profiles.insert(actor, profile);
    }

    pub fn remove_actor_behavior(&mut self, actor: ActorId) {
        self.actor_profiles.remove(&actor);
    }

    pub fn with_kind_behavior(mut self, kind: ActorKind, profile: ActorBehaviorProfile) -> Self {
        self.set_kind_behavior(kind, profile);
        self
    }

    pub fn with_actor_behavior(mut self, actor: ActorId, profile: ActorBehaviorProfile) -> Self {
        self.set_actor_behavior(actor, profile);
        self
    }

    pub fn behavior_for(&self, actor: ActorId, kind: ActorKind) -> ActorBehaviorProfile {
        self.actor_profiles
            .get(&actor)
            .copied()
            .or_else(|| self.kind_profiles.get(&kind).copied())
            .unwrap_or(self.default_profile)
    }

    pub fn manifest(&self) -> ActorBehaviorScriptManifest {
        ActorBehaviorScriptManifest {
            default_profile: self.default_profile,
            kind_profiles: self
                .kind_profiles
                .iter()
                .map(|(kind, profile)| ActorKindBehaviorProfile {
                    kind: *kind,
                    profile: *profile,
                })
                .collect(),
            actor_profiles: self
                .actor_profiles
                .iter()
                .map(|(actor, profile)| ActorInstanceBehaviorProfile {
                    actor: *actor,
                    profile: *profile,
                })
                .collect(),
        }
    }

    fn with_hyperspace_source_seed(&self, seed: ActorHyperspaceSourceSeed) -> Self {
        let mut script = self.clone();
        if script
            .default_profile
            .player_hyperspace_source_seed
            .is_none()
        {
            script.default_profile.player_hyperspace_source_seed = Some(seed);
        }
        for profile in script.kind_profiles.values_mut() {
            if profile.player_hyperspace_source_seed.is_none() {
                profile.player_hyperspace_source_seed = Some(seed);
            }
        }
        script
    }

    fn with_input_overrides(
        &self,
        input: GameInput,
        snapshots: impl Iterator<Item = ActorSnapshot>,
    ) -> Self {
        let mut script = self.clone();
        if input.xyzzy.invincible {
            let mut player_kind_behavior = script.behavior_for(ActorId::new(0), ActorKind::Player);
            player_kind_behavior.player_takes_enemy_collision_damage = false;
            script.set_kind_behavior(ActorKind::Player, player_kind_behavior);

            let actor_ids = script.actor_profiles.keys().copied().collect::<Vec<_>>();
            for actor in actor_ids {
                let mut behavior = script.behavior_for(actor, ActorKind::Player);
                behavior.player_takes_enemy_collision_damage = false;
                script.set_actor_behavior(actor, behavior);
            }

            for snapshot in
                snapshots.filter(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            {
                let mut behavior = script.behavior_for(snapshot.id, ActorKind::Player);
                behavior.player_takes_enemy_collision_damage = false;
                script.set_actor_behavior(snapshot.id, behavior);
            }
        }
        script
    }
}

impl FromStr for ActorBehaviorScript {
    type Err = ActorBehaviorScriptParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let mut script = Self::from_arcade_profile();
        for (line_index, raw_line) in source.lines().enumerate() {
            let line_number = line_index + 1;
            let line = raw_line
                .split_once('#')
                .map_or(raw_line, |(before_comment, _)| before_comment)
                .trim();
            if line.is_empty() {
                continue;
            }
            parse_behavior_script_line(line_number, line, &mut script)?;
        }
        Ok(script)
    }
}

impl ActorBehaviorScript {
    pub fn parse_text(source: &str) -> Result<Self, ActorBehaviorScriptParseError> {
        source.parse()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorBehaviorScriptParseError {
    pub line: usize,
    pub message: String,
}

impl ActorBehaviorScriptParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ActorBehaviorScriptParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "actor behavior script line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for ActorBehaviorScriptParseError {}

fn parse_behavior_script_line(
    line_number: usize,
    line: &str,
    script: &mut ActorBehaviorScript,
) -> Result<(), ActorBehaviorScriptParseError> {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    match tokens.as_slice() {
        ["default", field, values @ ..] => {
            let mut profile = script.default_profile;
            apply_behavior_profile_field(line_number, &mut profile, field, values)?;
            script.set_default_profile(profile);
            Ok(())
        }
        ["kind", kind, field, values @ ..] => {
            let kind = parse_behavior_actor_kind(line_number, kind)?;
            let mut profile = script
                .kind_profiles
                .get(&kind)
                .copied()
                .unwrap_or_else(|| script.behavior_for(ActorId::new(0), kind));
            apply_behavior_profile_field(line_number, &mut profile, field, values)?;
            script.set_kind_behavior(kind, profile);
            Ok(())
        }
        ["actor", actor, field, values @ ..] => {
            let actor = ActorId::new(parse_behavior_u64(line_number, actor, "actor id")?);
            let mut profile = script
                .actor_profiles
                .get(&actor)
                .copied()
                .unwrap_or(script.default_profile);
            apply_behavior_profile_field(line_number, &mut profile, field, values)?;
            script.set_actor_behavior(actor, profile);
            Ok(())
        }
        [scope, ..] => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("unknown profile scope `{scope}`"),
        )),
        [] => Err(ActorBehaviorScriptParseError::new(
            line_number,
            "missing profile scope",
        )),
    }
}

fn apply_behavior_profile_field(
    line_number: usize,
    profile: &mut ActorBehaviorProfile,
    field: &str,
    values: &[&str],
) -> Result<(), ActorBehaviorScriptParseError> {
    let field = normalize_script_token(field);
    match field.as_str() {
        "player_speed" => {
            profile.player_speed = parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "player_laser_cooldown_steps" => {
            profile.player_laser_cooldown_steps =
                parse_behavior_u8_value(line_number, values, field.as_str())?;
        }
        "player_hyperspace_hidden_steps" => {
            profile.player_hyperspace_hidden_steps =
                parse_behavior_u8_value(line_number, values, field.as_str())?;
        }
        "player_hyperspace_rematerialize_x" => {
            profile.player_hyperspace_rematerialize_x =
                parse_behavior_i16_value(line_number, values, field.as_str())?;
        }
        "player_hyperspace_rematerialize_y" => {
            profile.player_hyperspace_rematerialize_y =
                parse_behavior_i16_value(line_number, values, field.as_str())?;
        }
        "player_hyperspace_source_seed" => {
            profile.player_hyperspace_source_seed =
                parse_behavior_hyperspace_seed_value(line_number, values, field.as_str())?;
        }
        "player_hyperspace_death_delay_steps" => {
            profile.player_hyperspace_death_delay_steps =
                parse_behavior_u8_value(line_number, values, field.as_str())?;
        }
        "player_hyperspace_death_lseed" => {
            profile.player_hyperspace_death_lseed =
                parse_behavior_u8_value(line_number, values, field.as_str())?;
        }
        "player_takes_enemy_collision_damage" => {
            profile.player_takes_enemy_collision_damage =
                parse_behavior_bool_value(line_number, values, field.as_str())?;
        }
        "laser_speed" => {
            profile.laser_speed = parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "laser_lifetime_steps" => {
            profile.laser_lifetime_steps =
                parse_behavior_u16_value(line_number, values, field.as_str())?;
        }
        "lander_seek_speed" => {
            profile.lander_seek_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "lander_drift_speed" => {
            profile.lander_drift_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "lander_carry_speed" => {
            profile.lander_carry_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "lander_pickup_radius_x" => {
            profile.lander_pickup_radius_x =
                parse_behavior_i16_value(line_number, values, field.as_str())?;
        }
        "lander_pickup_radius_y" => {
            profile.lander_pickup_radius_y =
                parse_behavior_i16_value(line_number, values, field.as_str())?;
        }
        "lander_conversion_y" => {
            profile.lander_conversion_y =
                parse_behavior_i16_value(line_number, values, field.as_str())?;
        }
        "lander_fire_period_steps" => {
            profile.lander_fire_period_steps =
                parse_behavior_u64_value(line_number, values, field.as_str())?;
        }
        "lander_shot_speed" => {
            profile.lander_shot_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "lander_shot_lifetime_steps" => {
            profile.lander_shot_lifetime_steps =
                parse_behavior_u16_value(line_number, values, field.as_str())?;
        }
        "lander_mode" => {
            profile.lander_mode =
                parse_lander_behavior_mode_value(line_number, values, field.as_str())?;
        }
        "mutant_seek_speed" => {
            profile.mutant_seek_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "mutant_shot_lifetime_steps" => {
            profile.mutant_shot_lifetime_steps =
                parse_behavior_u16_value(line_number, values, field.as_str())?;
        }
        "mutant_mode" => {
            profile.mutant_mode =
                parse_hostile_movement_mode_value(line_number, values, field.as_str())?;
        }
        "bomber_drift_speed" => {
            profile.bomber_drift_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "bomber_bomb_period_steps" => {
            profile.bomber_bomb_period_steps =
                parse_behavior_u64_value(line_number, values, field.as_str())?;
        }
        "bomber_mode" => {
            profile.bomber_mode =
                parse_hostile_movement_mode_value(line_number, values, field.as_str())?;
        }
        "pod_drift_speed" => {
            profile.pod_drift_speed = parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "pod_mode" => {
            profile.pod_mode =
                parse_hostile_movement_mode_value(line_number, values, field.as_str())?;
        }
        "swarmer_seek_speed" => {
            profile.swarmer_seek_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "swarmer_fire_period_steps" => {
            profile.swarmer_fire_period_steps =
                parse_behavior_u64_value(line_number, values, field.as_str())?;
        }
        "swarmer_shot_speed" => {
            profile.swarmer_shot_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "swarmer_mode" => {
            profile.swarmer_mode =
                parse_hostile_movement_mode_value(line_number, values, field.as_str())?;
        }
        "baiter_seek_speed" => {
            profile.baiter_seek_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "baiter_fire_period_steps" => {
            profile.baiter_fire_period_steps =
                parse_behavior_u64_value(line_number, values, field.as_str())?;
        }
        "baiter_shot_speed" => {
            profile.baiter_shot_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "baiter_mode" => {
            profile.baiter_mode =
                parse_hostile_movement_mode_value(line_number, values, field.as_str())?;
        }
        "bomb_lifetime_steps" => {
            profile.bomb_lifetime_steps =
                parse_behavior_u16_value(line_number, values, field.as_str())?
        }
        "human_ground_y" => {
            profile.human_ground_y = parse_behavior_i16_value(line_number, values, field.as_str())?
        }
        "human_fall_acceleration" => {
            profile.human_fall_acceleration =
                parse_behavior_i16_value(line_number, values, field.as_str())?;
        }
        "human_max_fall_speed" => {
            profile.human_max_fall_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?;
        }
        "human_safe_landing_speed" => {
            profile.human_safe_landing_speed =
                parse_behavior_i16_value(line_number, values, field.as_str())?;
        }
        "human_carried_offset_y" => {
            profile.human_carried_offset_y =
                parse_behavior_i16_value(line_number, values, field.as_str())?;
        }
        "explosion_lifetime_steps" => {
            profile.explosion_lifetime_steps =
                parse_behavior_u16_value(line_number, values, field.as_str())?;
        }
        "score_popup_lifetime_steps" => {
            profile.score_popup_lifetime_steps =
                parse_behavior_u16_value(line_number, values, field.as_str())?;
        }
        _ => {
            return Err(ActorBehaviorScriptParseError::new(
                line_number,
                format!("unknown behavior field `{field}`"),
            ));
        }
    }
    Ok(())
}

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

fn parse_behavior_hyperspace_seed_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<Option<ActorHyperspaceSourceSeed>, ActorBehaviorScriptParseError> {
    if values.len() == 1 && normalize_script_token(values[0]) == "none" {
        return Ok(None);
    }
    if values.len() != 3 {
        return Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} needs `none` or three seed bytes"),
        ));
    }
    Ok(Some(ActorHyperspaceSourceSeed {
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
pub struct ActorSourceWaveProfile {
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
pub struct ActorSourceLanderMetadata {
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
pub struct ActorSourceBomberMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub picture_frame: u8,
    pub cruise_altitude: i16,
    pub sleep_ticks: u8,
    pub source_slot: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourcePodMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceSwarmerMetadata {
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
pub struct ActorSourceBaiterMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub picture_frame: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceMutantMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub hop_rng: ActorSourceRngSnapshot,
    pub render_x_correction: u16,
    pub target6_first_shot_deferred: bool,
}

impl ActorSourceMutantMetadata {
    fn from_lander_conversion(
        source_lander: ActorSourceLanderMetadata,
        profile: ActorSourceWaveProfile,
        hop_rng: ActorSourceRngSnapshot,
    ) -> Self {
        Self {
            x_fraction: source_lander.x_fraction,
            y_fraction: source_lander.y_fraction,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8,
            sleep_ticks: 0,
            hop_rng,
            render_x_correction: actor_source_target6_mutant_conversion_x_correction(source_lander)
                .unwrap_or(0),
            target6_first_shot_deferred: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceEnemyProjectileMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub lifetime_ticks: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorLanderSpawn {
    pub position: Point,
    pub source: Option<ActorSourceLanderMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBomberSpawn {
    pub position: Point,
    pub source: Option<ActorSourceBomberMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorPodSpawn {
    pub position: Point,
    pub source: Option<ActorSourcePodMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSwarmerSpawn {
    pub position: Point,
    pub source: Option<ActorSourceSwarmerMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBaiterSpawn {
    pub position: Point,
    pub source: Option<ActorSourceBaiterMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorMutantSpawn {
    pub position: Point,
    pub source: Option<ActorSourceMutantMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorSourceHumanMetadata {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub picture_frame: u8,
    pub target_slot_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorHumanSpawn {
    pub position: Point,
    pub mode: HumanMode,
    pub source: Option<ActorSourceHumanMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceFirstWaveLanderStart {
    x16: u16,
    y16: u16,
    x_velocity: u16,
    y_velocity: u16,
    shot_timer: u8,
    sleep_ticks: u8,
    picture_frame: u8,
    target_human_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceFirstWaveHumanStart {
    x16: u16,
    y16: u16,
    picture_frame: u8,
}

impl ActorLanderSpawn {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            source: None,
        }
    }

    const fn source_first_wave(start: ActorSourceFirstWaveLanderStart) -> Self {
        Self {
            position: Point::new((start.x16 >> 8) as i16, (start.y16 >> 8) as i16),
            source: Some(ActorSourceLanderMetadata {
                x_fraction: (start.x16 & 0x00FF) as u8,
                y_fraction: (start.y16 & 0x00FF) as u8,
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
        source_rng: &mut ActorSourceRng,
        profile: ActorSourceWaveProfile,
        target_human_index: Option<usize>,
    ) -> Self {
        let placement_state = source_rng.advance();
        let x = placement_state.hseed;
        let x_fraction = placement_state.lseed;
        let y = SOURCE_PLAYFIELD_Y_MIN.wrapping_add(2);
        let y_velocity =
            u16::from_be_bytes([profile.lander_y_velocity_msb, profile.lander_y_velocity_lsb]);
        let shot_timer =
            source_rng.advance_rmax(profile.lander_shot_time.min(u32::from(u8::MAX)) as u8);
        let x_velocity_byte = source_rng.advance_rmax(profile.lander_x_velocity);
        let x_velocity = if x_velocity_byte & 1 == 0 {
            u16::from(x_velocity_byte)
        } else {
            !u16::from(x_velocity_byte)
        };

        Self {
            position: Point::new(i16::from(x), i16::from(y)),
            source: Some(ActorSourceLanderMetadata {
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

    const fn source_initial(position: Point, source_x_velocity: u8, spawn_index: usize) -> Self {
        let velocity_low = if spawn_index < 2 {
            0u8.wrapping_sub(source_x_velocity)
        } else {
            source_x_velocity
        };
        Self {
            position,
            source: Some(ActorSourceBomberMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: actor_sign_extend_u8_to_u16(velocity_low),
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                source_slot: (spawn_index % 4) as u8,
            }),
        }
    }

    fn source_restore_batch(
        profile: ActorSourceWaveProfile,
        player_absolute_x: u16,
        count: usize,
    ) -> Vec<Self> {
        let mut bombers = Vec::with_capacity(count);
        let mut remaining = count;
        let mut positive_x_velocity = true;

        while remaining > 0 {
            let squad_count = remaining.min(SOURCE_BOMBER_SQUAD_SIZE);
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
                    position: Point::new(i16::from(x), SOURCE_BOMBER_CRUISE_ALTITUDE),
                    source: Some(ActorSourceBomberMetadata {
                        x_fraction,
                        y_fraction: 0,
                        x_velocity,
                        y_velocity: 0,
                        picture_frame: 0,
                        cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                        sleep_ticks: 0,
                        source_slot: (squad_remaining - 1) as u8,
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
            0u8.wrapping_sub(SOURCE_INITIAL_POD_X_SPEED)
        } else {
            SOURCE_INITIAL_POD_X_SPEED
        };
        Self {
            position,
            source: Some(ActorSourcePodMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: actor_sign_extend_u8_to_u16(velocity_low),
                y_velocity: 0,
            }),
        }
    }

    fn source_restore(source_rng: &mut ActorSourceRng) -> Self {
        let state = source_rng.advance();
        let [x, x_fraction] =
            u16::from_be_bytes([(state.hseed & 0x3F).wrapping_add(0x10), state.lseed])
                .to_be_bytes();
        let y = state
            .lseed
            .wrapping_shr(1)
            .wrapping_add(SOURCE_PLAYFIELD_Y_MIN);
        let x_velocity = actor_sign_extend_u8_to_u16((state.seed & 0x3F).wrapping_sub(0x20));
        let mut y_velocity_low = (state.lseed & 0x7F).wrapping_sub(0x40);
        if y_velocity_low & 0x80 == 0 {
            y_velocity_low |= 0x20;
        } else {
            y_velocity_low &= 0xDF;
        }

        Self {
            position: Point::new(i16::from(x), i16::from(y)),
            source: Some(ActorSourcePodMetadata {
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
        source_rng: &mut ActorSourceRng,
        profile: ActorSourceWaveProfile,
        position: Point,
    ) -> Self {
        let velocity_rand = source_rng.advance();
        let y_velocity = actor_sign_extend_u8_to_u16(velocity_rand.seed).wrapping_shl(1);
        let x_velocity =
            actor_sign_extend_u8_to_u16((velocity_rand.lseed & 0x3F).wrapping_sub(0x20));
        let acceleration = velocity_rand.lseed & profile.swarmer_acceleration_mask;
        let sleep_ticks = velocity_rand.hseed & 0x1F;
        let shot_timer =
            source_rng.advance_rmax(profile.swarmer_shot_time.min(u32::from(u8::MAX)) as u8);

        Self {
            position,
            source: Some(ActorSourceSwarmerMetadata {
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
        source_rng: &mut ActorSourceRng,
        profile: ActorSourceWaveProfile,
        count: usize,
    ) -> Vec<Self> {
        if count == 0 {
            return Vec::new();
        }

        let y16 = u16::from_be_bytes([
            source_rng
                .seed
                .wrapping_shr(1)
                .wrapping_add(SOURCE_PLAYFIELD_Y_MIN),
            0,
        ]);
        let placement_rand = source_rng.advance();
        let x16 = u16::from_be_bytes([
            (placement_rand.seed & 0x3F).wrapping_add(0x80),
            SOURCE_MINI_SWARMER_RESTORE_X_LOW,
        ]);
        let [x, x_fraction] = x16.to_be_bytes();
        let [y, y_fraction] = y16.to_be_bytes();
        let position = Point::new(i16::from(x), i16::from(y));

        (0..count)
            .map(|_| {
                let mut spawn = Self::source_from_pod(source_rng, profile, position);
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
        profile: ActorSourceWaveProfile,
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
        let mut source = ActorSourceBaiterMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: SOURCE_BAITER_INITIAL_SHOT_TIMER,
            sleep_ticks: 0,
            picture_frame: 0,
        };
        source_baiter_velocity_update(
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
        profile: ActorSourceWaveProfile,
        spawn_index: usize,
    ) -> Self {
        let mut source_rng = SOURCE_DEFAULT_RNG;
        for _ in 0..=spawn_index {
            source_rng.advance();
        }
        let shot_timer =
            source_rng.advance_rmax(profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8);
        Self {
            position,
            source: Some(ActorSourceMutantMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer,
                sleep_ticks: 0,
                hop_rng: source_rng.snapshot(),
                render_x_correction: 0,
                target6_first_shot_deferred: false,
            }),
        }
    }

    fn source_restore(
        source_rng: &mut ActorSourceRng,
        profile: ActorSourceWaveProfile,
        background_absolute_x: u16,
    ) -> Self {
        let placement_state = source_rng.advance();
        let avoid_left = background_absolute_x.wrapping_sub(SOURCE_MUTANT_RESTORE_AVOID_HALF_WIDTH);
        let mut relative = u16::from_be_bytes([placement_state.hseed, placement_state.lseed])
            .wrapping_sub(avoid_left);
        if relative < SOURCE_MUTANT_RESTORE_AVOID_WIDTH {
            relative = relative.wrapping_add(0x8000);
        }
        let x16 = relative.wrapping_add(avoid_left);
        let [x, x_fraction] = x16.to_be_bytes();
        let y = placement_state
            .seed
            .wrapping_shr(1)
            .wrapping_add(SOURCE_PLAYFIELD_Y_MIN);
        let shot_timer =
            source_rng.advance_rmax(profile.mutant_shot_time.min(u32::from(u8::MAX)) as u8);

        Self {
            position: Point::new(i16::from(x), i16::from(y)),
            source: Some(ActorSourceMutantMetadata {
                x_fraction,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer,
                sleep_ticks: 0,
                hop_rng: source_rng.snapshot(),
                render_x_correction: 0,
                target6_first_shot_deferred: false,
            }),
        }
    }
}

fn actor_source_initial_target_list_humans() -> Vec<ActorHumanSpawn> {
    let mut source_rng = SOURCE_DEFAULT_RNG;
    actor_source_target_list_restore_humans(&mut source_rng, SOURCE_START_HUMAN_COUNT)
}

fn actor_source_target_list_restore_humans(
    source_rng: &mut ActorSourceRng,
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
                source_rng,
                quadrant_count,
                x_bank,
                slot_index,
            );
        }
        remainder = target_count.wrapping_sub(quadrant_count << 2);
    }

    for _ in 0..remainder {
        let x_bank = source_rng.hseed;
        slot_index = actor_source_target_list_restore_human_group(
            &mut humans,
            source_rng,
            1,
            x_bank,
            slot_index,
        );
    }

    humans
}

fn actor_source_target_list_restore_human_group(
    humans: &mut Vec<ActorHumanSpawn>,
    source_rng: &mut ActorSourceRng,
    count: u8,
    x_bank: u8,
    mut slot_index: usize,
) -> usize {
    for _ in 0..count {
        let state = source_rng.advance();
        let source_x = (state.hseed & 0x1F).wrapping_add(x_bank);
        let picture_frame = if state.lseed & 0x01 != 0 { 2 } else { 0 };
        humans.push(ActorHumanSpawn {
            position: Point::new(i16::from(source_x), i16::from(SOURCE_ASTRO_RESTORE_Y)),
            mode: HumanMode::Grounded,
            source: Some(ActorSourceHumanMetadata {
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
        .filter(|slot| *slot < SOURCE_TARGET_LIST_ENTRY_COUNT)
        .unwrap_or(0);
    let mut probe = original_cursor;
    for _ in 0..SOURCE_TARGET_LIST_ENTRY_COUNT {
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
    if slot_index + 1 < SOURCE_TARGET_LIST_ENTRY_COUNT {
        slot_index + 1
    } else {
        0
    }
}

const fn actor_source_astronaut_next_slot_index(slot_index: usize) -> usize {
    if slot_index + 1 < SOURCE_ASTRONAUT_TARGET_CURSOR_ENTRY_COUNT {
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

    const fn source_first_wave(
        target_slot_index: usize,
        start: ActorSourceFirstWaveHumanStart,
    ) -> Self {
        Self {
            position: Point::new((start.x16 >> 8) as i16, (start.y16 >> 8) as i16),
            mode: HumanMode::Grounded,
            source: Some(ActorSourceHumanMetadata {
                x_fraction: (start.x16 & 0x00FF) as u8,
                y_fraction: (start.y16 & 0x00FF) as u8,
                picture_frame: start.picture_frame,
                target_slot_index,
            }),
        }
    }
}

impl ActorSourceWaveProfile {
    pub fn for_wave(wave: u16) -> Self {
        let wave = u8::try_from(wave.min(u16::from(u8::MAX))).unwrap_or(u8::MAX);
        Self {
            landers: actor_source_wave_u8("landers", wave),
            bombers: actor_source_wave_u8("bombers", wave),
            pods: actor_source_wave_u8("pods", wave),
            mutants: actor_source_wave_u8("mutants", wave),
            swarmers: actor_source_wave_u8("swarmers", wave),
            wave_size: actor_source_wave_u8("wave_size", wave),
            lander_x_velocity: actor_source_wave_u8("lander_x_velocity", wave),
            lander_y_velocity_msb: actor_source_wave_u8("lander_y_velocity_msb", wave),
            lander_y_velocity_lsb: actor_source_wave_u8("lander_y_velocity_lsb", wave),
            bomber_x_velocity: actor_source_wave_u8("bomber_x_velocity", wave),
            swarmer_x_velocity: actor_source_wave_u8("swarmer_x_velocity", wave),
            swarmer_shot_time: actor_source_wave_u32("swarmer_shot_time", wave),
            swarmer_acceleration_mask: actor_source_wave_u8("swarmer_acceleration_mask", wave),
            baiter_delay: actor_source_wave_u32("baiter_time", wave),
            baiter_shot_time: actor_source_wave_u32("baiter_shot_time", wave),
            baiter_seek_probability: actor_source_wave_u8("baiter_seek_probability", wave),
            lander_shot_time: actor_source_wave_u32("lander_shot_time", wave),
            mutant_random_y: actor_source_wave_u8("mutant_random_y", wave),
            mutant_y_velocity_msb: actor_source_wave_u8("mutant_y_velocity_msb", wave),
            mutant_y_velocity_lsb: actor_source_wave_u8("mutant_y_velocity_lsb", wave),
            mutant_x_velocity: actor_source_wave_u8("mutant_x_velocity", wave),
            mutant_shot_time: actor_source_wave_u32("mutant_shot_time", wave),
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
        let mut source_rng = SOURCE_DEFAULT_RNG;
        let mut target_cursor = Some(0usize);
        self.active_family_slots()
            .into_iter()
            .filter_map(|slot| {
                if slot.kind != ActorSourceEnemyKind::Lander {
                    return None;
                }
                let spawn = if wave == 1 {
                    ACTOR_SOURCE_FIRST_WAVE_LANDER_SPAWNS
                        .get(source_lander_index)
                        .copied()
                        .unwrap_or_else(|| ActorLanderSpawn::new(slot.position))
                } else {
                    ActorLanderSpawn::source_restore(
                        &mut source_rng,
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
            ACTOR_SOURCE_FIRST_WAVE_HUMAN_SPAWNS.to_vec()
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
        let mut source_rng = SOURCE_DEFAULT_RNG;
        self.active_family_slots()
            .into_iter()
            .filter(|slot| slot.kind == ActorSourceEnemyKind::Swarmer)
            .map(|slot| ActorSwarmerSpawn::source_from_pod(&mut source_rng, self, slot.position))
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
            .min(SOURCE_MAX_ACTIVE_WAVE_ENEMIES)
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
    profile: ActorSourceWaveProfile,
) -> Vec<ActorSourceEnemyKind> {
    if reserve.landers > 0 {
        let target = SOURCE_MAX_ACTIVE_WAVE_ENEMIES.min(usize::from(reserve.landers));
        let mut kinds = Vec::with_capacity(target);
        while kinds.len() < target
            && actor_enemy_reserve_take(reserve, ActorSourceEnemyKind::Lander)
        {
            kinds.push(ActorSourceEnemyKind::Lander);
        }
        return kinds;
    }

    let target = usize::from(profile.wave_size)
        .min(SOURCE_MAX_ACTIVE_WAVE_ENEMIES)
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

fn actor_source_wave_u8(key: &str, wave: u8) -> u8 {
    u8::try_from(actor_source_wave_value(key, wave))
        .unwrap_or_else(|_| panic!("actor source wave table {key} should fit u8"))
}

fn actor_source_wave_u32(key: &str, wave: u8) -> u32 {
    u32::try_from(actor_source_wave_value(key, wave))
        .unwrap_or_else(|_| panic!("actor source wave table {key} should be non-negative"))
}

fn actor_source_wave_value(key: &str, wave: u8) -> i32 {
    let mut lines = ACTOR_SOURCE_WAVE_TABLE_TSV.lines();
    let header = lines
        .next()
        .expect("actor source wave table should have a header");
    assert_eq!(header, ACTOR_SOURCE_WAVE_TABLE_HEADER);

    for row in lines.map(str::trim).filter(|row| !row.is_empty()) {
        let fields = row.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 9, "actor source wave table row width changed");
        if fields[0] != key {
            continue;
        }

        let ceiling = parse_actor_wave_i32(fields[1], key, "ceiling");
        let floor = parse_actor_wave_i32(fields[2], key, "floor");
        let inter_delta = parse_actor_wave_i32(fields[4], key, "inter_delta");
        let wave = wave.max(1);
        let wave_index = usize::from(wave.min(4));
        let mut value = parse_actor_wave_i32(fields[4 + wave_index], key, "wave");
        for _ in 0..actor_wave_inter_delta_iterations(wave) {
            value = apply_actor_wave_delta(value, inter_delta, floor, ceiling);
        }
        return value;
    }

    panic!("missing actor source wave table key {key}");
}

fn actor_wave_inter_delta_iterations(wave: u8) -> u16 {
    let wave_delta = wave.saturating_sub(4);
    let pre_ceiling = ACTOR_SOURCE_DEFAULT_DIFFICULTY_INITIAL.saturating_add(wave_delta);
    u16::from(pre_ceiling.min(ACTOR_SOURCE_DEFAULT_DIFFICULTY_CEILING))
}

fn parse_actor_wave_i32(value: &str, key: &str, field: &str) -> i32 {
    value
        .parse()
        .unwrap_or_else(|_| panic!("actor source wave table {key}.{field} is not an integer"))
}

fn apply_actor_wave_delta(value: i32, delta: i32, floor: i32, ceiling: i32) -> i32 {
    if delta > 0 {
        (value + delta).min(ceiling)
    } else if delta < 0 {
        (value + delta).max(floor)
    } else {
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveProfile {
    pub wave: u16,
    pub source_wave: Option<ActorSourceWaveProfile>,
    pub behavior_script: ActorBehaviorScript,
    pub lander_spawns: Vec<ActorLanderSpawn>,
    pub bomber_spawns: Vec<ActorBomberSpawn>,
    pub pod_spawns: Vec<ActorPodSpawn>,
    pub mutant_spawns: Vec<ActorMutantSpawn>,
    pub swarmer_spawns: Vec<ActorSwarmerSpawn>,
    pub baiter_spawns: Vec<ActorBaiterSpawn>,
    pub human_spawns: Vec<ActorHumanSpawn>,
    pub enemy_reserve: EnemyReserveSnapshot,
    pub spawn_behavior_profiles: Vec<ActorWaveSpawnBehaviorProfile>,
}

impl ActorWaveProfile {
    pub fn new(wave: u16, behavior_script: ActorBehaviorScript, lander_spawns: Vec<Point>) -> Self {
        Self::with_lander_spawns(
            wave,
            behavior_script,
            lander_spawns
                .into_iter()
                .map(ActorLanderSpawn::new)
                .collect(),
        )
    }

    pub fn with_lander_spawns(
        wave: u16,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<ActorLanderSpawn>,
    ) -> Self {
        Self::with_spawns(
            wave,
            behavior_script,
            lander_spawns,
            ACTOR_SOURCE_FIRST_WAVE_HUMAN_SPAWNS.to_vec(),
        )
    }

    pub fn with_spawns(
        wave: u16,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<ActorLanderSpawn>,
        human_spawns: Vec<ActorHumanSpawn>,
    ) -> Self {
        Self::with_family_spawns(
            wave,
            behavior_script,
            lander_spawns,
            Vec::new(),
            Vec::new(),
            human_spawns,
        )
    }

    pub fn with_family_spawns(
        wave: u16,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<ActorLanderSpawn>,
        bomber_spawns: Vec<ActorBomberSpawn>,
        pod_spawns: Vec<ActorPodSpawn>,
        human_spawns: Vec<ActorHumanSpawn>,
    ) -> Self {
        Self {
            wave: wave.max(1),
            source_wave: None,
            behavior_script,
            lander_spawns,
            bomber_spawns,
            pod_spawns,
            mutant_spawns: Vec::new(),
            swarmer_spawns: Vec::new(),
            baiter_spawns: Vec::new(),
            human_spawns,
            enemy_reserve: EnemyReserveSnapshot::default(),
            spawn_behavior_profiles: Vec::new(),
        }
    }

    pub fn with_mutant_spawns(mut self, mutant_spawns: Vec<ActorMutantSpawn>) -> Self {
        self.mutant_spawns = mutant_spawns;
        self
    }

    pub fn with_source_wave(mut self, source_wave: ActorSourceWaveProfile) -> Self {
        self.source_wave = Some(source_wave);
        self
    }

    fn with_optional_source_wave(mut self, source_wave: Option<ActorSourceWaveProfile>) -> Self {
        self.source_wave = source_wave;
        self
    }

    pub fn with_swarmer_spawns(mut self, swarmer_spawns: Vec<ActorSwarmerSpawn>) -> Self {
        self.swarmer_spawns = swarmer_spawns;
        self
    }

    pub fn with_baiter_spawns(mut self, baiter_spawns: Vec<ActorBaiterSpawn>) -> Self {
        self.baiter_spawns = baiter_spawns;
        self
    }

    pub fn with_enemy_reserve(mut self, enemy_reserve: EnemyReserveSnapshot) -> Self {
        self.enemy_reserve = enemy_reserve;
        self
    }

    pub fn with_spawn_behavior_profiles(
        mut self,
        spawn_behavior_profiles: Vec<ActorWaveSpawnBehaviorProfile>,
    ) -> Self {
        self.spawn_behavior_profiles = spawn_behavior_profiles;
        self
    }

    pub fn spawn_behavior_profile(
        &self,
        kind: ActorKind,
        spawn_index: usize,
    ) -> Option<ActorBehaviorProfile> {
        self.spawn_behavior_profiles
            .iter()
            .find(|entry| entry.kind == kind && entry.spawn_index == spawn_index)
            .map(|entry| entry.profile)
    }

    pub fn lander_spawn_points(&self) -> Vec<Point> {
        self.lander_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn human_spawn_points(&self) -> Vec<Point> {
        self.human_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn bomber_spawn_points(&self) -> Vec<Point> {
        self.bomber_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn pod_spawn_points(&self) -> Vec<Point> {
        self.pod_spawns.iter().map(|spawn| spawn.position).collect()
    }

    pub fn mutant_spawn_points(&self) -> Vec<Point> {
        self.mutant_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn swarmer_spawn_points(&self) -> Vec<Point> {
        self.swarmer_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn baiter_spawn_points(&self) -> Vec<Point> {
        self.baiter_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn manifest(&self) -> ActorWaveProfileManifest {
        ActorWaveProfileManifest {
            wave: self.wave,
            source_wave: self.source_wave,
            behavior_script: self.behavior_script.manifest(),
            lander_spawns: self.lander_spawns.clone(),
            bomber_spawns: self.bomber_spawns.clone(),
            pod_spawns: self.pod_spawns.clone(),
            mutant_spawns: self.mutant_spawns.clone(),
            swarmer_spawns: self.swarmer_spawns.clone(),
            baiter_spawns: self.baiter_spawns.clone(),
            human_spawns: self.human_spawns.clone(),
            enemy_reserve: self.enemy_reserve,
            spawn_behavior_profiles: self.spawn_behavior_profiles.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorWaveSpawnBehaviorProfile {
    pub kind: ActorKind,
    pub spawn_index: usize,
    pub profile: ActorBehaviorProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveProfileManifest {
    pub wave: u16,
    pub source_wave: Option<ActorSourceWaveProfile>,
    pub behavior_script: ActorBehaviorScriptManifest,
    pub lander_spawns: Vec<ActorLanderSpawn>,
    pub bomber_spawns: Vec<ActorBomberSpawn>,
    pub pod_spawns: Vec<ActorPodSpawn>,
    pub mutant_spawns: Vec<ActorMutantSpawn>,
    pub swarmer_spawns: Vec<ActorSwarmerSpawn>,
    pub baiter_spawns: Vec<ActorBaiterSpawn>,
    pub human_spawns: Vec<ActorHumanSpawn>,
    pub enemy_reserve: EnemyReserveSnapshot,
    pub spawn_behavior_profiles: Vec<ActorWaveSpawnBehaviorProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveScript {
    name: String,
    waves: Vec<ActorWaveProfile>,
    behavior_presets: Vec<ActorWaveBehaviorPresetManifest>,
    spawn_behavior_presets: Vec<ActorWaveSpawnBehaviorPresetManifest>,
}

impl ActorWaveScript {
    pub fn new(name: impl Into<String>, waves: Vec<ActorWaveProfile>) -> Self {
        Self::with_presets(name, waves, Vec::new(), Vec::new())
    }

    fn with_presets(
        name: impl Into<String>,
        mut waves: Vec<ActorWaveProfile>,
        behavior_presets: Vec<ActorWaveBehaviorPresetManifest>,
        spawn_behavior_presets: Vec<ActorWaveSpawnBehaviorPresetManifest>,
    ) -> Self {
        if waves.is_empty() {
            waves.push(Self::source_backed_profile(1));
        }
        waves.sort_by_key(|profile| profile.wave);
        Self {
            name: name.into(),
            waves,
            behavior_presets,
            spawn_behavior_presets,
        }
    }

    pub fn single_wave(
        name: impl Into<String>,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<Point>,
    ) -> Self {
        Self::new(
            name,
            vec![ActorWaveProfile::new(1, behavior_script, lander_spawns)],
        )
    }

    pub fn default_progression() -> Self {
        Self::parse_text(ACTOR_RED_LABEL_WAVE_SCRIPT)
            .unwrap_or_else(|error| panic!("embedded actor wave script is invalid: {error}"))
    }

    pub fn source_table_progression() -> Self {
        let waves = (1..=ACTOR_SOURCE_BACKED_WAVES)
            .map(Self::source_backed_profile)
            .collect::<Vec<_>>();
        Self::new("actor-source-wave-table", waves)
    }

    fn source_backed_profile(wave: u16) -> ActorWaveProfile {
        let source = ActorSourceWaveProfile::for_wave(wave);
        Self::source_backed_profile_from_source(wave, source)
    }

    fn source_backed_profile_from_source(
        wave: u16,
        source: ActorSourceWaveProfile,
    ) -> ActorWaveProfile {
        Self::source_backed_profile_from_source_with_behavior(
            wave,
            source,
            &ActorBehaviorScript::from_arcade_profile(),
        )
    }

    fn source_backed_profile_from_source_with_behavior(
        wave: u16,
        source: ActorSourceWaveProfile,
        base_behavior: &ActorBehaviorScript,
    ) -> ActorWaveProfile {
        let human_spawns = source.human_spawns(wave);
        ActorWaveProfile::with_family_spawns(
            wave,
            base_behavior
                .clone()
                .with_kind_behavior(ActorKind::Lander, source.lander_behavior())
                .with_kind_behavior(
                    ActorKind::Bomber,
                    ActorBehaviorProfile {
                        bomber_drift_speed: actor_velocity_pixels_from_source(
                            source.bomber_x_velocity,
                        ),
                        ..ActorBehaviorProfile::default()
                    },
                ),
            source.lander_spawns(wave, &human_spawns),
            source.bomber_spawns(),
            source.pod_spawns(),
            human_spawns,
        )
        .with_source_wave(source)
        .with_mutant_spawns(source.mutant_spawns())
        .with_swarmer_spawns(source.swarmer_spawns())
        .with_enemy_reserve(source.enemy_reserve_after_active_batch())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn profile_for_wave(&self, wave: u16) -> &ActorWaveProfile {
        self.waves
            .iter()
            .rev()
            .find(|profile| wave >= profile.wave)
            .unwrap_or(&self.waves[0])
    }

    pub fn manifest(&self) -> ActorWaveScriptManifest {
        ActorWaveScriptManifest {
            name: self.name.clone(),
            behavior_presets: self.behavior_presets.clone(),
            spawn_behavior_presets: self.spawn_behavior_presets.clone(),
            waves: self.waves.iter().map(ActorWaveProfile::manifest).collect(),
        }
    }
}

impl FromStr for ActorWaveScript {
    type Err = ActorWaveScriptParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let base_behavior = ActorBehaviorScript::from_arcade_profile();
        Self::parse_text_with_base_behavior(source, &base_behavior)
    }
}

impl ActorWaveScript {
    pub fn parse_text(source: &str) -> Result<Self, ActorWaveScriptParseError> {
        source.parse()
    }

    pub fn parse_text_with_base_behavior(
        source: &str,
        base_behavior: &ActorBehaviorScript,
    ) -> Result<Self, ActorWaveScriptParseError> {
        let mut parser = ParsedActorWaveScript::with_base_behavior(base_behavior.clone());
        for (line_index, raw_line) in source.lines().enumerate() {
            let line_number = line_index + 1;
            let line = raw_line
                .split_once('#')
                .map_or(raw_line, |(before_comment, _)| before_comment)
                .trim();
            if line.is_empty() {
                continue;
            }
            parser.parse_line(line_number, line)?;
        }
        parser.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveScriptParseError {
    pub line: usize,
    pub message: String,
}

impl ActorWaveScriptParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ActorWaveScriptParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "actor wave script line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for ActorWaveScriptParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedActorWaveScript {
    name: String,
    waves: Vec<ParsedActorWaveProfile>,
    behavior_presets: BTreeMap<String, Vec<ParsedBehaviorPresetUpdate>>,
    spawn_behavior_presets: BTreeMap<String, Vec<ParsedSpawnBehaviorPresetUpdate>>,
    base_behavior: ActorBehaviorScript,
}

impl ParsedActorWaveScript {
    fn with_base_behavior(base_behavior: ActorBehaviorScript) -> Self {
        Self {
            name: "parsed-wave-script".to_string(),
            waves: Vec::new(),
            behavior_presets: BTreeMap::new(),
            spawn_behavior_presets: BTreeMap::new(),
            base_behavior,
        }
    }

    fn parse_line(
        &mut self,
        line_number: usize,
        line: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        let mut parts = line.split_whitespace();
        let action = parts
            .next()
            .ok_or_else(|| ActorWaveScriptParseError::new(line_number, "missing action"))?;
        match normalize_script_token(action).as_str() {
            "name" => {
                let name = parts.collect::<Vec<_>>().join(" ");
                if name.is_empty() {
                    return Err(ActorWaveScriptParseError::new(
                        line_number,
                        "name action needs a value",
                    ));
                }
                self.name = name;
                Ok(())
            }
            "behavior_preset" | "behaviour_preset" => {
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                let behavior_line = parts.collect::<Vec<_>>().join(" ");
                if behavior_line.is_empty() {
                    return Err(ActorWaveScriptParseError::new(
                        line_number,
                        format!("behavior preset `{name}` needs a profile update"),
                    ));
                }
                self.define_behavior_preset(line_number, name, behavior_line)
            }
            "spawn_behavior_preset" | "spawn_behaviour_preset" => {
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                let field = parts.next().ok_or_else(|| {
                    ActorWaveScriptParseError::new(line_number, "missing behavior field")
                })?;
                let values = parts.collect::<Vec<_>>();
                self.define_spawn_behavior_preset(line_number, name, field, &values)
            }
            "wave" => {
                let wave = parse_wave_u16(line_number, parts.next(), "wave")?;
                reject_extra_wave_fields(line_number, parts)?;
                self.push_profile(
                    line_number,
                    ParsedActorWaveProfile::new_with_behavior(wave, self.base_behavior.clone()),
                )
            }
            "source_wave" | "source_backed_wave" => {
                let wave = parse_wave_u16(line_number, parts.next(), "wave")?;
                let mut source = ActorSourceWaveProfile::for_wave(wave);
                parse_source_wave_profile_updates(line_number, &mut source, parts)?;
                self.push_profile(
                    line_number,
                    ParsedActorWaveProfile::source_backed_from_source_with_behavior(
                        wave,
                        source,
                        &self.base_behavior,
                    ),
                )
            }
            "source_waves" | "source_backed_waves" => {
                let first = parse_wave_u16(line_number, parts.next(), "first wave")?.max(1);
                let last = parse_wave_u16(line_number, parts.next(), "last wave")?.max(1);
                let source_update_tokens = parts.collect::<Vec<_>>();
                if last < first {
                    return Err(ActorWaveScriptParseError::new(
                        line_number,
                        format!("source wave range `{first}..{last}` is invalid"),
                    ));
                }
                for wave in first..=last {
                    let mut source = ActorSourceWaveProfile::for_wave(wave);
                    parse_source_wave_profile_updates(
                        line_number,
                        &mut source,
                        source_update_tokens.iter().copied(),
                    )?;
                    self.push_profile(
                        line_number,
                        ParsedActorWaveProfile::source_backed_from_source_with_behavior(
                            wave,
                            source,
                            &self.base_behavior,
                        ),
                    )?;
                }
                Ok(())
            }
            "behavior" => {
                let behavior_line = parts.collect::<Vec<_>>().join(" ");
                if behavior_line.is_empty() {
                    return Err(ActorWaveScriptParseError::new(
                        line_number,
                        "behavior action needs a profile update",
                    ));
                }
                let profile = self.current_profile_mut(line_number)?;
                parse_behavior_script_line(
                    line_number,
                    &behavior_line,
                    &mut profile.behavior_script,
                )
                .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))
            }
            "use_behavior"
            | "use_behaviour"
            | "apply_behavior_preset"
            | "apply_behaviour_preset" => {
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                reject_extra_wave_fields(line_number, parts)?;
                self.apply_behavior_preset_to_current_wave(line_number, &name)
            }
            "behavior_waves" | "behaviour_waves" | "behavior_range" | "behaviour_range" => {
                let first = parse_wave_u16(line_number, parts.next(), "first wave")?.max(1);
                let last = parse_wave_u16(line_number, parts.next(), "last wave")?.max(1);
                let behavior_line = parts.collect::<Vec<_>>().join(" ");
                if behavior_line.is_empty() {
                    return Err(ActorWaveScriptParseError::new(
                        line_number,
                        "behavior range action needs a profile update",
                    ));
                }
                self.apply_behavior_to_wave_range(line_number, first, last, &behavior_line)
            }
            "use_behavior_waves"
            | "use_behaviour_waves"
            | "behavior_preset_waves"
            | "behaviour_preset_waves" => {
                let first = parse_wave_u16(line_number, parts.next(), "first wave")?.max(1);
                let last = parse_wave_u16(line_number, parts.next(), "last wave")?.max(1);
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                reject_extra_wave_fields(line_number, parts)?;
                self.apply_behavior_preset_to_wave_range(line_number, first, last, &name)
            }
            "spawn_behavior" | "spawn_behaviour" => {
                let kind = parse_wave_actor_kind(line_number, parts.next())?;
                let spawn_index = parse_wave_usize(line_number, parts.next(), "spawn index")?;
                let field = parts.next().ok_or_else(|| {
                    ActorWaveScriptParseError::new(line_number, "missing behavior field")
                })?;
                let values = parts.collect::<Vec<_>>();
                let profile = self
                    .current_profile_mut(line_number)?
                    .spawn_behavior_profile_mut(kind, spawn_index);
                apply_behavior_profile_field(line_number, profile, field, &values)
                    .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))
            }
            "use_spawn_behavior"
            | "use_spawn_behaviour"
            | "apply_spawn_behavior_preset"
            | "apply_spawn_behaviour_preset" => {
                let kind = parse_wave_actor_kind(line_number, parts.next())?;
                let spawn_index = parse_wave_usize(line_number, parts.next(), "spawn index")?;
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                reject_extra_wave_fields(line_number, parts)?;
                self.apply_spawn_behavior_preset_to_current_wave(
                    line_number,
                    kind,
                    spawn_index,
                    &name,
                )
            }
            "spawn_behavior_waves"
            | "spawn_behaviour_waves"
            | "spawn_behavior_range"
            | "spawn_behaviour_range" => {
                let first = parse_wave_u16(line_number, parts.next(), "first wave")?.max(1);
                let last = parse_wave_u16(line_number, parts.next(), "last wave")?.max(1);
                let kind = parse_wave_actor_kind(line_number, parts.next())?;
                let spawn_index = parse_wave_usize(line_number, parts.next(), "spawn index")?;
                let field = parts.next().ok_or_else(|| {
                    ActorWaveScriptParseError::new(line_number, "missing behavior field")
                })?;
                let values = parts.collect::<Vec<_>>();
                self.apply_spawn_behavior_to_wave_range(
                    line_number,
                    first,
                    last,
                    ParsedWaveSpawnBehaviorUpdate {
                        kind,
                        spawn_index,
                        field,
                        values: &values,
                    },
                )
            }
            "use_spawn_behavior_waves"
            | "use_spawn_behaviour_waves"
            | "spawn_behavior_preset_waves"
            | "spawn_behaviour_preset_waves" => {
                let first = parse_wave_u16(line_number, parts.next(), "first wave")?.max(1);
                let last = parse_wave_u16(line_number, parts.next(), "last wave")?.max(1);
                let kind = parse_wave_actor_kind(line_number, parts.next())?;
                let spawn_index = parse_wave_usize(line_number, parts.next(), "spawn index")?;
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                reject_extra_wave_fields(line_number, parts)?;
                self.apply_spawn_behavior_preset_to_wave_range(
                    line_number,
                    first,
                    last,
                    kind,
                    spawn_index,
                    &name,
                )
            }
            "lander" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .lander_spawns
                    .push(ActorLanderSpawn::new(position));
                Ok(())
            }
            "bomber" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .bomber_spawns
                    .push(ActorBomberSpawn::new(position));
                Ok(())
            }
            "pod" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .pod_spawns
                    .push(ActorPodSpawn::new(position));
                Ok(())
            }
            "mutant" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .mutant_spawns
                    .push(ActorMutantSpawn::new(position));
                Ok(())
            }
            "swarmer" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .swarmer_spawns
                    .push(ActorSwarmerSpawn::new(position));
                Ok(())
            }
            "baiter" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .baiter_spawns
                    .push(ActorBaiterSpawn::new(position));
                Ok(())
            }
            "human" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                let mode = parse_wave_human_mode(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .human_spawns
                    .push(ActorHumanSpawn::new(position, mode));
                Ok(())
            }
            "enemy_reserve" | "reserve" => {
                let landers = parse_wave_u8(line_number, parts.next(), "reserve landers")?;
                let bombers = parse_wave_u8(line_number, parts.next(), "reserve bombers")?;
                let pods = parse_wave_u8(line_number, parts.next(), "reserve pods")?;
                let swarmers = parts
                    .next()
                    .map(|field| parse_wave_u8(line_number, Some(field), "reserve swarmers"))
                    .transpose()?
                    .unwrap_or(0);
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?.enemy_reserve = EnemyReserveSnapshot {
                    landers,
                    bombers,
                    pods,
                    swarmers,
                    ..EnemyReserveSnapshot::default()
                };
                Ok(())
            }
            "enemy_reserve_full" | "reserve_full" => {
                let landers = parse_wave_u8(line_number, parts.next(), "reserve landers")?;
                let bombers = parse_wave_u8(line_number, parts.next(), "reserve bombers")?;
                let pods = parse_wave_u8(line_number, parts.next(), "reserve pods")?;
                let mutants = parse_wave_u8(line_number, parts.next(), "reserve mutants")?;
                let swarmers = parse_wave_u8(line_number, parts.next(), "reserve swarmers")?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?.enemy_reserve = EnemyReserveSnapshot {
                    landers,
                    bombers,
                    pods,
                    mutants,
                    swarmers,
                };
                Ok(())
            }
            _ => Err(ActorWaveScriptParseError::new(
                line_number,
                format!("unknown wave action `{action}`"),
            )),
        }
    }

    fn define_spawn_behavior_preset(
        &mut self,
        line_number: usize,
        name: String,
        field: &str,
        values: &[&str],
    ) -> Result<(), ActorWaveScriptParseError> {
        let mut validation = ActorBehaviorProfile::default();
        apply_behavior_profile_field(line_number, &mut validation, field, values)
            .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
        self.spawn_behavior_presets.entry(name).or_default().push(
            ParsedSpawnBehaviorPresetUpdate {
                line_number,
                field: field.to_string(),
                values: values.iter().map(|value| (*value).to_string()).collect(),
            },
        );
        Ok(())
    }

    fn define_behavior_preset(
        &mut self,
        line_number: usize,
        name: String,
        behavior_line: String,
    ) -> Result<(), ActorWaveScriptParseError> {
        let mut validation = ActorBehaviorScript::from_arcade_profile();
        parse_behavior_script_line(line_number, &behavior_line, &mut validation)
            .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
        self.behavior_presets
            .entry(name)
            .or_default()
            .push(ParsedBehaviorPresetUpdate {
                line_number,
                line: behavior_line,
            });
        Ok(())
    }

    fn behavior_preset_updates(
        &self,
        line_number: usize,
        name: &str,
    ) -> Result<Vec<ParsedBehaviorPresetUpdate>, ActorWaveScriptParseError> {
        self.behavior_presets.get(name).cloned().ok_or_else(|| {
            ActorWaveScriptParseError::new(line_number, format!("unknown behavior preset `{name}`"))
        })
    }

    fn spawn_behavior_preset_updates(
        &self,
        line_number: usize,
        name: &str,
    ) -> Result<Vec<ParsedSpawnBehaviorPresetUpdate>, ActorWaveScriptParseError> {
        self.spawn_behavior_presets
            .get(name)
            .cloned()
            .ok_or_else(|| {
                ActorWaveScriptParseError::new(
                    line_number,
                    format!("unknown spawn behavior preset `{name}`"),
                )
            })
    }

    fn apply_behavior_preset_to_current_wave(
        &mut self,
        line_number: usize,
        name: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        let updates = self.behavior_preset_updates(line_number, name)?;
        let profile = self.current_profile_mut(line_number)?;
        apply_behavior_preset_updates(&updates, &mut profile.behavior_script)
    }

    fn apply_spawn_behavior_preset_to_current_wave(
        &mut self,
        line_number: usize,
        kind: ActorKind,
        spawn_index: usize,
        name: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        let updates = self.spawn_behavior_preset_updates(line_number, name)?;
        let profile = self
            .current_profile_mut(line_number)?
            .spawn_behavior_profile_mut(kind, spawn_index);
        apply_spawn_behavior_preset_updates(&updates, profile)
    }

    fn current_profile_mut(
        &mut self,
        line_number: usize,
    ) -> Result<&mut ParsedActorWaveProfile, ActorWaveScriptParseError> {
        self.waves.last_mut().ok_or_else(|| {
            ActorWaveScriptParseError::new(line_number, "wave action must appear before this line")
        })
    }

    fn push_profile(
        &mut self,
        line_number: usize,
        profile: ParsedActorWaveProfile,
    ) -> Result<(), ActorWaveScriptParseError> {
        if self
            .waves
            .iter()
            .any(|candidate| candidate.wave == profile.wave)
        {
            return Err(ActorWaveScriptParseError::new(
                line_number,
                format!("duplicate wave `{}`", profile.wave),
            ));
        }
        self.waves.push(profile);
        Ok(())
    }

    fn profile_for_wave_mut(
        &mut self,
        line_number: usize,
        wave: u16,
    ) -> Result<&mut ParsedActorWaveProfile, ActorWaveScriptParseError> {
        self.waves
            .iter_mut()
            .find(|profile| profile.wave == wave)
            .ok_or_else(|| {
                ActorWaveScriptParseError::new(
                    line_number,
                    format!("wave range references undefined wave `{wave}`"),
                )
            })
    }

    fn validate_wave_range(
        line_number: usize,
        first: u16,
        last: u16,
        label: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        if last < first {
            return Err(ActorWaveScriptParseError::new(
                line_number,
                format!("{label} range `{first}..{last}` is invalid"),
            ));
        }
        Ok(())
    }

    fn apply_behavior_preset_to_wave_range(
        &mut self,
        line_number: usize,
        first: u16,
        last: u16,
        name: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        Self::validate_wave_range(line_number, first, last, "behavior preset wave")?;
        let updates = self.behavior_preset_updates(line_number, name)?;
        for wave in first..=last {
            let profile = self.profile_for_wave_mut(line_number, wave)?;
            apply_behavior_preset_updates(&updates, &mut profile.behavior_script)?;
        }
        Ok(())
    }

    fn apply_behavior_to_wave_range(
        &mut self,
        line_number: usize,
        first: u16,
        last: u16,
        behavior_line: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        Self::validate_wave_range(line_number, first, last, "behavior wave")?;
        for wave in first..=last {
            let profile = self.profile_for_wave_mut(line_number, wave)?;
            parse_behavior_script_line(line_number, behavior_line, &mut profile.behavior_script)
                .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
        }
        Ok(())
    }

    fn apply_spawn_behavior_to_wave_range(
        &mut self,
        line_number: usize,
        first: u16,
        last: u16,
        update: ParsedWaveSpawnBehaviorUpdate<'_>,
    ) -> Result<(), ActorWaveScriptParseError> {
        Self::validate_wave_range(line_number, first, last, "spawn behavior wave")?;
        for wave in first..=last {
            let profile = self
                .profile_for_wave_mut(line_number, wave)?
                .spawn_behavior_profile_mut(update.kind, update.spawn_index);
            apply_behavior_profile_field(line_number, profile, update.field, update.values)
                .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
        }
        Ok(())
    }

    fn apply_spawn_behavior_preset_to_wave_range(
        &mut self,
        line_number: usize,
        first: u16,
        last: u16,
        kind: ActorKind,
        spawn_index: usize,
        name: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        Self::validate_wave_range(line_number, first, last, "spawn behavior preset wave")?;
        let updates = self.spawn_behavior_preset_updates(line_number, name)?;
        for wave in first..=last {
            let profile = self
                .profile_for_wave_mut(line_number, wave)?
                .spawn_behavior_profile_mut(kind, spawn_index);
            apply_spawn_behavior_preset_updates(&updates, profile)?;
        }
        Ok(())
    }

    fn finish(self) -> Result<ActorWaveScript, ActorWaveScriptParseError> {
        let Self {
            name,
            waves,
            behavior_presets,
            spawn_behavior_presets,
            base_behavior: _,
        } = self;
        if waves.is_empty() {
            return Err(ActorWaveScriptParseError::new(
                0,
                "wave script needs at least one wave",
            ));
        }
        Ok(ActorWaveScript::with_presets(
            name,
            waves
                .into_iter()
                .map(ParsedActorWaveProfile::finish)
                .collect(),
            behavior_presets
                .into_iter()
                .map(|(name, updates)| ActorWaveBehaviorPresetManifest {
                    name,
                    updates: updates.into_iter().map(|update| update.line).collect(),
                })
                .collect(),
            spawn_behavior_presets
                .into_iter()
                .map(|(name, updates)| ActorWaveSpawnBehaviorPresetManifest {
                    name,
                    updates: updates
                        .into_iter()
                        .map(|update| ActorWaveSpawnBehaviorPresetUpdateManifest {
                            field: update.field,
                            values: update.values,
                        })
                        .collect(),
                })
                .collect(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedBehaviorPresetUpdate {
    line_number: usize,
    line: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSpawnBehaviorPresetUpdate {
    line_number: usize,
    field: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct ParsedWaveSpawnBehaviorUpdate<'a> {
    kind: ActorKind,
    spawn_index: usize,
    field: &'a str,
    values: &'a [&'a str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedActorWaveProfile {
    wave: u16,
    source_wave: Option<ActorSourceWaveProfile>,
    behavior_script: ActorBehaviorScript,
    lander_spawns: Vec<ActorLanderSpawn>,
    bomber_spawns: Vec<ActorBomberSpawn>,
    pod_spawns: Vec<ActorPodSpawn>,
    mutant_spawns: Vec<ActorMutantSpawn>,
    swarmer_spawns: Vec<ActorSwarmerSpawn>,
    baiter_spawns: Vec<ActorBaiterSpawn>,
    human_spawns: Vec<ActorHumanSpawn>,
    enemy_reserve: EnemyReserveSnapshot,
    spawn_behavior_profiles: Vec<ActorWaveSpawnBehaviorProfile>,
}

impl ParsedActorWaveProfile {
    fn new_with_behavior(wave: u16, behavior_script: ActorBehaviorScript) -> Self {
        Self {
            wave: wave.max(1),
            source_wave: None,
            behavior_script,
            lander_spawns: Vec::new(),
            bomber_spawns: Vec::new(),
            pod_spawns: Vec::new(),
            mutant_spawns: Vec::new(),
            swarmer_spawns: Vec::new(),
            baiter_spawns: Vec::new(),
            human_spawns: Vec::new(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            spawn_behavior_profiles: Vec::new(),
        }
    }

    fn source_backed_from_source_with_behavior(
        wave: u16,
        source: ActorSourceWaveProfile,
        base_behavior: &ActorBehaviorScript,
    ) -> Self {
        let profile = ActorWaveScript::source_backed_profile_from_source_with_behavior(
            wave,
            source,
            base_behavior,
        );
        Self {
            wave: profile.wave,
            source_wave: profile.source_wave,
            behavior_script: profile.behavior_script,
            lander_spawns: profile.lander_spawns,
            bomber_spawns: profile.bomber_spawns,
            pod_spawns: profile.pod_spawns,
            mutant_spawns: profile.mutant_spawns,
            swarmer_spawns: profile.swarmer_spawns,
            baiter_spawns: profile.baiter_spawns,
            human_spawns: profile.human_spawns,
            enemy_reserve: profile.enemy_reserve,
            spawn_behavior_profiles: profile.spawn_behavior_profiles,
        }
    }

    fn spawn_behavior_profile_mut(
        &mut self,
        kind: ActorKind,
        spawn_index: usize,
    ) -> &mut ActorBehaviorProfile {
        let entry_index = self
            .spawn_behavior_profiles
            .iter()
            .position(|entry| entry.kind == kind && entry.spawn_index == spawn_index);
        let entry_index = match entry_index {
            Some(entry_index) => entry_index,
            None => {
                self.spawn_behavior_profiles
                    .push(ActorWaveSpawnBehaviorProfile {
                        kind,
                        spawn_index,
                        profile: self.behavior_script.behavior_for(ActorId::new(0), kind),
                    });
                self.spawn_behavior_profiles.len() - 1
            }
        };
        &mut self.spawn_behavior_profiles[entry_index].profile
    }

    fn finish(self) -> ActorWaveProfile {
        ActorWaveProfile::with_family_spawns(
            self.wave,
            self.behavior_script,
            self.lander_spawns,
            self.bomber_spawns,
            self.pod_spawns,
            self.human_spawns,
        )
        .with_optional_source_wave(self.source_wave)
        .with_mutant_spawns(self.mutant_spawns)
        .with_swarmer_spawns(self.swarmer_spawns)
        .with_baiter_spawns(self.baiter_spawns)
        .with_enemy_reserve(self.enemy_reserve)
        .with_spawn_behavior_profiles(self.spawn_behavior_profiles)
    }
}

fn apply_behavior_preset_updates(
    updates: &[ParsedBehaviorPresetUpdate],
    behavior_script: &mut ActorBehaviorScript,
) -> Result<(), ActorWaveScriptParseError> {
    for update in updates {
        parse_behavior_script_line(update.line_number, &update.line, behavior_script)
            .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
    }
    Ok(())
}

fn apply_spawn_behavior_preset_updates(
    updates: &[ParsedSpawnBehaviorPresetUpdate],
    behavior_profile: &mut ActorBehaviorProfile,
) -> Result<(), ActorWaveScriptParseError> {
    for update in updates {
        let values = update.values.iter().map(String::as_str).collect::<Vec<_>>();
        apply_behavior_profile_field(update.line_number, behavior_profile, &update.field, &values)
            .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
    }
    Ok(())
}

fn parse_wave_point<'a>(
    line_number: usize,
    parts: &mut impl Iterator<Item = &'a str>,
) -> Result<Point, ActorWaveScriptParseError> {
    let x = parse_wave_i16(line_number, parts.next(), "x")?;
    let y = parse_wave_i16(line_number, parts.next(), "y")?;
    Ok(Point::new(x, y))
}

fn parse_wave_human_mode<'a>(
    line_number: usize,
    mut parts: impl Iterator<Item = &'a str>,
) -> Result<HumanMode, ActorWaveScriptParseError> {
    let Some(mode) = parts.next() else {
        return Ok(HumanMode::Grounded);
    };
    match normalize_script_token(mode).as_str() {
        "grounded" => {
            reject_extra_wave_fields(line_number, parts)?;
            Ok(HumanMode::Grounded)
        }
        "falling" => {
            let velocity = parse_wave_i16(line_number, parts.next(), "fall velocity")?;
            reject_extra_wave_fields(line_number, parts)?;
            Ok(HumanMode::Falling { velocity })
        }
        "carried" | "carried_by" => {
            let actor = ActorId::new(parse_wave_u64(line_number, parts.next(), "carrier actor")?);
            reject_extra_wave_fields(line_number, parts)?;
            Ok(HumanMode::CarriedBy(actor))
        }
        _ => Err(ActorWaveScriptParseError::new(
            line_number,
            format!("unknown human mode `{mode}`"),
        )),
    }
}

fn parse_wave_behavior_preset_name(
    line_number: usize,
    token: Option<&str>,
) -> Result<String, ActorWaveScriptParseError> {
    let token = token.ok_or_else(|| {
        ActorWaveScriptParseError::new(line_number, "missing behavior preset name")
    })?;
    let name = normalize_script_token(token);
    if name.is_empty() {
        return Err(ActorWaveScriptParseError::new(
            line_number,
            "missing behavior preset name",
        ));
    }
    Ok(name)
}

fn parse_source_wave_profile_updates<'a>(
    line_number: usize,
    source: &mut ActorSourceWaveProfile,
    mut parts: impl Iterator<Item = &'a str>,
) -> Result<(), ActorWaveScriptParseError> {
    while let Some(field) = parts.next() {
        let value = parts.next().ok_or_else(|| {
            ActorWaveScriptParseError::new(
                line_number,
                format!("source wave field `{field}` needs a value"),
            )
        })?;
        apply_source_wave_profile_field(line_number, source, field, value)?;
    }
    Ok(())
}

fn apply_source_wave_profile_field(
    line_number: usize,
    source: &mut ActorSourceWaveProfile,
    field: &str,
    value: &str,
) -> Result<(), ActorWaveScriptParseError> {
    match normalize_script_token(field).as_str() {
        "landers" => source.landers = parse_wave_u8(line_number, Some(value), field)?,
        "bombers" => source.bombers = parse_wave_u8(line_number, Some(value), field)?,
        "pods" => source.pods = parse_wave_u8(line_number, Some(value), field)?,
        "mutants" => source.mutants = parse_wave_u8(line_number, Some(value), field)?,
        "swarmers" => source.swarmers = parse_wave_u8(line_number, Some(value), field)?,
        "wave_size" => source.wave_size = parse_wave_u8(line_number, Some(value), field)?,
        "lander_x_velocity" => {
            source.lander_x_velocity = parse_wave_u8(line_number, Some(value), field)?
        }
        "lander_y_velocity_msb" => {
            source.lander_y_velocity_msb = parse_wave_u8(line_number, Some(value), field)?
        }
        "lander_y_velocity_lsb" => {
            source.lander_y_velocity_lsb = parse_wave_u8(line_number, Some(value), field)?
        }
        "bomber_x_velocity" => {
            source.bomber_x_velocity = parse_wave_u8(line_number, Some(value), field)?
        }
        "swarmer_x_velocity" => {
            source.swarmer_x_velocity = parse_wave_u8(line_number, Some(value), field)?
        }
        "swarmer_shot_time" => {
            source.swarmer_shot_time = parse_wave_u32(line_number, Some(value), field)?
        }
        "swarmer_acceleration_mask" => {
            source.swarmer_acceleration_mask = parse_wave_u8(line_number, Some(value), field)?
        }
        "baiter_time" | "baiter_delay" => {
            source.baiter_delay = parse_wave_u32(line_number, Some(value), field)?
        }
        "baiter_shot_time" => {
            source.baiter_shot_time = parse_wave_u32(line_number, Some(value), field)?
        }
        "baiter_seek_probability" => {
            source.baiter_seek_probability = parse_wave_u8(line_number, Some(value), field)?
        }
        "lander_shot_time" => {
            source.lander_shot_time = parse_wave_u32(line_number, Some(value), field)?
        }
        "mutant_random_y" => {
            source.mutant_random_y = parse_wave_u8(line_number, Some(value), field)?
        }
        "mutant_y_velocity_msb" => {
            source.mutant_y_velocity_msb = parse_wave_u8(line_number, Some(value), field)?
        }
        "mutant_y_velocity_lsb" => {
            source.mutant_y_velocity_lsb = parse_wave_u8(line_number, Some(value), field)?
        }
        "mutant_x_velocity" => {
            source.mutant_x_velocity = parse_wave_u8(line_number, Some(value), field)?
        }
        "mutant_shot_time" => {
            source.mutant_shot_time = parse_wave_u32(line_number, Some(value), field)?
        }
        _ => {
            return Err(ActorWaveScriptParseError::new(
                line_number,
                format!("unknown source wave field `{field}`"),
            ));
        }
    }
    Ok(())
}

fn parse_wave_actor_kind(
    line_number: usize,
    token: Option<&str>,
) -> Result<ActorKind, ActorWaveScriptParseError> {
    let token =
        token.ok_or_else(|| ActorWaveScriptParseError::new(line_number, "missing actor kind"))?;
    parse_behavior_actor_kind(line_number, token)
        .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))
}

fn parse_wave_usize(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<usize, ActorWaveScriptParseError> {
    let value = parse_wave_u64(line_number, token, field)?;
    usize::try_from(value).map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_wave_u8(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u8, ActorWaveScriptParseError> {
    let value = parse_wave_u64(line_number, token, field)?;
    u8::try_from(value).map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_wave_u16(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u16, ActorWaveScriptParseError> {
    let value = parse_wave_u64(line_number, token, field)?;
    u16::try_from(value).map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_wave_u32(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u32, ActorWaveScriptParseError> {
    let value = parse_wave_u64(line_number, token, field)?;
    u32::try_from(value).map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_wave_i16(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<i16, ActorWaveScriptParseError> {
    let token = token
        .ok_or_else(|| ActorWaveScriptParseError::new(line_number, format!("missing {field}")))?;
    i16::try_from(parse_wave_i64_value(line_number, token, field)?).map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{token}` is invalid: {error}"),
        )
    })
}

fn parse_wave_u64(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u64, ActorWaveScriptParseError> {
    let token = token
        .ok_or_else(|| ActorWaveScriptParseError::new(line_number, format!("missing {field}")))?;
    parse_wave_u64_value(line_number, token, field)
}

fn parse_wave_u64_value(
    line_number: usize,
    value: &str,
    field: &str,
) -> Result<u64, ActorWaveScriptParseError> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16).map_err(|error| {
            ActorWaveScriptParseError::new(
                line_number,
                format!("{field} `{value}` is invalid: {error}"),
            )
        });
    }
    value.parse::<u64>().map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_wave_i64_value(
    line_number: usize,
    value: &str,
    field: &str,
) -> Result<i64, ActorWaveScriptParseError> {
    if let Some(hex) = value
        .strip_prefix("-0x")
        .or_else(|| value.strip_prefix("-0X"))
    {
        return i64::from_str_radix(hex, 16)
            .map(|value| -value)
            .map_err(|error| {
                ActorWaveScriptParseError::new(
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
            ActorWaveScriptParseError::new(
                line_number,
                format!("{field} `{value}` is invalid: {error}"),
            )
        });
    }
    value.parse::<i64>().map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn reject_extra_wave_fields<'a>(
    line_number: usize,
    mut parts: impl Iterator<Item = &'a str>,
) -> Result<(), ActorWaveScriptParseError> {
    if let Some(extra) = parts.next() {
        Err(ActorWaveScriptParseError::new(
            line_number,
            format!("unexpected extra field `{extra}`"),
        ))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveScriptManifest {
    pub name: String,
    pub behavior_presets: Vec<ActorWaveBehaviorPresetManifest>,
    pub spawn_behavior_presets: Vec<ActorWaveSpawnBehaviorPresetManifest>,
    pub waves: Vec<ActorWaveProfileManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveBehaviorPresetManifest {
    pub name: String,
    pub updates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveSpawnBehaviorPresetManifest {
    pub name: String,
    pub updates: Vec<ActorWaveSpawnBehaviorPresetUpdateManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveSpawnBehaviorPresetUpdateManifest {
    pub field: String,
    pub values: Vec<String>,
}

impl Default for ActorWaveScript {
    fn default() -> Self {
        Self::default_progression()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteKey {
    WilliamsLogo,
    DefenderCoalescence,
    DefenderWordmark,
    DefenderLogo,
    HighScoreText,
    PlayerRight,
    PlayerLeft,
    Lander,
    Mutant,
    Bomber,
    Bomb,
    Pod,
    Swarmer,
    Baiter,
    Human,
    HumanFalling,
    HumanCarried,
    Laser,
    EnemyLaser,
    Explosion,
    Score250,
    Score500,
    Text,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VisualEffect {
    #[default]
    Static,
    SourceMessage {
        top_left_screen_address: u16,
        visual_offset: Point,
    },
    AttractScoringSurface {
        scoring_tick: u16,
    },
    WilliamsReveal {
        stroke_step: u16,
        color_phase: u8,
    },
    DefenderCoalescence {
        slot: u8,
        row_pair: u8,
    },
    SourceLanderFrame {
        frame: u8,
    },
    SourceBomberFrame {
        frame: u8,
    },
    SourcePod,
    SourceBaiterFrame {
        frame: u8,
    },
    SourceHumanFrame {
        frame: u8,
    },
    ExplosionCloud {
        kind: ExplosionKind,
        age: u16,
        source_center: Option<Point>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionKind {
    Lander,
    Mutant,
    Bomber,
    Pod,
    Swarmer,
    Baiter,
    Bomb,
    Player,
    Human,
    Terrain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCue {
    Credit,
    Start,
    Thrust,
    Laser,
    SmartBomb,
    Hyperspace,
    PlayerAppear,
    HyperspaceMaterialize,
    Explosion,
    LanderHit,
    LanderPickup,
    HumanPulled,
    HumanReleased,
    HumanRescued,
    HumanSafeLanding,
    HumanLost,
    MutantSpawn,
    MutantHit,
    BomberHit,
    BombHit,
    PodHit,
    SwarmerHit,
    LanderShot,
    MutantShot,
    SwarmerShot,
    BaiterHit,
    BaiterShot,
    AttractPulse,
    GameOver,
    SourceCommand(u8),
}

impl SoundCue {
    pub const fn source_sound_command(self) -> Option<u8> {
        match self {
            Self::Credit => Some(0xE6),
            Self::Start => Some(0xF5),
            Self::Thrust => Some(0xE9),
            Self::Laser => Some(0xEB),
            Self::SmartBomb => Some(0xEE),
            Self::Hyperspace => None,
            Self::PlayerAppear => Some(0xEA),
            Self::HyperspaceMaterialize => Some(0xEA),
            Self::Explosion => Some(0xEE),
            Self::LanderHit => Some(0xF9),
            Self::LanderPickup => Some(0xF4),
            Self::HumanPulled => Some(0xF1),
            Self::HumanReleased => Some(SOURCE_ASCSND_SOUND_COMMAND),
            Self::HumanRescued => Some(SOURCE_ACSND_SOUND_COMMAND),
            Self::HumanSafeLanding => Some(0xE0),
            Self::HumanLost => Some(0xEE),
            Self::MutantSpawn => Some(0xEE),
            Self::MutantHit => Some(0xE8),
            Self::BomberHit => Some(0xFE),
            Self::BombHit => Some(0xEE),
            Self::PodHit => Some(0xFA),
            Self::SwarmerHit => Some(0xF8),
            Self::LanderShot => Some(0xFC),
            Self::MutantShot => Some(0xF6),
            Self::SwarmerShot => Some(0xF3),
            Self::BaiterHit => Some(0xF8),
            Self::BaiterShot => Some(0xFC),
            Self::AttractPulse => None,
            Self::GameOver => Some(0xEC),
            Self::SourceCommand(command) => Some(command),
        }
    }

    pub const fn sound_event(self) -> Option<SoundEvent> {
        match self {
            Self::Credit => Some(SoundEvent::CreditAdded),
            Self::Start => Some(SoundEvent::GameStarted),
            Self::Thrust => Some(SoundEvent::ThrustStarted),
            _ => match self.source_sound_command() {
                Some(command) => Some(SoundEvent::UnmappedSoundCommand { command }),
                None => None,
            },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActorSoundEventBridge {
    thrust_active: bool,
}

impl ActorSoundEventBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sound_events_for_report(&mut self, report: &StepReport) -> Vec<SoundEvent> {
        self.sound_events_for_cues(&report.sounds)
    }

    pub fn sound_events_for_cues(&mut self, cues: &[SoundCue]) -> Vec<SoundEvent> {
        let mut events = Vec::new();
        let mut thrust_requested = false;
        for cue in cues.iter().copied() {
            if cue == SoundCue::Thrust {
                if !thrust_requested && !self.thrust_active {
                    events.push(SoundEvent::ThrustStarted);
                }
                thrust_requested = true;
                continue;
            }

            if let Some(event) = cue.sound_event() {
                events.push(event);
            }
        }

        if !thrust_requested && self.thrust_active {
            events.push(SoundEvent::ThrustStopped);
        }
        self.thrust_active = thrust_requested;
        events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScript {
    events: Vec<AttractScriptEvent>,
    cycle_steps: Option<u64>,
}

impl AttractScript {
    pub fn new(events: Vec<AttractScriptEvent>) -> Self {
        Self::with_cycle_steps(events, None)
    }

    pub fn with_cycle_steps(mut events: Vec<AttractScriptEvent>, cycle_steps: Option<u64>) -> Self {
        events.sort_by_key(|event| event.start_after_steps);
        Self {
            events,
            cycle_steps: cycle_steps.filter(|steps| *steps > 0),
        }
    }

    pub fn parse_text(source: &str) -> Result<Self, AttractScriptParseError> {
        source.parse()
    }

    pub fn red_label_title() -> Self {
        Self::parse_text(ACTOR_RED_LABEL_ATTRACT_SCRIPT)
            .unwrap_or_else(|error| panic!("embedded actor attract script is invalid: {error}"))
    }

    pub fn arcade_title() -> Self {
        Self::red_label_title()
    }

    pub fn red_label_title_from_events() -> Self {
        let mut events = vec![
            AttractScriptEvent::williams_logo(
                1,
                Some(SOURCE_ATTRACT_WILLIAMS_LOGO_DURATION_STEPS),
                SOURCE_ATTRACT_WILLIAMS_LOGO_POSITION,
            ),
            AttractScriptEvent::source_message(
                SOURCE_ATTRACT_PRESENTS_START_STEP,
                Some(SOURCE_ATTRACT_PRESENTS_DURATION_STEPS),
                SOURCE_PRESENTS_MESSAGE_LABEL,
                SOURCE_ATTRACT_PRESENTS_ELECTRONICS_SCREEN,
            ),
            AttractScriptEvent::defender_wordmark(
                DEFENDER_WORDMARK_START_STEP,
                Some(SOURCE_ATTRACT_DEFENDER_WORDMARK_DURATION_STEPS),
                SOURCE_ATTRACT_DEFENDER_WORDMARK_POSITION,
            ),
            AttractScriptEvent::credits_when_nonzero(
                1,
                Some(SOURCE_ATTRACT_HALL_OF_FAME_START_STEP.saturating_sub(1)),
                SOURCE_ATTRACT_CREDIT_LABEL_POSITION,
                SOURCE_ATTRACT_CREDIT_COUNT_POSITION,
            ),
            AttractScriptEvent::credits(
                SOURCE_ATTRACT_HALL_OF_FAME_START_STEP,
                None,
                SOURCE_ATTRACT_CREDIT_LABEL_POSITION,
                SOURCE_ATTRACT_CREDIT_COUNT_POSITION,
            ),
            AttractScriptEvent::source_message_with_offset(
                SOURCE_ATTRACT_HALL_OF_FAME_START_STEP,
                Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                SOURCE_ATTRACT_HALL_TITLE_LABEL,
                0x3854,
                SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
            AttractScriptEvent::source_message_with_offset(
                SOURCE_ATTRACT_HALL_OF_FAME_START_STEP,
                Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                SOURCE_ATTRACT_HALL_TODAYS_LABEL,
                0x2268,
                SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
            AttractScriptEvent::source_message_with_offset(
                SOURCE_ATTRACT_HALL_OF_FAME_START_STEP,
                Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                SOURCE_ATTRACT_HALL_ALL_TIME_LABEL,
                0x6068,
                SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
            AttractScriptEvent::source_message_with_offset(
                SOURCE_ATTRACT_HALL_OF_FAME_START_STEP,
                Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                SOURCE_ATTRACT_HALL_GREATEST_LABEL,
                0x1E72,
                SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
            AttractScriptEvent::source_message_with_offset(
                SOURCE_ATTRACT_HALL_OF_FAME_START_STEP,
                Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                SOURCE_ATTRACT_HALL_GREATEST_LABEL,
                0x5F72,
                SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
            AttractScriptEvent::sprite(
                SOURCE_ATTRACT_HALL_OF_FAME_START_STEP,
                Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                SpriteKey::DefenderLogo,
                SOURCE_ATTRACT_HALL_DEFENDER_LOGO_POSITION,
            ),
            AttractScriptEvent::hall_scores(
                SOURCE_ATTRACT_HALL_OF_FAME_START_STEP,
                Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                SOURCE_ATTRACT_HALL_TODAYS_TABLE_SCREEN,
                SOURCE_ATTRACT_HALL_ALL_TIME_TABLE_SCREEN,
                SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
        ];
        events.push(AttractScriptEvent::scoring_surface(
            SOURCE_ATTRACT_SCORING_SEQUENCE_START_STEP,
            None,
        ));
        for (line_index, (label, screen_address)) in SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES
            .iter()
            .copied()
            .enumerate()
        {
            events.push(AttractScriptEvent::source_message_with_offset(
                actor_attract_scoring_instruction_text_start_step(line_index),
                None,
                label,
                screen_address,
                SOURCE_ATTRACT_SCORING_VISUAL_OFFSET,
            ));
        }
        Self::with_cycle_steps(events, Some(SOURCE_ATTRACT_CYCLE_STEPS))
    }

    fn draws_for(
        &self,
        actor: ActorId,
        step: u64,
        high_scores: &[u32; 5],
        credits: u8,
    ) -> Vec<DrawCommand> {
        let step = self.cycled_step(step);
        self.events
            .iter()
            .filter(|event| event.active_at(step))
            .flat_map(|event| event.draws(actor, step, high_scores, credits))
            .collect()
    }

    pub fn manifest(&self) -> AttractScriptManifest {
        AttractScriptManifest {
            cycle_steps: self.cycle_steps,
            events: self
                .events
                .iter()
                .map(AttractScriptEvent::manifest)
                .collect(),
        }
    }

    fn cycled_step(&self, step: u64) -> u64 {
        let Some(cycle_steps) = self.cycle_steps else {
            return step;
        };
        let wrapped = step % cycle_steps;
        if wrapped == 0 { 1 } else { wrapped }
    }
}

impl FromStr for AttractScript {
    type Err = AttractScriptParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let mut events = Vec::new();
        let mut cycle_steps = None;
        for (line_index, raw_line) in source.lines().enumerate() {
            let line_number = line_index + 1;
            let line = raw_line
                .split_once('#')
                .map_or(raw_line, |(before_comment, _)| before_comment)
                .trim();
            if line.is_empty() {
                continue;
            }
            if let Some(cycle) = parse_attract_script_cycle_directive(line_number, line)? {
                if cycle_steps.replace(cycle).is_some() {
                    return Err(AttractScriptParseError::new(
                        line_number,
                        "duplicate cycle directive",
                    ));
                }
                continue;
            }
            events.push(parse_attract_script_event(line_number, line)?);
        }
        Ok(Self::with_cycle_steps(events, cycle_steps))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScriptParseError {
    pub line: usize,
    pub message: String,
}

impl AttractScriptParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for AttractScriptParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "attract script line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for AttractScriptParseError {}

fn parse_attract_script_cycle_directive(
    line_number: usize,
    line: &str,
) -> Result<Option<u64>, AttractScriptParseError> {
    let mut parts = line.split_whitespace();
    let action = parts
        .next()
        .ok_or_else(|| AttractScriptParseError::new(line_number, "missing action"))?;
    if !matches!(
        normalize_script_token(action).as_str(),
        "cycle" | "loop" | "repeat"
    ) {
        return Ok(None);
    }

    let cycle_steps = parse_attract_u64(line_number, parts.next(), "cycle steps")?;
    if cycle_steps == 0 {
        return Err(AttractScriptParseError::new(
            line_number,
            "cycle steps must be greater than zero",
        ));
    }
    reject_extra_attract_fields(line_number, parts)?;
    Ok(Some(cycle_steps))
}

fn parse_attract_script_event(
    line_number: usize,
    line: &str,
) -> Result<AttractScriptEvent, AttractScriptParseError> {
    let mut parts = line.split_whitespace();
    let action = parts
        .next()
        .ok_or_else(|| AttractScriptParseError::new(line_number, "missing action"))?;
    let start_after_steps = parse_attract_u64(line_number, parts.next(), "start step")?;
    let duration_steps = parse_attract_duration(line_number, parts.next())?;

    match normalize_script_token(action).as_str() {
        "text" => {
            let position = parse_attract_point(line_number, &mut parts)?;
            let value = parts.collect::<Vec<_>>().join(" ");
            if value.is_empty() {
                return Err(AttractScriptParseError::new(
                    line_number,
                    "text action needs a value",
                ));
            }
            Ok(AttractScriptEvent::text(
                start_after_steps,
                duration_steps,
                position,
                value,
            ))
        }
        "message" | "source_message" => {
            let label = parse_attract_source_message_label(line_number, parts.next())?;
            let top_left_screen_address =
                parse_attract_u16(line_number, parts.next(), "top-left screen address")?;
            let visual_offset = parse_optional_attract_point(line_number, &mut parts)?;
            Ok(AttractScriptEvent::source_message_with_offset(
                start_after_steps,
                duration_steps,
                label,
                top_left_screen_address,
                visual_offset,
            ))
        }
        "scoring_surface" | "scoring" | "attract_scoring" => {
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::scoring_surface(
                start_after_steps,
                duration_steps,
            ))
        }
        "sprite" => {
            let sprite_token = parts.next().ok_or_else(|| {
                AttractScriptParseError::new(line_number, "sprite action needs a sprite key")
            })?;
            let sprite = parse_attract_sprite_key(line_number, sprite_token)?;
            let position = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::sprite(
                start_after_steps,
                duration_steps,
                sprite,
                position,
            ))
        }
        "williams" | "williams_logo" => {
            let position = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::williams_logo(
                start_after_steps,
                duration_steps,
                position,
            ))
        }
        "defender" | "defender_wordmark" | "wordmark" => {
            let position = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::defender_wordmark(
                start_after_steps,
                duration_steps,
                position,
            ))
        }
        "high_scores" | "high_score_table" => {
            let position = parse_attract_point(line_number, &mut parts)?;
            let row_height = parse_attract_i16(line_number, parts.next(), "row height")?;
            let rows = parse_attract_usize(line_number, parts.next(), "rows")?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::high_scores(
                start_after_steps,
                duration_steps,
                position,
                row_height,
                rows,
            ))
        }
        "hall_scores" | "hall_of_fame_scores" | "hall_of_fame_table" => {
            let todays_top_left_screen_address =
                parse_attract_u16(line_number, parts.next(), "today's table screen address")?;
            let all_time_top_left_screen_address =
                parse_attract_u16(line_number, parts.next(), "all-time table screen address")?;
            let visual_offset = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::hall_scores(
                start_after_steps,
                duration_steps,
                todays_top_left_screen_address,
                all_time_top_left_screen_address,
                visual_offset,
            ))
        }
        "credits" | "credit_count" => {
            let label_position = parse_attract_point(line_number, &mut parts)?;
            let count_position = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::credits(
                start_after_steps,
                duration_steps,
                label_position,
                count_position,
            ))
        }
        "credits_nonzero" | "credit_count_nonzero" | "credits_when_nonzero" => {
            let label_position = parse_attract_point(line_number, &mut parts)?;
            let count_position = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::credits_when_nonzero(
                start_after_steps,
                duration_steps,
                label_position,
                count_position,
            ))
        }
        _ => Err(AttractScriptParseError::new(
            line_number,
            format!("unknown action `{action}`"),
        )),
    }
}

fn parse_attract_point<'a>(
    line_number: usize,
    parts: &mut impl Iterator<Item = &'a str>,
) -> Result<Point, AttractScriptParseError> {
    let x = parse_attract_i16(line_number, parts.next(), "x")?;
    let y = parse_attract_i16(line_number, parts.next(), "y")?;
    Ok(Point::new(x, y))
}

fn parse_optional_attract_point<'a>(
    line_number: usize,
    parts: &mut impl Iterator<Item = &'a str>,
) -> Result<Point, AttractScriptParseError> {
    let Some(x_token) = parts.next() else {
        return Ok(Point::new(0, 0));
    };
    let x = parse_attract_i16(line_number, Some(x_token), "offset x")?;
    let y = parse_attract_i16(line_number, parts.next(), "offset y")?;
    reject_extra_attract_fields(line_number, parts.by_ref())?;
    Ok(Point::new(x, y))
}

fn parse_attract_duration(
    line_number: usize,
    token: Option<&str>,
) -> Result<Option<u64>, AttractScriptParseError> {
    let token =
        token.ok_or_else(|| AttractScriptParseError::new(line_number, "missing duration"))?;
    if token == "-" {
        return Ok(None);
    }
    match normalize_script_token(token).as_str() {
        "none" | "forever" | "infinite" => Ok(None),
        _ => Ok(Some(parse_attract_u64(
            line_number,
            Some(token),
            "duration",
        )?)),
    }
}

fn parse_attract_u64(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u64, AttractScriptParseError> {
    let token = token
        .ok_or_else(|| AttractScriptParseError::new(line_number, format!("missing {field}")))?;
    token.parse::<u64>().map_err(|error| {
        AttractScriptParseError::new(
            line_number,
            format!("{field} `{token}` is invalid: {error}"),
        )
    })
}

fn parse_attract_i16(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<i16, AttractScriptParseError> {
    let token = token
        .ok_or_else(|| AttractScriptParseError::new(line_number, format!("missing {field}")))?;
    token.parse::<i16>().map_err(|error| {
        AttractScriptParseError::new(
            line_number,
            format!("{field} `{token}` is invalid: {error}"),
        )
    })
}

fn parse_attract_usize(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<usize, AttractScriptParseError> {
    let token = token
        .ok_or_else(|| AttractScriptParseError::new(line_number, format!("missing {field}")))?;
    token.parse::<usize>().map_err(|error| {
        AttractScriptParseError::new(
            line_number,
            format!("{field} `{token}` is invalid: {error}"),
        )
    })
}

fn parse_attract_u16(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u16, AttractScriptParseError> {
    let token = token
        .ok_or_else(|| AttractScriptParseError::new(line_number, format!("missing {field}")))?;
    let parsed = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
        .map_or_else(|| token.parse::<u16>(), |hex| u16::from_str_radix(hex, 16))
        .map_err(|error| {
            AttractScriptParseError::new(
                line_number,
                format!("{field} `{token}` is invalid: {error}"),
            )
        })?;
    Ok(parsed)
}

fn parse_attract_source_message_label(
    line_number: usize,
    token: Option<&str>,
) -> Result<String, AttractScriptParseError> {
    let token = token.ok_or_else(|| {
        AttractScriptParseError::new(line_number, "message action needs a source label")
    })?;
    let label = token.to_ascii_uppercase();
    if source_message_text(&label).is_none() {
        return Err(AttractScriptParseError::new(
            line_number,
            format!("unknown source message label `{token}`"),
        ));
    }
    Ok(label)
}

fn parse_attract_sprite_key(
    line_number: usize,
    token: &str,
) -> Result<SpriteKey, AttractScriptParseError> {
    match normalize_script_token(token).as_str() {
        "williams_logo" | "williams" => Ok(SpriteKey::WilliamsLogo),
        "defender_wordmark" | "wordmark" => Ok(SpriteKey::DefenderWordmark),
        "defender_logo" | "defender" => Ok(SpriteKey::DefenderLogo),
        "high_score_text" | "high_scores" => Ok(SpriteKey::HighScoreText),
        "player_right" | "player" => Ok(SpriteKey::PlayerRight),
        "player_left" => Ok(SpriteKey::PlayerLeft),
        "lander" => Ok(SpriteKey::Lander),
        "mutant" => Ok(SpriteKey::Mutant),
        "bomber" => Ok(SpriteKey::Bomber),
        "bomb" => Ok(SpriteKey::Bomb),
        "pod" => Ok(SpriteKey::Pod),
        "swarmer" => Ok(SpriteKey::Swarmer),
        "baiter" => Ok(SpriteKey::Baiter),
        "human" => Ok(SpriteKey::Human),
        "human_falling" => Ok(SpriteKey::HumanFalling),
        "human_carried" => Ok(SpriteKey::HumanCarried),
        "laser" => Ok(SpriteKey::Laser),
        "enemy_laser" => Ok(SpriteKey::EnemyLaser),
        "explosion" => Ok(SpriteKey::Explosion),
        "score250" | "score_250" => Ok(SpriteKey::Score250),
        "score500" | "score_500" => Ok(SpriteKey::Score500),
        "text" => Ok(SpriteKey::Text),
        _ => Err(AttractScriptParseError::new(
            line_number,
            format!("unknown sprite key `{token}`"),
        )),
    }
}

fn reject_extra_attract_fields<'a>(
    line_number: usize,
    mut parts: impl Iterator<Item = &'a str>,
) -> Result<(), AttractScriptParseError> {
    if let Some(extra) = parts.next() {
        Err(AttractScriptParseError::new(
            line_number,
            format!("unexpected extra field `{extra}`"),
        ))
    } else {
        Ok(())
    }
}

fn normalize_script_token(token: &str) -> String {
    token.trim().replace('-', "_").to_ascii_lowercase()
}

impl Default for AttractScript {
    fn default() -> Self {
        Self::red_label_title()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScriptEvent {
    pub start_after_steps: u64,
    pub duration_steps: Option<u64>,
    pub action: AttractScriptAction,
}

impl AttractScriptEvent {
    pub fn text(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
        value: impl Into<String>,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::Text {
                position,
                value: value.into(),
            },
        }
    }

    pub fn source_message(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        label: impl Into<String>,
        top_left_screen_address: u16,
    ) -> Self {
        Self::source_message_with_offset(
            start_after_steps,
            duration_steps,
            label,
            top_left_screen_address,
            Point::new(0, 0),
        )
    }

    pub fn source_message_with_offset(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        label: impl Into<String>,
        top_left_screen_address: u16,
        visual_offset: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::SourceMessage {
                label: label.into(),
                top_left_screen_address,
                visual_offset,
            },
        }
    }

    pub fn sprite(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        sprite: SpriteKey,
        position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::Sprite { sprite, position },
        }
    }

    pub fn williams_logo(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::WilliamsLogo {
                position,
                reveal_steps: WILLIAMS_REVEAL_STEPS,
                color_period: WILLIAMS_COLOR_PERIOD,
            },
        }
    }

    pub fn defender_wordmark(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::DefenderWordmark {
                position,
                slots: DEFENDER_WORDMARK_SLOTS,
                row_pairs: DEFENDER_WORDMARK_ROW_PAIRS,
            },
        }
    }

    pub fn high_scores(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
        row_height: i16,
        rows: usize,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::HighScores {
                position,
                row_height,
                rows,
            },
        }
    }

    pub fn hall_scores(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        todays_top_left_screen_address: u16,
        all_time_top_left_screen_address: u16,
        visual_offset: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::HallScores {
                todays_top_left_screen_address,
                all_time_top_left_screen_address,
                visual_offset,
            },
        }
    }

    pub fn scoring_surface(start_after_steps: u64, duration_steps: Option<u64>) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::ScoringSurface,
        }
    }

    pub fn credits(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        label_position: Point,
        count_position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::Credits {
                label_position,
                count_position,
                minimum_credits: 0,
            },
        }
    }

    pub fn credits_when_nonzero(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        label_position: Point,
        count_position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::Credits {
                label_position,
                count_position,
                minimum_credits: 1,
            },
        }
    }

    fn active_at(&self, step: u64) -> bool {
        if step < self.start_after_steps {
            return false;
        }
        match self.duration_steps {
            Some(duration_steps) => step < self.start_after_steps.saturating_add(duration_steps),
            None => true,
        }
    }

    fn draws(
        &self,
        actor: ActorId,
        step: u64,
        high_scores: &[u32; 5],
        credits: u8,
    ) -> Vec<DrawCommand> {
        self.action
            .draws(actor, self.age(step), high_scores, credits)
    }

    fn age(&self, step: u64) -> u64 {
        step.saturating_sub(self.start_after_steps)
            .saturating_add(1)
    }

    fn manifest(&self) -> AttractScriptEventManifest {
        AttractScriptEventManifest {
            start_after_steps: self.start_after_steps,
            duration_steps: self.duration_steps,
            action: self.action.manifest(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttractScriptAction {
    Text {
        position: Point,
        value: String,
    },
    SourceMessage {
        label: String,
        top_left_screen_address: u16,
        visual_offset: Point,
    },
    Sprite {
        sprite: SpriteKey,
        position: Point,
    },
    WilliamsLogo {
        position: Point,
        reveal_steps: u16,
        color_period: u16,
    },
    DefenderWordmark {
        position: Point,
        slots: u16,
        row_pairs: u16,
    },
    HighScores {
        position: Point,
        row_height: i16,
        rows: usize,
    },
    HallScores {
        todays_top_left_screen_address: u16,
        all_time_top_left_screen_address: u16,
        visual_offset: Point,
    },
    ScoringSurface,
    Credits {
        label_position: Point,
        count_position: Point,
        minimum_credits: u8,
    },
}

impl AttractScriptAction {
    fn draws(
        &self,
        actor: ActorId,
        age: u64,
        high_scores: &[u32; 5],
        credits: u8,
    ) -> Vec<DrawCommand> {
        match self {
            Self::Text { position, value } => {
                vec![DrawCommand::text(actor, *position, value.clone())]
            }
            Self::SourceMessage {
                label,
                top_left_screen_address,
                visual_offset,
            } => source_message_text(label).map_or_else(Vec::new, |text| {
                vec![DrawCommand::source_message_with_offset(
                    actor,
                    text,
                    *top_left_screen_address,
                    *visual_offset,
                )]
            }),
            Self::Sprite { sprite, position } => {
                vec![DrawCommand::sprite(actor, *sprite, *position)]
            }
            Self::WilliamsLogo {
                position,
                reveal_steps,
                color_period,
            } => {
                let color_period = (*color_period).max(1);
                let color_phase = ((age.saturating_sub(1) / u64::from(color_period)) % 4) as u8;
                vec![DrawCommand::sprite_with_effect(
                    actor,
                    SpriteKey::WilliamsLogo,
                    *position,
                    VisualEffect::WilliamsReveal {
                        stroke_step: (age as u16).min(*reveal_steps),
                        color_phase,
                    },
                )]
            }
            Self::DefenderWordmark {
                position,
                slots,
                row_pairs,
            } => {
                let row_pairs = (*row_pairs).max(1);
                let progress = age.saturating_sub(1) as u16;
                let total_steps = slots.saturating_mul(row_pairs);
                if progress >= total_steps {
                    vec![DrawCommand::sprite(
                        actor,
                        SpriteKey::DefenderWordmark,
                        *position,
                    )]
                } else {
                    vec![DrawCommand::sprite_with_effect(
                        actor,
                        SpriteKey::DefenderCoalescence,
                        *position,
                        VisualEffect::DefenderCoalescence {
                            slot: (progress / row_pairs) as u8,
                            row_pair: (progress % row_pairs) as u8,
                        },
                    )]
                }
            }
            Self::HighScores {
                position,
                row_height,
                rows,
            } => high_scores
                .iter()
                .copied()
                .take((*rows).min(high_scores.len()))
                .enumerate()
                .map(|(index, score)| {
                    DrawCommand::text(
                        actor,
                        Point::new(
                            position.x,
                            position.y
                                + i16::try_from(index)
                                    .unwrap_or(i16::MAX)
                                    .saturating_mul(*row_height),
                        ),
                        format!("{}. {}", index + 1, format_status_score(score)),
                    )
                })
                .collect(),
            Self::HallScores {
                todays_top_left_screen_address,
                all_time_top_left_screen_address,
                visual_offset,
            } => {
                let entries = source_hall_score_entries(high_scores);
                let mut draws = hall_score_table_draws(
                    actor,
                    entries,
                    *todays_top_left_screen_address,
                    *visual_offset,
                );
                draws.extend(hall_score_table_draws(
                    actor,
                    entries,
                    *all_time_top_left_screen_address,
                    *visual_offset,
                ));
                draws
            }
            Self::ScoringSurface => vec![DrawCommand::sprite_with_effect(
                actor,
                SpriteKey::Text,
                Point::new(0, 0),
                VisualEffect::AttractScoringSurface {
                    scoring_tick: u16::try_from(age.saturating_sub(1)).unwrap_or(u16::MAX),
                },
            )],
            Self::Credits {
                label_position,
                count_position,
                minimum_credits,
            } => {
                if credits < *minimum_credits {
                    Vec::new()
                } else {
                    vec![
                        DrawCommand::text(actor, *label_position, source_credits_label_text()),
                        DrawCommand::text(
                            actor,
                            *count_position,
                            format!("{:02}", credits.min(99)),
                        ),
                    ]
                }
            }
        }
    }

    fn manifest(&self) -> AttractScriptActionManifest {
        match self {
            Self::Text { position, value } => AttractScriptActionManifest::Text {
                position: *position,
                value: value.clone(),
            },
            Self::SourceMessage {
                label,
                top_left_screen_address,
                visual_offset,
            } => AttractScriptActionManifest::SourceMessage {
                label: label.clone(),
                top_left_screen_address: *top_left_screen_address,
                visual_offset: *visual_offset,
            },
            Self::Sprite { sprite, position } => AttractScriptActionManifest::Sprite {
                sprite: *sprite,
                position: *position,
            },
            Self::WilliamsLogo {
                position,
                reveal_steps,
                color_period,
            } => AttractScriptActionManifest::WilliamsLogo {
                position: *position,
                reveal_steps: *reveal_steps,
                color_period: *color_period,
            },
            Self::DefenderWordmark {
                position,
                slots,
                row_pairs,
            } => AttractScriptActionManifest::DefenderWordmark {
                position: *position,
                slots: *slots,
                row_pairs: *row_pairs,
            },
            Self::HighScores {
                position,
                row_height,
                rows,
            } => AttractScriptActionManifest::HighScores {
                position: *position,
                row_height: *row_height,
                rows: *rows,
            },
            Self::HallScores {
                todays_top_left_screen_address,
                all_time_top_left_screen_address,
                visual_offset,
            } => AttractScriptActionManifest::HallScores {
                todays_top_left_screen_address: *todays_top_left_screen_address,
                all_time_top_left_screen_address: *all_time_top_left_screen_address,
                visual_offset: *visual_offset,
            },
            Self::ScoringSurface => AttractScriptActionManifest::ScoringSurface,
            Self::Credits {
                label_position,
                count_position,
                minimum_credits,
            } => AttractScriptActionManifest::Credits {
                label_position: *label_position,
                count_position: *count_position,
                minimum_credits: *minimum_credits,
            },
        }
    }
}

fn source_credits_label_text() -> &'static str {
    source_message_text(SOURCE_CREDITS_MESSAGE_LABEL).unwrap_or("CREDITS:")
}

fn source_hall_score_entries(
    high_scores: &[u32; 5],
) -> [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES] {
    std::array::from_fn(|index| {
        let seed = source_high_score_seed_entry(index);
        HighScoreTableEntrySnapshot {
            rank: seed.rank,
            score: high_scores.get(index).copied().unwrap_or(seed.score),
            initials: seed.initials,
        }
    })
}

fn source_high_score_seed_entry(index: usize) -> HighScoreTableEntrySnapshot {
    let line = ACTOR_SOURCE_HIGH_SCORES_TSV
        .lines()
        .skip(1)
        .nth(index)
        .unwrap_or_else(|| panic!("missing red-label high-score seed row {index}"));
    let mut fields = line.split('\t');
    let initials = fields
        .next()
        .unwrap_or_else(|| panic!("missing red-label high-score initials row {index}"));
    let score = fields
        .next()
        .unwrap_or_else(|| panic!("missing red-label high-score value row {index}"))
        .parse::<u32>()
        .unwrap_or_else(|error| panic!("red-label high-score row {index} score: {error}"));
    HighScoreTableEntrySnapshot {
        rank: u8::try_from(index + 1).expect("red-label high-score rank fits u8"),
        score,
        initials: high_score_initials_from_seed(initials),
    }
}

fn high_score_initials_from_seed(initials: &str) -> [Option<char>; 3] {
    let mut result = [None; 3];
    for (index, character) in initials.chars().take(result.len()).enumerate() {
        if character.is_ascii_alphabetic() {
            result[index] = Some(character.to_ascii_uppercase());
        }
    }
    result
}

fn hall_score_table_draws(
    actor: ActorId,
    entries: [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES],
    top_left_screen_address: u16,
    visual_offset: Point,
) -> Vec<DrawCommand> {
    let mut draws = Vec::with_capacity(entries.len() * 3);
    for (index, entry) in entries.iter().copied().enumerate() {
        let vertical_offset = u8::try_from(index).expect("high-score table index fits in u8")
            * SOURCE_ATTRACT_HALL_TABLE_ROW_STEP;
        draws.push(DrawCommand::text(
            actor,
            offset_point(
                source_screen_point_with_offset(top_left_screen_address, 0, vertical_offset),
                visual_offset,
            ),
            char::from(b'1' + u8::try_from(index).expect("high-score rank fits u8")).to_string(),
        ));
        draws.push(DrawCommand::text(
            actor,
            offset_point(
                source_screen_point_with_offset(
                    top_left_screen_address,
                    SOURCE_ATTRACT_HALL_TABLE_INITIALS_OFFSET,
                    vertical_offset,
                ),
                visual_offset,
            ),
            hall_score_initials_text(entry.initials),
        ));
        draws.push(DrawCommand::text(
            actor,
            offset_point(
                source_screen_point_with_offset(
                    top_left_screen_address,
                    SOURCE_ATTRACT_HALL_TABLE_SCORE_OFFSET,
                    vertical_offset,
                ),
                visual_offset,
            ),
            hall_score_text(entry.score),
        ));
    }
    draws
}

fn source_screen_point_with_offset(
    top_left_screen_address: u16,
    horizontal: u8,
    vertical: u8,
) -> Point {
    let [column, row] = top_left_screen_address.to_be_bytes();
    Point::new(
        i16::from(column.wrapping_add(horizontal)) * 2,
        i16::from(row.wrapping_add(vertical)),
    )
}

fn offset_point(point: Point, offset: Point) -> Point {
    Point::new(
        point.x.saturating_add(offset.x),
        point.y.saturating_add(offset.y),
    )
}

fn hall_score_initials_text(initials: [Option<char>; 3]) -> String {
    initials
        .into_iter()
        .map(|initial| initial.unwrap_or(' '))
        .collect()
}

fn hall_score_text(score: u32) -> String {
    let mut text = [b' '; SOURCE_ATTRACT_HALL_SCORE_TEXT_LEN];
    let mut place = 100_000;
    let mut seen_non_zero = false;
    for byte in &mut text {
        let digit = ((score.min(999_999) / place) % 10) as u8;
        if digit != 0 || seen_non_zero {
            seen_non_zero = true;
            *byte = b'0' + digit;
        }
        place /= 10;
    }
    String::from_utf8_lossy(&text).into_owned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScriptManifest {
    pub cycle_steps: Option<u64>,
    pub events: Vec<AttractScriptEventManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScriptEventManifest {
    pub start_after_steps: u64,
    pub duration_steps: Option<u64>,
    pub action: AttractScriptActionManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttractScriptActionManifest {
    Text {
        position: Point,
        value: String,
    },
    SourceMessage {
        label: String,
        top_left_screen_address: u16,
        visual_offset: Point,
    },
    Sprite {
        sprite: SpriteKey,
        position: Point,
    },
    WilliamsLogo {
        position: Point,
        reveal_steps: u16,
        color_period: u16,
    },
    DefenderWordmark {
        position: Point,
        slots: u16,
        row_pairs: u16,
    },
    HighScores {
        position: Point,
        row_height: i16,
        rows: usize,
    },
    HallScores {
        todays_top_left_screen_address: u16,
        all_time_top_left_screen_address: u16,
        visual_offset: Point,
    },
    ScoringSurface,
    Credits {
        label_position: Point,
        count_position: Point,
        minimum_credits: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionBody {
    pub owner: ActorId,
    pub kind: ActorKind,
    pub position: Point,
    pub bounds: Rect,
    pub source_mutant: Option<ActorSourceMutantMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSnapshot {
    pub id: ActorId,
    pub kind: ActorKind,
    pub position: Point,
    pub velocity: Velocity,
    pub direction: Option<Direction>,
    pub bounds: Option<Rect>,
    pub alive: bool,
    pub source_lander: Option<ActorSourceLanderMetadata>,
    pub source_bomber: Option<ActorSourceBomberMetadata>,
    pub source_pod: Option<ActorSourcePodMetadata>,
    pub source_swarmer: Option<ActorSourceSwarmerMetadata>,
    pub source_baiter: Option<ActorSourceBaiterMetadata>,
    pub source_mutant: Option<ActorSourceMutantMetadata>,
    pub source_human: Option<ActorSourceHumanMetadata>,
    pub source_enemy_projectile: Option<ActorSourceEnemyProjectileMetadata>,
}

impl ActorSnapshot {
    fn collision_body(&self) -> Option<CollisionBody> {
        Some(CollisionBody {
            owner: self.id,
            kind: self.kind,
            position: self.position,
            bounds: self.bounds?,
            source_mutant: self.source_mutant,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanMode {
    Grounded,
    Falling { velocity: i16 },
    CarriedBy(ActorId),
}

impl HumanMode {
    const fn sprite(self) -> SpriteKey {
        match self {
            Self::Grounded => SpriteKey::Human,
            Self::Falling { .. } => SpriteKey::HumanFalling,
            Self::CarriedBy(_) => SpriteKey::HumanCarried,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawCommand {
    pub actor: ActorId,
    pub sprite: SpriteKey,
    pub position: Point,
    pub effect: VisualEffect,
    pub text: Option<String>,
}

impl DrawCommand {
    pub fn sprite(actor: ActorId, sprite: SpriteKey, position: Point) -> Self {
        Self::sprite_with_effect(actor, sprite, position, VisualEffect::Static)
    }

    pub fn sprite_with_effect(
        actor: ActorId,
        sprite: SpriteKey,
        position: Point,
        effect: VisualEffect,
    ) -> Self {
        Self {
            actor,
            sprite,
            position,
            effect,
            text: None,
        }
    }

    pub fn text(actor: ActorId, position: Point, value: impl Into<String>) -> Self {
        Self {
            actor,
            sprite: SpriteKey::Text,
            position,
            effect: VisualEffect::Static,
            text: Some(value.into()),
        }
    }

    pub fn source_message(
        actor: ActorId,
        value: impl Into<String>,
        top_left_screen_address: u16,
    ) -> Self {
        Self::source_message_with_offset(actor, value, top_left_screen_address, Point::new(0, 0))
    }

    pub fn source_message_with_offset(
        actor: ActorId,
        value: impl Into<String>,
        top_left_screen_address: u16,
        visual_offset: Point,
    ) -> Self {
        Self {
            actor,
            sprite: SpriteKey::Text,
            position: Point::new(0, 0),
            effect: VisualEffect::SourceMessage {
                top_left_screen_address,
                visual_offset,
            },
            text: Some(value.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnRequest {
    Laser {
        position: Point,
        direction: Direction,
        owner: ActorId,
    },
    EnemyLaser {
        position: Point,
        velocity: Velocity,
        source: Option<ActorSourceEnemyProjectileMetadata>,
    },
    Lander {
        position: Point,
    },
    Mutant {
        position: Point,
        source: Option<ActorSourceMutantMetadata>,
    },
    Bomber {
        position: Point,
    },
    Bomb {
        position: Point,
        source: Option<ActorSourceEnemyProjectileMetadata>,
    },
    Pod {
        position: Point,
    },
    Swarmer {
        position: Point,
        source: Option<ActorSourceSwarmerMetadata>,
    },
    Baiter {
        position: Point,
        source: Option<ActorSourceBaiterMetadata>,
    },
    Human {
        position: Point,
        mode: HumanMode,
    },
    Explosion {
        position: Point,
        kind: ExplosionKind,
        source_center: Option<Point>,
    },
    ScorePopup {
        position: Point,
        points: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommand {
    Credit,
    StartOnePlayer,
    StartTwoPlayer,
    Spawn(SpawnRequest),
    Destroy(ActorId),
    SetSourceBackgroundLeft(u16),
    AttachHuman {
        lander: ActorId,
        human: ActorId,
        position: Point,
    },
    SmartBomb {
        consume_stock: bool,
    },
    Hyperspace,
    HumanLost(ActorId),
    AddScore(u32),
    PlaySound(SoundCue),
    PlayerKilled,
    WaveCleared {
        next_wave: u16,
    },
    AdvanceWave {
        wave: u16,
    },
    EnterGameOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepPrompt {
    pub step: u64,
    pub phase: Phase,
    pub input: GameInput,
    pub wave: u16,
    pub source_wave: ActorSourceWaveProfile,
    pub current_player: u8,
    pub player_count: u8,
    pub score: u32,
    pub player_scores: [u32; 2],
    pub credits: u8,
    pub lives: u8,
    pub smart_bombs: u8,
    pub smart_bomb_pending: bool,
    pub player_stocks: [PlayerStockSnapshot; 2],
    pub game_over_hall_of_fame_stall_remaining: Option<u8>,
    pub player_switch: Option<PlayerSwitchReport>,
    pub player_start: Option<PlayerStartReport>,
    pub high_scores: [u32; 5],
    pub high_score_initials: HighScoreInitialsState,
    pub snapshots: Vec<ActorSnapshot>,
    pub behavior_script: ActorBehaviorScript,
    pub source_background_left: u16,
    pub source_rng: Option<ActorSourceRngSnapshot>,
    pub source_human_walk_target_slot: Option<usize>,
    pub source_shell_scan_tick: bool,
}

impl StepPrompt {
    pub fn behavior_for(&self, actor: ActorId, kind: ActorKind) -> ActorBehaviorProfile {
        self.behavior_script.behavior_for(actor, kind)
    }

    pub fn player_position(&self) -> Option<Point> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.position)
    }

    pub fn player_velocity(&self) -> Option<Velocity> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.velocity)
    }

    fn snapshot(&self, id: ActorId) -> Option<&ActorSnapshot> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.id == id && snapshot.alive)
    }

    fn nearest_human(&self, position: Point) -> Option<&ActorSnapshot> {
        self.snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
            .min_by_key(|snapshot| manhattan_distance(position, snapshot.position))
    }

    fn source_target_human(&self, target_slot_index: usize) -> Option<&ActorSnapshot> {
        self.snapshots.iter().find(|snapshot| {
            snapshot.kind == ActorKind::Human
                && snapshot.alive
                && snapshot.bounds.is_some()
                && snapshot
                    .source_human
                    .is_some_and(|source| source.target_slot_index == target_slot_index)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorReply {
    pub id: ActorId,
    pub snapshot: ActorSnapshot,
    pub commands: Vec<GameCommand>,
    pub draws: Vec<DrawCommand>,
}

trait AssetActor: Send + 'static {
    fn id(&self) -> ActorId;

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply;

    fn apply_driver_command(&mut self, _command: ActorDriverCommand) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorDriverCommand {
    AdjustSourceLanderShotTimer {
        target_human_index: usize,
        x_velocity: u16,
        delta: u8,
    },
}

enum ActorRequest {
    Prompt(Box<StepPrompt>),
    DriverCommand(ActorDriverCommand),
    Stop,
}

struct ThreadedAsset {
    sender: Sender<ActorRequest>,
    receiver: Receiver<ActorReply>,
    handle: Option<JoinHandle<()>>,
}

impl ThreadedAsset {
    fn spawn(actor: impl AssetActor) -> Self {
        let (request_sender, request_receiver) = mpsc::channel();
        let (reply_sender, reply_receiver) = mpsc::channel();
        let handle = thread::spawn(move || run_actor_thread(actor, request_receiver, reply_sender));
        Self {
            sender: request_sender,
            receiver: reply_receiver,
            handle: Some(handle),
        }
    }

    fn prompt(&self, prompt: StepPrompt) -> Option<ActorReply> {
        self.sender
            .send(ActorRequest::Prompt(Box::new(prompt)))
            .ok()?;
        self.receiver.recv().ok()
    }

    fn apply_driver_command(&self, command: ActorDriverCommand) {
        let _ = self.sender.send(ActorRequest::DriverCommand(command));
    }
}

impl Drop for ThreadedAsset {
    fn drop(&mut self) {
        let _ = self.sender.send(ActorRequest::Stop);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_actor_thread(
    mut actor: impl AssetActor,
    receiver: Receiver<ActorRequest>,
    sender: Sender<ActorReply>,
) {
    while let Ok(request) = receiver.recv() {
        match request {
            ActorRequest::Prompt(prompt) => {
                if sender.send(actor.update(prompt.as_ref())).is_err() {
                    break;
                }
            }
            ActorRequest::DriverCommand(command) => {
                actor.apply_driver_command(command);
            }
            ActorRequest::Stop => break,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepReport {
    pub step: u64,
    pub phase: Phase,
    pub wave: u16,
    pub current_player: u8,
    pub player_count: u8,
    pub score: u32,
    pub player_scores: [u32; 2],
    pub credits: u8,
    pub lives: u8,
    pub smart_bombs: u8,
    pub smart_bomb_flash_steps_remaining: u8,
    pub player_stocks: [PlayerStockSnapshot; 2],
    pub next_bonus: u32,
    pub game_over_hall_of_fame_stall_remaining: Option<u8>,
    pub player_switch: Option<PlayerSwitchReport>,
    pub player_start: Option<PlayerStartReport>,
    pub high_scores: [u32; 5],
    pub source_wave: ActorSourceWaveProfile,
    pub high_score_initials: HighScoreInitialsState,
    pub high_score_initial_accepted: bool,
    pub high_score_submitted: bool,
    pub bonus_awarded: bool,
    pub survivor_bonus: Option<SurvivorBonusReport>,
    pub behavior_script: ActorBehaviorScriptManifest,
    pub enemy_reserve: EnemyReserveSnapshot,
    pub source_background_left: u16,
    pub source_rng: Option<ActorSourceRngSnapshot>,
    pub terrain_blow: Option<TerrainBlowSnapshot>,
    pub snapshots: Vec<ActorSnapshot>,
    pub draws: Vec<DrawCommand>,
    pub sounds: Vec<SoundCue>,
    pub commands: Vec<GameCommand>,
}

impl StepReport {
    pub fn sound_events(&self, bridge: &mut ActorSoundEventBridge) -> Vec<SoundEvent> {
        bridge.sound_events_for_report(self)
    }

    pub fn render_scene(&self) -> RenderScene {
        ActorRenderSceneBridge::new().render_scene_for_report(self)
    }

    pub fn render_scene_with(&self, bridge: &ActorRenderSceneBridge) -> RenderScene {
        bridge.render_scene_for_report(self)
    }

    pub fn game_state(&self) -> GameState {
        ActorStateBridge::new().state_for_report(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurvivorBonusReport {
    pub next_wave: u16,
    pub multiplier: u8,
    pub total_survivors: u8,
    pub visible_icons: u8,
    pub remaining_awards: u8,
    pub awarded_points: Option<u32>,
    pub astronaut_sleep_steps_remaining: u8,
    pub wave_advance_sleep_steps_remaining: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSwitchReport {
    pub sleep_steps_remaining: u8,
    pub from_player: u8,
    pub to_player: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStartReport {
    pub delay_steps_remaining: u8,
    pub player: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActorStateBridge;

impl ActorStateBridge {
    pub const fn new() -> Self {
        Self
    }

    pub fn state_for_report(&self, report: &StepReport) -> GameState {
        let phase = clean_phase(report.phase);
        let wave = clean_wave(report.wave);
        let high_score_tables = high_score_tables_for_report(report);
        GameState {
            frame: report.step,
            phase,
            credits: report.credits,
            current_player: report.current_player,
            player_count: report.player_count,
            wave,
            wave_profile: actor_wave_profile_for_report(report),
            player: player_snapshot_for_report(report),
            player_stocks: report.player_stocks,
            scores: ScoreSnapshot {
                player_one: report.player_scores[0],
                player_two: report.player_scores[1],
                high_score: report.high_scores[0]
                    .max(report.player_scores[0])
                    .max(report.player_scores[1]),
                next_bonus: report.next_bonus,
            },
            attract: attract_snapshot_for_report(report),
            post_game_playfield: None,
            high_score_initials: report.high_score_initials,
            high_score_entry: high_score_entry_for_report(report),
            high_score_submission: None,
            high_score_tables,
            game_over: game_over_snapshot_for_report(report),
            world: world_snapshot_for_report(report),
        }
    }
}

fn clean_phase(phase: Phase) -> GamePhase {
    match phase {
        Phase::Attract => GamePhase::Attract,
        Phase::Playing => GamePhase::Playing,
        Phase::GameOver => GamePhase::GameOver,
        Phase::HighScoreEntry => GamePhase::HighScoreEntry,
    }
}

fn clean_wave(wave: u16) -> u8 {
    u8::try_from(wave.max(1)).unwrap_or(u8::MAX)
}

fn actor_wave_profile_for_report(report: &StepReport) -> WaveProfileSnapshot {
    let mut profile = WaveProfileSnapshot::for_wave(clean_wave(report.wave));
    let source = report.source_wave;
    profile.landers = source.landers;
    profile.bombers = source.bombers;
    profile.pods = source.pods;
    profile.mutants = source.mutants;
    profile.swarmers = source.swarmers;
    profile.lander_x_velocity = source.lander_x_velocity;
    profile.lander_y_velocity_msb = source.lander_y_velocity_msb;
    profile.lander_y_velocity_lsb = source.lander_y_velocity_lsb;
    profile.mutant_random_y = source.mutant_random_y;
    profile.mutant_y_velocity_msb = source.mutant_y_velocity_msb;
    profile.mutant_y_velocity_lsb = source.mutant_y_velocity_lsb;
    profile.mutant_x_velocity = source.mutant_x_velocity;
    profile.swarmer_x_velocity = source.swarmer_x_velocity;
    profile.wave_size = source.wave_size;
    profile.lander_shot_time = source.lander_shot_time;
    profile.bomber_x_velocity = source.bomber_x_velocity;
    profile.mutant_shot_time = source.mutant_shot_time;
    profile.swarmer_shot_time = source.swarmer_shot_time;
    profile.swarmer_acceleration_mask = source.swarmer_acceleration_mask;
    profile.baiter_delay = source.baiter_delay;
    profile.baiter_shot_time = source.baiter_shot_time;
    profile.baiter_seek_probability = source.baiter_seek_probability;
    profile
}

fn attract_snapshot_for_report(report: &StepReport) -> AttractPresentationSnapshot {
    if report.phase == Phase::Attract {
        AttractPresentationSnapshot::for_page_frame(u16::try_from(report.step).unwrap_or(u16::MAX))
    } else {
        AttractPresentationSnapshot::INACTIVE
    }
}

fn player_snapshot_for_report(report: &StepReport) -> PlayerSnapshot {
    let snapshot = report
        .snapshots
        .iter()
        .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive);
    let position = snapshot
        .map(|snapshot| snapshot.position)
        .unwrap_or_default();
    let velocity = snapshot
        .map(|snapshot| snapshot.velocity)
        .unwrap_or_default();
    PlayerSnapshot {
        position: (world_vector(position.x), world_vector(position.y)),
        velocity: (world_vector(velocity.dx), world_vector(velocity.dy)),
        direction: snapshot
            .and_then(|snapshot| snapshot.direction)
            .map(clean_direction)
            .unwrap_or_else(|| player_direction_for_report(report)),
        lives: report.lives,
        smart_bombs: report.smart_bombs,
    }
}

fn clean_direction(direction: Direction) -> CleanDirection {
    match direction {
        Direction::Left => CleanDirection::Left,
        Direction::Right => CleanDirection::Right,
    }
}

fn player_direction_for_report(report: &StepReport) -> CleanDirection {
    report
        .draws
        .iter()
        .rev()
        .find_map(|draw| match draw.sprite {
            SpriteKey::PlayerLeft => Some(CleanDirection::Left),
            SpriteKey::PlayerRight => Some(CleanDirection::Right),
            _ => None,
        })
        .unwrap_or(CleanDirection::Right)
}

fn high_score_tables_for_report(report: &StepReport) -> HighScoreTablesSnapshot {
    let entries = source_hall_score_entries(&report.high_scores);
    HighScoreTablesSnapshot {
        all_time: entries,
        todays_greatest: entries,
    }
}

fn high_score_entry_for_report(report: &StepReport) -> Option<HighScoreEntrySnapshot> {
    if report.phase != Phase::HighScoreEntry {
        return None;
    }

    report
        .high_scores
        .iter()
        .position(|score| *score == report.score)
        .map(|index| HighScoreEntrySnapshot {
            score: report.score,
            rank: u8::try_from(index + 1).expect("actor high-score rank should fit u8"),
        })
}

fn game_over_snapshot_for_report(report: &StepReport) -> GameOverSnapshot {
    if let Some(player_switch) = report.player_switch {
        return GameOverSnapshot {
            player_switch_sleep_remaining: Some(player_switch.sleep_steps_remaining),
            player_switch_from: Some(player_switch.from_player),
            player_switch_to: Some(player_switch.to_player),
            ..GameOverSnapshot::NONE
        };
    }

    let Some(remaining) = report.game_over_hall_of_fame_stall_remaining else {
        return GameOverSnapshot::NONE;
    };

    GameOverSnapshot {
        hall_of_fame_stall_remaining: Some(remaining),
        ..GameOverSnapshot::NONE
    }
}

fn world_snapshot_for_report(report: &StepReport) -> WorldSnapshot {
    if report.player_start.is_some() {
        return WorldSnapshot::default();
    }

    let mut world = WorldSnapshot {
        terrain: actor_playfield_terrain_segments(report),
        terrain_blow: report.terrain_blow,
        enemies: actor_enemies_for_report(report),
        humans: actor_humans_for_report(report),
        projectiles: actor_projectiles_for_report(report),
        enemy_projectiles: actor_enemy_projectiles_for_report(report),
        explosions: actor_explosions_for_report(report),
        score_popups: actor_score_popups_for_report(report),
        enemy_reserve: report.enemy_reserve,
        source_rng: report.source_rng.map(clean_source_rng).unwrap_or_default(),
        ..WorldSnapshot::default()
    };
    world.scanner.enabled = report.phase == Phase::Playing && report.terrain_blow.is_none();
    world
}

fn actor_playfield_terrain_segments(report: &StepReport) -> Vec<TerrainSegment> {
    if report.phase != Phase::Playing || report.terrain_blow.is_some() {
        return Vec::new();
    }

    source_playfield_terrain_segments()
}

fn source_playfield_terrain_segments() -> Vec<TerrainSegment> {
    vec![
        TerrainSegment {
            position: ScreenPosition::new(0, 224),
            size: (64, 8),
        },
        TerrainSegment {
            position: ScreenPosition::new(64, 222),
            size: (64, 8),
        },
        TerrainSegment {
            position: ScreenPosition::new(128, 226),
            size: (64, 8),
        },
        TerrainSegment {
            position: ScreenPosition::new(192, 220),
            size: (56, 8),
        },
        TerrainSegment {
            position: ScreenPosition::new(248, 224),
            size: (44, 8),
        },
    ]
}

fn actor_source_human_target_y(position_x: i16, offset: u8) -> Option<i16> {
    actor_source_terrain_altitude(position_x)
        .map(|altitude| i16::from(altitude.wrapping_add(offset).min(SOURCE_HUMAN_MAX_TARGET_Y)))
}

fn actor_source_terrain_altitude(position_x: i16) -> Option<u8> {
    let object_x = u16::from(u8::try_from(position_x).ok()?);
    source_playfield_terrain_segments()
        .into_iter()
        .find(|segment| {
            let start = u16::from(segment.position.x);
            let end = start.saturating_add(u16::from(segment.size.0));
            object_x >= start && object_x < end
        })
        .map(|segment| segment.position.y)
}

fn actor_source_step_human_y(position_y: i16, target_y: i16) -> i16 {
    match position_y.cmp(&target_y) {
        Ordering::Less => position_y + 1,
        Ordering::Equal => position_y,
        Ordering::Greater => position_y - 1,
    }
}

fn actor_enemies_for_report(report: &StepReport) -> Vec<CleanEnemySnapshot> {
    report
        .snapshots
        .iter()
        .filter_map(clean_enemy_snapshot)
        .collect()
}

fn clean_enemy_snapshot(snapshot: &ActorSnapshot) -> Option<CleanEnemySnapshot> {
    let kind = match snapshot.kind {
        ActorKind::Lander => CleanEnemyKind::Lander,
        ActorKind::Mutant => CleanEnemyKind::Mutant,
        ActorKind::Bomber => CleanEnemyKind::Bomber,
        ActorKind::Pod => CleanEnemyKind::Pod,
        ActorKind::Swarmer => CleanEnemyKind::Swarmer,
        ActorKind::Baiter => CleanEnemyKind::Baiter,
        _ => return None,
    };
    let mut enemy = CleanEnemySnapshot::new(
        kind,
        screen_position(snapshot.position),
        screen_velocity(snapshot.velocity),
    );
    enemy.source_lander = snapshot.source_lander.map(clean_source_lander);
    enemy.source_bomber = snapshot.source_bomber.map(clean_source_bomber);
    enemy.source_pod = snapshot.source_pod.map(clean_source_pod);
    enemy.source_swarmer = snapshot.source_swarmer.map(clean_source_swarmer);
    enemy.source_baiter = snapshot.source_baiter.map(clean_source_baiter);
    enemy.source_mutant = snapshot.source_mutant.map(clean_source_mutant);
    Some(enemy)
}

fn clean_source_lander(source: ActorSourceLanderMetadata) -> SourceLanderSnapshot {
    SourceLanderSnapshot {
        x_fraction: source.x_fraction,
        y_fraction: source.y_fraction,
        x_velocity: source.x_velocity,
        y_velocity: source.y_velocity,
        shot_timer: source.shot_timer,
        sleep_ticks: source.sleep_ticks,
        picture_frame: source.picture_frame,
        target_human_index: source.target_human_index,
    }
}

fn clean_source_bomber(source: ActorSourceBomberMetadata) -> SourceBomberSnapshot {
    SourceBomberSnapshot {
        x_fraction: source.x_fraction,
        y_fraction: source.y_fraction,
        x_velocity: source.x_velocity,
        y_velocity: source.y_velocity,
        picture_frame: source.picture_frame,
        cruise_altitude: screen_coordinate(source.cruise_altitude),
        sleep_ticks: source.sleep_ticks,
        source_slot: source.source_slot,
    }
}

fn clean_source_pod(source: ActorSourcePodMetadata) -> SourcePodSnapshot {
    SourcePodSnapshot {
        x_fraction: source.x_fraction,
        y_fraction: source.y_fraction,
        x_velocity: source.x_velocity,
        y_velocity: source.y_velocity,
    }
}

fn clean_source_swarmer(source: ActorSourceSwarmerMetadata) -> SourceSwarmerSnapshot {
    SourceSwarmerSnapshot {
        x_fraction: source.x_fraction,
        y_fraction: source.y_fraction,
        x_velocity: source.x_velocity,
        y_velocity: source.y_velocity,
        acceleration: source.acceleration,
        shot_timer: source.shot_timer,
        sleep_ticks: source.sleep_ticks,
        horizontal_seek_pending: source.horizontal_seek_pending,
    }
}

fn clean_source_baiter(source: ActorSourceBaiterMetadata) -> SourceBaiterSnapshot {
    SourceBaiterSnapshot {
        x_fraction: source.x_fraction,
        y_fraction: source.y_fraction,
        x_velocity: source.x_velocity,
        y_velocity: source.y_velocity,
        shot_timer: source.shot_timer,
        sleep_ticks: source.sleep_ticks,
        picture_frame: source.picture_frame,
    }
}

fn clean_source_mutant(source: ActorSourceMutantMetadata) -> SourceMutantSnapshot {
    SourceMutantSnapshot {
        x_fraction: source.x_fraction,
        y_fraction: source.y_fraction,
        x_velocity: source.x_velocity,
        y_velocity: source.y_velocity,
        shot_timer: source.shot_timer,
        sleep_ticks: source.sleep_ticks,
        hop_rng: clean_source_rng(source.hop_rng),
        render_x_correction: source.render_x_correction,
        target6_first_shot_deferred: source.target6_first_shot_deferred,
    }
}

const fn clean_source_rng(source: ActorSourceRngSnapshot) -> SourceRandSnapshot {
    SourceRandSnapshot {
        seed: source.seed,
        hseed: source.hseed,
        lseed: source.lseed,
    }
}

fn actor_humans_for_report(report: &StepReport) -> Vec<CleanHumanSnapshot> {
    report
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
        .map(|snapshot| {
            let mut human = CleanHumanSnapshot::new(screen_position(snapshot.position));
            human.carried = report.draws.iter().any(|draw| {
                draw.actor == snapshot.id && matches!(draw.sprite, SpriteKey::HumanCarried)
            });
            if let Some(source) = snapshot.source_human {
                human.source_x_fraction = source.x_fraction;
                human.source_picture_frame = source.picture_frame;
            }
            human
        })
        .collect()
}

fn actor_projectiles_for_report(report: &StepReport) -> Vec<CleanProjectileSnapshot> {
    report
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.kind == ActorKind::Laser && snapshot.alive)
        .map(|snapshot| CleanProjectileSnapshot {
            position: screen_position(snapshot.position),
            source_tail_position: screen_position(Point::new(
                snapshot.position.x.saturating_sub(16),
                snapshot.position.y,
            )),
            velocity: screen_velocity(snapshot.velocity),
        })
        .collect()
}

fn actor_enemy_projectiles_for_report(report: &StepReport) -> Vec<CleanEnemyProjectileSnapshot> {
    report
        .snapshots
        .iter()
        .filter(|snapshot| {
            matches!(snapshot.kind, ActorKind::EnemyLaser | ActorKind::Bomb) && snapshot.alive
        })
        .map(|snapshot| CleanEnemyProjectileSnapshot {
            position: screen_position(snapshot.position),
            velocity: screen_velocity(snapshot.velocity),
            source_kind: if snapshot.kind == ActorKind::Bomb {
                EnemyProjectileSourceKind::BomberBombShell
            } else {
                EnemyProjectileSourceKind::Fireball
            },
            source_x_fraction: snapshot
                .source_enemy_projectile
                .map_or(0, |source| source.x_fraction),
            source_y_fraction: snapshot
                .source_enemy_projectile
                .map_or(0, |source| source.y_fraction),
            source_x_velocity: snapshot
                .source_enemy_projectile
                .map_or(0, |source| source.x_velocity),
            source_y_velocity: snapshot
                .source_enemy_projectile
                .map_or(0, |source| source.y_velocity),
            source_lifetime_ticks: snapshot
                .source_enemy_projectile
                .map_or(0, |source| source.lifetime_ticks),
        })
        .collect()
}

fn actor_explosions_for_report(report: &StepReport) -> Vec<CleanExplosionSnapshot> {
    report
        .draws
        .iter()
        .filter_map(|draw| match draw.effect {
            VisualEffect::ExplosionCloud {
                kind,
                age,
                source_center,
            } => {
                let mut explosion = CleanExplosionSnapshot::source_spawn(
                    clean_explosion_kind(kind),
                    screen_position(draw.position),
                );
                explosion.source_center = source_center.map(screen_position);
                explosion.source_size = actor_source_explosion_size_for_kind(kind, age);
                Some(explosion)
            }
            _ => None,
        })
        .collect()
}

fn clean_explosion_kind(kind: ExplosionKind) -> CleanExplosionKind {
    match kind {
        ExplosionKind::Lander => CleanExplosionKind::Lander,
        ExplosionKind::Mutant => CleanExplosionKind::Mutant,
        ExplosionKind::Bomber => CleanExplosionKind::Bomber,
        ExplosionKind::Pod => CleanExplosionKind::Pod,
        ExplosionKind::Swarmer => CleanExplosionKind::Swarmer,
        ExplosionKind::Baiter => CleanExplosionKind::Baiter,
        ExplosionKind::Bomb => CleanExplosionKind::Bomb,
        ExplosionKind::Player => CleanExplosionKind::PlayerShip,
        ExplosionKind::Human => CleanExplosionKind::Astronaut,
        ExplosionKind::Terrain => CleanExplosionKind::Terrain,
    }
}

fn actor_score_popups_for_report(report: &StepReport) -> Vec<CleanScorePopupSnapshot> {
    report
        .draws
        .iter()
        .filter_map(|draw| {
            let kind = match draw.sprite {
                SpriteKey::Score250 => CleanScorePopupKind::Points250,
                SpriteKey::Score500 => CleanScorePopupKind::Points500,
                _ => return None,
            };
            Some(CleanScorePopupSnapshot::source_spawn(
                kind,
                screen_position(draw.position),
            ))
        })
        .collect()
}

fn world_vector(value: i16) -> WorldVector {
    WorldVector::from_subpixels(i32::from(value) * WorldVector::SUBPIXELS_PER_PIXEL)
}

fn screen_position(point: Point) -> ScreenPosition {
    ScreenPosition::new(screen_coordinate(point.x), screen_coordinate(point.y))
}

fn try_screen_position(point: Point) -> Option<ScreenPosition> {
    Some(ScreenPosition::new(
        u8::try_from(point.x).ok()?,
        u8::try_from(point.y).ok()?,
    ))
}

fn screen_velocity(velocity: Velocity) -> ScreenVelocity {
    ScreenVelocity::new(
        screen_velocity_component(velocity.dx),
        screen_velocity_component(velocity.dy),
    )
}

fn screen_velocity_component(value: i16) -> i8 {
    i8::try_from(value.clamp(i16::from(i8::MIN), i16::from(i8::MAX)))
        .expect("screen velocity should be clamped to i8")
}

fn screen_coordinate(value: i16) -> u8 {
    u8::try_from(value.clamp(0, 255)).expect("screen coordinate should be clamped to u8")
}

fn actor_source_projectile_velocity_component(value: i16) -> u16 {
    let clamped = value.clamp(i16::from(i8::MIN), i16::from(i8::MAX)) as i8;
    ((i16::from(clamped)) << 8) as u16
}

fn actor_source_projectile_lifetime_ticks(remaining_steps: u16) -> u8 {
    remaining_steps.min(u16::from(u8::MAX)) as u8
}

fn actor_source_enemy_shot_metadata(
    x_fraction: u8,
    y_fraction: u8,
    velocity: Velocity,
    lifetime_steps: u16,
) -> ActorSourceEnemyProjectileMetadata {
    ActorSourceEnemyProjectileMetadata {
        x_fraction,
        y_fraction,
        x_velocity: actor_source_projectile_velocity_component(velocity.dx),
        y_velocity: actor_source_projectile_velocity_component(velocity.dy),
        lifetime_ticks: actor_source_projectile_lifetime_ticks(lifetime_steps),
    }
}

fn actor_source_projectile_axis_step(position: i16, fraction: u8, velocity: u16) -> (i16, u8) {
    let raw = i32::from(position) * 256 + i32::from(fraction) + i32::from(velocity as i16);
    let next_position = raw.div_euclid(256);
    let next_fraction = raw.rem_euclid(256);
    (
        next_position.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16,
        next_fraction as u8,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorRenderSceneBridge {
    surface: SurfaceSize,
}

impl ActorRenderSceneBridge {
    pub const fn new() -> Self {
        Self {
            surface: ACTOR_RENDER_SURFACE,
        }
    }

    pub const fn with_surface(surface: SurfaceSize) -> Self {
        Self { surface }
    }

    pub const fn surface(self) -> SurfaceSize {
        self.surface
    }

    pub fn render_scene_for_report(&self, report: &StepReport) -> RenderScene {
        let mut scene = RenderScene::empty(report.step, self.surface);
        if report.phase == Phase::Playing && report.smart_bomb_flash_steps_remaining > 0 {
            scene.clear_color = Color::WHITE;
        }
        if report.phase == Phase::Playing
            && let Some(terrain_blow) = report.terrain_blow
            && terrain_blow.terrain_erased()
        {
            let flash_tint = source_terrain_blow_flash_tint(terrain_blow.source_elapsed_frames);
            if flash_tint.rgba[3] != 0 {
                scene.clear_color = flash_tint;
            }
        }
        if report.phase == Phase::Playing
            && report.player_start.is_none()
            && report.terrain_blow.is_none()
        {
            push_source_bgout_terrain_sprites(&mut scene, report.source_background_left);
        }
        for draw in &report.draws {
            self.push_draw(&mut scene, report.phase, draw);
        }
        push_actor_player_switch_prompt_sprites(&mut scene, report);
        push_actor_player_start_prompt_sprites(&mut scene, report);
        push_actor_wave_completion_status_sprites(&mut scene, report);
        push_actor_survivor_bonus_icon_sprites(&mut scene, report);
        scene
    }

    fn push_draw(&self, scene: &mut RenderScene, phase: Phase, draw: &DrawCommand) {
        if let Some(text) = &draw.text {
            let layer = if phase == Phase::Attract {
                RenderLayer::Overlay
            } else {
                RenderLayer::Hud
            };
            if let VisualEffect::SourceMessage {
                top_left_screen_address,
                visual_offset,
            } = draw.effect
            {
                push_source_controlled_message_sprites_with_offset(
                    scene,
                    text,
                    top_left_screen_address,
                    layer,
                    visual_offset,
                );
                return;
            }
            push_source_text_bytes_sprites(
                scene,
                text.as_bytes(),
                point_position(draw.position),
                layer,
            );
            return;
        }

        match draw.effect {
            VisualEffect::WilliamsReveal {
                stroke_step,
                color_phase,
            } => self.push_williams_reveal(scene, draw.position, stroke_step, color_phase),
            VisualEffect::DefenderCoalescence { slot, row_pair } => {
                self.push_defender_coalescence(scene, slot, row_pair)
            }
            VisualEffect::AttractScoringSurface { scoring_tick } => {
                self.push_attract_scoring_surface(scene, scoring_tick)
            }
            VisualEffect::ExplosionCloud {
                kind,
                age,
                source_center,
            } => self.push_explosion_sprite(scene, draw.position, kind, age, source_center),
            VisualEffect::Static
            | VisualEffect::SourceMessage { .. }
            | VisualEffect::SourceLanderFrame { .. }
            | VisualEffect::SourceBomberFrame { .. }
            | VisualEffect::SourcePod
            | VisualEffect::SourceBaiterFrame { .. }
            | VisualEffect::SourceHumanFrame { .. } => self.push_static_sprite(scene, draw),
        }
    }

    fn push_williams_reveal(
        &self,
        scene: &mut RenderScene,
        position: Point,
        stroke_step: u16,
        color_phase: u8,
    ) {
        let tint = williams_logo_phase_tint(color_phase);
        let pixel_path = source_attract_williams_logo_pixel_path();
        let visible_pixel_count =
            williams_reveal_visible_pixel_count(stroke_step, pixel_path.len());
        if visible_pixel_count < pixel_path.len() {
            let origin = point_position(position);
            for [pixel_x, pixel_y] in pixel_path.into_iter().take(visible_pixel_count) {
                scene.push_sprite(SceneSprite {
                    sprite: SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
                    layer: RenderLayer::Overlay,
                    position: [
                        origin[0] + f32::from(pixel_x),
                        origin[1] + f32::from(pixel_y),
                    ],
                    size: [1.0, 1.0],
                    tint,
                });
            }
            return;
        }

        scene.push_sprite(SceneSprite {
            sprite: SpriteId::ATTRACT_WILLIAMS_LOGO,
            layer: RenderLayer::Overlay,
            position: point_position(position),
            size: WILLIAMS_LOGO_SCENE_SIZE,
            tint,
        });
    }

    fn push_defender_coalescence(&self, scene: &mut RenderScene, slot: u8, row_pair: u8) {
        let progress = u16::from(slot)
            .saturating_mul(DEFENDER_WORDMARK_ROW_PAIRS)
            .saturating_add(u16::from(row_pair));
        let total_steps = DEFENDER_WORDMARK_SLOTS.saturating_mul(DEFENDER_WORDMARK_ROW_PAIRS);
        let appearance_tick = if total_steps == 0 {
            0
        } else {
            progress
                .saturating_mul(u16::from(SOURCE_ATTRACT_DEFENDER_APPEARANCE_FINAL_TICK))
                .checked_div(total_steps)
                .unwrap_or(0)
        };
        let appearance_tick = u8::try_from(appearance_tick)
            .expect("Defender appearance tick fits")
            .min(SOURCE_ATTRACT_DEFENDER_APPEARANCE_FINAL_TICK);

        for pixel in source_attract_defender_appearance_pixels(scene.surface, appearance_tick) {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
                layer: RenderLayer::Overlay,
                position: [f32::from(pixel.position[0]), f32::from(pixel.position[1])],
                size: [1.0, 1.0],
                tint: Color { rgba: pixel.color },
            });
        }
    }

    fn push_attract_scoring_surface(&self, scene: &mut RenderScene, scoring_tick: u16) {
        push_attract_scoring_top_display_border(scene);
        push_attract_scoring_scanner_terrain(scene);
        push_attract_scoring_demo_scene(scene, scoring_tick);
    }

    fn push_explosion_sprite(
        &self,
        scene: &mut RenderScene,
        position: Point,
        kind: ExplosionKind,
        age: u16,
        source_center: Option<Point>,
    ) {
        let (sprite, base_size) = match kind {
            ExplosionKind::Lander => (SpriteId::ENEMY_LANDER, LANDER_SCENE_SIZE),
            ExplosionKind::Mutant => (SpriteId::ENEMY_MUTANT, MUTANT_SCENE_SIZE),
            ExplosionKind::Bomber => (SpriteId::ENEMY_BOMBER, BOMBER_SCENE_SIZE),
            ExplosionKind::Pod => (SpriteId::ENEMY_POD, POD_SCENE_SIZE),
            ExplosionKind::Swarmer => (SpriteId::SWARMER_EXPLOSION, EXPLOSION_SCENE_SIZE),
            ExplosionKind::Baiter => (SpriteId::ENEMY_BAITER, BAITER_SCENE_SIZE),
            ExplosionKind::Bomb => (SpriteId::BOMB_EXPLOSION, EXPLOSION_SCENE_SIZE),
            ExplosionKind::Human => (SpriteId::ASTRONAUT_EXPLOSION, EXPLOSION_SCENE_SIZE),
            ExplosionKind::Terrain => (SpriteId::TERRAIN_EXPLOSION, EXPLOSION_SCENE_SIZE),
            ExplosionKind::Player => (
                SpriteId::PLAYER_EXPLOSION_PIXEL,
                PLAYER_EXPLOSION_PIXEL_SCENE_SIZE,
            ),
        };
        let source_size = actor_source_explosion_size_for_kind(kind, age);
        if let Some(source_position) = try_screen_position(position)
            && push_source_explosion_cloud_pixels(
                scene,
                clean_explosion_kind(kind),
                source_position,
                source_center.and_then(try_screen_position),
                source_size,
            )
        {
            return;
        }

        let scale = actor_source_explosion_render_scale(source_size);
        let size = [base_size[0] * scale, base_size[1] * scale];
        let origin = point_position(position);
        let centered_position = [
            origin[0] + base_size[0] / 2.0 - size[0] / 2.0,
            origin[1] + base_size[1] / 2.0 - size[1] / 2.0,
        ];
        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Objects,
            position: centered_position,
            size,
            tint: Color::WHITE,
        });
    }

    fn push_static_sprite(&self, scene: &mut RenderScene, draw: &DrawCommand) {
        let Some(sprite) = actor_scene_sprite(draw.sprite, draw.position) else {
            return;
        };
        scene.push_sprite(sprite);
    }
}

impl Default for ActorRenderSceneBridge {
    fn default() -> Self {
        Self::new()
    }
}

fn push_actor_wave_completion_status_sprites(scene: &mut RenderScene, report: &StepReport) {
    if !should_show_actor_wave_completion_status(report) {
        return;
    }

    for (label, screen_address) in SOURCE_WAVE_COMPLETION_STATUS_LINES {
        if let Some(text) = source_message_text(label) {
            push_source_text_bytes_sprites(
                scene,
                text.as_bytes(),
                source_screen_position(*screen_address),
                RenderLayer::Overlay,
            );
        }
    }

    let (wave_digits, wave_digit_count) = actor_visible_decimal_digits(clean_wave(report.wave));
    push_source_text_bytes_sprites(
        scene,
        &wave_digits[..wave_digit_count],
        source_screen_position(SOURCE_WAVE_COMPLETION_WAVE_NUMBER_SCREEN),
        RenderLayer::Overlay,
    );

    let multiplier = clean_wave(report.wave).min(5);
    let (multiplier_digits, multiplier_digit_count) = actor_visible_decimal_digits(multiplier);
    push_source_text_bytes_sprites(
        scene,
        &multiplier_digits[..multiplier_digit_count],
        source_screen_position(SOURCE_WAVE_COMPLETION_MULTIPLIER_NUMBER_SCREEN),
        RenderLayer::Overlay,
    );
}

fn push_actor_survivor_bonus_icon_sprites(scene: &mut RenderScene, report: &StepReport) {
    if !should_show_actor_wave_completion_status(report) {
        return;
    }

    for index in 0..actor_visible_survivor_bonus_icon_count(report) {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::HUMAN,
            layer: RenderLayer::Overlay,
            position: source_screen_position_with_offset(
                SOURCE_SURVIVOR_BONUS_FIRST_HUMAN_SCREEN,
                (index as u8) * SOURCE_SURVIVOR_BONUS_HUMAN_STEP,
                0,
            ),
            size: SOURCE_SURVIVOR_BONUS_HUMAN_SIZE,
            tint: Color::WHITE,
        });
    }
}

fn should_show_actor_wave_completion_status(report: &StepReport) -> bool {
    report.phase == Phase::Playing
        && (report.survivor_bonus.is_some()
            || report
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::WaveCleared { .. })))
        && report
            .snapshots
            .iter()
            .any(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
}

fn push_actor_player_switch_prompt_sprites(scene: &mut RenderScene, report: &StepReport) {
    let Some(player_switch) = report.player_switch else {
        return;
    };

    push_actor_source_message_sprites(
        scene,
        player_source_message_label(player_switch.from_player),
        SOURCE_PLAYER_SWITCH_LABEL_SCREEN,
        RenderLayer::Overlay,
    );
    push_actor_source_message_sprites(
        scene,
        "GO",
        SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN,
        RenderLayer::Overlay,
    );
}

fn push_actor_player_start_prompt_sprites(scene: &mut RenderScene, report: &StepReport) {
    let Some(player_start) = report.player_start else {
        return;
    };
    if report.player_count <= 1 {
        return;
    }

    push_actor_source_message_sprites(
        scene,
        player_source_message_label(player_start.player),
        SOURCE_PLAYER_START_PROMPT_SCREEN,
        RenderLayer::Overlay,
    );
}

fn push_actor_source_message_sprites(
    scene: &mut RenderScene,
    label: &str,
    top_left_screen_address: u16,
    layer: RenderLayer,
) {
    if let Some(text) = source_message_text(label) {
        push_source_controlled_message_sprites(scene, text, top_left_screen_address, layer);
    }
}

fn actor_visible_survivor_bonus_icon_count(report: &StepReport) -> usize {
    report
        .survivor_bonus
        .map(|bonus| usize::from(bonus.visible_icons).min(SOURCE_SURVIVOR_BONUS_HUMAN_LIMIT))
        .unwrap_or(0)
}

fn actor_visible_decimal_digits(value: u8) -> ([u8; 2], usize) {
    let value = value.min(99);
    if value < 10 {
        ([b'0' + value, b' '], 1)
    } else {
        ([b'0' + value / 10, b'0' + value % 10], 2)
    }
}

fn push_source_controlled_message_sprites_with_offset(
    scene: &mut RenderScene,
    text: &str,
    top_left_screen_address: u16,
    layer: RenderLayer,
    visual_offset: Point,
) {
    let first_sprite = scene.sprites.len();
    push_source_controlled_message_sprites(scene, text, top_left_screen_address, layer);
    offset_new_sprites(scene, first_sprite, point_position(visual_offset));
}

fn push_attract_scoring_top_display_border(scene: &mut RenderScene) {
    for (screen_address, size) in SOURCE_TOP_DISPLAY_BORDER_SEGMENTS {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::TOP_DISPLAY_BORDER_WORD,
            layer: RenderLayer::Hud,
            position: offset_f32_position(
                source_screen_position(screen_address),
                point_position(SOURCE_ATTRACT_SCORING_VISUAL_OFFSET),
            ),
            size,
            tint: ATTRACT_SCORING_SCANNER_BORDER_TINT,
        });
    }
}

fn push_attract_scoring_scanner_terrain(scene: &mut RenderScene) {
    for record in source_scanner_mini_terrain_records() {
        let origin = offset_f32_position(
            source_screen_position(record.screen_address),
            point_position(SOURCE_ATTRACT_SCORING_VISUAL_OFFSET),
        );
        for (row, byte) in record.word.to_be_bytes().into_iter().enumerate() {
            for column in 0..2 {
                let nibble = if column == 0 { byte >> 4 } else { byte & 0x0F };
                if nibble == 0 {
                    continue;
                }
                scene.push_sprite(SceneSprite {
                    sprite: SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL,
                    layer: RenderLayer::Hud,
                    position: [origin[0] + column as f32, origin[1] + row as f32],
                    size: ATTRACT_SCORING_SCANNER_TERRAIN_PIXEL_SIZE,
                    tint: ATTRACT_SCORING_SCANNER_TERRAIN_TINT,
                });
            }
        }
    }
}

fn push_attract_scoring_demo_scene(scene: &mut RenderScene, scoring_tick: u16) {
    let frame = actor_attract_scoring_frame(scoring_tick);
    for object in frame.scanner_objects.iter().copied() {
        push_attract_scoring_scanner_object(scene, object);
    }

    let mut player_ship = None;
    let mut laser_target = None;
    let mut laser_active = false;
    for object in frame.scene_objects.iter().copied() {
        match object.kind {
            ActorAttractScoringObjectKind::PlayerShip
                if object.visual == ActorAttractScoringVisual::Sprite =>
            {
                player_ship = Some(object);
            }
            ActorAttractScoringObjectKind::Enemy(_)
                if object.visual == ActorAttractScoringVisual::Sprite =>
            {
                laser_target = Some(object);
            }
            ActorAttractScoringObjectKind::PlayerShot => {
                laser_active = true;
                continue;
            }
            _ => {}
        }
        push_attract_scoring_scene_object(scene, object);
    }

    if laser_active && let (Some(player_ship), Some(laser_target)) = (player_ship, laser_target) {
        push_actor_attract_scoring_laser_beam(scene, player_ship, laser_target, frame.display_step);
    }

    if let Some(bonus) = frame.bonus {
        scene.push_sprite(SceneSprite {
            sprite: bonus.sprite,
            layer: RenderLayer::Objects,
            position: actor_attract_scoring_scene_position(bonus.x16, bonus.y16),
            size: SCORE_POPUP_SCENE_SIZE,
            tint: Color::WHITE,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActorAttractScoringFrame {
    display_step: u16,
    scene_objects: Vec<ActorAttractScoringObject>,
    scanner_objects: Vec<ActorAttractScoringObject>,
    bonus: Option<ActorAttractScoringBonus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorAttractScoringLegendEntry {
    enemy: ActorAttractScoringEnemyKind,
    table_x16: i32,
    table_y16: i32,
    scanner_color_word: u16,
}

const ACTOR_ATTRACT_SCORING_LEGEND: [ActorAttractScoringLegendEntry;
    ATTRACT_SCORING_LEGEND_ENTRIES as usize] = [
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Lander,
        table_x16: 0x07A0,
        table_y16: 0x5900,
        scanner_color_word: 0x4433,
    },
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Mutant,
        table_x16: 0x0FA0,
        table_y16: 0x5900,
        scanner_color_word: 0xCC33,
    },
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Baiter,
        table_x16: 0x1820,
        table_y16: 0x5B00,
        scanner_color_word: 0x3333,
    },
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Bomber,
        table_x16: 0x0800,
        table_y16: 0x9100,
        scanner_color_word: 0x8888,
    },
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Pod,
        table_x16: 0x1000,
        table_y16: 0x9100,
        scanner_color_word: 0xCCCC,
    },
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Swarmer,
        table_x16: 0x1880,
        table_y16: 0x9300,
        scanner_color_word: 0x2424,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorAttractScoringBonus {
    sprite: SpriteId,
    x16: i32,
    y16: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorAttractScoringObject {
    kind: ActorAttractScoringObjectKind,
    x16: i32,
    y16: i32,
    visual: ActorAttractScoringVisual,
    visual_step: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorAttractScoringObjectKind {
    PlayerShip,
    Human,
    PlayerShot,
    Enemy(ActorAttractScoringEnemyKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorAttractScoringEnemyKind {
    Lander,
    Mutant,
    Baiter,
    Bomber,
    Pod,
    Swarmer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorAttractScoringVisual {
    Sprite,
    Explosion,
    Materialize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorAttractScoringStage {
    RescueDescend,
    RescueAscend,
    RescueLaser,
    RescueFall,
    RescueScore,
    RescueReturn,
    LegendApproach(usize),
    LegendLaser(usize),
    LegendTransfer(usize),
    LegendReveal(usize),
    LegendHold,
}

fn actor_attract_scoring_frame(scoring_tick: u16) -> ActorAttractScoringFrame {
    let display_step = actor_attract_scoring_display_step(scoring_tick);
    let (stage, local_step) = actor_attract_scoring_stage_for_step(display_step);
    let scanner_display_step = display_step - (display_step % 4);
    let (scanner_stage, scanner_local_step) =
        actor_attract_scoring_stage_for_step(scanner_display_step);
    ActorAttractScoringFrame {
        display_step,
        scene_objects: actor_attract_scoring_objects_for_stage(stage, local_step),
        scanner_objects: actor_attract_scoring_objects_for_stage(scanner_stage, scanner_local_step),
        bonus: actor_attract_scoring_bonus(stage, local_step),
    }
}

fn actor_attract_scoring_display_step(scoring_tick: u16) -> u16 {
    (scoring_tick % ATTRACT_SCORING_DEMO_TOTAL_STEPS + ATTRACT_SCORING_PROTECTED_DEMO_STEP_OFFSET)
        % ATTRACT_SCORING_DEMO_TOTAL_STEPS
}

fn actor_attract_scoring_tick_for_display_step(display_step: u16) -> u16 {
    (display_step % ATTRACT_SCORING_DEMO_TOTAL_STEPS + ATTRACT_SCORING_DEMO_TOTAL_STEPS
        - ATTRACT_SCORING_PROTECTED_DEMO_STEP_OFFSET)
        % ATTRACT_SCORING_DEMO_TOTAL_STEPS
}

fn actor_attract_scoring_display_step_for_stage(
    target_stage: ActorAttractScoringStage,
    local_step: u16,
) -> u16 {
    let mut elapsed = 0;
    for (stage, duration) in ACTOR_ATTRACT_SCORING_RESCUE_TIMELINE {
        if stage == target_stage {
            return elapsed + local_step.min(duration.saturating_sub(1));
        }
        elapsed += duration;
    }

    for index in 0..ACTOR_ATTRACT_SCORING_LEGEND.len() {
        for (stage, duration) in actor_attract_scoring_legend_timeline(index) {
            if stage == target_stage {
                return elapsed + local_step.min(duration.saturating_sub(1));
            }
            elapsed += duration;
        }
    }

    elapsed + local_step.min(ATTRACT_SCORING_LEGEND_HOLD_STEPS.saturating_sub(1))
}

fn actor_attract_scoring_instruction_text_start_step(line_index: usize) -> u64 {
    let Some(legend_index) = line_index.checked_sub(1) else {
        return SOURCE_ATTRACT_SCORING_SEQUENCE_START_STEP;
    };

    let reveal_display_step = actor_attract_scoring_display_step_for_stage(
        ActorAttractScoringStage::LegendReveal(legend_index),
        0,
    );
    SOURCE_ATTRACT_SCORING_SEQUENCE_START_STEP
        + u64::from(actor_attract_scoring_tick_for_display_step(
            next_actor_attract_scoring_text_process_step(reveal_display_step),
        ))
}

fn next_actor_attract_scoring_text_process_step(step: u16) -> u16 {
    let remainder = step % 6;
    if remainder == 0 {
        step
    } else {
        step + (6 - remainder)
    }
}

const ACTOR_ATTRACT_SCORING_RESCUE_TIMELINE: [(ActorAttractScoringStage, u16); 6] = [
    (
        ActorAttractScoringStage::RescueDescend,
        ATTRACT_SCORING_RESCUE_DESCENT_STEPS,
    ),
    (
        ActorAttractScoringStage::RescueAscend,
        ATTRACT_SCORING_RESCUE_ASCENT_STEPS,
    ),
    (
        ActorAttractScoringStage::RescueLaser,
        ATTRACT_SCORING_RESCUE_LASER_STEPS,
    ),
    (
        ActorAttractScoringStage::RescueFall,
        ATTRACT_SCORING_RESCUE_FALL_STEPS,
    ),
    (
        ActorAttractScoringStage::RescueScore,
        ATTRACT_SCORING_RESCUE_SCORE_STEPS,
    ),
    (
        ActorAttractScoringStage::RescueReturn,
        ATTRACT_SCORING_RESCUE_RETURN_STEPS,
    ),
];

fn actor_attract_scoring_legend_timeline(index: usize) -> [(ActorAttractScoringStage, u16); 4] {
    [
        (
            ActorAttractScoringStage::LegendApproach(index),
            ATTRACT_SCORING_LEGEND_APPROACH_STEPS,
        ),
        (
            ActorAttractScoringStage::LegendLaser(index),
            ATTRACT_SCORING_LEGEND_LASER_STEPS,
        ),
        (
            ActorAttractScoringStage::LegendTransfer(index),
            ATTRACT_SCORING_LEGEND_TRANSFER_STEPS,
        ),
        (
            ActorAttractScoringStage::LegendReveal(index),
            ATTRACT_SCORING_LEGEND_REVEAL_STEPS,
        ),
    ]
}

fn actor_attract_scoring_stage_for_step(mut display_step: u16) -> (ActorAttractScoringStage, u16) {
    for (stage, duration) in ACTOR_ATTRACT_SCORING_RESCUE_TIMELINE {
        if display_step < duration {
            return (stage, display_step);
        }
        display_step -= duration;
    }

    for index in 0..ACTOR_ATTRACT_SCORING_LEGEND.len() {
        for (stage, duration) in actor_attract_scoring_legend_timeline(index) {
            if display_step < duration {
                return (stage, display_step);
            }
            display_step -= duration;
        }
    }

    (
        ActorAttractScoringStage::LegendHold,
        display_step.min(ATTRACT_SCORING_LEGEND_HOLD_STEPS.saturating_sub(1)),
    )
}

fn actor_attract_scoring_objects_for_stage(
    stage: ActorAttractScoringStage,
    local_step: u16,
) -> Vec<ActorAttractScoringObject> {
    let mut objects = Vec::new();
    match stage {
        ActorAttractScoringStage::RescueDescend => {
            objects.push(actor_attract_scoring_enemy_object(
                ActorAttractScoringEnemyKind::Lander,
                ATTRACT_SCORING_LANDER_X16,
                ATTRACT_SCORING_LANDER_Y16 + i32::from(local_step) * 0x00A0,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::Human,
                ATTRACT_SCORING_HUMAN_X16,
                ATTRACT_SCORING_HUMAN_Y16,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShip,
                ATTRACT_SCORING_PLAYER_X16,
                ATTRACT_SCORING_PLAYER_Y16,
            ));
        }
        ActorAttractScoringStage::RescueAscend | ActorAttractScoringStage::RescueLaser => {
            let rise_step = if stage == ActorAttractScoringStage::RescueAscend {
                local_step
            } else {
                ATTRACT_SCORING_RESCUE_ASCENT_STEPS + local_step
            };
            let lander_y = ATTRACT_SCORING_LANDER_Y16
                + i32::from(ATTRACT_SCORING_RESCUE_DESCENT_STEPS) * 0x00A0
                - i32::from(rise_step) * 0x00B0;
            let human_y = ATTRACT_SCORING_HUMAN_Y16 - i32::from(rise_step) * 0x00B0;
            objects.push(actor_attract_scoring_enemy_object(
                ActorAttractScoringEnemyKind::Lander,
                ATTRACT_SCORING_LANDER_X16,
                lander_y,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::Human,
                ATTRACT_SCORING_HUMAN_X16,
                human_y,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShip,
                ATTRACT_SCORING_PLAYER_X16,
                ATTRACT_SCORING_PLAYER_Y16,
            ));
            if stage == ActorAttractScoringStage::RescueLaser {
                objects.push(actor_attract_scoring_object(
                    ActorAttractScoringObjectKind::PlayerShot,
                    ATTRACT_SCORING_LANDER_X16,
                    lander_y,
                ));
            }
        }
        ActorAttractScoringStage::RescueFall => {
            let (ship_x, ship_y, human_y) = actor_attract_scoring_intercept_state(local_step);
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShip,
                ship_x,
                ship_y,
            ));
            if local_step < 12 {
                let lander_y = ATTRACT_SCORING_LANDER_Y16
                    + i32::from(ATTRACT_SCORING_RESCUE_DESCENT_STEPS) * 0x00A0
                    - i32::from(
                        ATTRACT_SCORING_RESCUE_ASCENT_STEPS + ATTRACT_SCORING_RESCUE_LASER_STEPS,
                    ) * 0x00B0;
                objects.push(actor_attract_scoring_visual_enemy_object(
                    ActorAttractScoringEnemyKind::Lander,
                    ATTRACT_SCORING_LANDER_X16,
                    lander_y,
                    ActorAttractScoringVisual::Explosion,
                    local_step,
                ));
            }
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::Human,
                ATTRACT_SCORING_HUMAN_X16,
                human_y,
            ));
        }
        ActorAttractScoringStage::RescueScore => {
            let (ship_x, ship_y, human_y) = actor_attract_scoring_drop_state(local_step);
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShip,
                ship_x,
                ship_y,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::Human,
                ATTRACT_SCORING_CAUGHT_HUMAN_X16,
                human_y,
            ));
        }
        ActorAttractScoringStage::RescueReturn => {
            let (ship_x, ship_y, _) =
                actor_attract_scoring_drop_state(ATTRACT_SCORING_RESCUE_SCORE_STEPS);
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShip,
                ship_x + i32::from(local_step) * ATTRACT_SCORING_RESCUE_RETURN_XV16,
                ship_y + i32::from(local_step) * ATTRACT_SCORING_RESCUE_RETURN_YV16,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::Human,
                ATTRACT_SCORING_CAUGHT_HUMAN_X16,
                ATTRACT_SCORING_GROUNDED_HUMAN_Y16,
            ));
        }
        ActorAttractScoringStage::LegendApproach(_)
        | ActorAttractScoringStage::LegendLaser(_)
        | ActorAttractScoringStage::LegendTransfer(_)
        | ActorAttractScoringStage::LegendReveal(_)
        | ActorAttractScoringStage::LegendHold => {
            let (player_x, player_y) = actor_attract_scoring_legend_player_position();
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShip,
                player_x,
                player_y,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::Human,
                ATTRACT_SCORING_CAUGHT_HUMAN_X16,
                ATTRACT_SCORING_GROUNDED_HUMAN_Y16,
            ));
            append_actor_attract_scoring_legend_objects(
                &mut objects,
                stage,
                local_step,
                player_x,
                player_y,
            );
        }
    }
    objects
}

fn actor_attract_scoring_intercept_state(fall_step: u16) -> (i32, i32, i32) {
    let mut ship_x = ATTRACT_SCORING_PLAYER_X16;
    let mut ship_y = ATTRACT_SCORING_PLAYER_Y16;
    let mut human_y = ATTRACT_SCORING_HUMAN_Y16
        - i32::from(ATTRACT_SCORING_RESCUE_ASCENT_STEPS + ATTRACT_SCORING_RESCUE_LASER_STEPS)
            * 0x00B0;
    let mut elapsed = 0;
    let mut human_velocity = 0;
    for _ in 0..(ATTRACT_SCORING_RESCUE_FALL_STEPS / 2) {
        human_velocity += ATTRACT_SCORING_RESCUE_HUMAN_ACCEL16;
        for _ in 0..2 {
            if elapsed >= fall_step {
                return (ship_x, ship_y, human_y);
            }
            ship_x += ATTRACT_SCORING_RESCUE_SHIP_XV16;
            ship_y += ATTRACT_SCORING_RESCUE_SHIP_YV16;
            human_y += human_velocity;
            elapsed += 1;
        }
    }
    (ship_x, ship_y, human_y)
}

fn actor_attract_scoring_drop_state(score_step: u16) -> (i32, i32, i32) {
    let (ship_x, ship_y, _) =
        actor_attract_scoring_intercept_state(ATTRACT_SCORING_RESCUE_FALL_STEPS);
    (
        ship_x,
        ship_y + i32::from(score_step) * ATTRACT_SCORING_RESCUE_DROP_YV16,
        ATTRACT_SCORING_CAUGHT_HUMAN_Y16 + i32::from(score_step) * ATTRACT_SCORING_RESCUE_DROP_YV16,
    )
}

fn actor_attract_scoring_legend_player_position() -> (i32, i32) {
    let (ship_x, ship_y, _) = actor_attract_scoring_drop_state(ATTRACT_SCORING_RESCUE_SCORE_STEPS);
    (
        ship_x
            + i32::from(ATTRACT_SCORING_RESCUE_RETURN_STEPS) * ATTRACT_SCORING_RESCUE_RETURN_XV16,
        ship_y
            + i32::from(ATTRACT_SCORING_RESCUE_RETURN_STEPS) * ATTRACT_SCORING_RESCUE_RETURN_YV16,
    )
}

fn append_actor_attract_scoring_legend_objects(
    objects: &mut Vec<ActorAttractScoringObject>,
    stage: ActorAttractScoringStage,
    local_step: u16,
    player_x16: i32,
    player_y16: i32,
) {
    for entry in ACTOR_ATTRACT_SCORING_LEGEND
        .iter()
        .take(actor_attract_scoring_revealed_legend_entries(stage))
    {
        objects.push(actor_attract_scoring_enemy_object(
            entry.enemy,
            entry.table_x16,
            entry.table_y16,
        ));
    }

    let current_index = match stage {
        ActorAttractScoringStage::LegendApproach(index)
        | ActorAttractScoringStage::LegendLaser(index)
        | ActorAttractScoringStage::LegendTransfer(index)
        | ActorAttractScoringStage::LegendReveal(index) => Some(index),
        ActorAttractScoringStage::LegendHold => None,
        _ => return,
    };
    let Some(index) = current_index else {
        return;
    };

    let entry = ACTOR_ATTRACT_SCORING_LEGEND[index];
    let source_y = actor_attract_scoring_legend_source_y16();
    match stage {
        ActorAttractScoringStage::LegendApproach(_) => {
            let enemy_y = ATTRACT_SCORING_LEGEND_SOURCE_START_Y16 - i32::from(local_step) * 0x00C0;
            objects.push(actor_attract_scoring_enemy_object(
                entry.enemy,
                ATTRACT_SCORING_LEGEND_SOURCE_X16,
                enemy_y,
            ));
        }
        ActorAttractScoringStage::LegendLaser(_) => {
            objects.push(actor_attract_scoring_enemy_object(
                entry.enemy,
                ATTRACT_SCORING_LEGEND_SOURCE_X16,
                source_y,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShot,
                player_x16,
                player_y16,
            ));
        }
        ActorAttractScoringStage::LegendTransfer(_) => {
            objects.push(actor_attract_scoring_visual_enemy_object(
                entry.enemy,
                ATTRACT_SCORING_LEGEND_SOURCE_X16,
                source_y,
                ActorAttractScoringVisual::Explosion,
                local_step,
            ));
            objects.push(actor_attract_scoring_visual_enemy_object(
                entry.enemy,
                entry.table_x16,
                entry.table_y16,
                ActorAttractScoringVisual::Materialize,
                local_step,
            ));
        }
        ActorAttractScoringStage::LegendReveal(_) => {
            objects.push(actor_attract_scoring_enemy_object(
                entry.enemy,
                entry.table_x16,
                entry.table_y16,
            ));
        }
        ActorAttractScoringStage::LegendHold => {}
        _ => {}
    }
}

fn actor_attract_scoring_revealed_legend_entries(stage: ActorAttractScoringStage) -> usize {
    match stage {
        ActorAttractScoringStage::LegendHold => ACTOR_ATTRACT_SCORING_LEGEND.len(),
        ActorAttractScoringStage::LegendApproach(index)
        | ActorAttractScoringStage::LegendLaser(index)
        | ActorAttractScoringStage::LegendTransfer(index)
        | ActorAttractScoringStage::LegendReveal(index) => index,
        _ => 0,
    }
}

fn actor_attract_scoring_legend_source_y16() -> i32 {
    ATTRACT_SCORING_LEGEND_SOURCE_START_Y16
        - i32::from(ATTRACT_SCORING_LEGEND_APPROACH_STEPS) * 0x00C0
}

fn actor_attract_scoring_enemy_object(
    enemy: ActorAttractScoringEnemyKind,
    x16: i32,
    y16: i32,
) -> ActorAttractScoringObject {
    actor_attract_scoring_object(ActorAttractScoringObjectKind::Enemy(enemy), x16, y16)
}

fn actor_attract_scoring_visual_enemy_object(
    enemy: ActorAttractScoringEnemyKind,
    x16: i32,
    y16: i32,
    visual: ActorAttractScoringVisual,
    visual_step: u16,
) -> ActorAttractScoringObject {
    ActorAttractScoringObject {
        kind: ActorAttractScoringObjectKind::Enemy(enemy),
        x16,
        y16,
        visual,
        visual_step,
    }
}

fn actor_attract_scoring_object(
    kind: ActorAttractScoringObjectKind,
    x16: i32,
    y16: i32,
) -> ActorAttractScoringObject {
    ActorAttractScoringObject {
        kind,
        x16,
        y16,
        visual: ActorAttractScoringVisual::Sprite,
        visual_step: 0,
    }
}

fn actor_attract_scoring_bonus(
    stage: ActorAttractScoringStage,
    local_step: u16,
) -> Option<ActorAttractScoringBonus> {
    match stage {
        ActorAttractScoringStage::RescueScore => Some(ActorAttractScoringBonus {
            sprite: SpriteId::SCORE_POPUP_500,
            x16: ATTRACT_SCORING_SCORE_500_X16,
            y16: ATTRACT_SCORING_SCORE_500_Y16,
        }),
        ActorAttractScoringStage::RescueReturn => Some(ActorAttractScoringBonus {
            sprite: SpriteId::SCORE_POPUP_500,
            x16: ATTRACT_SCORING_SCORE_500_DROP_X16,
            y16: ATTRACT_SCORING_SCORE_500_DROP_Y16 + i32::from(local_step / 2) * 0x0010,
        }),
        ActorAttractScoringStage::LegendTransfer(index) if local_step == 0 => {
            let entry = ACTOR_ATTRACT_SCORING_LEGEND[index];
            Some(ActorAttractScoringBonus {
                sprite: SpriteId::SCORE_POPUP_250,
                x16: entry.table_x16,
                y16: entry.table_y16,
            })
        }
        _ => None,
    }
}

fn push_attract_scoring_scene_object(scene: &mut RenderScene, object: ActorAttractScoringObject) {
    if matches!(object.kind, ActorAttractScoringObjectKind::PlayerShot) {
        return;
    }

    if matches!(
        object.visual,
        ActorAttractScoringVisual::Explosion | ActorAttractScoringVisual::Materialize
    ) {
        push_actor_attract_scoring_fragment_pixels(scene, object);
        return;
    }

    let (sprite, size) = match object.kind {
        ActorAttractScoringObjectKind::PlayerShip => {
            (SpriteId::PLAYER_SHIP, PLAYER_SHIP_SCENE_SIZE)
        }
        ActorAttractScoringObjectKind::Human => (SpriteId::HUMAN, HUMAN_SCENE_SIZE),
        ActorAttractScoringObjectKind::PlayerShot => return,
        ActorAttractScoringObjectKind::Enemy(enemy) => (
            actor_attract_scoring_enemy_sprite(enemy),
            actor_attract_scoring_enemy_size(enemy),
        ),
    };
    scene.push_sprite(SceneSprite {
        sprite,
        layer: RenderLayer::Objects,
        position: actor_attract_scoring_scene_position(object.x16, object.y16),
        size,
        tint: Color::WHITE,
    });
}

fn push_attract_scoring_scanner_object(scene: &mut RenderScene, object: ActorAttractScoringObject) {
    let (sprite, size, color_word) = match object.kind {
        ActorAttractScoringObjectKind::PlayerShip => (
            SpriteId::SCANNER_PLAYER_BLIP,
            ATTRACT_SCORING_PLAYER_SCANNER_SIZE,
            ATTRACT_SCORING_PLAYER_SCANNER_COLOR_WORD,
        ),
        ActorAttractScoringObjectKind::Human => (
            SpriteId::SCANNER_OBJECT_BLIP,
            ATTRACT_SCORING_OBJECT_SCANNER_SIZE,
            ATTRACT_SCORING_HUMAN_SCANNER_COLOR_WORD,
        ),
        ActorAttractScoringObjectKind::PlayerShot => return,
        ActorAttractScoringObjectKind::Enemy(enemy) => {
            let color_word = ACTOR_ATTRACT_SCORING_LEGEND
                .iter()
                .find(|entry| entry.enemy == enemy)
                .map_or(ATTRACT_SCORING_LANDER_SCANNER_COLOR_WORD, |entry| {
                    entry.scanner_color_word
                });
            (
                SpriteId::SCANNER_OBJECT_BLIP,
                ATTRACT_SCORING_OBJECT_SCANNER_SIZE,
                color_word,
            )
        }
    };
    scene.push_sprite(SceneSprite {
        sprite,
        layer: RenderLayer::Hud,
        position: actor_attract_scoring_scanner_position(object),
        size,
        tint: source_pseudo_color_tint((color_word & 0x00FF) as u8),
    });
}

fn push_actor_attract_scoring_laser_beam(
    scene: &mut RenderScene,
    player_ship: ActorAttractScoringObject,
    target: ActorAttractScoringObject,
    display_step: u16,
) {
    let start = actor_attract_scoring_laser_ship_anchor(actor_attract_scoring_scene_position(
        player_ship.x16,
        player_ship.y16,
    ));
    let target_position = actor_attract_scoring_scene_position(target.x16, target.y16);
    let end_x = match target.kind {
        ActorAttractScoringObjectKind::Enemy(enemy) => {
            actor_attract_scoring_laser_enemy_anchor(enemy, target_position)[0]
        }
        _ => target_position[0],
    };
    push_actor_scoring_sparse_laser(scene, start[0], start[1], end_x, display_step);
}

fn actor_attract_scoring_laser_ship_anchor(position: [f32; 2]) -> [f32; 2] {
    [
        position[0] + PLAYER_SHIP_SCENE_SIZE[0],
        position[1] + PLAYER_SHIP_SCENE_SIZE[1] / 2.0 + 1.0,
    ]
}

fn actor_attract_scoring_laser_enemy_anchor(
    enemy: ActorAttractScoringEnemyKind,
    position: [f32; 2],
) -> [f32; 2] {
    let size = actor_attract_scoring_enemy_size(enemy);
    [position[0] + size[0] / 4.0, position[1] + size[1] / 2.0]
}

fn push_actor_scoring_sparse_laser(
    scene: &mut RenderScene,
    start_x: f32,
    y: f32,
    end_x: f32,
    display_step: u16,
) {
    let left = start_x.min(end_x).round() as i32;
    let right = start_x.max(end_x).round() as i32;
    if right <= left {
        return;
    }

    let direction = if end_x >= start_x { 1 } else { -1 };
    let head_x = if direction > 0 { right - 1 } else { left };
    let mut x = left;
    let mut cell = 0_i32;
    while x <= right {
        let cells_from_head = if direction > 0 {
            (head_x - x).div_euclid(SOURCE_LASER_BYTE_PIXELS)
        } else {
            (x - head_x).div_euclid(SOURCE_LASER_BYTE_PIXELS)
        }
        .max(0);
        let (byte, tint) = if cells_from_head == 0 {
            let byte = if x >= right {
                SOURCE_LASER_TIP_BYTE & 0xF0
            } else {
                SOURCE_LASER_TIP_BYTE
            };
            (byte, SOURCE_LASER_TIP_TINT)
        } else if cells_from_head <= SOURCE_LASER_BODY_CELLS {
            (SOURCE_LASER_BODY_BYTE, SOURCE_LASER_BODY_TINT)
        } else {
            let fizzle_seed = i32::from(display_step) + cell * 7 + x;
            let byte = if fizzle_seed.rem_euclid(5) == 0 {
                SOURCE_LASER_BODY_BYTE
            } else {
                actor_source_laser_fizzle_byte(fizzle_seed as u8)
            };
            (byte, SOURCE_LASER_FIZZLE_TINT)
        };
        push_actor_scoring_laser_byte(scene, x, y.round() as i32, byte, tint);
        x += SOURCE_LASER_BYTE_PIXELS;
        cell += 1;
    }
}

fn push_actor_scoring_laser_byte(scene: &mut RenderScene, x: i32, y: i32, byte: u8, tint: Color) {
    if byte & 0xF0 != 0 {
        push_actor_scoring_laser_pixel(scene, x, y, tint);
    }
    if byte & 0x0F != 0 {
        push_actor_scoring_laser_pixel(scene, x + 1, y, tint);
    }
}

fn push_actor_scoring_laser_pixel(scene: &mut RenderScene, x: i32, y: i32, tint: Color) {
    if x < 0 || y < 0 || x >= scene.surface.width as i32 || y >= scene.surface.height as i32 {
        return;
    }
    scene.push_sprite(SceneSprite {
        sprite: SpriteId::PLAYER_PROJECTILE,
        layer: RenderLayer::Projectiles,
        position: [x as f32, y as f32],
        size: PLAYER_EXPLOSION_PIXEL_SCENE_SIZE,
        tint,
    });
}

const fn actor_source_laser_fizzle_byte(seed: u8) -> u8 {
    (seed & 0x01) | ((seed & 0x02) << 3)
}

fn push_actor_attract_scoring_fragment_pixels(
    scene: &mut RenderScene,
    object: ActorAttractScoringObject,
) {
    let ActorAttractScoringObjectKind::Enemy(enemy) = object.kind else {
        return;
    };
    let position = actor_attract_scoring_scene_position(object.x16, object.y16);
    match object.visual {
        ActorAttractScoringVisual::Materialize => {
            push_actor_attract_scoring_materialize_pixels(
                scene,
                enemy,
                position,
                object.visual_step,
            );
        }
        ActorAttractScoringVisual::Explosion => {
            push_actor_attract_scoring_explosion_pixels(scene, enemy, position, object.visual_step);
        }
        ActorAttractScoringVisual::Sprite => {}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActorSourceObjectImagePixel {
    x: u8,
    y: u8,
    tint: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActorSourceObjectImageSpec {
    label: &'static str,
    rows: u8,
    bytes_per_row: u8,
}

fn push_actor_attract_scoring_materialize_pixels(
    scene: &mut RenderScene,
    enemy: ActorAttractScoringEnemyKind,
    position: [f32; 2],
    visual_step: u16,
) {
    let pixels = actor_source_object_image_pixels(enemy);
    if pixels.is_empty() {
        return;
    }
    let reveal_count = ((usize::from(visual_step) + 1) * pixels.len()
        / usize::from(ATTRACT_SCORING_LEGEND_TRANSFER_STEPS))
    .clamp(1, pixels.len());
    for (index, pixel) in pixels.iter().copied().enumerate().take(reveal_count) {
        let jitter_x = actor_source_fragment_jitter(index, visual_step, 0);
        let jitter_y = actor_source_fragment_jitter(index, visual_step, 3);
        push_actor_source_fragment_pixel(
            scene,
            [
                position[0] + f32::from(pixel.x) + jitter_x,
                position[1] + f32::from(pixel.y) + jitter_y,
            ],
            pixel.tint,
        );
    }
}

fn push_actor_attract_scoring_explosion_pixels(
    scene: &mut RenderScene,
    enemy: ActorAttractScoringEnemyKind,
    position: [f32; 2],
    visual_step: u16,
) {
    let pixels = actor_source_object_image_pixels(enemy);
    if pixels.is_empty() {
        return;
    }
    let size = actor_attract_scoring_enemy_size(enemy);
    let center = [position[0] + size[0] / 2.0, position[1] + size[1] / 2.0];
    let source_center = [size[0] / 2.0, size[1] / 2.0];
    let spread = 1.0 + f32::from(visual_step.min(18)) / 4.0;

    for (index, pixel) in pixels.iter().copied().enumerate() {
        if (index + usize::from(visual_step)).is_multiple_of(5) {
            continue;
        }
        let jitter_x = actor_source_fragment_jitter(index, visual_step, 5);
        let jitter_y = actor_source_fragment_jitter(index, visual_step, 9);
        push_actor_source_fragment_pixel(
            scene,
            [
                center[0] + (f32::from(pixel.x) - source_center[0]) * spread + jitter_x,
                center[1] + (f32::from(pixel.y) - source_center[1]) * spread + jitter_y,
            ],
            pixel.tint,
        );
    }
}

fn push_actor_source_fragment_pixel(scene: &mut RenderScene, position: [f32; 2], tint: Color) {
    if position[0] < 0.0
        || position[1] < 0.0
        || position[0] >= scene.surface.width as f32
        || position[1] >= scene.surface.height as f32
    {
        return;
    }
    scene.push_sprite(SceneSprite {
        sprite: SpriteId::PLAYER_EXPLOSION_PIXEL,
        layer: RenderLayer::Objects,
        position: [position[0].round(), position[1].round()],
        size: PLAYER_EXPLOSION_PIXEL_SCENE_SIZE,
        tint,
    });
}

fn actor_source_fragment_jitter(index: usize, visual_step: u16, salt: usize) -> f32 {
    match (index * 11 + usize::from(visual_step) * 3 + salt) % 7 {
        0 => -2.0,
        1 | 2 => -1.0,
        3 => 0.0,
        4 | 5 => 1.0,
        _ => 2.0,
    }
}

fn actor_source_object_image_pixels(
    enemy: ActorAttractScoringEnemyKind,
) -> Vec<ActorSourceObjectImagePixel> {
    let spec = actor_source_object_image_spec(enemy);
    let bytes = actor_source_object_image_bytes(spec.label);
    let expected_bytes = usize::from(spec.rows) * usize::from(spec.bytes_per_row);
    if bytes.len() != expected_bytes {
        return Vec::new();
    }

    let mut pixels = Vec::new();
    for column in 0..usize::from(spec.bytes_per_row) {
        let column_start = column * usize::from(spec.rows);
        for row in 0..usize::from(spec.rows) {
            let byte = bytes[column_start + row];
            if let Some(tint) = actor_source_picture_nibble_tint(byte >> 4) {
                pixels.push(ActorSourceObjectImagePixel {
                    x: (column * 2) as u8,
                    y: row as u8,
                    tint,
                });
            }
            if let Some(tint) = actor_source_picture_nibble_tint(byte & 0x0F) {
                pixels.push(ActorSourceObjectImagePixel {
                    x: (column * 2 + 1) as u8,
                    y: row as u8,
                    tint,
                });
            }
        }
    }
    pixels
}

fn actor_source_object_image_spec(
    enemy: ActorAttractScoringEnemyKind,
) -> ActorSourceObjectImageSpec {
    match enemy {
        ActorAttractScoringEnemyKind::Lander => ActorSourceObjectImageSpec {
            label: "LND10",
            rows: 8,
            bytes_per_row: 5,
        },
        ActorAttractScoringEnemyKind::Mutant => ActorSourceObjectImageSpec {
            label: "SCZD10",
            rows: 8,
            bytes_per_row: 5,
        },
        ActorAttractScoringEnemyKind::Baiter => ActorSourceObjectImageSpec {
            label: "UFOD10",
            rows: 4,
            bytes_per_row: 6,
        },
        ActorAttractScoringEnemyKind::Bomber => ActorSourceObjectImageSpec {
            label: "TIED10",
            rows: 8,
            bytes_per_row: 4,
        },
        ActorAttractScoringEnemyKind::Pod => ActorSourceObjectImageSpec {
            label: "PRBD10",
            rows: 8,
            bytes_per_row: 4,
        },
        ActorAttractScoringEnemyKind::Swarmer => ActorSourceObjectImageSpec {
            label: "SWMD10",
            rows: 4,
            bytes_per_row: 3,
        },
    }
}

fn actor_source_object_image_bytes(label: &'static str) -> Vec<u8> {
    for line in SOURCE_OBJECT_IMAGES_TSV.lines().skip(1) {
        let mut fields = line.split('\t');
        let Some(row_label) = fields.next() else {
            continue;
        };
        let _source_address = fields.next();
        let Some(hex_bytes) = fields.next() else {
            continue;
        };
        if row_label == label {
            return actor_decode_source_hex_bytes(label, hex_bytes);
        }
    }
    Vec::new()
}

fn actor_decode_source_hex_bytes(label: &'static str, hex_bytes: &str) -> Vec<u8> {
    assert!(
        hex_bytes.len().is_multiple_of(2),
        "source object image {label} hex payload must have whole bytes"
    );
    (0..hex_bytes.len())
        .step_by(2)
        .map(|start| {
            u8::from_str_radix(&hex_bytes[start..start + 2], 16).unwrap_or_else(|error| {
                panic!("source object image {label} byte must be hexadecimal: {error}")
            })
        })
        .collect()
}

fn actor_source_picture_nibble_tint(nibble: u8) -> Option<Color> {
    match nibble {
        0x0 => None,
        0x1 | 0xA | 0xC | 0xD | 0xE | 0xF => Some(Color::WHITE),
        0x2..=0x9 => Some(source_pseudo_color_tint(
            SOURCE_NORMAL_PALETTE_BYTES[usize::from(nibble)],
        )),
        0xB => Some(Color::from_rgba(170, 170, 186, 0xFF)),
        _ => None,
    }
}

fn actor_attract_scoring_enemy_sprite(enemy: ActorAttractScoringEnemyKind) -> SpriteId {
    match enemy {
        ActorAttractScoringEnemyKind::Lander => SpriteId::ENEMY_LANDER,
        ActorAttractScoringEnemyKind::Mutant => SpriteId::ENEMY_MUTANT,
        ActorAttractScoringEnemyKind::Baiter => SpriteId::ENEMY_BAITER,
        ActorAttractScoringEnemyKind::Bomber => SpriteId::ENEMY_BOMBER,
        ActorAttractScoringEnemyKind::Pod => SpriteId::ENEMY_POD,
        ActorAttractScoringEnemyKind::Swarmer => SpriteId::ENEMY_SWARMER,
    }
}

fn actor_attract_scoring_enemy_size(enemy: ActorAttractScoringEnemyKind) -> [f32; 2] {
    match enemy {
        ActorAttractScoringEnemyKind::Lander => LANDER_SCENE_SIZE,
        ActorAttractScoringEnemyKind::Mutant => MUTANT_SCENE_SIZE,
        ActorAttractScoringEnemyKind::Baiter => BAITER_SCENE_SIZE,
        ActorAttractScoringEnemyKind::Bomber => BOMBER_SCENE_SIZE,
        ActorAttractScoringEnemyKind::Pod => POD_SCENE_SIZE,
        ActorAttractScoringEnemyKind::Swarmer => SWARMER_SCENE_SIZE,
    }
}

fn actor_attract_scoring_scene_position(x16: i32, y16: i32) -> [f32; 2] {
    offset_f32_position(
        actor_attract_scoring_native_position(x16, y16),
        ATTRACT_SCORING_OBJECT_REFERENCE_OFFSET,
    )
}

fn actor_attract_scoring_scanner_position(object: ActorAttractScoringObject) -> [f32; 2] {
    let [native_x, native_y] = actor_attract_scoring_native_position(object.x16, object.y16);
    offset_f32_position(
        [
            ATTRACT_SCORING_SCANNER_ORIGIN[0]
                + native_x * ATTRACT_SCORING_SCANNER_SIZE[0] / ATTRACT_SCORING_PLAYFIELD_SIZE[0],
            ATTRACT_SCORING_SCANNER_ORIGIN[1]
                + native_y * ATTRACT_SCORING_SCANNER_SIZE[1] / ATTRACT_SCORING_PLAYFIELD_SIZE[1],
        ],
        point_position(SOURCE_ATTRACT_SCORING_VISUAL_OFFSET),
    )
}

fn actor_attract_scoring_native_position(x16: i32, y16: i32) -> [f32; 2] {
    [
        ((x16 + 0x10) >> 5).clamp(0, 319) as f32,
        ((y16 + 0x80) >> 8).clamp(0, 255) as f32,
    ]
}

fn source_pseudo_color_tint(value: u8) -> Color {
    if value == 0 {
        return Color::from_rgba(0, 0, 0, 0);
    }
    Color::from_rgba(
        SOURCE_WILLIAMS_RED_GREEN_LEVELS[usize::from(value & 0x07)],
        SOURCE_WILLIAMS_RED_GREEN_LEVELS[usize::from((value >> 3) & 0x07)],
        SOURCE_WILLIAMS_BLUE_LEVELS[usize::from((value >> 6) & 0x03)],
        0xFF,
    )
}

fn source_scanner_mini_terrain_records()
-> &'static [SourceScannerTerrainRecord; SOURCE_SCANNER_TERRAIN_RECORDS] {
    static RECORDS: OnceLock<[SourceScannerTerrainRecord; SOURCE_SCANNER_TERRAIN_RECORDS]> =
        OnceLock::new();
    RECORDS.get_or_init(|| {
        source_generate_scanner_mini_terrain_records(
            0u16.wrapping_sub(SOURCE_SCANNER_SCAN_CENTER_OFFSET),
        )
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SourceScannerTerrainRecord {
    screen_address: u16,
    word: u16,
}

fn source_generate_scanner_mini_terrain_records(
    scan_left: u16,
) -> [SourceScannerTerrainRecord; SOURCE_SCANNER_TERRAIN_RECORDS] {
    let bytes = source_mterr_bytes();
    let first_record = usize::from(scan_left.to_be_bytes()[0] >> 2);
    assert!(
        first_record + SOURCE_SCANNER_TERRAIN_RECORDS <= SOURCE_SCANNER_MINI_TERRAIN_RECORDS,
        "MTERR slice must contain 64 source scanner terrain records"
    );

    let mut records = [SourceScannerTerrainRecord::default(); SOURCE_SCANNER_TERRAIN_RECORDS];
    let mut source_column = SOURCE_SCANNER_OBJECT_BASE_SCREEN.to_be_bytes()[0];
    for (index, record) in records.iter_mut().enumerate() {
        let source_index = (first_record + index) * 3;
        *record = SourceScannerTerrainRecord {
            screen_address: u16::from_be_bytes([source_column, bytes[source_index]]),
            word: u16::from_be_bytes([bytes[source_index + 1], bytes[source_index + 2]]),
        };
        source_column = source_column.wrapping_add(1);
    }

    records
}

fn source_mterr_bytes() -> &'static [u8; SOURCE_TERRAIN_MTERR_BYTES] {
    static MTERR: OnceLock<[u8; SOURCE_TERRAIN_MTERR_BYTES]> = OnceLock::new();
    MTERR.get_or_init(parse_source_mterr_bytes)
}

fn parse_source_mterr_bytes() -> [u8; SOURCE_TERRAIN_MTERR_BYTES] {
    let mut output = [0; SOURCE_TERRAIN_MTERR_BYTES];
    for (line_index, line) in SOURCE_TERRAIN_DATA_TSV.lines().enumerate().skip(1) {
        let mut fields = line.split('\t');
        let label = fields.next().unwrap_or_default();
        let address = fields.next().unwrap_or_default();
        let bytes = fields.next().unwrap_or_default();
        if label != SOURCE_TERRAIN_MTERR_LABEL {
            continue;
        }
        let expected_address = format!("0x{SOURCE_TERRAIN_MTERR_ADDRESS:04X}");
        assert_eq!(
            address,
            expected_address.as_str(),
            "terrain-data line {} must preserve MTERR source address",
            line_index + 1
        );
        assert_eq!(
            bytes.len(),
            SOURCE_TERRAIN_MTERR_BYTES * 2,
            "MTERR hex payload must contain exactly 0x180 bytes"
        );
        for index in 0..SOURCE_TERRAIN_MTERR_BYTES {
            output[index] = parse_source_hex_byte(&bytes[index * 2..index * 2 + 2]);
        }
        return output;
    }

    panic!("terrain-data.tsv must contain the MTERR record")
}

fn parse_source_hex_byte(value: &str) -> u8 {
    u8::from_str_radix(value, 16).expect("source terrain byte must be hexadecimal")
}

fn offset_new_sprites(scene: &mut RenderScene, first_sprite: usize, offset: [f32; 2]) {
    if offset == [0.0, 0.0] {
        return;
    }
    for sprite in &mut scene.sprites[first_sprite..] {
        sprite.position = offset_f32_position(sprite.position, offset);
    }
}

fn offset_f32_position(position: [f32; 2], offset: [f32; 2]) -> [f32; 2] {
    [position[0] + offset[0], position[1] + offset[1]]
}

fn actor_source_explosion_render_scale(source_size: u16) -> f32 {
    source_explosion_render_scale(source_size)
        .map(f32::from)
        .unwrap_or(1.0)
}

fn actor_source_explosion_size_for_kind(kind: ExplosionKind, age: u16) -> u16 {
    if kind == ExplosionKind::Terrain {
        return source_terrain_explosion_size_for_age(
            u8::try_from(age).unwrap_or(SOURCE_TERRAIN_EXPLOSION_LIFETIME_FRAMES),
        );
    }

    source_explosion_size_for_age(age)
}

fn point_position(point: Point) -> [f32; 2] {
    [f32::from(point.x), f32::from(point.y)]
}

fn williams_reveal_visible_pixel_count(stroke_step: u16, total_pixels: usize) -> usize {
    if total_pixels == 0 {
        return 0;
    }
    if stroke_step >= WILLIAMS_REVEAL_STEPS {
        return total_pixels;
    }
    if stroke_step == 0 {
        return 0;
    }

    let operation_counts = source_attract_williams_logo_operation_pixel_counts();
    let operation_index = usize::from(stroke_step)
        .saturating_mul(operation_counts.len())
        .checked_div(usize::from(WILLIAMS_REVEAL_STEPS))
        .unwrap_or(0)
        .saturating_sub(1);
    operation_counts
        .get(operation_index)
        .copied()
        .unwrap_or(total_pixels)
        .clamp(1, total_pixels)
}

fn williams_logo_phase_tint(color_phase: u8) -> Color {
    match color_phase % 4 {
        0 => Color::from_rgba(0xFF, 0xFF, 0xFF, 0xFF),
        1 => Color::from_rgba(0xFF, 0xD8, 0x40, 0xFF),
        2 => Color::from_rgba(0x80, 0xE8, 0xFF, 0xFF),
        _ => Color::from_rgba(0xFF, 0x80, 0xE8, 0xFF),
    }
}

fn actor_scene_sprite(sprite: SpriteKey, position: Point) -> Option<SceneSprite> {
    let (sprite, layer, size, tint) = match sprite {
        SpriteKey::WilliamsLogo => (
            SpriteId::ATTRACT_WILLIAMS_LOGO,
            RenderLayer::Overlay,
            WILLIAMS_LOGO_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::DefenderCoalescence => return None,
        SpriteKey::DefenderWordmark | SpriteKey::DefenderLogo => (
            SpriteId::HALL_OF_FAME_DEFENDER_LOGO,
            RenderLayer::Overlay,
            DEFENDER_WORDMARK_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::HighScoreText => (
            SpriteId::STATUS_TEXT,
            RenderLayer::Overlay,
            [8.0, 8.0],
            Color::WHITE,
        ),
        SpriteKey::PlayerRight => (
            SpriteId::PLAYER_SHIP,
            RenderLayer::Objects,
            PLAYER_SHIP_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::PlayerLeft => (
            SpriteId::PLAYER_SHIP_LEFT,
            RenderLayer::Objects,
            PLAYER_SHIP_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Lander => (
            SpriteId::ENEMY_LANDER,
            RenderLayer::Objects,
            LANDER_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Mutant => (
            SpriteId::ENEMY_MUTANT,
            RenderLayer::Objects,
            MUTANT_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Bomber => (
            SpriteId::ENEMY_BOMBER,
            RenderLayer::Objects,
            BOMBER_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Bomb => (
            SpriteId::ENEMY_BOMB,
            RenderLayer::Projectiles,
            ENEMY_BOMB_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Pod => (
            SpriteId::ENEMY_POD,
            RenderLayer::Objects,
            POD_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Swarmer => (
            SpriteId::ENEMY_SWARMER,
            RenderLayer::Objects,
            SWARMER_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Baiter => (
            SpriteId::ENEMY_BAITER,
            RenderLayer::Objects,
            BAITER_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Human | SpriteKey::HumanFalling => (
            SpriteId::HUMAN,
            RenderLayer::Objects,
            HUMAN_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::HumanCarried => (
            SpriteId::HUMAN,
            RenderLayer::Objects,
            HUMAN_SCENE_SIZE,
            Color::from_rgba(0xFF, 0xF8, 0x80, 0xFF),
        ),
        SpriteKey::Laser => (
            SpriteId::PLAYER_PROJECTILE,
            RenderLayer::Projectiles,
            PLAYER_PROJECTILE_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::EnemyLaser => (
            SpriteId::ENEMY_BOMB,
            RenderLayer::Projectiles,
            ENEMY_BOMB_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Explosion => (
            SpriteId::BOMB_EXPLOSION,
            RenderLayer::Objects,
            EXPLOSION_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Score250 => (
            SpriteId::SCORE_POPUP_250,
            RenderLayer::Objects,
            SCORE_POPUP_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Score500 => (
            SpriteId::SCORE_POPUP_500,
            RenderLayer::Objects,
            SCORE_POPUP_SCENE_SIZE,
            Color::WHITE,
        ),
        SpriteKey::Text => return None,
    };

    Some(SceneSprite {
        sprite,
        layer,
        position: point_position(position),
        size,
        tint,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActorFrame {
    pub state: GameState,
    pub report: StepReport,
    pub events: GameEvents,
    pub scene: RenderScene,
}

impl ActorFrame {
    pub fn new(
        report: StepReport,
        state: GameState,
        events: GameEvents,
        scene: RenderScene,
    ) -> Self {
        Self {
            state,
            report,
            events,
            scene,
        }
    }

    pub fn game_frame(&self) -> GameFrame {
        GameFrame {
            state: self.state.clone(),
            events: self.events.clone(),
            scene: self.scene.clone(),
        }
    }
}

pub struct ActorRuntimeAdapter {
    driver: ActorGameDriver,
    sound_bridge: ActorSoundEventBridge,
    render_bridge: ActorRenderSceneBridge,
}

impl ActorRuntimeAdapter {
    pub fn new() -> Self {
        Self::with_driver(ActorGameDriver::new())
    }

    pub fn with_scripts(scripts: ActorDriverScripts) -> Self {
        Self::with_driver(ActorGameDriver::with_scripts(scripts))
    }

    pub fn with_driver(driver: ActorGameDriver) -> Self {
        Self::with_components(
            driver,
            ActorSoundEventBridge::new(),
            ActorRenderSceneBridge::new(),
        )
    }

    pub fn with_components(
        driver: ActorGameDriver,
        sound_bridge: ActorSoundEventBridge,
        render_bridge: ActorRenderSceneBridge,
    ) -> Self {
        Self {
            driver,
            sound_bridge,
            render_bridge,
        }
    }

    pub fn driver(&self) -> &ActorGameDriver {
        &self.driver
    }

    pub fn driver_mut(&mut self) -> &mut ActorGameDriver {
        &mut self.driver
    }

    pub fn step(&mut self, input: GameInput) -> ActorFrame {
        let report = self.driver.step(input);
        self.frame_for_report(report)
    }

    pub fn step_clean_input(&mut self, input: CleanGameInput, xyzzy: XyzzyMode) -> ActorFrame {
        self.step(GameInput::from_clean_input(input, xyzzy))
    }

    fn frame_for_report(&mut self, report: StepReport) -> ActorFrame {
        let state = report.game_state();
        let gameplay_events = actor_gameplay_events_for_report(&report);
        let sound_events = self.sound_bridge.sound_events_for_report(&report);
        let scene = self.render_bridge.render_scene_for_report(&report);
        ActorFrame::new(
            report,
            state,
            GameEvents::new(gameplay_events, sound_events),
            scene,
        )
    }
}

impl Default for ActorRuntimeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct HighScoreEntryStep {
    accepted: bool,
    submitted: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AppliedCommands {
    sounds: Vec<SoundCue>,
    draws: Vec<DrawCommand>,
    bonus_awarded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingSurvivorBonus {
    next_wave: u16,
    multiplier: u8,
    total_survivors: u8,
    visible_icons: u8,
    remaining_awards: u8,
    astronaut_sleep_steps_remaining: u8,
    wave_advance_sleep_steps_remaining: Option<u8>,
}

impl PendingSurvivorBonus {
    fn new(current_wave: u16, next_wave: u16, survivors: usize) -> Self {
        let total_survivors = u8::try_from(survivors.min(SOURCE_SURVIVOR_BONUS_HUMAN_LIMIT))
            .unwrap_or(SOURCE_SURVIVOR_BONUS_HUMAN_LIMIT as u8);
        Self {
            next_wave,
            multiplier: clean_wave(current_wave).min(5),
            total_survivors,
            visible_icons: 0,
            remaining_awards: total_survivors,
            astronaut_sleep_steps_remaining: 0,
            wave_advance_sleep_steps_remaining: None,
        }
    }

    fn bonus_points(&self) -> u32 {
        u32::from(self.multiplier) * 100
    }

    fn award_next_survivor(&mut self) -> Option<u32> {
        if self.remaining_awards == 0 {
            return None;
        }

        self.remaining_awards = self.remaining_awards.saturating_sub(1);
        self.visible_icons = self
            .visible_icons
            .saturating_add(1)
            .min(self.total_survivors);
        self.astronaut_sleep_steps_remaining = SOURCE_SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS;
        Some(self.bonus_points())
    }

    fn report(self, awarded_points: Option<u32>) -> SurvivorBonusReport {
        SurvivorBonusReport {
            next_wave: self.next_wave,
            multiplier: self.multiplier,
            total_survivors: self.total_survivors,
            visible_icons: self.visible_icons,
            remaining_awards: self.remaining_awards,
            awarded_points,
            astronaut_sleep_steps_remaining: self.astronaut_sleep_steps_remaining,
            wave_advance_sleep_steps_remaining: self.wave_advance_sleep_steps_remaining,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingPlayerSwitch {
    sleep_steps_remaining: u8,
    from_player: u8,
    to_player: u8,
}

impl PendingPlayerSwitch {
    const fn new(from_player: u8, to_player: u8) -> Self {
        Self {
            sleep_steps_remaining: SOURCE_PLAYER_SWITCH_SLEEP_STEPS,
            from_player,
            to_player,
        }
    }

    const fn report(self) -> PlayerSwitchReport {
        PlayerSwitchReport {
            sleep_steps_remaining: self.sleep_steps_remaining,
            from_player: self.from_player,
            to_player: self.to_player,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingPlayerStart {
    delay_steps_remaining: u8,
    player: u8,
}

impl PendingPlayerStart {
    const fn new(player: u8) -> Self {
        Self {
            delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS,
            player,
        }
    }

    const fn report(self) -> PlayerStartReport {
        PlayerStartReport {
            delay_steps_remaining: self.delay_steps_remaining,
            player: self.player,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingActorSoundCommand {
    steps_remaining: u8,
    command: u8,
    source: PendingActorSoundSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingActorSoundSource {
    SmartBomb,
    TerrainBlow,
    AstronautRescue,
    FirstWaveLanderRefill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SurvivorBonusStep {
    Waiting,
    Award(u32),
    StartNextWave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerStartStep {
    Waiting,
    StartPlayfield,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorDriverScripts {
    pub attract_script: AttractScript,
    pub behavior_script: ActorBehaviorScript,
    pub wave_script: ActorWaveScript,
}

impl ActorDriverScripts {
    pub fn new(
        attract_script: AttractScript,
        behavior_script: ActorBehaviorScript,
        wave_script: ActorWaveScript,
    ) -> Self {
        Self {
            attract_script,
            behavior_script,
            wave_script,
        }
    }

    pub fn parse_text(source: &str) -> Result<Self, ActorDriverScriptsParseError> {
        let sections = ParsedActorDriverScriptSections::parse(source)?;
        Self::parse_texts(
            sections.attract.as_str(),
            sections.behavior.as_str(),
            sections.wave.as_str(),
        )
    }

    pub fn parse_texts(
        attract_source: &str,
        behavior_source: &str,
        wave_source: &str,
    ) -> Result<Self, ActorDriverScriptsParseError> {
        let attract_script = AttractScript::parse_text(attract_source)
            .map_err(ActorDriverScriptsParseError::from_attract)?;
        let behavior_script = ActorBehaviorScript::parse_text(behavior_source)
            .map_err(ActorDriverScriptsParseError::from_behavior)?;
        let wave_script =
            ActorWaveScript::parse_text_with_base_behavior(wave_source, &behavior_script)
                .map_err(ActorDriverScriptsParseError::from_wave)?;
        Ok(Self::new(attract_script, behavior_script, wave_script))
    }

    pub fn manifest(&self) -> ActorDriverScriptsManifest {
        ActorDriverScriptsManifest {
            attract_script: self.attract_script.manifest(),
            behavior_script: self.behavior_script.manifest(),
            wave_script: self.wave_script.manifest(),
        }
    }
}

impl FromStr for ActorDriverScripts {
    type Err = ActorDriverScriptsParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::parse_text(source)
    }
}

impl Default for ActorDriverScripts {
    fn default() -> Self {
        let behavior_script = ActorBehaviorScript::red_label_default();
        let wave_script = ActorWaveScript::parse_text_with_base_behavior(
            ACTOR_RED_LABEL_WAVE_SCRIPT,
            &behavior_script,
        )
        .unwrap_or_else(|error| panic!("embedded actor wave script is invalid: {error}"));
        Self::new(
            AttractScript::red_label_title(),
            behavior_script,
            wave_script,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorDriverScriptsManifest {
    pub attract_script: AttractScriptManifest,
    pub behavior_script: ActorBehaviorScriptManifest,
    pub wave_script: ActorWaveScriptManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorDriverScriptSection {
    Driver,
    Attract,
    Behavior,
    Wave,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorDriverScriptsParseError {
    pub section: ActorDriverScriptSection,
    pub line: usize,
    pub message: String,
}

impl ActorDriverScriptsParseError {
    fn new(section: ActorDriverScriptSection, line: usize, message: impl Into<String>) -> Self {
        Self {
            section,
            line,
            message: message.into(),
        }
    }

    fn from_attract(error: AttractScriptParseError) -> Self {
        Self::new(ActorDriverScriptSection::Attract, error.line, error.message)
    }

    fn from_behavior(error: ActorBehaviorScriptParseError) -> Self {
        Self::new(
            ActorDriverScriptSection::Behavior,
            error.line,
            error.message,
        )
    }

    fn from_wave(error: ActorWaveScriptParseError) -> Self {
        Self::new(ActorDriverScriptSection::Wave, error.line, error.message)
    }
}

impl fmt::Display for ActorDriverScriptsParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let section = match self.section {
            ActorDriverScriptSection::Driver => "driver",
            ActorDriverScriptSection::Attract => "attract",
            ActorDriverScriptSection::Behavior => "behavior",
            ActorDriverScriptSection::Wave => "wave",
        };
        write!(
            formatter,
            "actor driver {section} script line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for ActorDriverScriptsParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorDriverScriptBundleSection {
    Attract,
    Behavior,
    Wave,
}

impl ActorDriverScriptBundleSection {
    fn parse(line_number: usize, token: &str) -> Result<Self, ActorDriverScriptsParseError> {
        match normalize_script_token(token).as_str() {
            "attract" | "attract_script" => Ok(Self::Attract),
            "behavior" | "behaviour" | "behavior_script" | "behaviour_script" => Ok(Self::Behavior),
            "wave" | "waves" | "wave_script" => Ok(Self::Wave),
            _ => Err(ActorDriverScriptsParseError::new(
                ActorDriverScriptSection::Driver,
                line_number,
                format!("unknown driver script section `{token}`"),
            )),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ParsedActorDriverScriptSections {
    attract: String,
    behavior: String,
    wave: String,
    saw_attract: bool,
    saw_behavior: bool,
    saw_wave: bool,
}

impl ParsedActorDriverScriptSections {
    fn parse(source: &str) -> Result<Self, ActorDriverScriptsParseError> {
        let mut sections = Self::default();
        let mut current = None;
        for (line_index, raw_line) in source.lines().enumerate() {
            let line_number = line_index + 1;
            let line = raw_line
                .split_once('#')
                .map_or(raw_line, |(before_comment, _)| before_comment)
                .trim();
            if line.is_empty() {
                continue;
            }
            if let Some(section) = parse_actor_driver_script_section_header(line_number, line)? {
                current = Some(section);
                sections.mark_seen(section);
                continue;
            }
            let Some(section) = current else {
                return Err(ActorDriverScriptsParseError::new(
                    ActorDriverScriptSection::Driver,
                    line_number,
                    "driver script line must appear inside [attract], [behavior], or [wave]",
                ));
            };
            sections.push_line(section, line_number, line);
        }
        sections.require_sections()?;
        Ok(sections)
    }

    fn mark_seen(&mut self, section: ActorDriverScriptBundleSection) {
        match section {
            ActorDriverScriptBundleSection::Attract => self.saw_attract = true,
            ActorDriverScriptBundleSection::Behavior => self.saw_behavior = true,
            ActorDriverScriptBundleSection::Wave => self.saw_wave = true,
        }
    }

    fn push_line(
        &mut self,
        section: ActorDriverScriptBundleSection,
        line_number: usize,
        line: &str,
    ) {
        let target = match section {
            ActorDriverScriptBundleSection::Attract => &mut self.attract,
            ActorDriverScriptBundleSection::Behavior => &mut self.behavior,
            ActorDriverScriptBundleSection::Wave => &mut self.wave,
        };
        append_source_line_with_original_number(target, line_number, line);
    }

    fn require_sections(&self) -> Result<(), ActorDriverScriptsParseError> {
        if !self.saw_attract {
            return Err(ActorDriverScriptsParseError::new(
                ActorDriverScriptSection::Attract,
                0,
                "driver script needs an [attract] section",
            ));
        }
        if !self.saw_behavior {
            return Err(ActorDriverScriptsParseError::new(
                ActorDriverScriptSection::Behavior,
                0,
                "driver script needs a [behavior] section",
            ));
        }
        if !self.saw_wave {
            return Err(ActorDriverScriptsParseError::new(
                ActorDriverScriptSection::Wave,
                0,
                "driver script needs a [wave] section",
            ));
        }
        Ok(())
    }
}

fn parse_actor_driver_script_section_header(
    line_number: usize,
    line: &str,
) -> Result<Option<ActorDriverScriptBundleSection>, ActorDriverScriptsParseError> {
    let Some(name) = line
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    else {
        return Ok(None);
    };
    Ok(Some(ActorDriverScriptBundleSection::parse(
        line_number,
        name.trim(),
    )?))
}

fn append_source_line_with_original_number(target: &mut String, line_number: usize, line: &str) {
    let current_line_count = target.lines().count();
    for _ in current_line_count..line_number.saturating_sub(1) {
        target.push('\n');
    }
    target.push_str(line);
    target.push('\n');
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorDriverScriptManifest {
    pub step: u64,
    pub phase: Phase,
    pub wave: u16,
    pub attract_script: AttractScriptManifest,
    pub behavior_script: ActorBehaviorScriptManifest,
    pub wave_script: ActorWaveScriptManifest,
    pub current_wave_profile: ActorWaveProfileManifest,
}

fn actor_gameplay_events_for_report(report: &StepReport) -> Vec<GameEvent> {
    let mut events = Vec::new();
    for command in &report.commands {
        match command {
            GameCommand::Credit => push_unique_game_event(&mut events, GameEvent::CreditAdded),
            GameCommand::StartOnePlayer | GameCommand::StartTwoPlayer => {
                if report.phase == Phase::Playing {
                    push_unique_game_event(&mut events, GameEvent::GameStarted);
                    if report.player_start.is_none() {
                        push_unique_game_event(&mut events, GameEvent::WaveStarted);
                    }
                }
            }
            GameCommand::SmartBomb { .. } => {
                push_unique_game_event(&mut events, GameEvent::SmartBombPressed)
            }
            GameCommand::Hyperspace => {
                push_unique_game_event(&mut events, GameEvent::HyperspacePressed)
            }
            GameCommand::PlayerKilled => {
                push_unique_game_event(&mut events, GameEvent::PlayerDestroyed)
            }
            GameCommand::WaveCleared { .. } => {
                push_unique_game_event(&mut events, GameEvent::WaveCleared);
            }
            GameCommand::AdvanceWave { .. } => {
                push_unique_game_event(&mut events, GameEvent::WaveStarted);
            }
            GameCommand::EnterGameOver => push_unique_game_event(&mut events, GameEvent::GameOver),
            GameCommand::Spawn(_)
            | GameCommand::Destroy(_)
            | GameCommand::SetSourceBackgroundLeft(_)
            | GameCommand::AttachHuman { .. }
            | GameCommand::HumanLost(_)
            | GameCommand::AddScore(_)
            | GameCommand::PlaySound(_) => {}
        }
    }
    if report.sounds.contains(&SoundCue::GameOver) {
        push_unique_game_event(&mut events, GameEvent::GameOver);
    }
    if report.phase == Phase::HighScoreEntry {
        push_unique_game_event(&mut events, GameEvent::HighScoreEntryStarted);
    }
    if report.high_score_initial_accepted {
        push_unique_game_event(&mut events, GameEvent::HighScoreInitialAccepted);
    }
    if report.high_score_submitted {
        push_unique_game_event(&mut events, GameEvent::HighScoreSubmitted);
    }
    if report.bonus_awarded {
        push_unique_game_event(&mut events, GameEvent::BonusAwarded);
    }
    events
}

fn push_unique_game_event(events: &mut Vec<GameEvent>, event: GameEvent) {
    if !events.contains(&event) {
        events.push(event);
    }
}

pub struct ActorGameDriver {
    step: u64,
    phase: Phase,
    wave: u16,
    current_player: u8,
    player_count: u8,
    score: u32,
    player_two_score: u32,
    credits: u8,
    lives: u8,
    smart_bombs: u8,
    player_two_lives: u8,
    player_two_smart_bombs: u8,
    next_bonus: u32,
    next_actor_id: u64,
    actors: BTreeMap<ActorId, ThreadedAsset>,
    snapshots: BTreeMap<ActorId, ActorSnapshot>,
    high_scores: HighScoreTable,
    high_score_initials: HighScoreInitialsState,
    attract_script: AttractScript,
    behavior_script: ActorBehaviorScript,
    wave_script: ActorWaveScript,
    wave_spawn_allocations: BTreeMap<ActorKind, usize>,
    enemy_reserve: EnemyReserveSnapshot,
    source_target_cursor: Option<usize>,
    source_astronaut_cursor: Option<usize>,
    source_astronaut_sleep_ticks: u8,
    source_reserve_activation_ready: bool,
    source_reserve_activation_cooldown_steps: u16,
    source_first_wave_early_reserve_steps_remaining: Option<u16>,
    source_first_wave_lander_refill_steps_remaining: Option<u8>,
    source_background_left: u16,
    baiter_timer_steps: Option<u32>,
    baiter_pacing_steps_remaining: u8,
    source_rng: ActorSourceRng,
    source_shell_scan_steps_remaining: u8,
    pending_smart_bomb_detonation_steps: Option<u8>,
    smart_bomb_flash_steps_remaining: u8,
    pending_sound_commands: Vec<PendingActorSoundCommand>,
    terrain_blow: Option<TerrainBlowSnapshot>,
    game_over_hall_of_fame_stall_remaining: Option<u8>,
    pending_survivor_bonus: Option<PendingSurvivorBonus>,
    pending_player_switch: Option<PendingPlayerSwitch>,
    pending_player_start: Option<PendingPlayerStart>,
    pending_start_sound_steps: Option<u8>,
}

impl ActorGameDriver {
    pub fn new() -> Self {
        Self::with_scripts(ActorDriverScripts::default())
    }

    pub fn with_attract_script(attract_script: AttractScript) -> Self {
        Self::with_attract_and_wave_scripts(attract_script, ActorWaveScript::default())
    }

    pub fn with_wave_script(wave_script: ActorWaveScript) -> Self {
        Self::with_attract_and_wave_scripts(AttractScript::red_label_title(), wave_script)
    }

    pub fn with_attract_and_wave_scripts(
        attract_script: AttractScript,
        wave_script: ActorWaveScript,
    ) -> Self {
        Self::with_attract_behavior_and_wave_scripts(
            attract_script,
            ActorBehaviorScript::default(),
            wave_script,
        )
    }

    pub fn with_scripts(scripts: ActorDriverScripts) -> Self {
        Self::with_attract_behavior_and_wave_scripts(
            scripts.attract_script,
            scripts.behavior_script,
            scripts.wave_script,
        )
    }

    pub fn with_attract_behavior_and_wave_scripts(
        attract_script: AttractScript,
        behavior_script: ActorBehaviorScript,
        wave_script: ActorWaveScript,
    ) -> Self {
        let mut driver = Self {
            step: 0,
            phase: Phase::Attract,
            wave: 0,
            current_player: 1,
            player_count: 1,
            score: 0,
            player_two_score: 0,
            credits: 0,
            lives: INITIAL_PLAYER_LIVES,
            smart_bombs: 0,
            player_two_lives: INITIAL_PLAYER_LIVES,
            player_two_smart_bombs: 0,
            next_bonus: SOURCE_REPLAY_SCORE,
            next_actor_id: 1,
            actors: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            high_scores: HighScoreTable::default(),
            high_score_initials: HighScoreInitialsState::EMPTY,
            attract_script: attract_script.clone(),
            behavior_script,
            wave_script,
            wave_spawn_allocations: BTreeMap::new(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            source_target_cursor: None,
            source_astronaut_cursor: Some(0),
            source_astronaut_sleep_ticks: 0,
            source_reserve_activation_ready: false,
            source_reserve_activation_cooldown_steps: 0,
            source_first_wave_early_reserve_steps_remaining: None,
            source_first_wave_lander_refill_steps_remaining: None,
            source_background_left: 0,
            baiter_timer_steps: None,
            baiter_pacing_steps_remaining: ACTOR_BAITER_TIMER_PACING_STEPS,
            source_rng: SOURCE_PLAYFIELD_START_RNG,
            source_shell_scan_steps_remaining: SOURCE_SHELL_SCAN_INITIAL_DELAY_STEPS,
            pending_smart_bomb_detonation_steps: None,
            smart_bomb_flash_steps_remaining: 0,
            pending_sound_commands: Vec::new(),
            terrain_blow: None,
            game_over_hall_of_fame_stall_remaining: None,
            pending_survivor_bonus: None,
            pending_player_switch: None,
            pending_player_start: None,
            pending_start_sound_steps: None,
        };
        let attract_id = driver.allocate_actor_id();
        let script_id = driver.allocate_actor_id();
        let status_id = driver.allocate_actor_id();
        driver.spawn_actor(AttractDirector::new(attract_id));
        driver.spawn_actor(ScriptedAttractProgram::new(script_id, attract_script));
        driver.spawn_actor(StatusDisplay::new(status_id));
        driver
    }

    pub fn step(&mut self, input: GameInput) -> StepReport {
        self.step = self.step.saturating_add(1);
        let mut step_commands = Vec::new();
        let mut delayed_sounds = self.advance_pending_start_sound();
        delayed_sounds.extend(self.advance_pending_sound_commands());
        self.advance_smart_bomb_flash();
        delayed_sounds.extend(self.advance_terrain_blow(&mut step_commands));
        let smart_bomb_replay_awarded =
            self.advance_pending_smart_bomb_detonation(&mut step_commands);
        let mut survivor_bonus_awarded_points = None;
        let mut survivor_bonus_replay_awarded = smart_bomb_replay_awarded;
        if let PlayerStartStep::StartPlayfield = self.advance_pending_player_start() {
            step_commands.push(GameCommand::AdvanceWave { wave: self.wave });
            delayed_sounds.push(SoundCue::PlayerAppear);
        }
        self.advance_pending_player_switch();
        self.advance_source_reserve_activation_cooldown();
        delayed_sounds.extend(self.activate_enemy_reserve_if_ready(&mut step_commands));
        self.advance_first_wave_lander_refill_if_ready(&mut step_commands);
        match self.advance_pending_survivor_bonus() {
            SurvivorBonusStep::StartNextWave => {
                self.start_pending_wave();
                step_commands.push(GameCommand::AdvanceWave { wave: self.wave });
            }
            SurvivorBonusStep::Award(points) => {
                survivor_bonus_awarded_points = Some(points);
                if self.award_points(points) {
                    survivor_bonus_replay_awarded = true;
                }
            }
            SurvivorBonusStep::Waiting => {}
        }
        let pre_applied_command_count = step_commands.len();
        let survivor_bonus_interstitial = self.pending_survivor_bonus.is_some();
        let prompt_player_switch = self.pending_player_switch.map(PendingPlayerSwitch::report);
        let player_switch_interstitial = prompt_player_switch.is_some();
        let prompt_player_start = self.pending_player_start.map(PendingPlayerStart::report);
        let player_start_interstitial = prompt_player_start.is_some();
        let was_playing = self.phase == Phase::Playing;
        let effective_input = if survivor_bonus_interstitial
            || player_switch_interstitial
            || player_start_interstitial
        {
            GameInput::NONE
        } else {
            input
        };
        let high_score_entry_step = self.apply_high_score_entry_input(effective_input);
        let mut behavior_script = self
            .behavior_script
            .with_input_overrides(effective_input, self.snapshots.values().cloned());
        let source_rng = if self.phase == Phase::Playing
            && !survivor_bonus_interstitial
            && !player_switch_interstitial
            && !player_start_interstitial
        {
            Some(self.source_rng.advance().snapshot())
        } else {
            None
        };
        if let Some(source_rng) = source_rng {
            behavior_script =
                behavior_script.with_hyperspace_source_seed(source_rng.hyperspace_seed());
        }
        let source_human_walk_target_slot = self.advance_source_astronaut_process(source_rng);
        let source_shell_scan_tick = if self.phase == Phase::Playing
            && !survivor_bonus_interstitial
            && !player_switch_interstitial
            && !player_start_interstitial
        {
            self.advance_source_shell_scan_tick()
        } else {
            false
        };
        let prompt_credits = self
            .credits
            .saturating_add(effective_input.coin_insertions());
        let base_prompt = StepPrompt {
            step: self.step,
            phase: self.phase,
            input: effective_input,
            wave: self.wave,
            source_wave: self.current_source_wave_profile(),
            current_player: self.current_player,
            player_count: self.player_count,
            score: self.active_score(),
            player_scores: self.player_scores(),
            credits: prompt_credits,
            lives: self.active_stock().lives,
            smart_bombs: self.active_stock().smart_bombs,
            smart_bomb_pending: self.pending_smart_bomb_detonation_steps.is_some(),
            player_stocks: self.player_stocks(),
            game_over_hall_of_fame_stall_remaining: self.game_over_hall_of_fame_stall_remaining,
            player_switch: prompt_player_switch,
            player_start: prompt_player_start,
            high_scores: self.high_scores.entries(),
            high_score_initials: self.high_score_initials,
            snapshots: self.snapshots.values().cloned().collect(),
            behavior_script: behavior_script.clone(),
            source_background_left: self.source_background_left,
            source_rng,
            source_human_walk_target_slot,
            source_shell_scan_tick,
        };

        let mut replies = Vec::new();
        for (id, actor) in &self.actors {
            if let Some(reply) = actor.prompt(base_prompt.clone()) {
                replies.push((*id, reply));
            }
        }
        replies.sort_by_key(|(id, _)| *id);

        let mut draws = Vec::new();
        let mut commands = step_commands;
        let mut dead_actor_ids = Vec::new();
        self.snapshots.clear();
        for (_, reply) in replies {
            if reply.snapshot.alive {
                self.snapshots.insert(reply.id, reply.snapshot);
            } else {
                dead_actor_ids.push(reply.id);
            }
            draws.extend(reply.draws);
            commands.extend(reply.commands);
        }

        self.resolve_collisions(&behavior_script, &mut commands);
        self.advance_baiter_timer(&mut commands);
        let applied_commands = self.apply_commands(&commands[pre_applied_command_count..]);
        self.remove_dead_actors(&dead_actor_ids);
        draws.extend(applied_commands.draws);
        delayed_sounds.extend(applied_commands.sounds);
        survivor_bonus_replay_awarded |= applied_commands.bonus_awarded;
        if self.start_survivor_bonus_if_wave_cleared(was_playing, &commands) {
            commands.push(GameCommand::WaveCleared {
                next_wave: self.wave.saturating_add(1),
            });
            if let SurvivorBonusStep::Award(points) = self.award_initial_survivor_bonus() {
                survivor_bonus_awarded_points = Some(points);
                if self.award_points(points) {
                    survivor_bonus_replay_awarded = true;
                }
            }
        }
        self.schedule_first_wave_lander_refill_if_needed();
        let survivor_bonus = self
            .pending_survivor_bonus
            .map(|bonus| bonus.report(survivor_bonus_awarded_points));
        let player_switch = self.pending_player_switch.map(PendingPlayerSwitch::report);
        let player_start = self.pending_player_start.map(PendingPlayerStart::report);
        if self.phase == Phase::Playing
            && !survivor_bonus_interstitial
            && !player_switch_interstitial
            && !player_start_interstitial
        {
            self.source_reserve_activation_ready = true;
        }

        let report = StepReport {
            step: self.step,
            phase: self.phase,
            wave: self.wave,
            current_player: self.current_player,
            player_count: self.player_count,
            score: self.active_score(),
            player_scores: self.player_scores(),
            credits: self.credits,
            lives: self.active_stock().lives,
            smart_bombs: self.active_stock().smart_bombs,
            smart_bomb_flash_steps_remaining: self.smart_bomb_flash_steps_remaining,
            player_stocks: self.player_stocks(),
            next_bonus: self.next_bonus,
            game_over_hall_of_fame_stall_remaining: self.game_over_hall_of_fame_stall_remaining,
            player_switch,
            player_start,
            high_scores: self.high_scores.entries(),
            source_wave: self.current_source_wave_profile(),
            high_score_initials: self.high_score_initials,
            high_score_initial_accepted: high_score_entry_step.accepted,
            high_score_submitted: high_score_entry_step.submitted,
            bonus_awarded: survivor_bonus_replay_awarded,
            survivor_bonus,
            behavior_script: behavior_script.manifest(),
            enemy_reserve: self.enemy_reserve,
            source_background_left: self.source_background_left,
            source_rng,
            terrain_blow: self.terrain_blow,
            snapshots: self.snapshots.values().cloned().collect(),
            draws,
            sounds: delayed_sounds,
            commands,
        };
        self.advance_game_over_hall_of_fame_return();
        report
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn wave(&self) -> u16 {
        self.wave
    }

    pub fn actor_count(&self) -> usize {
        self.actors.len()
    }

    pub fn wave_script_name(&self) -> &str {
        self.wave_script.name()
    }

    pub fn behavior_script(&self) -> &ActorBehaviorScript {
        &self.behavior_script
    }

    pub fn behavior_script_mut(&mut self) -> &mut ActorBehaviorScript {
        &mut self.behavior_script
    }

    pub fn script_manifest(&self) -> ActorDriverScriptManifest {
        let current_wave = self.wave.max(1);
        ActorDriverScriptManifest {
            step: self.step,
            phase: self.phase,
            wave: self.wave,
            attract_script: self.attract_script.manifest(),
            behavior_script: self.behavior_script.manifest(),
            wave_script: self.wave_script.manifest(),
            current_wave_profile: self.wave_script.profile_for_wave(current_wave).manifest(),
        }
    }

    pub fn set_default_behavior(&mut self, profile: ActorBehaviorProfile) {
        self.behavior_script.set_default_profile(profile);
    }

    pub fn set_kind_behavior(&mut self, kind: ActorKind, profile: ActorBehaviorProfile) {
        self.behavior_script.set_kind_behavior(kind, profile);
    }

    pub fn set_actor_behavior(&mut self, actor: ActorId, profile: ActorBehaviorProfile) {
        self.behavior_script.set_actor_behavior(actor, profile);
    }

    pub fn snapshot_count(&self, kind: ActorKind) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == kind)
            .count()
    }

    pub fn spawn_lander_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_lander(position)
    }

    pub fn spawn_mutant_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_mutant(position)
    }

    pub fn spawn_bomber_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_bomber(position)
    }

    pub fn spawn_bomb_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_bomb(position, None)
    }

    pub fn spawn_pod_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_pod(position)
    }

    pub fn spawn_swarmer_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_swarmer(position)
    }

    pub fn spawn_baiter_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_baiter(position)
    }

    pub fn set_baiter_timer_for_test(&mut self, timer_steps: u32) {
        self.baiter_timer_steps = Some(timer_steps.max(1));
        self.baiter_pacing_steps_remaining = 1;
    }

    pub fn spawn_human_for_test(&mut self, position: Point) -> ActorId {
        self.spawn_human(position, HumanMode::Grounded)
    }

    pub fn spawn_falling_human_for_test(&mut self, position: Point, velocity: i16) -> ActorId {
        self.spawn_human(position, HumanMode::Falling { velocity })
    }

    pub fn spawn_carried_human_for_test(&mut self, position: Point, carrier: ActorId) -> ActorId {
        self.spawn_human(position, HumanMode::CarriedBy(carrier))
    }

    fn allocate_actor_id(&mut self) -> ActorId {
        let id = ActorId::new(self.next_actor_id);
        self.next_actor_id = self.next_actor_id.saturating_add(1);
        id
    }

    fn spawn_actor(&mut self, actor: impl AssetActor) {
        let id = actor.id();
        self.actors.insert(id, ThreadedAsset::spawn(actor));
    }

    fn spawn_player(&mut self) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(PlayerShip::new(id, Point::new(42, 120)));
        id
    }

    fn spawn_lander(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Lander::new(id, position));
        id
    }

    fn spawn_lander_from_spawn(&mut self, spawn: ActorLanderSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Lander::from_spawn(id, spawn));
        id
    }

    fn spawn_mutant(&mut self, position: Point) -> ActorId {
        self.spawn_mutant_from_spawn(ActorMutantSpawn::new(position))
    }

    fn spawn_mutant_from_spawn(&mut self, spawn: ActorMutantSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Mutant::from_spawn(id, spawn));
        id
    }

    fn spawn_bomber(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Bomber::new(id, position));
        id
    }

    fn spawn_bomber_from_spawn(&mut self, spawn: ActorBomberSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Bomber::from_spawn(id, spawn));
        id
    }

    fn spawn_bomb(
        &mut self,
        position: Point,
        source: Option<ActorSourceEnemyProjectileMetadata>,
    ) -> ActorId {
        let id = self.allocate_actor_id();
        let lifetime = self
            .behavior_script
            .behavior_for(id, ActorKind::Bomb)
            .bomb_lifetime_steps;
        self.spawn_actor(Bomb::new(id, position, lifetime, source));
        id
    }

    fn spawn_pod(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Pod::new(id, position));
        id
    }

    fn spawn_pod_from_spawn(&mut self, spawn: ActorPodSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Pod::from_spawn(id, spawn));
        id
    }

    fn spawn_swarmer(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Swarmer::new(id, position));
        id
    }

    fn spawn_swarmer_from_spawn(&mut self, spawn: ActorSwarmerSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Swarmer::from_spawn(id, spawn));
        id
    }

    fn spawn_baiter(&mut self, position: Point) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Baiter::new(id, position));
        id
    }

    fn spawn_baiter_from_spawn(&mut self, spawn: ActorBaiterSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Baiter::from_spawn(id, spawn));
        id
    }

    fn spawn_human(&mut self, position: Point, mode: HumanMode) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Human::new(id, position, mode));
        id
    }

    fn spawn_human_from_spawn(&mut self, spawn: ActorHumanSpawn) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Human::from_spawn(id, spawn));
        id
    }

    fn spawn_laser(&mut self, position: Point, direction: Direction, owner: ActorId) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(LaserShot::new(id, position, direction, owner));
        id
    }

    fn spawn_enemy_laser_from_spawn(
        &mut self,
        position: Point,
        velocity: Velocity,
        source: Option<ActorSourceEnemyProjectileMetadata>,
    ) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(EnemyLaserShot::new(id, position, velocity, source));
        id
    }

    fn spawn_explosion(&mut self, position: Point, kind: ExplosionKind) -> ActorId {
        self.spawn_explosion_with_source_center(position, kind, None)
    }

    fn spawn_explosion_with_source_center(
        &mut self,
        position: Point,
        kind: ExplosionKind,
        source_center: Option<Point>,
    ) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(Explosion::new(id, position, kind, source_center));
        id
    }

    fn spawn_score_popup(&mut self, position: Point, points: u32) -> ActorId {
        let id = self.allocate_actor_id();
        self.spawn_actor(ScorePopup::new(id, position, points));
        id
    }

    fn resolve_collisions(
        &mut self,
        behavior_script: &ActorBehaviorScript,
        commands: &mut Vec<GameCommand>,
    ) {
        if self.phase != Phase::Playing {
            return;
        }

        let bodies = self
            .snapshots
            .values()
            .filter_map(ActorSnapshot::collision_body)
            .collect::<Vec<_>>();
        let mut destroyed = BTreeSet::new();
        for laser in bodies.iter().filter(|body| body.kind == ActorKind::Laser) {
            for enemy in bodies
                .iter()
                .filter(|body| is_player_laser_target(body.kind))
            {
                if laser.bounds.intersects(enemy.bounds) {
                    destroyed.insert(laser.owner);
                    destroyed.insert(enemy.owner);
                    commands.push(GameCommand::Destroy(laser.owner));
                    commands.push(GameCommand::Destroy(enemy.owner));
                    if let Some(kind) = explosion_kind_for_target(enemy.kind) {
                        commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                            position: center_of(enemy.bounds),
                            kind,
                            source_center: None,
                        }));
                    }
                    commands.push(GameCommand::AddScore(score_for_hostile(enemy.kind)));
                    commands.push(GameCommand::PlaySound(hit_sound_for_hostile(enemy.kind)));
                    if enemy.kind == ActorKind::Pod {
                        commands.extend(self.pod_swarmer_spawn_commands(center_of(enemy.bounds)));
                    }
                    break;
                }
            }
        }

        let Some(player) = bodies.iter().find(|body| body.kind == ActorKind::Player) else {
            return;
        };
        let player_behavior = behavior_script.behavior_for(player.owner, ActorKind::Player);
        if !player_behavior.player_takes_enemy_collision_damage {
            return;
        }
        let hyperspace_clears_enemy_lasers = commands
            .iter()
            .any(|command| matches!(command, GameCommand::Hyperspace));
        for enemy in bodies.iter().filter(|body| is_player_hazard(body.kind)) {
            if destroyed.contains(&enemy.owner) {
                continue;
            }
            if hyperspace_clears_enemy_lasers && enemy.kind == ActorKind::EnemyLaser {
                continue;
            }
            if actor_source_target6_mutant_waits_for_fire2524_collision(
                enemy.position,
                enemy.source_mutant,
            ) {
                continue;
            }
            if player.bounds.intersects(enemy.bounds) {
                commands.push(GameCommand::Destroy(player.owner));
                commands.push(GameCommand::Destroy(enemy.owner));
                if is_player_enemy_collision_target(enemy.kind) {
                    if let Some(kind) = explosion_kind_for_target(enemy.kind) {
                        let placement = actor_player_enemy_collision_explosion_placement(enemy);
                        commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                            position: placement.position,
                            kind,
                            source_center: placement.source_center,
                        }));
                    }
                    commands.push(GameCommand::AddScore(score_for_hostile(enemy.kind)));
                    commands.push(GameCommand::PlaySound(hit_sound_for_hostile(enemy.kind)));
                }
                commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                    position: center_of(player.bounds),
                    kind: player_hazard_explosion_kind(enemy.kind),
                    source_center: None,
                }));
                commands.push(GameCommand::PlaySound(player_hazard_sound(enemy.kind)));
                commands.push(GameCommand::PlayerKilled);
                break;
            }
        }
    }

    fn apply_commands(&mut self, commands: &[GameCommand]) -> AppliedCommands {
        let mut applied = AppliedCommands::default();
        let mut active_source_shells = self.active_source_shell_count();
        let mut active_bomb_shells = self.active_bomb_shell_count();
        let mut removed_source_shells = BTreeSet::new();
        let mut removed_bomb_shells = BTreeSet::new();
        let mut terrain_blow_started_this_batch = false;
        for command in commands {
            match *command {
                GameCommand::Credit => {
                    self.credits = self.credits.saturating_add(1);
                    applied.sounds.push(SoundCue::Credit);
                }
                GameCommand::StartOnePlayer => {
                    if self.phase == Phase::Attract && self.credits > 0 {
                        self.credits = self.credits.saturating_sub(1);
                        self.start_play(1);
                        self.pending_start_sound_steps = Some(SOURCE_START_SOUND_DELAY_STEPS);
                    }
                }
                GameCommand::StartTwoPlayer => {
                    if self.phase == Phase::Attract && self.credits > 1 {
                        self.credits = self.credits.saturating_sub(2);
                        self.start_play(2);
                        self.pending_start_sound_steps = Some(SOURCE_START_SOUND_DELAY_STEPS);
                    }
                }
                GameCommand::Spawn(SpawnRequest::Laser {
                    position,
                    direction,
                    owner,
                }) => {
                    self.spawn_laser(position, direction, owner);
                }
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position,
                    velocity,
                    source,
                }) => {
                    if source_shell_spawn_in_bounds(position)
                        && reserve_source_shell_slot(&mut active_source_shells)
                    {
                        self.spawn_enemy_laser_from_spawn(position, velocity, source);
                    }
                }
                GameCommand::Spawn(SpawnRequest::Lander { position }) => {
                    let actor = self.spawn_lander(position);
                    self.apply_next_wave_spawn_behavior(ActorKind::Lander, actor);
                }
                GameCommand::Spawn(SpawnRequest::Mutant { position, source }) => {
                    let actor = self.spawn_mutant_from_spawn(ActorMutantSpawn { position, source });
                    self.apply_next_wave_spawn_behavior(ActorKind::Mutant, actor);
                }
                GameCommand::Spawn(SpawnRequest::Bomber { position }) => {
                    let actor = self.spawn_bomber(position);
                    self.apply_next_wave_spawn_behavior(ActorKind::Bomber, actor);
                }
                GameCommand::Spawn(SpawnRequest::Bomb { position, source }) => {
                    if bomb_shell_spawn_in_source_bounds(position, source)
                        && source_shell_slot_available(active_source_shells)
                        && bomb_shell_slot_available(active_bomb_shells)
                    {
                        active_source_shells += 1;
                        active_bomb_shells += 1;
                        self.spawn_bomb(position, source);
                    }
                }
                GameCommand::Spawn(SpawnRequest::Pod { position }) => {
                    let actor = self.spawn_pod(position);
                    self.apply_next_wave_spawn_behavior(ActorKind::Pod, actor);
                }
                GameCommand::Spawn(SpawnRequest::Swarmer { position, source }) => {
                    let actor =
                        self.spawn_swarmer_from_spawn(ActorSwarmerSpawn { position, source });
                    self.apply_next_wave_spawn_behavior(ActorKind::Swarmer, actor);
                }
                GameCommand::Spawn(SpawnRequest::Baiter { position, source }) => {
                    let actor = self.spawn_baiter_from_spawn(ActorBaiterSpawn { position, source });
                    self.apply_next_wave_spawn_behavior(ActorKind::Baiter, actor);
                }
                GameCommand::Spawn(SpawnRequest::Human { position, mode }) => {
                    let actor = self.spawn_human(position, mode);
                    self.apply_next_wave_spawn_behavior(ActorKind::Human, actor);
                }
                GameCommand::Spawn(SpawnRequest::Explosion {
                    position,
                    kind,
                    source_center,
                }) => {
                    self.spawn_explosion_with_source_center(position, kind, source_center);
                }
                GameCommand::Spawn(SpawnRequest::ScorePopup { position, points }) => {
                    self.spawn_score_popup(position, points);
                }
                GameCommand::Destroy(id) => {
                    let removed_kind = self.snapshots.get(&id).map(|snapshot| snapshot.kind);
                    if let Some(snapshot) = self.snapshots.get(&id) {
                        if removed_source_shells.insert(id) && is_source_shell_kind(snapshot.kind) {
                            active_source_shells = active_source_shells.saturating_sub(1);
                        }
                        if removed_bomb_shells.insert(id) && snapshot.kind == ActorKind::Bomb {
                            active_bomb_shells = active_bomb_shells.saturating_sub(1);
                        }
                    }
                    self.snapshots.remove(&id);
                    self.actors.remove(&id);
                    self.behavior_script.remove_actor_behavior(id);
                    if removed_kind == Some(ActorKind::Human) {
                        let draws = self.start_terrain_blow_if_no_humans();
                        terrain_blow_started_this_batch |= !draws.is_empty();
                        applied.draws.extend(draws);
                    }
                }
                GameCommand::SetSourceBackgroundLeft(background_left) => {
                    self.source_background_left = background_left;
                }
                GameCommand::AttachHuman {
                    lander,
                    human,
                    position,
                } => {
                    let source = self
                        .snapshots
                        .get(&human)
                        .and_then(|snapshot| snapshot.source_human);
                    self.snapshots.remove(&human);
                    self.actors.remove(&human);
                    self.spawn_actor(Human::with_source(
                        human,
                        position,
                        HumanMode::CarriedBy(lander),
                        source,
                    ));
                }
                GameCommand::SmartBomb { consume_stock } => {
                    self.start_smart_bomb(consume_stock);
                }
                GameCommand::Hyperspace => {
                    self.clear_enemy_projectiles_for_hyperspace();
                }
                GameCommand::HumanLost(id) => {
                    self.snapshots.remove(&id);
                    self.actors.remove(&id);
                    self.behavior_script.remove_actor_behavior(id);
                    let draws = self.start_terrain_blow_if_no_humans();
                    terrain_blow_started_this_batch |= !draws.is_empty();
                    applied.draws.extend(draws);
                }
                GameCommand::AddScore(points) => {
                    if self.award_points(points) {
                        applied.bonus_awarded = true;
                    }
                }
                GameCommand::PlaySound(sound) => {
                    if !(terrain_blow_started_this_batch && sound == SoundCue::HumanLost) {
                        applied.sounds.push(sound);
                        if sound == SoundCue::HumanRescued {
                            self.queue_astronaut_rescue_sound_tail();
                        }
                    }
                }
                GameCommand::PlayerKilled => {
                    self.lose_player_life(&mut applied.sounds);
                }
                GameCommand::WaveCleared { .. } => {}
                GameCommand::AdvanceWave { .. } => {}
                GameCommand::EnterGameOver => {
                    self.enter_game_over(&mut applied.sounds);
                }
            }
        }
        applied
    }

    fn active_source_shell_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| is_source_shell_kind(snapshot.kind) && snapshot.alive)
            .count()
    }

    fn active_bomb_shell_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomb && snapshot.alive)
            .count()
    }

    fn active_score(&self) -> u32 {
        if self.current_player == 2 {
            self.player_two_score
        } else {
            self.score
        }
    }

    fn player_scores(&self) -> [u32; 2] {
        [self.score, self.player_two_score]
    }

    fn active_stock(&self) -> PlayerStock {
        self.stock_for_player(self.current_player)
    }

    fn stock_for_player(&self, player: u8) -> PlayerStock {
        if player == 2 {
            PlayerStock::new(self.player_two_lives, self.player_two_smart_bombs)
        } else {
            PlayerStock::new(self.lives, self.smart_bombs)
        }
    }

    fn set_active_stock(&mut self, stock: PlayerStock) {
        self.set_stock_for_player(self.current_player, stock);
    }

    fn set_stock_for_player(&mut self, player: u8, stock: PlayerStock) {
        if player == 2 {
            self.player_two_lives = stock.lives;
            self.player_two_smart_bombs = stock.smart_bombs;
        } else {
            self.lives = stock.lives;
            self.smart_bombs = stock.smart_bombs;
        }
    }

    fn player_stocks(&self) -> [PlayerStockSnapshot; 2] {
        [
            PlayerStockSnapshot::new(self.lives, self.smart_bombs),
            PlayerStockSnapshot::new(self.player_two_lives, self.player_two_smart_bombs),
        ]
    }

    fn highest_visible_score(&self) -> u32 {
        self.high_scores.entries()[0]
            .max(self.score)
            .max(self.player_two_score)
    }

    fn lose_player_life(&mut self, sounds: &mut Vec<SoundCue>) {
        let from_player = self.current_player;
        let mut stock = self.active_stock();
        stock.lives = stock.lives.saturating_sub(1);
        self.set_active_stock(stock);

        if let Some(to_player) = self.next_other_stocked_player(from_player) {
            self.begin_player_switch(from_player, to_player);
            return;
        }

        if stock.lives > 0 {
            self.spawn_player();
            return;
        }

        self.enter_game_over(sounds);
    }

    fn next_other_stocked_player(&self, from_player: u8) -> Option<u8> {
        let player_count = self.player_count.clamp(1, 2);
        for offset in 1..=player_count {
            let candidate = ((from_player.saturating_sub(1) + offset) % player_count) + 1;
            if candidate != from_player && self.stock_for_player(candidate).lives > 0 {
                return Some(candidate);
            }
        }
        None
    }

    fn begin_player_switch(&mut self, from_player: u8, to_player: u8) {
        self.phase = Phase::GameOver;
        self.game_over_hall_of_fame_stall_remaining = None;
        self.pending_survivor_bonus = None;
        self.pending_player_switch = Some(PendingPlayerSwitch::new(from_player, to_player));
        self.pending_player_start = None;
        self.pending_start_sound_steps = None;
        self.clear_pending_smart_bomb();
        self.clear_terrain_blow();
        self.clear_pending_astronaut_rescue();
        self.clear_first_wave_lander_refill();
        self.enemy_reserve = EnemyReserveSnapshot::default();
        self.source_target_cursor = None;
        self.source_reserve_activation_ready = false;
        self.source_reserve_activation_cooldown_steps = 0;
        self.source_first_wave_early_reserve_steps_remaining = None;
        self.source_background_left = 0;
        self.baiter_timer_steps = None;
        self.clear_turn_playfield_actors();
    }

    fn advance_pending_player_switch(&mut self) {
        let Some(mut pending) = self.pending_player_switch else {
            return;
        };

        pending.sleep_steps_remaining = pending.sleep_steps_remaining.saturating_sub(1);
        if pending.sleep_steps_remaining > 0 {
            self.pending_player_switch = Some(pending);
            return;
        }

        self.pending_player_switch = None;
        self.start_next_player_turn(pending.to_player);
    }

    fn start_next_player_turn(&mut self, player: u8) {
        let player = player.clamp(1, self.player_count.clamp(1, 2));
        self.phase = Phase::Playing;
        self.current_player = player;
        self.wave = 1;
        self.high_score_initials = HighScoreInitialsState::EMPTY;
        self.game_over_hall_of_fame_stall_remaining = None;
        self.pending_survivor_bonus = None;
        self.pending_player_switch = None;
        self.pending_player_start = Some(PendingPlayerStart::new(player));
        self.pending_start_sound_steps = None;
        self.clear_pending_smart_bomb();
        self.clear_terrain_blow();
        self.clear_pending_astronaut_rescue();
        self.clear_first_wave_lander_refill();
        self.source_rng = SOURCE_PLAYFIELD_START_RNG;
        self.source_background_left = 0;
        self.source_reserve_activation_cooldown_steps = 0;
        self.source_first_wave_early_reserve_steps_remaining = None;
        self.reset_source_shell_scan();
        self.clear_turn_playfield_actors();
        self.apply_wave_profile();
    }

    fn advance_pending_start_sound(&mut self) -> Vec<SoundCue> {
        let Some(mut remaining) = self.pending_start_sound_steps else {
            return Vec::new();
        };

        remaining = remaining.saturating_sub(1);
        if remaining > 0 {
            self.pending_start_sound_steps = Some(remaining);
            Vec::new()
        } else {
            self.pending_start_sound_steps = None;
            vec![SoundCue::Start]
        }
    }

    fn advance_pending_sound_commands(&mut self) -> Vec<SoundCue> {
        let mut sounds = Vec::new();
        let mut pending = Vec::new();
        for mut command in self.pending_sound_commands.drain(..) {
            command.steps_remaining = command.steps_remaining.saturating_sub(1);
            if command.steps_remaining == 0 {
                sounds.push(SoundCue::SourceCommand(command.command));
            } else {
                pending.push(command);
            }
        }
        self.pending_sound_commands = pending;
        sounds
    }

    fn queue_smart_bomb_sound_sequence(&mut self) {
        self.pending_sound_commands
            .extend(SOURCE_SMART_BOMB_SOUND_SEQUENCE.iter().copied().map(
                |(steps_remaining, command)| PendingActorSoundCommand {
                    steps_remaining,
                    command,
                    source: PendingActorSoundSource::SmartBomb,
                },
            ));
    }

    fn queue_terrain_blow_sound_tail(&mut self) {
        self.pending_sound_commands.extend(
            SOURCE_TERRAIN_BLOW_SOUND_TAIL_SEQUENCE.iter().copied().map(
                |(steps_remaining, command)| PendingActorSoundCommand {
                    steps_remaining,
                    command,
                    source: PendingActorSoundSource::TerrainBlow,
                },
            ),
        );
    }

    fn queue_astronaut_rescue_sound_tail(&mut self) {
        self.pending_sound_commands
            .extend(SOURCE_ACSND_SOUND_TAIL_SEQUENCE.iter().copied().map(
                |(steps_remaining, command)| PendingActorSoundCommand {
                    steps_remaining,
                    command,
                    source: PendingActorSoundSource::AstronautRescue,
                },
            ));
    }

    fn queue_first_wave_lander_refill_appearance_sound(&mut self) {
        self.pending_sound_commands.push(PendingActorSoundCommand {
            steps_remaining: SOURCE_FIRST_WAVE_LANDER_REFILL_APPEAR_SOUND_DELAY_STEPS,
            command: SOURCE_APPEAR_SOUND_COMMAND,
            source: PendingActorSoundSource::FirstWaveLanderRefill,
        });
    }

    fn advance_smart_bomb_flash(&mut self) {
        self.smart_bomb_flash_steps_remaining =
            self.smart_bomb_flash_steps_remaining.saturating_sub(1);
    }

    fn start_terrain_blow_if_no_humans(&mut self) -> Vec<DrawCommand> {
        if self.terrain_blow.is_some()
            || self
                .snapshots
                .values()
                .any(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
        {
            return Vec::new();
        }

        self.terrain_blow = Some(TerrainBlowSnapshot::source_started());
        self.spawn_terrain_blow_explosion_births(0)
    }

    fn advance_terrain_blow(&mut self, commands: &mut Vec<GameCommand>) -> Vec<SoundCue> {
        let Some(terrain_blow) = self.terrain_blow else {
            return Vec::new();
        };
        if terrain_blow.stage == TerrainBlowStage::Completed {
            return Vec::new();
        }

        let elapsed = terrain_blow.source_elapsed_frames.saturating_add(1);
        for draw in self.spawn_terrain_blow_explosion_births(elapsed) {
            commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                position: draw.position,
                kind: ExplosionKind::Terrain,
                source_center: None,
            }));
        }

        let mut sounds = Vec::new();
        if elapsed >= SOURCE_TERRAIN_BLOW_COMPLETE_FRAME {
            if let Some(terrain_blow) = self.terrain_blow.as_mut() {
                terrain_blow.stage = TerrainBlowStage::Completed;
                terrain_blow.source_elapsed_frames = elapsed;
                terrain_blow.source_iteration = terrain_blow.source_iteration_limit;
                terrain_blow.source_sleep_remaining = None;
                terrain_blow.source_pseudo_color = 0;
            }
            sounds.push(SoundCue::SourceCommand(SOURCE_TBSND_SOUND_COMMAND));
            self.queue_terrain_blow_sound_tail();
            return sounds;
        }

        let start_sound_index = SOURCE_TERRAIN_BLOW_START_SOUND_FRAMES
            .iter()
            .position(|frame| *frame == elapsed);
        let next_frame = SOURCE_TERRAIN_BLOW_START_SOUND_FRAMES
            .iter()
            .copied()
            .find(|frame| *frame > elapsed)
            .unwrap_or(SOURCE_TERRAIN_BLOW_COMPLETE_FRAME);
        if let Some(terrain_blow) = self.terrain_blow.as_mut() {
            terrain_blow.source_elapsed_frames = elapsed;
            terrain_blow.source_sleep_remaining = u8::try_from(next_frame - elapsed).ok();
            terrain_blow.source_overload_counter = SOURCE_TERRAIN_BLOW_OVERLOAD_COUNTER;
            if let Some(start_sound_index) = start_sound_index {
                terrain_blow.stage = TerrainBlowStage::ExplosionPassSleeping;
                terrain_blow.source_iteration = terrain_blow.source_iteration.saturating_add(1);
                terrain_blow.source_pseudo_color =
                    SOURCE_TERRAIN_BLOW_FLASH_COLOR_BYTES[start_sound_index];
            } else {
                terrain_blow.stage = TerrainBlowStage::FlashClearedSleeping;
                terrain_blow.source_pseudo_color = 0;
            }
        }
        if start_sound_index.is_some() {
            sounds.push(SoundCue::SourceCommand(SOURCE_SBSND_SOUND_COMMAND));
        }
        sounds
    }

    fn spawn_terrain_blow_explosion_births(&mut self, elapsed: u16) -> Vec<DrawCommand> {
        const TERRAIN_BLOW_EXPLOSION_BIRTHS: [(u16, Point); 17] = [
            (0, Point::new(0x4C, 0xC2)),
            (4, Point::new(0x14, 0xE2)),
            (4, Point::new(0x5C, 0xDE)),
            (8, Point::new(0x80, 0xDE)),
            (12, Point::new(0x00, 0xE0)),
            (16, Point::new(0x68, 0xDC)),
            (21, Point::new(0x30, 0xE0)),
            (26, Point::new(0x80, 0xDE)),
            (31, Point::new(0x44, 0xD2)),
            (31, Point::new(0x50, 0xC6)),
            (51, Point::new(0x20, 0xE2)),
            (52, Point::new(0x70, 0xD8)),
            (60, Point::new(0x6C, 0xD4)),
            (60, Point::new(0x28, 0xE0)),
            (70, Point::new(0x94, 0xDC)),
            (70, Point::new(0x00, 0xE0)),
            (81, Point::new(0x0C, 0xE2)),
        ];

        TERRAIN_BLOW_EXPLOSION_BIRTHS
            .iter()
            .copied()
            .filter(|(birth_frame, _)| *birth_frame == elapsed)
            .map(|(_, position)| {
                let id = self.spawn_explosion(position, ExplosionKind::Terrain);
                DrawCommand::sprite_with_effect(
                    id,
                    SpriteKey::Explosion,
                    position,
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Terrain,
                        age: 0,
                        source_center: None,
                    },
                )
            })
            .collect()
    }

    fn clear_pending_smart_bomb(&mut self) {
        self.pending_smart_bomb_detonation_steps = None;
        self.smart_bomb_flash_steps_remaining = 0;
        self.pending_sound_commands
            .retain(|command| command.source != PendingActorSoundSource::SmartBomb);
    }

    fn clear_terrain_blow(&mut self) {
        self.terrain_blow = None;
        self.pending_sound_commands
            .retain(|command| command.source != PendingActorSoundSource::TerrainBlow);
    }

    fn clear_pending_astronaut_rescue(&mut self) {
        self.pending_sound_commands
            .retain(|command| command.source != PendingActorSoundSource::AstronautRescue);
    }

    fn clear_first_wave_lander_refill(&mut self) {
        self.source_first_wave_lander_refill_steps_remaining = None;
        self.pending_sound_commands
            .retain(|command| command.source != PendingActorSoundSource::FirstWaveLanderRefill);
    }

    fn start_smart_bomb(&mut self, consume_stock: bool) -> bool {
        if self.pending_smart_bomb_detonation_steps.is_some() {
            return false;
        }

        if consume_stock {
            let mut stock = self.active_stock();
            if stock.smart_bombs == 0 {
                return false;
            }
            stock.smart_bombs = stock.smart_bombs.saturating_sub(1);
            self.set_active_stock(stock);
        }

        self.pending_smart_bomb_detonation_steps = Some(SOURCE_SMART_BOMB_DETONATION_DELAY_STEPS);
        self.source_reserve_activation_cooldown_steps = SOURCE_SMART_BOMB_RESERVE_DELAY_STEPS;
        self.source_first_wave_early_reserve_steps_remaining = None;
        self.clear_first_wave_lander_refill();
        self.queue_smart_bomb_sound_sequence();
        true
    }

    fn advance_pending_smart_bomb_detonation(&mut self, commands: &mut Vec<GameCommand>) -> bool {
        let Some(mut remaining) = self.pending_smart_bomb_detonation_steps else {
            return false;
        };

        remaining = remaining.saturating_sub(1);
        if remaining > 0 {
            self.pending_smart_bomb_detonation_steps = Some(remaining);
            return false;
        }

        self.pending_smart_bomb_detonation_steps = None;
        self.smart_bomb_flash_steps_remaining = SOURCE_SMART_BOMB_FLASH_STEPS;
        self.detonate_smart_bomb_targets(commands)
    }

    fn advance_pending_player_start(&mut self) -> PlayerStartStep {
        let Some(mut pending) = self.pending_player_start else {
            return PlayerStartStep::Waiting;
        };

        pending.delay_steps_remaining = pending.delay_steps_remaining.saturating_sub(1);
        if pending.delay_steps_remaining > 0 {
            self.pending_player_start = Some(pending);
            return PlayerStartStep::Waiting;
        }

        self.pending_player_start = None;
        self.start_delayed_playfield();
        PlayerStartStep::StartPlayfield
    }

    fn start_delayed_playfield(&mut self) {
        self.clear_turn_playfield_actors();
        self.apply_wave_profile();
        self.spawn_player();
        self.spawn_wave_hostiles();
        self.spawn_initial_humans();
        self.arm_first_wave_early_lander_reserve_delay();
    }

    fn enter_game_over(&mut self, sounds: &mut Vec<SoundCue>) {
        let active_score = self.active_score();
        self.set_active_stock(PlayerStock::new(0, 0));
        self.wave = 0;
        self.source_rng = SOURCE_PLAYFIELD_START_RNG;
        self.source_background_left = 0;
        self.reset_source_shell_scan();
        self.high_score_initials = HighScoreInitialsState::EMPTY;
        self.game_over_hall_of_fame_stall_remaining = None;
        self.pending_survivor_bonus = None;
        self.pending_player_switch = None;
        self.pending_player_start = None;
        self.pending_start_sound_steps = None;
        self.clear_pending_smart_bomb();
        self.clear_terrain_blow();
        self.clear_pending_astronaut_rescue();
        self.clear_first_wave_lander_refill();
        self.enemy_reserve = EnemyReserveSnapshot::default();
        self.source_target_cursor = None;
        self.source_reserve_activation_ready = false;
        self.source_reserve_activation_cooldown_steps = 0;
        self.source_first_wave_early_reserve_steps_remaining = None;
        self.high_scores.record(active_score);
        self.phase = if self.high_scores.qualifies(active_score) {
            Phase::HighScoreEntry
        } else {
            Phase::GameOver
        };
        self.baiter_timer_steps = None;
        sounds.push(SoundCue::GameOver);
    }

    fn apply_high_score_entry_input(&mut self, input: GameInput) -> HighScoreEntryStep {
        if self.phase != Phase::HighScoreEntry {
            return HighScoreEntryStep::default();
        }

        let frame = HighScoreEntrySystem::enter_initial(
            self.high_score_initials,
            input.high_score_initial,
            input.high_score_backspace,
        );
        self.high_score_initials = frame.state;
        if frame.submitted {
            self.phase = Phase::GameOver;
            self.game_over_hall_of_fame_stall_remaining = Some(SOURCE_HIGH_SCORE_HALL_STALL_STEPS);
        }
        HighScoreEntryStep {
            accepted: frame.accepted,
            submitted: frame.submitted,
        }
    }

    fn start_play(&mut self, player_count: u8) {
        let player_count = player_count.clamp(1, 2);
        self.phase = Phase::Playing;
        self.current_player = 1;
        self.player_count = player_count;
        self.wave = 1;
        self.score = 0;
        self.player_two_score = 0;
        self.lives = INITIAL_PLAYER_LIVES;
        self.smart_bombs = INITIAL_SMART_BOMBS;
        self.player_two_lives = INITIAL_PLAYER_LIVES;
        self.player_two_smart_bombs = INITIAL_SMART_BOMBS;
        self.next_bonus = SOURCE_REPLAY_SCORE;
        self.high_score_initials = HighScoreInitialsState::EMPTY;
        self.game_over_hall_of_fame_stall_remaining = None;
        self.pending_survivor_bonus = None;
        self.pending_player_switch = None;
        self.pending_player_start = None;
        self.pending_start_sound_steps = None;
        self.clear_pending_smart_bomb();
        self.clear_terrain_blow();
        self.clear_pending_astronaut_rescue();
        self.clear_first_wave_lander_refill();
        self.enemy_reserve = EnemyReserveSnapshot::default();
        self.source_target_cursor = None;
        self.source_reserve_activation_ready = false;
        self.source_reserve_activation_cooldown_steps = 0;
        self.source_first_wave_early_reserve_steps_remaining = None;
        self.source_rng = SOURCE_PLAYFIELD_START_RNG;
        self.source_background_left = 0;
        self.reset_source_shell_scan();
        self.clear_turn_playfield_actors();
        self.apply_wave_profile();
        self.pending_player_start = Some(PendingPlayerStart::new(self.current_player));
    }

    fn start_pending_wave(&mut self) {
        self.wave = self.wave.saturating_add(1);
        self.pending_survivor_bonus = None;
        self.pending_player_switch = None;
        self.pending_player_start = None;
        self.pending_start_sound_steps = None;
        self.clear_pending_smart_bomb();
        self.clear_terrain_blow();
        self.clear_pending_astronaut_rescue();
        self.clear_first_wave_lander_refill();
        self.clear_wave_playfield_actors();
        self.apply_wave_profile();
        self.source_reserve_activation_cooldown_steps = 0;
        self.spawn_wave_hostiles();
        self.spawn_initial_humans();
        self.arm_first_wave_early_lander_reserve_delay();
    }

    fn advance_pending_survivor_bonus(&mut self) -> SurvivorBonusStep {
        let Some(mut bonus) = self.pending_survivor_bonus else {
            return SurvivorBonusStep::Waiting;
        };

        if let Some(wave_sleep) = bonus.wave_advance_sleep_steps_remaining {
            let next_sleep = wave_sleep.saturating_sub(1);
            if next_sleep == 0 {
                self.pending_survivor_bonus = None;
                return SurvivorBonusStep::StartNextWave;
            }
            bonus.wave_advance_sleep_steps_remaining = Some(next_sleep);
            self.pending_survivor_bonus = Some(bonus);
            return SurvivorBonusStep::Waiting;
        }

        if bonus.astronaut_sleep_steps_remaining > 0 {
            bonus.astronaut_sleep_steps_remaining =
                bonus.astronaut_sleep_steps_remaining.saturating_sub(1);
            if bonus.astronaut_sleep_steps_remaining > 0 {
                self.pending_survivor_bonus = Some(bonus);
                return SurvivorBonusStep::Waiting;
            }
        }

        if let Some(points) = bonus.award_next_survivor() {
            self.pending_survivor_bonus = Some(bonus);
            return SurvivorBonusStep::Award(points);
        }

        bonus.wave_advance_sleep_steps_remaining =
            Some(SOURCE_SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS);
        self.pending_survivor_bonus = Some(bonus);
        SurvivorBonusStep::Waiting
    }

    fn award_initial_survivor_bonus(&mut self) -> SurvivorBonusStep {
        let Some(mut bonus) = self.pending_survivor_bonus else {
            return SurvivorBonusStep::Waiting;
        };

        if let Some(points) = bonus.award_next_survivor() {
            self.pending_survivor_bonus = Some(bonus);
            return SurvivorBonusStep::Award(points);
        }

        bonus.wave_advance_sleep_steps_remaining =
            Some(SOURCE_SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS);
        self.pending_survivor_bonus = Some(bonus);
        SurvivorBonusStep::Waiting
    }

    fn reset_source_shell_scan(&mut self) {
        self.source_shell_scan_steps_remaining = SOURCE_SHELL_SCAN_INITIAL_DELAY_STEPS;
    }

    fn reset_source_astronaut_process(&mut self) {
        self.source_astronaut_cursor = Some(0);
        self.source_astronaut_sleep_ticks = 0;
    }

    fn advance_source_astronaut_process(
        &mut self,
        source_rng: Option<ActorSourceRngSnapshot>,
    ) -> Option<usize> {
        if source_rng.is_none() || !self.has_source_human_snapshots() {
            return None;
        }
        if self.source_astronaut_sleep_ticks > 0 {
            self.source_astronaut_sleep_ticks = self.source_astronaut_sleep_ticks.saturating_sub(1);
            return None;
        }

        let current_cursor = self
            .source_astronaut_cursor
            .filter(|slot| *slot < SOURCE_ASTRONAUT_TARGET_CURSOR_ENTRY_COUNT)
            .unwrap_or(0);
        let next_cursor = actor_source_astronaut_next_slot_index(current_cursor);
        self.source_astronaut_cursor = Some(next_cursor);
        self.source_astronaut_sleep_ticks = SOURCE_ASTRONAUT_PROCESS_SLEEP_TICKS;

        let human_count = self.human_snapshot_count();
        self.snapshots.values().find_map(|snapshot| {
            let source = snapshot.source_human?;
            (source.target_slot_index == next_cursor
                && actor_source_astronaut_walk_targetable(human_count, snapshot))
            .then_some(next_cursor)
        })
    }

    fn advance_source_reserve_activation_cooldown(&mut self) {
        self.source_reserve_activation_cooldown_steps = self
            .source_reserve_activation_cooldown_steps
            .saturating_sub(1);
    }

    fn advance_game_over_hall_of_fame_return(&mut self) {
        if self.phase != Phase::GameOver {
            return;
        }
        let Some(remaining) = self.game_over_hall_of_fame_stall_remaining else {
            return;
        };

        let next = remaining.saturating_sub(1);
        if next > 0 {
            self.game_over_hall_of_fame_stall_remaining = Some(next);
            return;
        }

        self.game_over_hall_of_fame_stall_remaining = None;
        self.phase = Phase::Attract;
    }

    fn advance_source_shell_scan_tick(&mut self) -> bool {
        if self.source_shell_scan_steps_remaining > 0 {
            self.source_shell_scan_steps_remaining =
                self.source_shell_scan_steps_remaining.saturating_sub(1);
            return false;
        }

        self.source_shell_scan_steps_remaining = SOURCE_SHELL_SCAN_CADENCE_STEPS - 1;
        true
    }

    fn apply_wave_profile(&mut self) {
        let wave_profile = self.wave_script.profile_for_wave(self.wave);
        self.behavior_script = wave_profile.behavior_script.clone();
        self.wave_spawn_allocations.clear();
        self.enemy_reserve = wave_profile.enemy_reserve;
        self.source_target_cursor = Some(0);
        self.reset_source_astronaut_process();
        self.source_reserve_activation_ready = false;
        self.source_reserve_activation_cooldown_steps = 0;
        self.source_first_wave_early_reserve_steps_remaining = None;
        self.clear_first_wave_lander_refill();
        if self.phase == Phase::Playing {
            self.reset_baiter_timer();
        }
    }

    fn current_source_wave_profile(&self) -> ActorSourceWaveProfile {
        self.wave_script
            .profile_for_wave(self.wave)
            .source_wave
            .unwrap_or_else(|| ActorSourceWaveProfile::for_wave(self.wave.max(1)))
    }

    fn reset_baiter_timer(&mut self) {
        let source_profile = self.current_source_wave_profile();
        self.baiter_timer_steps = Some(source_profile.baiter_delay.max(1));
        self.baiter_pacing_steps_remaining = ACTOR_BAITER_TIMER_PACING_STEPS;
    }

    fn activate_enemy_reserve_if_ready(
        &mut self,
        commands: &mut Vec<GameCommand>,
    ) -> Vec<SoundCue> {
        if self.phase != Phase::Playing
            || self.wave == 0
            || self.pending_survivor_bonus.is_some()
            || self.pending_player_switch.is_some()
            || self.pending_player_start.is_some()
            || !self.source_reserve_activation_ready
            || self.source_reserve_activation_cooldown_steps > 0
            || actor_enemy_reserve_is_empty(self.enemy_reserve)
        {
            return Vec::new();
        }

        if self.has_hostile_snapshots() {
            if self.activate_first_wave_early_lander_reserve_if_ready(commands) {
                return vec![SoundCue::HyperspaceMaterialize];
            }
            return Vec::new();
        }

        self.source_first_wave_early_reserve_steps_remaining = None;
        self.clear_first_wave_lander_refill();
        let source_profile = self.current_source_wave_profile();
        let reserve_kinds =
            actor_source_reserve_enemy_kinds(&mut self.enemy_reserve, source_profile);
        let mut index = 0;
        while index < reserve_kinds.len() {
            match reserve_kinds[index] {
                ActorSourceEnemyKind::Lander => {
                    let target_index = self.select_next_source_lander_target_index();
                    if let Some(target_index) = target_index {
                        let spawn = ActorLanderSpawn::source_restore(
                            &mut self.source_rng,
                            source_profile,
                            Some(target_index),
                        );
                        commands.push(GameCommand::Spawn(SpawnRequest::Lander {
                            position: spawn.position,
                        }));
                        let actor = self.spawn_lander_from_spawn(spawn);
                        self.apply_next_wave_spawn_behavior(ActorKind::Lander, actor);
                        index += 1;
                    } else {
                        let lander_count = reserve_kinds[index..]
                            .iter()
                            .take_while(|&&kind| kind == ActorSourceEnemyKind::Lander)
                            .count();
                        for _ in 0..lander_count {
                            let spawn = ActorMutantSpawn::source_restore(
                                &mut self.source_rng,
                                source_profile,
                                self.source_background_left,
                            );
                            commands.push(GameCommand::Spawn(SpawnRequest::Mutant {
                                position: spawn.position,
                                source: spawn.source,
                            }));
                            let actor = self.spawn_mutant_from_spawn(spawn);
                            self.apply_next_wave_spawn_behavior(ActorKind::Mutant, actor);
                        }
                        index += lander_count;
                    }
                }
                ActorSourceEnemyKind::Bomber => {
                    let bomber_count = reserve_kinds[index..]
                        .iter()
                        .take_while(|&&kind| kind == ActorSourceEnemyKind::Bomber)
                        .count();
                    let player_absolute_x = self
                        .active_player_position()
                        .map_or(0, |position| actor_source_absolute_x(position, 0));
                    for spawn in ActorBomberSpawn::source_restore_batch(
                        source_profile,
                        player_absolute_x,
                        bomber_count,
                    ) {
                        commands.push(GameCommand::Spawn(SpawnRequest::Bomber {
                            position: spawn.position,
                        }));
                        let actor = self.spawn_bomber_from_spawn(spawn);
                        self.apply_next_wave_spawn_behavior(ActorKind::Bomber, actor);
                    }
                    index += bomber_count;
                }
                ActorSourceEnemyKind::Pod => {
                    let spawn = ActorPodSpawn::source_restore(&mut self.source_rng);
                    commands.push(GameCommand::Spawn(SpawnRequest::Pod {
                        position: spawn.position,
                    }));
                    let actor = self.spawn_pod_from_spawn(spawn);
                    self.apply_next_wave_spawn_behavior(ActorKind::Pod, actor);
                    index += 1;
                }
                ActorSourceEnemyKind::Mutant => {
                    let spawn = ActorMutantSpawn::source_restore(
                        &mut self.source_rng,
                        source_profile,
                        self.source_background_left,
                    );
                    commands.push(GameCommand::Spawn(SpawnRequest::Mutant {
                        position: spawn.position,
                        source: spawn.source,
                    }));
                    let actor = self.spawn_mutant_from_spawn(spawn);
                    self.apply_next_wave_spawn_behavior(ActorKind::Mutant, actor);
                    index += 1;
                }
                ActorSourceEnemyKind::Swarmer => {
                    let swarmer_count = reserve_kinds[index..]
                        .iter()
                        .take_while(|&&kind| kind == ActorSourceEnemyKind::Swarmer)
                        .count();
                    for spawn in ActorSwarmerSpawn::source_restore_batch(
                        &mut self.source_rng,
                        source_profile,
                        swarmer_count,
                    ) {
                        commands.push(GameCommand::Spawn(SpawnRequest::Swarmer {
                            position: spawn.position,
                            source: spawn.source,
                        }));
                        let actor = self.spawn_swarmer_from_spawn(spawn);
                        self.apply_next_wave_spawn_behavior(ActorKind::Swarmer, actor);
                    }
                    index += swarmer_count;
                }
            }
        }
        Vec::new()
    }

    fn advance_first_wave_lander_refill_if_ready(&mut self, commands: &mut Vec<GameCommand>) {
        let Some(remaining) = self.source_first_wave_lander_refill_steps_remaining else {
            return;
        };
        if self.phase != Phase::Playing
            || self.wave != 1
            || self.pending_survivor_bonus.is_some()
            || self.pending_player_switch.is_some()
            || self.pending_player_start.is_some()
        {
            self.clear_first_wave_lander_refill();
            return;
        }

        let remaining = remaining.saturating_sub(1);
        if remaining > 0 {
            self.source_first_wave_lander_refill_steps_remaining = Some(remaining);
            return;
        }

        self.source_first_wave_lander_refill_steps_remaining = None;
        self.activate_first_wave_lander_refill(commands);
    }

    fn arm_first_wave_early_lander_reserve_delay(&mut self) {
        self.source_first_wave_early_reserve_steps_remaining = (self.wave == 1
            && self.enemy_reserve.landers > 0)
            .then_some(SOURCE_FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS);
    }

    fn schedule_first_wave_lander_refill_if_needed(&mut self) {
        if self
            .source_first_wave_lander_refill_steps_remaining
            .is_some()
            || self.phase != Phase::Playing
            || self.wave != 1
            || self.pending_survivor_bonus.is_some()
            || self.pending_player_switch.is_some()
            || self.pending_player_start.is_some()
            || self.pending_smart_bomb_detonation_steps.is_some()
            || self.source_reserve_activation_cooldown_steps > 0
        {
            return;
        }

        let active_landers = self.source_counted_lander_snapshot_count();
        if active_landers == 0
            || active_landers >= SOURCE_FIRST_WAVE_LANDER_REFILL_ACTIVE_THRESHOLD
            || self.enemy_reserve.landers == 0
            || usize::from(self.enemy_reserve.landers) > SOURCE_MAX_ACTIVE_WAVE_ENEMIES
        {
            return;
        }

        self.source_first_wave_lander_refill_steps_remaining =
            Some(SOURCE_FIRST_WAVE_LANDER_REFILL_DELAY_STEPS);
    }

    fn activate_first_wave_early_lander_reserve_if_ready(
        &mut self,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(remaining) = self.source_first_wave_early_reserve_steps_remaining else {
            return false;
        };
        if self.wave != 1 || self.enemy_reserve.landers == 0 {
            self.source_first_wave_early_reserve_steps_remaining = None;
            return false;
        }

        let remaining = remaining.saturating_sub(1);
        if remaining > 0 {
            self.source_first_wave_early_reserve_steps_remaining = Some(remaining);
            return false;
        }

        self.source_first_wave_early_reserve_steps_remaining = None;
        if self.source_counted_hostile_snapshot_count()
            >= SOURCE_FIRST_WAVE_EARLY_RESERVE_ACTIVE_LIMIT
        {
            return false;
        }

        let reserve_count = ACTOR_SOURCE_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS
            .len()
            .min(usize::from(self.enemy_reserve.landers));
        if reserve_count == 0 {
            return false;
        }

        self.apply_first_wave_early_reserve_shot_phase_delay();
        for spawn in ACTOR_SOURCE_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS
            .iter()
            .copied()
            .take(reserve_count)
        {
            commands.push(GameCommand::Spawn(SpawnRequest::Lander {
                position: spawn.position,
            }));
            let actor = self.spawn_lander_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Lander, actor);
        }
        self.enemy_reserve.landers = self
            .enemy_reserve
            .landers
            .saturating_sub(u8::try_from(reserve_count).expect("early reserve count fits u8"));
        self.source_rng = SOURCE_FIRST_WAVE_EARLY_RESERVE_RNG;
        self.source_target_cursor = Some(SOURCE_FIRST_WAVE_EARLY_RESERVE_TARGET_CURSOR_SLOT);
        true
    }

    fn activate_first_wave_lander_refill(&mut self, commands: &mut Vec<GameCommand>) -> bool {
        if self.enemy_reserve.landers == 0 {
            return false;
        }

        let reserve_count = ACTOR_SOURCE_FIRST_WAVE_REFILL_LANDER_SPAWNS
            .len()
            .min(usize::from(self.enemy_reserve.landers));
        if reserve_count == 0 {
            return false;
        }

        let mut materialized = false;
        for spawn in ACTOR_SOURCE_FIRST_WAVE_REFILL_LANDER_SPAWNS
            .iter()
            .copied()
            .take(reserve_count)
        {
            materialized |= spawn.source.is_some_and(source_lander_output_visible);
            commands.push(GameCommand::Spawn(SpawnRequest::Lander {
                position: spawn.position,
            }));
            let actor = self.spawn_lander_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Lander, actor);
        }
        self.enemy_reserve.landers = self
            .enemy_reserve
            .landers
            .saturating_sub(u8::try_from(reserve_count).expect("refill count fits u8"));
        if materialized {
            self.queue_first_wave_lander_refill_appearance_sound();
        }
        true
    }

    fn apply_first_wave_early_reserve_shot_phase_delay(&self) {
        for actor in self.actors.values() {
            actor.apply_driver_command(ActorDriverCommand::AdjustSourceLanderShotTimer {
                target_human_index: 2,
                x_velocity: 0xFFEE,
                delta: SOURCE_FIRST_WAVE_EARLY_RESERVE_TARGET2_SHOT_PHASE_DELAY,
            });
        }
    }

    fn spawn_wave_hostiles(&mut self) {
        let wave_profile = self.wave_script.profile_for_wave(self.wave).clone();
        for spawn in wave_profile.lander_spawns.iter().copied() {
            let actor = self.spawn_lander_from_spawn(spawn);
            if let Some(target_index) = spawn.source.and_then(|source| source.target_human_index) {
                self.source_target_cursor = Some(target_index);
            }
            self.apply_next_wave_spawn_behavior(ActorKind::Lander, actor);
        }
        for spawn in wave_profile.bomber_spawns.iter().copied() {
            let actor = self.spawn_bomber_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Bomber, actor);
        }
        for spawn in wave_profile.pod_spawns.iter().copied() {
            let actor = self.spawn_pod_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Pod, actor);
        }
        for spawn in wave_profile.mutant_spawns.iter().copied() {
            let actor = self.spawn_mutant_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Mutant, actor);
        }
        for spawn in wave_profile.swarmer_spawns.iter().copied() {
            let actor = self.spawn_swarmer_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Swarmer, actor);
        }
        for spawn in wave_profile.baiter_spawns.iter().copied() {
            let actor = self.spawn_baiter_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Baiter, actor);
        }
    }

    fn active_player_position(&self) -> Option<Point> {
        self.snapshots
            .values()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.position)
    }

    fn has_source_human_snapshots(&self) -> bool {
        self.snapshots.values().any(|snapshot| {
            snapshot.kind == ActorKind::Human && snapshot.alive && snapshot.source_human.is_some()
        })
    }

    fn human_snapshot_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
            .count()
    }

    fn source_counted_hostile_snapshot_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| is_hostile(snapshot.kind))
            .count()
    }

    fn source_counted_lander_snapshot_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .count()
    }

    fn select_next_source_lander_target_index(&mut self) -> Option<usize> {
        if !self.has_source_human_snapshots() {
            return None;
        }

        let original_cursor = self
            .source_target_cursor
            .filter(|slot| *slot < SOURCE_TARGET_LIST_ENTRY_COUNT)
            .unwrap_or(0);
        let mut probe = original_cursor;
        for _ in 0..SOURCE_TARGET_LIST_ENTRY_COUNT {
            probe = actor_source_target_list_next_slot_index(probe);
            if self.snapshots.values().any(|snapshot| {
                snapshot.kind == ActorKind::Human
                    && snapshot.alive
                    && snapshot
                        .source_human
                        .is_some_and(|source| source.target_slot_index == probe)
            }) {
                self.source_target_cursor = Some(probe);
                return Some(probe);
            }
            if probe == original_cursor {
                break;
            }
        }

        None
    }

    fn apply_next_wave_spawn_behavior(&mut self, kind: ActorKind, actor: ActorId) {
        let spawn_index = self.next_wave_spawn_index(kind);
        if let Some(profile) = self
            .wave_script
            .profile_for_wave(self.wave)
            .spawn_behavior_profile(kind, spawn_index)
        {
            self.behavior_script.set_actor_behavior(actor, profile);
        }
    }

    fn next_wave_spawn_index(&mut self, kind: ActorKind) -> usize {
        let next = self.wave_spawn_allocations.entry(kind).or_insert(0);
        let spawn_index = *next;
        *next = next.saturating_add(1);
        spawn_index
    }

    fn start_survivor_bonus_if_wave_cleared(
        &mut self,
        was_playing: bool,
        commands: &[GameCommand],
    ) -> bool {
        if !was_playing
            || self.phase != Phase::Playing
            || self.wave == 0
            || self.pending_survivor_bonus.is_some()
            || self.pending_player_start.is_some()
            || !actor_enemy_reserve_is_empty(self.enemy_reserve)
            || self.has_hostile_snapshots()
            || commands_spawn_hostiles(commands)
        {
            return false;
        }

        self.pending_survivor_bonus = Some(PendingSurvivorBonus::new(
            self.wave,
            self.wave.saturating_add(1),
            self.surviving_human_count(),
        ));
        true
    }

    fn surviving_human_count(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Human && snapshot.alive)
            .count()
    }

    fn has_hostile_snapshots(&self) -> bool {
        self.snapshots
            .values()
            .any(|snapshot| is_hostile(snapshot.kind))
    }

    fn pod_swarmer_spawn_commands(&mut self, position: Point) -> Vec<GameCommand> {
        let active_swarmers = self
            .snapshots
            .values()
            .filter(|snapshot| snapshot.kind == ActorKind::Swarmer)
            .count();
        let spawn_count = SOURCE_POD_SWARMER_REQUEST_LIMIT
            .min(SOURCE_ACTIVE_SWARMER_LIMIT.saturating_sub(active_swarmers));
        let source_profile = self.current_source_wave_profile();

        (0..spawn_count)
            .map(|_| {
                let spawn = ActorSwarmerSpawn::source_from_pod(
                    &mut self.source_rng,
                    source_profile,
                    position,
                );
                GameCommand::Spawn(SpawnRequest::Swarmer {
                    position: spawn.position,
                    source: spawn.source,
                })
            })
            .collect()
    }

    fn advance_baiter_timer(&mut self, commands: &mut Vec<GameCommand>) {
        if self.phase != Phase::Playing || self.wave == 0 {
            return;
        }
        let enemy_total = self.source_wave_enemy_total();
        if enemy_total == 0 {
            return;
        }
        let Some(timer_steps) = self.baiter_timer_steps else {
            return;
        };

        if self.baiter_pacing_steps_remaining > 1 {
            self.baiter_pacing_steps_remaining =
                self.baiter_pacing_steps_remaining.saturating_sub(1);
            return;
        }
        self.baiter_pacing_steps_remaining = ACTOR_BAITER_TIMER_PACING_STEPS;

        let profile = self.current_source_wave_profile();
        let timer_steps = source_baiter_accelerated_timer_steps(timer_steps, profile, enemy_total);
        let decremented_steps = timer_steps.saturating_sub(1);
        if decremented_steps > 0 {
            self.baiter_timer_steps = Some(decremented_steps);
            return;
        }

        self.baiter_timer_steps = Some(source_baiter_reset_timer_steps(profile, enemy_total));
        let active_baiters = self.snapshot_count(ActorKind::Baiter);
        if active_baiters >= SOURCE_ACTIVE_BAITER_LIMIT {
            return;
        }
        let Some(player_position) = self
            .snapshots
            .values()
            .find(|snapshot| snapshot.kind == ActorKind::Player && snapshot.alive)
            .map(|snapshot| snapshot.position)
        else {
            return;
        };
        let spawn = ActorBaiterSpawn::source_from_player(profile, player_position, active_baiters);
        commands.push(GameCommand::Spawn(SpawnRequest::Baiter {
            position: spawn.position,
            source: spawn.source,
        }));
    }

    fn source_wave_enemy_total(&self) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| is_hostile(snapshot.kind))
            .count()
    }

    fn spawn_initial_humans(&mut self) {
        let wave_profile = self.wave_script.profile_for_wave(self.wave).clone();
        for spawn in wave_profile.human_spawns.iter().copied() {
            let actor = self.spawn_human_from_spawn(spawn);
            self.apply_next_wave_spawn_behavior(ActorKind::Human, actor);
        }
    }

    fn detonate_smart_bomb_targets(&mut self, commands: &mut Vec<GameCommand>) -> bool {
        let mut bonus_awarded = false;
        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| is_smart_bomb_target(snapshot.kind))
            .map(|snapshot| (snapshot.id, snapshot.kind, snapshot.position))
            .collect::<Vec<_>>();
        for (id, kind, position) in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
            commands.push(GameCommand::Destroy(id));
            if let Some(explosion_kind) = explosion_kind_for_target(kind) {
                self.spawn_explosion(position, explosion_kind);
                commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                    position,
                    kind: explosion_kind,
                    source_center: None,
                }));
            }
            let points = score_for_hostile(kind);
            commands.push(GameCommand::AddScore(points));
            if self.award_points(points) {
                bonus_awarded = true;
            }
        }
        bonus_awarded
    }

    fn award_points(&mut self, points: u32) -> bool {
        let frame = ScoreSystem::award_points(
            ScoreSnapshot {
                player_one: self.score,
                player_two: self.player_two_score,
                high_score: self.highest_visible_score(),
                next_bonus: self.next_bonus,
            },
            self.active_stock(),
            self.current_player,
            points,
        );
        self.score = frame.scores.player_one;
        self.player_two_score = frame.scores.player_two;
        self.set_active_stock(frame.stock);
        self.next_bonus = frame.scores.next_bonus;
        frame.bonus_awards > 0
    }

    fn clear_enemy_projectiles_for_hyperspace(&mut self) {
        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| is_source_shell_kind(snapshot.kind))
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>();
        for id in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
        }
    }

    fn remove_dead_actors(&mut self, dead_actor_ids: &[ActorId]) {
        for id in dead_actor_ids {
            self.snapshots.remove(id);
            self.actors.remove(id);
            self.behavior_script.remove_actor_behavior(*id);
        }
    }

    fn clear_wave_playfield_actors(&mut self) {
        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| clears_for_next_wave(snapshot.kind))
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>();
        for id in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
        }
    }

    fn clear_turn_playfield_actors(&mut self) {
        let targets = self
            .snapshots
            .values()
            .filter(|snapshot| clears_for_next_turn(snapshot.kind))
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>();
        for id in targets {
            self.snapshots.remove(&id);
            self.actors.remove(&id);
            self.behavior_script.remove_actor_behavior(id);
        }
    }
}

impl Default for ActorGameDriver {
    fn default() -> Self {
        Self::new()
    }
}

fn center_of(bounds: Rect) -> Point {
    Point::new(
        (bounds.left + bounds.right) / 2,
        (bounds.top + bounds.bottom) / 2,
    )
}

fn manhattan_distance(left: Point, right: Point) -> i16 {
    (left.x - right.x).abs() + (left.y - right.y).abs()
}

fn is_hostile(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Pod
            | ActorKind::Swarmer
    )
}

fn clears_for_next_wave(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::Human
            | ActorKind::Laser
            | ActorKind::EnemyLaser
            | ActorKind::Explosion
            | ActorKind::ScorePopup
    )
}

fn clears_for_next_turn(kind: ActorKind) -> bool {
    kind == ActorKind::Player || clears_for_next_wave(kind)
}

fn is_player_laser_target(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::EnemyLaser
    )
}

fn is_source_shell_kind(kind: ActorKind) -> bool {
    matches!(kind, ActorKind::EnemyLaser | ActorKind::Bomb)
}

fn source_shell_slot_available(active_source_shells: usize) -> bool {
    active_source_shells < SOURCE_SHELL_LIMIT
}

fn bomb_shell_slot_available(active_bomb_shells: usize) -> bool {
    active_bomb_shells < SOURCE_ACTIVE_BOMBER_BOMB_LIMIT
}

fn source_shell_spawn_in_bounds(position: Point) -> bool {
    position.x < SOURCE_SHELL_X_MAX && position.y > i16::from(SOURCE_PLAYFIELD_Y_MIN)
}

fn bomb_shell_spawn_in_source_bounds(
    position: Point,
    source: Option<ActorSourceEnemyProjectileMetadata>,
) -> bool {
    source.is_none() || source_shell_spawn_in_bounds(position)
}

fn reserve_source_shell_slot(active_source_shells: &mut usize) -> bool {
    if !source_shell_slot_available(*active_source_shells) {
        return false;
    }
    *active_source_shells += 1;
    true
}

fn is_player_hazard(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::EnemyLaser
    )
}

fn is_player_enemy_collision_target(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
    )
}

fn is_smart_bomb_target(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::EnemyLaser
    )
}

fn commands_spawn_hostiles(commands: &[GameCommand]) -> bool {
    commands.iter().any(|command| {
        matches!(
            command,
            GameCommand::Spawn(SpawnRequest::Lander { .. })
                | GameCommand::Spawn(SpawnRequest::Mutant { .. })
                | GameCommand::Spawn(SpawnRequest::Bomber { .. })
                | GameCommand::Spawn(SpawnRequest::Pod { .. })
                | GameCommand::Spawn(SpawnRequest::Swarmer { .. })
        )
    })
}

fn score_for_hostile(kind: ActorKind) -> u32 {
    match kind {
        ActorKind::Lander => LANDER_SCORE,
        ActorKind::Mutant => MUTANT_SCORE,
        ActorKind::Bomber => BOMBER_SCORE,
        ActorKind::Bomb => 0,
        ActorKind::Pod => POD_SCORE,
        ActorKind::Swarmer => SWARMER_SCORE,
        ActorKind::Baiter => BAITER_SCORE,
        _ => 0,
    }
}

fn hit_sound_for_hostile(kind: ActorKind) -> SoundCue {
    match kind {
        ActorKind::Lander => SoundCue::LanderHit,
        ActorKind::Mutant => SoundCue::MutantHit,
        ActorKind::Bomber => SoundCue::BomberHit,
        ActorKind::Bomb => SoundCue::BombHit,
        ActorKind::Pod => SoundCue::PodHit,
        ActorKind::Swarmer => SoundCue::SwarmerHit,
        ActorKind::Baiter => SoundCue::BaiterHit,
        _ => SoundCue::Explosion,
    }
}

fn player_hazard_sound(kind: ActorKind) -> SoundCue {
    match kind {
        ActorKind::Bomb => SoundCue::BombHit,
        _ => SoundCue::Explosion,
    }
}

fn explosion_kind_for_target(kind: ActorKind) -> Option<ExplosionKind> {
    let kind = match kind {
        ActorKind::Lander => ExplosionKind::Lander,
        ActorKind::Mutant => ExplosionKind::Mutant,
        ActorKind::Bomber => ExplosionKind::Bomber,
        ActorKind::Pod => ExplosionKind::Pod,
        ActorKind::Swarmer => ExplosionKind::Swarmer,
        ActorKind::Baiter => ExplosionKind::Baiter,
        ActorKind::Bomb | ActorKind::EnemyLaser => ExplosionKind::Bomb,
        _ => return None,
    };
    Some(kind)
}

fn player_hazard_explosion_kind(kind: ActorKind) -> ExplosionKind {
    match kind {
        ActorKind::Bomb => ExplosionKind::Bomb,
        _ => ExplosionKind::Player,
    }
}

fn source_baiter_accelerated_timer_steps(
    current_steps: u32,
    profile: ActorSourceWaveProfile,
    enemy_total: usize,
) -> u32 {
    if enemy_total > 8 {
        return current_steps;
    }

    let mut target_steps = profile.baiter_delay / 2;
    if enemy_total <= 3 {
        target_steps /= 2;
    }
    target_steps = target_steps.saturating_add(1).max(1);
    current_steps.min(target_steps)
}

fn source_baiter_reset_timer_steps(profile: ActorSourceWaveProfile, enemy_total: usize) -> u32 {
    if enemy_total < 4 {
        (profile.baiter_delay / 4).max(1)
    } else {
        profile.baiter_delay.max(1)
    }
}

#[derive(Debug, Clone)]
struct HighScoreTable {
    entries: [u32; 5],
}

impl HighScoreTable {
    fn entries(&self) -> [u32; 5] {
        self.entries
    }

    fn qualifies(&self, score: u32) -> bool {
        self.entries.iter().any(|entry| score > *entry)
    }

    fn record(&mut self, score: u32) {
        if !self.qualifies(score) {
            return;
        }
        let mut entries = self.entries.to_vec();
        entries.push(score);
        entries.sort_by(|left, right| right.cmp(left));
        self.entries.copy_from_slice(&entries[..5]);
    }
}

impl Default for HighScoreTable {
    fn default() -> Self {
        Self {
            entries: [10_000, 7_500, 5_000, 2_500, 1_000],
        }
    }
}

#[derive(Debug)]
struct AttractDirector {
    id: ActorId,
}

impl AttractDirector {
    fn new(id: ActorId) -> Self {
        Self { id }
    }
}

impl AssetActor for AttractDirector {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        for _ in 0..prompt.input.coin_insertions() {
            commands.push(GameCommand::Credit);
        }
        if prompt.input.start_one {
            commands.push(GameCommand::StartOnePlayer);
        }
        if prompt.input.start_two {
            commands.push(GameCommand::StartTwoPlayer);
        }
        if prompt.phase == Phase::HighScoreEntry {
            draws.push(DrawCommand::text(
                self.id,
                Point::new(66, 120),
                "ENTER INITIALS",
            ));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::AttractDirector,
                position: Point::new(0, 0),
                velocity: Velocity::default(),
                direction: None,
                bounds: None,
                alive: true,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct ScriptedAttractProgram {
    id: ActorId,
    script: AttractScript,
    elapsed_steps: u64,
}

impl ScriptedAttractProgram {
    fn new(id: ActorId, script: AttractScript) -> Self {
        Self {
            id,
            script,
            elapsed_steps: 0,
        }
    }
}

impl AssetActor for ScriptedAttractProgram {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let draws = match prompt.phase {
            Phase::Attract => {
                self.elapsed_steps = self.elapsed_steps.saturating_add(1);
                self.script.draws_for(
                    self.id,
                    self.elapsed_steps,
                    &prompt.high_scores,
                    prompt.credits,
                )
            }
            Phase::GameOver if prompt.game_over_hall_of_fame_stall_remaining.is_some() => {
                self.elapsed_steps = 0;
                self.script.draws_for(
                    self.id,
                    SOURCE_ATTRACT_HALL_OF_FAME_START_STEP,
                    &prompt.high_scores,
                    prompt.credits,
                )
            }
            Phase::GameOver if prompt.player_switch.is_some() => {
                self.elapsed_steps = 0;
                Vec::new()
            }
            Phase::GameOver => {
                self.elapsed_steps = self.elapsed_steps.saturating_add(1);
                self.script.draws_for(
                    self.id,
                    self.elapsed_steps,
                    &prompt.high_scores,
                    prompt.credits,
                )
            }
            Phase::Playing | Phase::HighScoreEntry => {
                self.elapsed_steps = 0;
                Vec::new()
            }
        };

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::AttractScript,
                position: Point::new(0, 0),
                velocity: Velocity::default(),
                direction: None,
                bounds: None,
                alive: true,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands: Vec::new(),
            draws,
        }
    }
}

#[derive(Debug)]
struct StatusDisplay {
    id: ActorId,
}

impl StatusDisplay {
    fn new(id: ActorId) -> Self {
        Self { id }
    }

    fn playing_draws(&self, prompt: &StepPrompt) -> Vec<DrawCommand> {
        let mut draws = vec![
            DrawCommand::text(
                self.id,
                STATUS_SCORE_POSITION,
                format!("1UP {}", format_status_score(prompt.player_scores[0])),
            ),
            DrawCommand::text(
                self.id,
                STATUS_HIGH_SCORE_POSITION,
                format!("HIGH {}", format_status_score(prompt.high_scores[0])),
            ),
            DrawCommand::text(
                self.id,
                STATUS_WAVE_POSITION,
                format!("WAVE {:02}", prompt.wave.min(99)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_LIVES_POSITION,
                format!("LIVES {:02}", prompt.lives.min(99)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_SMART_BOMBS_POSITION,
                format!("BOMBS {:02}", prompt.smart_bombs.min(99)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_CREDITS_POSITION,
                format!("CREDIT {:02}", prompt.credits.min(99)),
            ),
        ];
        if prompt.player_count > 1 {
            draws.push(DrawCommand::text(
                self.id,
                STATUS_PLAYER_TWO_SCORE_POSITION,
                format!("2UP {}", format_status_score(prompt.player_scores[1])),
            ));
        }
        draws
    }

    fn high_score_entry_draws(&self, prompt: &StepPrompt) -> Vec<DrawCommand> {
        let mut draws = vec![
            DrawCommand::text(
                self.id,
                STATUS_FINAL_SCORE_POSITION,
                format!("FINAL SCORE {}", format_status_score(prompt.score)),
            ),
            DrawCommand::text(
                self.id,
                STATUS_HIGH_SCORE_TABLE_TITLE_POSITION,
                "HIGH SCORES",
            ),
            DrawCommand::text(
                self.id,
                Point::new(66, 104),
                format!(
                    "INITIALS {}",
                    format_high_score_initials(prompt.high_score_initials)
                ),
            ),
        ];

        for (index, score) in prompt.high_scores.iter().enumerate() {
            draws.push(DrawCommand::text(
                self.id,
                Point::new(
                    82,
                    STATUS_HIGH_SCORE_TABLE_START_Y
                        + i16::try_from(index).unwrap_or(0) * STATUS_HIGH_SCORE_TABLE_ROW_HEIGHT,
                ),
                format!("{}. {}", index + 1, format_status_score(*score)),
            ));
        }
        draws
    }

    fn player_switch_draws(&self, prompt: &StepPrompt) -> Vec<DrawCommand> {
        if prompt.player_switch.is_none() {
            return Vec::new();
        };

        vec![
            DrawCommand::text(
                self.id,
                STATUS_SCORE_POSITION,
                format!("1UP {}", format_status_score(prompt.player_scores[0])),
            ),
            DrawCommand::text(
                self.id,
                STATUS_PLAYER_TWO_SCORE_POSITION,
                format!("2UP {}", format_status_score(prompt.player_scores[1])),
            ),
            DrawCommand::text(
                self.id,
                STATUS_HIGH_SCORE_POSITION,
                format!("HIGH {}", format_status_score(prompt.high_scores[0])),
            ),
            DrawCommand::text(
                self.id,
                STATUS_CREDITS_POSITION,
                format!("CREDIT {:02}", prompt.credits.min(99)),
            ),
        ]
    }

    fn snapshot(&self) -> ActorSnapshot {
        ActorSnapshot {
            id: self.id,
            kind: ActorKind::StatusDisplay,
            position: Point::new(0, 0),
            velocity: Velocity::default(),
            direction: None,
            bounds: None,
            alive: true,
            source_lander: None,
            source_bomber: None,
            source_pod: None,
            source_swarmer: None,
            source_baiter: None,
            source_mutant: None,
            source_human: None,
            source_enemy_projectile: None,
        }
    }
}

impl AssetActor for StatusDisplay {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let draws = match prompt.phase {
            Phase::Playing => self.playing_draws(prompt),
            Phase::HighScoreEntry => self.high_score_entry_draws(prompt),
            Phase::GameOver if prompt.player_switch.is_some() => self.player_switch_draws(prompt),
            Phase::Attract | Phase::GameOver => Vec::new(),
        };

        ActorReply {
            id: self.id,
            snapshot: self.snapshot(),
            commands: Vec::new(),
            draws,
        }
    }
}

fn format_status_score(score: u32) -> String {
    format!("{:06}", score.min(999_999))
}

fn format_high_score_initials(state: HighScoreInitialsState) -> String {
    state
        .initials
        .iter()
        .map(|initial| initial.unwrap_or('_'))
        .collect()
}

fn player_source_message_label(player: u8) -> &'static str {
    if player == 2 { "PLYR2" } else { "PLYR1" }
}

#[derive(Debug)]
struct PlayerShip {
    id: ActorId,
    position: Point,
    direction: Direction,
    laser_cooldown: u8,
    hyperspace_steps_remaining: u8,
    hyperspace_entry_lseed: Option<u8>,
    hyperspace_death_steps_remaining: Option<u8>,
}

impl PlayerShip {
    fn new(id: ActorId, position: Point) -> Self {
        Self {
            id,
            position,
            direction: Direction::Right,
            laser_cooldown: 0,
            hyperspace_steps_remaining: 0,
            hyperspace_entry_lseed: None,
            hyperspace_death_steps_remaining: None,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 18, 10)
    }

    fn is_hidden_for_hyperspace(&self) -> bool {
        self.hyperspace_steps_remaining > 0
    }

    fn enter_hyperspace(&mut self, behavior: ActorBehaviorProfile) {
        self.hyperspace_steps_remaining = behavior.player_hyperspace_hidden_steps;
        self.hyperspace_entry_lseed = Some(Self::hyperspace_death_lseed(behavior));
    }

    fn hyperspace_death_lseed(behavior: ActorBehaviorProfile) -> u8 {
        behavior
            .player_hyperspace_source_seed
            .map_or(behavior.player_hyperspace_death_lseed, |source| {
                source.lseed
            })
    }

    fn hyperspace_rematerialization(
        &self,
        behavior: ActorBehaviorProfile,
    ) -> (Point, Direction, Option<u16>) {
        if let Some(source) = behavior.player_hyperspace_source_seed {
            let (x, direction) = if source.hseed & 1 != 0 {
                (0x20, Direction::Right)
            } else {
                (0x70, Direction::Left)
            };
            let y = (source.hseed >> 1).wrapping_add(SOURCE_PLAYFIELD_Y_MIN);
            return (
                Point::new(x, i16::from(y)),
                direction,
                Some(actor_source_hyperspace_background_left(source)),
            );
        }

        (
            Point::new(
                behavior.player_hyperspace_rematerialize_x,
                behavior.player_hyperspace_rematerialize_y,
            ),
            self.direction,
            None,
        )
    }

    fn advance_hyperspace(
        &mut self,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        if self.hyperspace_steps_remaining == 0 {
            return false;
        }

        self.hyperspace_steps_remaining = self.hyperspace_steps_remaining.saturating_sub(1);
        if self.hyperspace_steps_remaining == 0 {
            let (position, direction, source_background_left) =
                self.hyperspace_rematerialization(behavior);
            self.position = PLAYER_BOUNDS.clamp_point(position);
            self.direction = direction;
            if let Some(source_background_left) = source_background_left {
                commands.push(GameCommand::SetSourceBackgroundLeft(source_background_left));
            }
            let death_lseed = self
                .hyperspace_entry_lseed
                .take()
                .unwrap_or_else(|| Self::hyperspace_death_lseed(behavior));
            if death_lseed > SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD {
                self.hyperspace_death_steps_remaining =
                    Some(behavior.player_hyperspace_death_delay_steps);
            }
            return true;
        }
        false
    }

    fn advance_hyperspace_death(&mut self, commands: &mut Vec<GameCommand>) -> bool {
        let Some(steps_remaining) = self.hyperspace_death_steps_remaining else {
            return false;
        };

        let steps_remaining = steps_remaining.saturating_sub(1);
        if steps_remaining > 0 {
            self.hyperspace_death_steps_remaining = Some(steps_remaining);
            return false;
        }

        self.hyperspace_death_steps_remaining = None;
        commands.push(GameCommand::Destroy(self.id));
        commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
            position: self.position,
            kind: ExplosionKind::Player,
            source_center: None,
        }));
        commands.push(GameCommand::PlaySound(SoundCue::Explosion));
        commands.push(GameCommand::PlayerKilled);
        true
    }
}

impl AssetActor for PlayerShip {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let mut death_due = false;
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Player);
            death_due = self.advance_hyperspace_death(&mut commands);
            let was_hidden = self.is_hidden_for_hyperspace();
            if self.advance_hyperspace(behavior, &mut commands) {
                commands.push(GameCommand::PlaySound(SoundCue::HyperspaceMaterialize));
            }
            let input_blocked = was_hidden || self.hyperspace_death_steps_remaining.is_some();
            if !death_due && !input_blocked {
                let mut velocity = Velocity::default();
                if prompt.input.altitude_up {
                    velocity.dy -= behavior.player_speed;
                }
                if prompt.input.altitude_down {
                    velocity.dy += behavior.player_speed;
                }
                if prompt.input.thrust {
                    velocity.dx += self.direction.sign() * behavior.player_speed;
                    commands.push(GameCommand::PlaySound(SoundCue::Thrust));
                }
                if prompt.input.reverse {
                    self.direction = match self.direction {
                        Direction::Left => Direction::Right,
                        Direction::Right => Direction::Left,
                    };
                }
                self.position = PLAYER_BOUNDS.clamp_point(self.position.offset(velocity));
                self.laser_cooldown = self.laser_cooldown.saturating_sub(1);
                if prompt.input.wants_fire() && self.laser_cooldown == 0 {
                    self.laser_cooldown = behavior.player_laser_cooldown_steps;
                    let muzzle = self
                        .position
                        .offset(Velocity::new(self.direction.sign() * 12, 0));
                    commands.push(GameCommand::Spawn(SpawnRequest::Laser {
                        position: muzzle,
                        direction: self.direction,
                        owner: self.id,
                    }));
                    commands.push(GameCommand::PlaySound(SoundCue::Laser));
                }
                if prompt.input.xyzzy.overlay_smart_bomb && !prompt.smart_bomb_pending {
                    commands.push(GameCommand::SmartBomb {
                        consume_stock: false,
                    });
                } else if prompt.input.wants_stock_smart_bomb()
                    && prompt.smart_bombs > 0
                    && !prompt.smart_bomb_pending
                {
                    commands.push(GameCommand::SmartBomb {
                        consume_stock: true,
                    });
                }
                if prompt.input.hyperspace {
                    commands.push(GameCommand::Hyperspace);
                    commands.push(GameCommand::PlaySound(SoundCue::Hyperspace));
                    self.enter_hyperspace(behavior);
                }
            }
            if !death_due && !self.is_hidden_for_hyperspace() {
                draws.push(DrawCommand::sprite(
                    self.id,
                    match self.direction {
                        Direction::Left => SpriteKey::PlayerLeft,
                        Direction::Right => SpriteKey::PlayerRight,
                    },
                    self.position,
                ));
            }
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Player,
                position: self.position,
                velocity: if prompt.phase == Phase::Playing {
                    observed_velocity(previous_position, self.position)
                } else {
                    Velocity::default()
                },
                direction: Some(self.direction),
                bounds: if prompt.phase == Phase::Playing
                    && !self.is_hidden_for_hyperspace()
                    && !death_due
                {
                    Some(self.bounds())
                } else {
                    None
                },
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct Lander {
    id: ActorId,
    position: Point,
    drift: i16,
    mode: LanderMode,
    source: Option<ActorSourceLanderMetadata>,
    source_output: SourceLanderOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LanderMode {
    Seeking,
    Carrying {
        human_id: ActorId,
        pull_sound_sent: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceLanderOutput {
    Normal,
    VisibleFirstWaveRefill,
    HiddenFirstWaveRefill,
}

impl Lander {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorLanderSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorLanderSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .source
                .map(|source| source_lander_drift_from_velocity(source.x_velocity))
                .unwrap_or(-1),
            mode: LanderMode::Seeking,
            source_output: source_lander_output(spawn.source),
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 14, 12)
    }
}

impl AssetActor for Lander {
    fn id(&self) -> ActorId {
        self.id
    }

    fn apply_driver_command(&mut self, command: ActorDriverCommand) {
        let ActorDriverCommand::AdjustSourceLanderShotTimer {
            target_human_index,
            x_velocity,
            delta,
        } = command;
        if let Some(source) = &mut self.source
            && source.target_human_index == Some(target_human_index)
            && source.x_velocity == x_velocity
        {
            source.shot_timer = source.shot_timer.wrapping_add(delta);
        }
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Lander);
            if self.source_output != SourceLanderOutput::HiddenFirstWaveRefill
                && !self.tick_source_sleep()
            {
                match self.mode {
                    LanderMode::Seeking => self.update_seeking(prompt, behavior, &mut commands),
                    LanderMode::Carrying {
                        human_id,
                        pull_sound_sent,
                    } => {
                        self.update_carrying(
                            prompt,
                            human_id,
                            pull_sound_sent,
                            behavior,
                            &mut commands,
                        );
                    }
                }
                self.tick_fire_timer(prompt, behavior, &mut commands);
            }
            if self.output_visible() {
                draws.push(DrawCommand::sprite_with_effect(
                    self.id,
                    SpriteKey::Lander,
                    self.position,
                    self.draw_effect(),
                ));
            }
        }
        let movement_velocity = observed_velocity(previous_position, self.position);
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Lander,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(
                    movement_velocity,
                    drift_direction(self.drift),
                )),
                bounds: self.output_visible().then_some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: self.source,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

impl Lander {
    fn output_visible(&self) -> bool {
        self.source_output != SourceLanderOutput::HiddenFirstWaveRefill
    }

    fn update_seeking(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        match behavior.lander_mode {
            LanderBehaviorMode::SeekNearestHuman => {
                self.seek_nearest_human(prompt, behavior, commands);
            }
            LanderBehaviorMode::ChasePlayer => {
                if let Some(player) = prompt.player_position() {
                    self.position = step_toward(self.position, player, behavior.lander_seek_speed);
                } else {
                    self.drift(behavior);
                }
            }
            LanderBehaviorMode::Drift => {
                if !self.advance_source_fixed_point_motion() {
                    self.drift(behavior);
                }
            }
        }
    }

    fn seek_nearest_human(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        let target = self
            .source_target_human(prompt)
            .or_else(|| prompt.nearest_human(self.position));

        if let Some(target) = target {
            if pickup_distance(self.position, target.position, behavior) {
                self.mode = LanderMode::Carrying {
                    human_id: target.id,
                    pull_sound_sent: false,
                };
                commands.push(GameCommand::AttachHuman {
                    lander: self.id,
                    human: target.id,
                    position: target.position,
                });
                commands.push(GameCommand::PlaySound(SoundCue::LanderPickup));
                return;
            }
            if self.advance_source_fixed_point_motion() {
                return;
            }
            self.position = step_toward(self.position, target.position, behavior.lander_seek_speed);
            return;
        }

        if let Some(player) = prompt.player_position() {
            self.drift = if player.x < self.position.x { -1 } else { 1 };
        }
        if !self.advance_source_fixed_point_motion() {
            self.drift(behavior);
        }
    }

    fn source_target_human<'a>(&self, prompt: &'a StepPrompt) -> Option<&'a ActorSnapshot> {
        self.source
            .and_then(|source| source.target_human_index)
            .and_then(|target_slot_index| prompt.source_target_human(target_slot_index))
    }

    fn drift(&mut self, behavior: ActorBehaviorProfile) {
        self.position = self.position.offset(Velocity::new(
            self.drift * behavior.lander_drift_speed.max(0),
            0,
        ));
    }

    fn advance_source_fixed_point_motion(&mut self) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };
        if source.x_velocity == 0 && source.y_velocity == 0 {
            return false;
        }

        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) = actor_source_active_object_y_step(
            self.position.y,
            source.y_fraction,
            source.y_velocity,
        );
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        self.drift = source_lander_drift_from_velocity(source.x_velocity);
        true
    }

    fn update_carrying(
        &mut self,
        prompt: &StepPrompt,
        human_id: ActorId,
        pull_sound_sent: bool,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        self.position = self
            .position
            .offset(Velocity::new(0, -behavior.lander_carry_speed));
        if !pull_sound_sent {
            self.mode = LanderMode::Carrying {
                human_id,
                pull_sound_sent: true,
            };
            commands.push(GameCommand::PlaySound(SoundCue::HumanPulled));
        }
        if self.position.y <= behavior.lander_conversion_y {
            commands.push(GameCommand::Destroy(self.id));
            commands.push(GameCommand::Destroy(human_id));
            commands.push(GameCommand::Spawn(SpawnRequest::Mutant {
                position: self.position,
                source: self.source_mutant_conversion(prompt),
            }));
            commands.push(GameCommand::PlaySound(SoundCue::MutantSpawn));
        }
    }

    fn source_mutant_conversion(&self, prompt: &StepPrompt) -> Option<ActorSourceMutantMetadata> {
        let source = self.source?;
        let hop_rng = prompt.source_rng?;
        Some(ActorSourceMutantMetadata::from_lander_conversion(
            source,
            prompt.source_wave,
            hop_rng,
        ))
    }

    fn tick_source_sleep(&mut self) -> bool {
        if let Some(source) = &mut self.source
            && source.sleep_ticks > 0
        {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return true;
        }
        false
    }

    fn tick_fire_timer(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        let mut source_fired = false;
        if let Some(source) = &mut self.source {
            if source.shot_timer > 0 {
                source.shot_timer = source.shot_timer.saturating_sub(1);
            }
            if source.shot_timer == 0 {
                source.shot_timer = clamped_source_lander_shot_reset(behavior);
                source_fired = true;
            }
        }
        if source_fired {
            self.fire_lander_shot(prompt, behavior, commands);
            return;
        }
        if self.source.is_some() {
            return;
        }

        let fire_period = behavior.lander_fire_period_steps.max(1);
        if prompt.step % fire_period == self.id.value() % fire_period {
            self.fire_lander_shot(prompt, behavior, commands);
        }
    }

    fn fire_lander_shot(
        &self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        let velocity = self.lander_shot_velocity(prompt, behavior);
        let source = self.source.map(|source| {
            actor_source_enemy_shot_metadata(
                source.x_fraction,
                source.y_fraction,
                velocity,
                behavior.lander_shot_lifetime_steps,
            )
        });
        commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
            position: self.position,
            velocity,
            source,
        }));
        commands.push(GameCommand::PlaySound(SoundCue::LanderShot));
    }

    fn lander_shot_velocity(
        &self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> Velocity {
        let speed = behavior.lander_shot_speed.max(1);
        if let Some(player) = prompt.player_position() {
            return Velocity::new(
                axis_step(player.x - self.position.x, speed),
                axis_step(player.y - self.position.y, speed),
            );
        }

        Velocity::new(self.drift.signum() * speed, 0)
    }

    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceLanderFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

const fn source_lander_drift_from_velocity(x_velocity: u16) -> i16 {
    actor_source_drift_from_velocity(x_velocity)
}

fn source_lander_output(source: Option<ActorSourceLanderMetadata>) -> SourceLanderOutput {
    let Some(source) = source else {
        return SourceLanderOutput::Normal;
    };
    source_first_wave_refill_lander_output(source).unwrap_or(SourceLanderOutput::Normal)
}

fn source_lander_output_visible(source: ActorSourceLanderMetadata) -> bool {
    source_first_wave_refill_lander_output(source)
        == Some(SourceLanderOutput::VisibleFirstWaveRefill)
}

fn source_first_wave_refill_lander_output(
    source: ActorSourceLanderMetadata,
) -> Option<SourceLanderOutput> {
    ACTOR_SOURCE_FIRST_WAVE_REFILL_LANDER_SPAWNS
        .iter()
        .copied()
        .filter_map(|spawn| spawn.source)
        .enumerate()
        .find_map(|(index, refill_source)| {
            source_lander_metadata_matches_refill_row(source, refill_source).then_some(
                if index == 2 {
                    SourceLanderOutput::VisibleFirstWaveRefill
                } else {
                    SourceLanderOutput::HiddenFirstWaveRefill
                },
            )
        })
}

fn source_lander_metadata_matches_refill_row(
    source: ActorSourceLanderMetadata,
    refill_source: ActorSourceLanderMetadata,
) -> bool {
    source.x_fraction == refill_source.x_fraction
        && source.y_fraction == refill_source.y_fraction
        && source.x_velocity == refill_source.x_velocity
        && source.y_velocity == refill_source.y_velocity
        && source.shot_timer == refill_source.shot_timer
        && source.sleep_ticks == refill_source.sleep_ticks
        && source.picture_frame == refill_source.picture_frame
        && source.target_human_index == refill_source.target_human_index
}

const fn actor_source_drift_from_velocity(x_velocity: u16) -> i16 {
    if x_velocity & 0x8000 != 0 {
        -1
    } else if x_velocity == 0 {
        0
    } else {
        1
    }
}

fn drift_direction(drift: i16) -> Direction {
    if drift < 0 {
        Direction::Left
    } else {
        Direction::Right
    }
}

fn clamped_source_lander_shot_reset(behavior: ActorBehaviorProfile) -> u8 {
    let clamped = behavior
        .lander_fire_period_steps
        .max(1)
        .min(u64::from(u8::MAX));
    u8::try_from(clamped).unwrap_or(u8::MAX)
}

fn actor_source_axis_step(position: i16, fraction: u8, velocity: u16) -> (i16, u8) {
    let [position, fraction] = u16::from_be_bytes([position as u8, fraction])
        .wrapping_add(velocity)
        .to_be_bytes();
    (i16::from(position), fraction)
}

fn actor_source_active_object_y_step(position: i16, fraction: u8, velocity: u16) -> (i16, u8) {
    let [mut position, fraction] = u16::from_be_bytes([position as u8, fraction])
        .wrapping_add(velocity)
        .to_be_bytes();
    if position < SOURCE_PLAYFIELD_Y_MIN {
        position = SOURCE_PLAYFIELD_Y_MAX;
    } else if position > SOURCE_PLAYFIELD_Y_MAX {
        position = SOURCE_PLAYFIELD_Y_MIN;
    }
    (i16::from(position), fraction)
}

fn pickup_distance(lander: Point, human: Point, behavior: ActorBehaviorProfile) -> bool {
    (lander.x - human.x).abs() <= behavior.lander_pickup_radius_x
        && (lander.y - human.y).abs() <= behavior.lander_pickup_radius_y
}

fn step_toward(position: Point, target: Point, speed: i16) -> Point {
    Point::new(
        position.x + axis_step(target.x - position.x, speed),
        position.y + axis_step(target.y - position.y, speed),
    )
}

fn axis_step(delta: i16, speed: i16) -> i16 {
    let speed = speed.max(0);
    if delta == 0 {
        0
    } else if delta > 0 {
        delta.min(speed)
    } else {
        delta.max(-speed)
    }
}

fn move_by_hostile_mode(
    position: Point,
    mode: HostileMovementMode,
    prompt: &StepPrompt,
    speed: i16,
    drift: i16,
) -> Option<Point> {
    match mode {
        HostileMovementMode::Drift => Some(position.offset(Velocity::new(drift * speed.max(0), 0))),
        HostileMovementMode::ChasePlayer => prompt
            .player_position()
            .map(|player| step_toward(position, player, speed)),
    }
}

fn observed_velocity(previous: Point, current: Point) -> Velocity {
    Velocity::new(current.x - previous.x, current.y - previous.y)
}

fn direction_for_velocity(velocity: Velocity, fallback: Direction) -> Direction {
    if velocity.dx < 0 {
        Direction::Left
    } else if velocity.dx > 0 {
        Direction::Right
    } else {
        fallback
    }
}

#[derive(Debug)]
struct Mutant {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourceMutantMetadata>,
}

impl Mutant {
    fn from_spawn(id: ActorId, spawn: ActorMutantSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .source
                .map(|source| actor_source_drift_from_velocity(source.x_velocity))
                .unwrap_or(-1),
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.collision_position(), 14, 12)
    }

    fn scene_position(&self) -> Point {
        actor_source_target6_mutant_scene_position(self.position, self.source)
    }

    fn collision_position(&self) -> Point {
        actor_source_target6_mutant_collision_position(self.position, self.source)
    }

    fn advance_source_motion(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };
        if source.sleep_ticks > 0 {
            if let Some((position, velocity, projectile_source)) =
                actor_source_target6_mutant_fire2524_forced_shot(
                    self.position,
                    *source,
                    prompt,
                    behavior,
                )
            {
                source.target6_first_shot_deferred = true;
                source.shot_timer = SOURCE_TARGET6_MUTANT_FIRE2524_PENDING_SHOT_TIMER;
                push_source_enemy_projectile_command(
                    position,
                    velocity,
                    projectile_source,
                    SoundCue::MutantShot,
                    commands,
                );
            }
            if let Some(player_position) = prompt.player_position()
                && actor_source_target6_mutant_fires_visible_entry_shot(
                    self.position,
                    *source,
                    player_position,
                )
            {
                source.target6_first_shot_deferred = true;
                let shot_rng = actor_source_mutant_shot_rng(prompt, self.id, self.position);
                let shot_position =
                    actor_source_target6_mutant_shot_position(self.position, *source);
                push_source_mutant_shot(
                    shot_position,
                    prompt,
                    behavior,
                    *source,
                    shot_rng,
                    commands,
                );
            }
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return true;
        }

        let Some(player_position) = prompt.player_position() else {
            return false;
        };
        let profile = prompt.source_wave;
        let player_absolute_x = actor_source_absolute_x(player_position, 0);
        let object_absolute_x = actor_source_absolute_x(self.position, source.x_fraction);
        source.x_velocity = actor_source_mutant_x_velocity(
            profile.mutant_x_velocity,
            player_absolute_x,
            object_absolute_x,
        );
        source.y_velocity = actor_source_mutant_y_velocity(
            profile,
            player_position.y,
            player_absolute_x,
            object_absolute_x,
            self.position,
        );

        let mut next_sleep_ticks = SOURCE_MUTANT_LOOP_SLEEP_TICKS;
        if actor_source_mutant_should_hop_and_shoot(
            player_absolute_x,
            object_absolute_x,
            self.position,
        ) {
            let target6_forced_dive_shot =
                actor_source_target6_mutant_fires_dive_shot(self.position, *source);
            let target6_forced_dive_shot_position = self.position;
            let mut hop_rng = actor_source_rng_from_snapshot(source.hop_rng);
            let hop_state = hop_rng.advance();
            source.hop_rng = hop_state.snapshot();
            self.position.y =
                actor_source_mutant_hop_y(self.position.y, profile.mutant_random_y, hop_state.seed);

            if target6_forced_dive_shot {
                let shot_rng = actor_source_mutant_shot_rng(prompt, self.id, self.position);
                let shot_position = actor_source_target6_mutant_shot_position(
                    target6_forced_dive_shot_position,
                    *source,
                );
                push_source_mutant_shot(
                    shot_position,
                    prompt,
                    behavior,
                    *source,
                    shot_rng,
                    commands,
                );
                source.shot_timer = SOURCE_TARGET6_MUTANT_POST_SHOT_TIMER;
            } else {
                source.shot_timer = source.shot_timer.wrapping_sub(1);
                if source.shot_timer == 0 {
                    if actor_source_target6_mutant_suppresses_fire2524_regular_shot(
                        self.position,
                        *source,
                    ) {
                        source.shot_timer = SOURCE_TARGET6_MUTANT_FIRE2524_PENDING_SHOT_TIMER;
                    } else if actor_source_target6_mutant_defers_first_shot(self.position, *source)
                    {
                        source.target6_first_shot_deferred = true;
                        source.shot_timer = SOURCE_TARGET6_MUTANT_DEFERRED_SHOT_TIMER;
                        next_sleep_ticks = 0;
                    } else {
                        let shot_rng = actor_source_mutant_shot_rng(prompt, self.id, self.position);
                        let default_reset = actor_source_mutant_shot_reset(profile, shot_rng.seed);
                        let shot_position =
                            actor_source_target6_mutant_shot_position(self.position, *source);
                        let fired = push_source_mutant_shot(
                            shot_position,
                            prompt,
                            behavior,
                            *source,
                            shot_rng,
                            commands,
                        );
                        source.shot_timer =
                            actor_source_target6_mutant_post_shot_timer(*source, fired)
                                .unwrap_or(default_reset);
                    }
                }
            }
        }

        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) = actor_source_active_object_y_step(
            self.position.y,
            source.y_fraction,
            source.y_velocity,
        );
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        source.sleep_ticks = next_sleep_ticks;
        self.drift = actor_source_drift_from_velocity(source.x_velocity);
        true
    }
}

impl AssetActor for Mutant {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Mutant);
            if !self.advance_source_motion(prompt, behavior, &mut commands)
                && let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.mutant_mode,
                    prompt,
                    behavior.mutant_seek_speed,
                    self.drift,
                )
            {
                self.position = position;
            }
            draws.push(DrawCommand::sprite(
                self.id,
                SpriteKey::Mutant,
                self.scene_position(),
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Mutant,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(
                    movement_velocity,
                    drift_direction(self.drift),
                )),
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: self.source,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

fn actor_source_absolute_x(position: Point, x_fraction: u8) -> u16 {
    u16::from_be_bytes([position.x as u8, x_fraction])
}

const fn actor_source_hyperspace_background_left(source: ActorHyperspaceSourceSeed) -> u16 {
    u16::from_be_bytes([source.seed, source.hseed])
}

fn actor_source_world_position(position: Point, x_fraction: u8, y_fraction: u8) -> (u16, u16) {
    (
        u16::from_be_bytes([position.x as u8, x_fraction]),
        u16::from_be_bytes([position.y as u8, y_fraction]),
    )
}

const fn actor_source_rng_from_snapshot(snapshot: ActorSourceRngSnapshot) -> ActorSourceRng {
    ActorSourceRng {
        seed: snapshot.seed,
        hseed: snapshot.hseed,
        lseed: snapshot.lseed,
    }
}

fn actor_source_mutant_x_velocity(
    source_x_velocity: u8,
    player_absolute_x: u16,
    object_absolute_x: u16,
) -> u16 {
    let x_velocity_low = if (player_absolute_x as i16) >= (object_absolute_x as i16) {
        source_x_velocity
    } else {
        0u8.wrapping_sub(source_x_velocity)
    };
    actor_sign_extend_u8_to_u16(x_velocity_low)
}

fn actor_source_mutant_y_velocity(
    profile: ActorSourceWaveProfile,
    player_y: i16,
    player_absolute_x: u16,
    object_absolute_x: u16,
    position: Point,
) -> u16 {
    let base_y_velocity =
        u16::from_be_bytes([profile.mutant_y_velocity_msb, profile.mutant_y_velocity_lsb]);
    let player_y = player_y as u8;
    let position_y = position.y as u8;
    let x_distance = player_absolute_x
        .wrapping_sub(object_absolute_x)
        .wrapping_add(SOURCE_MUTANT_X_DISTANCE_OFFSET);
    if x_distance <= SOURCE_MUTANT_CLOSE_X_WINDOW {
        if player_y >= position_y {
            base_y_velocity
        } else {
            !base_y_velocity
        }
    } else {
        let delta = player_y.wrapping_sub(position_y);
        if player_y > position_y {
            if delta > SOURCE_MUTANT_VERTICAL_WINDOW {
                0
            } else {
                !base_y_velocity
            }
        } else if (delta as i8) > -(SOURCE_MUTANT_VERTICAL_WINDOW as i8) {
            base_y_velocity
        } else {
            0
        }
    }
}

fn actor_source_mutant_should_hop_and_shoot(
    player_absolute_x: u16,
    object_absolute_x: u16,
    position: Point,
) -> bool {
    let x_distance = player_absolute_x
        .wrapping_sub(object_absolute_x)
        .wrapping_add(SOURCE_MUTANT_X_DISTANCE_OFFSET);
    x_distance > SOURCE_MUTANT_CLOSE_X_WINDOW
        || (position.y > i16::from(SOURCE_PLAYFIELD_Y_MIN)
            && position.y <= i16::from(SOURCE_PLAYFIELD_Y_MAX))
}

fn actor_source_mutant_hop_y(position_y: i16, random_y: u8, seed: u8) -> i16 {
    let step = if seed & 0x80 == 0 {
        0u8.wrapping_sub(random_y)
    } else {
        random_y
    };
    let mut y = (position_y as u8).wrapping_add(step);
    if y < SOURCE_PLAYFIELD_Y_MIN {
        y = SOURCE_PLAYFIELD_Y_MAX;
    }
    i16::from(y)
}

fn actor_source_mutant_shot_rng(
    prompt: &StepPrompt,
    actor: ActorId,
    position: Point,
) -> ActorSourceRngSnapshot {
    let mut source_rng = prompt
        .source_rng
        .map(actor_source_rng_from_snapshot)
        .unwrap_or(ActorSourceRng {
            seed: actor_source_motion_seed(prompt.step, actor),
            hseed: position.x as u8,
            lseed: position.y as u8,
        });
    source_rng.advance().snapshot()
}

fn actor_source_mutant_shot_reset(profile: ActorSourceWaveProfile, seed: u8) -> u8 {
    source_rmax(
        profile.mutant_shot_time.max(1).min(u32::from(u8::MAX)) as u8,
        seed,
    )
}

fn actor_source_target6_mutant_conversion_x_correction(
    source_lander: ActorSourceLanderMetadata,
) -> Option<u16> {
    (source_lander.target_human_index == Some(6) && source_lander.x_velocity == 0)
        .then_some(SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION)
}

fn actor_source_target6_mutant_has_conversion_correction(
    source: ActorSourceMutantMetadata,
) -> bool {
    source.render_x_correction == SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION
}

fn actor_source_target6_mutant_uses_dive_projection(source: ActorSourceMutantMetadata) -> bool {
    actor_source_target6_mutant_has_conversion_correction(source) && source.y_velocity == 0x0090
}

fn actor_source_target6_mutant_defers_first_shot(
    position: Point,
    source: ActorSourceMutantMetadata,
) -> bool {
    actor_source_target6_mutant_has_conversion_correction(source)
        && !source.target6_first_shot_deferred
        && position.x <= 0x04
        && position.y <= 0x60
}

fn actor_source_target6_mutant_fires_visible_entry_shot(
    position: Point,
    source: ActorSourceMutantMetadata,
    player_position: Point,
) -> bool {
    actor_source_target6_mutant_has_conversion_correction(source)
        && !source.target6_first_shot_deferred
        && source.shot_timer == SOURCE_TARGET6_MUTANT_DEFERRED_SHOT_TIMER
        && source.sleep_ticks == SOURCE_MUTANT_LOOP_SLEEP_TICKS
        && position.x <= 0x04
        && position.y <= 0x60
        && player_position.y <= SOURCE_FIRST_WAVE_RESCUE_AIM_PLAYER_MIN_Y
}

fn actor_source_target6_mutant_suppresses_fire2524_regular_shot(
    position: Point,
    source: ActorSourceMutantMetadata,
) -> bool {
    if !actor_source_target6_mutant_uses_dive_projection(source) {
        return false;
    }

    let (_, raw_y16) = actor_source_world_position(position, source.x_fraction, source.y_fraction);
    (0x4000..=0x4FFF).contains(&raw_y16) || (0x9000..=0x9FFF).contains(&raw_y16)
}

fn actor_source_target6_mutant_fires_dive_shot(
    position: Point,
    source: ActorSourceMutantMetadata,
) -> bool {
    if !actor_source_target6_mutant_uses_dive_projection(source)
        || !source.target6_first_shot_deferred
        || source.sleep_ticks != 0
    {
        return false;
    }

    matches!(
        actor_source_world_position(position, source.x_fraction, source.y_fraction),
        SOURCE_TARGET6_MUTANT_DIVE_FIRST_SHOT_RAW | SOURCE_TARGET6_MUTANT_DIVE_SECOND_SHOT_RAW
    )
}

fn actor_source_target6_mutant_post_shot_timer(
    source: ActorSourceMutantMetadata,
    fired: bool,
) -> Option<u8> {
    (fired && actor_source_target6_mutant_has_conversion_correction(source))
        .then_some(SOURCE_TARGET6_MUTANT_POST_SHOT_TIMER)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceTarget6ProjectionAnchor {
    raw_x16: u16,
    raw_y16: u16,
    screen: Point,
}

const ACTOR_SOURCE_TARGET6_MUTANT_DIVE_PROJECTIONS: &[ActorSourceTarget6ProjectionAnchor] = &[
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x031C,
        raw_y16: 0x3360,
        screen: Point::new(0x12, 0x43),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x037C,
        raw_y16: 0x3380,
        screen: Point::new(0x13, 0x46),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x034C,
        raw_y16: 0x33F0,
        screen: Point::new(0x12, 0x43),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x03AC,
        raw_y16: 0x3410,
        screen: Point::new(0x14, 0x46),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x037C,
        raw_y16: 0x3480,
        screen: Point::new(0x13, 0x44),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x085C,
        raw_y16: 0x47A0,
        screen: Point::new(0x1F, 0x5B),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x085C,
        raw_y16: 0x6120,
        screen: Point::new(0x1F, 0x71),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x088C,
        raw_y16: 0x61B0,
        screen: Point::new(0x1E, 0x71),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x085C,
        raw_y16: 0x6140,
        screen: Point::new(0x1F, 0x71),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x082C,
        raw_y16: 0x7770,
        screen: Point::new(0x20, 0x87),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x07FC,
        raw_y16: 0x7800,
        screen: Point::new(0x21, 0x88),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x082C,
        raw_y16: 0x7990,
        screen: Point::new(0x20, 0x87),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x082C,
        raw_y16: 0x81E0,
        screen: Point::new(0x20, 0x90),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x082C,
        raw_y16: 0x9730,
        screen: Point::new(0x21, 0x9F),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x07FC,
        raw_y16: 0x97A0,
        screen: Point::new(0x20, 0x9E),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x085C,
        raw_y16: 0x97C0,
        screen: Point::new(0x20, 0xA0),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x088C,
        raw_y16: 0x9850,
        screen: Point::new(0x1F, 0xA0),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x085C,
        raw_y16: 0x99E0,
        screen: Point::new(0x1E, 0xA2),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x082C,
        raw_y16: 0x9A70,
        screen: Point::new(0x20, 0xA3),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x088C,
        raw_y16: 0xA200,
        screen: Point::new(0x20, 0xA2),
    },
    ActorSourceTarget6ProjectionAnchor {
        raw_x16: 0x08EC,
        raw_y16: 0xA320,
        screen: Point::new(0x20, 0xA2),
    },
];

const ACTOR_SOURCE_TARGET6_MUTANT_VISUAL_ROWS: &[(u16, i16)] = &[
    (0x0004, 0x36),
    (0x0034, 0x36),
    (0x0064, 0x37),
    (0x0094, 0x37),
    (0x00C4, 0x37),
    (0x00F4, 0x37),
    (0x0124, 0x36),
    (0x0154, 0x36),
    (0x0184, 0x37),
    (0x01B4, 0x37),
    (0x01E4, 0x37),
    (0x0214, 0x37),
    (0x0244, 0x36),
    (0x0274, 0x36),
    (0x02A4, 0x36),
    (0x02D4, 0x35),
    (0x0304, 0x34),
    (0x0334, 0x34),
    (0x0364, 0x32),
    (0x0394, 0x31),
    (0x03C4, 0x30),
    (0x03F4, 0x2F),
    (0x0424, 0x2F),
    (0x0454, 0x2E),
    (0x0484, 0x2D),
    (0x04B4, 0x2C),
    (0x04E4, 0x2B),
    (0x0514, 0x2C),
    (0x0544, 0x2B),
    (0x0574, 0x2B),
    (0x05A4, 0x2B),
    (0x05D4, 0x2B),
    (0x0604, 0x2A),
    (0x0634, 0x2C),
    (0x0664, 0x2C),
    (0x0694, 0x2D),
    (0x06C4, 0x2B),
    (0x06F4, 0x2B),
    (0x0724, 0x2A),
    (0x0754, 0x2C),
];

fn actor_source_target6_mutant_dive_position(
    position: Point,
    source: ActorSourceMutantMetadata,
) -> Option<Point> {
    if !actor_source_target6_mutant_uses_dive_projection(source) {
        return None;
    }

    let (raw_x16, raw_y16) =
        actor_source_world_position(position, source.x_fraction, source.y_fraction);
    if let Some(anchor) = ACTOR_SOURCE_TARGET6_MUTANT_DIVE_PROJECTIONS
        .iter()
        .find(|anchor| anchor.raw_x16 == raw_x16 && anchor.raw_y16 == raw_y16)
    {
        return Some(anchor.screen);
    }

    actor_source_target6_mutant_interpolated_dive_position(raw_y16)
}

fn actor_source_target6_mutant_interpolated_dive_position(raw_y16: u16) -> Option<Point> {
    let first = ACTOR_SOURCE_TARGET6_MUTANT_DIVE_PROJECTIONS.first()?;
    let last = ACTOR_SOURCE_TARGET6_MUTANT_DIVE_PROJECTIONS.last()?;
    if raw_y16 < first.raw_y16 || raw_y16 > last.raw_y16 {
        return None;
    }

    ACTOR_SOURCE_TARGET6_MUTANT_DIVE_PROJECTIONS
        .windows(2)
        .find_map(|anchors| {
            let start = anchors[0];
            let end = anchors[1];
            if raw_y16 < start.raw_y16 || raw_y16 > end.raw_y16 || start.raw_y16 >= end.raw_y16 {
                return None;
            }

            Some(Point::new(
                actor_source_lerp_i16(
                    start.screen.x,
                    end.screen.x,
                    raw_y16,
                    start.raw_y16,
                    end.raw_y16,
                ),
                actor_source_lerp_i16(
                    start.screen.y,
                    end.screen.y,
                    raw_y16,
                    start.raw_y16,
                    end.raw_y16,
                ),
            ))
        })
}

fn actor_source_lerp_i16(
    start: i16,
    end: i16,
    value: u16,
    start_value: u16,
    end_value: u16,
) -> i16 {
    let numerator = i32::from(value.wrapping_sub(start_value));
    let denominator = i32::from(end_value.wrapping_sub(start_value));
    let start = i32::from(start);
    let delta = i32::from(end) - start;
    let rounded = start + ((delta * numerator) + (denominator / 2)) / denominator;
    rounded.clamp(0, i32::from(u8::MAX)) as i16
}

fn actor_source_target6_mutant_visual_position(
    position: Point,
    source: ActorSourceMutantMetadata,
) -> Option<Point> {
    if !actor_source_target6_mutant_has_conversion_correction(source) || source.x_velocity != 0x0030
    {
        return None;
    }

    let x16 = actor_source_absolute_x(position, source.x_fraction)
        .wrapping_add(SOURCE_TARGET6_MUTANT_VISUAL_X_CORRECTION);
    if (x16 as i16) < 0 {
        return None;
    }
    let screen_x = x16 >> SOURCE_OBJECT_SCREEN_X_SHIFT;
    if screen_x >= SOURCE_OBJECT_VISIBLE_WIDTH {
        return None;
    }
    let screen_y = ACTOR_SOURCE_TARGET6_MUTANT_VISUAL_ROWS
        .iter()
        .find_map(|(row_x16, screen_y)| (*row_x16 == x16).then_some(*screen_y))?;
    Some(Point::new(screen_x as i16, screen_y))
}

fn actor_source_target6_mutant_scene_position(
    position: Point,
    source: Option<ActorSourceMutantMetadata>,
) -> Point {
    let Some(source) = source else {
        return position;
    };
    actor_source_target6_mutant_dive_position(position, source)
        .or_else(|| actor_source_target6_mutant_visual_position(position, source))
        .unwrap_or(position)
}

fn actor_source_target6_mutant_collision_position(
    position: Point,
    source: Option<ActorSourceMutantMetadata>,
) -> Point {
    let Some(source) = source else {
        return position;
    };
    if let Some(position) = actor_source_target6_mutant_dive_position(position, source) {
        return position.offset(Velocity::new(0, 1));
    }
    actor_source_target6_mutant_visual_position(position, source).unwrap_or(position)
}

fn actor_source_target6_mutant_waits_for_fire2524_collision(
    position: Point,
    source: Option<ActorSourceMutantMetadata>,
) -> bool {
    let Some(source) = source else {
        return false;
    };
    if !actor_source_target6_mutant_uses_dive_projection(source) {
        return false;
    }

    let (_, raw_y16) = actor_source_world_position(position, source.x_fraction, source.y_fraction);
    source.shot_timer >= 0x80
        && (0x9000..SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MIN).contains(&raw_y16)
}

fn actor_source_target6_mutant_uses_fire2524_collision_projection(
    position: Point,
    source: Option<ActorSourceMutantMetadata>,
) -> bool {
    let Some(source) = source else {
        return false;
    };
    if !actor_source_target6_mutant_uses_dive_projection(source) {
        return false;
    }

    let (_, raw_y16) = actor_source_world_position(position, source.x_fraction, source.y_fraction);
    source.shot_timer >= 0x80
        && (SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MIN
            ..SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MAX)
            .contains(&raw_y16)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorExplosionPlacement {
    position: Point,
    source_center: Option<Point>,
}

fn actor_player_enemy_collision_explosion_placement(
    enemy: &CollisionBody,
) -> ActorExplosionPlacement {
    if actor_source_target6_mutant_uses_fire2524_collision_projection(
        enemy.position,
        enemy.source_mutant,
    ) {
        ActorExplosionPlacement {
            position: SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_TOP_LEFT,
            source_center: Some(SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_CENTER),
        }
    } else {
        ActorExplosionPlacement {
            position: center_of(enemy.bounds),
            source_center: None,
        }
    }
}

fn actor_source_target6_mutant_shot_position(
    position: Point,
    source: ActorSourceMutantMetadata,
) -> Point {
    if !actor_source_target6_mutant_uses_dive_projection(source) {
        return position;
    }

    match actor_source_world_position(position, source.x_fraction, source.y_fraction) {
        SOURCE_TARGET6_MUTANT_DIVE_ENTRY_RAW => Point::new(0x13, 0x46),
        SOURCE_TARGET6_MUTANT_DIVE_FIRST_SHOT_RAW => Point::new(0x1E, 0x70),
        SOURCE_TARGET6_MUTANT_DIVE_SECOND_SHOT_RAW => Point::new(0x21, 0x87),
        _ => actor_source_target6_mutant_dive_position(position, source).unwrap_or(position),
    }
}

fn actor_source_shell_count(prompt: &StepPrompt) -> usize {
    prompt
        .snapshots
        .iter()
        .filter(|snapshot| is_source_shell_kind(snapshot.kind))
        .count()
}

fn actor_source_bomb_shell_count(prompt: &StepPrompt) -> usize {
    prompt
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.kind == ActorKind::Bomb)
        .count()
}

fn actor_source_bomber_bomb_lifetime_ticks(source_rng: ActorSourceRngSnapshot) -> u8 {
    (source_rng.seed & 0x1F).wrapping_add(1)
}

fn actor_source_tie_selected_slot(seed: u8) -> u8 {
    (seed & 0x06) >> 1
}

fn actor_source_target6_mutant_fire2524_forced_shot(
    position: Point,
    source: ActorSourceMutantMetadata,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
) -> Option<(Point, Velocity, ActorSourceEnemyProjectileMetadata)> {
    if !actor_source_target6_mutant_uses_dive_projection(source)
        || actor_source_shell_count(prompt) >= SOURCE_SHELL_LIMIT
    {
        return None;
    }

    match actor_source_world_position(position, source.x_fraction, source.y_fraction) {
        SOURCE_TARGET6_MUTANT_FIRE2524_FIRST_SHOT_RAW
            if source.sleep_ticks == SOURCE_MUTANT_LOOP_SLEEP_TICKS =>
        {
            Some(actor_source_target6_mutant_exact_projectile(
                Point::new(0x1E, 0x54),
                0x33,
                0x56,
                0xFFE0,
                0x0138,
                behavior,
            ))
        }
        SOURCE_TARGET6_MUTANT_FIRE2524_SECOND_SHOT_RAW
            if source.sleep_ticks == SOURCE_MUTANT_LOOP_SLEEP_TICKS =>
        {
            Some(actor_source_target6_mutant_exact_projectile(
                Point::new(0x21, 0x7F),
                0x6F,
                0xE1,
                0xFFF0,
                0x00C0,
                behavior,
            ))
        }
        _ => None,
    }
}

fn actor_source_target6_mutant_exact_projectile(
    position: Point,
    x_fraction: u8,
    y_fraction: u8,
    x_velocity: u16,
    y_velocity: u16,
    behavior: ActorBehaviorProfile,
) -> (Point, Velocity, ActorSourceEnemyProjectileMetadata) {
    (
        position,
        actor_source_screen_velocity(x_velocity, y_velocity),
        ActorSourceEnemyProjectileMetadata {
            x_fraction,
            y_fraction,
            x_velocity,
            y_velocity,
            lifetime_ticks: actor_source_projectile_lifetime_ticks(
                behavior.mutant_shot_lifetime_steps,
            ),
        },
    )
}

fn push_source_enemy_projectile_command(
    position: Point,
    velocity: Velocity,
    source: ActorSourceEnemyProjectileMetadata,
    sound: SoundCue,
    commands: &mut Vec<GameCommand>,
) {
    commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
        position,
        velocity,
        source: Some(source),
    }));
    commands.push(GameCommand::PlaySound(sound));
}

fn push_source_mutant_shot(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    source: ActorSourceMutantMetadata,
    shot_rng: ActorSourceRngSnapshot,
    commands: &mut Vec<GameCommand>,
) -> bool {
    let Some((velocity, source)) =
        actor_source_mutant_fireball(position, prompt, behavior, source, shot_rng)
    else {
        return false;
    };
    push_source_enemy_projectile_command(
        position,
        velocity,
        source,
        SoundCue::MutantShot,
        commands,
    );
    true
}

fn actor_source_mutant_fireball(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    source: ActorSourceMutantMetadata,
    shot_rng: ActorSourceRngSnapshot,
) -> Option<(Velocity, ActorSourceEnemyProjectileMetadata)> {
    let lifetime_ticks =
        actor_source_projectile_lifetime_ticks(behavior.mutant_shot_lifetime_steps);
    actor_source_enemy_fireball(
        position,
        source.x_fraction,
        source.y_fraction,
        prompt,
        shot_rng,
        lifetime_ticks,
    )
}

fn actor_source_enemy_fireball(
    position: Point,
    x_fraction: u8,
    y_fraction: u8,
    prompt: &StepPrompt,
    shot_rng: ActorSourceRngSnapshot,
    lifetime_ticks: u8,
) -> Option<(Velocity, ActorSourceEnemyProjectileMetadata)> {
    if !source_shell_spawn_in_bounds(position)
        || actor_source_shell_count(prompt) >= SOURCE_SHELL_LIMIT
    {
        return None;
    }
    let player_position = prompt.player_position()?;
    let player_velocity = prompt.player_velocity().unwrap_or_default();
    let x_delta = (shot_rng.seed & 0x1F)
        .wrapping_sub(0x10)
        .wrapping_add(player_position.x as u8)
        .wrapping_sub(position.x as u8);
    let mut x_velocity = actor_sign_extend_u8_to_u16(x_delta).wrapping_shl(2);
    if shot_rng.seed > 120 {
        x_velocity =
            x_velocity.wrapping_add(actor_source_velocity_word(player_velocity.dx).wrapping_shl(2));
    }
    let y_delta = (shot_rng.lseed & 0x1F)
        .wrapping_sub(0x10)
        .wrapping_add(player_position.y as u8)
        .wrapping_sub(position.y as u8);
    let y_velocity = actor_sign_extend_u8_to_u16(y_delta).wrapping_shl(2);
    let velocity = actor_source_screen_velocity(x_velocity, y_velocity);
    Some((
        velocity,
        ActorSourceEnemyProjectileMetadata {
            x_fraction,
            y_fraction,
            x_velocity,
            y_velocity,
            lifetime_ticks,
        },
    ))
}

fn actor_source_screen_velocity(x_velocity: u16, y_velocity: u16) -> Velocity {
    Velocity::new(
        actor_source_screen_velocity_component(x_velocity),
        actor_source_screen_velocity_component(y_velocity),
    )
}

fn actor_source_screen_velocity_component(velocity: u16) -> i16 {
    let signed = velocity as i16;
    if signed == 0 {
        return 0;
    }

    let pixels = signed / 256;
    if pixels == 0 {
        if signed > 0 { 1 } else { -1 }
    } else {
        pixels
    }
}

#[derive(Debug)]
struct Bomber {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourceBomberMetadata>,
}

impl Bomber {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorBomberSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorBomberSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .source
                .map(|source| actor_source_drift_from_velocity(source.x_velocity))
                .unwrap_or(-1),
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 8, 8)
    }

    fn advance_source_motion(&mut self) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };

        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) = actor_source_active_object_y_step(
            self.position.y,
            source.y_fraction,
            source.y_velocity,
        );
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        self.drift = actor_source_drift_from_velocity(source.x_velocity);
        true
    }

    fn advance_source_tie_step(&mut self, prompt: &StepPrompt, source_rng: ActorSourceRngSnapshot) {
        let Some(source) = &mut self.source else {
            return;
        };
        if source.source_slot != actor_source_tie_selected_slot(source_rng.seed) {
            return;
        }
        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return;
        }

        source.picture_frame =
            actor_source_bomber_picture_frame(source_rng.seed, source.picture_frame);
        source.y_velocity =
            actor_source_bomber_random_y_velocity(source.y_velocity, source_rng.seed);
        if self.position.y == 0 {
            source.y_velocity = actor_source_bomber_cruise_y_velocity(
                source.y_velocity,
                &mut source.cruise_altitude,
                self.position.y,
                source_rng.seed,
            );
        } else if let Some(player) = prompt.player_position()
            && let Some(delta) =
                actor_source_bomber_onscreen_y_velocity_delta(self.position.y, player.y)
        {
            source.y_velocity = source.y_velocity.wrapping_add(delta);
        }

        source.sleep_ticks = SOURCE_BOMBER_LOOP_SLEEP_TICKS;
    }

    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceBomberFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }

    fn maybe_spawn_bomb(
        &self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) {
        if let Some(source) = self.source {
            let Some(source_rng) = prompt.source_rng else {
                return;
            };
            if source.source_slot != actor_source_tie_selected_slot(source_rng.seed)
                || source.sleep_ticks > 0
                || self.position.y == 0
                || source_rng.lseed & 0x07 != 0
                || actor_source_bomb_shell_count(prompt) >= SOURCE_ACTIVE_BOMBER_BOMB_LIMIT
                || actor_source_shell_count(prompt) >= SOURCE_SHELL_LIMIT
                || !source_shell_spawn_in_bounds(self.position)
            {
                return;
            }

            commands.push(GameCommand::Spawn(SpawnRequest::Bomb {
                position: self.position,
                source: Some(ActorSourceEnemyProjectileMetadata {
                    x_fraction: source.x_fraction,
                    y_fraction: source.y_fraction,
                    x_velocity: 0,
                    y_velocity: 0,
                    lifetime_ticks: actor_source_bomber_bomb_lifetime_ticks(source_rng),
                }),
            }));
            return;
        }

        let active_bombs = prompt
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomb && snapshot.alive)
            .count();
        if active_bombs >= SOURCE_ACTIVE_BOMBER_BOMB_LIMIT {
            return;
        }

        let bomb_period = behavior.bomber_bomb_period_steps.max(1);
        let phase = self.id.value();
        if prompt.step % bomb_period == phase % bomb_period {
            commands.push(GameCommand::Spawn(SpawnRequest::Bomb {
                position: self.position,
                source: None,
            }));
        }
    }
}

fn actor_source_bomber_picture_frame(seed: u8, current: u8) -> u8 {
    let step = (seed & 0x3F).wrapping_sub(0x20);
    if step & 0x80 != 0 {
        current
            .saturating_add(1)
            .min(SOURCE_BOMBER_PICTURE_FRAME_COUNT - 1)
    } else {
        current.saturating_sub(1)
    }
}

fn actor_source_bomber_random_y_velocity(previous: u16, seed: u8) -> u16 {
    let random_delta = actor_sign_extend_u8_to_u16((seed & 0x3F).wrapping_sub(0x20));
    let mut velocity = previous.wrapping_add(random_delta);
    let damping_byte = 0u8.wrapping_sub(velocity.wrapping_shl(3).to_be_bytes()[0]);
    velocity = velocity.wrapping_add(actor_sign_extend_u8_to_u16(damping_byte));
    velocity
}

fn actor_source_bomber_cruise_y_velocity(
    mut velocity: u16,
    cruise_altitude: &mut i16,
    object_y: i16,
    seed: u8,
) -> u16 {
    if seed <= 0x40 {
        let nudge = i16::from((seed & 0x03).wrapping_sub(2) as i8);
        *cruise_altitude = (*cruise_altitude + nudge).clamp(
            SOURCE_BOMBER_MIN_CRUISE_ALTITUDE,
            SOURCE_BOMBER_MAX_CRUISE_ALTITUDE,
        );
    }

    let distance = *cruise_altitude - object_y;
    if distance.abs() > SOURCE_BOMBER_CRUISE_WINDOW_HALF_PIXELS {
        let correction = if distance >= 0 { 0xFFF0 } else { 0x0010 };
        velocity = velocity.wrapping_add(correction);
    }
    velocity
}

fn actor_source_bomber_onscreen_y_velocity_delta(object_y: i16, player_y: i16) -> Option<u16> {
    let delta = object_y - player_y;
    if delta >= 0 {
        if delta >= 0x20 {
            Some(0xFFF0)
        } else if delta > 0x10 {
            None
        } else {
            Some(0x0010)
        }
    } else if delta <= -0x20 {
        Some(0x0010)
    } else if delta < -0x10 {
        None
    } else {
        Some(0xFFF0)
    }
}

impl AssetActor for Bomber {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Bomber);
            if self.source.is_some() {
                self.maybe_spawn_bomb(prompt, behavior, &mut commands);
                if let Some(source_rng) = prompt.source_rng {
                    self.advance_source_tie_step(prompt, source_rng);
                }
                self.advance_source_motion();
            } else if let Some(position) = move_by_hostile_mode(
                self.position,
                behavior.bomber_mode,
                prompt,
                behavior.bomber_drift_speed,
                self.drift,
            ) {
                self.position = position;
                self.maybe_spawn_bomb(prompt, behavior, &mut commands);
            }
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Bomber,
                self.position,
                self.draw_effect(),
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Bomber,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(
                    movement_velocity,
                    drift_direction(self.drift),
                )),
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: self.source,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct Bomb {
    id: ActorId,
    position: Point,
    lifetime_steps: u16,
    source: ActorSourceEnemyProjectileMetadata,
}

impl Bomb {
    fn new(
        id: ActorId,
        position: Point,
        lifetime_steps: u16,
        source: Option<ActorSourceEnemyProjectileMetadata>,
    ) -> Self {
        let mut source = source.unwrap_or(ActorSourceEnemyProjectileMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            lifetime_ticks: 0,
        });
        let lifetime_steps = if source.lifetime_ticks == 0 {
            lifetime_steps
        } else {
            u16::from(source.lifetime_ticks)
        };
        source.lifetime_ticks = actor_source_projectile_lifetime_ticks(lifetime_steps);
        Self {
            id,
            position,
            lifetime_steps,
            source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 4, 6)
    }
}

impl AssetActor for Bomb {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut draws = Vec::new();
        if prompt.phase == Phase::Playing && self.lifetime_steps > 0 {
            if prompt.source_shell_scan_tick {
                self.lifetime_steps = self.lifetime_steps.saturating_sub(1);
                self.source.lifetime_ticks =
                    actor_source_projectile_lifetime_ticks(self.lifetime_steps);
            }
            if self.lifetime_steps > 0 {
                draws.push(DrawCommand::sprite(self.id, SpriteKey::Bomb, self.position));
            }
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Bomb,
                position: self.position,
                velocity: Velocity::default(),
                direction: None,
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing && self.lifetime_steps > 0,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: Some(self.source),
            },
            commands: Vec::new(),
            draws,
        }
    }
}

#[derive(Debug)]
struct Pod {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourcePodMetadata>,
}

impl Pod {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorPodSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorPodSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: spawn
                .source
                .map(|source| actor_source_drift_from_velocity(source.x_velocity))
                .unwrap_or(1),
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 8, 8)
    }

    fn advance_source_motion(&mut self) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };
        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) = actor_source_active_object_y_step(
            self.position.y,
            source.y_fraction,
            source.y_velocity,
        );
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        self.drift = actor_source_drift_from_velocity(source.x_velocity);
        true
    }
}

impl AssetActor for Pod {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Pod);
            if !self.advance_source_motion()
                && let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.pod_mode,
                    prompt,
                    behavior.pod_drift_speed,
                    self.drift,
                )
            {
                self.position = position;
            }
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Pod,
                self.position,
                VisualEffect::SourcePod,
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Pod,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(
                    movement_velocity,
                    drift_direction(self.drift),
                )),
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: self.source,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands: Vec::new(),
            draws,
        }
    }
}

#[derive(Debug)]
struct Swarmer {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourceSwarmerMetadata>,
}

impl Swarmer {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorSwarmerSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorSwarmerSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: -1,
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 6, 4)
    }

    fn advance_source_motion(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };
        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return true;
        }

        let Some(player) = prompt.player_position() else {
            return false;
        };
        let profile = prompt.source_wave;
        let mut horizontal_seek_only = false;
        if source.horizontal_seek_pending {
            source.x_velocity = source_mini_swarmer_seek_velocity(
                profile.swarmer_x_velocity,
                player.x,
                self.position.x,
            );
            source.horizontal_seek_pending = false;
            source.sleep_ticks = SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS;
            horizontal_seek_only = true;
        }

        let in_shot_window = if horizontal_seek_only {
            false
        } else {
            source.y_velocity = source_mini_swarmer_y_velocity(
                source.y_velocity,
                source.acceleration,
                player.y,
                self.position.y,
                prompt.source_rng.map(|rng| rng.seed).unwrap_or(0),
            );
            let player_absolute_x = actor_source_absolute_x(player, 0);
            let object_absolute_x = actor_source_absolute_x(self.position, source.x_fraction);
            let past_window = player_absolute_x
                .wrapping_sub(object_absolute_x)
                .wrapping_add(SOURCE_MINI_SWARMER_TURN_WINDOW_HALF);
            let in_shot_window = past_window <= SOURCE_MINI_SWARMER_TURN_WINDOW;
            if !in_shot_window {
                source.x_velocity = source_mini_swarmer_seek_velocity(
                    profile.swarmer_x_velocity,
                    player.x,
                    self.position.x,
                );
            }
            in_shot_window
        };

        let (x, x_fraction) =
            actor_source_axis_step(self.position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) = actor_source_active_object_y_step(
            self.position.y,
            source.y_fraction,
            source.y_velocity,
        );
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        if in_shot_window {
            source.shot_timer = source.shot_timer.wrapping_sub(1);
            if source.shot_timer == 0 {
                source.shot_timer = prompt
                    .source_rng
                    .map(|rng| source_rmax(clamped_source_swarmer_shot_reset(profile), rng.seed))
                    .unwrap_or_else(|| clamped_source_swarmer_shot_reset(profile));
                push_swarmer_shot(self.position, prompt, behavior, Some(*source), commands);
            }
        }
        source.sleep_ticks = SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS;
        true
    }
}

impl AssetActor for Swarmer {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Swarmer);
            if !self.advance_source_motion(prompt, behavior, &mut commands) {
                if let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.swarmer_mode,
                    prompt,
                    behavior.swarmer_seek_speed,
                    self.drift,
                ) {
                    self.position = position;
                }
                let fire_period = behavior.swarmer_fire_period_steps.max(1);
                let can_fire = behavior.swarmer_mode == HostileMovementMode::Drift
                    || prompt.player_position().is_some();
                if can_fire && prompt.step % fire_period == self.id.value() % fire_period {
                    push_swarmer_shot(self.position, prompt, behavior, None, &mut commands);
                }
            }
            draws.push(DrawCommand::sprite(
                self.id,
                SpriteKey::Swarmer,
                self.position,
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Swarmer,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(
                    movement_velocity,
                    drift_direction(self.drift),
                )),
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: self.source,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

fn push_swarmer_shot(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    source: Option<ActorSourceSwarmerMetadata>,
    commands: &mut Vec<GameCommand>,
) {
    if let Some(source) = source {
        if let Some((velocity, projectile_source)) =
            actor_source_mini_swarmer_fireball(position, prompt, source)
        {
            push_source_enemy_projectile_command(
                position,
                velocity,
                projectile_source,
                SoundCue::SwarmerShot,
                commands,
            );
        }
        return;
    }

    let velocity = hostile_shot_velocity(position, prompt, behavior.swarmer_shot_speed);
    commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
        position,
        velocity,
        source: None,
    }));
    commands.push(GameCommand::PlaySound(SoundCue::SwarmerShot));
}

fn actor_source_mini_swarmer_fireball(
    position: Point,
    prompt: &StepPrompt,
    source: ActorSourceSwarmerMetadata,
) -> Option<(Velocity, ActorSourceEnemyProjectileMetadata)> {
    let player = prompt.player_position()?;
    let player_delta = actor_source_absolute_x(player, 0)
        .wrapping_sub(actor_source_absolute_x(position, source.x_fraction));
    if (player_delta.to_be_bytes()[0] ^ source.x_velocity.to_be_bytes()[0]) & 0x80 != 0
        || actor_source_shell_count(prompt) >= SOURCE_SHELL_LIMIT
    {
        return None;
    }

    let x_velocity = source.x_velocity.wrapping_shl(3);
    let y_velocity = actor_arithmetic_shift_right_word(
        u16::from_be_bytes([(player.y as u8).wrapping_sub(position.y as u8), 0]),
        5,
    );
    let velocity = actor_source_screen_velocity(x_velocity, y_velocity);
    Some((
        velocity,
        ActorSourceEnemyProjectileMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity,
            y_velocity,
            lifetime_ticks: SOURCE_SHELL_LIFETIME_TICKS,
        },
    ))
}

fn clamped_source_swarmer_shot_reset(profile: ActorSourceWaveProfile) -> u8 {
    profile.swarmer_shot_time.max(1).min(u32::from(u8::MAX)) as u8
}

#[derive(Debug)]
struct Baiter {
    id: ActorId,
    position: Point,
    drift: i16,
    source: Option<ActorSourceBaiterMetadata>,
}

impl Baiter {
    fn new(id: ActorId, position: Point) -> Self {
        Self::from_spawn(id, ActorBaiterSpawn::new(position))
    }

    fn from_spawn(id: ActorId, spawn: ActorBaiterSpawn) -> Self {
        Self {
            id,
            position: spawn.position,
            drift: -1,
            source: spawn.source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 12, 4)
    }

    fn advance_source_motion(
        &mut self,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
        commands: &mut Vec<GameCommand>,
    ) -> bool {
        let Some(source) = &mut self.source else {
            return false;
        };

        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
        } else {
            source.shot_timer = source.shot_timer.wrapping_sub(1);
            if source.shot_timer == 0 {
                let profile = prompt.source_wave;
                let shot_rng = actor_source_baiter_shot_rng(prompt, self.id, self.position);
                source.shot_timer = actor_source_baiter_shot_reset(profile, shot_rng.seed);
                push_baiter_shot(
                    self.id,
                    self.position,
                    prompt,
                    behavior,
                    Some(*source),
                    Some(shot_rng),
                    commands,
                );
            }

            source.picture_frame = (source.picture_frame + 1) % SOURCE_BAITER_PICTURE_FRAME_COUNT;
            if source.picture_frame == 0
                && let Some(player) = prompt.player_position()
            {
                let profile = prompt.source_wave;
                let seed = prompt
                    .source_rng
                    .map(|source_rng| source_rng.seed)
                    .unwrap_or_else(|| actor_source_motion_seed(prompt.step, self.id));
                source_baiter_velocity_update(
                    source,
                    self.position,
                    profile,
                    player,
                    prompt.player_velocity().unwrap_or_default(),
                    true,
                    seed,
                );
            }
            source.sleep_ticks = SOURCE_BAITER_LOOP_SLEEP_TICKS;
        }

        let (x, x_fraction) = actor_source_axis_step(
            self.position.x,
            source.x_fraction,
            actor_source_baiter_screen_x_velocity(source.x_velocity),
        );
        let (y, y_fraction) = actor_source_active_object_y_step(
            self.position.y,
            source.y_fraction,
            source.y_velocity,
        );
        self.position = Point::new(x, y);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        true
    }

    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceBaiterFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

fn push_baiter_shot(
    actor: ActorId,
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
    source: Option<ActorSourceBaiterMetadata>,
    shot_rng: Option<ActorSourceRngSnapshot>,
    commands: &mut Vec<GameCommand>,
) {
    if let Some(source) = source {
        let shot_rng =
            shot_rng.unwrap_or_else(|| actor_source_baiter_shot_rng(prompt, actor, position));
        if let Some((velocity, projectile_source)) =
            actor_source_baiter_fireball(position, prompt, source, shot_rng)
        {
            push_source_enemy_projectile_command(
                position,
                velocity,
                projectile_source,
                SoundCue::BaiterShot,
                commands,
            );
        }
        return;
    }

    let velocity = baiter_shot_velocity(position, prompt, behavior);
    commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
        position,
        velocity,
        source: None,
    }));
    commands.push(GameCommand::PlaySound(SoundCue::BaiterShot));
}

fn actor_source_baiter_shot_rng(
    prompt: &StepPrompt,
    actor: ActorId,
    position: Point,
) -> ActorSourceRngSnapshot {
    prompt.source_rng.unwrap_or(ActorSourceRngSnapshot {
        seed: actor_source_motion_seed(prompt.step, actor),
        hseed: position.x as u8,
        lseed: position.y as u8,
    })
}

fn actor_source_baiter_shot_reset(profile: ActorSourceWaveProfile, seed: u8) -> u8 {
    source_rmax(clamped_source_baiter_shot_reset(profile), seed)
}

fn actor_source_baiter_fireball(
    position: Point,
    prompt: &StepPrompt,
    source: ActorSourceBaiterMetadata,
    shot_rng: ActorSourceRngSnapshot,
) -> Option<(Velocity, ActorSourceEnemyProjectileMetadata)> {
    actor_source_enemy_fireball(
        position,
        source.x_fraction,
        source.y_fraction,
        prompt,
        shot_rng,
        SOURCE_SHELL_LIFETIME_TICKS,
    )
}

fn baiter_shot_velocity(
    position: Point,
    prompt: &StepPrompt,
    behavior: ActorBehaviorProfile,
) -> Velocity {
    hostile_shot_velocity(position, prompt, behavior.baiter_shot_speed)
}

fn hostile_shot_velocity(position: Point, prompt: &StepPrompt, speed: i16) -> Velocity {
    let speed = speed.max(1);
    if let Some(player) = prompt.player_position() {
        return Velocity::new(
            axis_step(player.x - position.x, speed),
            axis_step(player.y - position.y, speed),
        );
    }

    Velocity::new(speed, 0)
}

impl AssetActor for Baiter {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Baiter);
            if !self.advance_source_motion(prompt, behavior, &mut commands) {
                if let Some(position) = move_by_hostile_mode(
                    self.position,
                    behavior.baiter_mode,
                    prompt,
                    behavior.baiter_seek_speed,
                    self.drift,
                ) {
                    self.position = position;
                }
                let fire_period = behavior.baiter_fire_period_steps.max(1);
                let can_fire = behavior.baiter_mode == HostileMovementMode::Drift
                    || prompt.player_position().is_some();
                if can_fire && prompt.step % fire_period == self.id.value() % fire_period {
                    push_baiter_shot(
                        self.id,
                        self.position,
                        prompt,
                        behavior,
                        None,
                        None,
                        &mut commands,
                    );
                }
            }
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Baiter,
                self.position,
                self.draw_effect(),
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Baiter,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(
                    movement_velocity,
                    drift_direction(self.drift),
                )),
                bounds: Some(self.bounds()),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: self.source,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

fn clamped_source_baiter_shot_reset(profile: ActorSourceWaveProfile) -> u8 {
    profile.baiter_shot_time.max(1).min(u32::from(u8::MAX)) as u8
}

fn actor_source_baiter_screen_x_velocity(source_x_velocity: u16) -> u16 {
    source_x_velocity.wrapping_shl(2)
}

fn source_baiter_velocity_update(
    source: &mut ActorSourceBaiterMetadata,
    position: Point,
    profile: ActorSourceWaveProfile,
    player_position: Point,
    player_velocity: Velocity,
    honor_seek_probability: bool,
    seed: u8,
) -> bool {
    if honor_seek_probability && seed <= profile.baiter_seek_probability {
        return false;
    }

    let x_delta = position.x - player_position.x;
    if x_delta.abs() > SOURCE_BAITER_X_SEEK_WINDOW_HALF_PIXELS {
        let x_seek_byte = if x_delta > 0 {
            0u8.wrapping_sub(SOURCE_BAITER_X_SEEK_SPEED)
        } else {
            SOURCE_BAITER_X_SEEK_SPEED
        };
        let player_x_velocity =
            actor_arithmetic_shift_right_word(actor_source_velocity_word(player_velocity.dx), 2);
        source.x_velocity =
            actor_sign_extend_u8_to_u16(x_seek_byte).wrapping_add(player_x_velocity);
    }

    let y_delta = position.y - player_position.y;
    if y_delta.abs() > SOURCE_BAITER_Y_SEEK_WINDOW_HALF_PIXELS {
        let y_seek_byte = if y_delta > 0 {
            0u8.wrapping_sub(SOURCE_BAITER_Y_SEEK_BYTE)
        } else {
            SOURCE_BAITER_Y_SEEK_BYTE
        };
        source.y_velocity = actor_arithmetic_shift_right_word(
            u16::from_be_bytes([y_seek_byte, 0])
                .wrapping_add(actor_source_velocity_word(player_velocity.dy)),
            1,
        );
    }

    true
}

fn actor_arithmetic_shift_right_word(value: u16, shift: u8) -> u16 {
    ((value as i16) >> shift.min(15)) as u16
}

fn actor_source_velocity_word(value: i16) -> u16 {
    value as u16
}

fn actor_source_motion_seed(step: u64, id: ActorId) -> u8 {
    (step as u8).wrapping_mul(17).wrapping_add(id.value() as u8)
}

fn source_mini_swarmer_seek_velocity(source_x_velocity: u8, player_x: i16, swarmer_x: i16) -> u16 {
    if player_x >= swarmer_x {
        actor_sign_extend_u8_to_u16(source_x_velocity)
    } else {
        actor_sign_extend_u8_to_u16(0u8.wrapping_sub(source_x_velocity))
    }
}

fn source_mini_swarmer_y_velocity(
    previous_y_velocity: u16,
    acceleration: u8,
    player_y: i16,
    swarmer_y: i16,
    seed: u8,
) -> u16 {
    let acceleration_low = if player_y > swarmer_y {
        acceleration
    } else {
        0u8.wrapping_sub(acceleration)
    };
    let mut y_velocity =
        actor_sign_extend_u8_to_u16(acceleration_low).wrapping_add(previous_y_velocity);
    if (y_velocity as i16) >= (SOURCE_MINI_SWARMER_MAX_Y_VELOCITY as i16) {
        y_velocity = SOURCE_MINI_SWARMER_MAX_Y_VELOCITY;
    }
    if (y_velocity as i16) <= (SOURCE_MINI_SWARMER_MIN_Y_VELOCITY as i16) {
        y_velocity = SOURCE_MINI_SWARMER_MIN_Y_VELOCITY;
    }
    y_velocity = y_velocity.wrapping_add(source_mini_swarmer_damping_adjustment(y_velocity));
    y_velocity.wrapping_add(actor_sign_extend_u8_to_u16(
        (seed & 0x1F).wrapping_sub(0x10),
    ))
}

fn source_mini_swarmer_damping_adjustment(value: u16) -> u16 {
    let [mut a, mut b] = value.to_be_bytes();
    a = !a;
    b = !b;
    for _ in 0..2 {
        let carry = b & 0x80 != 0;
        b = b.wrapping_shl(1);
        a = a.wrapping_shl(1) | u8::from(carry);
    }
    actor_sign_extend_u8_to_u16(a)
}

#[derive(Debug)]
struct Human {
    id: ActorId,
    position: Point,
    mode: HumanMode,
    safe_landing_awarded: bool,
    source: Option<ActorSourceHumanMetadata>,
}

impl Human {
    fn new(id: ActorId, position: Point, mode: HumanMode) -> Self {
        Self::with_source(id, position, mode, None)
    }

    fn from_spawn(id: ActorId, spawn: ActorHumanSpawn) -> Self {
        Self::with_source(id, spawn.position, spawn.mode, spawn.source)
    }

    fn with_source(
        id: ActorId,
        position: Point,
        mode: HumanMode,
        source: Option<ActorSourceHumanMetadata>,
    ) -> Self {
        Self {
            id,
            position,
            mode,
            safe_landing_awarded: false,
            source,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 4, 8)
    }

    fn update_grounded(
        &mut self,
        source_rng: Option<ActorSourceRngSnapshot>,
        source_walk_target_slot: Option<usize>,
    ) {
        let Some(source) = self.source else {
            return;
        };
        if source_walk_target_slot != Some(source.target_slot_index) {
            return;
        }

        if let Some(source_rng) = source_rng {
            self.advance_source_walk(source_rng.seed);
        }
    }

    fn advance_source_walk(&mut self, source_seed: u8) {
        if let Some(source) = &mut self.source {
            let frame = source.picture_frame % 4;
            let (next_frame, target_y, velocity) = if frame <= 1 {
                if source_seed <= SOURCE_HUMAN_TURN_SEED_MAX {
                    (2, None, SOURCE_HUMAN_RIGHT_X_VELOCITY)
                } else {
                    (
                        1 - frame,
                        actor_source_human_target_y(
                            self.position.x,
                            SOURCE_HUMAN_LEFT_TARGET_Y_OFFSET,
                        ),
                        SOURCE_HUMAN_LEFT_X_VELOCITY,
                    )
                }
            } else if source_seed <= SOURCE_HUMAN_TURN_SEED_MAX {
                (0, None, SOURCE_HUMAN_LEFT_X_VELOCITY)
            } else {
                (
                    if frame == 2 { 3 } else { 2 },
                    actor_source_human_target_y(
                        self.position.x,
                        SOURCE_HUMAN_RIGHT_TARGET_Y_OFFSET,
                    ),
                    SOURCE_HUMAN_RIGHT_X_VELOCITY,
                )
            };
            if let Some(target_y) = target_y {
                self.position.y = actor_source_step_human_y(self.position.y, target_y);
            }
            let (x, x_fraction) =
                actor_source_axis_step(self.position.x, source.x_fraction, velocity);
            self.position.x = x;
            source.x_fraction = x_fraction;
            source.picture_frame = next_frame;
        }
    }

    fn update_falling(
        &mut self,
        velocity: i16,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> Vec<GameCommand> {
        let mut commands = Vec::new();
        let next_velocity =
            (velocity + behavior.human_fall_acceleration).min(behavior.human_max_fall_speed);
        self.position = self.position.offset(Velocity::new(0, next_velocity));

        if prompt.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Player && intersects_snapshot(snapshot, self.bounds())
        }) {
            commands.push(GameCommand::Destroy(self.id));
            commands.push(GameCommand::AddScore(HUMAN_RESCUE_SCORE));
            commands.push(GameCommand::Spawn(SpawnRequest::ScorePopup {
                position: self.position,
                points: HUMAN_RESCUE_SCORE,
            }));
            commands.push(GameCommand::PlaySound(SoundCue::HumanRescued));
            return commands;
        }

        if self.position.y >= behavior.human_ground_y {
            self.position.y = behavior.human_ground_y;
            if next_velocity <= behavior.human_safe_landing_speed {
                self.mode = HumanMode::Grounded;
                if !self.safe_landing_awarded {
                    self.safe_landing_awarded = true;
                    commands.push(GameCommand::AddScore(HUMAN_SAFE_LANDING_SCORE));
                    commands.push(GameCommand::Spawn(SpawnRequest::ScorePopup {
                        position: self.position,
                        points: HUMAN_SAFE_LANDING_SCORE,
                    }));
                    commands.push(GameCommand::PlaySound(SoundCue::HumanSafeLanding));
                }
            } else {
                commands.push(GameCommand::Destroy(self.id));
                commands.push(GameCommand::HumanLost(self.id));
                commands.push(GameCommand::Spawn(SpawnRequest::Explosion {
                    position: self.position,
                    kind: ExplosionKind::Human,
                    source_center: None,
                }));
                commands.push(GameCommand::PlaySound(SoundCue::HumanLost));
            }
        } else {
            self.mode = HumanMode::Falling {
                velocity: next_velocity,
            };
        }

        commands
    }

    fn update_carried(
        &mut self,
        carrier: ActorId,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> Vec<GameCommand> {
        let mut commands = Vec::new();
        if let Some(carrier_snapshot) = prompt.snapshot(carrier) {
            self.position = carrier_snapshot
                .position
                .offset(Velocity::new(0, behavior.human_carried_offset_y));
        } else {
            self.mode = HumanMode::Falling { velocity: 0 };
            commands.push(GameCommand::PlaySound(SoundCue::HumanReleased));
        }
        commands
    }
}

impl AssetActor for Human {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let previous_position = self.position;
        if prompt.phase == Phase::Playing {
            let behavior = prompt.behavior_for(self.id, ActorKind::Human);
            commands.extend(match self.mode {
                HumanMode::Grounded => {
                    self.update_grounded(prompt.source_rng, prompt.source_human_walk_target_slot);
                    Vec::new()
                }
                HumanMode::Falling { velocity } => self.update_falling(velocity, prompt, behavior),
                HumanMode::CarriedBy(carrier) => self.update_carried(carrier, prompt, behavior),
            });
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                self.mode.sprite(),
                self.position,
                self.draw_effect(),
            ));
        }
        let movement_velocity = observed_velocity(previous_position, self.position);

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Human,
                position: self.position,
                velocity: movement_velocity,
                direction: None,
                bounds: human_collision_bounds(self.mode, self.position),
                alive: prompt.phase == Phase::Playing,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: self.source,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

impl Human {
    fn draw_effect(&self) -> VisualEffect {
        self.source
            .map(|source| VisualEffect::SourceHumanFrame {
                frame: source.picture_frame,
            })
            .unwrap_or(VisualEffect::Static)
    }
}

fn intersects_snapshot(snapshot: &ActorSnapshot, bounds: Rect) -> bool {
    snapshot
        .bounds
        .is_some_and(|snapshot_bounds| snapshot_bounds.intersects(bounds))
}

fn human_collision_bounds(mode: HumanMode, position: Point) -> Option<Rect> {
    match mode {
        HumanMode::CarriedBy(_) => None,
        HumanMode::Grounded | HumanMode::Falling { .. } => Some(Rect::from_center(position, 4, 8)),
    }
}

fn actor_source_astronaut_walk_targetable(human_count: usize, snapshot: &ActorSnapshot) -> bool {
    snapshot.kind == ActorKind::Human
        && snapshot.alive
        && snapshot.bounds.is_some()
        && snapshot.source_human.is_some_and(|source| {
            human_count != usize::from(SOURCE_START_HUMAN_COUNT) || source.target_slot_index < 2
        })
}

#[derive(Debug)]
struct ScorePopup {
    id: ActorId,
    position: Point,
    points: u32,
    age: u16,
}

impl ScorePopup {
    fn new(id: ActorId, position: Point, points: u32) -> Self {
        Self {
            id,
            position,
            points,
            age: 0,
        }
    }
}

impl AssetActor for ScorePopup {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let behavior = prompt.behavior_for(self.id, ActorKind::ScorePopup);
        if self.age < behavior.score_popup_lifetime_steps {
            let sprite = match self.points {
                HUMAN_RESCUE_SCORE => SpriteKey::Score500,
                HUMAN_SAFE_LANDING_SCORE => SpriteKey::Score250,
                _ => SpriteKey::Text,
            };
            draws.push(DrawCommand::sprite(self.id, sprite, self.position));
            self.age = self.age.saturating_add(1);
        }
        if self.age >= behavior.score_popup_lifetime_steps {
            commands.push(GameCommand::Destroy(self.id));
        }

        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::ScorePopup,
                position: self.position,
                velocity: Velocity::default(),
                direction: None,
                bounds: None,
                alive: self.age < behavior.score_popup_lifetime_steps,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct LaserShot {
    id: ActorId,
    position: Point,
    direction: Direction,
    age: u16,
}

impl LaserShot {
    fn new(id: ActorId, position: Point, direction: Direction, _owner: ActorId) -> Self {
        Self {
            id,
            position,
            direction,
            age: 0,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 10, 2)
    }
}

impl AssetActor for LaserShot {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let mut movement_velocity = Velocity::default();
        let behavior = prompt.behavior_for(self.id, ActorKind::Laser);
        if prompt.phase == Phase::Playing && self.age < behavior.laser_lifetime_steps {
            movement_velocity = Velocity::new(self.direction.sign() * behavior.laser_speed, 0);
            self.position = self.position.offset(movement_velocity);
            self.age = self.age.saturating_add(1);
            draws.push(DrawCommand::sprite(
                self.id,
                SpriteKey::Laser,
                self.position,
            ));
        }
        if self.age >= behavior.laser_lifetime_steps || self.position.x < 0 || self.position.x > 255
        {
            commands.push(GameCommand::Destroy(self.id));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Laser,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(self.direction),
                bounds: Some(self.bounds()),
                alive: self.age < behavior.laser_lifetime_steps,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct EnemyLaserShot {
    id: ActorId,
    position: Point,
    source: ActorSourceEnemyProjectileMetadata,
    lifetime_steps: Option<u16>,
}

impl EnemyLaserShot {
    fn new(
        id: ActorId,
        position: Point,
        velocity: Velocity,
        source: Option<ActorSourceEnemyProjectileMetadata>,
    ) -> Self {
        let source = source.unwrap_or(ActorSourceEnemyProjectileMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: actor_source_projectile_velocity_component(velocity.dx),
            y_velocity: actor_source_projectile_velocity_component(velocity.dy),
            lifetime_ticks: 0,
        });
        let lifetime_steps = if source.lifetime_ticks == 0 {
            None
        } else {
            Some(u16::from(source.lifetime_ticks))
        };
        Self {
            id,
            position,
            source,
            lifetime_steps,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_center(self.position, 4, 4)
    }

    fn in_playfield(&self) -> bool {
        self.position.x >= 0
            && self.position.x <= 255
            && self.position.y >= 0
            && self.position.y <= 255
    }

    fn initialize_lifetime(&mut self, behavior: ActorBehaviorProfile) {
        let lifetime_steps = self
            .lifetime_steps
            .get_or_insert(behavior.lander_shot_lifetime_steps);
        self.source.lifetime_ticks = actor_source_projectile_lifetime_ticks(*lifetime_steps);
    }

    fn advance_source_projectile(&mut self) -> Velocity {
        let previous_position = self.position;
        let (x, x_fraction) = actor_source_projectile_axis_step(
            self.position.x,
            self.source.x_fraction,
            self.source.x_velocity,
        );
        let (y, y_fraction) = actor_source_projectile_axis_step(
            self.position.y,
            self.source.y_fraction,
            self.source.y_velocity,
        );
        self.position = Point::new(x, y);
        self.source.x_fraction = x_fraction;
        self.source.y_fraction = y_fraction;
        observed_velocity(previous_position, self.position)
    }
}

impl AssetActor for EnemyLaserShot {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let mut movement_velocity = Velocity::default();
        let behavior = prompt.behavior_for(self.id, ActorKind::EnemyLaser);
        self.initialize_lifetime(behavior);
        if prompt.phase == Phase::Playing && self.lifetime_steps.is_some_and(|steps| steps > 0) {
            if prompt.source_shell_scan_tick
                && let Some(lifetime_steps) = &mut self.lifetime_steps
            {
                *lifetime_steps = lifetime_steps.saturating_sub(1);
                self.source.lifetime_ticks =
                    actor_source_projectile_lifetime_ticks(*lifetime_steps);
            }
            if self.lifetime_steps.is_some_and(|steps| steps > 0) {
                movement_velocity = self.advance_source_projectile();
                draws.push(DrawCommand::sprite(
                    self.id,
                    SpriteKey::EnemyLaser,
                    self.position,
                ));
            }
        }
        let expired_or_out_of_bounds = self.lifetime_steps == Some(0) || !self.in_playfield();
        if expired_or_out_of_bounds {
            commands.push(GameCommand::Destroy(self.id));
        }
        let alive = prompt.phase == Phase::Playing && !expired_or_out_of_bounds;
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::EnemyLaser,
                position: self.position,
                velocity: movement_velocity,
                direction: Some(direction_for_velocity(movement_velocity, Direction::Right)),
                bounds: Some(self.bounds()),
                alive,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: Some(self.source),
            },
            commands,
            draws,
        }
    }
}

#[derive(Debug)]
struct Explosion {
    id: ActorId,
    position: Point,
    kind: ExplosionKind,
    source_center: Option<Point>,
    age: u16,
}

impl Explosion {
    fn new(
        id: ActorId,
        position: Point,
        kind: ExplosionKind,
        source_center: Option<Point>,
    ) -> Self {
        Self {
            id,
            position,
            kind,
            source_center,
            age: 0,
        }
    }
}

impl AssetActor for Explosion {
    fn id(&self) -> ActorId {
        self.id
    }

    fn update(&mut self, prompt: &StepPrompt) -> ActorReply {
        let mut commands = Vec::new();
        let mut draws = Vec::new();
        let behavior = prompt.behavior_for(self.id, ActorKind::Explosion);
        let lifetime_steps = explosion_lifetime_steps(self.kind, behavior);
        if self.age < lifetime_steps {
            draws.push(DrawCommand::sprite_with_effect(
                self.id,
                SpriteKey::Explosion,
                self.position,
                VisualEffect::ExplosionCloud {
                    kind: self.kind,
                    age: self.age,
                    source_center: self.source_center,
                },
            ));
            self.age = self.age.saturating_add(1);
        }
        if self.age >= lifetime_steps {
            commands.push(GameCommand::Destroy(self.id));
        }
        ActorReply {
            id: self.id,
            snapshot: ActorSnapshot {
                id: self.id,
                kind: ActorKind::Explosion,
                position: self.position,
                velocity: Velocity::default(),
                direction: None,
                bounds: None,
                alive: self.age < lifetime_steps,
                source_lander: None,
                source_bomber: None,
                source_pod: None,
                source_swarmer: None,
                source_baiter: None,
                source_mutant: None,
                source_human: None,
                source_enemy_projectile: None,
            },
            commands,
            draws,
        }
    }
}

fn explosion_lifetime_steps(kind: ExplosionKind, behavior: ActorBehaviorProfile) -> u16 {
    if kind == ExplosionKind::Terrain {
        return u16::from(SOURCE_TERRAIN_EXPLOSION_LIFETIME_FRAMES);
    }

    behavior.explosion_lifetime_steps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_sound_cues_expose_red_label_sound_commands() {
        let expected = [
            (SoundCue::Credit, 0xE6),
            (SoundCue::Start, 0xF5),
            (SoundCue::Thrust, 0xE9),
            (SoundCue::Laser, 0xEB),
            (SoundCue::SmartBomb, 0xEE),
            (SoundCue::PlayerAppear, 0xEA),
            (SoundCue::HyperspaceMaterialize, 0xEA),
            (SoundCue::Explosion, 0xEE),
            (SoundCue::LanderHit, 0xF9),
            (SoundCue::LanderPickup, 0xF4),
            (SoundCue::HumanPulled, 0xF1),
            (SoundCue::HumanReleased, 0xE5),
            (SoundCue::HumanRescued, 0xF7),
            (SoundCue::HumanSafeLanding, 0xE0),
            (SoundCue::HumanLost, 0xEE),
            (SoundCue::MutantSpawn, 0xEE),
            (SoundCue::MutantHit, 0xE8),
            (SoundCue::BomberHit, 0xFE),
            (SoundCue::BombHit, 0xEE),
            (SoundCue::PodHit, 0xFA),
            (SoundCue::SwarmerHit, 0xF8),
            (SoundCue::LanderShot, 0xFC),
            (SoundCue::MutantShot, 0xF6),
            (SoundCue::SwarmerShot, 0xF3),
            (SoundCue::BaiterHit, 0xF8),
            (SoundCue::BaiterShot, 0xFC),
            (SoundCue::GameOver, 0xEC),
            (SoundCue::SourceCommand(0xE8), 0xE8),
        ];

        for (cue, command) in expected {
            assert_eq!(cue.source_sound_command(), Some(command), "{cue:?}");
        }
        for cue in [SoundCue::Hyperspace, SoundCue::AttractPulse] {
            assert_eq!(cue.source_sound_command(), None, "{cue:?}");
        }
    }

    #[test]
    fn actor_sound_cues_map_to_clean_sound_events() {
        assert_eq!(
            SoundCue::Credit.sound_event(),
            Some(SoundEvent::CreditAdded)
        );
        assert_eq!(SoundCue::Start.sound_event(), Some(SoundEvent::GameStarted));
        assert_eq!(
            SoundCue::Thrust.sound_event(),
            Some(SoundEvent::ThrustStarted)
        );
        assert_eq!(
            SoundCue::Laser.sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xEB })
        );
        assert_eq!(
            SoundCue::PlayerAppear.sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xEA })
        );
        assert_eq!(
            SoundCue::LanderShot.sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xFC })
        );
        assert_eq!(
            SoundCue::MutantShot.sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xF6 })
        );
        assert_eq!(
            SoundCue::SourceCommand(0xE8).sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xE8 })
        );
        assert_eq!(
            SoundCue::HumanReleased.sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xE5 })
        );
        assert_eq!(SoundCue::Hyperspace.sound_event(), None);
    }

    #[test]
    fn actor_sound_event_bridge_emits_clean_audio_events_and_thrust_edges() {
        let mut bridge = ActorSoundEventBridge::new();

        assert_eq!(
            bridge.sound_events_for_cues(&[SoundCue::Credit, SoundCue::Laser]),
            [
                SoundEvent::CreditAdded,
                SoundEvent::UnmappedSoundCommand { command: 0xEB },
            ]
        );
        assert_eq!(
            bridge.sound_events_for_cues(&[SoundCue::Thrust, SoundCue::LanderShot]),
            [
                SoundEvent::ThrustStarted,
                SoundEvent::UnmappedSoundCommand { command: 0xFC },
            ]
        );
        assert_eq!(
            bridge.sound_events_for_cues(&[SoundCue::Thrust, SoundCue::SwarmerShot]),
            [SoundEvent::UnmappedSoundCommand { command: 0xF3 }]
        );
        assert_eq!(
            ActorSoundEventBridge::new().sound_events_for_cues(&[SoundCue::MutantShot]),
            [SoundEvent::UnmappedSoundCommand { command: 0xF6 }]
        );
        assert_eq!(
            ActorSoundEventBridge::new().sound_events_for_cues(&[SoundCue::SourceCommand(0xE8)]),
            [SoundEvent::UnmappedSoundCommand { command: 0xE8 }]
        );
        assert_eq!(
            bridge.sound_events_for_cues(&[SoundCue::HumanReleased]),
            [
                SoundEvent::UnmappedSoundCommand { command: 0xE5 },
                SoundEvent::ThrustStopped,
            ]
        );
        assert!(bridge.sound_events_for_cues(&[]).is_empty());
    }

    #[test]
    fn step_report_uses_actor_sound_event_bridge() {
        let mut driver = started_driver();
        let mut bridge = ActorSoundEventBridge::new();

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });

        assert_eq!(
            fired.sound_events(&mut bridge),
            [SoundEvent::UnmappedSoundCommand { command: 0xEB }]
        );

        let thrusting = driver.step(GameInput {
            thrust: true,
            ..GameInput::NONE
        });
        assert_eq!(
            thrusting.sound_events(&mut bridge),
            [SoundEvent::ThrustStarted]
        );

        let coasting = driver.step(GameInput::NONE);
        assert_eq!(
            coasting.sound_events(&mut bridge),
            [SoundEvent::ThrustStopped]
        );
    }

    #[test]
    fn actor_render_scene_bridge_projects_attract_source_pixels() {
        let mut driver = ActorGameDriver::new();

        let williams = driver.step(GameInput::NONE);
        let williams_scene = williams.render_scene();
        assert_eq!(williams_scene.frame, williams.step);
        assert_eq!(williams_scene.surface, ACTOR_RENDER_SURFACE);
        assert!(williams_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(
            !williams_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO)
        );
        assert!(!williams_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_H && sprite.layer == RenderLayer::Overlay
        }));

        let mut coalescing = None;
        for _ in 0..DEFENDER_WORDMARK_START_STEP {
            let report = driver.step(GameInput::NONE);
            if report
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderCoalescence)
            {
                coalescing = Some(report);
                break;
            }
        }
        let coalescing = coalescing.expect("Defender wordmark should coalesce");
        let coalescing_scene = coalescing.render_scene();
        assert!(coalescing_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(
            !coalescing_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO)
        );

        let mut hall = None;
        for _ in SOURCE_ATTRACT_DEFENDER_WORDMARK_START_STEP..SOURCE_ATTRACT_HALL_OF_FAME_START_STEP
        {
            hall = Some(driver.step(GameInput::NONE));
        }
        let hall_scene = hall
            .expect("hall-of-fame page boundary should be reached")
            .render_scene();
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_H && sprite.layer == RenderLayer::Overlay
        }));
    }

    #[test]
    fn actor_render_scene_bridge_projects_playing_actors_and_status_text() {
        let mut driver = started_driver();

        let report = driver.step(GameInput::NONE);
        let scene = report.render_scene();

        assert_eq!(scene.frame, report.step);
        assert!(scene.sprites.iter().any(|sprite| {
            matches!(
                sprite.sprite,
                SpriteId::PLAYER_SHIP | SpriteId::PLAYER_SHIP_LEFT
            ) && sprite.layer == RenderLayer::Objects
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER && sprite.layer == RenderLayer::Objects
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN && sprite.layer == RenderLayer::Objects
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            SpriteId::SCORE_DIGITS.contains(&sprite.sprite) && sprite.layer == RenderLayer::Hud
        }));
    }

    #[test]
    fn actor_render_scene_bridge_ignores_armed_terrain_flash_until_erased() {
        let mut driver = started_driver();
        let mut report = driver.step(GameInput::NONE);
        let mut terrain_blow = TerrainBlowSnapshot::source_armed_terrain_visible();
        terrain_blow.source_elapsed_frames = 2;
        report.terrain_blow = Some(terrain_blow);

        let scene = report.render_scene();

        assert!(!terrain_blow.terrain_erased());
        assert_ne!(
            source_terrain_blow_flash_tint(terrain_blow.source_elapsed_frames),
            Color { rgba: [0; 4] }
        );
        assert_eq!(scene.clear_color, Color { rgba: [0; 4] });
    }

    #[test]
    fn actor_wave_clear_delays_next_wave_and_draws_source_survivor_bonus_scene() {
        let wave_script = ActorWaveScript::new(
            "wave-clear-interstitial",
            vec![
                ActorWaveProfile::with_spawns(
                    1,
                    ActorBehaviorScript::default(),
                    vec![ActorLanderSpawn::new(Point::new(100, 80))],
                    vec![
                        ActorHumanSpawn::new(Point::new(40, HUMAN_GROUND_Y), HumanMode::Grounded),
                        ActorHumanSpawn::new(Point::new(64, HUMAN_GROUND_Y), HumanMode::Grounded),
                    ],
                ),
                ActorWaveProfile::with_spawns(
                    2,
                    ActorBehaviorScript::default(),
                    vec![ActorLanderSpawn::new(Point::new(120, 80))],
                    Vec::new(),
                ),
            ],
        );
        let mut runtime =
            ActorRuntimeAdapter::with_driver(ActorGameDriver::with_wave_script(wave_script));
        runtime.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        runtime.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        step_until_player_start_completes(&mut runtime, 1);

        let pressed = runtime.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });
        assert_eq!(pressed.report.score, 0);
        assert!(
            pressed
                .events
                .gameplay()
                .contains(&GameEvent::SmartBombPressed)
        );
        assert!(!pressed.events.gameplay().contains(&GameEvent::WaveCleared));
        assert_eq!(pressed.state.world.enemies.len(), 1);

        let cleared = step_until_smart_bomb_detonates(&mut runtime);

        assert_eq!(cleared.state.wave, 1);
        assert!(cleared.state.world.enemies.is_empty());
        assert_eq!(cleared.state.world.humans.len(), 2);
        assert_eq!(cleared.report.score, 250);
        assert_eq!(
            cleared.report.survivor_bonus,
            Some(SurvivorBonusReport {
                next_wave: 2,
                multiplier: 1,
                total_survivors: 2,
                visible_icons: 1,
                remaining_awards: 1,
                awarded_points: Some(100),
                astronaut_sleep_steps_remaining: SOURCE_SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS,
                wave_advance_sleep_steps_remaining: None,
            })
        );
        assert!(
            cleared
                .report
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );
        assert!(cleared.events.gameplay().contains(&GameEvent::WaveCleared));
        assert!(!cleared.events.gameplay().contains(&GameEvent::WaveStarted));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_A
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [112.0, 80.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_C
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [122.0, 96.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_B
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [120.0, 144.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [202.0, 80.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [176.0, 144.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(scene_has_survivor_bonus_icon(
            &cleared.scene,
            [120.0, 160.0]
        ));
        assert!(!scene_has_survivor_bonus_icon(
            &cleared.scene,
            [128.0, 160.0]
        ));

        for expected_sleep in [3, 2, 1] {
            let sleep = runtime.step(GameInput::NONE);
            assert_eq!(sleep.state.wave, 1);
            assert_eq!(sleep.report.score, 250);
            assert_eq!(
                sleep
                    .report
                    .survivor_bonus
                    .expect("survivor bonus should remain active")
                    .astronaut_sleep_steps_remaining,
                expected_sleep
            );
            assert!(!sleep.events.gameplay().contains(&GameEvent::WaveStarted));
        }

        let second_survivor = runtime.step(GameInput::NONE);

        assert_eq!(second_survivor.state.wave, 1);
        assert_eq!(second_survivor.report.score, 350);
        assert_eq!(
            second_survivor.report.survivor_bonus,
            Some(SurvivorBonusReport {
                next_wave: 2,
                multiplier: 1,
                total_survivors: 2,
                visible_icons: 2,
                remaining_awards: 0,
                awarded_points: Some(100),
                astronaut_sleep_steps_remaining: SOURCE_SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS,
                wave_advance_sleep_steps_remaining: None,
            })
        );
        assert!(scene_has_survivor_bonus_icon(
            &second_survivor.scene,
            [120.0, 160.0]
        ));
        assert!(scene_has_survivor_bonus_icon(
            &second_survivor.scene,
            [128.0, 160.0]
        ));

        for expected_sleep in [3, 2, 1] {
            let sleep = runtime.step(GameInput::NONE);
            assert_eq!(
                sleep
                    .report
                    .survivor_bonus
                    .expect("survivor bonus should remain active")
                    .astronaut_sleep_steps_remaining,
                expected_sleep
            );
            assert!(!sleep.events.gameplay().contains(&GameEvent::WaveStarted));
        }

        let wave_sleep = runtime.step(GameInput::NONE);
        assert_eq!(
            wave_sleep
                .report
                .survivor_bonus
                .expect("survivor bonus should enter wave sleep")
                .wave_advance_sleep_steps_remaining,
            Some(SOURCE_SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS)
        );
        assert!(
            !wave_sleep
                .events
                .gameplay()
                .contains(&GameEvent::WaveStarted)
        );

        for expected_sleep in (1..SOURCE_SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS).rev() {
            let sleep = runtime.step(GameInput::NONE);
            assert_eq!(
                sleep
                    .report
                    .survivor_bonus
                    .expect("survivor bonus wave sleep should remain active")
                    .wave_advance_sleep_steps_remaining,
                Some(expected_sleep)
            );
            assert!(!sleep.events.gameplay().contains(&GameEvent::WaveStarted));
        }

        let next_wave = runtime.step(GameInput::NONE);

        assert_eq!(next_wave.state.wave, 2);
        assert!(
            next_wave
                .events
                .gameplay()
                .contains(&GameEvent::WaveStarted)
        );
        assert!(
            next_wave
                .report
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 2 })
        );
        assert_eq!(next_wave.report.survivor_bonus, None);
        assert_eq!(next_wave.state.world.humans.len(), 0);
        assert!(
            next_wave
                .state
                .world
                .enemies
                .iter()
                .any(|enemy| enemy.kind == CleanEnemyKind::Lander)
        );
    }

    #[test]
    fn actor_score_awards_replay_stock_and_bonus_event() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.score = 9_900;
        driver.next_bonus = SOURCE_REPLAY_SCORE;
        driver.lives = 3;
        driver.smart_bombs = 1;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(62, 120));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        runtime.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let scored = runtime.step(GameInput::NONE);

        assert_eq!(scored.report.score, 10_050);
        assert_eq!(scored.report.next_bonus, 20_000);
        assert!(scored.report.bonus_awarded);
        assert_eq!(scored.state.scores.player_one, 10_050);
        assert_eq!(scored.state.scores.high_score, 10_050);
        assert_eq!(scored.state.scores.next_bonus, 20_000);
        assert_eq!(scored.state.player.lives, 4);
        assert_eq!(scored.state.player.smart_bombs, 2);
        assert_eq!(
            scored.state.player_stocks[0],
            PlayerStockSnapshot::new(4, 2)
        );
        assert!(scored.events.gameplay().contains(&GameEvent::BonusAwarded));
    }

    #[test]
    fn actor_survivor_bonus_uses_source_wave_multiplier_and_replay_stock() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 3;
        driver.score = 9_700;
        driver.next_bonus = SOURCE_REPLAY_SCORE;
        driver.lives = 3;
        driver.smart_bombs = 3;
        driver.spawn_player();
        driver.spawn_human_for_test(Point::new(40, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(64, HUMAN_GROUND_Y));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let cleared = runtime.step(GameInput::NONE);

        assert!(
            cleared
                .report
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 4 })
        );
        assert_eq!(cleared.report.score, 10_000);
        assert_eq!(cleared.report.next_bonus, 20_000);
        assert!(cleared.report.bonus_awarded);
        assert_eq!(
            cleared.report.survivor_bonus,
            Some(SurvivorBonusReport {
                next_wave: 4,
                multiplier: 3,
                total_survivors: 2,
                visible_icons: 1,
                remaining_awards: 1,
                awarded_points: Some(300),
                astronaut_sleep_steps_remaining: SOURCE_SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS,
                wave_advance_sleep_steps_remaining: None,
            })
        );
        assert_eq!(cleared.state.player.lives, 4);
        assert_eq!(cleared.state.player.smart_bombs, 4);
        assert!(cleared.events.gameplay().contains(&GameEvent::BonusAwarded));

        for _ in 0..SOURCE_SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS - 1 {
            runtime.step(GameInput::NONE);
        }
        let second_survivor = runtime.step(GameInput::NONE);

        assert_eq!(second_survivor.report.score, 10_300);
        assert_eq!(
            second_survivor.report.survivor_bonus,
            Some(SurvivorBonusReport {
                next_wave: 4,
                multiplier: 3,
                total_survivors: 2,
                visible_icons: 2,
                remaining_awards: 0,
                awarded_points: Some(300),
                astronaut_sleep_steps_remaining: SOURCE_SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS,
                wave_advance_sleep_steps_remaining: None,
            })
        );
        assert!(!second_survivor.report.bonus_awarded);
    }

    #[test]
    fn actor_render_scene_bridge_maps_projectiles_and_explosion_variants() {
        let report = StepReport {
            step: 99,
            phase: Phase::Playing,
            wave: 1,
            current_player: 1,
            player_count: 1,
            score: 0,
            player_scores: [0, 0],
            credits: 0,
            lives: 3,
            smart_bombs: 3,
            smart_bomb_flash_steps_remaining: 0,
            player_stocks: [PlayerStockSnapshot::new(3, 3); 2],
            next_bonus: SOURCE_REPLAY_SCORE,
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [10_000, 7_500, 5_000, 2_500, 1_000],
            source_wave: ActorSourceWaveProfile::for_wave(1),
            high_score_initials: HighScoreInitialsState::EMPTY,
            high_score_initial_accepted: false,
            high_score_submitted: false,
            bonus_awarded: false,
            survivor_bonus: None,
            behavior_script: ActorBehaviorScript::default().manifest(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            source_background_left: 0,
            source_rng: None,
            terrain_blow: None,
            snapshots: Vec::new(),
            draws: vec![
                DrawCommand::sprite(ActorId(101), SpriteKey::Laser, Point::new(40, 80)),
                DrawCommand::sprite(ActorId(102), SpriteKey::EnemyLaser, Point::new(90, 82)),
                DrawCommand::sprite_with_effect(
                    ActorId(103),
                    SpriteKey::Explosion,
                    Point::new(104, 82),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Lander,
                        age: 2,
                        source_center: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(104),
                    SpriteKey::Explosion,
                    Point::new(108, 84),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Mutant,
                        age: 2,
                        source_center: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(105),
                    SpriteKey::Explosion,
                    Point::new(112, 86),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Bomber,
                        age: 2,
                        source_center: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(106),
                    SpriteKey::Explosion,
                    Point::new(116, 88),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Pod,
                        age: 2,
                        source_center: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(107),
                    SpriteKey::Explosion,
                    Point::new(120, 90),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Swarmer,
                        age: 2,
                        source_center: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(108),
                    SpriteKey::Explosion,
                    Point::new(122, 92),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Baiter,
                        age: 2,
                        source_center: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(109),
                    SpriteKey::Explosion,
                    Point::new(124, 94),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Bomb,
                        age: 2,
                        source_center: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(110),
                    SpriteKey::Explosion,
                    Point::new(124, 96),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Human,
                        age: 1,
                        source_center: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(111),
                    SpriteKey::Explosion,
                    Point::new(128, 100),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Player,
                        age: 1,
                        source_center: None,
                    },
                ),
            ],
            sounds: Vec::new(),
            commands: Vec::new(),
        };

        let bridge = ActorRenderSceneBridge::with_surface(SurfaceSize::new(320, 240));
        let scene = report.render_scene_with(&bridge);

        assert_eq!(bridge.surface(), SurfaceSize::new(320, 240));
        assert_eq!(scene.surface, SurfaceSize::new(320, 240));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.layer == RenderLayer::Projectiles
                && sprite.size == PLAYER_PROJECTILE_SCENE_SIZE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_BOMB && sprite.layer == RenderLayer::Projectiles
        }));
        for sprite_id in [
            SpriteId::ENEMY_LANDER,
            SpriteId::ENEMY_MUTANT,
            SpriteId::ENEMY_BOMBER,
            SpriteId::ENEMY_POD,
            SpriteId::SWARMER_EXPLOSION,
            SpriteId::ENEMY_BAITER,
        ] {
            assert!(
                !scene.sprites.iter().any(
                    |sprite| sprite.sprite == sprite_id && sprite.layer == RenderLayer::Objects
                ),
                "source-family explosion should use pixel cloud, not {sprite_id:?}"
            );
        }
        let source_cloud_pixels = scene
            .sprites
            .iter()
            .filter(|sprite| {
                sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                    && sprite.layer == RenderLayer::Objects
            })
            .count();
        assert!(
            source_cloud_pixels > 1,
            "source-family explosions should project expanded-object pixels"
        );
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::BOMB_EXPLOSION && sprite.layer == RenderLayer::Objects
        }));
        let bomb_explosion = scene
            .sprites
            .iter()
            .find(|sprite| sprite.sprite == SpriteId::BOMB_EXPLOSION)
            .expect("bomb explosion sprite should be projected");
        assert_eq!(bomb_explosion.size, [16.0, 16.0]);
        assert_eq!(bomb_explosion.position, [120.0, 90.0]);
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ASTRONAUT_EXPLOSION && sprite.layer == RenderLayer::Objects
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                && sprite.layer == RenderLayer::Objects
        }));
    }

    #[test]
    fn actor_source_explosion_render_scale_uses_source_size_curve() {
        assert_eq!(
            actor_source_explosion_render_scale(source_explosion_size_for_age(0)),
            1.0
        );
        assert_eq!(
            actor_source_explosion_render_scale(source_explosion_size_for_age(1)),
            1.0
        );
        assert_eq!(
            actor_source_explosion_render_scale(source_explosion_size_for_age(2)),
            2.0
        );
        assert_eq!(
            actor_source_explosion_render_scale(source_explosion_size_for_age(18)),
            3.0
        );
    }

    #[test]
    fn actor_explosion_source_center_reaches_state_and_render_bridges() {
        let top_left = Point::new(0x20, 0xA2);
        let source_center = Point::new(0x21, 0xA9);
        let report = StepReport {
            step: 7,
            phase: Phase::Playing,
            wave: 1,
            current_player: 1,
            player_count: 1,
            score: 0,
            player_scores: [0, 0],
            credits: 0,
            lives: 3,
            smart_bombs: 3,
            smart_bomb_flash_steps_remaining: 0,
            player_stocks: [PlayerStockSnapshot::new(3, 3); 2],
            next_bonus: SOURCE_REPLAY_SCORE,
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [10_000, 7_500, 5_000, 2_500, 1_000],
            source_wave: ActorSourceWaveProfile::for_wave(1),
            high_score_initials: HighScoreInitialsState::EMPTY,
            high_score_initial_accepted: false,
            high_score_submitted: false,
            bonus_awarded: false,
            survivor_bonus: None,
            behavior_script: ActorBehaviorScript::default().manifest(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            source_background_left: 0,
            source_rng: None,
            terrain_blow: None,
            snapshots: Vec::new(),
            draws: vec![DrawCommand::sprite_with_effect(
                ActorId(101),
                SpriteKey::Explosion,
                top_left,
                VisualEffect::ExplosionCloud {
                    kind: ExplosionKind::Mutant,
                    age: 2,
                    source_center: Some(source_center),
                },
            )],
            sounds: Vec::new(),
            commands: Vec::new(),
        };

        let state = ActorStateBridge::new().state_for_report(&report);
        assert_eq!(state.world.explosions.len(), 1);
        assert_eq!(state.world.explosions[0].kind, CleanExplosionKind::Mutant);
        assert_eq!(
            state.world.explosions[0].position,
            ScreenPosition::new(0x20, 0xA2)
        );
        assert_eq!(
            state.world.explosions[0].source_center,
            Some(ScreenPosition::new(0x21, 0xA9))
        );
        assert_eq!(
            state.world.explosions[0].source_size,
            source_explosion_size_for_age(2)
        );

        let scene = report.render_scene();
        let mut expected = RenderScene::empty(report.step, ACTOR_RENDER_SURFACE);
        assert!(push_source_explosion_cloud_pixels(
            &mut expected,
            CleanExplosionKind::Mutant,
            ScreenPosition::new(0x20, 0xA2),
            Some(ScreenPosition::new(0x21, 0xA9)),
            source_explosion_size_for_age(2),
        ));
        let object_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.layer == RenderLayer::Objects)
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(object_sprites, expected.sprites);
    }

    #[test]
    fn actor_state_bridge_maps_report_snapshots_and_draw_effects_to_clean_state() {
        let mut player = actor_snapshot(11, ActorKind::Player, Point::new(40, 70));
        player.bounds = Some(Rect::from_center(player.position, 16, 6));
        player.velocity = Velocity::new(3, -1);
        player.direction = Some(Direction::Right);
        let mut lander = actor_snapshot(12, ActorKind::Lander, Point::new(0x3F, 0x2C));
        lander.velocity = Velocity::new(-2, 1);
        lander.source_lander = Some(ActorSourceLanderMetadata {
            x_fraction: 0x4A,
            y_fraction: 0xE0,
            x_velocity: 0xFFEE,
            y_velocity: 0x0070,
            shot_timer: 0x3B,
            sleep_ticks: 0x04,
            picture_frame: 1,
            target_human_index: Some(2),
        });
        let mut human = actor_snapshot(13, ActorKind::Human, Point::new(0x1C, 0xE1));
        human.source_human = Some(ActorSourceHumanMetadata {
            x_fraction: 0x81,
            y_fraction: 0,
            picture_frame: 3,
            target_slot_index: 1,
        });
        let mut laser = actor_snapshot(14, ActorKind::Laser, Point::new(80, 72));
        laser.velocity = Velocity::new(8, 0);
        laser.direction = Some(Direction::Right);
        let mut enemy_laser = actor_snapshot(15, ActorKind::EnemyLaser, Point::new(90, 80));
        enemy_laser.velocity = Velocity::new(-3, 2);
        enemy_laser.source_enemy_projectile = Some(ActorSourceEnemyProjectileMetadata {
            x_fraction: 0x22,
            y_fraction: 0x77,
            x_velocity: 0xFD00,
            y_velocity: 0x0200,
            lifetime_ticks: 17,
        });
        let mut bomb = actor_snapshot(16, ActorKind::Bomb, Point::new(100, 84));
        bomb.source_enemy_projectile = Some(ActorSourceEnemyProjectileMetadata {
            x_fraction: 0x44,
            y_fraction: 0x55,
            x_velocity: 0,
            y_velocity: 0,
            lifetime_ticks: 9,
        });

        let report = StepReport {
            step: 77,
            phase: Phase::HighScoreEntry,
            wave: 2,
            current_player: 1,
            player_count: 1,
            score: 12_000,
            player_scores: [12_000, 0],
            credits: 1,
            lives: 2,
            smart_bombs: 1,
            smart_bomb_flash_steps_remaining: 0,
            player_stocks: [
                PlayerStockSnapshot::new(2, 1),
                PlayerStockSnapshot::new(3, 3),
            ],
            next_bonus: 20_000,
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [12_000, 10_000, 7_500, 5_000, 2_500],
            source_wave: ActorSourceWaveProfile::for_wave(2),
            high_score_initials: HighScoreInitialsState {
                initials: [Some('R'), None, None],
                cursor: 1,
            },
            high_score_initial_accepted: false,
            high_score_submitted: false,
            bonus_awarded: false,
            survivor_bonus: None,
            behavior_script: ActorBehaviorScript::default().manifest(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            source_background_left: 0,
            source_rng: None,
            terrain_blow: None,
            snapshots: vec![player, lander, human, laser, enemy_laser, bomb],
            draws: vec![
                DrawCommand::sprite(ActorId(11), SpriteKey::PlayerLeft, Point::new(40, 70)),
                DrawCommand::sprite(ActorId(13), SpriteKey::HumanCarried, Point::new(0x1C, 0xE1)),
                DrawCommand::sprite_with_effect(
                    ActorId(17),
                    SpriteKey::Explosion,
                    Point::new(120, 90),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Lander,
                        age: 0,
                        source_center: None,
                    },
                ),
                DrawCommand::sprite(ActorId(18), SpriteKey::Score500, Point::new(122, 88)),
            ],
            sounds: Vec::new(),
            commands: Vec::new(),
        };

        let state = report.game_state();

        assert_eq!(state.frame, 77);
        assert_eq!(state.phase, GamePhase::HighScoreEntry);
        assert_eq!(state.credits, 1);
        assert_eq!(state.wave, 2);
        assert_eq!(state.player.direction, CleanDirection::Right);
        assert_eq!(state.player.position.0.subpixels(), 40 * 256);
        assert_eq!(state.player.velocity.0.subpixels(), 3 * 256);
        assert_eq!(state.player.velocity.1.subpixels(), -256);
        assert_eq!(state.player_stocks[0], PlayerStockSnapshot::new(2, 1));
        assert_eq!(state.scores.player_one, 12_000);
        assert_eq!(state.scores.high_score, 12_000);
        assert_eq!(state.high_score_initials.initials, [Some('R'), None, None]);
        assert_eq!(
            state.high_score_entry,
            Some(HighScoreEntrySnapshot {
                score: 12_000,
                rank: 1
            })
        );
        assert_eq!(state.high_score_tables.all_time[0].score, 12_000);

        assert_eq!(state.world.enemies.len(), 1);
        assert_eq!(state.world.enemies[0].kind, CleanEnemyKind::Lander);
        assert_eq!(state.world.enemies[0].velocity, ScreenVelocity::new(-2, 1));
        assert_eq!(
            state.world.enemies[0].source_lander,
            Some(SourceLanderSnapshot {
                x_fraction: 0x4A,
                y_fraction: 0xE0,
                x_velocity: 0xFFEE,
                y_velocity: 0x0070,
                shot_timer: 0x3B,
                sleep_ticks: 0x04,
                picture_frame: 1,
                target_human_index: Some(2),
            })
        );
        assert_eq!(state.world.humans.len(), 1);
        assert!(state.world.humans[0].carried);
        assert_eq!(state.world.humans[0].source_picture_frame, 3);
        assert_eq!(state.world.projectiles.len(), 1);
        assert_eq!(
            state.world.projectiles[0].velocity,
            ScreenVelocity::new(8, 0)
        );
        assert_eq!(state.world.enemy_projectiles.len(), 2);
        let fireball = state
            .world
            .enemy_projectiles
            .iter()
            .find(|projectile| projectile.source_kind == EnemyProjectileSourceKind::Fireball)
            .expect("actor enemy laser should bridge as a source fireball");
        assert_eq!(fireball.velocity, ScreenVelocity::new(-3, 2));
        assert_eq!(fireball.source_x_fraction, 0x22);
        assert_eq!(fireball.source_y_fraction, 0x77);
        assert_eq!(fireball.source_x_velocity, 0xFD00);
        assert_eq!(fireball.source_y_velocity, 0x0200);
        assert_eq!(fireball.source_lifetime_ticks, 17);
        let bomb_shell = state
            .world
            .enemy_projectiles
            .iter()
            .find(|projectile| projectile.source_kind == EnemyProjectileSourceKind::BomberBombShell)
            .expect("actor bomb should bridge as a source bomb shell");
        assert_eq!(bomb_shell.source_x_fraction, 0x44);
        assert_eq!(bomb_shell.source_y_fraction, 0x55);
        assert_eq!(bomb_shell.source_x_velocity, 0);
        assert_eq!(bomb_shell.source_y_velocity, 0);
        assert_eq!(bomb_shell.source_lifetime_ticks, 9);
        assert!(
            state
                .world
                .enemy_projectiles
                .iter()
                .any(|projectile| projectile.velocity == ScreenVelocity::new(-3, 2))
        );
        assert!(state.world.enemy_projectiles.iter().any(|projectile| {
            projectile.source_kind == EnemyProjectileSourceKind::BomberBombShell
        }));
        assert_eq!(state.world.explosions.len(), 1);
        assert_eq!(state.world.explosions[0].kind, CleanExplosionKind::Lander);
        assert_eq!(state.world.score_popups.len(), 1);
        assert_eq!(
            state.world.score_popups[0].kind,
            CleanScorePopupKind::Points500
        );
    }

    #[test]
    fn actor_state_bridge_preserves_enemy_family_explosion_kinds() {
        let draws = [
            (ExplosionKind::Lander, Point::new(20, 40)),
            (ExplosionKind::Mutant, Point::new(24, 44)),
            (ExplosionKind::Bomber, Point::new(28, 48)),
            (ExplosionKind::Pod, Point::new(32, 52)),
            (ExplosionKind::Swarmer, Point::new(36, 56)),
            (ExplosionKind::Baiter, Point::new(40, 60)),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (kind, position))| {
            DrawCommand::sprite_with_effect(
                ActorId(200 + index as u64),
                SpriteKey::Explosion,
                position,
                VisualEffect::ExplosionCloud {
                    kind,
                    age: 0,
                    source_center: None,
                },
            )
        })
        .collect::<Vec<_>>();

        let report = StepReport {
            step: 101,
            phase: Phase::Playing,
            wave: 3,
            current_player: 1,
            player_count: 1,
            score: 0,
            player_scores: [0, 0],
            credits: 0,
            lives: 3,
            smart_bombs: 3,
            smart_bomb_flash_steps_remaining: 0,
            player_stocks: [PlayerStockSnapshot::new(3, 3); 2],
            next_bonus: SOURCE_REPLAY_SCORE,
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [10_000, 7_500, 5_000, 2_500, 1_000],
            source_wave: ActorSourceWaveProfile::for_wave(3),
            high_score_initials: HighScoreInitialsState::EMPTY,
            high_score_initial_accepted: false,
            high_score_submitted: false,
            bonus_awarded: false,
            survivor_bonus: None,
            behavior_script: ActorBehaviorScript::default().manifest(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            source_background_left: 0,
            source_rng: None,
            terrain_blow: None,
            snapshots: Vec::new(),
            draws,
            sounds: Vec::new(),
            commands: Vec::new(),
        };

        let kinds = report
            .game_state()
            .world
            .explosions
            .iter()
            .map(|explosion| explosion.kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                CleanExplosionKind::Lander,
                CleanExplosionKind::Mutant,
                CleanExplosionKind::Bomber,
                CleanExplosionKind::Pod,
                CleanExplosionKind::Swarmer,
                CleanExplosionKind::Baiter,
            ]
        );
    }

    #[test]
    fn actor_input_converts_clean_live_key_contract_with_xyzzy_overlay() {
        let xyzzy = XyzzyMode {
            active: true,
            auto_fire: true,
            invincible: true,
            overlay_smart_bomb: true,
        };
        let input = GameInput::from_clean_input(
            CleanGameInput {
                coin: true,
                coin_two: true,
                coin_three: true,
                start_one: true,
                start_two: true,
                altitude_up: true,
                altitude_down: true,
                reverse: true,
                thrust: true,
                fire: true,
                smart_bomb: true,
                hyperspace: true,
                service_auto_up: true,
                service_advance: true,
                high_score_reset: true,
                high_score_initial: Some('A'),
                high_score_backspace: true,
                tilt: true,
            },
            xyzzy,
        );

        assert!(input.coin);
        assert!(input.coin_two);
        assert!(input.coin_three);
        assert!(input.start_one);
        assert!(input.start_two);
        assert!(input.altitude_up);
        assert!(input.altitude_down);
        assert!(input.reverse);
        assert!(input.thrust);
        assert!(input.fire);
        assert!(input.smart_bomb);
        assert!(input.hyperspace);
        assert!(input.service_advance);
        assert!(input.high_score_reset);
        assert_eq!(input.high_score_initial, Some('A'));
        assert!(input.high_score_backspace);
        assert!(input.auto_up_manual_down);
        assert!(input.tilt);
        assert_eq!(input.xyzzy, xyzzy);
    }

    #[test]
    fn actor_runtime_adapter_bundles_report_events_audio_and_scene() {
        let mut runtime = ActorRuntimeAdapter::new();

        let credited = runtime.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.report.phase, Phase::Attract);
        assert_eq!(credited.report.credits, 1);
        assert_eq!(credited.state.phase, GamePhase::Attract);
        assert_eq!(credited.state.credits, 1);
        assert_eq!(credited.events.gameplay(), &[GameEvent::CreditAdded]);
        assert_eq!(credited.events.sounds(), &[SoundEvent::CreditAdded]);
        assert_eq!(credited.scene.frame, credited.report.step);
        assert!(credited.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL
                && sprite.layer == RenderLayer::Overlay
        }));

        let started = runtime.step_clean_input(
            CleanGameInput {
                start_one: true,
                ..CleanGameInput::NONE
            },
            XyzzyMode::INACTIVE,
        );
        assert_eq!(started.report.phase, Phase::Playing);
        assert_eq!(started.events.gameplay(), &[GameEvent::GameStarted]);
        assert_eq!(
            started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert!(started.state.world.enemies.is_empty());
        assert_no_source_message(&started.report, "PLYR1", SOURCE_PLAYER_START_PROMPT_SCREEN);
        assert!(started.events.sounds().is_empty());

        let start_sound = runtime.step(GameInput::NONE);
        assert_eq!(start_sound.events.sounds(), &[SoundEvent::GameStarted]);
        assert_eq!(
            start_sound.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS - 1,
                player: 1,
            })
        );

        let settled = step_until_player_start_completes(&mut runtime, 1);
        assert_eq!(settled.state.phase, GamePhase::Playing);
        assert_eq!(settled.state.wave, 1);
        assert_eq!(
            settled.state.player_stocks[0],
            PlayerStockSnapshot::new(3, 3)
        );
        assert_eq!(settled.state.world.humans.len(), 10);
        assert_eq!(
            settled
                .state
                .world
                .enemies
                .iter()
                .filter(|enemy| enemy.kind == CleanEnemyKind::Lander)
                .count(),
            5
        );
        let clean_frame = settled.game_frame();
        assert_eq!(clean_frame.state, settled.state);
        assert_eq!(clean_frame.events, settled.events);
        assert_eq!(clean_frame.scene, settled.scene);
        assert!(settled.scene.sprites.iter().any(|sprite| {
            matches!(
                sprite.sprite,
                SpriteId::PLAYER_SHIP | SpriteId::PLAYER_SHIP_LEFT
            )
        }));
        assert!(
            settled
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER)
        );
    }

    #[test]
    fn actor_runtime_adapter_carries_audio_edge_state_between_frames() {
        let mut runtime = ActorRuntimeAdapter::new();
        runtime.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        runtime.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        step_until_player_start_completes(&mut runtime, 1);

        let thrusting = runtime.step(GameInput {
            thrust: true,
            ..GameInput::NONE
        });
        assert_eq!(thrusting.events.sounds(), &[SoundEvent::ThrustStarted]);

        let still_thrusting = runtime.step(GameInput {
            thrust: true,
            ..GameInput::NONE
        });
        assert!(still_thrusting.events.sounds().is_empty());

        let coasting = runtime.step(GameInput::NONE);
        assert_eq!(coasting.events.sounds(), &[SoundEvent::ThrustStopped]);
    }

    #[test]
    fn attract_actor_accepts_credit_and_start_commands() {
        let mut driver = ActorGameDriver::new();

        let credited = driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.phase, Phase::Attract);
        assert_eq!(credited.credits, 1);
        assert!(credited.sounds.contains(&SoundCue::Credit));
        assert!(
            credited
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::WilliamsLogo
                    && matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
        );
        assert!(
            credited
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(source_credits_label_text()))
        );
        assert!(
            credited
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("01"))
        );

        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.phase, Phase::Playing);
        assert_eq!(started.credits, 0);
        assert!(!started.sounds.contains(&SoundCue::Start));
        assert_eq!(
            started.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert_no_source_message(&started, "PLYR1", SOURCE_PLAYER_START_PROMPT_SCREEN);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        let start_sound = driver.step(GameInput::NONE);
        assert_eq!(start_sound.phase, Phase::Playing);
        assert_eq!(start_sound.sounds, [SoundCue::Start]);
        assert_eq!(
            start_sound.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS - 1,
                player: 1,
            })
        );
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        let settled = step_until_driver_player_start_completes(&mut driver, 1);
        assert_eq!(settled.phase, Phase::Playing);
        assert!(
            settled
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 1 })
        );
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert_eq!(driver.snapshot_count(ActorKind::Human), 10);
    }

    #[test]
    fn attract_title_uses_williams_animation_and_defender_coalescence() {
        let mut driver = ActorGameDriver::new();

        let williams = driver.step(GameInput::NONE);
        assert!(williams.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::WilliamsLogo
                && matches!(
                    draw.effect,
                    VisualEffect::WilliamsReveal {
                        stroke_step: 1,
                        color_phase: 0,
                    }
                )
        }));
        assert!(
            williams
                .draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some("HIGH SCORES"))
        );
        assert!(
            williams
                .draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some("1. 010000"))
        );
        assert!(
            williams
                .draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some(source_credits_label_text()))
        );
        assert!(
            williams
                .draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some("00"))
        );
        let presents_text =
            source_message_text(SOURCE_PRESENTS_MESSAGE_LABEL).expect("ELECV source message");
        assert!(
            !williams
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(presents_text))
        );
        assert!(
            !williams
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderCoalescence)
        );

        let mut presents = None;
        for _ in 1..SOURCE_ATTRACT_PRESENTS_START_STEP {
            presents = Some(driver.step(GameInput::NONE));
        }
        let presents = presents.expect("presents page should be reached");
        assert!(presents.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(presents_text)
                && matches!(
                    draw.effect,
                    VisualEffect::SourceMessage {
                        top_left_screen_address: SOURCE_ATTRACT_PRESENTS_ELECTRONICS_SCREEN,
                        visual_offset: Point { x: 0, y: 0 },
                    }
                )
        }));
        let presents_scene = ActorRenderSceneBridge::new().render_scene_for_report(&presents);
        assert!(presents_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E
                && sprite.position == [100.0, 88.0]
                && sprite.layer == RenderLayer::Overlay
        }));

        let mut coalescing = None;
        for _ in SOURCE_ATTRACT_PRESENTS_START_STEP..DEFENDER_WORDMARK_START_STEP {
            let step = driver.step(GameInput::NONE);
            if step
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderCoalescence)
            {
                coalescing = Some(step);
                break;
            }
        }
        let coalescing = coalescing.expect("wordmark should enter coalescence");
        assert!(coalescing.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::DefenderCoalescence
                && matches!(
                    draw.effect,
                    VisualEffect::DefenderCoalescence {
                        slot: 0,
                        row_pair: 0,
                    }
                )
        }));

        let mut settled = None;
        for _ in 0..(DEFENDER_WORDMARK_SLOTS * DEFENDER_WORDMARK_ROW_PAIRS + 1) {
            let step = driver.step(GameInput::NONE);
            if step
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderWordmark)
            {
                settled = Some(step);
                break;
            }
        }
        let settled = settled.expect("wordmark should settle before hall-of-fame");
        assert!(settled.step < SOURCE_ATTRACT_HALL_OF_FAME_START_STEP);

        let mut hall = settled;
        while hall.step < SOURCE_ATTRACT_HALL_OF_FAME_START_STEP {
            hall = driver.step(GameInput::NONE);
        }
        assert_eq!(hall.step, SOURCE_ATTRACT_HALL_OF_FAME_START_STEP);
        assert!(
            hall.draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some("HIGH SCORES"))
        );
        assert!(
            hall.draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some("1. 010000"))
        );
        assert!(
            hall.draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(source_credits_label_text()))
        );
        assert!(
            hall.draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("00"))
        );
        let hall_title_text =
            source_message_text(SOURCE_ATTRACT_HALL_TITLE_LABEL).expect("Hall title source text");
        assert!(hall.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(hall_title_text)
                && matches!(
                    draw.effect,
                    VisualEffect::SourceMessage {
                        top_left_screen_address: 0x3854,
                        visual_offset: SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
                    }
                )
        }));
        let hall_greatest_text = source_message_text(SOURCE_ATTRACT_HALL_GREATEST_LABEL)
            .expect("Hall greatest source text");
        assert_eq!(
            hall.draws
                .iter()
                .filter(|draw| draw.text.as_deref() == Some(hall_greatest_text))
                .count(),
            2
        );
        assert!(hall.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::DefenderLogo
                && draw.position == SOURCE_ATTRACT_HALL_DEFENDER_LOGO_POSITION
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(37, 128) && draw.text.as_deref() == Some("1")
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(47, 128) && draw.text.as_deref() == Some("DRJ")
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(75, 128) && draw.text.as_deref() == Some(" 10000")
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(167, 128) && draw.text.as_deref() == Some("1")
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(177, 128) && draw.text.as_deref() == Some("DRJ")
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(205, 198) && draw.text.as_deref() == Some("  6010")
        }));
        let hall_scene = ActorRenderSceneBridge::new().render_scene_for_report(&hall);
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_H
                && sprite.position == [101.0, 78.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_D
                && sprite.position == [47.0, 128.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.position == [79.0, 128.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position
                    == [
                        f32::from(SOURCE_ATTRACT_HALL_DEFENDER_LOGO_POSITION.x),
                        f32::from(SOURCE_ATTRACT_HALL_DEFENDER_LOGO_POSITION.y),
                    ]
                && sprite.layer == RenderLayer::Overlay
        }));

        let mut scoring = hall;
        while scoring.step < SOURCE_ATTRACT_SCORING_SEQUENCE_START_STEP {
            scoring = driver.step(GameInput::NONE);
        }
        assert_eq!(scoring.step, SOURCE_ATTRACT_SCORING_SEQUENCE_START_STEP);
        assert!(
            scoring
                .draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some(hall_title_text))
        );
        assert!(
            scoring
                .draws
                .iter()
                .all(|draw| draw.sprite != SpriteKey::DefenderLogo)
        );
        let scan_text = source_message_text("SCANV").expect("scanner instruction source message");
        assert!(scoring.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(scan_text)
                && matches!(
                    draw.effect,
                    VisualEffect::SourceMessage {
                        top_left_screen_address: 0x4330,
                        visual_offset: SOURCE_ATTRACT_SCORING_VISUAL_OFFSET,
                    }
                )
        }));
        for (label, _) in SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES.iter().skip(1) {
            let text = source_message_text(label).expect("instruction source message");
            assert!(
                !scoring
                    .draws
                    .iter()
                    .any(|draw| draw.text.as_deref() == Some(text)),
                "{label} should wait for the score-card reveal cadence"
            );
        }
        assert!(scoring.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::Text
                && matches!(
                    draw.effect,
                    VisualEffect::AttractScoringSurface { scoring_tick: 0 }
                )
        }));
        let scoring_scene = ActorRenderSceneBridge::new().render_scene_for_report(&scoring);
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_S
                && sprite.position == [123.0, 41.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(!scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_L
                && sprite.position == [45.0, 105.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert_eq!(
            scoring_scene
                .sprites
                .iter()
                .filter(|sprite| sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD)
                .count(),
            SOURCE_TOP_DISPLAY_BORDER_SEGMENTS.len()
        );
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [-11.0, 33.0]
                && sprite.size == [312.0, 2.0]
                && sprite.tint == ATTRACT_SCORING_SCANNER_BORDER_TINT
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [85.0, 30.0]
                && sprite.size == ATTRACT_SCORING_SCANNER_TERRAIN_PIXEL_SIZE
                && sprite.tint == ATTRACT_SCORING_SCANNER_TERRAIN_TINT
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_SHIP
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [49.0, 70.0]
                && sprite.size == PLAYER_SHIP_SCENE_SIZE
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [225.0, 85.0]
                && sprite.size == HUMAN_SCENE_SIZE
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_PLAYER_BLIP
                && sprite.layer == RenderLayer::Hud
                && (sprite.position[0] - 98.6).abs() < 0.01
                && (sprite.position[1] - 3.0).abs() < 0.01
                && sprite.size == ATTRACT_SCORING_PLAYER_SCANNER_SIZE
                && sprite.tint == source_pseudo_color_tint(0x99)
        }));
        assert_eq!(
            scoring_scene
                .sprites
                .iter()
                .filter(|sprite| sprite.sprite == SpriteId::SCANNER_OBJECT_BLIP)
                .count(),
            2
        );
        let mut rescue_score = scoring;
        let rescue_score_tick = actor_attract_scoring_tick_for_display_step(
            ATTRACT_SCORING_PROTECTED_DEMO_STEP_OFFSET + ATTRACT_SCORING_RESCUE_FALL_STEPS,
        );
        for _ in 0..rescue_score_tick {
            rescue_score = driver.step(GameInput::NONE);
        }
        let rescue_score_scene =
            ActorRenderSceneBridge::new().render_scene_for_report(&rescue_score);
        assert!(rescue_score_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_POPUP_500
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [225.0, 134.0]
                && sprite.size == SCORE_POPUP_SCENE_SIZE
        }));
        let lander_label_start = actor_attract_scoring_instruction_text_start_step(1);
        let mut lander_label = rescue_score;
        while lander_label.step < lander_label_start {
            lander_label = driver.step(GameInput::NONE);
        }
        assert_eq!(lander_label.step, lander_label_start);
        let lander_text = source_message_text("LANDV").expect("lander instruction source message");
        let mutant_text = source_message_text("MUTV").expect("mutant instruction source message");
        assert!(lander_label.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(lander_text)
                && matches!(
                    draw.effect,
                    VisualEffect::SourceMessage {
                        top_left_screen_address: 0x1C70,
                        visual_offset: SOURCE_ATTRACT_SCORING_VISUAL_OFFSET,
                    }
                )
        }));
        assert!(
            !lander_label
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(mutant_text))
        );
        let lander_label_scene =
            ActorRenderSceneBridge::new().render_scene_for_report(&lander_label);
        assert!(lander_label_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_L
                && sprite.position == [45.0, 105.0]
                && sprite.layer == RenderLayer::Overlay
        }));

        let last_label_start = actor_attract_scoring_instruction_text_start_step(
            SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES.len() - 1,
        );
        let mut last_label = lander_label;
        while last_label.step < last_label_start {
            last_label = driver.step(GameInput::NONE);
        }
        assert_eq!(last_label.step, last_label_start);
        for (label, screen_address) in SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES {
            let text = source_message_text(label).expect("instruction source message");
            assert!(last_label.draws.iter().any(|draw| {
                draw.text.as_deref() == Some(text)
                    && matches!(
                        draw.effect,
                        VisualEffect::SourceMessage {
                            top_left_screen_address,
                            visual_offset: SOURCE_ATTRACT_SCORING_VISUAL_OFFSET,
                        } if top_left_screen_address == screen_address
                    )
            }));
        }
        assert!(!scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position
                    == [
                        f32::from(SOURCE_ATTRACT_HALL_DEFENDER_LOGO_POSITION.x),
                        f32::from(SOURCE_ATTRACT_HALL_DEFENDER_LOGO_POSITION.y),
                    ]
        }));
    }

    #[test]
    fn actor_attract_scoring_surface_projects_laser_fragments_and_legend_transfer() {
        let mut laser_scene = RenderScene::empty(0, ACTOR_RENDER_SURFACE);
        let rescue_laser_step =
            actor_attract_scoring_display_step_for_stage(ActorAttractScoringStage::RescueLaser, 2);
        push_attract_scoring_demo_scene(
            &mut laser_scene,
            actor_attract_scoring_tick_for_display_step(rescue_laser_step),
        );
        assert!(laser_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.layer == RenderLayer::Projectiles
                && sprite.size == PLAYER_EXPLOSION_PIXEL_SCENE_SIZE
                && sprite.tint == SOURCE_LASER_TIP_TINT
        }));
        assert!(laser_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.layer == RenderLayer::Projectiles
                && sprite.size == PLAYER_EXPLOSION_PIXEL_SCENE_SIZE
                && sprite.tint == SOURCE_LASER_BODY_TINT
        }));

        let mut explosion_scene = RenderScene::empty(0, ACTOR_RENDER_SURFACE);
        let rescue_fall_step =
            actor_attract_scoring_display_step_for_stage(ActorAttractScoringStage::RescueFall, 5);
        push_attract_scoring_demo_scene(
            &mut explosion_scene,
            actor_attract_scoring_tick_for_display_step(rescue_fall_step),
        );
        assert!(explosion_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                && sprite.layer == RenderLayer::Objects
                && sprite.size == PLAYER_EXPLOSION_PIXEL_SCENE_SIZE
        }));
        assert!(!explosion_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER && sprite.layer == RenderLayer::Objects
        }));

        let mut transfer_scene = RenderScene::empty(0, ACTOR_RENDER_SURFACE);
        let legend_transfer_step = actor_attract_scoring_display_step_for_stage(
            ActorAttractScoringStage::LegendTransfer(0),
            0,
        );
        push_attract_scoring_demo_scene(
            &mut transfer_scene,
            actor_attract_scoring_tick_for_display_step(legend_transfer_step),
        );
        assert!(transfer_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_POPUP_250
                && sprite.layer == RenderLayer::Objects
                && sprite.position == actor_attract_scoring_scene_position(0x07A0, 0x5900)
                && sprite.size == SCORE_POPUP_SCENE_SIZE
        }));
        assert!(
            transfer_scene
                .sprites
                .iter()
                .filter(|sprite| {
                    sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                        && sprite.layer == RenderLayer::Objects
                        && sprite.size == PLAYER_EXPLOSION_PIXEL_SCENE_SIZE
                })
                .count()
                > 8
        );

        let mut reveal_scene = RenderScene::empty(0, ACTOR_RENDER_SURFACE);
        let legend_reveal_step = actor_attract_scoring_display_step_for_stage(
            ActorAttractScoringStage::LegendReveal(0),
            0,
        );
        push_attract_scoring_demo_scene(
            &mut reveal_scene,
            actor_attract_scoring_tick_for_display_step(legend_reveal_step),
        );
        assert!(reveal_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER
                && sprite.layer == RenderLayer::Objects
                && sprite.position == actor_attract_scoring_scene_position(0x07A0, 0x5900)
                && sprite.size == LANDER_SCENE_SIZE
        }));
    }

    #[test]
    fn embedded_actor_attract_script_matches_red_label_constructor_fallback() {
        let parsed = AttractScript::parse_text(ACTOR_RED_LABEL_ATTRACT_SCRIPT)
            .expect("embedded actor attract script should parse");

        assert_eq!(
            parsed.manifest().cycle_steps,
            Some(SOURCE_ATTRACT_CYCLE_STEPS)
        );
        assert_eq!(
            AttractScript::red_label_title().manifest(),
            parsed.manifest()
        );
        assert_eq!(
            AttractScript::red_label_title_from_events().manifest(),
            parsed.manifest()
        );
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::WilliamsLogo { .. }
        ) && event.start_after_steps == 1
            && event.duration_steps == Some(SOURCE_ATTRACT_WILLIAMS_LOGO_DURATION_STEPS)));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::SourceMessage {
                ref label,
                top_left_screen_address: SOURCE_ATTRACT_PRESENTS_ELECTRONICS_SCREEN,
                visual_offset: Point { x: 0, y: 0 },
            } if label == SOURCE_PRESENTS_MESSAGE_LABEL
        ) && event.start_after_steps
            == SOURCE_ATTRACT_PRESENTS_START_STEP
            && event.duration_steps == Some(SOURCE_ATTRACT_PRESENTS_DURATION_STEPS)));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::DefenderWordmark { .. }
        ) && event.start_after_steps
            == SOURCE_ATTRACT_DEFENDER_WORDMARK_START_STEP
            && event.duration_steps == Some(SOURCE_ATTRACT_DEFENDER_WORDMARK_DURATION_STEPS)));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::HallScores {
                todays_top_left_screen_address: SOURCE_ATTRACT_HALL_TODAYS_TABLE_SCREEN,
                all_time_top_left_screen_address: SOURCE_ATTRACT_HALL_ALL_TIME_TABLE_SCREEN,
                visual_offset: SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            } if event.start_after_steps == SOURCE_ATTRACT_HALL_OF_FAME_START_STEP
                && event.duration_steps == Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS)
        )));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::Credits {
                label_position: SOURCE_ATTRACT_CREDIT_LABEL_POSITION,
                count_position: SOURCE_ATTRACT_CREDIT_COUNT_POSITION,
                minimum_credits: 1,
            } if event.start_after_steps == 1
                && event.duration_steps
                    == Some(SOURCE_ATTRACT_HALL_OF_FAME_START_STEP.saturating_sub(1))
        )));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::Credits {
                label_position: SOURCE_ATTRACT_CREDIT_LABEL_POSITION,
                count_position: SOURCE_ATTRACT_CREDIT_COUNT_POSITION,
                minimum_credits: 0,
            } if event.start_after_steps == SOURCE_ATTRACT_HALL_OF_FAME_START_STEP
                && event.duration_steps.is_none()
        )));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::SourceMessage {
                ref label,
                top_left_screen_address: 0x3854,
                visual_offset: SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            } if event.start_after_steps == SOURCE_ATTRACT_HALL_OF_FAME_START_STEP
                && label == SOURCE_ATTRACT_HALL_TITLE_LABEL
                && event.duration_steps == Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS)
        )));
        assert_eq!(
            parsed
                .manifest()
                .events
                .iter()
                .filter(|event| matches!(
                    event.action,
                    AttractScriptActionManifest::SourceMessage {
                        ref label,
                        top_left_screen_address: 0x1E72 | 0x5F72,
                        visual_offset: SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
                    } if event.start_after_steps == SOURCE_ATTRACT_HALL_OF_FAME_START_STEP
                        && label == SOURCE_ATTRACT_HALL_GREATEST_LABEL
                        && event.duration_steps == Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS)
                ))
                .count(),
            2
        );
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::Sprite {
                sprite: SpriteKey::DefenderLogo,
                position: SOURCE_ATTRACT_HALL_DEFENDER_LOGO_POSITION,
            } if event.start_after_steps == SOURCE_ATTRACT_HALL_OF_FAME_START_STEP
                && event.duration_steps == Some(SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS)
        )));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::ScoringSurface
                if event.start_after_steps == SOURCE_ATTRACT_SCORING_SEQUENCE_START_STEP
                    && event.duration_steps.is_none()
        )));
        for (line_index, (label, screen_address)) in SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES
            .iter()
            .copied()
            .enumerate()
        {
            assert!(parsed.manifest().events.iter().any(|event| matches!(
                event.action,
                AttractScriptActionManifest::SourceMessage {
                    label: ref event_label,
                    top_left_screen_address,
                    visual_offset: SOURCE_ATTRACT_SCORING_VISUAL_OFFSET,
                } if event.start_after_steps
                    == actor_attract_scoring_instruction_text_start_step(line_index)
                    && event.duration_steps.is_none()
                    && event_label == label
                    && top_left_screen_address == screen_address
            )));
        }
    }

    #[test]
    fn actor_attract_scoring_instruction_labels_follow_source_reveal_cadence() {
        assert_eq!(
            SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES
                .iter()
                .enumerate()
                .map(|(index, _)| actor_attract_scoring_instruction_text_start_step(index))
                .collect::<Vec<_>>(),
            vec![1088, 1505, 1691, 1871, 2051, 2237, 2417]
        );

        let parsed = AttractScript::parse_text(ACTOR_RED_LABEL_ATTRACT_SCRIPT)
            .expect("embedded actor attract script should parse");
        for (line_index, (label, _)) in SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES
            .iter()
            .copied()
            .enumerate()
        {
            assert!(parsed.manifest().events.iter().any(|event| matches!(
                event.action,
                AttractScriptActionManifest::SourceMessage {
                    label: ref event_label,
                    ..
                } if event_label == label
                    && event.start_after_steps
                        == actor_attract_scoring_instruction_text_start_step(line_index)
                    && event.duration_steps.is_none()
            )));
        }
    }

    #[test]
    fn default_actor_attract_script_loops_after_source_scoring_cycle() {
        let script = AttractScript::red_label_title();
        let high_scores = HighScoreTable::default().entries;

        assert_eq!(
            script.manifest().cycle_steps,
            Some(SOURCE_ATTRACT_CYCLE_STEPS)
        );
        assert_eq!(SOURCE_ATTRACT_CYCLE_STEPS, 3367);

        let final_scoring_draws = script.draws_for(
            ActorId::new(99),
            SOURCE_ATTRACT_CYCLE_STEPS - 1,
            &high_scores,
            0,
        );
        assert!(
            final_scoring_draws
                .iter()
                .any(|draw| { matches!(draw.effect, VisualEffect::AttractScoringSurface { .. }) })
        );
        assert!(
            !final_scoring_draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::WilliamsLogo)
        );

        let wrapped_draws = script.draws_for(
            ActorId::new(99),
            SOURCE_ATTRACT_CYCLE_STEPS,
            &high_scores,
            0,
        );
        assert!(wrapped_draws.iter().any(|draw| {
            draw.sprite == SpriteKey::WilliamsLogo
                && matches!(
                    draw.effect,
                    VisualEffect::WilliamsReveal { stroke_step: 1, .. }
                )
        }));
        assert!(
            !wrapped_draws
                .iter()
                .any(|draw| { matches!(draw.effect, VisualEffect::AttractScoringSurface { .. }) })
        );
        let scan_text = source_message_text("SCANV").expect("SCANV source message");
        assert!(
            !wrapped_draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(scan_text))
        );
    }

    #[test]
    fn actor_source_scanner_mini_terrain_records_match_reference_slice() {
        let records = source_scanner_mini_terrain_records();

        assert_eq!(records.len(), SOURCE_SCANNER_TERRAIN_RECORDS);
        assert_eq!(
            &records[..8],
            &[
                SourceScannerTerrainRecord {
                    screen_address: 0x3025,
                    word: 0x7700,
                },
                SourceScannerTerrainRecord {
                    screen_address: 0x3124,
                    word: 0x7700,
                },
                SourceScannerTerrainRecord {
                    screen_address: 0x3222,
                    word: 0x0770,
                },
                SourceScannerTerrainRecord {
                    screen_address: 0x3320,
                    word: 0x0770,
                },
                SourceScannerTerrainRecord {
                    screen_address: 0x341E,
                    word: 0x0770,
                },
                SourceScannerTerrainRecord {
                    screen_address: 0x351C,
                    word: 0x0770,
                },
                SourceScannerTerrainRecord {
                    screen_address: 0x361D,
                    word: 0x7007,
                },
                SourceScannerTerrainRecord {
                    screen_address: 0x371F,
                    word: 0x7007,
                },
            ]
        );
        assert_eq!(
            &source_mterr_bytes()[..9],
            &[0x25, 0x70, 0x07, 0x26, 0x77, 0x00, 0x26, 0x07, 0x70]
        );
    }

    #[test]
    fn custom_driver_can_script_its_own_attract_screen() {
        let script = AttractScript::new(vec![
            AttractScriptEvent::text(2, Some(5), Point::new(12, 20), "CUSTOM ATTRACT"),
            AttractScriptEvent::sprite(3, None, SpriteKey::DefenderLogo, Point::new(40, 44)),
            AttractScriptEvent::defender_wordmark(4, None, Point::new(70, 80)),
        ]);
        let mut driver = ActorGameDriver::with_attract_script(script);

        let first = driver.step(GameInput::NONE);
        assert!(!first.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("CUSTOM ATTRACT")
                || draw.sprite == SpriteKey::DefenderLogo
                || draw.sprite == SpriteKey::DefenderCoalescence
        }));

        let second = driver.step(GameInput::NONE);
        assert!(
            second
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("CUSTOM ATTRACT"))
        );
        assert!(
            !second
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderLogo)
        );

        let third = driver.step(GameInput::NONE);
        assert!(
            third
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderLogo)
        );

        let fourth = driver.step(GameInput::NONE);
        assert!(fourth.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::DefenderCoalescence
                && matches!(
                    draw.effect,
                    VisualEffect::DefenderCoalescence {
                        slot: 0,
                        row_pair: 0
                    }
                )
        }));
    }

    #[test]
    fn attract_script_manifest_exposes_custom_driver_events() {
        let script = AttractScript::new(vec![
            AttractScriptEvent::defender_wordmark(9, Some(12), Point::new(70, 80)),
            AttractScriptEvent::text(2, Some(5), Point::new(12, 20), "CUSTOM ATTRACT"),
            AttractScriptEvent::williams_logo(5, None, Point::new(18, 44)),
        ]);
        let mut driver = ActorGameDriver::with_attract_script(script.clone());

        let manifest = driver.script_manifest();

        assert_eq!(manifest.attract_script.cycle_steps, None);
        assert_eq!(manifest.attract_script, script.manifest());
        assert_eq!(
            manifest
                .attract_script
                .events
                .iter()
                .map(|event| event.start_after_steps)
                .collect::<Vec<_>>(),
            vec![2, 5, 9]
        );
        assert_eq!(
            manifest.attract_script.events[0].action,
            AttractScriptActionManifest::Text {
                position: Point::new(12, 20),
                value: "CUSTOM ATTRACT".to_string(),
            }
        );
        assert_eq!(
            manifest.attract_script.events[1].action,
            AttractScriptActionManifest::WilliamsLogo {
                position: Point::new(18, 44),
                reveal_steps: WILLIAMS_REVEAL_STEPS,
                color_period: WILLIAMS_COLOR_PERIOD,
            }
        );
        assert_eq!(
            manifest.attract_script.events[2].action,
            AttractScriptActionManifest::DefenderWordmark {
                position: Point::new(70, 80),
                slots: DEFENDER_WORDMARK_SLOTS,
                row_pairs: DEFENDER_WORDMARK_ROW_PAIRS,
            }
        );

        driver.step(GameInput::NONE);
        assert_eq!(driver.script_manifest().attract_script, script.manifest());
    }

    #[test]
    fn attract_script_text_parser_builds_sorted_event_manifest() {
        let script = AttractScript::parse_text(
            "\
            # Custom attract script\n\
            cycle 12\n\
            defender_wordmark 9 12 70 80\n\
            text 2 5 12 20 CUSTOM ATTRACT\n\
            high_scores 4 forever 80 100 9 3\n\
            hall_scores 4 forever 0x1886 0x5986 -11 -6\n\
            scoring_surface 4 forever\n\
            credits 4 forever 12 228 82 228\n\
            credits_nonzero 4 8 14 226 84 226\n\
            sprite 6 forever defender_logo 40 44\n\
            williams_logo 5 - 18 44\n",
        )
        .expect("custom attract script text should parse");

        let manifest = script.manifest();

        assert_eq!(manifest.cycle_steps, Some(12));
        assert_eq!(
            manifest
                .events
                .iter()
                .map(|event| event.start_after_steps)
                .collect::<Vec<_>>(),
            vec![2, 4, 4, 4, 4, 4, 5, 6, 9]
        );
        assert_eq!(
            manifest.events[0].action,
            AttractScriptActionManifest::Text {
                position: Point::new(12, 20),
                value: "CUSTOM ATTRACT".to_string(),
            }
        );
        assert_eq!(
            manifest.events[1].action,
            AttractScriptActionManifest::HighScores {
                position: Point::new(80, 100),
                row_height: 9,
                rows: 3,
            }
        );
        assert_eq!(
            manifest.events[2].action,
            AttractScriptActionManifest::HallScores {
                todays_top_left_screen_address: SOURCE_ATTRACT_HALL_TODAYS_TABLE_SCREEN,
                all_time_top_left_screen_address: SOURCE_ATTRACT_HALL_ALL_TIME_TABLE_SCREEN,
                visual_offset: SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            }
        );
        assert_eq!(
            manifest.events[3].action,
            AttractScriptActionManifest::ScoringSurface
        );
        assert_eq!(
            manifest.events[4].action,
            AttractScriptActionManifest::Credits {
                label_position: Point::new(12, 228),
                count_position: Point::new(82, 228),
                minimum_credits: 0,
            }
        );
        assert_eq!(
            manifest.events[5].action,
            AttractScriptActionManifest::Credits {
                label_position: Point::new(14, 226),
                count_position: Point::new(84, 226),
                minimum_credits: 1,
            }
        );
        assert_eq!(manifest.events[5].duration_steps, Some(8));
        assert_eq!(
            manifest.events[7].action,
            AttractScriptActionManifest::Sprite {
                sprite: SpriteKey::DefenderLogo,
                position: Point::new(40, 44),
            }
        );
        assert_eq!(
            manifest.events[8].action,
            AttractScriptActionManifest::DefenderWordmark {
                position: Point::new(70, 80),
                slots: DEFENDER_WORDMARK_SLOTS,
                row_pairs: DEFENDER_WORDMARK_ROW_PAIRS,
            }
        );
    }

    #[test]
    fn custom_attract_scripts_only_loop_when_cycle_is_declared() {
        let high_scores = HighScoreTable::default().entries;
        let unlooped = AttractScript::parse_text("text 2 forever 12 20 UNBOUNDED")
            .expect("custom unlooped script should parse");
        let unlooped_draws = unlooped.draws_for(ActorId::new(1), 12, &high_scores, 0);
        assert!(
            unlooped_draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("UNBOUNDED"))
        );

        let looped = AttractScript::parse_text(
            "\
            cycle 5\n\
            text 2 forever 12 20 LOOPED\n",
        )
        .expect("custom looped script should parse");
        let wrapped_to_first_step = looped.draws_for(ActorId::new(1), 5, &high_scores, 0);
        assert!(
            wrapped_to_first_step
                .iter()
                .all(|draw| draw.text.as_deref() != Some("LOOPED"))
        );
        let wrapped_to_second_step = looped.draws_for(ActorId::new(1), 7, &high_scores, 0);
        assert!(
            wrapped_to_second_step
                .iter()
                .any(|draw| draw.text.as_deref() == Some("LOOPED"))
        );
    }

    #[test]
    fn parsed_attract_script_drives_draws_and_preserves_start_controls() {
        let script = "\
            text 1 forever 10 10 PRESS START\n\
            sprite 2 forever defender_logo 40 44\n"
            .parse::<AttractScript>()
            .expect("script text should parse");
        let mut driver = ActorGameDriver::with_attract_script(script);

        let first = driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(first.credits, 1);
        assert!(
            first
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("PRESS START"))
        );

        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.phase, Phase::Playing);
        assert!(!started.sounds.contains(&SoundCue::Start));
        assert!(
            started
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::StartOnePlayer))
        );
        let start_sound = driver.step(GameInput::NONE);
        assert_eq!(start_sound.sounds, [SoundCue::Start]);
    }

    #[test]
    fn parsed_attract_script_draws_prompt_high_score_rows() {
        let script = "high_scores 1 forever 20 40 8 3"
            .parse::<AttractScript>()
            .expect("high-score script action should parse");
        let mut driver = ActorGameDriver::with_attract_script(script);
        driver.high_scores.record(12_000);

        let report = driver.step(GameInput::NONE);

        assert!(report.draws.iter().any(|draw| {
            draw.position == Point::new(20, 40) && draw.text.as_deref() == Some("1. 012000")
        }));
        assert!(report.draws.iter().any(|draw| {
            draw.position == Point::new(20, 48) && draw.text.as_deref() == Some("2. 010000")
        }));
        assert!(report.draws.iter().any(|draw| {
            draw.position == Point::new(20, 56) && draw.text.as_deref() == Some("3. 007500")
        }));
    }

    #[test]
    fn parsed_attract_script_draws_prompt_credit_count() {
        let script = "credits 1 forever 12 228 82 228"
            .parse::<AttractScript>()
            .expect("credit script action should parse");
        let mut driver = ActorGameDriver::with_attract_script(script);
        let source_credits_label = source_message_text(SOURCE_CREDITS_MESSAGE_LABEL)
            .expect("CREDV source message should be checked in");

        let first = driver.step(GameInput::NONE);
        assert!(first.draws.iter().any(|draw| {
            draw.position == Point::new(12, 228)
                && draw.text.as_deref() == Some(source_credits_label)
        }));
        assert!(first.draws.iter().any(|draw| {
            draw.position == Point::new(82, 228) && draw.text.as_deref() == Some("00")
        }));

        let credited = driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.credits, 1);
        assert!(credited.draws.iter().any(|draw| {
            draw.position == Point::new(82, 228) && draw.text.as_deref() == Some("01")
        }));
    }

    #[test]
    fn parsed_attract_script_draws_source_message_with_controls() {
        let script = "message 1 forever ELECV 0x3258"
            .parse::<AttractScript>()
            .expect("source message script action should parse");
        assert_eq!(
            script.manifest().events[0].action,
            AttractScriptActionManifest::SourceMessage {
                label: "ELECV".to_string(),
                top_left_screen_address: 0x3258,
                visual_offset: Point::new(0, 0),
            }
        );
        let mut driver = ActorGameDriver::with_attract_script(script);
        let source_text = source_message_text("ELECV").expect("ELECV source message");

        let report = driver.step(GameInput::NONE);
        assert!(report.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(source_text)
                && matches!(
                    draw.effect,
                    VisualEffect::SourceMessage {
                        top_left_screen_address: 0x3258,
                        visual_offset: Point { x: 0, y: 0 },
                    }
                )
        }));

        let scene = ActorRenderSceneBridge::new().render_scene_for_report(&report);
        assert_eq!(scene.sprites.len(), 23);
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E
                && sprite.position == [100.0, 88.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.position == [124.0, 108.0]
                && sprite.layer == RenderLayer::Overlay
        }));
    }

    #[test]
    fn parsed_attract_script_draws_source_message_with_visual_offset() {
        let script = "message 1 forever ELECV 0x3258 -11 -7"
            .parse::<AttractScript>()
            .expect("source message script action should parse with offset");
        assert_eq!(
            script.manifest().events[0].action,
            AttractScriptActionManifest::SourceMessage {
                label: "ELECV".to_string(),
                top_left_screen_address: 0x3258,
                visual_offset: SOURCE_ATTRACT_SCORING_VISUAL_OFFSET,
            }
        );
        let mut driver = ActorGameDriver::with_attract_script(script);
        let source_text = source_message_text("ELECV").expect("ELECV source message");

        let report = driver.step(GameInput::NONE);
        assert!(report.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(source_text)
                && matches!(
                    draw.effect,
                    VisualEffect::SourceMessage {
                        top_left_screen_address: 0x3258,
                        visual_offset: SOURCE_ATTRACT_SCORING_VISUAL_OFFSET,
                    }
                )
        }));

        let scene = ActorRenderSceneBridge::new().render_scene_for_report(&report);
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E
                && sprite.position == [89.0, 81.0]
                && sprite.layer == RenderLayer::Overlay
        }));
    }

    #[test]
    fn attract_script_text_parser_reports_line_errors() {
        let error = AttractScript::parse_text("text 1 forever 10\n")
            .expect_err("missing text y coordinate should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("missing y"));

        let error = AttractScript::parse_text("sprite 1 forever no_such_sprite 1 2\n")
            .expect_err("unknown sprite key should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown sprite key"));

        let error = AttractScript::parse_text("message 1 forever NO_SUCH_MESSAGE 0x3258\n")
            .expect_err("unknown source message label should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown source message label"));

        let error =
            AttractScript::parse_text("cycle 0\n").expect_err("zero cycle length should fail");
        assert_eq!(error.line, 1);
        assert!(
            error
                .to_string()
                .contains("cycle steps must be greater than zero")
        );

        let error = AttractScript::parse_text("cycle 12\ncycle 13\n")
            .expect_err("duplicate cycle directive should fail");
        assert_eq!(error.line, 2);
        assert!(error.to_string().contains("duplicate cycle directive"));
    }

    #[test]
    fn custom_attract_script_keeps_coin_and_start_controls() {
        let script = AttractScript::new(vec![AttractScriptEvent::text(
            1,
            None,
            Point::new(10, 10),
            "PRESS START",
        )]);
        let mut driver = ActorGameDriver::with_attract_script(script);

        let credited = driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.credits, 1);

        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.phase, Phase::Playing);
        assert!(!started.sounds.contains(&SoundCue::Start));
        let start_sound = driver.step(GameInput::NONE);
        assert_eq!(start_sound.sounds, [SoundCue::Start]);
    }

    #[test]
    fn actor_two_player_start_requires_two_credits() {
        let mut driver = ActorGameDriver::new();
        driver.credits = 1;
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let blocked = runtime.step(GameInput {
            start_two: true,
            ..GameInput::NONE
        });

        assert_eq!(blocked.report.phase, Phase::Attract);
        assert_eq!(blocked.report.credits, 1);
        assert_eq!(blocked.state.phase, GamePhase::Attract);
        assert_eq!(blocked.state.credits, 1);
        assert_eq!(blocked.state.current_player, 1);
        assert_eq!(blocked.state.player_count, 1);
        assert_eq!(blocked.state.scores.player_two, 0);
        assert!(!blocked.events.gameplay().contains(&GameEvent::GameStarted));
        assert!(!blocked.report.sounds.contains(&SoundCue::Start));
    }

    #[test]
    fn actor_one_player_start_uses_source_playfield_delay_without_source_prompt() {
        let mut driver = ActorGameDriver::new();
        driver.credits = 1;
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let started = runtime.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        assert_eq!(started.report.phase, Phase::Playing);
        assert_eq!(started.report.credits, 0);
        assert_eq!(started.report.current_player, 1);
        assert_eq!(started.report.player_count, 1);
        assert_eq!(started.events.gameplay(), &[GameEvent::GameStarted]);
        assert!(!started.events.gameplay().contains(&GameEvent::WaveStarted));
        assert!(started.events.sounds().is_empty());
        assert_eq!(
            started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert!(started.report.snapshots.iter().all(|snapshot| !matches!(
            snapshot.kind,
            ActorKind::Player
                | ActorKind::Lander
                | ActorKind::Bomber
                | ActorKind::Pod
                | ActorKind::Human
        )));
        assert!(started.state.world.enemies.is_empty());
        assert!(started.state.world.humans.is_empty());
        assert_no_source_message(&started.report, "PLYR1", SOURCE_PLAYER_START_PROMPT_SCREEN);

        let start_sound = runtime.step(GameInput::NONE);
        assert_eq!(start_sound.events.sounds(), &[SoundEvent::GameStarted]);
        assert_eq!(
            start_sound.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS - 1,
                player: 1,
            })
        );
        assert!(start_sound.state.world.enemies.is_empty());
        assert_no_source_message(
            &start_sound.report,
            "PLYR1",
            SOURCE_PLAYER_START_PROMPT_SCREEN,
        );

        let active = step_until_player_start_completes(&mut runtime, 1);

        assert_eq!(active.report.phase, Phase::Playing);
        assert_eq!(active.report.current_player, 1);
        assert_eq!(active.report.player_count, 1);
        assert!(active.report.player_start.is_none());
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
        assert_eq!(
            active.events.sounds(),
            &[SoundEvent::UnmappedSoundCommand { command: 0xEA }]
        );
        assert_eq!(active.state.world.humans.len(), 10);
        assert!(active.state.world.enemies.iter().any(|enemy| {
            enemy.kind == CleanEnemyKind::Lander
                || enemy.kind == CleanEnemyKind::Bomber
                || enemy.kind == CleanEnemyKind::Pod
        }));
    }

    #[test]
    fn actor_two_player_start_initializes_session_state_bridge() {
        let mut driver = ActorGameDriver::new();
        driver.credits = 2;
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let started = runtime.step(GameInput {
            start_two: true,
            ..GameInput::NONE
        });

        assert_eq!(started.report.phase, Phase::Playing);
        assert_eq!(started.report.credits, 0);
        assert_eq!(started.report.current_player, 1);
        assert_eq!(started.report.player_count, 2);
        assert_eq!(started.report.player_scores, [0, 0]);
        assert_eq!(
            started.report.player_stocks,
            [PlayerStockSnapshot::new(3, 3); 2]
        );
        assert_eq!(started.state.phase, GamePhase::Playing);
        assert_eq!(started.state.credits, 0);
        assert_eq!(started.state.current_player, 1);
        assert_eq!(started.state.player_count, 2);
        assert_eq!(started.state.scores.player_one, 0);
        assert_eq!(started.state.scores.player_two, 0);
        assert_eq!(
            started.state.player_stocks,
            [PlayerStockSnapshot::new(3, 3); 2]
        );
        assert!(!started.report.sounds.contains(&SoundCue::Start));
        assert!(started.events.sounds().is_empty());
        assert!(started.events.gameplay().contains(&GameEvent::GameStarted));
        assert!(!started.events.gameplay().contains(&GameEvent::WaveStarted));
        assert_eq!(
            started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert!(started.state.world.enemies.is_empty());
        assert_source_message(&started.report, "PLYR1", SOURCE_PLAYER_START_PROMPT_SCREEN);
        assert_source_message_scene(&started.scene, "PLYR1", SOURCE_PLAYER_START_PROMPT_SCREEN);

        let status = runtime.step(GameInput::NONE);
        assert_text(&status.report, "2UP 000000");
        assert_eq!(status.events.sounds(), &[SoundEvent::GameStarted]);
        assert_eq!(
            status.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS - 1,
                player: 1,
            })
        );
        assert_source_message(&status.report, "PLYR1", SOURCE_PLAYER_START_PROMPT_SCREEN);
        assert_source_message_scene(&status.scene, "PLYR1", SOURCE_PLAYER_START_PROMPT_SCREEN);

        let active = step_until_player_start_completes(&mut runtime, 1);
        assert_eq!(active.report.phase, Phase::Playing);
        assert_eq!(active.report.current_player, 1);
        assert!(active.report.player_start.is_none());
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
        assert!(active.state.world.enemies.iter().any(|enemy| {
            enemy.kind == CleanEnemyKind::Lander
                || enemy.kind == CleanEnemyKind::Bomber
                || enemy.kind == CleanEnemyKind::Pod
        }));
    }

    #[test]
    fn actor_score_awards_follow_current_player_two_stock() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 2;
        driver.player_count = 2;
        driver.score = 900;
        driver.player_two_score = 9_900;
        driver.next_bonus = SOURCE_REPLAY_SCORE;
        driver.lives = 2;
        driver.smart_bombs = 1;
        driver.player_two_lives = 3;
        driver.player_two_smart_bombs = 1;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(62, 120));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        runtime.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let scored = runtime.step(GameInput::NONE);

        assert_eq!(scored.report.current_player, 2);
        assert_eq!(scored.report.player_scores, [900, 10_050]);
        assert_eq!(scored.report.score, 10_050);
        assert_eq!(scored.report.next_bonus, 20_000);
        assert_eq!(
            scored.report.player_stocks,
            [
                PlayerStockSnapshot::new(2, 1),
                PlayerStockSnapshot::new(4, 2),
            ]
        );
        assert_eq!(scored.state.scores.player_one, 900);
        assert_eq!(scored.state.scores.player_two, 10_050);
        assert_eq!(scored.state.scores.high_score, 10_050);
        assert_eq!(
            scored.state.player_stocks[1],
            PlayerStockSnapshot::new(4, 2)
        );
        assert!(scored.events.gameplay().contains(&GameEvent::BonusAwarded));
    }

    #[test]
    fn actor_two_player_non_final_death_starts_player_switch_sleep() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 1;
        driver.player_count = 2;
        driver.lives = 2;
        driver.smart_bombs = 3;
        driver.player_two_lives = 3;
        driver.player_two_smart_bombs = 3;
        driver.spawn_player();
        driver.spawn_enemy_laser_from_spawn(Point::new(42, 120), Velocity::new(0, 0), None);
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let killed = runtime.step(GameInput::NONE);

        assert_eq!(killed.report.phase, Phase::GameOver);
        assert_eq!(killed.report.current_player, 1);
        assert_eq!(killed.report.lives, 1);
        assert_eq!(
            killed.report.player_stocks,
            [
                PlayerStockSnapshot::new(1, 3),
                PlayerStockSnapshot::new(3, 3),
            ]
        );
        assert_eq!(
            killed.report.player_switch,
            Some(PlayerSwitchReport {
                sleep_steps_remaining: SOURCE_PLAYER_SWITCH_SLEEP_STEPS,
                from_player: 1,
                to_player: 2,
            })
        );
        assert_eq!(
            killed.state.game_over,
            GameOverSnapshot {
                player_switch_sleep_remaining: Some(SOURCE_PLAYER_SWITCH_SLEEP_STEPS),
                player_switch_from: Some(1),
                player_switch_to: Some(2),
                ..GameOverSnapshot::NONE
            }
        );
        assert!(
            killed
                .events
                .gameplay()
                .contains(&GameEvent::PlayerDestroyed)
        );
        assert!(!killed.events.gameplay().contains(&GameEvent::GameOver));
        assert!(!killed.report.sounds.contains(&SoundCue::GameOver));

        let switched = step_until_player_switch_completes(&mut runtime, 2);

        assert_eq!(switched.report.phase, Phase::Playing);
        assert_eq!(switched.report.current_player, 2);
        assert_eq!(switched.report.lives, 3);
        assert_eq!(switched.report.smart_bombs, 3);
        assert_eq!(
            switched.report.player_stocks,
            [
                PlayerStockSnapshot::new(1, 3),
                PlayerStockSnapshot::new(3, 3),
            ]
        );
        assert!(switched.report.player_switch.is_none());
        assert_eq!(
            switched.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS,
                player: 2,
            })
        );
        assert!(switched.state.world.enemies.is_empty());
        assert_source_message(&switched.report, "PLYR2", SOURCE_PLAYER_START_PROMPT_SCREEN);
        assert_source_message_scene(&switched.scene, "PLYR2", SOURCE_PLAYER_START_PROMPT_SCREEN);

        let active = step_until_player_start_completes(&mut runtime, 2);
        assert_eq!(active.report.phase, Phase::Playing);
        assert_eq!(active.report.current_player, 2);
        assert!(active.report.player_start.is_none());
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
        assert!(active.state.world.enemies.iter().any(|enemy| {
            enemy.kind == CleanEnemyKind::Lander
                || enemy.kind == CleanEnemyKind::Bomber
                || enemy.kind == CleanEnemyKind::Pod
        }));
    }

    #[test]
    fn actor_two_player_final_death_switches_to_other_stocked_player() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 1;
        driver.player_count = 2;
        driver.lives = 1;
        driver.smart_bombs = 2;
        driver.player_two_lives = 2;
        driver.player_two_smart_bombs = 1;
        driver.spawn_player();
        driver.spawn_enemy_laser_from_spawn(Point::new(42, 120), Velocity::new(0, 0), None);
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let killed = runtime.step(GameInput::NONE);

        assert_eq!(killed.report.phase, Phase::GameOver);
        assert_eq!(killed.report.lives, 0);
        assert_eq!(
            killed.report.player_stocks,
            [
                PlayerStockSnapshot::new(0, 2),
                PlayerStockSnapshot::new(2, 1),
            ]
        );
        assert_eq!(
            killed.report.player_switch,
            Some(PlayerSwitchReport {
                sleep_steps_remaining: SOURCE_PLAYER_SWITCH_SLEEP_STEPS,
                from_player: 1,
                to_player: 2,
            })
        );
        assert!(!killed.report.sounds.contains(&SoundCue::GameOver));
        assert!(!killed.events.gameplay().contains(&GameEvent::GameOver));

        let switched = step_until_player_switch_completes(&mut runtime, 2);

        assert_eq!(switched.report.phase, Phase::Playing);
        assert_eq!(switched.report.current_player, 2);
        assert_eq!(switched.report.lives, 2);
        assert_eq!(switched.report.smart_bombs, 1);
        assert_eq!(
            switched.report.player_stocks,
            [
                PlayerStockSnapshot::new(0, 2),
                PlayerStockSnapshot::new(2, 1),
            ]
        );
        assert_eq!(
            switched.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS,
                player: 2,
            })
        );
        assert_source_message(&switched.report, "PLYR2", SOURCE_PLAYER_START_PROMPT_SCREEN);
        assert_source_message_scene(&switched.scene, "PLYR2", SOURCE_PLAYER_START_PROMPT_SCREEN);

        let active = step_until_player_start_completes(&mut runtime, 2);
        assert_eq!(active.report.current_player, 2);
        assert_eq!(active.report.lives, 2);
        assert_eq!(active.report.smart_bombs, 1);
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
    }

    #[test]
    fn actor_two_player_final_death_enters_game_over_when_no_other_stock() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 1;
        driver.player_count = 2;
        driver.lives = 1;
        driver.smart_bombs = 2;
        driver.player_two_lives = 0;
        driver.player_two_smart_bombs = 1;
        driver.spawn_player();
        driver.spawn_enemy_laser_from_spawn(Point::new(42, 120), Velocity::new(0, 0), None);
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let killed = runtime.step(GameInput::NONE);

        assert_eq!(killed.report.phase, Phase::GameOver);
        assert_eq!(killed.report.lives, 0);
        assert!(killed.report.player_switch.is_none());
        assert!(killed.report.sounds.contains(&SoundCue::GameOver));
        assert!(killed.events.gameplay().contains(&GameEvent::GameOver));
        assert_eq!(killed.state.game_over, GameOverSnapshot::NONE);
    }

    #[test]
    fn actor_two_player_second_player_final_death_switches_back_to_player_one() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 2;
        driver.player_count = 2;
        driver.lives = 2;
        driver.smart_bombs = 1;
        driver.player_two_lives = 1;
        driver.player_two_smart_bombs = 2;
        driver.spawn_player();
        driver.spawn_enemy_laser_from_spawn(Point::new(42, 120), Velocity::new(0, 0), None);
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let killed = runtime.step(GameInput::NONE);

        assert_eq!(killed.report.phase, Phase::GameOver);
        assert_eq!(killed.report.current_player, 2);
        assert_eq!(killed.report.lives, 0);
        assert_eq!(
            killed.report.player_stocks,
            [
                PlayerStockSnapshot::new(2, 1),
                PlayerStockSnapshot::new(0, 2),
            ]
        );
        assert_eq!(
            killed.report.player_switch,
            Some(PlayerSwitchReport {
                sleep_steps_remaining: SOURCE_PLAYER_SWITCH_SLEEP_STEPS,
                from_player: 2,
                to_player: 1,
            })
        );
        assert!(!killed.report.sounds.contains(&SoundCue::GameOver));

        let switched = step_until_player_switch_completes(&mut runtime, 1);

        assert_eq!(switched.report.phase, Phase::Playing);
        assert_eq!(switched.report.current_player, 1);
        assert_eq!(switched.report.lives, 2);
        assert_eq!(switched.report.smart_bombs, 1);
        assert_eq!(
            switched.report.player_stocks,
            [
                PlayerStockSnapshot::new(2, 1),
                PlayerStockSnapshot::new(0, 2),
            ]
        );
        assert_eq!(
            switched.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: SOURCE_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert_source_message(&switched.report, "PLYR1", SOURCE_PLAYER_START_PROMPT_SCREEN);
        assert_source_message_scene(&switched.scene, "PLYR1", SOURCE_PLAYER_START_PROMPT_SCREEN);

        let active = step_until_player_start_completes(&mut runtime, 1);
        assert_eq!(active.report.current_player, 1);
        assert_eq!(active.report.lives, 2);
        assert_eq!(active.report.smart_bombs, 1);
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
    }

    #[test]
    fn status_display_actor_draws_play_state_from_prompt() {
        let mut driver = started_driver();
        driver.score = 9_875;
        driver.wave = 7;
        driver.lives = 2;
        driver.smart_bombs = 1;
        driver.credits = 3;

        let report = driver.step(GameInput::NONE);

        assert!(
            report
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::StatusDisplay)
        );
        assert_text(&report, "1UP 009875");
        assert_text(&report, "HIGH 010000");
        assert_text(&report, "WAVE 07");
        assert_text(&report, "LIVES 02");
        assert_text(&report, "BOMBS 01");
        assert_text(&report, "CREDIT 03");
    }

    #[test]
    fn status_display_actor_draws_high_score_entry_state() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.score = 12_000;
        driver.spawn_player();
        driver.spawn_enemy_laser_from_spawn(Point::new(42, 120), Velocity::default(), None);

        let game_over = driver.step(GameInput::NONE);
        assert_eq!(game_over.phase, Phase::HighScoreEntry);

        let entry = driver.step(GameInput::NONE);
        assert_text(&entry, "FINAL SCORE 012000");
        assert_text(&entry, "HIGH SCORES");
        assert_text(&entry, "INITIALS ___");
        assert_text(&entry, "1. 012000");
        assert_text(&entry, "2. 010000");
        assert_text(&entry, "ENTER INITIALS");
    }

    #[test]
    fn high_score_entry_accepts_initials_and_backspace_from_actor_input() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.score = 12_000;
        driver.next_bonus = 20_000;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));

        assert_eq!(driver.step(GameInput::NONE).phase, Phase::HighScoreEntry);

        let first = driver.step(GameInput {
            high_score_initial: Some('a'),
            ..GameInput::NONE
        });
        assert_eq!(first.phase, Phase::HighScoreEntry);
        assert!(first.high_score_initial_accepted);
        assert!(!first.high_score_submitted);
        assert_text(&first, "INITIALS A__");

        let erased = driver.step(GameInput {
            high_score_backspace: true,
            ..GameInput::NONE
        });
        assert_eq!(erased.phase, Phase::HighScoreEntry);
        assert_text(&erased, "INITIALS ___");

        let second = driver.step(GameInput {
            high_score_initial: Some('B'),
            ..GameInput::NONE
        });
        assert!(second.high_score_initial_accepted);
        assert_text(&second, "INITIALS B__");
        let third = driver.step(GameInput {
            high_score_initial: Some('C'),
            ..GameInput::NONE
        });
        assert!(third.high_score_initial_accepted);
        assert_text(&third, "INITIALS BC_");
        let submitted = driver.step(GameInput {
            high_score_initial: Some('D'),
            ..GameInput::NONE
        });

        assert_eq!(submitted.phase, Phase::GameOver);
        assert!(submitted.high_score_initial_accepted);
        assert!(submitted.high_score_submitted);
        assert_eq!(
            submitted.game_over_hall_of_fame_stall_remaining,
            Some(SOURCE_HIGH_SCORE_HALL_STALL_STEPS)
        );
        assert_eq!(
            submitted.game_state().game_over,
            GameOverSnapshot {
                hall_of_fame_stall_remaining: Some(SOURCE_HIGH_SCORE_HALL_STALL_STEPS),
                ..GameOverSnapshot::NONE
            }
        );
        assert_text(&submitted, "HALL OF FAME");
        assert!(!submitted.draws.iter().any(|draw| {
            draw.text
                .as_deref()
                .is_some_and(|text| text.contains("INITIALS"))
        }));

        for expected_timer in (1..SOURCE_HIGH_SCORE_HALL_STALL_STEPS).rev() {
            let waiting = driver.step(GameInput::NONE);
            assert_eq!(waiting.phase, Phase::GameOver);
            assert_eq!(
                waiting.game_over_hall_of_fame_stall_remaining,
                Some(expected_timer)
            );
            assert_text(&waiting, "HALL OF FAME");
        }

        let returned = driver.step(GameInput::NONE);
        assert_eq!(returned.phase, Phase::Attract);
        assert_eq!(returned.game_over_hall_of_fame_stall_remaining, None);
        assert!(
            returned
                .draws
                .iter()
                .any(|draw| matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
        );
    }

    #[test]
    fn planetoid_mapper_matches_current_live_key_contract() {
        let mut mapper = KeyboardMapper::new(KeyboardProfile::Planetoid);
        let mut step = KeyboardPoll::default();

        for key in [
            KeyboardKey::Character('5'),
            KeyboardKey::Character('6'),
            KeyboardKey::Character('7'),
            KeyboardKey::Enter,
            KeyboardKey::Character('A'),
            KeyboardKey::Character('Z'),
            KeyboardKey::LeftShift,
            KeyboardKey::Character(' '),
            KeyboardKey::Tab,
            KeyboardKey::Character('H'),
            KeyboardKey::Function(2),
            KeyboardKey::Function(3),
            KeyboardKey::Function(4),
            KeyboardKey::Function(5),
        ] {
            mapper.map_event(KeyboardEvent::press(key), &mut step);
        }
        mapper.finish_poll(&mut step);

        assert!(step.input.coin);
        assert!(step.input.coin_two);
        assert!(step.input.coin_three);
        assert!(step.input.start_one);
        assert!(step.input.fire);
        assert!(step.input.altitude_up);
        assert!(step.input.altitude_down);
        assert!(step.input.thrust);
        assert!(step.input.reverse);
        assert!(step.input.smart_bomb);
        assert!(step.input.hyperspace);
        assert!(step.input.service_advance);
        assert!(step.input.high_score_reset);
        assert!(step.input.auto_up_manual_down);
        assert!(step.input.tilt);
    }

    #[test]
    fn cabinet_mapper_keeps_enter_out_of_the_start_binding() {
        let mut mapper = KeyboardMapper::new(KeyboardProfile::Cabinet);
        let mut enter_poll = KeyboardPoll::default();

        mapper.map_event(KeyboardEvent::press(KeyboardKey::Enter), &mut enter_poll);
        mapper.finish_poll(&mut enter_poll);
        assert!(!enter_poll.input.start_one);
        assert!(!enter_poll.input.fire);

        let mut step = KeyboardPoll::default();
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('1')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('2')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::ArrowUp), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::ArrowDown), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('R')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('T')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('F')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('B')), &mut step);
        mapper.finish_poll(&mut step);

        assert!(step.input.start_one);
        assert!(step.input.start_two);
        assert!(step.input.altitude_up);
        assert!(step.input.altitude_down);
        assert!(step.input.reverse);
        assert!(step.input.thrust);
        assert!(step.input.fire);
        assert!(step.input.smart_bomb);
    }

    #[test]
    fn xyzzy_overlay_toggles_auto_fire_invincibility_and_overlay_bomb() {
        let mut mapper = KeyboardMapper::default();
        let mut step = KeyboardPoll::default();

        for character in ['x', 'y', 'z', 'z', 'y', 'f', 'g'] {
            mapper.map_event(
                KeyboardEvent::press(KeyboardKey::Character(character)),
                &mut step,
            );
        }
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Tab), &mut step);
        mapper.finish_poll(&mut step);

        assert!(step.input.xyzzy.active);
        assert!(step.input.xyzzy.auto_fire);
        assert!(step.input.xyzzy.invincible);
        assert!(step.input.xyzzy.overlay_smart_bomb);
        assert!(step.input.fire);

        for character in ['x', 'y', 'z', 'z', 'y'] {
            mapper.map_event(
                KeyboardEvent::press(KeyboardKey::Character(character)),
                &mut KeyboardPoll::default(),
            );
        }
        assert_eq!(mapper.xyzzy_mode(), XyzzyMode::INACTIVE);
    }

    #[test]
    fn player_actor_emits_spawn_and_sound_when_prompted_to_fire() {
        let mut driver = started_driver();

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });

        assert!(fired.sounds.contains(&SoundCue::Laser));
        assert!(fired.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Laser {
                    direction: Direction::Right,
                    ..
                })
            )
        }));

        let next = driver.step(GameInput::NONE);
        assert!(
            next.snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::Laser)
        );
    }

    #[test]
    fn smart_bomb_clears_hostiles_with_explosions_and_score() {
        let mut driver = started_driver();

        let pressed = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(pressed.score, 0);
        assert_eq!(pressed.smart_bombs, INITIAL_SMART_BOMBS - 1);
        assert_eq!(
            pressed.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 10,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert!(pressed.survivor_bonus.is_none());
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert_eq!(driver.snapshot_count(ActorKind::Human), 10);
        assert!(pressed.sounds.is_empty());
        assert!(pressed.commands.contains(&GameCommand::SmartBomb {
            consume_stock: true,
        }));

        let held_during_delay = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });
        assert_eq!(held_during_delay.score, 0);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert!(
            !held_during_delay
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::SmartBomb { .. }))
        );

        let detonated = step_until_driver_smart_bomb_detonates(&mut driver);
        assert_eq!(detonated.score, LANDER_SCORE * 5);
        assert_eq!(
            detonated.smart_bomb_flash_steps_remaining,
            SOURCE_SMART_BOMB_FLASH_STEPS
        );
        assert_eq!(detonated.render_scene().clear_color, Color::WHITE);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Human), 10);
        assert_eq!(
            detonated
                .commands
                .iter()
                .filter(|command| matches!(command, GameCommand::Destroy(_)))
                .count(),
            5
        );

        let blocked_restore = driver.step(GameInput::NONE);
        let mut sounds = blocked_restore.sounds.clone();
        assert_eq!(blocked_restore.enemy_reserve, detonated.enemy_reserve);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);

        sounds.extend(collect_driver_smart_bomb_sound_sequence(&mut driver));
        assert_eq!(sounds, source_smart_bomb_sound_cues());

        let restored = step_until_driver_source_reserve_activates(&mut driver);
        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            SOURCE_MAX_ACTIVE_WAVE_ENEMIES
        );
    }

    #[test]
    fn smart_bomb_input_without_stock_does_not_clear_hostiles() {
        let mut driver = started_driver();
        driver.smart_bombs = 0;

        let report = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(report.score, 0);
        assert_eq!(report.smart_bombs, 0);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert!(report.sounds.is_empty());
        assert!(
            !report
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::SmartBomb { .. }))
        );
    }

    #[test]
    fn xyzzy_overlay_smart_bomb_does_not_consume_stock() {
        let mut driver = started_driver();
        driver.smart_bombs = 0;

        let pressed = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                overlay_smart_bomb: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });

        assert_eq!(pressed.score, 0);
        assert_eq!(pressed.smart_bombs, 0);
        assert_eq!(
            pressed.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 10,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert!(pressed.survivor_bonus.is_none());
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert!(pressed.sounds.is_empty());
        assert!(pressed.commands.contains(&GameCommand::SmartBomb {
            consume_stock: false,
        }));

        let detonated = step_until_driver_smart_bomb_detonates(&mut driver);
        assert_eq!(detonated.score, LANDER_SCORE * 5);
        assert_eq!(detonated.smart_bombs, 0);
        assert_eq!(
            detonated.smart_bomb_flash_steps_remaining,
            SOURCE_SMART_BOMB_FLASH_STEPS
        );
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);

        let blocked_restore = driver.step(GameInput::NONE);
        assert_eq!(blocked_restore.enemy_reserve, detonated.enemy_reserve);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);

        let restored = step_until_driver_source_reserve_activates(&mut driver);
        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
    }

    #[test]
    fn default_wave_script_uses_source_wave_table_values() {
        let script = ActorWaveScript::default_progression();
        assert_eq!(script.name(), "actor-source-wave-table");
        let first_source = ActorSourceWaveProfile::for_wave(1);
        assert_eq!(first_source.baiter_delay, 192);
        assert_eq!(first_source.baiter_shot_time, 10);
        assert_eq!(first_source.baiter_seek_probability, 200);
        assert_eq!(first_source.mutant_random_y, 1);
        assert_eq!(first_source.mutant_y_velocity_msb, 0);
        assert_eq!(first_source.mutant_y_velocity_lsb, 128);
        assert_eq!(first_source.mutant_x_velocity, 32);
        assert_eq!(first_source.mutant_shot_time, 32);

        let first = script.profile_for_wave(1);
        assert_eq!(first.source_wave, Some(first_source));
        let first_lander = first
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Lander);
        assert_eq!(first.lander_spawns.len(), 5);
        assert_eq!(
            first.lander_spawn_points(),
            vec![
                Point::new(0xFB, 0x2C),
                Point::new(0x3F, 0x2C),
                Point::new(0x67, 0x2C),
                Point::new(0x0D, 0x2C),
                Point::new(0x41, 0x2C),
            ]
        );
        assert_eq!(first.human_spawns.len(), 10);
        assert!(first.bomber_spawns.is_empty());
        assert!(first.pod_spawns.is_empty());
        assert_eq!(
            first.human_spawn_points(),
            vec![
                Point::new(0x18, 0xE0),
                Point::new(0x1C, 0xE1),
                Point::new(0x4E, 0xE0),
                Point::new(0x57, 0xE0),
                Point::new(0x9B, 0xE0),
                Point::new(0x9D, 0xE0),
                Point::new(0xCE, 0xE0),
                Point::new(0xD7, 0xE0),
                Point::new(0xD2, 0xE0),
                Point::new(0xE8, 0xE0),
            ]
        );
        assert_eq!(
            first.human_spawns[1].source,
            Some(ActorSourceHumanMetadata {
                x_fraction: 0x81,
                y_fraction: 0x00,
                picture_frame: 3,
                target_slot_index: 1,
            })
        );
        assert_eq!(
            first.lander_spawns[0].source,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0x33,
                y_fraction: 0xE0,
                x_velocity: 0xFFDE,
                y_velocity: 0x0070,
                shot_timer: 0x27,
                sleep_ticks: 0x04,
                picture_frame: 1,
                target_human_index: Some(1),
            })
        );
        assert_eq!(
            first.lander_spawns[3].source,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0x11,
                y_fraction: 0x70,
                x_velocity: 0x0014,
                y_velocity: 0x0070,
                shot_timer: 0x3C,
                sleep_ticks: 0x04,
                picture_frame: 0,
                target_human_index: Some(4),
            })
        );
        assert_eq!(first_lander.lander_seek_speed, 2);
        assert_eq!(first_lander.lander_fire_period_steps, 64);

        let second = script.profile_for_wave(2);
        assert_eq!(
            second.source_wave,
            Some(ActorSourceWaveProfile::for_wave(2))
        );
        let second_lander = second
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Lander);
        let second_bomber = second
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Bomber);
        assert_eq!(second.lander_spawns.len(), 3);
        assert_eq!(
            second.lander_spawn_points(),
            vec![
                Point::new(0xD2, 0x2C),
                Point::new(0x1A, 0x2C),
                Point::new(0xE3, 0x2C),
            ]
        );
        assert_eq!(
            second.lander_spawns[0].source,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0xAD,
                y_fraction: 0,
                x_velocity: 0x001E,
                y_velocity: 0x00B0,
                shot_timer: 0x21,
                sleep_ticks: 0,
                picture_frame: 0,
                target_human_index: Some(1),
            })
        );
        assert_eq!(
            second.lander_spawns[1].source,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0x55,
                y_fraction: 0,
                x_velocity: 0xFFDE,
                y_velocity: 0x00B0,
                shot_timer: 0x2F,
                sleep_ticks: 0,
                picture_frame: 0,
                target_human_index: Some(2),
            })
        );
        assert_eq!(
            second.lander_spawns[2].source,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0x4A,
                y_fraction: 0,
                x_velocity: 0x0020,
                y_velocity: 0x00B0,
                shot_timer: 0x1D,
                sleep_ticks: 0,
                picture_frame: 0,
                target_human_index: Some(3),
            })
        );
        assert_eq!(second.bomber_spawns.len(), 1);
        assert_eq!(second.bomber_spawn_points(), vec![Point::new(228, 104)]);
        assert_eq!(
            second.bomber_spawns[0].source,
            Some(ActorSourceBomberMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0xFFD8,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                source_slot: 1,
            })
        );
        assert_eq!(second.pod_spawns.len(), 1);
        assert_eq!(second.pod_spawn_points(), vec![Point::new(184, 72)]);
        assert_eq!(
            second.pod_spawns[0].source,
            Some(ActorSourcePodMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0020,
                y_velocity: 0,
            })
        );
        assert_eq!(
            second.human_spawn_points(),
            vec![
                Point::new(0x12, 0xE0),
                Point::new(0x09, 0xE0),
                Point::new(0x54, 0xE0),
                Point::new(0x5A, 0xE0),
                Point::new(0x8D, 0xE0),
                Point::new(0x86, 0xE0),
                Point::new(0xC3, 0xE0),
                Point::new(0xD1, 0xE0),
                Point::new(0x09, 0xE0),
                Point::new(0x14, 0xE0),
            ]
        );
        assert_eq!(
            second.human_spawns[0].source,
            Some(ActorSourceHumanMetadata {
                x_fraction: 0xAD,
                y_fraction: 0,
                picture_frame: 2,
                target_slot_index: 0,
            })
        );
        assert_eq!(
            second.human_spawns[9].source,
            Some(ActorSourceHumanMetadata {
                x_fraction: 0x69,
                y_fraction: 0,
                picture_frame: 2,
                target_slot_index: 9,
            })
        );
        assert_eq!(
            second.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 17,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(second_lander.lander_seek_speed, 2);
        assert_eq!(second_lander.lander_fire_period_steps, 48);
        assert_eq!(second_bomber.bomber_drift_speed, 1);

        let fifth = script.profile_for_wave(5);
        assert_eq!(fifth.source_wave, Some(ActorSourceWaveProfile::for_wave(5)));
        let fifth_lander = fifth
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Lander);
        assert_eq!(fifth_lander.lander_seek_speed, 3);
        assert_eq!(fifth_lander.lander_fire_period_steps, 30);
    }

    #[test]
    fn first_wave_early_reserve_lander_spawns_match_mame_rows() {
        let rows = ACTOR_SOURCE_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS
            .iter()
            .copied()
            .map(source_lander_spawn_row_for_test)
            .collect::<Vec<_>>();

        assert_eq!(
            rows,
            vec![
                (0x689A, 0x2C70, 0x001E, 0x0070, 0x10, 0, 1, Some(7)),
                (0x43D3, 0x2C70, 0xFFEC, 0x0070, 0x3A, 0, 1, Some(9)),
                (0x1F51, 0x2C70, 0x0014, 0x0070, 0x13, 0, 0, Some(8)),
                (0xFA03, 0x2C70, 0x0016, 0x0070, 0x26, 0, 1, Some(7)),
                (0xCF34, 0x2CE0, 0, 0, 0x34, 1, 0, Some(6)),
            ]
        );
        assert_eq!(SOURCE_FIRST_WAVE_EARLY_RESERVE_TARGET2_SHOT_PHASE_DELAY, 2);
    }

    #[test]
    fn first_wave_refill_lander_spawns_match_mame_rows() {
        let rows = ACTOR_SOURCE_FIRST_WAVE_REFILL_LANDER_SPAWNS
            .iter()
            .copied()
            .map(source_lander_spawn_row_for_test)
            .collect::<Vec<_>>();

        assert_eq!(
            rows,
            vec![
                (0xBC29, 0x2CFD, 0x001C, 0x0090, 0x36, 6, 1, Some(7)),
                (0xE14C, 0x2CAE, 0x000E, 0x0090, 0x2F, 0, 0, Some(4)),
                (0x0A63, 0x2CF0, 0xFFF4, 0x0090, 0x23, 1, 0, Some(3)),
                (0x531B, 0x2CC0, 0xFFF6, 0x0090, 0x30, 1, 0, Some(2)),
                (0x98D9, 0x2CB8, 0x001A, 0x0090, 0x1F, 1, 0, Some(1)),
            ]
        );
        assert_eq!(SOURCE_FIRST_WAVE_LANDER_REFILL_DELAY_STEPS, 47);
        assert_eq!(SOURCE_FIRST_WAVE_LANDER_REFILL_APPEAR_SOUND_DELAY_STEPS, 1);
    }

    #[test]
    fn embedded_actor_wave_script_expands_source_wave_range() {
        let parsed = ActorWaveScript::parse_text(ACTOR_RED_LABEL_WAVE_SCRIPT)
            .expect("embedded actor wave script should parse");

        assert_eq!(
            ActorWaveScript::default_progression().manifest(),
            parsed.manifest()
        );
        assert_eq!(
            ActorWaveScript::source_table_progression().manifest(),
            parsed.manifest()
        );
        assert_eq!(parsed.name(), "actor-source-wave-table");
        assert_eq!(
            parsed.manifest().waves.len(),
            usize::from(ACTOR_SOURCE_BACKED_WAVES)
        );
        assert_eq!(
            parsed.manifest().waves[0].source_wave,
            Some(ActorSourceWaveProfile::for_wave(1))
        );
        assert_eq!(
            parsed.manifest().waves[4].source_wave,
            Some(ActorSourceWaveProfile::for_wave(5))
        );
        assert_eq!(parsed.profile_for_wave(1).lander_spawns.len(), 5);
        assert_eq!(
            parsed.profile_for_wave(1).enemy_reserve,
            EnemyReserveSnapshot {
                landers: 10,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(parsed.profile_for_wave(2).bomber_spawns.len(), 1);
        assert_eq!(parsed.profile_for_wave(2).pod_spawns.len(), 1);
    }

    #[test]
    fn second_source_wave_spawns_bomber_and_pod_actor_families() {
        let (driver, live) = started_source_wave_driver(2);
        assert_eq!(live.wave, 2);

        assert_eq!(driver.snapshot_count(ActorKind::Lander), 3);
        assert_eq!(driver.snapshot_count(ActorKind::Bomber), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 1);
        assert_eq!(
            live.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 17,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(live.game_state().world.enemy_reserve, live.enemy_reserve);
        let bomber_snapshot = live
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Bomber)
            .expect("second wave should publish source bomber snapshot");
        let (expected_bomber_position, expected_bomber_source) =
            expected_source_bomber_after_motion(
                Point::new(228, 104),
                ActorSourceBomberMetadata {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0xFFD8,
                    y_velocity: 0,
                    picture_frame: 0,
                    cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                    sleep_ticks: 0,
                    source_slot: 1,
                },
                live.step,
                bomber_snapshot.id,
                live.source_rng,
                None,
            );
        assert!(live.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Lander
                && snapshot.position == Point::new(0xD2, 0x2C)
                && snapshot.source_lander
                    == Some(ActorSourceLanderMetadata {
                        x_fraction: 0xCB,
                        y_fraction: 0xB0,
                        x_velocity: 0x001E,
                        y_velocity: 0x00B0,
                        shot_timer: 0x20,
                        sleep_ticks: 0,
                        picture_frame: 0,
                        target_human_index: Some(1),
                    })
        }));
        assert_eq!(bomber_snapshot.position, expected_bomber_position);
        assert_eq!(bomber_snapshot.source_bomber, Some(expected_bomber_source));
        assert!(live.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Pod
                && snapshot.source_pod
                    == Some(ActorSourcePodMetadata {
                        x_fraction: 0x20,
                        y_fraction: 0,
                        x_velocity: 0x0020,
                        y_velocity: 0,
                    })
        }));
        assert!(
            live.draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::Bomber
                    && matches!(
                        draw.effect,
                        VisualEffect::SourceBomberFrame { frame }
                            if frame == expected_bomber_source.picture_frame
                    ))
        );
        assert!(
            live.draws.iter().any(|draw| draw.sprite == SpriteKey::Pod
                && matches!(draw.effect, VisualEffect::SourcePod))
        );
    }

    #[test]
    fn actor_source_reserve_landers_activate_before_wave_clear() {
        let (mut driver, live) = started_source_wave_driver(2);
        assert_eq!(
            live.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 17,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );

        destroy_source_counted_hostiles(&mut driver, &live);
        let restored = driver.step(GameInput::NONE);

        assert_eq!(restored.phase, Phase::Playing);
        assert!(
            !restored
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 3 })
        );
        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 12,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(
            restored.game_state().world.enemy_reserve,
            restored.enemy_reserve
        );
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            SOURCE_MAX_ACTIVE_WAVE_ENEMIES
        );
        let source_landers = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .collect::<Vec<_>>();
        assert_eq!(source_landers.len(), SOURCE_MAX_ACTIVE_WAVE_ENEMIES);
        assert!(
            source_landers
                .iter()
                .all(|snapshot| snapshot.source_lander.is_some())
        );
        assert!(source_landers.iter().any(|snapshot| {
            snapshot
                .source_lander
                .is_some_and(|source| source.target_human_index == Some(4))
        }));
    }

    #[test]
    fn actor_first_wave_early_lander_reserve_materializes_on_source_cadence() {
        let mut driver = started_driver();
        driver.set_kind_behavior(
            ActorKind::Player,
            ActorBehaviorProfile {
                player_takes_enemy_collision_damage: false,
                ..ActorBehaviorProfile::default()
            },
        );
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert_eq!(
            driver.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 10,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(
            driver.source_first_wave_early_reserve_steps_remaining,
            Some(SOURCE_FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS)
        );

        let mut materialized = None;
        for offset in 1..=SOURCE_FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            let spawn_count = report
                .commands
                .iter()
                .filter(|command| {
                    matches!(command, GameCommand::Spawn(SpawnRequest::Lander { .. }))
                })
                .count();
            if spawn_count > 0 || report.sounds.contains(&SoundCue::HyperspaceMaterialize) {
                materialized = Some((offset, report));
                break;
            }

            assert_eq!(spawn_count, 0);
            assert!(!report.sounds.contains(&SoundCue::HyperspaceMaterialize));
        }

        let (offset, report) = materialized.unwrap_or_else(|| {
            panic!(
                "first-wave early lander reserve should materialize on source cadence; \
                 ready={} cooldown={} early={:?} reserve={:?} hostiles={} phase={:?}",
                driver.source_reserve_activation_ready,
                driver.source_reserve_activation_cooldown_steps,
                driver.source_first_wave_early_reserve_steps_remaining,
                driver.enemy_reserve,
                driver.source_counted_hostile_snapshot_count(),
                driver.phase
            )
        });
        assert_eq!(offset, SOURCE_FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS);
        assert!(report.sounds.contains(&SoundCue::HyperspaceMaterialize));
        assert!(
            !report
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );
        assert_eq!(
            report
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            ACTOR_SOURCE_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS.len()
        );
        assert_eq!(
            report.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(
            report.game_state().world.enemy_reserve,
            report.enemy_reserve
        );
        assert_eq!(
            report
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Lander)
                .count(),
            10
        );
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Lander
                && snapshot.source_lander.is_some_and(|source| {
                    source.target_human_index
                        == Some(SOURCE_FIRST_WAVE_EARLY_RESERVE_TARGET_CURSOR_SLOT)
                        && source.x_velocity == 0
                        && source.y_velocity == 0
                })
        }));
    }

    #[test]
    fn actor_first_wave_lander_refill_keeps_hidden_lanes_suppressed() {
        let mut driver = started_driver();
        driver.set_kind_behavior(
            ActorKind::Player,
            ActorBehaviorProfile {
                player_takes_enemy_collision_damage: false,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Lander,
            ActorBehaviorProfile {
                lander_mode: LanderBehaviorMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        let early_reserve = step_until_first_wave_early_reserve_materializes(&mut driver);
        assert_eq!(
            early_reserve.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(
            early_reserve
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Lander)
                .count(),
            10
        );

        let destroy_three_landers = early_reserve
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .take(3)
            .map(|snapshot| GameCommand::Destroy(snapshot.id))
            .collect::<Vec<_>>();
        driver.apply_commands(&destroy_three_landers);

        let scheduled = driver.step(GameInput::NONE);
        assert_eq!(
            scheduled
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Lander)
                .count(),
            7
        );
        assert_eq!(
            driver.source_first_wave_lander_refill_steps_remaining,
            Some(SOURCE_FIRST_WAVE_LANDER_REFILL_DELAY_STEPS)
        );

        let mut materialized = None;
        for offset in 1..=SOURCE_FIRST_WAVE_LANDER_REFILL_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            let spawn_count = report
                .commands
                .iter()
                .filter(|command| {
                    matches!(command, GameCommand::Spawn(SpawnRequest::Lander { .. }))
                })
                .count();
            if spawn_count > 0 {
                materialized = Some((offset, report));
                break;
            }

            assert_eq!(spawn_count, 0);
        }

        let (offset, report) =
            materialized.expect("first-wave refill should materialize on source cadence");
        assert_eq!(offset, SOURCE_FIRST_WAVE_LANDER_REFILL_DELAY_STEPS);
        assert_eq!(
            report
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            ACTOR_SOURCE_FIRST_WAVE_REFILL_LANDER_SPAWNS.len()
        );
        assert_eq!(report.enemy_reserve, EnemyReserveSnapshot::default());

        let refill_landers = report
            .snapshots
            .iter()
            .filter(|snapshot| {
                snapshot.kind == ActorKind::Lander
                    && snapshot
                        .source_lander
                        .is_some_and(|source| source.y_velocity == 0x0090)
            })
            .collect::<Vec<_>>();
        assert_eq!(refill_landers.len(), 5);
        assert_eq!(
            refill_landers
                .iter()
                .filter(|snapshot| snapshot.bounds.is_some())
                .count(),
            1
        );
        let visible_refill = refill_landers
            .iter()
            .find(|snapshot| snapshot.bounds.is_some())
            .expect("target-3 refill lane should be visible");
        assert_eq!(
            visible_refill.source_lander,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0x63,
                y_fraction: 0xF0,
                x_velocity: 0xFFF4,
                y_velocity: 0x0090,
                shot_timer: 0x23,
                sleep_ticks: 0,
                picture_frame: 0,
                target_human_index: Some(3),
            })
        );

        let refill_ids = refill_landers
            .iter()
            .map(|snapshot| snapshot.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            report
                .draws
                .iter()
                .filter(|draw| refill_ids.contains(&draw.actor) && draw.sprite == SpriteKey::Lander)
                .count(),
            1
        );

        let delayed_sound = driver.step(GameInput::NONE);
        assert!(
            delayed_sound
                .sounds
                .contains(&SoundCue::SourceCommand(SOURCE_APPEAR_SOUND_COMMAND))
        );
    }

    #[test]
    fn actor_source_reserve_landers_without_humans_restore_source_mutants() {
        let (mut driver, live) = started_source_wave_driver(2);
        let player_position = live
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player)
            .expect("seed step should publish the player")
            .position;
        let clear_playfield = live
            .snapshots
            .iter()
            .filter(|snapshot| is_hostile(snapshot.kind) || snapshot.kind == ActorKind::Human)
            .map(|snapshot| GameCommand::Destroy(snapshot.id))
            .collect::<Vec<_>>();
        driver.apply_commands(&clear_playfield);
        driver.enemy_reserve = EnemyReserveSnapshot {
            landers: 2,
            ..EnemyReserveSnapshot::default()
        };
        driver.source_rng = ActorSourceRng {
            seed: 0x20,
            hseed: 0x66,
            lseed: 0x99,
        };
        driver.source_background_left = 0x3400;
        let mut expected_rng = driver.source_rng;
        let first_spawn = ActorMutantSpawn::source_restore(
            &mut expected_rng,
            ActorSourceWaveProfile::for_wave(2),
            driver.source_background_left,
        );
        let second_spawn = ActorMutantSpawn::source_restore(
            &mut expected_rng,
            ActorSourceWaveProfile::for_wave(2),
            driver.source_background_left,
        );

        let restored = driver.step(GameInput::NONE);

        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 0,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(restored.source_background_left, 0x3400);
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            0
        );
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Mutant { .. })
                ))
                .count(),
            2
        );
        assert_eq!(
            restored
                .source_rng
                .expect("playing report should carry source rng"),
            expected_rng.advance().snapshot()
        );

        let prompt = source_mutant_prompt_for_test(
            restored.step,
            restored.wave,
            restored
                .source_rng
                .expect("playing report should carry source rng"),
            player_position,
            Velocity::default(),
        );
        let behavior = ActorBehaviorProfile::default();
        let source_mutants = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Mutant)
            .collect::<Vec<_>>();
        assert_eq!(source_mutants.len(), 2);
        for (snapshot, spawn) in source_mutants.iter().zip([first_spawn, second_spawn]) {
            let (expected_position, expected_source, _) = expected_source_mutant_after_motion(
                spawn.position,
                spawn.source.expect("source mutant restore metadata"),
                snapshot.id,
                &prompt,
                behavior,
            );
            assert_eq!(snapshot.position, expected_position);
            assert_eq!(snapshot.source_mutant, Some(expected_source));
            assert!(snapshot.source_lander.is_none());
        }

        let followup = driver.step(GameInput::NONE);
        assert_eq!(
            followup
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Lander)
                .count(),
            0
        );
        assert_eq!(
            followup
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Mutant)
                .count(),
            2
        );
        assert!(
            followup
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Mutant)
                .all(|snapshot| snapshot.source_mutant.is_some())
        );
    }

    #[test]
    fn actor_source_mutant_reserves_use_restore_state() {
        let (mut driver, seeded) = started_source_wave_driver(2);
        let player_position = seeded
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player)
            .expect("seed step should publish the player")
            .position;
        destroy_source_counted_hostiles(&mut driver, &seeded);
        driver.enemy_reserve = EnemyReserveSnapshot {
            mutants: 2,
            ..EnemyReserveSnapshot::default()
        };
        driver.source_rng = ActorSourceRng {
            seed: 0x37,
            hseed: 0x5A,
            lseed: 0x91,
        };
        driver.source_background_left = 0x5420;
        let profile = ActorSourceWaveProfile::for_wave(2);
        let mut expected_rng = driver.source_rng;
        let first_spawn = ActorMutantSpawn::source_restore(
            &mut expected_rng,
            profile,
            driver.source_background_left,
        );
        let second_spawn = ActorMutantSpawn::source_restore(
            &mut expected_rng,
            profile,
            driver.source_background_left,
        );

        let restored = driver.step(GameInput::NONE);

        assert_eq!(restored.enemy_reserve, EnemyReserveSnapshot::default());
        assert_eq!(restored.source_background_left, 0x5420);
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Mutant { .. })
                ))
                .count(),
            2
        );
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            0
        );
        assert_eq!(
            restored
                .source_rng
                .expect("playing report should carry source rng"),
            expected_rng.advance().snapshot()
        );

        let prompt = source_mutant_prompt_for_test(
            restored.step,
            restored.wave,
            restored
                .source_rng
                .expect("playing report should carry source rng"),
            player_position,
            Velocity::default(),
        );
        let behavior = ActorBehaviorProfile::default();
        let mut source_mutants = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Mutant)
            .collect::<Vec<_>>();
        source_mutants.sort_by_key(|snapshot| snapshot.id);
        assert_eq!(source_mutants.len(), 2);
        for (snapshot, spawn) in source_mutants.iter().zip([first_spawn, second_spawn]) {
            let (expected_position, expected_source, _) = expected_source_mutant_after_motion(
                spawn.position,
                spawn.source.expect("source mutant restore metadata"),
                snapshot.id,
                &prompt,
                behavior,
            );
            assert_eq!(snapshot.position, expected_position);
            assert_eq!(snapshot.source_mutant, Some(expected_source));
            assert!(snapshot.source_lander.is_none());
        }
    }

    #[test]
    fn actor_source_bomber_and_pod_reserves_use_restore_state() {
        let (mut driver, seeded) = started_source_wave_driver(2);
        let player_position = seeded
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player)
            .expect("seed step should publish the player")
            .position;
        destroy_source_counted_hostiles(&mut driver, &seeded);
        driver.enemy_reserve = EnemyReserveSnapshot {
            bombers: 5,
            pods: 2,
            ..EnemyReserveSnapshot::default()
        };
        driver.source_rng = ActorSourceRng {
            seed: 0x12,
            hseed: 0x6D,
            lseed: 0x80,
        };
        let mut expected_rng = driver.source_rng;
        let mut expected_pod = ActorPodSpawn::source_restore(&mut expected_rng);
        if let Some(source) = &mut expected_pod.source {
            let (x, x_fraction) = actor_source_axis_step(
                expected_pod.position.x,
                source.x_fraction,
                source.x_velocity,
            );
            let (y, y_fraction) = actor_source_active_object_y_step(
                expected_pod.position.y,
                source.y_fraction,
                source.y_velocity,
            );
            expected_pod.position = Point::new(x, y);
            source.x_fraction = x_fraction;
            source.y_fraction = y_fraction;
        }

        let restored = driver.step(GameInput::NONE);

        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                bombers: 1,
                pods: 1,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Bomber { .. })
                ))
                .count(),
            4
        );
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(command, GameCommand::Spawn(SpawnRequest::Pod { .. })))
                .count(),
            1
        );
        let bombers = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomber)
            .collect::<Vec<_>>();
        assert_eq!(bombers.len(), 4);
        assert!(
            bombers
                .iter()
                .all(|snapshot| snapshot.source_bomber.is_some())
        );
        assert!(bombers.iter().any(|snapshot| {
            let source = snapshot.source_bomber.expect("source bomber");
            source.x_velocity
                == actor_sign_extend_u8_to_u16(
                    ActorSourceWaveProfile::for_wave(2).bomber_x_velocity,
                )
                && source.source_slot == 0
        }));
        assert!(restored.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Pod
                && snapshot.position == expected_pod.position
                && snapshot.source_pod == expected_pod.source
        }));
        assert!(
            restored
                .snapshots
                .iter()
                .filter_map(|snapshot| snapshot.source_bomber)
                .any(|source| {
                    let expected_spawn = ActorBomberSpawn::source_restore_batch(
                        ActorSourceWaveProfile::for_wave(2),
                        actor_source_absolute_x(player_position, 0),
                        1,
                    )[0];
                    let expected_source = expected_spawn.source.expect("expected source bomber");
                    let (_, x_fraction) = actor_source_axis_step(
                        expected_spawn.position.x,
                        expected_source.x_fraction,
                        expected_source.x_velocity,
                    );
                    source.x_fraction == x_fraction
                })
        );
    }

    #[test]
    fn actor_source_swarmer_reserves_use_plres_restore_state() {
        let (mut driver, seeded) = started_source_wave_driver(2);
        destroy_source_counted_hostiles(&mut driver, &seeded);
        driver.enemy_reserve = EnemyReserveSnapshot {
            swarmers: 4,
            ..EnemyReserveSnapshot::default()
        };
        driver.source_rng = ActorSourceRng {
            seed: 0x20,
            hseed: 0x41,
            lseed: 0xC0,
        };
        let profile = ActorSourceWaveProfile::for_wave(2);
        let mut expected_rng = driver.source_rng;
        let expected_spawns =
            ActorSwarmerSpawn::source_restore_batch(&mut expected_rng, profile, 4);

        let restored = driver.step(GameInput::NONE);

        assert_eq!(restored.enemy_reserve, EnemyReserveSnapshot::default());
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Swarmer { .. })
                ))
                .count(),
            4
        );
        assert_eq!(
            restored
                .source_rng
                .expect("playing report should carry source rng"),
            expected_rng.advance().snapshot()
        );

        let mut source_swarmers = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Swarmer)
            .collect::<Vec<_>>();
        source_swarmers.sort_by_key(|snapshot| snapshot.id);
        assert_eq!(source_swarmers.len(), 4);
        for (snapshot, spawn) in source_swarmers.iter().zip(expected_spawns) {
            let mut expected_source = spawn.source.expect("source swarmer restore metadata");
            expected_source.sleep_ticks = expected_source.sleep_ticks.saturating_sub(1);
            assert_eq!(snapshot.position, spawn.position);
            assert_eq!(snapshot.source_swarmer, Some(expected_source));
        }
        assert!(source_swarmers.iter().all(|snapshot| {
            let source = snapshot.source_swarmer.expect("source swarmer");
            snapshot.position == source_swarmers[0].position
                && source.x_fraction == SOURCE_MINI_SWARMER_RESTORE_X_LOW
                && source.y_fraction == 0
        }));
    }

    #[test]
    fn source_pod_y_motion_wraps_through_source_playfield_bounds() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let top = driver.spawn_pod_from_spawn(ActorPodSpawn {
            position: Point::new(0xD0, i16::from(SOURCE_PLAYFIELD_Y_MIN)),
            source: Some(ActorSourcePodMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0xFFFF,
            }),
        });
        let bottom = driver.spawn_pod_from_spawn(ActorPodSpawn {
            position: Point::new(0xE0, i16::from(SOURCE_PLAYFIELD_Y_MAX)),
            source: Some(ActorSourcePodMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0x0100,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert_eq!(
            snapshot_for(&report, top).position,
            Point::new(0xD0, i16::from(SOURCE_PLAYFIELD_Y_MAX))
        );
        assert_eq!(
            snapshot_for(&report, top).source_pod,
            Some(ActorSourcePodMetadata {
                x_fraction: 0,
                y_fraction: 0xFF,
                x_velocity: 0,
                y_velocity: 0xFFFF,
            })
        );
        assert_eq!(
            snapshot_for(&report, bottom).position,
            Point::new(0xE0, i16::from(SOURCE_PLAYFIELD_Y_MIN))
        );
        assert_eq!(
            snapshot_for(&report, bottom).source_pod,
            Some(ActorSourcePodMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0x0100,
            })
        );
    }

    #[test]
    fn source_hostile_y_motion_wraps_through_source_playfield_bounds() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.snapshots.insert(
            player,
            actor_snapshot(player.value(), ActorKind::Player, Point::new(42, 120)),
        );
        let lander = driver.spawn_lander_from_spawn(ActorLanderSpawn {
            position: Point::new(0x70, i16::from(SOURCE_PLAYFIELD_Y_MIN)),
            source: Some(ActorSourceLanderMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0xFFFF,
                shot_timer: 8,
                sleep_ticks: 0,
                picture_frame: 0,
                target_human_index: None,
            }),
        });
        let swarmer = driver.spawn_swarmer_from_spawn(ActorSwarmerSpawn {
            position: Point::new(0x80, i16::from(SOURCE_PLAYFIELD_Y_MAX)),
            source: Some(ActorSourceSwarmerMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0x0100,
                acceleration: 0,
                sleep_ticks: 0,
                shot_timer: 3,
                horizontal_seek_pending: true,
            }),
        });
        let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
            position: Point::new(0x90, i16::from(SOURCE_PLAYFIELD_Y_MIN)),
            source: Some(ActorSourceBaiterMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0xFFFF,
                shot_timer: 3,
                sleep_ticks: 1,
                picture_frame: 0,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert_eq!(
            snapshot_for(&report, lander).position,
            Point::new(0x70, i16::from(SOURCE_PLAYFIELD_Y_MAX))
        );
        assert_eq!(
            snapshot_for(&report, lander).source_lander,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0,
                y_fraction: 0xFF,
                x_velocity: 0,
                y_velocity: 0xFFFF,
                shot_timer: 7,
                sleep_ticks: 0,
                picture_frame: 0,
                target_human_index: None,
            })
        );
        assert_eq!(
            snapshot_for(&report, swarmer).position,
            Point::new(0x7F, i16::from(SOURCE_PLAYFIELD_Y_MIN))
        );
        assert_eq!(
            snapshot_for(&report, swarmer).source_swarmer,
            Some(ActorSourceSwarmerMetadata {
                x_fraction: 0xE0,
                y_fraction: 0,
                x_velocity: 0xFFE0,
                y_velocity: 0x0100,
                acceleration: 0,
                sleep_ticks: SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS,
                shot_timer: 3,
                horizontal_seek_pending: false,
            })
        );
        assert_eq!(
            snapshot_for(&report, baiter).position,
            Point::new(0x90, i16::from(SOURCE_PLAYFIELD_Y_MAX))
        );
        assert_eq!(
            snapshot_for(&report, baiter).source_baiter,
            Some(ActorSourceBaiterMetadata {
                x_fraction: 0,
                y_fraction: 0xFF,
                x_velocity: 0,
                y_velocity: 0xFFFF,
                shot_timer: 3,
                sleep_ticks: 0,
                picture_frame: 0,
            })
        );
    }

    #[test]
    fn first_wave_landers_publish_source_metadata_and_picture_frames() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let live = step_until_driver_player_start_completes(&mut driver, 1);
        let lander = live
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.kind == ActorKind::Lander && snapshot.position == Point::new(0xFB, 0x2C)
            })
            .expect("source first-wave lander should publish its restore position");

        assert_eq!(
            lander.source_lander,
            Some(ActorSourceLanderMetadata {
                x_fraction: 0x33,
                y_fraction: 0xE0,
                x_velocity: 0xFFDE,
                y_velocity: 0x0070,
                shot_timer: 0x27,
                sleep_ticks: 0x03,
                picture_frame: 1,
                target_human_index: Some(1),
            })
        );
        assert!(live.draws.iter().any(|draw| {
            draw.actor == lander.id
                && draw.sprite == SpriteKey::Lander
                && matches!(draw.effect, VisualEffect::SourceLanderFrame { frame: 1 })
        }));
    }

    #[test]
    fn source_lander_sleep_ticks_delay_first_wave_motion() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let initial = Point::new(0xFB, 0x2C);
        let mut lander_id = None;
        let mut current_report = Some(step_until_driver_player_start_completes(&mut driver, 1));
        for expected_sleep in [3, 2, 1, 0] {
            let sleeping = current_report
                .take()
                .unwrap_or_else(|| driver.step(GameInput::NONE));
            let lander = sleeping
                .snapshots
                .iter()
                .find(|snapshot| {
                    let matches_known_lander = match lander_id {
                        Some(id) => snapshot.id == id,
                        None => snapshot.position == initial,
                    };
                    snapshot.kind == ActorKind::Lander && matches_known_lander
                })
                .expect("sleeping source lander should stay visible");
            lander_id = Some(lander.id);
            assert_eq!(lander.position, initial);
            assert_eq!(
                lander.source_lander.map(|source| source.sleep_ticks),
                Some(expected_sleep)
            );
        }

        let awake = driver.step(GameInput::NONE);
        let lander = snapshot_for(&awake, lander_id.expect("source lander id should be known"));
        assert_eq!(lander.position, Point::new(0xFB, 0x2D));
        assert_eq!(
            lander
                .source_lander
                .map(|source| (source.x_fraction, source.y_fraction)),
            Some((0x11, 0x50))
        );
        assert_eq!(
            lander.source_lander.map(|source| source.sleep_ticks),
            Some(0)
        );
    }

    #[test]
    fn source_lander_shot_timer_controls_first_wave_shot_sound() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let mut first_laser_step = None;
        let live = step_until_driver_player_start_completes(&mut driver, 1);
        if live.sounds.contains(&SoundCue::LanderShot) {
            first_laser_step = Some(1);
        }
        for live_step in 2..=50 {
            let report = driver.step(GameInput {
                xyzzy: XyzzyMode {
                    active: true,
                    invincible: true,
                    ..XyzzyMode::INACTIVE
                },
                ..GameInput::NONE
            });
            if report.sounds.contains(&SoundCue::LanderShot) {
                first_laser_step = Some(live_step);
                break;
            }
        }

        assert_eq!(first_laser_step, Some(39));
    }

    #[test]
    fn source_lander_shot_timer_spawns_hostile_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let mut shot_report = None;
        let live = step_until_driver_player_start_completes(&mut driver, 1);
        if live
            .commands
            .iter()
            .any(|command| matches!(command, GameCommand::Spawn(SpawnRequest::EnemyLaser { .. })))
        {
            shot_report = Some(live);
        }
        for _ in 2..=50 {
            let report = driver.step(GameInput {
                xyzzy: XyzzyMode {
                    active: true,
                    invincible: true,
                    ..XyzzyMode::INACTIVE
                },
                ..GameInput::NONE
            });
            if report.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::EnemyLaser { .. }))
            }) {
                shot_report = Some(report);
                break;
            }
        }
        let shot_report = shot_report.expect("source lander should spawn a hostile shot");

        assert!(shot_report.sounds.contains(&SoundCue::LanderShot));
        let (shot_position, shot_velocity, shot_source) = shot_report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position,
                    velocity,
                    source,
                }) => Some((*position, *velocity, *source)),
                _ => None,
            })
            .expect("source lander should emit a hostile shot command");
        let lander_source = shot_report
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.kind == ActorKind::Lander && snapshot.position == shot_position
            })
            .and_then(|snapshot| snapshot.source_lander)
            .expect("source lander snapshot should own shot fractions");
        assert_eq!(
            shot_source,
            Some(ActorSourceEnemyProjectileMetadata {
                x_fraction: lander_source.x_fraction,
                y_fraction: lander_source.y_fraction,
                x_velocity: actor_source_projectile_velocity_component(shot_velocity.dx),
                y_velocity: actor_source_projectile_velocity_component(shot_velocity.dy),
                lifetime_ticks: actor_source_projectile_lifetime_ticks(LANDER_SHOT_LIFETIME),
            })
        );
        let settled = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                invincible: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });
        let enemy_laser = settled
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
            .expect("spawned hostile shot should publish an actor snapshot");
        let source_projectile = enemy_laser
            .source_enemy_projectile
            .expect("hostile shot should publish source projectile metadata");
        assert_eq!(
            source_projectile.x_velocity,
            actor_source_projectile_velocity_component(enemy_laser.velocity.dx)
        );
        assert_eq!(
            source_projectile.y_velocity,
            actor_source_projectile_velocity_component(enemy_laser.velocity.dy)
        );
        assert!(source_projectile.lifetime_ticks > 0);
        assert!(
            settled
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::EnemyLaser)
        );
    }

    #[test]
    fn enemy_laser_actor_advances_source_fixed_point_motion_state() {
        let behavior = ActorBehaviorProfile {
            lander_shot_lifetime_steps: 4,
            ..ActorBehaviorProfile::default()
        };
        let prompt = StepPrompt {
            step: 1,
            phase: Phase::Playing,
            input: GameInput::NONE,
            wave: 1,
            source_wave: ActorSourceWaveProfile::for_wave(1),
            current_player: 1,
            player_count: 1,
            score: 0,
            player_scores: [0, 0],
            credits: 0,
            lives: 3,
            smart_bombs: 3,
            smart_bomb_pending: false,
            player_stocks: [PlayerStockSnapshot::new(3, 3); 2],
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [0; 5],
            high_score_initials: HighScoreInitialsState::EMPTY,
            snapshots: Vec::new(),
            behavior_script: ActorBehaviorScript::default()
                .with_kind_behavior(ActorKind::EnemyLaser, behavior),
            source_background_left: 0,
            source_rng: None,
            source_human_walk_target_slot: None,
            source_shell_scan_tick: false,
        };
        let mut shot = EnemyLaserShot::new(
            ActorId::new(101),
            Point::new(10, 80),
            Velocity::new(1, -1),
            None,
        );
        shot.source.x_velocity = 0x0180;
        shot.source.y_velocity = 0xFF80;

        let first = shot.update(&prompt);

        assert_eq!(first.snapshot.position, Point::new(11, 79));
        assert_eq!(first.snapshot.velocity, Velocity::new(1, -1));
        assert_eq!(
            first.snapshot.source_enemy_projectile,
            Some(ActorSourceEnemyProjectileMetadata {
                x_fraction: 0x80,
                y_fraction: 0x80,
                x_velocity: 0x0180,
                y_velocity: 0xFF80,
                lifetime_ticks: 4,
            })
        );

        let mut tick_prompt = prompt;
        tick_prompt.source_shell_scan_tick = true;
        let second = shot.update(&tick_prompt);

        assert_eq!(second.snapshot.position, Point::new(13, 79));
        assert_eq!(second.snapshot.velocity, Velocity::new(2, 0));
        assert_eq!(
            second.snapshot.source_enemy_projectile,
            Some(ActorSourceEnemyProjectileMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0180,
                y_velocity: 0xFF80,
                lifetime_ticks: 3,
            })
        );
    }

    #[test]
    fn driver_applies_source_shell_scan_lifetime_cadence() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::EnemyLaser,
            ActorBehaviorProfile {
                lander_shot_lifetime_steps: 20,
                ..ActorBehaviorProfile::default()
            },
        );
        let shot =
            driver.spawn_enemy_laser_from_spawn(Point::new(80, 120), Velocity::new(0, 0), None);

        let lifetimes = (0..=SOURCE_SHELL_SCAN_INITIAL_DELAY_STEPS)
            .map(|_| {
                let report = driver.step(GameInput::NONE);
                snapshot_for(&report, shot)
                    .source_enemy_projectile
                    .expect("enemy laser should publish source projectile metadata")
                    .lifetime_ticks
            })
            .collect::<Vec<_>>();

        assert_eq!(lifetimes, vec![20, 20, 20, 20, 20, 20, 19]);
    }

    #[test]
    fn enemy_laser_collision_consumes_life_and_respawns_player() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_enemy_laser_from_spawn(Point::new(42, 120), Velocity::new(0, 0), None);

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::Playing);
        assert_eq!(report.lives, 2);
        assert!(report.sounds.contains(&SoundCue::Explosion));
        assert!(!report.sounds.contains(&SoundCue::GameOver));
        assert!(report.commands.contains(&GameCommand::PlayerKilled));
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        let respawned = driver.step(GameInput::NONE);
        assert_eq!(respawned.phase, Phase::Playing);
        assert_eq!(respawned.lives, 2);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn enemy_laser_collision_on_final_life_enters_game_over() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.spawn_player();
        driver.spawn_enemy_laser_from_spawn(Point::new(42, 120), Velocity::new(0, 0), None);

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::GameOver);
        assert_eq!(report.lives, 0);
        assert!(report.sounds.contains(&SoundCue::Explosion));
        assert!(report.sounds.contains(&SoundCue::GameOver));
    }

    #[test]
    fn hyperspace_clears_source_shells_without_spending_stock_or_life() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 3;
        driver.smart_bombs = INITIAL_SMART_BOMBS;
        driver.spawn_player();
        driver.spawn_enemy_laser_from_spawn(Point::new(42, 120), Velocity::new(0, 0), None);
        driver.spawn_bomb_for_test(Point::new(90, 120));

        let report = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert_eq!(report.lives, 3);
        assert_eq!(report.smart_bombs, INITIAL_SMART_BOMBS);
        assert!(report.commands.contains(&GameCommand::Hyperspace));
        assert!(!report.commands.contains(&GameCommand::PlayerKilled));
        assert!(report.sounds.contains(&SoundCue::Hyperspace));
        assert!(!report.sounds.contains(&SoundCue::GameOver));
        assert_eq!(driver.snapshot_count(ActorKind::EnemyLaser), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Bomb), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn hyperspace_leaves_hostiles_and_player_lasers_active() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 3;
        driver.smart_bombs = INITIAL_SMART_BOMBS;
        let player = driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(90, 120));
        driver.spawn_laser(Point::new(10, 40), Direction::Right, player);
        driver.spawn_enemy_laser_from_spawn(Point::new(70, 120), Velocity::new(0, 0), None);
        driver.spawn_bomb_for_test(Point::new(120, 120));

        let report = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert!(report.commands.contains(&GameCommand::Hyperspace));
        let emitted_smart_bomb = report
            .commands
            .iter()
            .any(|command| matches!(command, GameCommand::SmartBomb { .. }));
        assert!(!emitted_smart_bomb);
        assert_eq!(driver.snapshot_count(ActorKind::EnemyLaser), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Bomb), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Laser), 1);
        assert_eq!(report.score, 0);
        assert_eq!(report.smart_bombs, INITIAL_SMART_BOMBS);
    }

    #[test]
    fn hyperspace_hides_player_until_scripted_rematerialization() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 2,
                player_hyperspace_rematerialize_x: 150,
                player_hyperspace_rematerialize_y: 92,
                ..ActorBehaviorProfile::default()
            },
        );

        let entered = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        let hidden_player = snapshot_for(&entered, player);
        assert_eq!(hidden_player.bounds, None);
        assert!(!entered.draws.iter().any(|draw| draw.actor == player));
        assert!(entered.sounds.contains(&SoundCue::Hyperspace));
        assert!(!entered.sounds.contains(&SoundCue::HyperspaceMaterialize));

        let still_hidden = driver.step(GameInput {
            thrust: true,
            fire: true,
            ..GameInput::NONE
        });

        let hidden_player = snapshot_for(&still_hidden, player);
        assert_eq!(hidden_player.bounds, None);
        assert!(!still_hidden.draws.iter().any(|draw| draw.actor == player));
        assert!(!still_hidden.sounds.contains(&SoundCue::Thrust));
        assert!(
            !still_hidden.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::Laser { .. }))
            })
        );

        let rematerialized = driver.step(GameInput::NONE);
        let player_snapshot = snapshot_for(&rematerialized, player);
        assert_eq!(player_snapshot.position, Point::new(150, 92));
        assert!(player_snapshot.bounds.is_some());
        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert!(rematerialized.draws.iter().any(|draw| {
            draw.actor == player
                && draw.position == Point::new(150, 92)
                && matches!(draw.sprite, SpriteKey::PlayerRight)
        }));
    }

    #[test]
    fn hyperspace_lseed_high_enters_delayed_player_death_path() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_death_delay_steps: 2,
                player_hyperspace_death_lseed: SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD + 1,
                ..ActorBehaviorProfile::default()
            },
        );

        let entered = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        assert!(entered.commands.contains(&GameCommand::Hyperspace));
        assert!(!entered.commands.contains(&GameCommand::PlayerKilled));

        let rematerialized = driver.step(GameInput::NONE);
        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert!(!rematerialized.commands.contains(&GameCommand::PlayerKilled));
        assert_eq!(rematerialized.lives, 3);

        let pending_death = driver.step(GameInput {
            thrust: true,
            fire: true,
            ..GameInput::NONE
        });
        assert!(!pending_death.commands.contains(&GameCommand::PlayerKilled));
        assert!(!pending_death.sounds.contains(&SoundCue::Thrust));
        assert!(
            !pending_death.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::Laser { .. }))
            })
        );

        let destroyed = driver.step(GameInput::NONE);

        assert_eq!(destroyed.phase, Phase::Playing);
        assert_eq!(destroyed.lives, 2);
        assert!(destroyed.commands.contains(&GameCommand::Destroy(player)));
        assert!(destroyed.commands.contains(&GameCommand::PlayerKilled));
        assert!(destroyed.sounds.contains(&SoundCue::Explosion));
        assert!(destroyed.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Explosion {
                    kind: ExplosionKind::Player,
                    ..
                })
            )
        }));
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        driver.step(GameInput::NONE);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn hyperspace_lseed_at_source_threshold_survives() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_death_delay_steps: 1,
                player_hyperspace_death_lseed: SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD,
                ..ActorBehaviorProfile::default()
            },
        );

        driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        let rematerialized = driver.step(GameInput::NONE);
        let settled = driver.step(GameInput::NONE);

        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert!(!rematerialized.commands.contains(&GameCommand::PlayerKilled));
        assert!(!settled.commands.contains(&GameCommand::PlayerKilled));
        assert_eq!(settled.lives, 3);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn hyperspace_source_seed_controls_rematerialization_position_and_direction() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_rematerialize_x: 150,
                player_hyperspace_rematerialize_y: 92,
                player_hyperspace_source_seed: Some(ActorHyperspaceSourceSeed {
                    seed: 0x12,
                    hseed: 0x34,
                    lseed: 0,
                }),
                ..ActorBehaviorProfile::default()
            },
        );

        driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        let rematerialized = driver.step(GameInput::NONE);

        let player_snapshot = snapshot_for(&rematerialized, player);
        assert_eq!(
            player_snapshot.position,
            Point::new(0x70, i16::from((0x34_u8 >> 1) + SOURCE_PLAYFIELD_Y_MIN))
        );
        assert!(rematerialized.draws.iter().any(|draw| {
            draw.actor == player
                && draw.position == player_snapshot.position
                && matches!(draw.sprite, SpriteKey::PlayerLeft)
        }));
        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert_eq!(rematerialized.source_background_left, 0x1234);
        assert!(
            rematerialized
                .commands
                .contains(&GameCommand::SetSourceBackgroundLeft(0x1234))
        );
        assert!(!rematerialized.commands.contains(&GameCommand::PlayerKilled));
    }

    #[test]
    fn driver_advances_hyperspace_source_rng_for_default_player_behavior() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_kind_behavior(
            ActorKind::Player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_rematerialize_x: 150,
                player_hyperspace_rematerialize_y: 92,
                ..ActorBehaviorProfile::default()
            },
        );
        let mut expected_source = SOURCE_PLAYFIELD_START_RNG;
        expected_source.advance();
        expected_source.advance();

        driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        let rematerialized = driver.step(GameInput::NONE);

        let expected_y =
            i16::from((expected_source.hseed >> 1).wrapping_add(SOURCE_PLAYFIELD_Y_MIN));
        let expected_position = if expected_source.hseed & 1 != 0 {
            Point::new(0x20, expected_y)
        } else {
            Point::new(0x70, expected_y)
        };
        let player_snapshot = snapshot_for(&rematerialized, player);
        assert_eq!(player_snapshot.position, expected_position);
        assert_eq!(
            rematerialized.source_background_left,
            u16::from_be_bytes([expected_source.seed, expected_source.hseed])
        );
        assert!(rematerialized.draws.iter().any(|draw| {
            draw.actor == player
                && draw.position == expected_position
                && matches!(
                    (expected_source.hseed & 1 != 0, draw.sprite),
                    (true, SpriteKey::PlayerRight) | (false, SpriteKey::PlayerLeft)
                )
        }));
    }

    #[test]
    fn playing_step_report_carries_driver_source_rng_snapshot() {
        let mut driver = ActorGameDriver::new();

        let attract = driver.step(GameInput::NONE);
        assert_eq!(attract.source_rng, None);

        driver.phase = Phase::Playing;
        let mut expected_source = SOURCE_PLAYFIELD_START_RNG;
        let expected_snapshot = expected_source.advance().snapshot();

        let playing = driver.step(GameInput::NONE);

        assert_eq!(playing.source_rng, Some(expected_snapshot));
        assert_eq!(
            playing.game_state().world.source_rng,
            clean_source_rng(expected_snapshot)
        );
    }

    #[test]
    fn xyzzy_invincibility_keeps_player_alive_on_enemy_laser_contact() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_enemy_laser_from_spawn(Point::new(42, 120), Velocity::new(0, 0), None);

        let report = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                invincible: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert_ne!(report.lives, 0);
    }

    #[test]
    fn first_wave_humans_publish_source_metadata_and_picture_frames() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let live = step_until_driver_player_start_completes(&mut driver, 1);
        let human = live
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.kind == ActorKind::Human && snapshot.position == Point::new(0x1C, 0xE1)
            })
            .expect("source first-wave human should publish its restore position");

        assert_eq!(
            human.source_human,
            Some(ActorSourceHumanMetadata {
                x_fraction: 0x81,
                y_fraction: 0x00,
                picture_frame: 3,
                target_slot_index: 1,
            })
        );
        assert!(live.draws.iter().any(|draw| {
            draw.actor == human.id
                && draw.sprite == SpriteKey::Human
                && matches!(draw.effect, VisualEffect::SourceHumanFrame { frame: 3 })
        }));
    }

    #[test]
    fn source_human_walk_uses_seeded_left_branch_and_terrain_y_target() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let human_id =
            driver.spawn_human_from_spawn(source_human_spawn_for_test(Point::new(64, 220), 1, 0));
        driver.step(GameInput::NONE);
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };

        let walked = driver.step(GameInput::NONE);
        let human = snapshot_for(&walked, human_id);

        assert_eq!(
            walked.source_rng.map(|source_rng| source_rng.seed),
            Some(17)
        );
        assert_eq!(human.position, Point::new(63, 221));
        assert_eq!(
            human
                .source_human
                .map(|source| (source.x_fraction, source.picture_frame)),
            Some((0xE0, 1))
        );
    }

    #[test]
    fn source_human_walk_turns_on_low_source_seed_without_y_step() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let human_id =
            driver.spawn_human_from_spawn(source_human_spawn_for_test(Point::new(64, 220), 1, 0));
        driver.step(GameInput::NONE);
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 222,
        };

        let turned = driver.step(GameInput::NONE);
        let human = snapshot_for(&turned, human_id);

        assert_eq!(turned.source_rng.map(|source_rng| source_rng.seed), Some(0));
        assert_eq!(human.position, Point::new(64, 220));
        assert_eq!(
            human
                .source_human
                .map(|source| (source.x_fraction, source.picture_frame)),
            Some((0x20, 2))
        );
    }

    #[test]
    fn source_human_walk_process_moves_only_selected_target_slot() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let slot0 =
            driver.spawn_human_from_spawn(source_human_spawn_for_test(Point::new(48, 220), 0, 0));
        let slot1 =
            driver.spawn_human_from_spawn(source_human_spawn_for_test(Point::new(64, 220), 1, 0));
        let slot2 =
            driver.spawn_human_from_spawn(source_human_spawn_for_test(Point::new(80, 220), 2, 0));

        driver.step(GameInput::NONE);
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        let walked = driver.step(GameInput::NONE);

        assert_eq!(driver.source_astronaut_cursor, Some(1));
        assert_eq!(
            driver.source_astronaut_sleep_ticks,
            SOURCE_ASTRONAUT_PROCESS_SLEEP_TICKS
        );
        assert_eq!(snapshot_for(&walked, slot0).position, Point::new(48, 220));
        assert_eq!(snapshot_for(&walked, slot1).position, Point::new(63, 221));
        assert_eq!(snapshot_for(&walked, slot2).position, Point::new(80, 220));

        let sleeping = driver.step(GameInput::NONE);
        assert_eq!(driver.source_astronaut_sleep_ticks, 1);
        assert_eq!(snapshot_for(&sleeping, slot1).position, Point::new(63, 221));
        assert_eq!(snapshot_for(&sleeping, slot2).position, Point::new(80, 220));
    }

    #[test]
    fn source_human_walk_process_suppresses_inactive_first_wave_slots() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let mut human_ids = Vec::new();
        for slot in 0..usize::from(SOURCE_START_HUMAN_COUNT) {
            human_ids.push(driver.spawn_human_from_spawn(source_human_spawn_for_test(
                Point::new(40 + i16::try_from(slot).expect("slot fits i16") * 8, 220),
                slot,
                0,
            )));
        }

        driver.step(GameInput::NONE);
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        let slot1_walked = driver.step(GameInput::NONE);
        assert_eq!(
            snapshot_for(&slot1_walked, human_ids[1]).position,
            Point::new(47, 221)
        );

        driver.source_astronaut_sleep_ticks = 0;
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        let slot2_suppressed = driver.step(GameInput::NONE);

        assert_eq!(driver.source_astronaut_cursor, Some(2));
        assert_eq!(
            snapshot_for(&slot2_suppressed, human_ids[2]).position,
            Point::new(56, 220)
        );
    }

    #[test]
    fn source_human_walk_process_suppression_counts_plain_humans() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let mut source_ids = Vec::new();
        for slot in 0..9usize {
            source_ids.push(driver.spawn_human_from_spawn(source_human_spawn_for_test(
                Point::new(40 + i16::try_from(slot).expect("slot fits i16") * 8, 220),
                slot,
                0,
            )));
        }
        driver.spawn_human_for_test(Point::new(128, 220));

        driver.step(GameInput::NONE);
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        let slot1_walked = driver.step(GameInput::NONE);
        assert_eq!(
            snapshot_for(&slot1_walked, source_ids[1]).position,
            Point::new(47, 221)
        );

        driver.source_astronaut_sleep_ticks = 0;
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        let slot2_suppressed = driver.step(GameInput::NONE);

        assert_eq!(driver.source_astronaut_cursor, Some(2));
        assert_eq!(
            snapshot_for(&slot2_suppressed, source_ids[2]).position,
            Point::new(56, 220)
        );
    }

    #[test]
    fn source_lander_prefers_configured_target_human_slot() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Lander,
            ActorBehaviorProfile {
                lander_seek_speed: 4,
                lander_fire_period_steps: u64::MAX,
                ..ActorBehaviorProfile::default()
            },
        );
        let lander_id = driver.spawn_lander_from_spawn(ActorLanderSpawn {
            position: Point::new(100, 100),
            source: Some(ActorSourceLanderMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: u8::MAX,
                sleep_ticks: 0,
                picture_frame: 0,
                target_human_index: Some(7),
            }),
        });
        driver.spawn_human_for_test(Point::new(90, 100));
        driver.spawn_human_from_spawn(ActorHumanSpawn {
            position: Point::new(160, 100),
            mode: HumanMode::Grounded,
            source: Some(ActorSourceHumanMetadata {
                x_fraction: 0,
                y_fraction: 0,
                picture_frame: 0,
                target_slot_index: 7,
            }),
        });

        driver.step(GameInput::NONE);
        let targeted = driver.step(GameInput::NONE);

        assert_eq!(
            snapshot_for(&targeted, lander_id).position,
            Point::new(104, 100)
        );
    }

    #[test]
    fn behavior_script_can_define_level_wide_actor_motion() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 4,
            lander_fire_period_steps: u64::MAX,
            ..ActorBehaviorProfile::default()
        };
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(ActorKind::Lander, lander_behavior);
        let lander_id = driver.spawn_lander_for_test(Point::new(80, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));

        let first = driver.step(GameInput::NONE);
        assert_eq!(snapshot_for(&first, lander_id).position.x, 79);

        let seeking = driver.step(GameInput::NONE);
        assert_eq!(snapshot_for(&seeking, lander_id).position.x, 83);
    }

    #[test]
    fn behavior_script_can_define_bomber_and_pod_motion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 3,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Pod,
            ActorBehaviorProfile {
                pod_drift_speed: 4,
                ..ActorBehaviorProfile::default()
            },
        );
        let bomber = driver.spawn_bomber_for_test(Point::new(100, 80));
        let pod = driver.spawn_pod_for_test(Point::new(100, 88));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, bomber).position, Point::new(97, 80));
        assert_eq!(snapshot_for(&report, pod).position, Point::new(104, 88));
    }

    #[test]
    fn behavior_script_can_choose_mutant_drift_mode() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Mutant,
            ActorBehaviorProfile {
                mutant_seek_speed: 4,
                mutant_mode: HostileMovementMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        let mutant = driver.spawn_mutant(Point::new(100, 100));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, mutant).position, Point::new(96, 100));
    }

    #[test]
    fn behavior_script_can_choose_bomber_and_pod_targeting_modes() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 5,
                bomber_bomb_period_steps: u64::MAX,
                bomber_mode: HostileMovementMode::ChasePlayer,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Pod,
            ActorBehaviorProfile {
                pod_drift_speed: 6,
                pod_mode: HostileMovementMode::ChasePlayer,
                ..ActorBehaviorProfile::default()
            },
        );
        let bomber = driver.spawn_bomber_for_test(Point::new(70, 80));
        let pod = driver.spawn_pod_for_test(Point::new(70, 88));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, bomber).position, Point::new(65, 85));
        assert_eq!(snapshot_for(&report, pod).position, Point::new(64, 94));
    }

    #[test]
    fn bomber_actor_lays_scripted_bomb_actor() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 0,
                bomber_bomb_period_steps: 1,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.spawn_bomber_for_test(Point::new(100, 80));

        let report = driver.step(GameInput::NONE);

        assert!(report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Bomb {
                    position: Point { x: 100, y: 80 },
                    source: None,
                })
            )
        }));

        let live = driver.step(GameInput::NONE);
        assert_eq!(driver.snapshot_count(ActorKind::Bomb), 1);
        let bomb = live
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Bomb)
            .expect("spawned bomb should publish an actor snapshot");
        let source_projectile = bomb
            .source_enemy_projectile
            .expect("bomb should publish source projectile metadata");
        assert_eq!(source_projectile.x_velocity, 0);
        assert_eq!(source_projectile.y_velocity, 0);
        assert!(source_projectile.lifetime_ticks > 0);
        assert!(live.draws.iter().any(|draw| draw.sprite == SpriteKey::Bomb));
    }

    #[test]
    fn source_bomber_bomb_spawn_carries_source_shell_fractions() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 0,
                bomber_bomb_period_steps: 1,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Bomb,
            ActorBehaviorProfile {
                bomb_lifetime_steps: 5,
                ..ActorBehaviorProfile::default()
            },
        );
        let initial_source = ActorSourceBomberMetadata {
            x_fraction: 0x6D,
            y_fraction: 0x7B,
            x_velocity: 0,
            y_velocity: 0,
            picture_frame: 0,
            cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
            sleep_ticks: 0,
            source_slot: 0,
        };
        let bomber = driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: Point::new(100, 80),
            source: Some(initial_source),
        });

        let report = driver.step(GameInput::NONE);
        let (expected_position, expected_source) = expected_source_bomber_after_motion(
            Point::new(100, 80),
            initial_source,
            report.step,
            bomber,
            report.source_rng,
            None,
        );
        let expected_lifetime_ticks = report
            .source_rng
            .map(actor_source_bomber_bomb_lifetime_ticks)
            .expect("playing report should carry source rng");
        let bomber_snapshot = snapshot_for(&report, bomber);
        assert_eq!(bomber_snapshot.position, expected_position);
        assert_eq!(bomber_snapshot.source_bomber, Some(expected_source));

        assert!(report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Bomb {
                    position,
                    source: Some(ActorSourceEnemyProjectileMetadata {
                        x_fraction,
                        y_fraction,
                        x_velocity: 0,
                        y_velocity: 0,
                        lifetime_ticks,
                    }),
                }) if *position == Point::new(100, 80)
                    && *x_fraction == initial_source.x_fraction
                    && *y_fraction == initial_source.y_fraction
                    && *lifetime_ticks == expected_lifetime_ticks
            )
        }));

        let live = driver.step(GameInput::NONE);
        let bomb = live
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.source_enemy_projectile.is_some_and(|source| {
                    source.x_fraction == initial_source.x_fraction
                        && source.y_fraction == initial_source.y_fraction
                })
            })
            .expect("source-backed bomber bomb should publish source shell fractions");

        assert_eq!(
            bomb.source_enemy_projectile,
            Some(ActorSourceEnemyProjectileMetadata {
                x_fraction: initial_source.x_fraction,
                y_fraction: initial_source.y_fraction,
                x_velocity: 0,
                y_velocity: 0,
                lifetime_ticks: expected_lifetime_ticks,
            })
        );
    }

    #[test]
    fn source_bomber_bomb_spawn_uses_source_rng_gate() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 14,
        };
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 0,
                bomber_bomb_period_steps: 1,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: Point::new(100, 80),
            source: Some(ActorSourceBomberMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                source_slot: 0,
            }),
        });

        let report = driver.step(GameInput::NONE);
        let source_rng = report
            .source_rng
            .expect("playing report should carry source rng");

        assert_ne!(source_rng.lseed & 0x07, 0);
        assert!(
            !report.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::Bomb { .. }))
            })
        );
    }

    #[test]
    fn source_bomber_bomb_spawn_respects_getshl_bounds() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 0,
                bomber_bomb_period_steps: 1,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: Point::new(SOURCE_SHELL_X_MAX, 80),
            source: Some(ActorSourceBomberMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                source_slot: 0,
            }),
        });
        driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: Point::new(SOURCE_SHELL_X_MAX - 1, i16::from(SOURCE_PLAYFIELD_Y_MIN)),
            source: Some(ActorSourceBomberMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                source_slot: 0,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert!(
            report.commands.iter().all(|command| {
                !matches!(command, GameCommand::Spawn(SpawnRequest::Bomb { .. }))
            })
        );
        let live = driver.step(GameInput::NONE);
        assert_eq!(bomb_shell_snapshot_count(&live), 0);
    }

    #[test]
    fn source_bomber_motion_uses_seeded_picture_and_y_velocity() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_bomb_period_steps: u64::MAX,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.spawn_player();
        let player_report = driver.step(GameInput::NONE);
        let player_position = player_report
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player)
            .map(|snapshot| snapshot.position)
            .expect("player should publish a prompt snapshot");
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 10,
        };
        let initial_source = ActorSourceBomberMetadata {
            x_fraction: 0x10,
            y_fraction: 0x20,
            x_velocity: 0x0100,
            y_velocity: 0,
            picture_frame: 2,
            cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
            sleep_ticks: 0,
            source_slot: 3,
        };
        let bomber_position = Point::new(96, player_position.y - 8);
        let bomber = driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: bomber_position,
            source: Some(initial_source),
        });

        let report = driver.step(GameInput::NONE);
        let (expected_position, expected_source) = expected_source_bomber_after_motion(
            bomber_position,
            initial_source,
            report.step,
            bomber,
            report.source_rng,
            Some(player_position),
        );
        let snapshot = snapshot_for(&report, bomber);

        assert_eq!(snapshot.position, expected_position);
        assert_eq!(snapshot.source_bomber, Some(expected_source));
        assert_ne!(expected_source.y_velocity, 0);
        assert!(report.draws.iter().any(|draw| {
            draw.actor == bomber
                && matches!(
                    draw.effect,
                    VisualEffect::SourceBomberFrame { frame }
                        if frame == expected_source.picture_frame
                )
        }));
        assert!(
            !report.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::Bomb { .. }))
            })
        );
    }

    #[test]
    fn source_bomber_offscreen_motion_adjusts_cruise_altitude() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.source_rng = ActorSourceRng {
            seed: 0,
            hseed: 0,
            lseed: 13,
        };
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_bomb_period_steps: u64::MAX,
                ..ActorBehaviorProfile::default()
            },
        );
        let initial_source = ActorSourceBomberMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            picture_frame: 1,
            cruise_altitude: SOURCE_BOMBER_CRUISE_ALTITUDE,
            sleep_ticks: 0,
            source_slot: 3,
        };
        let bomber_position = Point::new(100, 0);
        let bomber = driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: bomber_position,
            source: Some(initial_source),
        });

        let report = driver.step(GameInput::NONE);
        let (expected_position, expected_source) = expected_source_bomber_after_motion(
            bomber_position,
            initial_source,
            report.step,
            bomber,
            report.source_rng,
            None,
        );
        let snapshot = snapshot_for(&report, bomber);

        assert_eq!(snapshot.position, expected_position);
        assert_eq!(snapshot.source_bomber, Some(expected_source));
        assert_ne!(
            expected_source.cruise_altitude,
            SOURCE_BOMBER_CRUISE_ALTITUDE
        );
        assert_ne!(expected_source.y_velocity, 0);
    }

    #[test]
    fn source_bomb_spawn_commands_respect_getshl_bounds() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;

        driver.apply_commands(&[
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(SOURCE_SHELL_X_MAX, 100),
                source: Some(ActorSourceEnemyProjectileMetadata {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0,
                    y_velocity: 0,
                    lifetime_ticks: 0,
                }),
            }),
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(SOURCE_SHELL_X_MAX - 1, i16::from(SOURCE_PLAYFIELD_Y_MIN)),
                source: Some(ActorSourceEnemyProjectileMetadata {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0,
                    y_velocity: 0,
                    lifetime_ticks: 0,
                }),
            }),
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(SOURCE_SHELL_X_MAX, 108),
                source: None,
            }),
        ]);
        let report = driver.step(GameInput::NONE);

        let bombs = report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomb)
            .collect::<Vec<_>>();
        assert_eq!(bombs.len(), 1);
        assert_eq!(bombs[0].position, Point::new(SOURCE_SHELL_X_MAX, 108));
    }

    #[test]
    fn source_bomb_spawn_preserves_scripted_lifetime_ticks() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Bomb,
            ActorBehaviorProfile {
                bomb_lifetime_steps: 40,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.apply_commands(&[GameCommand::Spawn(SpawnRequest::Bomb {
            position: Point::new(80, 100),
            source: Some(ActorSourceEnemyProjectileMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                lifetime_ticks: 9,
            }),
        })]);

        let lifetimes = (0..=SOURCE_SHELL_SCAN_INITIAL_DELAY_STEPS)
            .map(|_| {
                let report = driver.step(GameInput::NONE);
                report
                    .snapshots
                    .iter()
                    .find(|snapshot| snapshot.kind == ActorKind::Bomb)
                    .and_then(|snapshot| snapshot.source_enemy_projectile)
                    .expect("source-backed bomb should publish source projectile metadata")
                    .lifetime_ticks
            })
            .collect::<Vec<_>>();

        assert_eq!(lifetimes, vec![9, 9, 9, 9, 9, 9, 8]);
    }

    #[test]
    fn source_enemy_laser_spawn_commands_respect_getshl_bounds() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;

        driver.apply_commands(&[
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position: Point::new(SOURCE_SHELL_X_MAX, 100),
                velocity: Velocity::new(0, 0),
                source: None,
            }),
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position: Point::new(SOURCE_SHELL_X_MAX - 1, i16::from(SOURCE_PLAYFIELD_Y_MIN)),
                velocity: Velocity::new(0, 0),
                source: None,
            }),
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position: Point::new(SOURCE_SHELL_X_MAX - 1, 100),
                velocity: Velocity::new(0, 0),
                source: None,
            }),
        ]);
        let report = driver.step(GameInput::NONE);

        let shots = report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
            .collect::<Vec<_>>();
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].position, Point::new(SOURCE_SHELL_X_MAX - 1, 100));
    }

    #[test]
    fn source_enemy_laser_spawn_preserves_scripted_projectile_metadata() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::EnemyLaser,
            ActorBehaviorProfile {
                lander_shot_lifetime_steps: 40,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.apply_commands(&[GameCommand::Spawn(SpawnRequest::EnemyLaser {
            position: Point::new(80, 100),
            velocity: Velocity::new(0, 0),
            source: Some(ActorSourceEnemyProjectileMetadata {
                x_fraction: 0x55,
                y_fraction: 0x66,
                x_velocity: 0,
                y_velocity: 0,
                lifetime_ticks: 7,
            }),
        })]);

        let mut first_source = None;
        let lifetimes = (0..=SOURCE_SHELL_SCAN_INITIAL_DELAY_STEPS)
            .map(|_| {
                let report = driver.step(GameInput::NONE);
                let source = report
                    .snapshots
                    .iter()
                    .find(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
                    .and_then(|snapshot| snapshot.source_enemy_projectile)
                    .expect("scripted enemy laser should publish source metadata");
                first_source.get_or_insert(source);
                source.lifetime_ticks
            })
            .collect::<Vec<_>>();

        assert_eq!(
            first_source,
            Some(ActorSourceEnemyProjectileMetadata {
                x_fraction: 0x55,
                y_fraction: 0x66,
                x_velocity: 0,
                y_velocity: 0,
                lifetime_ticks: 7,
            })
        );
        assert_eq!(lifetimes, vec![7, 7, 7, 7, 7, 7, 6]);
    }

    #[test]
    fn source_shell_cap_blocks_and_releases_hostile_projectile_slots() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        for index in 0..SOURCE_SHELL_LIMIT {
            driver.spawn_enemy_laser_from_spawn(
                Point::new(40 + index as i16, 120),
                Velocity::new(0, 0),
                None,
            );
        }
        let filled = driver.step(GameInput::NONE);
        assert_eq!(source_shell_snapshot_count(&filled), SOURCE_SHELL_LIMIT);

        driver.apply_commands(&[
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position: Point::new(96, 96),
                velocity: Velocity::new(0, 0),
                source: None,
            }),
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(100, 100),
                source: None,
            }),
        ]);
        let capped = driver.step(GameInput::NONE);
        assert_eq!(source_shell_snapshot_count(&capped), SOURCE_SHELL_LIMIT);
        assert!(
            capped
                .snapshots
                .iter()
                .all(|snapshot| snapshot.kind != ActorKind::Bomb)
        );

        let freed_shell = capped
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
            .expect("filled source shell list should contain enemy lasers")
            .id;
        driver.apply_commands(&[
            GameCommand::Destroy(freed_shell),
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(100, 100),
                source: None,
            }),
        ]);
        let refilled = driver.step(GameInput::NONE);

        assert_eq!(source_shell_snapshot_count(&refilled), SOURCE_SHELL_LIMIT);
        assert!(
            refilled
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::Bomb)
        );
    }

    #[test]
    fn source_bomb_shell_cap_blocks_bombs_without_blocking_enemy_lasers() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        for index in 0..SOURCE_ACTIVE_BOMBER_BOMB_LIMIT {
            driver.spawn_bomb_for_test(Point::new(40 + (index as i16 * 4), 120));
        }
        let filled = driver.step(GameInput::NONE);
        assert_eq!(
            bomb_shell_snapshot_count(&filled),
            SOURCE_ACTIVE_BOMBER_BOMB_LIMIT
        );
        assert!(
            source_shell_snapshot_count(&filled) < SOURCE_SHELL_LIMIT,
            "bomb shell cap should fill before the shared source shell cap"
        );

        driver.apply_commands(&[
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(112, 100),
                source: None,
            }),
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position: Point::new(116, 100),
                velocity: Velocity::new(0, 0),
                source: None,
            }),
        ]);
        let capped = driver.step(GameInput::NONE);
        assert_eq!(
            bomb_shell_snapshot_count(&capped),
            SOURCE_ACTIVE_BOMBER_BOMB_LIMIT
        );
        assert_eq!(enemy_laser_snapshot_count(&capped), 1);
        assert!(
            capped
                .snapshots
                .iter()
                .all(|snapshot| snapshot.kind != ActorKind::Bomb
                    || snapshot.position != Point::new(112, 100))
        );

        let freed_bomb = capped
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Bomb)
            .expect("filled source bomb list should contain bomb shells")
            .id;
        driver.apply_commands(&[
            GameCommand::Destroy(freed_bomb),
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(112, 100),
                source: None,
            }),
        ]);
        let refilled = driver.step(GameInput::NONE);

        assert_eq!(
            bomb_shell_snapshot_count(&refilled),
            SOURCE_ACTIVE_BOMBER_BOMB_LIMIT
        );
        assert_eq!(enemy_laser_snapshot_count(&refilled), 1);
        assert!(refilled.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Bomb && snapshot.position == Point::new(112, 100)
        }));
    }

    #[test]
    fn behavior_script_can_define_swarmer_motion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.set_kind_behavior(
            ActorKind::Swarmer,
            ActorBehaviorProfile {
                swarmer_seek_speed: 5,
                ..ActorBehaviorProfile::default()
            },
        );
        let swarmer = driver.spawn_swarmer_for_test(Point::new(70, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, swarmer).position, Point::new(65, 120));
    }

    #[test]
    fn behavior_script_can_define_baiter_motion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.set_kind_behavior(
            ActorKind::Baiter,
            ActorBehaviorProfile {
                baiter_seek_speed: 6,
                baiter_fire_period_steps: u64::MAX,
                ..ActorBehaviorProfile::default()
            },
        );
        let baiter = driver.spawn_baiter_for_test(Point::new(70, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, baiter).position, Point::new(64, 120));
    }

    #[test]
    fn behavior_script_can_choose_swarmer_and_baiter_drift_modes() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Swarmer,
            ActorBehaviorProfile {
                swarmer_seek_speed: 4,
                swarmer_fire_period_steps: u64::MAX,
                swarmer_mode: HostileMovementMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Baiter,
            ActorBehaviorProfile {
                baiter_seek_speed: 5,
                baiter_fire_period_steps: u64::MAX,
                baiter_mode: HostileMovementMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        let swarmer = driver.spawn_swarmer_for_test(Point::new(70, 120));
        let baiter = driver.spawn_baiter_for_test(Point::new(80, 124));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, swarmer).position, Point::new(66, 120));
        assert_eq!(snapshot_for(&report, baiter).position, Point::new(75, 124));
    }

    #[test]
    fn baiter_timer_spawns_source_baiter_from_wave_profile() {
        let mut driver = started_driver();

        driver.set_baiter_timer_for_test(1);
        let report = driver.step(GameInput::NONE);

        let baiter_spawn = report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::Baiter { position, source }) => {
                    Some((*position, *source))
                }
                _ => None,
            })
            .expect("expired baiter timer should spawn a baiter");
        assert_eq!(
            baiter_spawn,
            (
                Point::new(228, 144),
                Some(ActorSourceBaiterMetadata {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0xFFC0,
                    y_velocity: 0xFF80,
                    shot_timer: SOURCE_BAITER_INITIAL_SHOT_TIMER,
                    sleep_ticks: 0,
                    picture_frame: 0,
                })
            )
        );

        let live = driver.step(GameInput::NONE);
        assert_eq!(driver.snapshot_count(ActorKind::Baiter), 1);
        assert!(live.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Baiter
                && snapshot.position == Point::new(227, 143)
                && snapshot.source_baiter
                    == Some(ActorSourceBaiterMetadata {
                        x_fraction: 0,
                        y_fraction: 0x80,
                        x_velocity: 0xFFC0,
                        y_velocity: 0xFF80,
                        shot_timer: 7,
                        sleep_ticks: SOURCE_BAITER_LOOP_SLEEP_TICKS,
                        picture_frame: 1,
                    })
        }));
        assert!(live.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::Baiter
                && matches!(draw.effect, VisualEffect::SourceBaiterFrame { frame: 1 })
        }));
    }

    #[test]
    fn source_swarmer_shot_timer_spawns_hostile_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        let player = driver.spawn_player();
        driver.snapshots.insert(
            player,
            actor_snapshot(player.value(), ActorKind::Player, Point::new(42, 120)),
        );
        let start = Point::new(25, 100);
        let source = ActorSourceSwarmerMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0x0020,
            y_velocity: 0,
            acceleration: 0,
            sleep_ticks: 0,
            shot_timer: 1,
            horizontal_seek_pending: false,
        };
        let swarmer = driver.spawn_swarmer_from_spawn(ActorSwarmerSpawn {
            position: start,
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let report_source_rng = report
            .source_rng
            .expect("playing report should carry source rng");
        let prompt = source_mutant_prompt_for_test(
            report.step,
            report.wave,
            report_source_rng,
            Point::new(42, 120),
            Velocity::default(),
        );
        let mut expected_source = source;
        expected_source.y_velocity = source_mini_swarmer_y_velocity(
            source.y_velocity,
            source.acceleration,
            120,
            start.y,
            report_source_rng.seed,
        );
        let (expected_x, expected_x_fraction) =
            actor_source_axis_step(start.x, source.x_fraction, expected_source.x_velocity);
        let (expected_y, expected_y_fraction) = actor_source_active_object_y_step(
            start.y,
            source.y_fraction,
            expected_source.y_velocity,
        );
        let expected_position = Point::new(expected_x, expected_y);
        expected_source.x_fraction = expected_x_fraction;
        expected_source.y_fraction = expected_y_fraction;
        expected_source.shot_timer = source_rmax(
            clamped_source_swarmer_shot_reset(ActorSourceWaveProfile::for_wave(report.wave)),
            report_source_rng.seed,
        );
        expected_source.sleep_ticks = SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS;
        let (expected_velocity, expected_projectile_source) =
            actor_source_mini_swarmer_fireball(expected_position, &prompt, expected_source)
                .expect("expected source swarmer fireball");

        assert!(report.sounds.contains(&SoundCue::SwarmerShot));
        let swarmer_shot = report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position,
                    velocity,
                    source,
                }) => Some((*position, *velocity, *source)),
                _ => None,
            })
            .expect("source swarmer should emit a hostile shot command");
        assert_eq!(
            swarmer_shot,
            (
                expected_position,
                expected_velocity,
                Some(expected_projectile_source)
            )
        );
        assert_eq!(
            snapshot_for(&report, swarmer).source_swarmer,
            Some(expected_source)
        );
    }

    #[test]
    fn source_swarmer_shot_direction_gate_suppresses_fireball_and_sound() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        let player = driver.spawn_player();
        driver.snapshots.insert(
            player,
            actor_snapshot(player.value(), ActorKind::Player, Point::new(42, 120)),
        );
        let start = Point::new(48, 100);
        let source = ActorSourceSwarmerMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0x0020,
            y_velocity: 0,
            acceleration: 0,
            sleep_ticks: 0,
            shot_timer: 1,
            horizontal_seek_pending: false,
        };
        let swarmer = driver.spawn_swarmer_from_spawn(ActorSwarmerSpawn {
            position: start,
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let report_source_rng = report
            .source_rng
            .expect("playing report should carry source rng");
        let mut expected_source = source;
        expected_source.y_velocity = source_mini_swarmer_y_velocity(
            source.y_velocity,
            source.acceleration,
            120,
            start.y,
            report_source_rng.seed,
        );
        let (expected_x, expected_x_fraction) =
            actor_source_axis_step(start.x, source.x_fraction, expected_source.x_velocity);
        let (expected_y, expected_y_fraction) = actor_source_active_object_y_step(
            start.y,
            source.y_fraction,
            expected_source.y_velocity,
        );
        expected_source.x_fraction = expected_x_fraction;
        expected_source.y_fraction = expected_y_fraction;
        expected_source.shot_timer = source_rmax(
            clamped_source_swarmer_shot_reset(ActorSourceWaveProfile::for_wave(report.wave)),
            report_source_rng.seed,
        );
        expected_source.sleep_ticks = SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS;

        assert!(!report.sounds.contains(&SoundCue::SwarmerShot));
        assert!(!report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    source: Some(_),
                    ..
                })
            )
        }));
        assert_eq!(
            snapshot_for(&report, swarmer).position,
            Point::new(expected_x, expected_y)
        );
        assert_eq!(
            snapshot_for(&report, swarmer).source_swarmer,
            Some(expected_source)
        );
    }

    #[test]
    fn source_swarmer_full_shell_cap_suppresses_fireball_and_sound() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        let player = driver.spawn_player();
        driver.snapshots.insert(
            player,
            actor_snapshot(player.value(), ActorKind::Player, Point::new(42, 120)),
        );
        for index in 0..SOURCE_SHELL_LIMIT {
            let id = ActorId::new(10_000 + index as u64);
            driver.snapshots.insert(
                id,
                actor_snapshot(id.value(), ActorKind::EnemyLaser, Point::new(64, 120)),
            );
        }
        driver.spawn_swarmer_from_spawn(ActorSwarmerSpawn {
            position: Point::new(25, 100),
            source: Some(ActorSourceSwarmerMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0020,
                y_velocity: 0,
                acceleration: 0,
                sleep_ticks: 0,
                shot_timer: 1,
                horizontal_seek_pending: false,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert!(!report.sounds.contains(&SoundCue::SwarmerShot));
        assert!(!report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    source: Some(_),
                    ..
                })
            )
        }));
    }

    #[test]
    fn source_baiter_shot_timer_spawns_hostile_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.spawn_player();
        driver.wave = 0;
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
            position: Point::new(70, 120),
            source: Some(ActorSourceBaiterMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: 0,
                picture_frame: 0,
            }),
        });

        let report = driver.step(GameInput::NONE);
        let report_source_rng = report
            .source_rng
            .expect("playing report should carry source rng");
        let prompt = source_mutant_prompt_for_test(
            report.step,
            report.wave,
            report_source_rng,
            Point::new(42, 120),
            Velocity::default(),
        );
        let mut expected_source = ActorSourceBaiterMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: actor_source_baiter_shot_reset(
                ActorSourceWaveProfile::for_wave(report.wave),
                report_source_rng.seed,
            ),
            sleep_ticks: SOURCE_BAITER_LOOP_SLEEP_TICKS,
            picture_frame: 1,
        };
        let (expected_velocity, expected_projectile_source) = actor_source_baiter_fireball(
            Point::new(70, 120),
            &prompt,
            expected_source,
            report_source_rng,
        )
        .expect("expected source baiter fireball");

        assert!(report.sounds.contains(&SoundCue::BaiterShot));
        let baiter_shot = report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position,
                    velocity,
                    source,
                }) => Some((*position, *velocity, *source)),
                _ => None,
            })
            .expect("source baiter should emit a hostile shot command");
        assert_eq!(
            baiter_shot,
            (
                Point::new(70, 120),
                expected_velocity,
                Some(expected_projectile_source)
            )
        );
        let (expected_x, expected_x_fraction) = actor_source_axis_step(
            70,
            expected_source.x_fraction,
            actor_source_baiter_screen_x_velocity(expected_source.x_velocity),
        );
        let (expected_y, expected_y_fraction) = actor_source_active_object_y_step(
            120,
            expected_source.y_fraction,
            expected_source.y_velocity,
        );
        expected_source.x_fraction = expected_x_fraction;
        expected_source.y_fraction = expected_y_fraction;
        assert_eq!(
            snapshot_for(&report, baiter).position,
            Point::new(expected_x, expected_y)
        );
        assert_eq!(
            snapshot_for(&report, baiter).source_baiter,
            Some(expected_source)
        );
    }

    #[test]
    fn source_baiter_full_shell_cap_suppresses_fireball_and_sound() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        let player = driver.spawn_player();
        driver.snapshots.insert(
            player,
            actor_snapshot(player.value(), ActorKind::Player, Point::new(42, 120)),
        );
        for index in 0..SOURCE_SHELL_LIMIT {
            let id = ActorId::new(20_000 + index as u64);
            driver.snapshots.insert(
                id,
                actor_snapshot(id.value(), ActorKind::EnemyLaser, Point::new(64, 120)),
            );
        }
        let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
            position: Point::new(70, 120),
            source: Some(ActorSourceBaiterMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: 0,
                picture_frame: 0,
            }),
        });

        let report = driver.step(GameInput::NONE);
        let report_source_rng = report
            .source_rng
            .expect("playing report should carry source rng");

        assert!(!report.sounds.contains(&SoundCue::BaiterShot));
        assert!(!report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    source: Some(_),
                    ..
                })
            )
        }));
        assert_eq!(
            snapshot_for(&report, baiter).source_baiter,
            Some(ActorSourceBaiterMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: actor_source_baiter_shot_reset(
                    ActorSourceWaveProfile::for_wave(report.wave),
                    report_source_rng.seed,
                ),
                sleep_ticks: SOURCE_BAITER_LOOP_SLEEP_TICKS,
                picture_frame: 1,
            })
        );
    }

    #[test]
    fn source_baiter_fireball_adds_player_velocity_when_seed_is_high() {
        let source_rng = ActorSourceRngSnapshot {
            seed: 0x90,
            hseed: 0,
            lseed: 0x44,
        };
        let prompt = source_mutant_prompt_for_test(
            7,
            2,
            source_rng,
            Point::new(80, 120),
            Velocity::new(5, -2),
        );
        let source = ActorSourceBaiterMetadata {
            x_fraction: 0x12,
            y_fraction: 0x34,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: 0,
            picture_frame: 0,
        };

        let (velocity, projectile) =
            actor_source_baiter_fireball(Point::new(70, 100), &prompt, source, source_rng)
                .expect("high-seed baiter shot should allocate");

        let expected_x_velocity = actor_sign_extend_u8_to_u16(
            (source_rng.seed & 0x1F)
                .wrapping_sub(0x10)
                .wrapping_add(80)
                .wrapping_sub(70),
        )
        .wrapping_shl(2)
        .wrapping_add(actor_source_velocity_word(5).wrapping_shl(2));
        let expected_y_velocity = actor_sign_extend_u8_to_u16(
            (source_rng.lseed & 0x1F)
                .wrapping_sub(0x10)
                .wrapping_add(120)
                .wrapping_sub(100),
        )
        .wrapping_shl(2);

        assert_eq!(
            projectile,
            ActorSourceEnemyProjectileMetadata {
                x_fraction: source.x_fraction,
                y_fraction: source.y_fraction,
                x_velocity: expected_x_velocity,
                y_velocity: expected_y_velocity,
                lifetime_ticks: SOURCE_SHELL_LIFETIME_TICKS,
            }
        );
        assert_eq!(
            velocity,
            actor_source_screen_velocity(expected_x_velocity, expected_y_velocity)
        );
    }

    #[test]
    fn source_baiter_retarget_uses_driver_source_rng_snapshot() {
        fn step_baiter_after_source_seed(seed: u8) -> (StepReport, ActorId) {
            let mut driver = ActorGameDriver::new();
            driver.phase = Phase::Playing;
            driver.wave = 1;
            driver.spawn_player();
            driver.spawn_lander_for_test(Point::new(220, 80));
            driver.step(GameInput::NONE);
            driver.source_rng = ActorSourceRng {
                seed,
                hseed: 0,
                lseed: 0,
            };
            let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
                position: Point::new(70, 120),
                source: Some(ActorSourceBaiterMetadata {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0,
                    y_velocity: 0,
                    shot_timer: 2,
                    sleep_ticks: 0,
                    picture_frame: 2,
                }),
            });

            (driver.step(GameInput::NONE), baiter)
        }

        let (held, held_baiter) = step_baiter_after_source_seed(0);
        assert_eq!(held.source_rng.map(|source_rng| source_rng.seed), Some(17));
        assert_eq!(
            snapshot_for(&held, held_baiter).position,
            Point::new(70, 120)
        );
        assert_eq!(
            snapshot_for(&held, held_baiter).source_baiter,
            Some(ActorSourceBaiterMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: SOURCE_BAITER_LOOP_SLEEP_TICKS,
                picture_frame: 0,
            })
        );

        let (retargeted, retargeted_baiter) = step_baiter_after_source_seed(70);
        assert_eq!(
            retargeted.source_rng.map(|source_rng| source_rng.seed),
            Some(227)
        );
        assert_eq!(
            snapshot_for(&retargeted, retargeted_baiter).position,
            Point::new(69, 120)
        );
        assert_eq!(
            snapshot_for(&retargeted, retargeted_baiter).source_baiter,
            Some(ActorSourceBaiterMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0xFFC0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: SOURCE_BAITER_LOOP_SLEEP_TICKS,
                picture_frame: 0,
            })
        );
    }

    #[test]
    fn source_baiter_retarget_adds_player_velocity() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.source_rng = ActorSourceRng {
            seed: 70,
            hseed: 0,
            lseed: 0,
        };
        let player_id = ActorId::new(90);
        let mut player = actor_snapshot(player_id.value(), ActorKind::Player, Point::new(42, 112));
        player.velocity = Velocity::new(8, 4);
        driver.snapshots.insert(player_id, player);
        let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
            position: Point::new(70, 140),
            source: Some(ActorSourceBaiterMetadata {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 2,
                sleep_ticks: 0,
                picture_frame: 2,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert_eq!(
            report.source_rng.map(|source_rng| source_rng.seed),
            Some(227)
        );
        assert_eq!(snapshot_for(&report, baiter).position, Point::new(69, 139));
        assert_eq!(
            snapshot_for(&report, baiter).source_baiter,
            Some(ActorSourceBaiterMetadata {
                x_fraction: 0x08,
                y_fraction: 0x82,
                x_velocity: 0xFFC2,
                y_velocity: 0xFF82,
                shot_timer: 1,
                sleep_ticks: SOURCE_BAITER_LOOP_SLEEP_TICKS,
                picture_frame: 0,
            })
        );
    }

    #[test]
    fn baiter_does_not_block_wave_completion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.reset_baiter_timer();
        let baiter = driver.spawn_baiter_for_test(Point::new(100, 100));

        let cleared = driver.step(GameInput::NONE);

        assert!(snapshot_for(&cleared, baiter).alive);
        assert_eq!(cleared.wave, 1);
        assert!(
            cleared
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );

        let next_wave = step_until_wave_started(&mut driver, 2);
        assert_eq!(next_wave.wave, 2);
        assert!(
            !next_wave
                .snapshots
                .iter()
                .any(|snapshot| snapshot.id == baiter)
        );
    }

    #[test]
    fn behavior_script_manifest_exports_resolution_order() {
        let default_behavior = ActorBehaviorProfile {
            player_speed: 3,
            ..ActorBehaviorProfile::default()
        };
        let lander_behavior = ActorBehaviorProfile {
            lander_drift_speed: 4,
            lander_fire_period_steps: u64::MAX,
            ..ActorBehaviorProfile::default()
        };
        let actor = ActorId::new(42);
        let actor_behavior = ActorBehaviorProfile {
            lander_drift_speed: 7,
            lander_fire_period_steps: u64::MAX,
            ..ActorBehaviorProfile::default()
        };
        let script = ActorBehaviorScript::new(default_behavior)
            .with_kind_behavior(ActorKind::Lander, lander_behavior)
            .with_actor_behavior(actor, actor_behavior);

        let manifest = script.manifest();

        assert_eq!(manifest.default_profile, default_behavior);
        assert_eq!(
            manifest.kind_profiles,
            [ActorKindBehaviorProfile {
                kind: ActorKind::Lander,
                profile: lander_behavior
            }]
        );
        assert_eq!(
            manifest.actor_profiles,
            [ActorInstanceBehaviorProfile {
                actor,
                profile: actor_behavior
            }]
        );
        assert_eq!(
            manifest.behavior_for(actor, ActorKind::Lander),
            actor_behavior
        );
        assert_eq!(
            manifest.behavior_for(ActorId::new(99), ActorKind::Lander),
            lander_behavior
        );
        assert_eq!(
            manifest.behavior_for(ActorId::new(99), ActorKind::Bomber),
            default_behavior
        );
    }

    #[test]
    fn behavior_script_text_parser_updates_default_kind_and_actor_profiles() {
        let script = ActorBehaviorScript::parse_text(
            "\
            # Movement and behavior script\n\
            default player_speed 5\n\
            default player_takes_enemy_collision_damage off\n\
            kind lander lander_mode chase_player\n\
            kind lander lander_seek_speed 6\n\
            actor 42 lander_drift_speed 7\n\
            actor 42 player_hyperspace_source_seed 0x52 0x62 0x0c\n",
        )
        .expect("behavior script text should parse");

        let manifest = script.manifest();

        assert_eq!(manifest.default_profile.player_speed, 5);
        assert!(!manifest.default_profile.player_takes_enemy_collision_damage);
        let lander = manifest
            .kind_profile(ActorKind::Lander)
            .expect("lander kind profile should be parsed");
        assert_eq!(lander.lander_mode, LanderBehaviorMode::ChasePlayer);
        assert_eq!(lander.lander_seek_speed, 6);
        let actor = manifest
            .actor_profile(ActorId::new(42))
            .expect("actor profile should be parsed");
        assert_eq!(actor.lander_drift_speed, 7);
        assert_eq!(
            actor.player_hyperspace_source_seed,
            Some(ActorHyperspaceSourceSeed {
                seed: 0x52,
                hseed: 0x62,
                lseed: 0x0C,
            })
        );
    }

    #[test]
    fn parsed_behavior_script_drives_motion_and_damage_policy() {
        let script = "\
            kind lander lander_mode chase_player\n\
            kind lander lander_seek_speed 4\n\
            kind player player_takes_enemy_collision_damage false\n"
            .parse::<ActorBehaviorScript>()
            .expect("behavior script should parse");
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.behavior_script = script;
        driver.spawn_player();
        let lander = driver.spawn_lander_for_test(Point::new(80, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));

        driver.step(GameInput::NONE);
        let chasing = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&chasing, lander).position.x, 75);

        let script = "kind player player_takes_enemy_collision_damage false"
            .parse::<ActorBehaviorScript>()
            .expect("damage behavior script should parse");
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.behavior_script = script;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));
        let protected = driver.step(GameInput::NONE);

        assert_eq!(protected.phase, Phase::Playing);
        assert_eq!(protected.lives, 3);
        assert!(
            !protected
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::PlayerKilled))
        );
    }

    #[test]
    fn behavior_script_text_parser_reports_line_errors() {
        let error = ActorBehaviorScript::parse_text("kind lander no_such_field 1\n")
            .expect_err("unknown behavior field should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown behavior field"));

        let error = ActorBehaviorScript::parse_text("kind no_such_kind lander_seek_speed 1\n")
            .expect_err("unknown actor kind should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown actor kind"));
    }

    #[test]
    fn embedded_actor_behavior_script_matches_arcade_profile_defaults() {
        let parsed = ActorBehaviorScript::parse_text(ACTOR_RED_LABEL_BEHAVIOR_SCRIPT)
            .expect("embedded actor behavior script should parse");

        assert_eq!(ActorBehaviorScript::default().manifest(), parsed.manifest());
        assert_eq!(
            ActorBehaviorScript::from_arcade_profile().manifest(),
            parsed.manifest()
        );
        assert_eq!(
            parsed.manifest().default_profile,
            ActorBehaviorProfile::arcade_default()
        );
        assert!(parsed.manifest().kind_profiles.is_empty());
        assert!(parsed.manifest().actor_profiles.is_empty());
    }

    #[test]
    fn default_driver_exposes_embedded_actor_script_manifests() {
        let driver = ActorGameDriver::new();
        let manifest = driver.script_manifest();

        assert_eq!(
            manifest.attract_script,
            AttractScript::parse_text(ACTOR_RED_LABEL_ATTRACT_SCRIPT)
                .expect("embedded attract script should parse")
                .manifest()
        );
        assert_eq!(
            manifest.behavior_script,
            ActorBehaviorScript::parse_text(ACTOR_RED_LABEL_BEHAVIOR_SCRIPT)
                .expect("embedded behavior script should parse")
                .manifest()
        );
        assert_eq!(
            manifest.wave_script,
            ActorWaveScript::parse_text(ACTOR_RED_LABEL_WAVE_SCRIPT)
                .expect("embedded wave script should parse")
                .manifest()
        );
    }

    #[test]
    fn driver_script_bundle_parses_and_installs_custom_scripts() {
        let scripts = ActorDriverScripts::parse_texts(
            "text 1 forever 12 20 CUSTOM DRIVER\n",
            "\
            default player_takes_enemy_collision_damage false\n\
            kind lander lander_mode drift\n\
            kind lander lander_drift_speed 5\n",
            "\
            name bundled custom waves\n\
            wave 1\n\
            lander 80 214\n\
            human 100 214\n",
        )
        .expect("driver script bundle should parse");
        let expected_attract = scripts.attract_script.manifest();
        let expected_wave = scripts.wave_script.manifest();
        let mut driver = ActorGameDriver::with_scripts(scripts);

        let attract = driver.step(GameInput::NONE);
        assert!(attract.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("CUSTOM DRIVER") && draw.position == Point::new(12, 20)
        }));

        let attract_manifest = driver.script_manifest();
        assert_eq!(attract_manifest.attract_script, expected_attract);
        assert_eq!(attract_manifest.wave_script, expected_wave);
        assert!(
            !attract_manifest
                .behavior_script
                .default_profile
                .player_takes_enemy_collision_damage
        );
        let attract_wave_lander = attract_manifest
            .current_wave_profile
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("wave profile should inherit bundled lander behavior");
        assert_eq!(attract_wave_lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(attract_wave_lander.lander_drift_speed, 5);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let playing = step_until_driver_player_start_completes(&mut driver, 1);
        let playing_manifest = driver.script_manifest();

        assert_eq!(playing.phase, Phase::Playing);
        assert!(
            !playing_manifest
                .behavior_script
                .default_profile
                .player_takes_enemy_collision_damage
        );
        let playing_lander = playing_manifest
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("playing behavior should come from bundled wave profile");
        assert_eq!(playing_lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(playing_lander.lander_drift_speed, 5);
    }

    #[test]
    fn sectioned_driver_script_parses_and_installs_custom_scripts() {
        let scripts = ActorDriverScripts::parse_text(
            "\
            [attract]\n\
            text 1 forever 12 20 SECTIONED DRIVER\n\
            [behavior]\n\
            default player_takes_enemy_collision_damage false\n\
            kind lander lander_mode drift\n\
            kind lander lander_drift_speed 4\n\
            [wave]\n\
            name sectioned waves\n\
            wave 1\n\
            lander 80 214\n\
            human 100 214\n",
        )
        .expect("sectioned driver script should parse");
        let expected_attract = scripts.attract_script.manifest();
        let expected_wave = scripts.wave_script.manifest();
        let mut driver = ActorGameDriver::with_scripts(scripts);

        let attract = driver.step(GameInput::NONE);
        assert!(attract.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("SECTIONED DRIVER") && draw.position == Point::new(12, 20)
        }));
        assert_eq!(driver.script_manifest().attract_script, expected_attract);
        assert_eq!(driver.script_manifest().wave_script, expected_wave);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let playing = step_until_driver_player_start_completes(&mut driver, 1);
        let manifest = driver.script_manifest();

        assert_eq!(playing.phase, Phase::Playing);
        assert!(
            !manifest
                .behavior_script
                .default_profile
                .player_takes_enemy_collision_damage
        );
        let lander = manifest
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("sectioned wave should inherit bundled behavior");
        assert_eq!(lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(lander.lander_drift_speed, 4);
    }

    #[test]
    fn sectioned_driver_script_parses_to_manifest_and_runtime_adapter() {
        let scripts = "\
            [attract]\n\
            text 1 forever 12 20 RUNTIME SCRIPT\n\
            [behavior]\n\
            default player_takes_enemy_collision_damage false\n\
            kind lander lander_mode drift\n\
            kind lander lander_drift_speed 3\n\
            [wave]\n\
            name runtime script waves\n\
            wave 1\n\
            lander 80 214\n\
            human 100 214\n"
            .parse::<ActorDriverScripts>()
            .expect("sectioned driver script should parse via FromStr");
        let manifest = scripts.manifest();

        assert_eq!(manifest.attract_script.events.len(), 1);
        assert!(
            !manifest
                .behavior_script
                .default_profile
                .player_takes_enemy_collision_damage
        );
        assert_eq!(manifest.wave_script.name, "runtime script waves");
        let wave_lander = manifest.wave_script.waves[0]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("wave should inherit bundled lander behavior");
        assert_eq!(wave_lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(wave_lander.lander_drift_speed, 3);

        let mut runtime = ActorRuntimeAdapter::with_scripts(scripts);
        let frame = runtime.step(GameInput::NONE);

        assert_eq!(
            runtime.driver().script_manifest().wave_script,
            manifest.wave_script
        );
        assert!(frame.report.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("RUNTIME SCRIPT") && draw.position == Point::new(12, 20)
        }));
        assert_eq!(frame.state.phase, GamePhase::Attract);
    }

    #[test]
    fn driver_script_bundle_reports_sectioned_parse_errors() {
        let error = ActorDriverScripts::parse_texts(
            "text 1 forever 12 20 CUSTOM DRIVER\n",
            "kind lander lander_mode drift\n",
            "\
            name broken wave\n\
            lander 80 214\n",
        )
        .expect_err("wave script should reject spawn before wave");

        assert_eq!(error.section, ActorDriverScriptSection::Wave);
        assert_eq!(error.line, 2);
        assert!(
            error
                .to_string()
                .contains("actor driver wave script line 2")
        );
        assert!(error.message.contains("wave action must appear"));
    }

    #[test]
    fn sectioned_driver_script_preserves_source_line_errors() {
        let error = ActorDriverScripts::parse_text(
            "\
            [attract]\n\
            text 1 forever 12 20 SECTIONED DRIVER\n\
            [behavior]\n\
            kind lander lander_mode drift\n\
            [wave]\n\
            name broken sectioned waves\n\
            lander 80 214\n",
        )
        .expect_err("sectioned wave script should reject spawn before wave");

        assert_eq!(error.section, ActorDriverScriptSection::Wave);
        assert_eq!(error.line, 7);
        assert!(
            error
                .to_string()
                .contains("actor driver wave script line 7")
        );
        assert!(error.message.contains("wave action must appear"));

        let error = ActorDriverScripts::parse_text(
            "\
            [attract]\n\
            text 1 forever 12 20 SECTIONED DRIVER\n\
            [driver]\n\
            noop\n",
        )
        .expect_err("unknown section should fail before parsing content");
        assert_eq!(error.section, ActorDriverScriptSection::Driver);
        assert_eq!(error.line, 3);
        assert!(
            error
                .message
                .contains("unknown driver script section `driver`")
        );
    }

    #[test]
    fn wave_script_text_parser_can_inherit_custom_base_behavior() {
        let behavior_script = ActorBehaviorScript::parse_text(
            "\
            default player_takes_enemy_collision_damage false\n\
            kind lander lander_mode drift\n\
            kind lander lander_drift_speed 6\n",
        )
        .expect("base behavior script should parse");
        let wave_script = ActorWaveScript::parse_text_with_base_behavior(
            "\
            name inherited wave behavior\n\
            source_wave 1 wave_size 5 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n\
            wave 2\n\
            lander 80 214\n",
            &behavior_script,
        )
        .expect("wave script should inherit base behavior");
        let manifest = wave_script.manifest();

        assert_eq!(manifest.waves.len(), 2);
        for profile in &manifest.waves {
            assert!(
                !profile
                    .behavior_script
                    .default_profile
                    .player_takes_enemy_collision_damage
            );
        }
        let source_lander = manifest.waves[0]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("source wave should keep source lander behavior");
        assert_eq!(
            source_lander.lander_fire_period_steps,
            ActorSourceWaveProfile::for_wave(1)
                .lander_behavior()
                .lander_fire_period_steps
        );
        let clean_lander = manifest.waves[1]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("clean wave should inherit base kind behavior");
        assert_eq!(clean_lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(clean_lander.lander_drift_speed, 6);
    }

    #[test]
    fn driver_script_manifest_exports_current_wave_and_spawns() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 6,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::ChasePlayer,
            ..ActorBehaviorProfile::default()
        };
        let wave_script = ActorWaveScript::new(
            "manifest-test",
            vec![ActorWaveProfile::with_family_spawns(
                1,
                ActorBehaviorScript::default()
                    .with_kind_behavior(ActorKind::Lander, lander_behavior),
                vec![ActorLanderSpawn::new(Point::new(80, 96))],
                vec![ActorBomberSpawn::new(Point::new(120, 80))],
                vec![ActorPodSpawn::new(Point::new(160, 88))],
                vec![ActorHumanSpawn::new(
                    Point::new(32, HUMAN_GROUND_Y),
                    HumanMode::Grounded,
                )],
            )],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        let attract_manifest = driver.script_manifest();
        assert_eq!(attract_manifest.phase, Phase::Attract);
        assert!(matches!(
            attract_manifest.attract_script.events[0].action,
            AttractScriptActionManifest::WilliamsLogo { .. }
        ));
        assert!(
            attract_manifest
                .attract_script
                .events
                .iter()
                .any(|event| matches!(
                    event.action,
                    AttractScriptActionManifest::DefenderWordmark { .. }
                ))
        );
        assert_eq!(attract_manifest.wave_script.name, "manifest-test");
        assert!(attract_manifest.wave_script.behavior_presets.is_empty());
        assert!(
            attract_manifest
                .wave_script
                .spawn_behavior_presets
                .is_empty()
        );
        assert_eq!(attract_manifest.current_wave_profile.source_wave, None);
        assert_eq!(
            attract_manifest.current_wave_profile.lander_spawns[0].position,
            Point::new(80, 96)
        );
        assert_eq!(
            attract_manifest
                .current_wave_profile
                .bomber_spawns
                .first()
                .map(|spawn| spawn.position),
            Some(Point::new(120, 80))
        );
        assert_eq!(
            attract_manifest
                .behavior_script
                .kind_profile(ActorKind::Lander),
            None
        );

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let playing_manifest = driver.script_manifest();

        assert_eq!(playing_manifest.phase, Phase::Playing);
        assert_eq!(playing_manifest.wave, 1);
        assert_eq!(
            playing_manifest
                .behavior_script
                .kind_profile(ActorKind::Lander),
            Some(lander_behavior)
        );
        assert_eq!(
            playing_manifest
                .current_wave_profile
                .behavior_script
                .kind_profile(ActorKind::Lander),
            Some(lander_behavior)
        );
    }

    #[test]
    fn step_report_manifest_carries_effective_xyzzy_behavior_override() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();

        let report = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                invincible: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });

        assert!(
            !report
                .behavior_script
                .behavior_for(player, ActorKind::Player)
                .player_takes_enemy_collision_damage
        );
        assert!(
            driver
                .script_manifest()
                .behavior_script
                .behavior_for(player, ActorKind::Player)
                .player_takes_enemy_collision_damage
        );
    }

    #[test]
    fn wave_script_applies_behavior_when_play_starts() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 5,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::ChasePlayer,
            ..ActorBehaviorProfile::default()
        };
        let wave_script = ActorWaveScript::single_wave(
            "opening-chasers",
            ActorBehaviorScript::default().with_kind_behavior(ActorKind::Lander, lander_behavior),
            vec![Point::new(80, HUMAN_GROUND_Y)],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.wave, 1);
        assert_eq!(driver.wave(), 1);
        assert_eq!(driver.wave_script_name(), "opening-chasers");

        step_until_driver_player_start_completes(&mut driver, 1);
        let chasing = driver.step(GameInput::NONE);
        let lander = chasing
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Lander)
            .expect("wave script should spawn a lander");
        assert_eq!(lander.position, Point::new(74, HUMAN_GROUND_Y - 5));
    }

    #[test]
    fn wave_script_can_configure_initial_human_spawns() {
        let wave_script = ActorWaveScript::new(
            "single-human-opening",
            vec![ActorWaveProfile::with_spawns(
                1,
                ActorBehaviorScript::default(),
                vec![ActorLanderSpawn::new(Point::new(220, 80))],
                vec![ActorHumanSpawn::new(
                    Point::new(32, HUMAN_GROUND_Y),
                    HumanMode::Grounded,
                )],
            )],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let settled = step_until_driver_player_start_completes(&mut driver, 1);

        assert_eq!(driver.snapshot_count(ActorKind::Human), 1);
        assert!(settled.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Human
                && snapshot.position == Point::new(32, HUMAN_GROUND_Y)
                && snapshot.source_human.is_none()
        }));
    }

    #[test]
    fn wave_script_text_parser_builds_sorted_profiles_and_spawns() {
        let wave_script = ActorWaveScript::parse_text(
            "\
            name parsed progression\n\
            wave 2\n\
            behavior kind lander lander_mode drift\n\
            behavior kind lander lander_drift_speed 5\n\
            lander 100 100\n\
            bomber 120 80\n\
            pod 160 88\n\
            mutant 140 90\n\
            swarmer 150 96\n\
            baiter 170 104\n\
            reserve_full 2 1 1 3 4\n\
            human 32 214 grounded\n\
            wave 1\n\
            behavior kind lander lander_mode chase_player\n\
            behavior kind lander lander_seek_speed 6\n\
            enemy_reserve 3 0 0 2\n\
            spawn_behavior lander 0 lander_seek_speed 8\n\
            lander 80 96\n\
            human 40 214 falling -1\n",
        )
        .expect("wave script text should parse");

        let manifest = wave_script.manifest();

        assert_eq!(manifest.name, "parsed progression");
        assert_eq!(
            manifest
                .waves
                .iter()
                .map(|profile| profile.wave)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            manifest.waves[0].lander_spawns[0].position,
            Point::new(80, 96)
        );
        assert_eq!(
            manifest.waves[0].human_spawns[0].mode,
            HumanMode::Falling { velocity: -1 }
        );
        assert_eq!(
            manifest.waves[0].enemy_reserve,
            EnemyReserveSnapshot {
                landers: 3,
                swarmers: 2,
                ..EnemyReserveSnapshot::default()
            }
        );
        let wave_one_lander = manifest.waves[0]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("wave 1 lander profile should parse");
        assert_eq!(wave_one_lander.lander_mode, LanderBehaviorMode::ChasePlayer);
        assert_eq!(wave_one_lander.lander_seek_speed, 6);
        assert_eq!(manifest.waves[0].spawn_behavior_profiles.len(), 1);
        assert_eq!(
            manifest.waves[0].spawn_behavior_profiles[0].kind,
            ActorKind::Lander
        );
        assert_eq!(manifest.waves[0].spawn_behavior_profiles[0].spawn_index, 0);
        assert_eq!(
            manifest.waves[0].spawn_behavior_profiles[0]
                .profile
                .lander_seek_speed,
            8
        );

        assert_eq!(
            manifest.waves[1].bomber_spawns[0].position,
            Point::new(120, 80)
        );
        assert_eq!(
            manifest.waves[1].pod_spawns[0].position,
            Point::new(160, 88)
        );
        assert_eq!(
            manifest.waves[1].mutant_spawns[0].position,
            Point::new(140, 90)
        );
        assert_eq!(
            manifest.waves[1].swarmer_spawns[0].position,
            Point::new(150, 96)
        );
        assert_eq!(
            manifest.waves[1].baiter_spawns[0].position,
            Point::new(170, 104)
        );
        assert_eq!(
            manifest.waves[1].enemy_reserve,
            EnemyReserveSnapshot {
                landers: 2,
                bombers: 1,
                pods: 1,
                mutants: 3,
                swarmers: 4,
            }
        );
        let wave_two_lander = manifest.waves[1]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("wave 2 lander profile should parse");
        assert_eq!(wave_two_lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(wave_two_lander.lander_drift_speed, 5);
    }

    #[test]
    fn parsed_wave_script_drives_wave_spawns_and_next_wave_behavior() {
        let wave_script = "\
            name parsed waves\n\
            wave 1\n\
            behavior kind lander lander_mode chase_player\n\
            behavior kind lander lander_seek_speed 5\n\
            lander 80 214\n\
            human 100 214\n\
            wave 2\n\
            behavior kind lander lander_mode drift\n\
            behavior kind lander lander_drift_speed 5\n\
            lander 100 100\n"
            .parse::<ActorWaveScript>()
            .expect("wave script should parse");
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        step_until_driver_player_start_completes(&mut driver, 1);
        let chasing = driver.step(GameInput::NONE);
        let lander = chasing
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Lander)
            .expect("wave 1 should spawn a lander");
        assert_eq!(lander.position, Point::new(74, 209));

        let pressed = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });
        assert!(
            !pressed
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );
        let cleared = step_until_driver_smart_bomb_detonates(&mut driver);
        assert_eq!(cleared.wave, 1);
        assert!(
            cleared
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );

        let next_wave = step_until_wave_started(&mut driver, 2);
        assert_eq!(next_wave.wave, 2);
        assert!(
            next_wave
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 2 })
        );
        let lander = next_wave
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Lander)
            .expect("wave 2 should spawn a lander");
        assert_eq!(lander.position, Point::new(95, 100));
    }

    #[test]
    fn parsed_wave_script_drives_custom_hostile_family_spawns() {
        let wave_script = "\
            name custom hostiles\n\
            wave 1\n\
            behavior kind mutant mutant_mode drift\n\
            behavior kind swarmer swarmer_mode drift\n\
            behavior kind swarmer swarmer_seek_speed 1\n\
            behavior kind baiter baiter_mode drift\n\
            behavior kind baiter baiter_seek_speed 1\n\
            mutant 80 72\n\
            swarmer 120 84\n\
            baiter 160 96\n\
            human 40 214\n"
            .parse::<ActorWaveScript>()
            .expect("custom hostile wave script should parse");
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let report = step_until_driver_player_start_completes(&mut driver, 1);

        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Mutant), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Swarmer), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Baiter), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Human), 1);
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Mutant && snapshot.position == Point::new(79, 72)
        }));
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Swarmer && snapshot.position == Point::new(119, 84)
        }));
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Baiter && snapshot.position == Point::new(159, 96)
        }));
    }

    #[test]
    fn parsed_source_wave_overrides_drive_source_shaped_custom_wave() {
        let wave_script = concat!(
            "name custom source shape\n",
            "source_wave 1 wave_size 5 landers 1 bombers 1 pods 1 mutants 1 swarmers 1 ",
            "swarmer_x_velocity 64 swarmer_shot_time 11 baiter_time 24 ",
            "mutant_x_velocity 48 mutant_random_y 2 mutant_shot_time 12\n",
        )
        .parse::<ActorWaveScript>()
        .expect("source wave overrides should parse");
        let manifest = wave_script.manifest();
        let profile = &manifest.waves[0];
        let source = profile
            .source_wave
            .expect("source_wave override should preserve source metadata");

        assert_eq!(source.wave_size, 5);
        assert_eq!(source.landers, 1);
        assert_eq!(source.bombers, 1);
        assert_eq!(source.pods, 1);
        assert_eq!(source.mutants, 1);
        assert_eq!(source.swarmers, 1);
        assert_eq!(source.swarmer_x_velocity, 64);
        assert_eq!(source.swarmer_shot_time, 11);
        assert_eq!(source.baiter_delay, 24);
        assert_eq!(source.mutant_x_velocity, 48);
        assert_eq!(source.mutant_random_y, 2);
        assert_eq!(source.mutant_shot_time, 12);
        assert_eq!(profile.lander_spawns.len(), 1);
        assert_eq!(profile.bomber_spawns.len(), 1);
        assert_eq!(profile.pod_spawns.len(), 1);
        assert_eq!(profile.mutant_spawns.len(), 1);
        assert_eq!(profile.swarmer_spawns.len(), 1);
        assert_eq!(profile.enemy_reserve, EnemyReserveSnapshot::default());
        assert!(profile.mutant_spawns[0].source.is_some());
        assert!(profile.swarmer_spawns[0].source.is_some());

        let mut driver = ActorGameDriver::with_wave_script(wave_script);
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let report = step_until_driver_player_start_completes(&mut driver, 1);

        assert_eq!(driver.snapshot_count(ActorKind::Lander), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Bomber), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Mutant), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Swarmer), 1);
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Mutant && snapshot.source_mutant.is_some()
        }));
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Swarmer && snapshot.source_swarmer.is_some()
        }));
        assert_eq!(report.source_wave.wave_size, 5);
        assert_eq!(report.source_wave.mutant_x_velocity, 48);
        assert_eq!(report.source_wave.swarmer_shot_time, 11);
        let state_profile = report.game_state().wave_profile;
        assert_eq!(state_profile.wave_size, 5);
        assert_eq!(state_profile.landers, 1);
        assert_eq!(state_profile.bombers, 1);
        assert_eq!(state_profile.pods, 1);
        assert_eq!(state_profile.mutants, 1);
        assert_eq!(state_profile.swarmers, 1);
        assert_eq!(state_profile.swarmer_x_velocity, 64);
        assert_eq!(state_profile.swarmer_shot_time, 11);
        assert_eq!(state_profile.baiter_delay, 24);
        assert_eq!(state_profile.mutant_x_velocity, 48);
        assert_eq!(state_profile.mutant_random_y, 2);
        assert_eq!(state_profile.mutant_shot_time, 12);
        assert_eq!(
            state_profile.wave_time,
            WaveProfileSnapshot::for_wave(1).wave_time
        );
        assert_eq!(
            driver
                .script_manifest()
                .current_wave_profile
                .source_wave
                .expect("current wave manifest should expose source override")
                .mutants,
            1
        );
    }

    #[test]
    fn parsed_source_wave_range_overrides_apply_to_each_expanded_profile() {
        let wave_script = concat!(
            "name ranged source shape\n",
            "source_waves 1 2 wave_size 5 landers 1 bombers 1 pods 1 mutants 1 swarmers 1 ",
            "swarmer_x_velocity 64 swarmer_shot_time 11 baiter_time 24 ",
            "mutant_x_velocity 48 mutant_random_y 2 mutant_shot_time 12\n",
        )
        .parse::<ActorWaveScript>()
        .expect("source wave range overrides should parse");
        let manifest = wave_script.manifest();

        assert_eq!(
            manifest
                .waves
                .iter()
                .map(|profile| profile.wave)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        for profile in &manifest.waves {
            let source = profile
                .source_wave
                .expect("range override should preserve source metadata");
            assert_eq!(source.wave_size, 5);
            assert_eq!(source.landers, 1);
            assert_eq!(source.bombers, 1);
            assert_eq!(source.pods, 1);
            assert_eq!(source.mutants, 1);
            assert_eq!(source.swarmers, 1);
            assert_eq!(source.swarmer_x_velocity, 64);
            assert_eq!(source.swarmer_shot_time, 11);
            assert_eq!(source.baiter_delay, 24);
            assert_eq!(source.mutant_x_velocity, 48);
            assert_eq!(source.mutant_random_y, 2);
            assert_eq!(source.mutant_shot_time, 12);
            assert_eq!(profile.lander_spawns.len(), 1);
            assert_eq!(profile.bomber_spawns.len(), 1);
            assert_eq!(profile.pod_spawns.len(), 1);
            assert_eq!(profile.mutant_spawns.len(), 1);
            assert_eq!(profile.swarmer_spawns.len(), 1);
            assert_eq!(profile.enemy_reserve, EnemyReserveSnapshot::default());
            assert!(profile.mutant_spawns[0].source.is_some());
            assert!(profile.swarmer_spawns[0].source.is_some());
        }

        let second = wave_script.profile_for_wave(2);
        assert_eq!(
            second
                .source_wave
                .expect("wave 2 should use the effective range override")
                .mutants,
            1
        );
        assert_eq!(second.mutant_spawns.len(), 1);
        assert_eq!(second.swarmer_spawns.len(), 1);
    }

    #[test]
    fn parsed_wave_script_applies_behavior_ranges_to_existing_profiles() {
        let wave_script = concat!(
            "name ranged behavior\n",
            "source_waves 1 2 wave_size 5 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n",
            "behavior_waves 1 2 kind lander lander_mode chase_player\n",
            "behavior_waves 1 2 kind lander lander_seek_speed 7\n",
            "spawn_behavior_waves 1 2 lander 0 lander_seek_speed 9\n",
        )
        .parse::<ActorWaveScript>()
        .expect("range behavior script should parse");
        let manifest = wave_script.manifest();

        assert_eq!(
            manifest
                .waves
                .iter()
                .map(|profile| profile.wave)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        for profile in &manifest.waves {
            let lander_behavior = profile
                .behavior_script
                .kind_profile(ActorKind::Lander)
                .expect("range behavior should install lander kind profile");
            assert_eq!(lander_behavior.lander_mode, LanderBehaviorMode::ChasePlayer);
            assert_eq!(lander_behavior.lander_seek_speed, 7);
            assert_eq!(profile.spawn_behavior_profiles.len(), 1);
            assert_eq!(profile.spawn_behavior_profiles[0].kind, ActorKind::Lander);
            assert_eq!(profile.spawn_behavior_profiles[0].spawn_index, 0);
            assert_eq!(
                profile.spawn_behavior_profiles[0].profile.lander_seek_speed,
                9
            );
        }
    }

    #[test]
    fn parsed_wave_script_applies_named_behavior_presets_to_current_and_range_profiles() {
        let wave_script = concat!(
            "name preset behavior\n",
            "behavior_preset hard_lander kind lander lander_mode chase_player\n",
            "behavior_preset hard_lander kind lander lander_seek_speed 7\n",
            "source_waves 1 2 wave_size 5 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n",
            "use_behavior_waves 1 2 hard_lander\n",
            "wave 3\n",
            "use_behavior hard_lander\n",
            "lander 80 214\n",
        )
        .parse::<ActorWaveScript>()
        .expect("behavior preset wave script should parse");
        let manifest = wave_script.manifest();

        assert_eq!(
            manifest
                .waves
                .iter()
                .map(|profile| profile.wave)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        for profile in manifest.waves.iter().take(2) {
            let source_lander = ActorSourceWaveProfile::for_wave(profile.wave).lander_behavior();
            let lander_behavior = profile
                .behavior_script
                .kind_profile(ActorKind::Lander)
                .expect("range preset should install lander kind profile");
            assert_eq!(lander_behavior.lander_mode, LanderBehaviorMode::ChasePlayer);
            assert_eq!(lander_behavior.lander_seek_speed, 7);
            assert_eq!(
                lander_behavior.lander_fire_period_steps,
                source_lander.lander_fire_period_steps
            );
        }
        let clean_wave_lander = manifest.waves[2]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("current-wave preset should install lander kind profile");
        assert_eq!(
            clean_wave_lander.lander_mode,
            LanderBehaviorMode::ChasePlayer
        );
        assert_eq!(clean_wave_lander.lander_seek_speed, 7);
        assert_eq!(manifest.waves[2].source_wave, None);
    }

    #[test]
    fn parsed_wave_script_manifest_exposes_reusable_behavior_presets() {
        let wave_script = concat!(
            "name preset manifest\n",
            "behavior_preset Hard-Lander kind lander lander_mode chase_player\n",
            "behavior_preset Hard-Lander kind lander lander_seek_speed 7\n",
            "spawn_behavior_preset Fast-Slot lander_mode chase_player\n",
            "spawn_behavior_preset Fast-Slot lander_seek_speed 9\n",
            "source_wave 1 wave_size 5 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n",
            "use_behavior hard_lander\n",
            "use_spawn_behavior lander 0 fast_slot\n",
        )
        .parse::<ActorWaveScript>()
        .expect("preset manifest wave script should parse");
        let manifest = wave_script.manifest();

        assert_eq!(
            manifest.behavior_presets,
            [ActorWaveBehaviorPresetManifest {
                name: "hard_lander".to_string(),
                updates: vec![
                    "kind lander lander_mode chase_player".to_string(),
                    "kind lander lander_seek_speed 7".to_string(),
                ],
            }]
        );
        assert_eq!(
            manifest.spawn_behavior_presets,
            [ActorWaveSpawnBehaviorPresetManifest {
                name: "fast_slot".to_string(),
                updates: vec![
                    ActorWaveSpawnBehaviorPresetUpdateManifest {
                        field: "lander_mode".to_string(),
                        values: vec!["chase_player".to_string()],
                    },
                    ActorWaveSpawnBehaviorPresetUpdateManifest {
                        field: "lander_seek_speed".to_string(),
                        values: vec!["9".to_string()],
                    },
                ],
            }]
        );
        let lander_behavior = manifest.waves[0]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("behavior preset should still apply to wave profile");
        assert_eq!(lander_behavior.lander_seek_speed, 7);
        assert_eq!(manifest.waves[0].spawn_behavior_profiles.len(), 1);
        assert_eq!(
            manifest.waves[0].spawn_behavior_profiles[0]
                .profile
                .lander_seek_speed,
            9
        );
    }

    #[test]
    fn parsed_wave_script_reports_missing_behavior_range_profiles() {
        let error = "\
            name missing behavior range\n\
            source_wave 1\n\
            behavior_waves 1 2 kind lander lander_seek_speed 7\n"
            .parse::<ActorWaveScript>()
            .expect_err("range behavior should require existing profiles");

        assert_eq!(error.line, 3);
        assert_eq!(error.message, "wave range references undefined wave `2`");
    }

    #[test]
    fn parsed_wave_script_reports_unknown_behavior_presets() {
        let error = "\
            name missing preset\n\
            source_wave 1\n\
            use_behavior missing\n"
            .parse::<ActorWaveScript>()
            .expect_err("preset use should require a definition");

        assert_eq!(error.line, 3);
        assert_eq!(error.message, "unknown behavior preset `missing`");
    }

    #[test]
    fn parsed_wave_script_applies_spawn_behavior_presets_to_current_and_range_profiles() {
        let wave_script = concat!(
            "name spawn preset behavior\n",
            "spawn_behavior_preset fast_slot lander_mode chase_player\n",
            "spawn_behavior_preset fast_slot lander_seek_speed 9\n",
            "source_waves 1 2 wave_size 5 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n",
            "use_spawn_behavior_waves 1 2 lander 0 fast_slot\n",
            "wave 3\n",
            "behavior kind lander lander_mode drift\n",
            "behavior kind lander lander_drift_speed 4\n",
            "use_spawn_behavior lander 1 fast_slot\n",
            "lander 80 214\n",
            "lander 120 214\n",
        )
        .parse::<ActorWaveScript>()
        .expect("spawn behavior preset wave script should parse");
        let manifest = wave_script.manifest();

        assert_eq!(
            manifest
                .waves
                .iter()
                .map(|profile| profile.wave)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        for profile in manifest.waves.iter().take(2) {
            let source_lander = ActorSourceWaveProfile::for_wave(profile.wave).lander_behavior();
            assert_eq!(profile.spawn_behavior_profiles.len(), 1);
            assert_eq!(profile.spawn_behavior_profiles[0].kind, ActorKind::Lander);
            assert_eq!(profile.spawn_behavior_profiles[0].spawn_index, 0);
            let spawn_profile = profile.spawn_behavior_profiles[0].profile;
            assert_eq!(spawn_profile.lander_mode, LanderBehaviorMode::ChasePlayer);
            assert_eq!(spawn_profile.lander_seek_speed, 9);
            assert_eq!(
                spawn_profile.lander_fire_period_steps,
                source_lander.lander_fire_period_steps
            );
        }

        let clean_spawn = manifest.waves[2].spawn_behavior_profiles[0];
        assert_eq!(clean_spawn.kind, ActorKind::Lander);
        assert_eq!(clean_spawn.spawn_index, 1);
        assert_eq!(
            clean_spawn.profile.lander_mode,
            LanderBehaviorMode::ChasePlayer
        );
        assert_eq!(clean_spawn.profile.lander_seek_speed, 9);
        assert_eq!(clean_spawn.profile.lander_drift_speed, 4);
    }

    #[test]
    fn parsed_wave_script_applies_spawn_behavior_preset_after_actor_allocation() {
        let wave_script = "\
            name spawn preset allocation\n\
            spawn_behavior_preset fast_slot lander_mode chase_player\n\
            spawn_behavior_preset fast_slot lander_seek_speed 5\n\
            wave 1\n\
            behavior kind lander lander_mode drift\n\
            behavior kind lander lander_drift_speed 1\n\
            use_spawn_behavior lander 1 fast_slot\n\
            lander 80 214\n\
            lander 120 214\n\
            human 100 214\n"
            .parse::<ActorWaveScript>()
            .expect("spawn behavior preset script should parse");
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let report = step_until_driver_player_start_completes(&mut driver, 1);
        let landers = report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .collect::<Vec<_>>();

        assert_eq!(landers.len(), 2);
        assert!(
            driver
                .script_manifest()
                .behavior_script
                .actor_profile(landers[0].id)
                .is_none()
        );
        let second_behavior = driver
            .script_manifest()
            .behavior_script
            .actor_profile(landers[1].id)
            .expect("preset spawn index should receive actor-id behavior");
        assert_eq!(second_behavior.lander_mode, LanderBehaviorMode::ChasePlayer);
        assert_eq!(second_behavior.lander_seek_speed, 5);
        assert_eq!(landers[0].position, Point::new(79, 214));
        assert!(landers[1].position.x < 120);
    }

    #[test]
    fn parsed_wave_script_reports_unknown_spawn_behavior_presets() {
        let error = "\
            name missing spawn preset\n\
            source_wave 1\n\
            use_spawn_behavior lander 0 missing\n"
            .parse::<ActorWaveScript>()
            .expect_err("spawn preset use should require a definition");

        assert_eq!(error.line, 3);
        assert_eq!(error.message, "unknown spawn behavior preset `missing`");
    }

    #[test]
    fn parsed_wave_script_applies_spawn_index_behavior_after_actor_allocation() {
        let wave_script = "\
            name spawn behavior\n\
            wave 1\n\
            behavior kind lander lander_mode drift\n\
            behavior kind lander lander_drift_speed 1\n\
            lander 80 214\n\
            lander 120 214\n\
            human 100 214\n\
            spawn_behavior lander 0 lander_mode chase_player\n\
            spawn_behavior lander 0 lander_seek_speed 5\n"
            .parse::<ActorWaveScript>()
            .expect("spawn behavior script should parse");
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let report = step_until_driver_player_start_completes(&mut driver, 1);
        let landers = report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .collect::<Vec<_>>();

        assert_eq!(landers.len(), 2);
        let first_behavior = driver
            .script_manifest()
            .behavior_script
            .actor_profile(landers[0].id)
            .expect("first spawn should receive actor-id behavior");
        assert_eq!(first_behavior.lander_mode, LanderBehaviorMode::ChasePlayer);
        assert_eq!(first_behavior.lander_seek_speed, 5);
        assert!(
            driver
                .script_manifest()
                .behavior_script
                .actor_profile(landers[1].id)
                .is_none()
        );
        assert!(landers[0].position.x < 80);
        assert_eq!(landers[1].position, Point::new(119, 214));
    }

    #[test]
    fn parsed_wave_script_applies_spawn_index_behavior_to_reserve_allocations() {
        let wave_script = "\
            name reserve spawn behavior\n\
            source_wave 2\n\
            spawn_behavior lander 3 lander_mode chase_player\n\
            spawn_behavior lander 3 lander_seek_speed 5\n"
            .parse::<ActorWaveScript>()
            .expect("reserve spawn behavior script should parse");
        let mut driver = ActorGameDriver::with_wave_script(wave_script);
        driver.phase = Phase::Playing;
        driver.wave = 2;
        driver.source_rng = SOURCE_PLAYFIELD_START_RNG;
        driver.apply_wave_profile();
        driver.spawn_player();
        driver.spawn_wave_hostiles();
        driver.spawn_initial_humans();
        driver.arm_first_wave_early_lander_reserve_delay();

        let initial = driver.step(GameInput::NONE);
        let mut initial_landers = initial
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .collect::<Vec<_>>();
        initial_landers.sort_by_key(|snapshot| snapshot.id);
        assert_eq!(initial_landers.len(), 3);
        assert!(
            driver
                .script_manifest()
                .behavior_script
                .actor_profile(initial_landers[0].id)
                .is_none()
        );
        assert_eq!(
            initial.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 17,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );

        destroy_source_counted_hostiles(&mut driver, &initial);
        let restored = driver.step(GameInput::NONE);
        let mut reserve_landers = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .collect::<Vec<_>>();
        reserve_landers.sort_by_key(|snapshot| snapshot.id);
        let reserve_lander = reserve_landers
            .first()
            .expect("reserve should spawn replacement landers");

        assert!(
            restored
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::Spawn(SpawnRequest::Lander { .. })))
        );
        assert_eq!(reserve_landers.len(), SOURCE_MAX_ACTIVE_WAVE_ENEMIES);
        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 12,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );
        let reserve_behavior = driver
            .script_manifest()
            .behavior_script
            .actor_profile(reserve_lander.id)
            .expect("reserve spawn index should receive actor-id behavior");
        assert_eq!(
            reserve_behavior.lander_mode,
            LanderBehaviorMode::ChasePlayer
        );
        assert_eq!(reserve_behavior.lander_seek_speed, 5);
    }

    #[test]
    fn wave_script_text_parser_reports_line_errors() {
        let error = ActorWaveScript::parse_text("lander 80 96\n")
            .expect_err("spawn before wave should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("wave action must appear"));

        let error = ActorWaveScript::parse_text("wave 1\nwave 1\n")
            .expect_err("duplicate wave should fail");
        assert_eq!(error.line, 2);
        assert!(error.to_string().contains("duplicate wave"));

        let error = ActorWaveScript::parse_text("wave 1\nbehavior kind lander no_such_field 1\n")
            .expect_err("bad behavior update should fail");
        assert_eq!(error.line, 2);
        assert!(error.to_string().contains("unknown behavior field"));

        let error =
            ActorWaveScript::parse_text("behavior_preset hard kind lander no_such_field 1\n")
                .expect_err("bad behavior preset update should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown behavior field"));

        let error = ActorWaveScript::parse_text("spawn_behavior_preset hard no_such_field 1\n")
            .expect_err("bad spawn behavior preset update should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown behavior field"));
    }

    #[test]
    fn cleared_wave_advances_to_next_behavior_script() {
        let wave_two_lander = ActorBehaviorProfile {
            lander_drift_speed: 5,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::Drift,
            ..ActorBehaviorProfile::default()
        };
        let wave_script = ActorWaveScript::new(
            "two-wave-test",
            vec![
                ActorWaveProfile::new(1, ActorBehaviorScript::default(), vec![Point::new(180, 88)]),
                ActorWaveProfile::new(
                    2,
                    ActorBehaviorScript::default()
                        .with_kind_behavior(ActorKind::Lander, wave_two_lander),
                    vec![Point::new(100, 100)],
                ),
            ],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        step_until_driver_player_start_completes(&mut driver, 1);

        let pressed = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });
        assert!(
            !pressed
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );
        let cleared = step_until_driver_smart_bomb_detonates(&mut driver);
        assert_eq!(cleared.wave, 1);
        assert!(
            cleared
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );

        let next_wave = step_until_wave_started(&mut driver, 2);
        assert_eq!(next_wave.wave, 2);
        assert!(
            next_wave
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 2 })
        );
        let lander = next_wave
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Lander)
            .expect("next wave should spawn a scripted lander");
        assert_eq!(lander.position, Point::new(95, 100));
    }

    #[test]
    fn behavior_script_can_choose_lander_targeting_mode() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 4,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::ChasePlayer,
            ..ActorBehaviorProfile::default()
        };
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(ActorKind::Lander, lander_behavior);
        driver.spawn_player();
        let lander_id = driver.spawn_lander_for_test(Point::new(80, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);

        let chasing = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&chasing, lander_id).position.x, 75);
    }

    #[test]
    fn actor_specific_behavior_override_changes_one_actor() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let fast_lander = driver.spawn_lander_for_test(Point::new(80, 100));
        let normal_lander = driver.spawn_lander_for_test(Point::new(100, 100));
        let fast_behavior = ActorBehaviorProfile {
            lander_drift_speed: 4,
            lander_fire_period_steps: u64::MAX,
            ..ActorBehaviorProfile::default()
        };
        driver.set_actor_behavior(fast_lander, fast_behavior);

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, fast_lander).position.x, 76);
        assert_eq!(snapshot_for(&report, normal_lander).position.x, 99);
    }

    #[test]
    fn script_can_tune_player_and_laser_behavior() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        let player_behavior = ActorBehaviorProfile {
            player_speed: 5,
            player_laser_cooldown_steps: 1,
            ..ActorBehaviorProfile::default()
        };
        driver.set_actor_behavior(player, player_behavior);

        let moved = driver.step(GameInput {
            thrust: true,
            fire: true,
            ..GameInput::NONE
        });
        assert_eq!(snapshot_for(&moved, player).position.x, 47);

        let laser_behavior = ActorBehaviorProfile {
            laser_speed: 2,
            laser_lifetime_steps: 4,
            ..ActorBehaviorProfile::default()
        };
        driver.set_kind_behavior(ActorKind::Laser, laser_behavior);

        let laser_step = driver.step(GameInput::NONE);
        let laser = laser_step
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Laser)
            .expect("configured laser should stay alive");
        assert_eq!(laser.position.x, 61);
    }

    #[test]
    fn scripted_player_damage_override_matches_xyzzy_invincibility() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));
        let player_behavior = ActorBehaviorProfile {
            player_takes_enemy_collision_damage: false,
            ..ActorBehaviorProfile::default()
        };
        driver.set_kind_behavior(ActorKind::Player, player_behavior);

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::Playing);
        assert_ne!(report.lives, 0);
    }

    #[test]
    fn lander_picks_up_and_carries_a_grounded_human() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_lander_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);

        let pickup = driver.step(GameInput::NONE);
        assert!(pickup.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::AttachHuman {
                    lander: _,
                    human: _,
                    position: Point {
                        x: 100,
                        y: HUMAN_GROUND_Y
                    },
                }
            )
        }));
        assert!(pickup.sounds.contains(&SoundCue::LanderPickup));

        let carried = driver.step(GameInput::NONE);
        assert!(carried.sounds.contains(&SoundCue::HumanPulled));
        assert!(
            carried
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::HumanCarried)
        );
    }

    #[test]
    fn carried_human_falls_when_carrier_disappears() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_carried_human_for_test(Point::new(100, 120), ActorId::new(99));

        let released = driver.step(GameInput::NONE);

        assert!(released.sounds.contains(&SoundCue::HumanReleased));
        assert_eq!(
            released.sound_events(&mut ActorSoundEventBridge::new()),
            [SoundEvent::UnmappedSoundCommand {
                command: SOURCE_ASCSND_SOUND_COMMAND,
            }]
        );
        assert!(
            released
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::HumanFalling)
        );
    }

    #[test]
    fn falling_human_rescue_awards_500_points_and_score_popup() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_falling_human_for_test(Point::new(42, 120), 0);
        driver.step(GameInput::NONE);

        let rescued = driver.step(GameInput::NONE);

        assert_eq!(rescued.score, HUMAN_RESCUE_SCORE);
        assert!(rescued.sounds.contains(&SoundCue::HumanRescued));
        assert!(
            rescued
                .commands
                .contains(&GameCommand::AddScore(HUMAN_RESCUE_SCORE))
        );
        assert!(rescued.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::ScorePopup {
                    points: HUMAN_RESCUE_SCORE,
                    ..
                })
            )
        }));
    }

    #[test]
    fn falling_human_rescue_queues_source_acsnd_tail() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_falling_human_for_test(Point::new(42, 120), 0);
        driver.spawn_human_for_test(Point::new(200, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);

        let rescued = driver.step(GameInput::NONE);
        assert_eq!(rescued.sounds, [SoundCue::HumanRescued]);

        let mut observed_tail = Vec::new();
        for offset in 1..=20u8 {
            let report = driver.step(GameInput::NONE);
            if !report.sounds.is_empty() {
                observed_tail.push((offset, report.sounds.clone()));
            }
        }

        assert_eq!(
            observed_tail,
            vec![
                (
                    10,
                    vec![SoundCue::SourceCommand(SOURCE_ACSND_SOUND_COMMAND)]
                ),
                (
                    20,
                    vec![SoundCue::SourceCommand(SOURCE_ACSND_SOUND_COMMAND)]
                ),
            ]
        );
    }

    #[test]
    fn slow_falling_human_lands_safely_for_250_points() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_falling_human_for_test(Point::new(100, HUMAN_GROUND_Y - 1), 1);

        let landed = driver.step(GameInput::NONE);

        assert_eq!(landed.score, HUMAN_SAFE_LANDING_SCORE);
        assert!(landed.sounds.contains(&SoundCue::HumanSafeLanding));
        assert!(
            landed
                .commands
                .contains(&GameCommand::AddScore(HUMAN_SAFE_LANDING_SCORE))
        );
        assert!(
            landed
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::Human)
        );
    }

    #[test]
    fn actor_playing_state_and_render_bridge_project_source_terrain_until_blow() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.spawn_player();

        let report = driver.step(GameInput::NONE);
        let state = report.game_state();
        let scene = report.render_scene();

        assert_eq!(state.world.terrain, source_playfield_terrain_segments());
        assert!(state.world.terrain_blow.is_none());
        assert!(state.world.scanner.enabled);
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TERRAIN_TILE && sprite.layer == RenderLayer::Terrain
        }));
    }

    #[test]
    fn last_human_loss_starts_actor_source_terrain_blow() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.spawn_pod_for_test(Point::new(180, 80));
        driver.spawn_falling_human_for_test(Point::new(100, HUMAN_GROUND_Y - 1), 4);

        let report = driver.step(GameInput::NONE);

        assert!(driver.snapshot_count(ActorKind::Human) == 0);
        assert!(report.sounds.is_empty());
        let terrain_blow = report.terrain_blow.expect("terrain blow should start");
        assert!(terrain_blow.status_terrain_blown);
        assert_eq!(terrain_blow.stage, TerrainBlowStage::ExplosionPassSleeping);
        assert_eq!(terrain_blow.source_elapsed_frames, 0);
        assert_eq!(terrain_blow.source_iteration, 0);
        assert_eq!(terrain_blow.source_sleep_remaining, Some(1));
        assert_eq!(
            terrain_blow.source_overload_counter,
            SOURCE_TERRAIN_BLOW_OVERLOAD_COUNTER
        );
        assert!(terrain_blow.terrain_erased());
        assert!(terrain_blow.scanner_terrain_erased());

        let state = report.game_state();
        assert!(state.world.terrain.is_empty());
        assert_eq!(state.world.terrain_blow, Some(terrain_blow));
        assert!(!state.world.scanner.enabled);
        assert!(state.world.explosions.iter().any(|explosion| {
            explosion.kind == CleanExplosionKind::Terrain
                && explosion.position == ScreenPosition::new(0x4C, 0xC2)
                && explosion.picture_label == "TEREX"
                && explosion.mapped_sprite == SpriteId::TERRAIN_EXPLOSION
        }));

        let scene = report.render_scene();
        assert!(!scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TERRAIN_TILE && sprite.layer == RenderLayer::Terrain
        }));
    }

    #[test]
    fn actor_source_terrain_blow_advances_flash_explosions_and_sound_tail() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.spawn_pod_for_test(Point::new(180, 80));
        driver.spawn_falling_human_for_test(Point::new(100, HUMAN_GROUND_Y - 1), 4);
        let start = driver.step(GameInput::NONE);
        assert!(start.terrain_blow.is_some());

        let mut observed_sounds = Vec::new();
        let mut saw_completion = false;
        for offset in 1..=SOURCE_TERRAIN_BLOW_COMPLETE_FRAME + 26 {
            let report = driver.step(GameInput::NONE);
            if !report.sounds.is_empty() {
                observed_sounds.push((offset, report.sounds.clone()));
            }
            let terrain_blow = report
                .terrain_blow
                .expect("terrain blow should remain published");
            if offset == 2 {
                assert_eq!(
                    report.render_scene().clear_color,
                    source_terrain_blow_flash_tint(terrain_blow.source_elapsed_frames)
                );
            }
            if offset == 4 {
                assert!(
                    report
                        .game_state()
                        .world
                        .explosions
                        .iter()
                        .any(|explosion| {
                            explosion.kind == CleanExplosionKind::Terrain
                                && explosion.position == ScreenPosition::new(0x14, 0xE2)
                        })
                );
            }
            if terrain_blow.stage == TerrainBlowStage::Completed {
                saw_completion = true;
                assert_eq!(
                    terrain_blow.source_iteration,
                    SOURCE_TERRAIN_BLOW_START_SOUND_FRAMES.len() as u8
                );
                assert_eq!(terrain_blow.source_sleep_remaining, None);
            }
        }

        assert!(saw_completion);
        assert_eq!(observed_sounds, source_terrain_blow_sound_cues());
    }

    #[test]
    fn completed_abduction_consumes_human_and_spawns_mutant() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_lander_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);
        driver.step(GameInput::NONE);

        let mut converted = None;
        for _ in 0..120 {
            let step = driver.step(GameInput::NONE);
            if step.sounds.contains(&SoundCue::MutantSpawn) {
                converted = Some(step);
                break;
            }
        }
        let converted = converted.expect("carried human should convert into a mutant");

        assert_eq!(driver.snapshot_count(ActorKind::Human), 0);
        assert!(
            converted
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::Spawn(SpawnRequest::Mutant { .. })))
        );
        let settled = driver.step(GameInput::NONE);
        assert!(
            settled
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::Mutant)
        );
    }

    #[test]
    fn source_lander_abduction_spawns_source_mutant_metadata() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        let source_lander = ActorSourceLanderMetadata {
            x_fraction: 0x12,
            y_fraction: 0x34,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: u8::MAX,
            sleep_ticks: 0,
            picture_frame: 3,
            target_human_index: None,
        };
        driver.spawn_lander_from_spawn(ActorLanderSpawn {
            position: Point::new(100, HUMAN_GROUND_Y),
            source: Some(source_lander),
        });
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);
        driver.step(GameInput::NONE);

        let (converted, source_mutant) = (0..120)
            .filter_map(|_| {
                let report = driver.step(GameInput::NONE);
                report.commands.iter().find_map(|command| {
                    if let GameCommand::Spawn(SpawnRequest::Mutant {
                        source: Some(source),
                        ..
                    }) = command
                    {
                        Some((report.clone(), *source))
                    } else {
                        None
                    }
                })
            })
            .next()
            .expect("source lander should spawn a source mutant");
        let expected_source = ActorSourceMutantMetadata {
            x_fraction: source_lander.x_fraction,
            y_fraction: source_lander.y_fraction,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: ActorSourceWaveProfile::for_wave(converted.wave)
                .mutant_shot_time
                .min(u32::from(u8::MAX)) as u8,
            sleep_ticks: 0,
            hop_rng: converted
                .source_rng
                .expect("playing report should expose source rng"),
            render_x_correction: 0,
            target6_first_shot_deferred: false,
        };
        assert_eq!(source_mutant, expected_source);

        let settled = driver.step(GameInput::NONE);
        let mutant = settled
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Mutant)
            .expect("source mutant should become a live actor");
        assert_eq!(mutant.source_mutant, Some(expected_source));

        let clean_state = settled.game_state();
        let clean_mutant = clean_state
            .world
            .enemies
            .iter()
            .find(|enemy| enemy.kind == CleanEnemyKind::Mutant)
            .expect("actor bridge should expose a clean mutant");
        assert_eq!(
            clean_mutant.source_mutant,
            Some(SourceMutantSnapshot {
                x_fraction: expected_source.x_fraction,
                y_fraction: expected_source.y_fraction,
                x_velocity: expected_source.x_velocity,
                y_velocity: expected_source.y_velocity,
                shot_timer: expected_source.shot_timer,
                sleep_ticks: expected_source.sleep_ticks,
                hop_rng: clean_source_rng(expected_source.hop_rng),
                render_x_correction: expected_source.render_x_correction,
                target6_first_shot_deferred: expected_source.target6_first_shot_deferred,
            })
        );
    }

    #[test]
    fn source_mutant_actor_advances_wave_velocity_and_hop_rng() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let source = ActorSourceMutantMetadata {
            x_fraction: 0x20,
            y_fraction: 0x40,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 9,
            sleep_ticks: 0,
            hop_rng: ActorSourceRngSnapshot {
                seed: 0x81,
                hseed: 0x22,
                lseed: 0x44,
            },
            render_x_correction: 0,
            target6_first_shot_deferred: false,
        };
        let start = Point::new(100, 80);
        let mutant = driver.spawn_mutant_from_spawn(ActorMutantSpawn {
            position: start,
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let prompt = source_mutant_prompt_for_test(
            report.step,
            report.wave,
            report
                .source_rng
                .expect("playing report should carry source rng"),
            Point::new(42, 120),
            Velocity::default(),
        );
        let behavior = ActorBehaviorProfile::default();
        let (expected_position, expected_source, shot) =
            expected_source_mutant_after_motion(start, source, mutant, &prompt, behavior);

        assert_eq!(shot, None);
        let snapshot = snapshot_for(&report, mutant);
        assert_eq!(snapshot.position, expected_position);
        assert_eq!(snapshot.source_mutant, Some(expected_source));
        assert_eq!(
            expected_source.x_velocity,
            actor_source_mutant_x_velocity(
                ActorSourceWaveProfile::for_wave(1).mutant_x_velocity,
                actor_source_absolute_x(Point::new(42, 120), 0),
                actor_source_absolute_x(start, source.x_fraction),
            )
        );
        assert_ne!(expected_source.hop_rng, source.hop_rng);
    }

    #[test]
    fn source_mutant_actor_uses_prompt_source_wave_profile() {
        let actor = ActorId::new(1001);
        let default_profile = ActorSourceWaveProfile::for_wave(1);
        let mut source_profile = default_profile;
        source_profile.mutant_x_velocity = 0x48;
        source_profile.mutant_y_velocity_msb = 0x00;
        source_profile.mutant_y_velocity_lsb = 0x40;
        source_profile.mutant_random_y = 2;
        source_profile.mutant_shot_time = 12;
        assert_ne!(
            source_profile.mutant_x_velocity,
            default_profile.mutant_x_velocity
        );

        let source = ActorSourceMutantMetadata {
            x_fraction: 0x20,
            y_fraction: 0x40,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 9,
            sleep_ticks: 0,
            hop_rng: ActorSourceRngSnapshot {
                seed: 0x81,
                hseed: 0x22,
                lseed: 0x44,
            },
            render_x_correction: 0,
            target6_first_shot_deferred: false,
        };
        let start = Point::new(100, 80);
        let prompt = source_mutant_prompt_with_source_wave_for_test(
            12,
            1,
            source_profile,
            ActorSourceRngSnapshot {
                seed: 0x52,
                hseed: 0x34,
                lseed: 0x12,
            },
            Point::new(42, 120),
            Velocity::default(),
        );
        let behavior = ActorBehaviorProfile::default();
        let (expected_position, expected_source, _shot) =
            expected_source_mutant_after_motion(start, source, actor, &prompt, behavior);
        let default_x_velocity = actor_source_mutant_x_velocity(
            default_profile.mutant_x_velocity,
            actor_source_absolute_x(Point::new(42, 120), 0),
            actor_source_absolute_x(start, source.x_fraction),
        );

        let mut mutant = Mutant::from_spawn(
            actor,
            ActorMutantSpawn {
                position: start,
                source: Some(source),
            },
        );
        let reply = mutant.update(&prompt);
        let updated_source = reply
            .snapshot
            .source_mutant
            .expect("source mutant should keep source metadata");

        assert_ne!(updated_source.x_velocity, default_x_velocity);
        assert_eq!(reply.snapshot.position, expected_position);
        assert_eq!(updated_source, expected_source);
    }

    #[test]
    fn target6_source_lander_conversion_sets_mutant_render_correction() {
        let profile = ActorSourceWaveProfile::for_wave(1);
        let hop_rng = ActorSourceRngSnapshot {
            seed: 0x33,
            hseed: 0x44,
            lseed: 0x55,
        };
        let source_lander = ActorSourceLanderMetadata {
            x_fraction: 0x12,
            y_fraction: 0x34,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 0,
            sleep_ticks: 0,
            picture_frame: 0,
            target_human_index: Some(6),
        };

        let source_mutant =
            ActorSourceMutantMetadata::from_lander_conversion(source_lander, profile, hop_rng);

        assert_eq!(
            source_mutant.render_x_correction,
            SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION
        );
        assert_eq!(source_mutant.x_fraction, source_lander.x_fraction);
        assert_eq!(source_mutant.y_fraction, source_lander.y_fraction);

        let moving_lander = ActorSourceLanderMetadata {
            x_velocity: 0x0030,
            ..source_lander
        };
        assert_eq!(
            ActorSourceMutantMetadata::from_lander_conversion(moving_lander, profile, hop_rng)
                .render_x_correction,
            0
        );
    }

    #[test]
    fn target6_source_mutant_defers_first_entry_shot() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let source = ActorSourceMutantMetadata {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: 0,
            hop_rng: ActorSourceRngSnapshot {
                seed: 0x81,
                hseed: 0x22,
                lseed: 0x44,
            },
            render_x_correction: SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: false,
        };
        let mutant = driver.spawn_mutant_from_spawn(ActorMutantSpawn {
            position: Point::new(4, 0x50),
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);

        assert!(!report.sounds.contains(&SoundCue::MutantShot));
        assert!(first_enemy_laser_command(&report).is_none());
        let snapshot = snapshot_for(&report, mutant);
        let source = snapshot
            .source_mutant
            .expect("target6 mutant should keep source metadata");
        assert!(source.target6_first_shot_deferred);
        assert_eq!(source.shot_timer, SOURCE_TARGET6_MUTANT_DEFERRED_SHOT_TIMER);
        assert_eq!(source.sleep_ticks, 0);
    }

    #[test]
    fn target6_source_mutant_visible_entry_shot_uses_projected_position() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let source = ActorSourceMutantMetadata {
            x_fraction: 0x7C,
            y_fraction: 0x80,
            x_velocity: 0x0030,
            y_velocity: 0x0090,
            shot_timer: SOURCE_TARGET6_MUTANT_DEFERRED_SHOT_TIMER,
            sleep_ticks: SOURCE_MUTANT_LOOP_SLEEP_TICKS,
            hop_rng: ActorSourceRngSnapshot {
                seed: 0x44,
                hseed: 0x55,
                lseed: 0x66,
            },
            render_x_correction: SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: false,
        };
        let mutant = driver.spawn_mutant_from_spawn(ActorMutantSpawn {
            position: Point::new(0x03, 0x33),
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let shot = first_enemy_laser_command(&report)
            .expect("visible target6 entry should emit a mutant shot");

        assert!(report.sounds.contains(&SoundCue::MutantShot));
        assert_eq!(shot.0, Point::new(0x13, 0x46));
        assert_eq!(shot.2.x_fraction, source.x_fraction);
        assert_eq!(shot.2.y_fraction, source.y_fraction);
        let snapshot = snapshot_for(&report, mutant);
        let source = snapshot
            .source_mutant
            .expect("target6 mutant should keep source metadata");
        assert!(source.target6_first_shot_deferred);
        assert_eq!(source.shot_timer, SOURCE_TARGET6_MUTANT_DEFERRED_SHOT_TIMER);
        assert_eq!(source.sleep_ticks, SOURCE_MUTANT_LOOP_SLEEP_TICKS - 1);
    }

    #[test]
    fn target6_source_mutant_fire2524_sleep_shot_uses_exact_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let source = ActorSourceMutantMetadata {
            x_fraction: 0x2C,
            y_fraction: 0x60,
            x_velocity: 0x0030,
            y_velocity: 0x0090,
            shot_timer: 0x80,
            sleep_ticks: SOURCE_MUTANT_LOOP_SLEEP_TICKS,
            hop_rng: ActorSourceRngSnapshot {
                seed: 0x11,
                hseed: 0x22,
                lseed: 0x33,
            },
            render_x_correction: SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: true,
        };
        let mutant = driver.spawn_mutant_from_spawn(ActorMutantSpawn {
            position: Point::new(0x08, 0x51),
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let shot =
            first_enemy_laser_command(&report).expect("fire2524 target6 row should force a shot");

        assert!(report.sounds.contains(&SoundCue::MutantShot));
        assert_eq!(shot.0, Point::new(0x1E, 0x54));
        assert_eq!(shot.1, actor_source_screen_velocity(0xFFE0, 0x0138));
        assert_eq!(
            shot.2,
            ActorSourceEnemyProjectileMetadata {
                x_fraction: 0x33,
                y_fraction: 0x56,
                x_velocity: 0xFFE0,
                y_velocity: 0x0138,
                lifetime_ticks: actor_source_projectile_lifetime_ticks(MUTANT_SHOT_LIFETIME),
            }
        );
        let snapshot = snapshot_for(&report, mutant);
        let source = snapshot
            .source_mutant
            .expect("target6 mutant should keep source metadata");
        assert!(source.target6_first_shot_deferred);
        assert_eq!(
            source.shot_timer,
            SOURCE_TARGET6_MUTANT_FIRE2524_PENDING_SHOT_TIMER
        );
        assert_eq!(source.sleep_ticks, SOURCE_MUTANT_LOOP_SLEEP_TICKS - 1);
    }

    #[test]
    fn target6_source_mutant_shot_position_uses_dive_anchor_overrides() {
        let source = ActorSourceMutantMetadata {
            x_fraction: 0x8C,
            y_fraction: 0xB0,
            x_velocity: 0,
            y_velocity: 0x0090,
            shot_timer: 0,
            sleep_ticks: 0,
            hop_rng: ActorSourceRngSnapshot {
                seed: 0,
                hseed: 0,
                lseed: 0,
            },
            render_x_correction: SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: true,
        };

        assert_eq!(
            actor_source_target6_mutant_shot_position(Point::new(0x08, 0x61), source),
            Point::new(0x1E, 0x70)
        );
        assert_eq!(
            actor_source_target6_mutant_shot_position(
                Point::new(0x07, 0x78),
                ActorSourceMutantMetadata {
                    x_fraction: 0xFC,
                    y_fraction: 0x00,
                    ..source
                },
            ),
            Point::new(0x21, 0x87)
        );
        assert_eq!(
            actor_source_target6_mutant_shot_position(
                Point::new(0x03, 0x33),
                ActorSourceMutantMetadata {
                    x_fraction: 0x7C,
                    y_fraction: 0x80,
                    ..source
                },
            ),
            Point::new(0x13, 0x46)
        );
    }

    #[test]
    fn target6_source_mutant_collision_position_offsets_dive_projection() {
        let source = ActorSourceMutantMetadata {
            x_fraction: 0x8C,
            y_fraction: 0xB0,
            x_velocity: 0,
            y_velocity: 0x0090,
            shot_timer: 0,
            sleep_ticks: 0,
            hop_rng: ActorSourceRngSnapshot {
                seed: 0,
                hseed: 0,
                lseed: 0,
            },
            render_x_correction: SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: true,
        };

        assert_eq!(
            actor_source_target6_mutant_scene_position(Point::new(0x08, 0x61), Some(source)),
            Point::new(0x1E, 0x71)
        );
        assert_eq!(
            actor_source_target6_mutant_collision_position(Point::new(0x08, 0x61), Some(source)),
            Point::new(0x1E, 0x72)
        );
    }

    #[test]
    fn target6_source_mutant_waits_for_fire2524_collision_window() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player_id = ActorId::new(100);
        let mutant_id = ActorId::new(101);
        let raw_position = Point::new(0x08, 0x99);
        let source = ActorSourceMutantMetadata {
            x_fraction: 0x5C,
            y_fraction: 0xE0,
            x_velocity: 0x0030,
            y_velocity: 0x0090,
            shot_timer: 0x80,
            sleep_ticks: 0,
            hop_rng: ActorSourceRngSnapshot {
                seed: 0,
                hseed: 0,
                lseed: 0,
            },
            render_x_correction: SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: true,
        };
        let collision_position =
            actor_source_target6_mutant_collision_position(raw_position, Some(source));
        driver.snapshots.insert(
            player_id,
            actor_snapshot_with_bounds(
                player_id,
                ActorKind::Player,
                collision_position,
                Rect::from_center(collision_position, 18, 10),
            ),
        );
        driver.snapshots.insert(
            mutant_id,
            source_mutant_snapshot_with_bounds(
                mutant_id,
                raw_position,
                source,
                Rect::from_center(collision_position, 14, 12),
            ),
        );

        let mut commands = Vec::new();
        driver.resolve_collisions(&ActorBehaviorScript::default(), &mut commands);

        assert!(
            commands.is_empty(),
            "pending fire2524 target6 mutant should not collide yet: {commands:?}"
        );
    }

    #[test]
    fn target6_source_mutant_fire2524_collision_projects_enemy_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player_id = ActorId::new(100);
        let mutant_id = ActorId::new(101);
        let raw_position = Point::new(0x08, 0xA5);
        let player_position = SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_CENTER;
        let source = ActorSourceMutantMetadata {
            x_fraction: 0x00,
            y_fraction: 0x00,
            x_velocity: 0x0030,
            y_velocity: 0x0090,
            shot_timer: 0x80,
            sleep_ticks: 0,
            hop_rng: ActorSourceRngSnapshot {
                seed: 0,
                hseed: 0,
                lseed: 0,
            },
            render_x_correction: SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: true,
        };
        driver.snapshots.insert(
            player_id,
            actor_snapshot_with_bounds(
                player_id,
                ActorKind::Player,
                player_position,
                Rect::from_center(player_position, 18, 10),
            ),
        );
        driver.snapshots.insert(
            mutant_id,
            source_mutant_snapshot_with_bounds(
                mutant_id,
                raw_position,
                source,
                Rect::from_center(player_position, 14, 12),
            ),
        );

        let mut commands = Vec::new();
        driver.resolve_collisions(&ActorBehaviorScript::default(), &mut commands);

        assert!(commands.contains(&GameCommand::Destroy(player_id)));
        assert!(commands.contains(&GameCommand::Destroy(mutant_id)));
        assert!(commands.contains(&GameCommand::AddScore(MUTANT_SCORE)));
        assert!(commands.contains(&GameCommand::PlaySound(SoundCue::MutantHit)));
        assert!(commands.contains(&GameCommand::PlayerKilled));
        let explosions = commands
            .iter()
            .filter_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::Explosion {
                    position,
                    kind,
                    source_center,
                }) => Some((*position, *kind, *source_center)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(explosions.contains(&(
            SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_TOP_LEFT,
            ExplosionKind::Mutant,
            Some(SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_CENTER),
        )));
        assert!(explosions.contains(&(player_position, ExplosionKind::Player, None)));
    }

    #[test]
    fn source_mutant_shot_timer_spawns_source_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let source = ActorSourceMutantMetadata {
            x_fraction: 0x12,
            y_fraction: 0x34,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: 0,
            hop_rng: ActorSourceRngSnapshot {
                seed: 0x71,
                hseed: 0x44,
                lseed: 0x88,
            },
            render_x_correction: 0,
            target6_first_shot_deferred: false,
        };
        let start = Point::new(70, 120);
        let mutant = driver.spawn_mutant_from_spawn(ActorMutantSpawn {
            position: start,
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let prompt = source_mutant_prompt_for_test(
            report.step,
            report.wave,
            report
                .source_rng
                .expect("playing report should carry source rng"),
            Point::new(42, 120),
            Velocity::default(),
        );
        let behavior = ActorBehaviorProfile::default();
        let (expected_position, expected_source, expected_shot) =
            expected_source_mutant_after_motion(start, source, mutant, &prompt, behavior);
        let expected_shot = expected_shot.expect("shot timer should emit a mutant fireball");

        assert!(report.sounds.contains(&SoundCue::MutantShot));
        let mutant_shot = report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position,
                    velocity,
                    source,
                }) => source.map(|source| (*position, *velocity, source)),
                _ => None,
            })
            .expect("source mutant should emit a hostile shot command");
        assert_eq!(mutant_shot, expected_shot);
        assert_eq!(
            mutant_shot.2.lifetime_ticks,
            actor_source_projectile_lifetime_ticks(MUTANT_SHOT_LIFETIME)
        );
        let snapshot = snapshot_for(&report, mutant);
        assert_eq!(snapshot.position, expected_position);
        assert_eq!(snapshot.source_mutant, Some(expected_source));
    }

    #[test]
    fn driver_resolves_laser_lander_collision_with_score_sound_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(62, 120));

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        assert!(fired.sounds.contains(&SoundCue::Laser));

        let collision = driver.step(GameInput::NONE);
        assert_eq!(collision.score, 150);
        assert!(collision.sounds.contains(&SoundCue::LanderHit));
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);
        assert!(collision.commands.contains(&GameCommand::AddScore(150)));
        assert!(collision.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Explosion {
                    kind: ExplosionKind::Lander,
                    ..
                })
            )
        }));
    }

    #[test]
    fn driver_resolves_laser_mutant_collision_with_source_score_sound() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_mutant_for_test(Point::new(62, 120));

        driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let collision = driver.step(GameInput::NONE);

        assert_eq!(collision.score, MUTANT_SCORE);
        assert!(collision.sounds.contains(&SoundCue::MutantHit));
        assert_eq!(driver.snapshot_count(ActorKind::Mutant), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(MUTANT_SCORE))
        );
    }

    #[test]
    fn driver_resolves_laser_bomber_collision_with_source_score_sound_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_bomber_for_test(Point::new(62, 120));

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        assert!(fired.sounds.contains(&SoundCue::Laser));

        let collision = driver.step(GameInput::NONE);
        assert_eq!(collision.score, BOMBER_SCORE);
        assert!(collision.sounds.contains(&SoundCue::BomberHit));
        assert_eq!(driver.snapshot_count(ActorKind::Bomber), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(BOMBER_SCORE))
        );
    }

    #[test]
    fn driver_resolves_laser_pod_collision_with_source_score_sound_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        driver.spawn_player();
        driver.spawn_pod_for_test(Point::new(62, 120));

        driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let mut expected_rng = driver.source_rng;
        expected_rng.advance();
        let expected_first_swarmer = ActorSwarmerSpawn::source_from_pod(
            &mut expected_rng,
            ActorSourceWaveProfile::for_wave(2),
            Point::new(64, 120),
        );
        for _ in 1..SOURCE_POD_SWARMER_REQUEST_LIMIT {
            ActorSwarmerSpawn::source_from_pod(
                &mut expected_rng,
                ActorSourceWaveProfile::for_wave(2),
                Point::new(64, 120),
            );
        }
        let collision = driver.step(GameInput::NONE);

        assert_eq!(collision.score, POD_SCORE);
        assert!(collision.sounds.contains(&SoundCue::PodHit));
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(POD_SCORE))
        );
        let swarmer_spawns = collision
            .commands
            .iter()
            .filter_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::Swarmer { position, source }) => {
                    Some((*position, *source))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(swarmer_spawns.len(), SOURCE_POD_SWARMER_REQUEST_LIMIT);
        assert_eq!(
            swarmer_spawns[0],
            (
                expected_first_swarmer.position,
                expected_first_swarmer.source
            )
        );
        assert_eq!(driver.source_rng, expected_rng);

        let live = driver.step(GameInput::NONE);
        assert_eq!(
            driver.snapshot_count(ActorKind::Swarmer),
            SOURCE_POD_SWARMER_REQUEST_LIMIT
        );
        assert!(
            live.draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::Swarmer)
        );
    }

    #[test]
    fn driver_resolves_laser_baiter_collision_with_source_score_sound_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_baiter_for_test(Point::new(62, 120));

        driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let collision = driver.step(GameInput::NONE);

        assert_eq!(collision.score, BAITER_SCORE);
        assert!(collision.sounds.contains(&SoundCue::BaiterHit));
        assert_eq!(driver.snapshot_count(ActorKind::Baiter), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(BAITER_SCORE))
        );
    }

    #[test]
    fn bomb_collision_enters_game_over_with_source_bomb_sound() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.spawn_player();
        driver.spawn_bomb_for_test(Point::new(42, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::GameOver);
        assert!(report.sounds.contains(&SoundCue::BombHit));
        assert!(report.sounds.contains(&SoundCue::GameOver));
        assert!(report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Explosion {
                    kind: ExplosionKind::Bomb,
                    ..
                })
            )
        }));
    }

    #[test]
    fn explosion_actor_draws_variant_metadata() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let explosion = driver.spawn_explosion(Point::new(90, 80), ExplosionKind::Bomb);

        let report = driver.step(GameInput::NONE);

        assert!(report.draws.iter().any(|draw| {
            draw.actor == explosion
                && draw.sprite == SpriteKey::Explosion
                && matches!(
                    draw.effect,
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Bomb,
                        age: 0,
                        ..
                    }
                )
        }));
    }

    #[test]
    fn smart_bomb_pod_score_does_not_spawn_swarmers() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        driver.smart_bombs = INITIAL_SMART_BOMBS;
        driver.spawn_player();
        driver.spawn_pod_for_test(Point::new(120, 120));

        let pressed = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(pressed.score, 0);
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 1);
        let report = step_until_driver_smart_bomb_detonates(&mut driver);
        assert_eq!(report.score, POD_SCORE);
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 0);
        assert!(!report.commands.iter().any(|command| {
            matches!(command, GameCommand::Spawn(SpawnRequest::Swarmer { .. }))
        }));
    }

    #[test]
    fn high_score_entry_is_driver_owned_phase() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.score = 12_000;
        driver.next_bonus = 20_000;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::HighScoreEntry);
        assert_eq!(report.lives, 0);
        assert!(report.sounds.contains(&SoundCue::GameOver));
    }

    #[test]
    fn xyzzy_invincibility_keeps_player_alive_on_contact() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));

        let report = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                invincible: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert_ne!(report.lives, 0);
    }

    #[test]
    fn threaded_asset_is_prompted_once_per_driver_step() {
        let mut driver = ActorGameDriver::new();
        let first = driver.step(GameInput::NONE);
        let second = driver.step(GameInput::NONE);

        assert_eq!(first.step, 1);
        assert_eq!(second.step, 2);
        assert!(
            second
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::AttractDirector)
        );
    }

    fn started_driver() -> ActorGameDriver {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        step_until_driver_player_start_completes(&mut driver, 1);
        driver
    }

    fn started_source_wave_driver(wave: u16) -> (ActorGameDriver, StepReport) {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = wave.max(1);
        driver.source_rng = SOURCE_PLAYFIELD_START_RNG;
        driver.apply_wave_profile();
        driver.spawn_player();
        driver.spawn_wave_hostiles();
        driver.spawn_initial_humans();
        driver.arm_first_wave_early_lander_reserve_delay();
        let report = driver.step(GameInput::NONE);
        (driver, report)
    }

    fn source_lander_spawn_row_for_test(
        spawn: ActorLanderSpawn,
    ) -> (u16, u16, u16, u16, u8, u8, u8, Option<usize>) {
        let source = spawn
            .source
            .expect("source lander spawn should carry metadata");
        let x16 = u16::from_be_bytes([spawn.position.x as u8, source.x_fraction]);
        let y16 = u16::from_be_bytes([spawn.position.y as u8, source.y_fraction]);
        (
            x16,
            y16,
            source.x_velocity,
            source.y_velocity,
            source.shot_timer,
            source.sleep_ticks,
            source.picture_frame,
            source.target_human_index,
        )
    }

    fn destroy_source_counted_hostiles(driver: &mut ActorGameDriver, report: &StepReport) {
        let commands = report
            .snapshots
            .iter()
            .filter(|snapshot| is_hostile(snapshot.kind))
            .map(|snapshot| GameCommand::Destroy(snapshot.id))
            .collect::<Vec<_>>();
        driver.apply_commands(&commands);
    }

    fn scene_has_survivor_bonus_icon(scene: &RenderScene, position: [f32; 2]) -> bool {
        scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == position
                && sprite.size == SOURCE_SURVIVOR_BONUS_HUMAN_SIZE
                && sprite.tint == Color::WHITE
        })
    }

    fn step_until_wave_started(driver: &mut ActorGameDriver, wave: u16) -> StepReport {
        for _ in 0..=256 {
            let report = driver.step(GameInput::NONE);
            if report.commands.contains(&GameCommand::AdvanceWave { wave }) {
                return report;
            }
        }

        panic!("wave {wave} should start after survivor bonus cadence");
    }

    fn step_until_driver_player_start_completes(
        driver: &mut ActorGameDriver,
        player: u8,
    ) -> StepReport {
        let mut previous_delay = SOURCE_START_PLAYFIELD_DELAY_STEPS.saturating_add(1);
        for _ in 0..=SOURCE_START_PLAYFIELD_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            if let Some(player_start) = report.player_start {
                assert_eq!(player_start.player, player);
                assert!(player_start.delay_steps_remaining < previous_delay);
                previous_delay = player_start.delay_steps_remaining;
                assert!(
                    !report
                        .commands
                        .contains(&GameCommand::AdvanceWave { wave: 1 })
                );
                assert_no_source_message(&report, "PLYR1", SOURCE_PLAYER_START_PROMPT_SCREEN);
                continue;
            }

            assert_eq!(report.phase, Phase::Playing);
            assert_eq!(report.current_player, player);
            assert!(
                report
                    .commands
                    .contains(&GameCommand::AdvanceWave { wave: 1 })
            );
            assert_eq!(report.sounds, [SoundCue::PlayerAppear]);
            return report;
        }

        panic!("player {player} start should complete after source delay");
    }

    fn step_until_driver_smart_bomb_detonates(driver: &mut ActorGameDriver) -> StepReport {
        for _ in 0..=SOURCE_SMART_BOMB_DETONATION_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            if report.smart_bomb_flash_steps_remaining == SOURCE_SMART_BOMB_FLASH_STEPS {
                return report;
            }
        }

        panic!("source smart bomb should detonate after the source delay");
    }

    fn step_until_player_switch_completes(
        runtime: &mut ActorRuntimeAdapter,
        to_player: u8,
    ) -> ActorFrame {
        let from_player = if to_player == 1 { 2 } else { 1 };
        for expected_sleep in (1..SOURCE_PLAYER_SWITCH_SLEEP_STEPS).rev() {
            let waiting = runtime.step(GameInput::NONE);
            assert_eq!(waiting.report.phase, Phase::GameOver);
            assert_eq!(
                waiting.report.player_switch,
                Some(PlayerSwitchReport {
                    sleep_steps_remaining: expected_sleep,
                    from_player,
                    to_player,
                })
            );
            assert_eq!(
                waiting.state.game_over.player_switch_sleep_remaining,
                Some(expected_sleep)
            );
            assert!(!waiting.events.gameplay().contains(&GameEvent::GameOver));
            assert!(!waiting.report.sounds.contains(&SoundCue::GameOver));
            assert_source_message(
                &waiting.report,
                player_source_message_label(from_player),
                SOURCE_PLAYER_SWITCH_LABEL_SCREEN,
            );
            assert_source_message(&waiting.report, "GO", SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN);
            assert_source_message_scene(
                &waiting.scene,
                player_source_message_label(from_player),
                SOURCE_PLAYER_SWITCH_LABEL_SCREEN,
            );
            assert_source_message_scene(
                &waiting.scene,
                "GO",
                SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN,
            );
            assert_no_source_message(
                &waiting.report,
                player_source_message_label(to_player),
                SOURCE_PLAYER_START_PROMPT_SCREEN,
            );
            assert!(
                !waiting
                    .report
                    .draws
                    .iter()
                    .any(|draw| matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
            );
        }

        let switched = runtime.step(GameInput::NONE);
        assert_no_source_message(
            &switched.report,
            player_source_message_label(from_player),
            SOURCE_PLAYER_SWITCH_LABEL_SCREEN,
        );
        assert_no_source_message(
            &switched.report,
            "GO",
            SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN,
        );
        assert_source_message(
            &switched.report,
            player_source_message_label(to_player),
            SOURCE_PLAYER_START_PROMPT_SCREEN,
        );
        assert_source_message_scene(
            &switched.scene,
            player_source_message_label(to_player),
            SOURCE_PLAYER_START_PROMPT_SCREEN,
        );
        switched
    }

    fn step_until_player_start_completes(
        runtime: &mut ActorRuntimeAdapter,
        player: u8,
    ) -> ActorFrame {
        let mut previous_delay = SOURCE_START_PLAYFIELD_DELAY_STEPS.saturating_add(1);
        for _ in 0..=SOURCE_START_PLAYFIELD_DELAY_STEPS {
            let frame = runtime.step(GameInput::NONE);
            if let Some(player_start) = frame.report.player_start {
                assert_eq!(player_start.player, player);
                assert!(player_start.delay_steps_remaining < previous_delay);
                previous_delay = player_start.delay_steps_remaining;
                assert!(!frame.events.gameplay().contains(&GameEvent::WaveStarted));
                assert!(frame.state.world.enemies.is_empty());
                if frame.report.player_count > 1 {
                    assert_source_message(
                        &frame.report,
                        player_source_message_label(player),
                        SOURCE_PLAYER_START_PROMPT_SCREEN,
                    );
                    assert_no_source_message(
                        &frame.report,
                        "GO",
                        SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN,
                    );
                } else {
                    assert_no_source_message(
                        &frame.report,
                        player_source_message_label(player),
                        SOURCE_PLAYER_START_PROMPT_SCREEN,
                    );
                }
                continue;
            }

            assert_eq!(frame.report.phase, Phase::Playing);
            assert_eq!(frame.report.current_player, player);
            assert_no_source_message(
                &frame.report,
                player_source_message_label(player),
                SOURCE_PLAYER_START_PROMPT_SCREEN,
            );
            assert_no_source_message(&frame.report, "GO", SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN);
            assert!(frame.events.gameplay().contains(&GameEvent::WaveStarted));
            assert_eq!(
                frame.events.sounds(),
                &[SoundEvent::UnmappedSoundCommand { command: 0xEA }]
            );
            assert_eq!(frame.report.sounds, [SoundCue::PlayerAppear]);
            return frame;
        }

        panic!("player {player} start should complete after source delay");
    }

    fn step_until_smart_bomb_detonates(runtime: &mut ActorRuntimeAdapter) -> ActorFrame {
        for _ in 0..=SOURCE_SMART_BOMB_DETONATION_DELAY_STEPS {
            let frame = runtime.step(GameInput::NONE);
            if frame.report.smart_bomb_flash_steps_remaining == SOURCE_SMART_BOMB_FLASH_STEPS {
                return frame;
            }
        }

        panic!("source smart bomb should detonate after the source delay");
    }

    fn source_smart_bomb_sound_cues() -> Vec<SoundCue> {
        SOURCE_SMART_BOMB_SOUND_SEQUENCE
            .iter()
            .map(|(_, command)| SoundCue::SourceCommand(*command))
            .collect()
    }

    fn source_terrain_blow_sound_cues() -> Vec<(u16, Vec<SoundCue>)> {
        SOURCE_TERRAIN_BLOW_START_SOUND_FRAMES
            .iter()
            .copied()
            .map(|frame| {
                (
                    frame,
                    vec![SoundCue::SourceCommand(SOURCE_SBSND_SOUND_COMMAND)],
                )
            })
            .chain(std::iter::once((
                SOURCE_TERRAIN_BLOW_COMPLETE_FRAME,
                vec![SoundCue::SourceCommand(SOURCE_TBSND_SOUND_COMMAND)],
            )))
            .chain(SOURCE_TERRAIN_BLOW_SOUND_TAIL_SEQUENCE.iter().copied().map(
                |(offset, command)| {
                    (
                        SOURCE_TERRAIN_BLOW_COMPLETE_FRAME + u16::from(offset),
                        vec![SoundCue::SourceCommand(command)],
                    )
                },
            ))
            .collect()
    }

    fn collect_driver_smart_bomb_sound_sequence(driver: &mut ActorGameDriver) -> Vec<SoundCue> {
        let mut sounds = Vec::new();
        let last_step = SOURCE_SMART_BOMB_SOUND_SEQUENCE
            .last()
            .expect("source smart bomb sound sequence should not be empty")
            .0;
        for _ in 0..last_step {
            sounds.extend(driver.step(GameInput::NONE).sounds);
        }
        sounds
    }

    fn step_until_driver_source_reserve_activates(driver: &mut ActorGameDriver) -> StepReport {
        for _ in 0..=SOURCE_SMART_BOMB_RESERVE_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            if report
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::Spawn(SpawnRequest::Lander { .. })))
            {
                return report;
            }
        }

        panic!("source enemy reserve should reactivate after smart-bomb cooldown");
    }

    fn step_until_first_wave_early_reserve_materializes(
        driver: &mut ActorGameDriver,
    ) -> StepReport {
        for _ in 0..=SOURCE_FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            if report.sounds.contains(&SoundCue::HyperspaceMaterialize) {
                return report;
            }
        }

        panic!("first-wave early reserve should materialize on source cadence");
    }

    fn snapshot_for(report: &StepReport, id: ActorId) -> &ActorSnapshot {
        report
            .snapshots
            .iter()
            .find(|snapshot| snapshot.id == id)
            .expect("actor snapshot should be present")
    }

    fn source_human_spawn_for_test(
        position: Point,
        target_slot_index: usize,
        picture_frame: u8,
    ) -> ActorHumanSpawn {
        ActorHumanSpawn {
            position,
            mode: HumanMode::Grounded,
            source: Some(ActorSourceHumanMetadata {
                x_fraction: 0,
                y_fraction: 0,
                picture_frame,
                target_slot_index,
            }),
        }
    }

    fn expected_source_bomber_after_motion(
        position: Point,
        mut source: ActorSourceBomberMetadata,
        _step: u64,
        _id: ActorId,
        source_rng: Option<ActorSourceRngSnapshot>,
        player_position: Option<Point>,
    ) -> (Point, ActorSourceBomberMetadata) {
        if let Some(source_rng) = source_rng
            && source.source_slot == actor_source_tie_selected_slot(source_rng.seed)
        {
            if source.sleep_ticks > 0 {
                source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            } else {
                source.picture_frame =
                    actor_source_bomber_picture_frame(source_rng.seed, source.picture_frame);
                source.y_velocity =
                    actor_source_bomber_random_y_velocity(source.y_velocity, source_rng.seed);
                if position.y == 0 {
                    source.y_velocity = actor_source_bomber_cruise_y_velocity(
                        source.y_velocity,
                        &mut source.cruise_altitude,
                        position.y,
                        source_rng.seed,
                    );
                } else if let Some(player) = player_position
                    && let Some(delta) =
                        actor_source_bomber_onscreen_y_velocity_delta(position.y, player.y)
                {
                    source.y_velocity = source.y_velocity.wrapping_add(delta);
                }
                source.sleep_ticks = SOURCE_BOMBER_LOOP_SLEEP_TICKS;
            }
        }

        let (x, x_fraction) =
            actor_source_axis_step(position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) =
            actor_source_active_object_y_step(position.y, source.y_fraction, source.y_velocity);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        (Point::new(x, y), source)
    }

    fn actor_snapshot(id: u64, kind: ActorKind, position: Point) -> ActorSnapshot {
        actor_snapshot_with_bounds(
            ActorId(id),
            kind,
            position,
            Rect::from_center(position, 4, 4),
        )
    }

    fn actor_snapshot_with_bounds(
        id: ActorId,
        kind: ActorKind,
        position: Point,
        bounds: Rect,
    ) -> ActorSnapshot {
        ActorSnapshot {
            id,
            kind,
            position,
            velocity: Velocity::default(),
            direction: None,
            bounds: Some(bounds),
            alive: true,
            source_lander: None,
            source_bomber: None,
            source_pod: None,
            source_swarmer: None,
            source_baiter: None,
            source_mutant: None,
            source_human: None,
            source_enemy_projectile: None,
        }
    }

    fn source_mutant_snapshot_with_bounds(
        id: ActorId,
        position: Point,
        source: ActorSourceMutantMetadata,
        bounds: Rect,
    ) -> ActorSnapshot {
        let mut snapshot = actor_snapshot_with_bounds(id, ActorKind::Mutant, position, bounds);
        snapshot.source_mutant = Some(source);
        snapshot
    }

    fn actor_snapshot_with_velocity(
        id: u64,
        kind: ActorKind,
        position: Point,
        velocity: Velocity,
    ) -> ActorSnapshot {
        let mut snapshot = actor_snapshot(id, kind, position);
        snapshot.velocity = velocity;
        snapshot
    }

    fn source_mutant_prompt_for_test(
        step: u64,
        wave: u16,
        source_rng: ActorSourceRngSnapshot,
        player_position: Point,
        player_velocity: Velocity,
    ) -> StepPrompt {
        source_mutant_prompt_with_source_wave_for_test(
            step,
            wave,
            ActorSourceWaveProfile::for_wave(wave),
            source_rng,
            player_position,
            player_velocity,
        )
    }

    fn source_mutant_prompt_with_source_wave_for_test(
        step: u64,
        wave: u16,
        source_wave: ActorSourceWaveProfile,
        source_rng: ActorSourceRngSnapshot,
        player_position: Point,
        player_velocity: Velocity,
    ) -> StepPrompt {
        StepPrompt {
            step,
            phase: Phase::Playing,
            input: GameInput::NONE,
            wave,
            source_wave,
            current_player: 1,
            player_count: 1,
            score: 0,
            player_scores: [0, 0],
            credits: 0,
            lives: 3,
            smart_bombs: INITIAL_SMART_BOMBS,
            smart_bomb_pending: false,
            player_stocks: [PlayerStockSnapshot::new(3, INITIAL_SMART_BOMBS); 2],
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [0; 5],
            high_score_initials: HighScoreInitialsState::EMPTY,
            snapshots: vec![actor_snapshot_with_velocity(
                999,
                ActorKind::Player,
                player_position,
                player_velocity,
            )],
            behavior_script: ActorBehaviorScript::default(),
            source_background_left: 0,
            source_rng: Some(source_rng),
            source_human_walk_target_slot: None,
            source_shell_scan_tick: false,
        }
    }

    fn expected_source_mutant_after_motion(
        mut position: Point,
        mut source: ActorSourceMutantMetadata,
        actor: ActorId,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> (
        Point,
        ActorSourceMutantMetadata,
        Option<(Point, Velocity, ActorSourceEnemyProjectileMetadata)>,
    ) {
        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return (position, source, None);
        }

        let player_position = prompt
            .player_position()
            .expect("source mutant expected helper needs a player");
        let profile = prompt.source_wave;
        let player_absolute_x = actor_source_absolute_x(player_position, 0);
        let object_absolute_x = actor_source_absolute_x(position, source.x_fraction);
        source.x_velocity = actor_source_mutant_x_velocity(
            profile.mutant_x_velocity,
            player_absolute_x,
            object_absolute_x,
        );
        source.y_velocity = actor_source_mutant_y_velocity(
            profile,
            player_position.y,
            player_absolute_x,
            object_absolute_x,
            position,
        );

        let mut shot = None;
        if actor_source_mutant_should_hop_and_shoot(player_absolute_x, object_absolute_x, position)
        {
            let mut hop_rng = actor_source_rng_from_snapshot(source.hop_rng);
            let hop_state = hop_rng.advance();
            source.hop_rng = hop_state.snapshot();
            position.y =
                actor_source_mutant_hop_y(position.y, profile.mutant_random_y, hop_state.seed);
            source.shot_timer = source.shot_timer.wrapping_sub(1);
            if source.shot_timer == 0 {
                let shot_rng = actor_source_mutant_shot_rng(prompt, actor, position);
                source.shot_timer = actor_source_mutant_shot_reset(profile, shot_rng.seed);
                shot = actor_source_mutant_fireball(position, prompt, behavior, source, shot_rng)
                    .map(|(velocity, projectile_source)| (position, velocity, projectile_source));
            }
        }

        let (x, x_fraction) =
            actor_source_axis_step(position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) =
            actor_source_active_object_y_step(position.y, source.y_fraction, source.y_velocity);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        source.sleep_ticks = SOURCE_MUTANT_LOOP_SLEEP_TICKS;
        (Point::new(x, y), source, shot)
    }

    fn source_shell_snapshot_count(report: &StepReport) -> usize {
        report
            .snapshots
            .iter()
            .filter(|snapshot| is_source_shell_kind(snapshot.kind))
            .count()
    }

    fn bomb_shell_snapshot_count(report: &StepReport) -> usize {
        report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomb)
            .count()
    }

    fn first_enemy_laser_command(
        report: &StepReport,
    ) -> Option<(Point, Velocity, ActorSourceEnemyProjectileMetadata)> {
        report.commands.iter().find_map(|command| match command {
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position,
                velocity,
                source: Some(source),
            }) => Some((*position, *velocity, *source)),
            _ => None,
        })
    }

    fn enemy_laser_snapshot_count(report: &StepReport) -> usize {
        report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
            .count()
    }

    fn assert_text(report: &StepReport, value: &str) {
        assert!(
            report
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(value)),
            "expected draw text {value:?}"
        );
    }

    fn assert_source_message(report: &StepReport, label: &str, top_left_screen_address: u16) {
        let scene = report.render_scene();
        assert_source_message_scene(&scene, label, top_left_screen_address);
    }

    fn assert_source_message_scene(scene: &RenderScene, label: &str, top_left_screen_address: u16) {
        for (sprite_id, position, size) in
            expected_plain_source_message_sprites(label, top_left_screen_address)
        {
            assert!(
                scene.sprites.iter().any(|sprite| {
                    sprite.sprite == sprite_id
                        && sprite.layer == RenderLayer::Overlay
                        && sprite.position == position
                        && sprite.size == size
                        && sprite.tint == Color::WHITE
                }),
                "expected full source message {label} glyph {sprite_id:?} at {top_left_screen_address:#06x}"
            );
        }
    }

    fn expected_plain_source_message_sprites(
        label: &str,
        top_left_screen_address: u16,
    ) -> Vec<(SpriteId, [f32; 2], [f32; 2])> {
        let text = source_message_text(label).expect("source message label should exist");
        let mut cursor = top_left_screen_address;
        let mut expected = Vec::new();
        for character in text.chars() {
            let size = SpriteId::message_glyph_size(character)
                .expect("test source prompt should use clean message glyphs");
            if character != ' ' {
                let sprite =
                    SpriteId::message_glyph(character).expect("visible prompt glyph should exist");
                expected.push((
                    sprite,
                    source_screen_position(cursor),
                    [size[0] as f32, size[1] as f32],
                ));
            }
            cursor = source_test_text_cursor_after_glyph(cursor, size[0]);
        }
        assert!(
            !expected.is_empty(),
            "source message {label} should contain visible glyphs"
        );
        expected
    }

    fn source_test_text_cursor_after_glyph(cursor: u16, width_pixels: u32) -> u16 {
        let [column, row] = cursor.to_be_bytes();
        let width_columns =
            u8::try_from(width_pixels / 2).expect("source glyph width should fit in u8");
        u16::from_be_bytes([column.wrapping_add(width_columns).wrapping_add(1), row])
    }

    fn assert_no_source_message(report: &StepReport, label: &str, top_left_screen_address: u16) {
        let text = source_message_text(label).expect("source message label should exist");
        let first_glyph = text
            .chars()
            .find_map(SpriteId::message_glyph)
            .expect("source message should contain a visible glyph");
        let position = source_screen_position(top_left_screen_address);
        let scene = report.render_scene();
        assert!(
            scene.sprites.iter().all(|sprite| {
                sprite.sprite != first_glyph
                    || sprite.layer != RenderLayer::Overlay
                    || sprite.position != position
            }),
            "unexpected source message {label} at {top_left_screen_address:#06x}"
        );
    }
}
