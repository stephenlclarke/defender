use crate::{
    game::{
        ATTRACT_SCORING_SEQUENCE_START_FRAME, AttractPresentationSnapshot,
        Direction as CleanDirection, EnemyKind as CleanEnemyKind,
        EnemyProjectileSnapshot as CleanEnemyProjectileSnapshot, EnemyProjectileKind,
        EnemyReserveSnapshot, EnemySnapshot as CleanEnemySnapshot,
        ExplosionKind as CleanExplosionKind, ExplosionSnapshot as CleanExplosionSnapshot,
        GameEvent, GameEvents, GameFrame, GameInput as CleanGameInput, GameOverSnapshot, GamePhase,
        GameState, HIGH_SCORE_TABLE_ENTRIES, HighScoreEntrySnapshot, HighScoreTableEntrySnapshot,
        HighScoreTablesSnapshot, HumanSnapshot as CleanHumanSnapshot, PlayerSnapshot,
        PlayerStockSnapshot, ProjectileSnapshot as CleanProjectileSnapshot,
        ScorePopupKind as CleanScorePopupKind, ScorePopupSnapshot as CleanScorePopupSnapshot,
        ScoreSnapshot, SoundEvent, BaiterRuntimeSnapshot, BomberRuntimeSnapshot,
        LanderRuntimeSnapshot, MutantRuntimeSnapshot, PodRuntimeSnapshot, ArcadeRngSnapshot,
        SwarmerRuntimeSnapshot, TERRAIN_BLOW_COMPLETE_FRAME, TERRAIN_BLOW_FLASH_COLOR_BYTES,
        TERRAIN_BLOW_OVERLOAD_COUNTER, TERRAIN_BLOW_START_SOUND_FRAMES,
        TERRAIN_EXPLOSION_LIFETIME_FRAMES, TerrainBlowSnapshot, TerrainBlowStage, TerrainSegment,
        VISUAL_STATE, WaveProfileSnapshot, WorldSnapshot, WorldVector, push_appearance_cloud_pixels,
        push_explosion_cloud_pixels, push_scanner_radar_sprites, push_source_bgout_terrain_sprites,
        source_appearance_size_for_age,
        source_explosion_render_scale, source_explosion_size_for_age,
        source_terrain_blow_flash_tint, source_terrain_explosion_size_for_age,
        source_wave_landscape_tint,
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

const PLAYER_SPEED: i16 = 1;
const ACTOR_RENDER_SURFACE: SurfaceSize = SurfaceSize::new(292, 240);
const INITIAL_PLAYER_LIVES: u8 = 3;
const INITIAL_SMART_BOMBS: u8 = 3;
const PLAYER_LASER_COOLDOWN_STEPS: u8 = 8;
const PLAYER_HYPERSPACE_HIDDEN_STEPS: u8 = 33;
const PLAYER_HYPERSPACE_REMATERIALIZE_X: i16 = 128;
const PLAYER_HYPERSPACE_REMATERIALIZE_Y: i16 = 120;
const PLAYER_HYPERSPACE_DEATH_DELAY_STEPS: u8 = 39;
const PLAYER_HYPERSPACE_DEATH_LOW_SEED: u8 = 0x0C; // original: PLAYER_HYPERSPACE_DEATH_LSEED
const HYPERSPACE_DEATH_LOW_SEED_THRESHOLD: u8 = 0xC0; // original: SOURCE_HYPERSPACE_DEATH_LSEED_THRESHOLD
const PLAYFIELD_TOP_EDGE_Y: u8 = 42; // original: SOURCE_PLAYFIELD_Y_MIN
const PLAYFIELD_BOTTOM_EDGE_Y: u8 = 240; // original: SOURCE_PLAYFIELD_Y_MAX
const MUTANT_RESTORE_AVOID_HALF_WIDTH: u16 = 300 * 32; // original: SOURCE_MUTANT_RESTORE_AVOID_HALF_WIDTH
const MUTANT_RESTORE_AVOID_WIDTH: u16 = 600 * 32; // original: SOURCE_MUTANT_RESTORE_AVOID_WIDTH
const ENEMY_PROJECTILE_SCAN_INITIAL_DELAY_STEPS: u8 = 6; // original: SOURCE_SHELL_SCAN_INITIAL_DELAY_STEPS
const ENEMY_PROJECTILE_SCAN_CADENCE_STEPS: u8 = 8; // original: SOURCE_SHELL_SCAN_CADENCE_STEPS
const ENEMY_PROJECTILE_SLOT_LIMIT: usize = 20; // original: SOURCE_SHELL_LIMIT
const ENEMY_PROJECTILE_LIFETIME_TICKS: u8 = 20; // original: SOURCE_SHELL_LIFETIME_TICKS
const SMART_BOMB_DETONATION_DELAY_STEPS: u8 = 3; // original: SOURCE_SMART_BOMB_DETONATION_DELAY_STEPS
const SMART_BOMB_FLASH_STEPS: u8 = 5; // original: SOURCE_SMART_BOMB_FLASH_STEPS
const SMART_BOMB_RESERVE_DELAY_STEPS: u16 = 240; // original: SOURCE_SMART_BOMB_RESERVE_DELAY_STEPS
const SMART_BOMB_SOUND_COMMAND: u8 = 0xEE; // original: SBSND
const CANNON_SOUND_COMMAND: u8 = 0xE8; // original: CANNON
const TERRAIN_BLOW_SOUND_COMMAND: u8 = 0xEB; // original: TBSND
const ASTRONAUT_CATCH_SOUND_COMMAND: u8 = 0xF7; // original: ACSND
const ASTRONAUT_SHORT_CATCH_SOUND_COMMAND: u8 = 0xE5; // original: ASCSND
const APPEARANCE_SOUND_COMMAND: u8 = 0xEA; // original: APPEAR
const SMART_BOMB_SOUND_SEQUENCE: [(u8, u8); 7] = [
    // original: SOURCE_SMART_BOMB_SOUND_SEQUENCE
    (4, SMART_BOMB_SOUND_COMMAND),
    (8, SMART_BOMB_SOUND_COMMAND),
    (12, SMART_BOMB_SOUND_COMMAND),
    (16, SMART_BOMB_SOUND_COMMAND),
    (20, SMART_BOMB_SOUND_COMMAND),
    (24, SMART_BOMB_SOUND_COMMAND),
    (28, CANNON_SOUND_COMMAND),
];
const TERRAIN_BLOW_SOUND_TAIL_SEQUENCE: [(u8, u8); 4] = [
    // original: SOURCE_TERRAIN_BLOW_SOUND_TAIL_SEQUENCE
    (4, SMART_BOMB_SOUND_COMMAND),
    (10, SMART_BOMB_SOUND_COMMAND),
    (16, CANNON_SOUND_COMMAND),
    (26, CANNON_SOUND_COMMAND),
];
const ASTRONAUT_CATCH_SOUND_TAIL_SEQUENCE: [(u8, u8); 2] = [
    // original: SOURCE_ACSND_SOUND_TAIL_SEQUENCE
    (10, ASTRONAUT_CATCH_SOUND_COMMAND),
    (20, ASTRONAUT_CATCH_SOUND_COMMAND),
];
const PLAYER_SWITCH_DELAY_STEPS: u8 = 0x60; // original: SOURCE_PLAYER_SWITCH_SLEEP_STEPS
const FINAL_GAME_OVER_DELAY_STEPS: u8 = 40; // original: SOURCE_FINAL_GAME_OVER_SLEEP_STEPS
const PLAYER_START_SOUND_DELAY_STEPS: u8 = 1; // original: SOURCE_START_SOUND_DELAY_STEPS
const PLAYER_START_PLAYFIELD_DELAY_STEPS: u8 = 138; // original: SOURCE_START_PLAYFIELD_DELAY_STEPS
const ENEMY_PROJECTILE_MAX_SCREEN_X: i16 = 0x98; // original: SOURCE_SHELL_X_MAX
const PLAYFIELD_START_RNG: ActorSourceRng = ActorSourceRng {
    // original: SOURCE_PLAYFIELD_START_RNG
    seed: 0x52,
    hseed: 0x62,
    lseed: 0x0C,
};
const DEFAULT_RNG: ActorSourceRng = ActorSourceRng {
    // original: SOURCE_DEFAULT_RNG
    seed: 0,
    hseed: 0xA5,
    lseed: 0x5A,
};
const FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS: u16 = 449; // original: SOURCE_FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS
const FIRST_WAVE_EARLY_RESERVE_ACTIVE_LIMIT: usize = 10; // original: SOURCE_FIRST_WAVE_EARLY_RESERVE_ACTIVE_LIMIT
const FIRST_WAVE_EARLY_RESERVE_TARGET_CURSOR_SLOT: usize = 6; // original: SOURCE_FIRST_WAVE_EARLY_RESERVE_TARGET_CURSOR_SLOT
const FIRST_WAVE_EARLY_RESERVE_TARGET2_SHOT_PHASE_DELAY: u8 = 2; // original: SOURCE_FIRST_WAVE_EARLY_RESERVE_TARGET2_SHOT_PHASE_DELAY
const FIRST_WAVE_EARLY_RESERVE_RNG: ActorSourceRng = ActorSourceRng {
    // original: SOURCE_FIRST_WAVE_EARLY_RESERVE_RNG
    seed: 0x3A,
    hseed: 0xDA,
    lseed: 0x1F,
};
const FIRST_WAVE_LANDER_REFILL_ACTIVE_THRESHOLD: usize = 8; // original: SOURCE_FIRST_WAVE_LANDER_REFILL_ACTIVE_THRESHOLD
const FIRST_WAVE_LANDER_REFILL_DELAY_STEPS: u8 = 47; // original: SOURCE_FIRST_WAVE_LANDER_REFILL_DELAY_STEPS
const FIRST_WAVE_LANDER_REFILL_APPEAR_SOUND_DELAY_STEPS: u8 = 1; // original: SOURCE_FIRST_WAVE_LANDER_REFILL_APPEAR_SOUND_DELAY_STEPS
const PLAYER_PLAYFIELD_TOP_Y: i16 = PLAYFIELD_TOP_EDGE_Y as i16;
const PLAYER_BOUNDS: Rect = Rect::new(0, PLAYER_PLAYFIELD_TOP_Y, 255, 220);
const PLAYER_SCROLL_CENTER_X: i16 = 128;
const BACKGROUND_WORD_PER_PIXEL: u16 = 0x0100; // original: SOURCE_BACKGROUND_WORD_PER_PIXEL
const LASER_SPEED: i16 = 8;
const LASER_LIFETIME: u16 = 34;
const LANDER_FIRE_PERIOD: u64 = 96;
const LANDER_SHOT_SPEED: i16 = 3;
const LANDER_SHOT_LIFETIME: u16 = 90;
const EXPLOSION_LIFETIME: u16 = 20;
const SCORE_POPUP_LIFETIME: u16 = 50;
const ATTRACT_PRESENTS_START_STEP: u64 = 236; // original: SOURCE_ATTRACT_PRESENTS_START_STEP
const ATTRACT_DEFENDER_WORDMARK_START_STEP: u64 = 365; // original: SOURCE_ATTRACT_DEFENDER_WORDMARK_START_STEP
const ATTRACT_HALL_OF_FAME_START_STEP: u64 = 600; // original: SOURCE_ATTRACT_HALL_OF_FAME_START_STEP
const ATTRACT_SCORING_SEQUENCE_START_STEP: u64 = ATTRACT_SCORING_SEQUENCE_START_FRAME as u64; // original: SOURCE_ATTRACT_SCORING_SEQUENCE_START_STEP
const ATTRACT_CYCLE_STEPS: u64 =
    ATTRACT_SCORING_SEQUENCE_START_STEP + ATTRACT_SCORING_DEMO_TOTAL_STEPS as u64; // original: SOURCE_ATTRACT_CYCLE_STEPS
const HIGH_SCORE_HALL_STALL_STEPS: u8 = 60; // original: SOURCE_HIGH_SCORE_HALL_STALL_STEPS
const ATTRACT_WILLIAMS_LOGO_DURATION_STEPS: u64 = ATTRACT_HALL_OF_FAME_START_STEP - 1; // original: SOURCE_ATTRACT_WILLIAMS_LOGO_DURATION_STEPS
const ATTRACT_PRESENTS_DURATION_STEPS: u64 =
    ATTRACT_HALL_OF_FAME_START_STEP - ATTRACT_PRESENTS_START_STEP; // original: SOURCE_ATTRACT_PRESENTS_DURATION_STEPS
const REPLAY_BONUS_SCORE: u32 = 10_000; // original: SOURCE_REPLAY_SCORE
const WAVE_COMPLETION_STATUS_LINES: &[(&str, u16)] =
    // original: SOURCE_WAVE_COMPLETION_STATUS_LINES
    &[("ATWV", 0x3850), ("COMPV", 0x3D60), ("BONSX", 0x3C90)];
const WAVE_COMPLETION_WAVE_NUMBER_SCREEN: u16 = 0x6550; // original: SOURCE_WAVE_COMPLETION_WAVE_NUMBER_SCREEN
const WAVE_COMPLETION_MULTIPLIER_NUMBER_SCREEN: u16 = 0x5890; // original: SOURCE_WAVE_COMPLETION_MULTIPLIER_NUMBER_SCREEN
const SURVIVOR_BONUS_FIRST_HUMAN_SCREEN: u16 = 0x3CA0; // original: SOURCE_SURVIVOR_BONUS_FIRST_HUMAN_SCREEN
const SURVIVOR_BONUS_HUMAN_STEP: u8 = 0x04; // original: SOURCE_SURVIVOR_BONUS_HUMAN_STEP
const SURVIVOR_BONUS_HUMAN_LIMIT: usize = 10; // original: SOURCE_SURVIVOR_BONUS_HUMAN_LIMIT
const SURVIVOR_BONUS_HUMAN_SIZE: [f32; 2] = [4.0, 8.0]; // original: SOURCE_SURVIVOR_BONUS_HUMAN_SIZE
const SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS: u8 = 4; // original: SOURCE_SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS
const SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS: u8 = 0x80; // original: SOURCE_SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS
const ATTRACT_DEFENDER_WORDMARK_DURATION_STEPS: u64 =
    ATTRACT_HALL_OF_FAME_START_STEP - ATTRACT_DEFENDER_WORDMARK_START_STEP; // original: SOURCE_ATTRACT_DEFENDER_WORDMARK_DURATION_STEPS
const ATTRACT_HALL_OF_FAME_DURATION_STEPS: u64 =
    ATTRACT_SCORING_SEQUENCE_START_STEP - ATTRACT_HALL_OF_FAME_START_STEP; // original: SOURCE_ATTRACT_HALL_OF_FAME_DURATION_STEPS
const WILLIAMS_REVEAL_STEPS: u16 = ATTRACT_PRESENTS_START_STEP as u16;
const WILLIAMS_COLOR_PERIOD: u16 = 8;
const ATTRACT_WILLIAMS_LOGO_POSITION: Point = Point::new(108, 60); // original: SOURCE_ATTRACT_WILLIAMS_LOGO_POSITION
const ATTRACT_DEFENDER_WORDMARK_POSITION: Point = Point::new(96, 144); // original: SOURCE_ATTRACT_DEFENDER_WORDMARK_POSITION
const ATTRACT_CREDIT_LABEL_POSITION: Point = Point::new(176, 226); // original: SOURCE_ATTRACT_CREDIT_LABEL_POSITION
const ATTRACT_CREDIT_COUNT_POSITION: Point = Point::new(248, 226); // original: SOURCE_ATTRACT_CREDIT_COUNT_POSITION
const ATTRACT_HALL_TITLE_LABEL: &str = "HALLD_TITLE"; // original: SOURCE_ATTRACT_HALL_TITLE_LABEL
const ATTRACT_HALL_TODAYS_LABEL: &str = "HALLD_TODAYS"; // original: SOURCE_ATTRACT_HALL_TODAYS_LABEL
const ATTRACT_HALL_ALL_TIME_LABEL: &str = "HALLD_ALL_TIME"; // original: SOURCE_ATTRACT_HALL_ALL_TIME_LABEL
const ATTRACT_HALL_GREATEST_LABEL: &str = "HALLD_GREATEST"; // original: SOURCE_ATTRACT_HALL_GREATEST_LABEL
const ATTRACT_HALL_DEFENDER_LOGO_POSITION: Point = Point::new(85, 50); // original: SOURCE_ATTRACT_HALL_DEFENDER_LOGO_POSITION
const ATTRACT_HALL_TODAYS_TABLE_SCREEN: u16 = 0x1886; // original: SOURCE_ATTRACT_HALL_TODAYS_TABLE_SCREEN
const ATTRACT_HALL_ALL_TIME_TABLE_SCREEN: u16 = 0x5986; // original: SOURCE_ATTRACT_HALL_ALL_TIME_TABLE_SCREEN
const ATTRACT_HALL_TABLE_ROW_STEP: u8 = 0x0A; // original: SOURCE_ATTRACT_HALL_TABLE_ROW_STEP
const ATTRACT_HALL_TABLE_INITIALS_OFFSET: u8 = 0x05; // original: SOURCE_ATTRACT_HALL_TABLE_INITIALS_OFFSET
const ATTRACT_HALL_TABLE_SCORE_OFFSET: u8 = 0x13; // original: SOURCE_ATTRACT_HALL_TABLE_SCORE_OFFSET
const ATTRACT_HALL_TABLE_VISUAL_OFFSET: Point = Point::new(-11, -6); // original: SOURCE_ATTRACT_HALL_TABLE_VISUAL_OFFSET
const ATTRACT_HALL_SCORE_TEXT_LEN: usize = 6; // original: SOURCE_ATTRACT_HALL_SCORE_TEXT_LEN
const ATTRACT_SCORING_VISUAL_OFFSET: Point = Point::new(-11, -7); // original: SOURCE_ATTRACT_SCORING_VISUAL_OFFSET
const ATTRACT_INSTRUCTION_TEXT_LINES: [(&str, u16); 7] = [
    // original: SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES
    ("SCANV", 0x4330),
    ("LANDV", 0x1C70),
    ("MUTV", 0x3C70),
    ("BAITV", 0x5F70),
    ("BOMBV", 0x1CA8),
    ("SWRMPV", 0x40A8),
    ("SWARMV", 0x5CA8),
];
const FINAL_GAME_OVER_SCREEN_ADDRESS: u16 = 0x3E80; // original: SOURCE_FINAL_GAME_OVER_SCREEN
const PLAYER_START_PROMPT_SCREEN_ADDRESS: u16 = 0x3C80; // original: SOURCE_PLAYER_START_PROMPT_SCREEN
const PLAYER_SWITCH_LABEL_SCREEN_ADDRESS: u16 = 0x3C78; // original: SOURCE_PLAYER_SWITCH_LABEL_SCREEN
const PLAYER_SWITCH_GAME_OVER_SCREEN_ADDRESS: u16 = 0x3E88; // original: SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN
const TOP_DISPLAY_BORDER_SEGMENTS: [(u16, [f32; 2]); 6] = [
    // original: SOURCE_TOP_DISPLAY_BORDER_SEGMENTS
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
const ATTRACT_SCORING_PROTECTED_DEMO_STEP_OFFSET: u16 = 0;
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
const ATTRACT_SCORING_LEGEND_ORIGIN_X16: i32 = 0x1F00; // original: ATTRACT_SCORING_LEGEND_SOURCE_X16
const ATTRACT_SCORING_LEGEND_ORIGIN_START_Y16: i32 = 0xA000; // original: ATTRACT_SCORING_LEGEND_SOURCE_START_Y16
const OBJECT_IMAGES_TSV: &str = include_str!("../assets/red-label/object-images.tsv"); // original: SOURCE_OBJECT_IMAGES_TSV
const NORMAL_PALETTE_BYTES: [u8; 16] = [
    // original: SOURCE_NORMAL_PALETTE_BYTES
    0x00, 0x00, 0x07, 0x28, 0x2F, 0x81, 0xA4, 0x15, 0xC7, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const LASER_BYTE_PIXELS: i32 = 2; // original: SOURCE_LASER_BYTE_PIXELS
const LASER_BODY_BYTE: u8 = 0x11; // original: SOURCE_LASER_BODY_BYTE
const LASER_TIP_BYTE: u8 = 0x99; // original: SOURCE_LASER_TIP_BYTE
const LASER_BODY_CELLS: i32 = 4; // original: SOURCE_LASER_BODY_CELLS
const LASER_BODY_TINT: Color = Color::from_rgba(0x00, 0xB8, 0xFF, 0xFF); // original: SOURCE_LASER_BODY_TINT
const LASER_TIP_TINT: Color = Color::from_rgba(0x8F, 0xE8, 0xFF, 0xFF); // original: SOURCE_LASER_TIP_TINT
const LASER_FIZZLE_TINT: Color = Color::from_rgba(0x00, 0x78, 0xD8, 0xFF); // original: SOURCE_LASER_FIZZLE_TINT
const WILLIAMS_RED_GREEN_LEVELS: [u8; 8] = [0, 38, 81, 118, 137, 174, 217, 255]; // original: SOURCE_WILLIAMS_RED_GREEN_LEVELS
const WILLIAMS_BLUE_LEVELS: [u8; 4] = [0, 95, 160, 255]; // original: SOURCE_WILLIAMS_BLUE_LEVELS
const TERRAIN_DATA_TSV: &str = include_str!("../assets/red-label/terrain-data.tsv"); // original: SOURCE_TERRAIN_DATA_TSV
const MAIN_TERRAIN_RECORD_LABEL: &str = "MTERR"; // original: SOURCE_TERRAIN_MTERR_LABEL
const MAIN_TERRAIN_RECORD_ADDRESS: u16 = 0xCD67; // original: SOURCE_TERRAIN_MTERR_ADDRESS
const MAIN_TERRAIN_RECORD_BYTE_COUNT: usize = 0x180; // original: SOURCE_TERRAIN_MTERR_BYTES
const SCANNER_TERRAIN_RECORDS: usize = 0x40; // original: SOURCE_SCANNER_TERRAIN_RECORDS
const SCANNER_MINI_TERRAIN_RECORDS: usize = MAIN_TERRAIN_RECORD_BYTE_COUNT / 3; // original: SOURCE_SCANNER_MINI_TERRAIN_RECORDS
const SCANNER_OBJECT_BASE_SCREEN: u16 = 0x3008; // original: SOURCE_SCANNER_OBJECT_BASE_SCREEN
const SCANNER_SCAN_CENTER_OFFSET: u16 = 0x6D40; // original: SOURCE_SCANNER_SCAN_CENTER_OFFSET
const DEFENDER_WORDMARK_START_STEP: u64 = ATTRACT_DEFENDER_WORDMARK_START_STEP;
const DEFENDER_WORDMARK_SLOTS: u16 = 15;
const DEFENDER_WORDMARK_ROW_PAIRS: u16 = 6;
const ATTRACT_DEFENDER_APPEARANCE_FINAL_TICK: u8 = 0x2E; // original: SOURCE_ATTRACT_DEFENDER_APPEARANCE_FINAL_TICK
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
const SCORE_POPUP_500_COLOR_CYCLE: [Color; 3] = [
    Color::from_rgba(0xFF, 0x50, 0x50, 0xFF),
    Color::from_rgba(0xFF, 0xBC, 0x00, 0xFF),
    Color::from_rgba(0x28, 0x38, 0xDC, 0xFF),
];
const HUMAN_GROUND_Y: i16 = 214;
const HUMAN_FALL_ACCELERATION: i16 = 1;
const HUMAN_MAX_FALL_SPEED: i16 = 8;
const HUMAN_SAFE_LANDING_SPEED: i16 = 3;
const HUMAN_CARRIED_OFFSET_Y: i16 = 8;
const ASTRONAUT_RESTORE_Y: u8 = 0xE0; // original: SOURCE_ASTRO_RESTORE_Y
const ASTRONAUT_TARGET_CURSOR_ENTRY_COUNT: usize = 16; // original: SOURCE_ASTRONAUT_TARGET_CURSOR_ENTRY_COUNT
const ASTRONAUT_PROCESS_SLEEP_TICKS: u8 = 2; // original: SOURCE_ASTRONAUT_PROCESS_SLEEP_TICKS
const HUMAN_TURN_SEED_MAX: u8 = 8; // original: SOURCE_HUMAN_TURN_SEED_MAX
const HUMAN_LEFT_TARGET_Y_OFFSET: u8 = 4; // original: SOURCE_HUMAN_LEFT_TARGET_Y_OFFSET
const HUMAN_RIGHT_TARGET_Y_OFFSET: u8 = 15; // original: SOURCE_HUMAN_RIGHT_TARGET_Y_OFFSET
const HUMAN_MAX_TARGET_Y: u8 = 0xE8; // original: SOURCE_HUMAN_MAX_TARGET_Y
const HUMAN_LEFT_X_VELOCITY: u16 = 0xFFE0; // original: SOURCE_HUMAN_LEFT_X_VELOCITY
const HUMAN_RIGHT_X_VELOCITY: u16 = 0x0020; // original: SOURCE_HUMAN_RIGHT_X_VELOCITY
const INITIAL_POD_X_SPEED: u8 = 0x20; // original: SOURCE_INITIAL_POD_X_SPEED
const BOMBER_SQUAD_SIZE: usize = 4; // original: SOURCE_BOMBER_SQUAD_SIZE
const POD_SWARMER_REQUEST_LIMIT: usize = 6; // original: SOURCE_POD_SWARMER_REQUEST_LIMIT
const ACTIVE_SWARMER_LIMIT: usize = 20; // original: SOURCE_ACTIVE_SWARMER_LIMIT
const ACTIVE_BAITER_LIMIT: usize = 12; // original: SOURCE_ACTIVE_BAITER_LIMIT
const MINI_SWARMER_LOOP_SLEEP_TICKS: u8 = 3; // original: SOURCE_MINI_SWARMER_LOOP_SLEEP_TICKS
const MINI_SWARMER_MAX_Y_VELOCITY: u16 = 0x0200; // original: SOURCE_MINI_SWARMER_MAX_Y_VELOCITY
const MINI_SWARMER_MIN_Y_VELOCITY: u16 = 0xFE00; // original: SOURCE_MINI_SWARMER_MIN_Y_VELOCITY
const MINI_SWARMER_TURN_WINDOW: u16 = 300 * 32; // original: SOURCE_MINI_SWARMER_TURN_WINDOW
const MINI_SWARMER_TURN_WINDOW_HALF: u16 = 150 * 32; // original: SOURCE_MINI_SWARMER_TURN_WINDOW_HALF
const MINI_SWARMER_RESTORE_X_LOW: u8 = 0x07; // original: SOURCE_MINI_SWARMER_RESTORE_X_LOW
const BAITER_INITIAL_SHOT_TIMER: u8 = 8; // original: SOURCE_BAITER_INITIAL_SHOT_TIMER
const BAITER_LOOP_SLEEP_TICKS: u8 = 6; // original: SOURCE_BAITER_LOOP_SLEEP_TICKS
const BAITER_X_SEEK_SPEED: u8 = 0x40; // original: SOURCE_BAITER_X_SEEK_SPEED
const BAITER_Y_SEEK_BYTE: u8 = 0x01; // original: SOURCE_BAITER_Y_SEEK_BYTE
const BAITER_X_SEEK_WINDOW_HALF_PIXELS: i16 = 20; // original: SOURCE_BAITER_X_SEEK_WINDOW_HALF_PIXELS
const BAITER_Y_SEEK_WINDOW_HALF_PIXELS: i16 = 10; // original: SOURCE_BAITER_Y_SEEK_WINDOW_HALF_PIXELS
const BAITER_PICTURE_FRAME_COUNT: u8 = 3; // original: SOURCE_BAITER_PICTURE_FRAME_COUNT
const ACTOR_BAITER_TIMER_PACING_STEPS: u8 = 15;
const BOMBER_LOOP_SLEEP_TICKS: u8 = 1; // original: SOURCE_BOMBER_LOOP_SLEEP_TICKS
const BOMBER_PICTURE_FRAME_COUNT: u8 = 4; // original: SOURCE_BOMBER_PICTURE_FRAME_COUNT
const BOMBER_CRUISE_ALTITUDE: i16 = 0x50; // original: SOURCE_BOMBER_CRUISE_ALTITUDE
const BOMBER_MIN_CRUISE_ALTITUDE: i16 = 0x40; // original: SOURCE_BOMBER_MIN_CRUISE_ALTITUDE
const BOMBER_MAX_CRUISE_ALTITUDE: i16 = 0x68; // original: SOURCE_BOMBER_MAX_CRUISE_ALTITUDE
const BOMBER_CRUISE_WINDOW_HALF_PIXELS: i16 = 0x10; // original: SOURCE_BOMBER_CRUISE_WINDOW_HALF_PIXELS
const ACTIVE_BOMBER_BOMB_LIMIT: usize = 10; // original: SOURCE_ACTIVE_BOMBER_BOMB_LIMIT
const MAX_ACTIVE_WAVE_ENEMIES: usize = 5; // original: SOURCE_MAX_ACTIVE_WAVE_ENEMIES
const START_HUMAN_COUNT: u8 = 10; // original: SOURCE_START_HUMAN_COUNT
const TARGET_LIST_ENTRY_COUNT: usize = 32; // original: SOURCE_TARGET_LIST_ENTRY_COUNT
const STATUS_SCORE_POSITION: Point = Point::new(8, 6);
const STATUS_HIGH_SCORE_POSITION: Point = Point::new(94, 6);
const STATUS_PLAYER_TWO_SCORE_POSITION: Point = Point::new(208, 6);
const STATUS_WAVE_POSITION: Point = Point::new(8, 32);
const STATUS_CREDITS_POSITION: Point = Point::new(176, 226);
const CREDITS_MESSAGE_LABEL: &str = "CREDV"; // original: SOURCE_CREDITS_MESSAGE_LABEL
const PRESENTS_MESSAGE_LABEL: &str = "ELECV"; // original: SOURCE_PRESENTS_MESSAGE_LABEL
const ATTRACT_PRESENTS_ELECTRONICS_SCREEN: u16 = 0x3258; // original: SOURCE_ATTRACT_PRESENTS_ELECTRONICS_SCREEN
const ACTOR_HUD_SCORE_DIGIT_DISPLAY_COUNT: usize = 6;
const ACTOR_HUD_SCORE_DISPLAY_MAX: u32 = 999_999;
const ACTOR_HUD_PLAYER_ONE_SCORE_ORIGIN: [f32; 2] = [18.0, 21.0];
const ACTOR_HUD_PLAYER_TWO_SCORE_ORIGIN: [f32; 2] = [214.0, 21.0];
const ACTOR_HUD_SCORE_DIGIT_STEP: [f32; 2] = [8.0, 0.0];
const ACTOR_HUD_SCORE_DIGIT_SIZE: [f32; 2] = [6.0, 8.0];
const ACTOR_HUD_PLAYER_LIFE_STOCK_DISPLAY_LIMIT: u8 = 5;
const ACTOR_HUD_SMART_BOMB_STOCK_DISPLAY_LIMIT: u8 = 3;
const ACTOR_HUD_PLAYER_ONE_LIFE_STOCK_ORIGIN: [f32; 2] = [18.0, 13.0];
const ACTOR_HUD_PLAYER_TWO_LIFE_STOCK_ORIGIN: [f32; 2] = [214.0, 13.0];
const ACTOR_HUD_PLAYER_LIFE_STOCK_STEP: [f32; 2] = [12.0, 0.0];
const ACTOR_HUD_PLAYER_LIFE_STOCK_SIZE: [f32; 2] = [10.0, 4.0];
const ACTOR_HUD_PLAYER_ONE_SMART_BOMB_STOCK_ORIGIN: [f32; 2] = [70.0, 20.0];
const ACTOR_HUD_PLAYER_TWO_SMART_BOMB_STOCK_ORIGIN: [f32; 2] = [266.0, 20.0];
const ACTOR_HUD_SMART_BOMB_STOCK_STEP: [f32; 2] = [0.0, 4.0];
const ACTOR_HUD_SMART_BOMB_STOCK_SIZE: [f32; 2] = [6.0, 3.0];
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
const MUTANT_LOOP_SLEEP_TICKS: u8 = 2; // original: SOURCE_MUTANT_LOOP_SLEEP_TICKS
const MUTANT_X_DISTANCE_OFFSET: u16 = 380; // original: SOURCE_MUTANT_X_DISTANCE_OFFSET
const MUTANT_CLOSE_X_WINDOW: u16 = 0x0700; // original: SOURCE_MUTANT_CLOSE_X_WINDOW
const MUTANT_VERTICAL_WINDOW: u8 = 8; // original: SOURCE_MUTANT_VERTICAL_WINDOW
const FIRST_WAVE_RESCUE_AIM_PLAYER_MIN_Y: i16 = 0xA0; // original: SOURCE_FIRST_WAVE_RESCUE_AIM_PLAYER_MIN_Y
const TARGET6_MUTANT_CONVERSION_X_CORRECTION: u16 = 0x0120; // original: SOURCE_TARGET6_MUTANT_CONVERSION_X_CORRECTION
const TARGET6_MUTANT_DEFERRED_SHOT_TIMER: u8 = 5; // original: SOURCE_TARGET6_MUTANT_DEFERRED_SHOT_TIMER
const TARGET6_MUTANT_POST_SHOT_TIMER: u8 = 0x2C; // original: SOURCE_TARGET6_MUTANT_POST_SHOT_TIMER
const TARGET6_MUTANT_FIRE2524_PENDING_SHOT_TIMER: u8 = 0xFE; // original: SOURCE_TARGET6_MUTANT_FIRE2524_PENDING_SHOT_TIMER
const TARGET6_MUTANT_DIVE_ENTRY_RAW: (u16, u16) = (0x037C, 0x3380); // original: SOURCE_TARGET6_MUTANT_DIVE_ENTRY_RAW
const TARGET6_MUTANT_DIVE_FIRST_SHOT_RAW: (u16, u16) = (0x088C, 0x61B0); // original: SOURCE_TARGET6_MUTANT_DIVE_FIRST_SHOT_RAW
const TARGET6_MUTANT_DIVE_SECOND_SHOT_RAW: (u16, u16) = (0x07FC, 0x7800); // original: SOURCE_TARGET6_MUTANT_DIVE_SECOND_SHOT_RAW
const TARGET6_MUTANT_FIRE2524_FIRST_SHOT_RAW: (u16, u16) = (0x082C, 0x5160); // original: SOURCE_TARGET6_MUTANT_FIRE2524_FIRST_SHOT_RAW
const TARGET6_MUTANT_FIRE2524_SECOND_SHOT_RAW: (u16, u16) = (0x07FC, 0x8150); // original: SOURCE_TARGET6_MUTANT_FIRE2524_SECOND_SHOT_RAW
const TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MIN: u16 = 0xA400; // original: SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MIN
const TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MAX: u16 = 0xA600; // original: SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_RAW_Y_MAX
const TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_TOP_LEFT: Point = Point::new(0x20, 0xA2); // original: SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_TOP_LEFT
const TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_CENTER: Point = Point::new(0x21, 0xA9); // original: SOURCE_TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_CENTER
const TARGET6_MUTANT_VISUAL_X_CORRECTION: u16 = 0x0168; // original: SOURCE_TARGET6_MUTANT_VISUAL_X_CORRECTION
const OBJECT_ACTIVE_LEFT_MARGIN: u16 = 100 * 32; // original: SOURCE_OBJECT_ACTIVE_LEFT_BUFFER
const OBJECT_ACTIVE_WORLD_WIDTH: u16 = 500 * 32; // original: SOURCE_OBJECT_ACTIVE_WINDOW
const OBJECT_WORLD_TO_SCREEN_SHIFT: u8 = 6; // original: SOURCE_OBJECT_SCREEN_X_SHIFT
const OBJECT_VISIBLE_SCREEN_WIDTH: u16 = 292; // original: SOURCE_OBJECT_VISIBLE_WIDTH
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
const ACTOR_ATTRACT_SCRIPT: &str = include_str!("../assets/red-label/actor-attract.script"); // original: ACTOR_RED_LABEL_ATTRACT_SCRIPT
const ACTOR_BEHAVIOR_SCRIPT: &str = include_str!("../assets/red-label/actor-behavior.script"); // original: ACTOR_RED_LABEL_BEHAVIOR_SCRIPT
const ACTOR_WAVE_SCRIPT: &str = include_str!("../assets/red-label/actor-waves.script"); // original: ACTOR_RED_LABEL_WAVE_SCRIPT
const ACTOR_WAVE_TABLE_TSV: &str = include_str!("../assets/red-label/wave-table.tsv"); // original: ACTOR_SOURCE_WAVE_TABLE_TSV
const ACTOR_HIGH_SCORES_TSV: &str = include_str!("../assets/red-label/high-scores.tsv"); // original: ACTOR_SOURCE_HIGH_SCORES_TSV
const ACTOR_WAVE_TABLE_HEADER: &str =
    "key\tceiling\tfloor\tintra_delta\tinter_delta\twave1\twave2\twave3\twave4"; // original: ACTOR_SOURCE_WAVE_TABLE_HEADER
const ACTOR_DEFAULT_DIFFICULTY_INITIAL: u8 = 5; // original: ACTOR_SOURCE_DEFAULT_DIFFICULTY_INITIAL
const ACTOR_DEFAULT_DIFFICULTY_CEILING: u8 = 15; // original: ACTOR_SOURCE_DEFAULT_DIFFICULTY_CEILING
const ACTOR_DATA_BACKED_WAVES: u16 = 16; // original: ACTOR_SOURCE_BACKED_WAVES
const ACTOR_FIRST_WAVE_HUMAN_SPAWNS: [ActorHumanSpawn; 10] = [
    // original: ACTOR_SOURCE_FIRST_WAVE_HUMAN_SPAWNS
    ActorHumanSpawn::from_first_wave_record(
        0,
        FirstWaveHumanSpawnRecord {
            world_x: 0x18C3,
            world_y: 0xE000,
            picture_frame: 2,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        1,
        FirstWaveHumanSpawnRecord {
            world_x: 0x1C81,
            world_y: 0xE100,
            picture_frame: 3,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        2,
        FirstWaveHumanSpawnRecord {
            world_x: 0x4E30,
            world_y: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        3,
        FirstWaveHumanSpawnRecord {
            world_x: 0x5718,
            world_y: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        4,
        FirstWaveHumanSpawnRecord {
            world_x: 0x9B8C,
            world_y: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        5,
        FirstWaveHumanSpawnRecord {
            world_x: 0x9DC6,
            world_y: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        6,
        FirstWaveHumanSpawnRecord {
            world_x: 0xCEE3,
            world_y: 0xE000,
            picture_frame: 2,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        7,
        FirstWaveHumanSpawnRecord {
            world_x: 0xD771,
            world_y: 0xE000,
            picture_frame: 2,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        8,
        FirstWaveHumanSpawnRecord {
            world_x: 0xD2B8,
            world_y: 0xE000,
            picture_frame: 0,
        },
    ),
    ActorHumanSpawn::from_first_wave_record(
        9,
        FirstWaveHumanSpawnRecord {
            world_x: 0xE8DC,
            world_y: 0xE000,
            picture_frame: 0,
        },
    ),
];
const ACTOR_FIRST_WAVE_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    // original: ACTOR_SOURCE_FIRST_WAVE_LANDER_SPAWNS
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xFB33,
        world_y: 0x2CE0,
        x_velocity: 0xFFDE,
        y_velocity: 0x0070,
        shot_timer: 0x27,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(1),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x3F4A,
        world_y: 0x2CE0,
        x_velocity: 0xFFEE,
        y_velocity: 0x0070,
        shot_timer: 0x3B,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(2),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x67FF,
        world_y: 0x2C70,
        x_velocity: 0x0012,
        y_velocity: 0x0070,
        shot_timer: 0x23,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(3),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x0D11,
        world_y: 0x2C70,
        x_velocity: 0x0014,
        y_velocity: 0x0070,
        shot_timer: 0x3C,
        sleep_ticks: 0x04,
        picture_frame: 0,
        target_human_index: Some(4),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x41B9,
        world_y: 0x2C70,
        x_velocity: 0x001A,
        y_velocity: 0x0070,
        shot_timer: 0x25,
        sleep_ticks: 0x04,
        picture_frame: 1,
        target_human_index: Some(5),
    }),
];
const ACTOR_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    // original: ACTOR_SOURCE_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x689A,
        world_y: 0x2C70,
        x_velocity: 0x001E,
        y_velocity: 0x0070,
        shot_timer: 0x10,
        sleep_ticks: 0,
        picture_frame: 1,
        target_human_index: Some(7),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x43D3,
        world_y: 0x2C70,
        x_velocity: 0xFFEC,
        y_velocity: 0x0070,
        shot_timer: 0x3A,
        sleep_ticks: 0,
        picture_frame: 1,
        target_human_index: Some(9),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x1F51,
        world_y: 0x2C70,
        x_velocity: 0x0014,
        y_velocity: 0x0070,
        shot_timer: 0x13,
        sleep_ticks: 0,
        picture_frame: 0,
        target_human_index: Some(8),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xFA03,
        world_y: 0x2C70,
        x_velocity: 0x0016,
        y_velocity: 0x0070,
        shot_timer: 0x26,
        sleep_ticks: 0,
        picture_frame: 1,
        target_human_index: Some(7),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xCF34,
        world_y: 0x2CE0,
        x_velocity: 0,
        y_velocity: 0,
        shot_timer: 0x34,
        sleep_ticks: 1,
        picture_frame: 0,
        target_human_index: Some(6),
    }),
];
const ACTOR_FIRST_WAVE_REFILL_LANDER_SPAWNS: [ActorLanderSpawn; 5] = [
    // original: ACTOR_SOURCE_FIRST_WAVE_REFILL_LANDER_SPAWNS
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xBC29,
        world_y: 0x2CFD,
        x_velocity: 0x001C,
        y_velocity: 0x0090,
        shot_timer: 0x36,
        sleep_ticks: 6,
        picture_frame: 1,
        target_human_index: Some(7),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0xE14C,
        world_y: 0x2CAE,
        x_velocity: 0x000E,
        y_velocity: 0x0090,
        shot_timer: 0x2F,
        sleep_ticks: 0,
        picture_frame: 0,
        target_human_index: Some(4),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x0A63,
        world_y: 0x2CF0,
        x_velocity: 0xFFF4,
        y_velocity: 0x0090,
        shot_timer: 0x23,
        sleep_ticks: 1,
        picture_frame: 0,
        target_human_index: Some(3),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x531B,
        world_y: 0x2CC0,
        x_velocity: 0xFFF6,
        y_velocity: 0x0090,
        shot_timer: 0x30,
        sleep_ticks: 1,
        picture_frame: 0,
        target_human_index: Some(2),
    }),
    ActorLanderSpawn::from_first_wave_record(FirstWaveLanderSpawnRecord {
        world_x: 0x98D9,
        world_y: 0x2CB8,
        x_velocity: 0x001A,
        y_velocity: 0x0090,
        shot_timer: 0x1F,
        sleep_ticks: 1,
        picture_frame: 0,
        target_human_index: Some(1),
    }),
];
const ACTOR_WAVE_ACTIVE_SPAWN_SLOTS: [Point; MAX_ACTIVE_WAVE_ENEMIES] = [
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
            KeyboardKey::Character('2') if active => step.input.start_two = true,
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
            KeyboardKey::LeftShift | KeyboardKey::RightShift if active => {
                step.input.reverse = true;
            }
            KeyboardKey::Character(' ') => set_held_control(
                &mut self.held.thrust,
                event.transition,
                &mut step.input.thrust,
            ),
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
        player_hyperspace_death_lseed: PLAYER_HYPERSPACE_DEATH_LOW_SEED,
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
        Self::parse_text(ACTOR_BEHAVIOR_SCRIPT)
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
