use crate::attract_rom::{WILLIAMS_TRACE_POINT_COUNT, attract_rom};
use crate::audio::SoundCue;
use crate::game::{Entity, EntityKind, EntityState, HorizontalDirection, Status, World};
use crate::high_scores::{HighScoreEntry, HighScoreTable};

const ATTRACT_SCORE_CARD: [(&str, u32); 6] = [
    ("LANDER", 150),
    ("MUTANT", 150),
    ("BAITER", 200),
    ("BOMBER", 250),
    ("POD", 1000),
    ("SWARMER", 150),
];

const ATTRACT_WORLD_WIDTH: usize = 64;
const ATTRACT_WORLD_HEIGHT: usize = 18;
const ATTRACT_PLAYER_Y: i32 = 7;
const ATTRACT_PLAYER_START_X: i32 = 8;
const ATTRACT_ROM_HZ: u64 = 60;
const TITLE_TRACE_SLEEP_TICKS: u64 = 2;
const TITLE_PRESENTS_TICKS: u64 = 5;
const TITLE_DEFENDER_DELAY_TICKS: u64 = 0x30;
const TITLE_DEFENDER_APPEAR_TICKS: u8 = 0x2E;
const TITLE_PRE_COPYRIGHT_HOLD_TICKS: u64 = 0x28;
const TITLE_COPYRIGHT_HOLD_TICKS: u64 = 600;
const HALL_OF_FAME_HOLD_TICKS: u64 = 600;
const RESCUE_DESCENT_TICKS: u16 = 0xE6;
const RESCUE_ASCENT_TICKS: u16 = 0xA0;
const RESCUE_LASER_TICKS: u16 = 0x15;
const RESCUE_FALL_TICKS: u16 = 0x2D * 2;
const RESCUE_SCORE_TICKS: u16 = 0x50;
const RESCUE_RETURN_TICKS: u16 = 0x60;
const LEGEND_APPROACH_TICKS: u16 = 0x5F;
const LEGEND_LASER_TICKS: u16 = 0x17;
const LEGEND_TEXT_TICKS: u16 = 0x20;
const LEGEND_SETTLE_TICKS: u16 = 0x20;
const LEGEND_ENTRY_TICKS: u16 =
    LEGEND_APPROACH_TICKS + LEGEND_LASER_TICKS + LEGEND_TEXT_TICKS + LEGEND_SETTLE_TICKS;
const LEGEND_HOLD_TICKS: u16 = 0xFF + 0xFF;
const ATTRACT_TABLE_XS: [i32; 6] = [0x0900, 0x1100, 0x1980, 0x0960, 0x1160, 0x19E0];
const ATTRACT_TABLE_YS: [i32; 6] = [0x6000, 0x6000, 0x6200, 0x9800, 0x9800, 0x9A00];
const ATTRACT_PLAYER_X16: i32 = 0x0800;
const ATTRACT_PLAYER_Y16: i32 = 0x5000;
const ATTRACT_HUMAN_X16: i32 = 0x1E00;
const ATTRACT_HUMAN_Y16: i32 = 0xDB00;
const ATTRACT_LANDER_X16: i32 = 0x1DA0;
const ATTRACT_LANDER_Y16: i32 = 0x4000;
const ATTRACT_SCORE_BONUS_X16: i32 = 0x1DFF;
const ATTRACT_SCORE_BONUS_Y16: i32 = 0x9000;
const ATTRACT_SCORE_BONUS_DROP_X16: i32 = 0x1C00;
const ATTRACT_SCORE_BONUS_DROP_Y16: i32 = 0xE000;
const ATTRACT_CAUGHT_HUMAN_X16: i32 = 0x1E80;
const ATTRACT_CAUGHT_HUMAN_Y16: i32 = 0xA2E0;
const ATTRACT_GROUNDED_HUMAN_Y16: i32 = 0xDEE0;
const ATTRACT_RESCUE_SHIP_XV16: i32 = 0x0040;
const ATTRACT_RESCUE_SHIP_YV16: i32 = 0x00D4;
const ATTRACT_RESCUE_HUMAN_ACCEL16: i32 = 0x0008;
const ATTRACT_RESCUE_DROP_YV16: i32 = 0x00C0;
const ATTRACT_RESCUE_RETURN_XV16: i32 = -0x0040;
const ATTRACT_RESCUE_RETURN_YV16: i32 = -0x0180;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractFrame {
    pub world: World,
    pub objects: Vec<AttractObject>,
    pub revealed_score_entries: usize,
    pub bonus_text: Option<AttractBonusText>,
    pub player_facing: HorizontalDirection,
    pub animation_tick: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttractObject {
    pub kind: EntityKind,
    pub x16: i32,
    pub y16: i32,
    pub state: EntityState,
    pub facing: HorizontalDirection,
    pub visual: AttractVisual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttractVisual {
    #[default]
    Sprite,
    Explosion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttractBonusText {
    pub text: &'static str,
    pub x16: i32,
    pub y16: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneKind {
    Logo,
    Attract,
    HighScore,
}

impl SceneKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "logo" => Some(Self::Logo),
            "attract" => Some(Self::Attract),
            "high-score" => Some(Self::HighScore),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    pub kind: SceneKind,
    pub lines: Vec<String>,
}

impl Scene {
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttractBeat {
    pub kind: SceneKind,
    pub cue: Option<SoundCue>,
    pub hold_ms: u64,
    pub world_steps: usize,
    pub revealed_score_entries: usize,
    pub demo_tick: Option<u16>,
    pub palette_phase: usize,
    pub logo_trace_points: usize,
    pub logo_show_title_text: bool,
    pub logo_show_full_defender: bool,
    pub logo_defender_appear_tick: Option<u8>,
    pub logo_show_copyright: bool,
}

impl AttractBeat {
    pub fn scene(self) -> Scene {
        self.scene_with_tables(&HighScoreTable::default(), &HighScoreTable::default())
    }

    pub fn scene_with_tables(self, todays: &HighScoreTable, all_time: &HighScoreTable) -> Scene {
        match self.kind {
            SceneKind::Logo => logo_scene(),
            SceneKind::Attract => {
                let frame = attract_frame_for_beat(self);
                attract_scene(&frame.world, frame.revealed_score_entries)
            }
            SceneKind::HighScore => high_score_scene_with_tables(todays, all_time),
        }
    }
}

pub fn attract_cycle() -> Vec<AttractBeat> {
    let mut beats = Vec::new();
    beats.extend(logo_trace_beats());
    beats.extend(logo_beat(
        ticks_to_ms(TITLE_PRESENTS_TICKS),
        WILLIAMS_TRACE_POINT_COUNT,
        true,
        false,
        None,
        false,
    ));
    beats.extend(logo_beat(
        ticks_to_ms(TITLE_DEFENDER_DELAY_TICKS),
        WILLIAMS_TRACE_POINT_COUNT,
        true,
        false,
        None,
        false,
    ));
    beats.extend(logo_appear_beats());
    beats.extend(logo_beat(
        ticks_to_ms(TITLE_PRE_COPYRIGHT_HOLD_TICKS),
        WILLIAMS_TRACE_POINT_COUNT,
        true,
        true,
        None,
        false,
    ));
    beats.extend(logo_beat(
        ticks_to_ms(TITLE_COPYRIGHT_HOLD_TICKS),
        WILLIAMS_TRACE_POINT_COUNT,
        true,
        true,
        None,
        true,
    ));
    beats.push(high_score_beat(ticks_to_ms(HALL_OF_FAME_HOLD_TICKS)));
    beats.extend(demo_beats());
    beats
}

fn ticks_to_ms(ticks: u64) -> u64 {
    (ticks * 1_000 + (ATTRACT_ROM_HZ / 2)) / ATTRACT_ROM_HZ
}

fn logo_beat(
    hold_ms: u64,
    logo_trace_points: usize,
    logo_show_title_text: bool,
    logo_show_full_defender: bool,
    logo_defender_appear_tick: Option<u8>,
    logo_show_copyright: bool,
) -> Vec<AttractBeat> {
    vec![AttractBeat {
        kind: SceneKind::Logo,
        cue: Some(SoundCue::LogoFanfare),
        hold_ms,
        world_steps: 0,
        revealed_score_entries: 0,
        demo_tick: None,
        palette_phase: 0,
        logo_trace_points,
        logo_show_title_text,
        logo_show_full_defender,
        logo_defender_appear_tick,
        logo_show_copyright,
    }]
}

fn logo_appear_beats() -> Vec<AttractBeat> {
    (0..TITLE_DEFENDER_APPEAR_TICKS)
        .map(|tick| AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: ticks_to_ms(1),
            world_steps: 0,
            revealed_score_entries: 0,
            demo_tick: None,
            palette_phase: 0,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT,
            logo_show_title_text: true,
            logo_show_full_defender: false,
            logo_defender_appear_tick: Some(tick),
            logo_show_copyright: false,
        })
        .collect()
}

fn logo_trace_beats() -> Vec<AttractBeat> {
    let prefixes = attract_rom().williams_point_prefixes();
    let mut beats = Vec::new();
    let mut byte_index = 0usize;

    while byte_index < prefixes.len() {
        let end = (byte_index + 3).min(prefixes.len()) - 1;
        beats.push(AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: ticks_to_ms(TITLE_TRACE_SLEEP_TICKS),
            world_steps: 0,
            revealed_score_entries: 0,
            demo_tick: None,
            palette_phase: 0,
            logo_trace_points: prefixes[end],
            logo_show_title_text: false,
            logo_show_full_defender: false,
            logo_defender_appear_tick: None,
            logo_show_copyright: false,
        });
        byte_index += 3;
    }

    beats
}

fn high_score_beat(hold_ms: u64) -> AttractBeat {
    AttractBeat {
        kind: SceneKind::HighScore,
        cue: Some(SoundCue::HighScoreChime),
        hold_ms,
        world_steps: 0,
        revealed_score_entries: 0,
        demo_tick: None,
        palette_phase: 0,
        logo_trace_points: 0,
        logo_show_title_text: false,
        logo_show_full_defender: false,
        logo_defender_appear_tick: None,
        logo_show_copyright: false,
    }
}

fn demo_beats() -> Vec<AttractBeat> {
    (0..demo_total_ticks())
        .map(|tick| AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(demo_cue_for_tick(tick)),
            hold_ms: ticks_to_ms(1),
            world_steps: 0,
            revealed_score_entries: 0,
            demo_tick: Some(tick),
            palette_phase: 0,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_show_full_defender: false,
            logo_defender_appear_tick: None,
            logo_show_copyright: false,
        })
        .collect()
}

fn demo_total_ticks() -> u16 {
    RESCUE_DESCENT_TICKS
        + RESCUE_ASCENT_TICKS
        + RESCUE_LASER_TICKS
        + RESCUE_FALL_TICKS
        + RESCUE_SCORE_TICKS
        + RESCUE_RETURN_TICKS
        + LEGEND_ENTRY_TICKS * ATTRACT_SCORE_CARD.len() as u16
        + LEGEND_HOLD_TICKS
}

fn demo_cue_for_tick(tick: u16) -> SoundCue {
    let rescue_tick_1 = RESCUE_DESCENT_TICKS;
    let rescue_tick_2 = rescue_tick_1 + RESCUE_ASCENT_TICKS;
    let rescue_tick_3 = rescue_tick_2 + RESCUE_LASER_TICKS;
    let rescue_tick_4 = rescue_tick_3 + RESCUE_FALL_TICKS;
    let rescue_tick_5 = rescue_tick_4 + RESCUE_SCORE_TICKS;
    let rescue_tick_6 = rescue_tick_5 + RESCUE_RETURN_TICKS;

    if tick == rescue_tick_1 {
        SoundCue::AttractHum
    } else if tick == rescue_tick_2 {
        SoundCue::PlayerShot
    } else if tick == rescue_tick_3 {
        SoundCue::Explosion
    } else if tick == rescue_tick_4 {
        SoundCue::HumanSaved
    } else if tick >= rescue_tick_6 {
        let local = tick - rescue_tick_6;
        let stage = local % LEGEND_ENTRY_TICKS;
        if stage == 0 {
            SoundCue::EnemySweep
        } else if stage == LEGEND_APPROACH_TICKS {
            SoundCue::PlayerShot
        } else if stage == LEGEND_APPROACH_TICKS + LEGEND_LASER_TICKS {
            SoundCue::Explosion
        } else {
            SoundCue::AttractHum
        }
    } else {
        SoundCue::AttractHum
    }
}

pub fn attract_frame_for_beat(beat: AttractBeat) -> AttractFrame {
    match beat.demo_tick {
        Some(tick) => scripted_attract_frame_for_tick(tick),
        None => {
            let mut world = World::bootstrap();
            for _ in 0..beat.world_steps {
                world.step();
            }
            AttractFrame {
                world,
                objects: Vec::new(),
                revealed_score_entries: beat.revealed_score_entries,
                bonus_text: None,
                player_facing: HorizontalDirection::Right,
                animation_tick: 0,
            }
        }
    }
}

pub fn attract_world_for_beat(beat: AttractBeat) -> (World, usize) {
    let frame = attract_frame_for_beat(beat);
    (frame.world, frame.revealed_score_entries)
}

fn scripted_world(
    score: u32,
    player_x: i32,
    player_y: i32,
    facing: HorizontalDirection,
    mut entities: Vec<Entity>,
) -> World {
    entities.insert(
        0,
        Entity::new(EntityKind::PlayerShip, player_x, player_y, 0, 0),
    );
    let mut world = World::with_entities(
        ATTRACT_WORLD_WIDTH,
        ATTRACT_WORLD_HEIGHT,
        Status {
            score,
            lives: 3,
            wave: 1,
        },
        entities,
    );
    world.set_player_facing(facing);
    world
}

fn scripted_attract_frame_for_tick(tick: u16) -> AttractFrame {
    let rescue_phase_end = RESCUE_DESCENT_TICKS + RESCUE_ASCENT_TICKS + RESCUE_LASER_TICKS;
    let rescue_fall_end = rescue_phase_end + RESCUE_FALL_TICKS;
    let rescue_score_end = rescue_fall_end + RESCUE_SCORE_TICKS;
    let rescue_return_end = rescue_score_end + RESCUE_RETURN_TICKS;
    let mut facing = HorizontalDirection::Right;
    let mut objects = Vec::new();
    let mut bonus_text = None;

    if tick < RESCUE_DESCENT_TICKS {
        objects.push(attract_object(
            EntityKind::Lander,
            ATTRACT_LANDER_X16,
            ATTRACT_LANDER_Y16 + i32::from(tick) * 0x00A0,
        ));
        objects.push(attract_object(
            EntityKind::Human,
            ATTRACT_HUMAN_X16,
            ATTRACT_HUMAN_Y16,
        ));
        objects.push(attract_object(
            EntityKind::PlayerShip,
            ATTRACT_PLAYER_X16,
            ATTRACT_PLAYER_Y16,
        ));
    } else if tick < rescue_phase_end {
        let rise_tick = tick - RESCUE_DESCENT_TICKS;
        let total_rise_tick = rise_tick.min(RESCUE_ASCENT_TICKS + RESCUE_LASER_TICKS);
        let enemy_y = ATTRACT_LANDER_Y16 + i32::from(RESCUE_DESCENT_TICKS) * 0x00A0
            - i32::from(total_rise_tick) * 0x00B0;
        let human_y = ATTRACT_HUMAN_Y16 - i32::from(total_rise_tick) * 0x00B0;
        objects.push(attract_object(
            EntityKind::Lander,
            ATTRACT_LANDER_X16,
            enemy_y,
        ));
        objects.push(attract_object_with_state(
            EntityKind::Human,
            ATTRACT_HUMAN_X16,
            human_y,
            EntityState::Abducted,
        ));
        objects.push(attract_object(
            EntityKind::PlayerShip,
            ATTRACT_PLAYER_X16,
            ATTRACT_PLAYER_Y16,
        ));
        if rise_tick >= RESCUE_ASCENT_TICKS {
            add_laser_column(
                &mut objects,
                ATTRACT_PLAYER_X16,
                ATTRACT_PLAYER_Y16,
                ATTRACT_LANDER_X16,
                enemy_y,
            );
        }
    } else if tick < rescue_fall_end {
        let fall_tick = tick - rescue_phase_end;
        // `AMODE3` / `AMODE4` move the ship toward Eugene while the falling
        // human accelerates every two ticks until the catch frame.
        let (ship_x, ship_y, human_y) = rescue_intercept_state(fall_tick);
        objects.push(attract_object(EntityKind::PlayerShip, ship_x, ship_y));
        if fall_tick < 12 {
            let explosion_y = ATTRACT_LANDER_Y16 + i32::from(RESCUE_DESCENT_TICKS) * 0x00A0
                - i32::from(RESCUE_ASCENT_TICKS + RESCUE_LASER_TICKS) * 0x00B0;
            objects.push(attract_explosion_object(ATTRACT_LANDER_X16, explosion_y));
        }
        objects.push(attract_object_with_state(
            EntityKind::Human,
            ATTRACT_HUMAN_X16,
            human_y,
            EntityState::Falling,
        ));
    } else if tick < rescue_score_end {
        // `AMODE5` teleports the caught human onto the ship's path, spawns the
        // `500`, and drops both ship and human straight toward the terrain.
        let score_tick = tick - rescue_fall_end;
        let (ship_x, ship_y, human_y) = rescue_drop_state(score_tick);
        objects.push(attract_object(EntityKind::PlayerShip, ship_x, ship_y));
        objects.push(attract_object(
            EntityKind::Human,
            ATTRACT_CAUGHT_HUMAN_X16,
            human_y,
        ));
        bonus_text = Some(AttractBonusText {
            text: "500",
            x16: ATTRACT_SCORE_BONUS_X16,
            y16: ATTRACT_SCORE_BONUS_Y16,
        });
    } else if tick < rescue_return_end {
        let return_tick = tick - rescue_score_end;
        facing = HorizontalDirection::Left;
        let (ship_start_x, ship_start_y, _) = rescue_drop_state(RESCUE_SCORE_TICKS);
        objects.push(attract_object_facing(
            EntityKind::PlayerShip,
            ship_start_x + i32::from(return_tick) * ATTRACT_RESCUE_RETURN_XV16,
            ship_start_y + i32::from(return_tick) * ATTRACT_RESCUE_RETURN_YV16,
            HorizontalDirection::Left,
        ));
        objects.push(attract_object(
            EntityKind::Human,
            ATTRACT_CAUGHT_HUMAN_X16,
            ATTRACT_GROUNDED_HUMAN_Y16,
        ));
        bonus_text = Some(AttractBonusText {
            text: "500",
            x16: ATTRACT_SCORE_BONUS_DROP_X16,
            y16: ATTRACT_SCORE_BONUS_DROP_Y16,
        });
    } else {
        let table_tick = tick - rescue_return_end;
        let (drop_ship_x, drop_ship_y, _) = rescue_drop_state(RESCUE_SCORE_TICKS);
        let player_x = drop_ship_x + i32::from(RESCUE_RETURN_TICKS) * ATTRACT_RESCUE_RETURN_XV16;
        let player_y = drop_ship_y + i32::from(RESCUE_RETURN_TICKS) * ATTRACT_RESCUE_RETURN_YV16;
        objects.push(attract_object(EntityKind::PlayerShip, player_x, player_y));
        objects.push(attract_object(
            EntityKind::Human,
            ATTRACT_CAUGHT_HUMAN_X16,
            ATTRACT_GROUNDED_HUMAN_Y16,
        ));
        append_legend_entities(&mut objects, table_tick, player_x, player_y);
    }

    let revealed_score_entries = revealed_score_entries_for_tick(tick);
    let entities: Vec<Entity> = objects
        .iter()
        .map(|object| rom_entity_with_state(object.kind, object.x16, object.y16, object.state))
        .collect();
    AttractFrame {
        world: scripted_world(
            0,
            rom_x_to_world(ATTRACT_PLAYER_START_X << 8),
            ATTRACT_PLAYER_Y,
            facing,
            entities,
        ),
        objects,
        revealed_score_entries,
        bonus_text,
        player_facing: facing,
        animation_tick: u32::from(tick),
    }
}

fn append_legend_entities(
    objects: &mut Vec<AttractObject>,
    table_tick: u16,
    player_x16: i32,
    player_y16: i32,
) {
    let hold_start = LEGEND_ENTRY_TICKS * ATTRACT_SCORE_CARD.len() as u16;
    for index in 0..ATTRACT_SCORE_CARD.len() {
        let entry_start = index as u16 * LEGEND_ENTRY_TICKS;
        let entry_show_tick = entry_start + LEGEND_APPROACH_TICKS + LEGEND_LASER_TICKS;
        if table_tick >= entry_show_tick {
            objects.push(attract_object(
                legend_kind(index),
                ATTRACT_TABLE_XS[index],
                ATTRACT_TABLE_YS[index],
            ));
            continue;
        }

        if table_tick >= entry_start {
            let local = table_tick - entry_start;
            let enemy_y = 0xA000 - i32::from(local.min(entry_show_tick - entry_start)) * 0x00C0;
            if local < LEGEND_APPROACH_TICKS {
                objects.push(attract_object(legend_kind(index), 0x1F00, enemy_y));
            } else if local < LEGEND_APPROACH_TICKS + LEGEND_LASER_TICKS / 2 {
                objects.push(attract_explosion_object(0x1F00, enemy_y));
            } else if local < entry_show_tick - entry_start {
                objects.push(attract_explosion_object(
                    ATTRACT_TABLE_XS[index],
                    ATTRACT_TABLE_YS[index],
                ));
            }
            if local >= LEGEND_APPROACH_TICKS && local < entry_show_tick - entry_start {
                add_laser_column(objects, player_x16, player_y16, 0x1F00, enemy_y);
            }
            break;
        }
    }

    if table_tick >= hold_start {
        for index in 0..ATTRACT_SCORE_CARD.len() {
            objects.push(attract_object(
                legend_kind(index),
                ATTRACT_TABLE_XS[index],
                ATTRACT_TABLE_YS[index],
            ));
        }
    }
}

fn revealed_score_entries_for_tick(tick: u16) -> usize {
    let table_tick = tick.saturating_sub(
        RESCUE_DESCENT_TICKS
            + RESCUE_ASCENT_TICKS
            + RESCUE_LASER_TICKS
            + RESCUE_FALL_TICKS
            + RESCUE_SCORE_TICKS
            + RESCUE_RETURN_TICKS,
    );
    let mut visible = 0;
    for index in 0..ATTRACT_SCORE_CARD.len() {
        let show_tick = index as u16 * LEGEND_ENTRY_TICKS
            + LEGEND_APPROACH_TICKS
            + LEGEND_LASER_TICKS
            + LEGEND_TEXT_TICKS;
        if table_tick >= show_tick {
            visible += 1;
        }
    }
    visible
}

fn add_laser_column(
    objects: &mut Vec<AttractObject>,
    ship_x16: i32,
    ship_y16: i32,
    target_x16: i32,
    target_y16: i32,
) {
    let dx = target_x16 - ship_x16;
    let dy = target_y16 - ship_y16;
    let steps = ((dx.abs().max(dy.abs()) + 0x07FF) / 0x0800).max(1);

    for step in 1..=steps {
        let x16 = ship_x16 + (dx * step) / steps;
        let y16 = ship_y16 + (dy * step) / steps;
        objects.push(attract_object(EntityKind::PlayerShot, x16, y16));
    }
}

fn rescue_intercept_state(fall_tick: u16) -> (i32, i32, i32) {
    let mut ship_x = ATTRACT_PLAYER_X16;
    let mut ship_y = ATTRACT_PLAYER_Y16;
    let mut human_y =
        ATTRACT_HUMAN_Y16 - i32::from(RESCUE_ASCENT_TICKS + RESCUE_LASER_TICKS) * 0x00B0;
    let mut tick_cursor = 0;
    let mut human_velocity = 0;

    for _ in 0..(RESCUE_FALL_TICKS / 2) {
        human_velocity += ATTRACT_RESCUE_HUMAN_ACCEL16;
        for _ in 0..2 {
            if tick_cursor >= fall_tick {
                return (ship_x, ship_y, human_y);
            }
            ship_x += ATTRACT_RESCUE_SHIP_XV16;
            ship_y += ATTRACT_RESCUE_SHIP_YV16;
            human_y += human_velocity;
            tick_cursor += 1;
        }
    }

    (ship_x, ship_y, human_y)
}

fn rescue_drop_state(score_tick: u16) -> (i32, i32, i32) {
    let (ship_x, ship_y, _) = rescue_intercept_state(RESCUE_FALL_TICKS);
    (
        ship_x,
        ship_y + i32::from(score_tick) * ATTRACT_RESCUE_DROP_YV16,
        ATTRACT_CAUGHT_HUMAN_Y16 + i32::from(score_tick) * ATTRACT_RESCUE_DROP_YV16,
    )
}

fn attract_object(kind: EntityKind, x16: i32, y16: i32) -> AttractObject {
    attract_object_with_state(kind, x16, y16, EntityState::Normal)
}

fn attract_object_with_state(
    kind: EntityKind,
    x16: i32,
    y16: i32,
    state: EntityState,
) -> AttractObject {
    AttractObject {
        kind,
        x16,
        y16,
        state,
        facing: HorizontalDirection::Right,
        visual: AttractVisual::Sprite,
    }
}

fn attract_object_facing(
    kind: EntityKind,
    x16: i32,
    y16: i32,
    facing: HorizontalDirection,
) -> AttractObject {
    AttractObject {
        kind,
        x16,
        y16,
        state: EntityState::Normal,
        facing,
        visual: AttractVisual::Sprite,
    }
}

fn attract_explosion_object(x16: i32, y16: i32) -> AttractObject {
    AttractObject {
        kind: EntityKind::Pod,
        x16,
        y16,
        state: EntityState::Normal,
        facing: HorizontalDirection::Right,
        visual: AttractVisual::Explosion,
    }
}

fn rom_entity_with_state(kind: EntityKind, x16: i32, y16: i32, state: EntityState) -> Entity {
    Entity::with_state(kind, rom_x_to_world(x16), rom_y_to_world(y16), 0, 0, state)
}

fn rom_x_to_world(x16: i32) -> i32 {
    ((x16 + 0x80) >> 8).clamp(0, ATTRACT_WORLD_WIDTH as i32 - 1)
}

fn rom_y_to_world(y16: i32) -> i32 {
    ((y16 + 0x0800) >> 12).clamp(1, ATTRACT_WORLD_HEIGHT as i32 - 2)
}

fn legend_kind(index: usize) -> EntityKind {
    match index {
        0 => EntityKind::Lander,
        1 => EntityKind::Mutant,
        2 => EntityKind::Baiter,
        3 => EntityKind::Bomber,
        4 => EntityKind::Pod,
        _ => EntityKind::Swarmer,
    }
}

pub fn scene_for_elapsed_ms(
    elapsed_ms: u64,
    todays: &HighScoreTable,
    all_time: &HighScoreTable,
) -> Scene {
    beat_for_elapsed_ms(elapsed_ms).scene_with_tables(todays, all_time)
}

pub fn beat_for_elapsed_ms(elapsed_ms: u64) -> AttractBeat {
    let cycle = attract_cycle();
    let cycle_ms = cycle.iter().map(|beat| beat.hold_ms).sum::<u64>();
    let mut remaining = if cycle_ms == 0 {
        0
    } else {
        elapsed_ms % cycle_ms
    };

    for beat in cycle.iter().copied() {
        if remaining < beat.hold_ms {
            return beat;
        }
        remaining -= beat.hold_ms;
    }

    cycle[0]
}

pub fn logo_scene() -> Scene {
    Scene {
        kind: SceneKind::Logo,
        lines: vec![
            // `AMODES` builds this first page as one Williams / Electronics Inc. /
            // Presents / Defender / copyright composition.
            String::from("                 WILLIAMS"),
            String::new(),
            String::from("             ELECTRONICS INC."),
            String::new(),
            String::from("                  PRESENTS"),
            String::new(),
            String::from("                  DEFENDER"),
            String::new(),
            String::from("               COPYRIGHT 1980"),
        ],
    }
}

pub fn attract_scene(world: &World, revealed_score_entries: usize) -> Scene {
    let mut lines = vec![String::new()];
    lines.extend(crate::render::render_grid(world));
    lines.push(String::new());
    // `TEXTAB` / `TENT` in `amode1.src` rotate the instruction legend in this
    // order: SCANNER, LANDER, MUTANT, BAITER, BOMBER, POD, SWARMER.
    lines.push(String::from("SCANNER"));
    lines.extend(
        ATTRACT_SCORE_CARD
            .into_iter()
            .take(revealed_score_entries)
            .map(|(name, score)| format!("{name:<8}{score:>8}")),
    );

    Scene {
        kind: SceneKind::Attract,
        lines,
    }
}

pub fn high_score_scene() -> Scene {
    high_score_scene_with_tables(&HighScoreTable::default(), &HighScoreTable::default())
}

pub fn high_score_scene_with_tables(todays: &HighScoreTable, all_time: &HighScoreTable) -> Scene {
    let mut lines = vec![
        String::from("DEFENDER"),
        String::from("HALL OF FAME"),
        String::new(),
        format!("{:<24}{}", "TODAYS GREATEST", "ALL TIME GREATEST"),
        format!("{:<24}{}", " RANK  INITIALS SCORE", " RANK  INITIALS SCORE"),
    ];

    // Red-label `HALDIS` renders the volatile `THSTAB` "TODAYS GREATEST" table on
    // the left and the CMOS-backed `CRHSTD` "ALL TIME GREATEST" table on the right.
    let row_count = todays.entries().len().max(all_time.entries().len());
    for index in 0..row_count {
        let left = compact_score_row(index + 1, todays.entries().get(index));
        let right = compact_score_row(index + 1, all_time.entries().get(index));
        lines.push(format!("{left:<24}{right}"));
    }

    Scene {
        kind: SceneKind::HighScore,
        lines,
    }
}

fn compact_score_row(rank: usize, entry: Option<&HighScoreEntry>) -> String {
    match entry {
        Some(entry) => format!("{rank:>2}. {:<3} {:>6}", entry.initials, entry.score),
        None => format!("{rank:>2}. --- ------"),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        audio::SoundCue,
        game::{EntityKind, World},
        high_scores::HighScoreTable,
    };

    use super::{
        ATTRACT_CAUGHT_HUMAN_X16, ATTRACT_HUMAN_X16, LEGEND_APPROACH_TICKS, LEGEND_LASER_TICKS,
        LEGEND_TEXT_TICKS, RESCUE_ASCENT_TICKS, RESCUE_DESCENT_TICKS, RESCUE_FALL_TICKS,
        RESCUE_LASER_TICKS, RESCUE_RETURN_TICKS, RESCUE_SCORE_TICKS, SceneKind, attract_cycle,
        attract_scene, high_score_scene, logo_scene, revealed_score_entries_for_tick,
        scene_for_elapsed_ms, scripted_attract_frame_for_tick,
    };

    #[test]
    fn parse_scene_kind_recognises_supported_values() {
        assert_eq!(SceneKind::parse("logo"), Some(SceneKind::Logo));
        assert_eq!(SceneKind::parse("attract"), Some(SceneKind::Attract));
        assert_eq!(SceneKind::parse("high-score"), Some(SceneKind::HighScore));
        assert_eq!(SceneKind::parse("unknown"), None);
    }

    #[test]
    fn logo_scene_contains_live_launch_hints() {
        let scene = logo_scene();
        let text = scene.text();

        assert!(text.contains("WILLIAMS"));
        assert!(text.contains("PRESENTS"));
    }

    #[test]
    fn attract_scene_wraps_rendered_world() {
        let scene = attract_scene(&World::bootstrap(), 6);
        let text = scene.text();

        assert!(text.contains("SCANNER"));
        assert!(text.contains("LANDER"));
        assert!(text.contains("SWARMER"));
        assert!(text.contains("THREAT"));
    }

    #[test]
    fn high_score_scene_lists_ranked_scores() {
        let scene = high_score_scene();
        let text = scene.text();

        assert!(text.contains("HALL OF FAME"));
        assert!(text.contains("TODAYS GREATEST"));
        assert!(text.contains("ALL TIME GREATEST"));
        assert!(text.contains("1."));
        assert!(text.contains("21270"));
    }

    #[test]
    fn attract_cycle_covers_logo_attract_and_high_score() {
        let cycle = attract_cycle();

        assert_eq!(cycle[0].kind, SceneKind::Logo);
        assert_eq!(cycle[0].cue, Some(SoundCue::LogoFanfare));
        assert!(cycle.iter().any(|beat| beat.kind == SceneKind::HighScore));
        assert!(cycle.iter().any(|beat| beat.kind == SceneKind::Attract));
        assert_eq!(
            cycle
                .last()
                .expect("attract cycle should not be empty")
                .kind,
            SceneKind::Attract
        );
    }

    #[test]
    fn attract_beat_scene_renders_the_expected_variant() {
        let cycle = attract_cycle();

        assert!(cycle[0].scene().text().contains("WILLIAMS"));
        assert!(
            cycle
                .iter()
                .find(|beat| beat.kind == SceneKind::HighScore)
                .expect("attract cycle should include the hall of fame page")
                .scene()
                .text()
                .contains("HALL OF FAME")
        );
        assert!(
            cycle
                .iter()
                .find(|beat| beat.kind == SceneKind::Attract)
                .expect("attract cycle should include the score-card demo")
                .scene()
                .text()
                .contains("SCANNER")
        );
        assert!(
            cycle
                .last()
                .expect("attract cycle should not be empty")
                .scene()
                .text()
                .contains("SWARMER")
        );
    }

    #[test]
    fn scene_for_elapsed_ms_wraps_across_the_attract_cycle() {
        let scene = scene_for_elapsed_ms(
            4_200,
            &HighScoreTable::default(),
            &HighScoreTable::default(),
        );
        let text = scene.text();

        assert!(
            text.contains("SCANNER") || text.contains("HALL OF FAME") || text.contains("WILLIAMS")
        );
    }

    #[test]
    fn rescue_sequence_switches_to_catch_and_bonus_on_rom_tick_boundary() {
        let rescue_phase_end = RESCUE_DESCENT_TICKS + RESCUE_ASCENT_TICKS + RESCUE_LASER_TICKS;

        let intercept_frame =
            scripted_attract_frame_for_tick(rescue_phase_end + RESCUE_FALL_TICKS - 1);
        let caught_frame = scripted_attract_frame_for_tick(rescue_phase_end + RESCUE_FALL_TICKS);

        assert_eq!(intercept_frame.bonus_text, None);
        assert!(
            intercept_frame
                .objects
                .iter()
                .any(|object| object.kind == EntityKind::Human && object.x16 == ATTRACT_HUMAN_X16)
        );
        assert_eq!(
            caught_frame
                .bonus_text
                .expect("catch frame should show the 500 bonus")
                .text,
            "500"
        );
        assert!(caught_frame.objects.iter().any(
            |object| object.kind == EntityKind::Human && object.x16 == ATTRACT_CAUGHT_HUMAN_X16
        ));
    }

    #[test]
    fn rescue_sequence_uses_player_shot_cue_on_the_laser_tick() {
        let laser_tick = RESCUE_DESCENT_TICKS + RESCUE_ASCENT_TICKS;

        assert_eq!(super::demo_cue_for_tick(laser_tick), SoundCue::PlayerShot);
    }

    #[test]
    fn legend_text_waits_for_post_spawn_rom_delay() {
        let table_start = RESCUE_DESCENT_TICKS
            + RESCUE_ASCENT_TICKS
            + RESCUE_LASER_TICKS
            + RESCUE_FALL_TICKS
            + RESCUE_SCORE_TICKS
            + RESCUE_RETURN_TICKS;
        let just_before_text =
            table_start + LEGEND_APPROACH_TICKS + LEGEND_LASER_TICKS + LEGEND_TEXT_TICKS - 1;
        let first_text_tick =
            table_start + LEGEND_APPROACH_TICKS + LEGEND_LASER_TICKS + LEGEND_TEXT_TICKS;

        assert_eq!(revealed_score_entries_for_tick(just_before_text), 0);
        assert_eq!(revealed_score_entries_for_tick(first_text_tick), 1);
    }
}
