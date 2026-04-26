use crate::attract_rom::{WILLIAMS_TRACE_POINT_COUNT, attract_rom};
use crate::audio::SoundCue;
use crate::game::{Entity, EntityKind, EntityState, HorizontalDirection, Status, World};
use crate::high_scores::{HighScoreEntry, HighScoreTable};

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
const LEGEND_TRANSFER_TICKS: u16 = 0x20;
const LEGEND_REVEAL_TICKS: u16 = 0x20;
const LEGEND_ENTRY_TICKS: u16 =
    LEGEND_APPROACH_TICKS + LEGEND_LASER_TICKS + LEGEND_TRANSFER_TICKS + LEGEND_REVEAL_TICKS;
const LEGEND_HOLD_TICKS: u16 = 0xFF + 0xFF;
const TEXTP_TICKS: u16 = 6;
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
const ATTRACT_LEGEND_SOURCE_X16: i32 = 0x1F00;
const ATTRACT_LEGEND_SOURCE_START_Y16: i32 = 0xA000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractFrame {
    pub world: World,
    pub objects: Vec<AttractObject>,
    pub scanner_objects: Vec<AttractObject>,
    pub revealed_score_entries: usize,
    pub visible_legend_text_entries: usize,
    pub demo_tick: u16,
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
    pub visual_tick: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttractVisual {
    #[default]
    Sprite,
    Explosion,
    Materialize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttractBonusText {
    pub text: &'static str,
    pub x16: i32,
    pub y16: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttractDemoStage {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AttractLegendEntry {
    kind: EntityKind,
    label: &'static str,
    score: u32,
    table_x16: i32,
    table_y16: i32,
}

const ATTRACT_ROM_LEGEND: [AttractLegendEntry; 6] = [
    // These are the red-label `PICTS` / `XS` / `ENMYTB` score-card entries.
    AttractLegendEntry {
        kind: EntityKind::Lander,
        label: "LANDER",
        score: 150,
        table_x16: 0x0900,
        table_y16: 0x6000,
    },
    AttractLegendEntry {
        kind: EntityKind::Mutant,
        label: "MUTANT",
        score: 150,
        table_x16: 0x1100,
        table_y16: 0x6000,
    },
    AttractLegendEntry {
        kind: EntityKind::Baiter,
        label: "BAITER",
        score: 200,
        table_x16: 0x1980,
        table_y16: 0x6200,
    },
    AttractLegendEntry {
        kind: EntityKind::Bomber,
        label: "BOMBER",
        score: 250,
        table_x16: 0x0960,
        table_y16: 0x9800,
    },
    AttractLegendEntry {
        kind: EntityKind::Pod,
        label: "POD",
        score: 1000,
        table_x16: 0x1160,
        table_y16: 0x9800,
    },
    AttractLegendEntry {
        kind: EntityKind::Swarmer,
        label: "SWARMER",
        score: 150,
        table_x16: 0x19E0,
        table_y16: 0x9A00,
    },
];

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
    pub demo_stage: Option<AttractDemoStage>,
    pub demo_stage_tick: u16,
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
        demo_stage: None,
        demo_stage_tick: 0,
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
            demo_stage: None,
            demo_stage_tick: 0,
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
            demo_stage: None,
            demo_stage_tick: 0,
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
        demo_stage: None,
        demo_stage_tick: 0,
        palette_phase: 0,
        logo_trace_points: 0,
        logo_show_title_text: false,
        logo_show_full_defender: false,
        logo_defender_appear_tick: None,
        logo_show_copyright: false,
    }
}

fn demo_beats() -> Vec<AttractBeat> {
    let mut beats = Vec::with_capacity(demo_total_ticks() as usize);
    for (stage, duration) in attract_demo_timeline() {
        append_demo_stage_beats(&mut beats, stage, duration);
    }
    beats
}

fn demo_total_ticks() -> u16 {
    RESCUE_DESCENT_TICKS
        + RESCUE_ASCENT_TICKS
        + RESCUE_LASER_TICKS
        + RESCUE_FALL_TICKS
        + RESCUE_SCORE_TICKS
        + RESCUE_RETURN_TICKS
        + LEGEND_ENTRY_TICKS * ATTRACT_ROM_LEGEND.len() as u16
        + LEGEND_HOLD_TICKS
}

fn append_demo_stage_beats(
    beats: &mut Vec<AttractBeat>,
    stage: AttractDemoStage,
    duration_ticks: u16,
) {
    for tick in 0..duration_ticks {
        beats.push(AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(demo_cue_for_stage(stage, tick)),
            hold_ms: ticks_to_ms(1),
            world_steps: 0,
            revealed_score_entries: revealed_score_entries_for_stage(stage),
            demo_stage: Some(stage),
            demo_stage_tick: tick,
            palette_phase: 0,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_show_full_defender: false,
            logo_defender_appear_tick: None,
            logo_show_copyright: false,
        });
    }
}

fn demo_cue_for_stage(stage: AttractDemoStage, local_tick: u16) -> SoundCue {
    match stage {
        AttractDemoStage::RescueLaser if local_tick == 0 => SoundCue::PlayerShot,
        AttractDemoStage::RescueFall if local_tick == 0 => SoundCue::Explosion,
        AttractDemoStage::RescueScore if local_tick == 0 => SoundCue::HumanSaved,
        AttractDemoStage::LegendApproach(_) if local_tick == 0 => SoundCue::EnemySweep,
        AttractDemoStage::LegendLaser(_) if local_tick == 0 => SoundCue::PlayerShot,
        AttractDemoStage::LegendTransfer(_) if local_tick == 0 => SoundCue::Explosion,
        _ => SoundCue::AttractHum,
    }
}

pub fn attract_frame_for_beat(beat: AttractBeat) -> AttractFrame {
    match beat.demo_stage {
        Some(stage) => scripted_attract_frame_for_stage(stage, beat.demo_stage_tick),
        None => {
            let mut world = World::bootstrap();
            for _ in 0..beat.world_steps {
                world.step();
            }
            AttractFrame {
                world,
                objects: Vec::new(),
                scanner_objects: Vec::new(),
                revealed_score_entries: beat.revealed_score_entries,
                visible_legend_text_entries: 0,
                demo_tick: 0,
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

#[cfg(test)]
fn scripted_attract_frame_for_tick(tick: u16) -> AttractFrame {
    let (stage, local_tick) = demo_stage_for_tick(tick);
    scripted_attract_frame_for_stage(stage, local_tick)
}

fn scripted_attract_frame_for_stage(stage: AttractDemoStage, tick: u16) -> AttractFrame {
    scripted_attract_frame_for_stage_core(stage, tick, true)
}

fn scripted_attract_frame_for_stage_core(
    stage: AttractDemoStage,
    tick: u16,
    include_scanner_snapshot: bool,
) -> AttractFrame {
    let mut facing = HorizontalDirection::Right;
    let mut objects = Vec::new();
    let mut bonus_text = None;
    let animation_tick = stage_animation_tick(stage, tick);

    match stage {
        AttractDemoStage::RescueDescend => {
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
        }
        AttractDemoStage::RescueAscend | AttractDemoStage::RescueLaser => {
            let total_rise_tick = match stage {
                AttractDemoStage::RescueAscend => tick,
                AttractDemoStage::RescueLaser => RESCUE_ASCENT_TICKS + tick,
                _ => 0,
            };
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
            if matches!(stage, AttractDemoStage::RescueLaser) {
                add_laser_column(
                    &mut objects,
                    ATTRACT_PLAYER_X16,
                    ATTRACT_PLAYER_Y16,
                    ATTRACT_LANDER_X16,
                    enemy_y,
                );
            }
        }
        AttractDemoStage::RescueFall => {
            // `AMODE3` / `AMODE4` move the ship toward Eugene while the falling
            // human accelerates every two ticks until the catch frame.
            let (ship_x, ship_y, human_y) = rescue_intercept_state(tick);
            objects.push(attract_object(EntityKind::PlayerShip, ship_x, ship_y));
            if tick < 12 {
                let explosion_y = ATTRACT_LANDER_Y16 + i32::from(RESCUE_DESCENT_TICKS) * 0x00A0
                    - i32::from(RESCUE_ASCENT_TICKS + RESCUE_LASER_TICKS) * 0x00B0;
                objects.push(attract_visual_object(
                    EntityKind::Lander,
                    ATTRACT_LANDER_X16,
                    explosion_y,
                    AttractVisual::Explosion,
                    tick,
                ));
            }
            objects.push(attract_object_with_state(
                EntityKind::Human,
                ATTRACT_HUMAN_X16,
                human_y,
                EntityState::Falling,
            ));
        }
        AttractDemoStage::RescueScore => {
            // `AMODE5` teleports the caught human onto the ship's path, spawns the
            // `500`, and drops both ship and human straight toward the terrain.
            let (ship_x, ship_y, human_y) = rescue_drop_state(tick);
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
        }
        AttractDemoStage::RescueReturn => {
            facing = HorizontalDirection::Left;
            let (ship_start_x, ship_start_y, _) = rescue_drop_state(RESCUE_SCORE_TICKS);
            objects.push(attract_object_facing(
                EntityKind::PlayerShip,
                ship_start_x + i32::from(tick) * ATTRACT_RESCUE_RETURN_XV16,
                ship_start_y + i32::from(tick) * ATTRACT_RESCUE_RETURN_YV16,
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
        }
        AttractDemoStage::LegendApproach(_)
        | AttractDemoStage::LegendLaser(_)
        | AttractDemoStage::LegendTransfer(_)
        | AttractDemoStage::LegendReveal(_)
        | AttractDemoStage::LegendHold => {
            let (drop_ship_x, drop_ship_y, _) = rescue_drop_state(RESCUE_SCORE_TICKS);
            let player_x =
                drop_ship_x + i32::from(RESCUE_RETURN_TICKS) * ATTRACT_RESCUE_RETURN_XV16;
            let player_y =
                drop_ship_y + i32::from(RESCUE_RETURN_TICKS) * ATTRACT_RESCUE_RETURN_YV16;
            objects.push(attract_object(EntityKind::PlayerShip, player_x, player_y));
            objects.push(attract_object(
                EntityKind::Human,
                ATTRACT_CAUGHT_HUMAN_X16,
                ATTRACT_GROUNDED_HUMAN_Y16,
            ));
            append_legend_entities(&mut objects, stage, tick, player_x, player_y);
        }
    }
    let revealed_score_entries = revealed_score_entries_for_stage(stage);
    let entities: Vec<Entity> = objects
        .iter()
        .map(|object| rom_entity_with_state(object.kind, object.x16, object.y16, object.state))
        .collect();
    let demo_tick = demo_tick_for_stage(stage, tick);
    let scanner_objects = if include_scanner_snapshot {
        scanner_snapshot_objects_for_demo_tick(demo_tick)
    } else {
        Vec::new()
    };

    AttractFrame {
        demo_tick,
        world: scripted_world(
            0,
            rom_x_to_world(ATTRACT_PLAYER_START_X << 8),
            ATTRACT_PLAYER_Y,
            facing,
            entities,
        ),
        objects,
        scanner_objects,
        revealed_score_entries,
        visible_legend_text_entries: legend_text_entries_for_stage(stage, tick),
        bonus_text,
        player_facing: facing,
        animation_tick,
    }
}

fn append_legend_entities(
    objects: &mut Vec<AttractObject>,
    stage: AttractDemoStage,
    local_tick: u16,
    player_x16: i32,
    player_y16: i32,
) {
    let source_y = ATTRACT_LEGEND_SOURCE_START_Y16 - i32::from(LEGEND_APPROACH_TICKS) * 0x00C0;
    let current_index = match stage {
        AttractDemoStage::LegendApproach(index)
        | AttractDemoStage::LegendLaser(index)
        | AttractDemoStage::LegendTransfer(index)
        | AttractDemoStage::LegendReveal(index) => Some(index),
        AttractDemoStage::LegendHold => None,
        _ => return,
    };

    let visible_entries = match stage {
        AttractDemoStage::LegendHold => ATTRACT_ROM_LEGEND.len(),
        _ => revealed_score_entries_for_stage(stage),
    };
    for entry in ATTRACT_ROM_LEGEND.iter().take(visible_entries) {
        objects.push(attract_object(entry.kind, entry.table_x16, entry.table_y16));
    }

    let Some(index) = current_index else {
        return;
    };
    let entry = ATTRACT_ROM_LEGEND[index];
    match stage {
        AttractDemoStage::LegendApproach(_) => {
            let enemy_y = ATTRACT_LEGEND_SOURCE_START_Y16 - i32::from(local_tick) * 0x00C0;
            objects.push(attract_object(
                entry.kind,
                ATTRACT_LEGEND_SOURCE_X16,
                enemy_y,
            ));
        }
        AttractDemoStage::LegendLaser(_) => {
            objects.push(attract_object(
                entry.kind,
                ATTRACT_LEGEND_SOURCE_X16,
                source_y,
            ));
            add_laser_column(
                objects,
                player_x16,
                player_y16,
                ATTRACT_LEGEND_SOURCE_X16,
                source_y,
            );
        }
        AttractDemoStage::LegendTransfer(_) => {
            objects.push(attract_visual_object(
                entry.kind,
                ATTRACT_LEGEND_SOURCE_X16,
                source_y,
                AttractVisual::Explosion,
                local_tick,
            ));
            objects.push(attract_visual_object(
                entry.kind,
                entry.table_x16,
                entry.table_y16,
                AttractVisual::Materialize,
                local_tick,
            ));
        }
        AttractDemoStage::LegendReveal(_) => {
            objects.push(attract_object(entry.kind, entry.table_x16, entry.table_y16));
        }
        AttractDemoStage::LegendHold => {}
        _ => unreachable!("non-legend stage routed into legend object renderer"),
    }
}

fn demo_stage_for_tick(mut tick: u16) -> (AttractDemoStage, u16) {
    for (stage, duration) in attract_demo_timeline() {
        if tick < duration {
            return (stage, tick);
        }
        tick -= duration;
    }
    (
        AttractDemoStage::LegendHold,
        LEGEND_HOLD_TICKS.saturating_sub(1),
    )
}

fn attract_demo_timeline() -> impl Iterator<Item = (AttractDemoStage, u16)> {
    let mut stages = Vec::with_capacity(6 + ATTRACT_ROM_LEGEND.len() * 4 + 1);
    stages.push((AttractDemoStage::RescueDescend, RESCUE_DESCENT_TICKS));
    stages.push((AttractDemoStage::RescueAscend, RESCUE_ASCENT_TICKS));
    stages.push((AttractDemoStage::RescueLaser, RESCUE_LASER_TICKS));
    stages.push((AttractDemoStage::RescueFall, RESCUE_FALL_TICKS));
    stages.push((AttractDemoStage::RescueScore, RESCUE_SCORE_TICKS));
    stages.push((AttractDemoStage::RescueReturn, RESCUE_RETURN_TICKS));
    for index in 0..ATTRACT_ROM_LEGEND.len() {
        stages.push((
            AttractDemoStage::LegendApproach(index),
            LEGEND_APPROACH_TICKS,
        ));
        stages.push((AttractDemoStage::LegendLaser(index), LEGEND_LASER_TICKS));
        stages.push((
            AttractDemoStage::LegendTransfer(index),
            LEGEND_TRANSFER_TICKS,
        ));
        stages.push((AttractDemoStage::LegendReveal(index), LEGEND_REVEAL_TICKS));
    }
    stages.push((AttractDemoStage::LegendHold, LEGEND_HOLD_TICKS));
    stages.into_iter()
}

fn revealed_score_entries_for_stage(stage: AttractDemoStage) -> usize {
    match stage {
        AttractDemoStage::LegendReveal(index) => index + 1,
        AttractDemoStage::LegendHold => ATTRACT_ROM_LEGEND.len(),
        AttractDemoStage::LegendApproach(index)
        | AttractDemoStage::LegendLaser(index)
        | AttractDemoStage::LegendTransfer(index) => index,
        _ => 0,
    }
}

fn legend_text_entries_for_stage(stage: AttractDemoStage, local_tick: u16) -> usize {
    let demo_tick = demo_tick_for_stage(stage, local_tick);

    ATTRACT_ROM_LEGEND
        .iter()
        .enumerate()
        .take_while(|(index, _)| {
            let bmode2_tick = demo_tick_for_stage(AttractDemoStage::LegendReveal(*index), 0);
            demo_tick >= next_text_process_tick(bmode2_tick)
        })
        .count()
}

fn next_text_process_tick(tick: u16) -> u16 {
    let remainder = tick % TEXTP_TICKS;
    if remainder == 0 {
        tick
    } else {
        tick + (TEXTP_TICKS - remainder)
    }
}

fn demo_tick_for_stage(target_stage: AttractDemoStage, local_tick: u16) -> u16 {
    let mut elapsed = 0;
    for (stage, duration) in attract_demo_timeline() {
        if stage == target_stage {
            return elapsed + local_tick.min(duration.saturating_sub(1));
        }
        elapsed += duration;
    }

    elapsed
}

fn scanner_snapshot_objects_for_demo_tick(demo_tick: u16) -> Vec<AttractObject> {
    let refresh_tick = demo_tick - (demo_tick % 4);
    let (stage, local_tick) = demo_stage_for_tick(refresh_tick);
    scripted_attract_frame_for_stage_core(stage, local_tick, false).objects
}

fn stage_animation_tick(stage: AttractDemoStage, local_tick: u16) -> u32 {
    match stage {
        AttractDemoStage::RescueDescend => u32::from(local_tick),
        AttractDemoStage::RescueAscend => u32::from(RESCUE_DESCENT_TICKS + local_tick),
        AttractDemoStage::RescueLaser => {
            u32::from(RESCUE_DESCENT_TICKS + RESCUE_ASCENT_TICKS + local_tick)
        }
        AttractDemoStage::RescueFall => {
            u32::from(RESCUE_DESCENT_TICKS + RESCUE_ASCENT_TICKS + RESCUE_LASER_TICKS + local_tick)
        }
        AttractDemoStage::RescueScore => u32::from(local_tick),
        AttractDemoStage::RescueReturn => u32::from(local_tick),
        AttractDemoStage::LegendApproach(index)
        | AttractDemoStage::LegendLaser(index)
        | AttractDemoStage::LegendTransfer(index)
        | AttractDemoStage::LegendReveal(index) => {
            u32::from(index as u16 * LEGEND_ENTRY_TICKS + local_tick)
        }
        AttractDemoStage::LegendHold => u32::from(local_tick),
    }
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
        visual_tick: 0,
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
        visual_tick: 0,
    }
}

fn attract_visual_object(
    kind: EntityKind,
    x16: i32,
    y16: i32,
    visual: AttractVisual,
    visual_tick: u16,
) -> AttractObject {
    AttractObject {
        kind,
        x16,
        y16,
        state: EntityState::Normal,
        facing: HorizontalDirection::Right,
        visual,
        visual_tick,
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
        ATTRACT_ROM_LEGEND
            .into_iter()
            .take(revealed_score_entries)
            .map(|entry| format!("{:<8}{:>8}", entry.label, entry.score)),
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
        ATTRACT_CAUGHT_HUMAN_X16, ATTRACT_HUMAN_X16, ATTRACT_LANDER_Y16, AttractDemoStage,
        AttractVisual, LEGEND_APPROACH_TICKS, LEGEND_LASER_TICKS, LEGEND_TRANSFER_TICKS,
        RESCUE_ASCENT_TICKS, RESCUE_DESCENT_TICKS, RESCUE_FALL_TICKS, RESCUE_LASER_TICKS,
        RESCUE_RETURN_TICKS, RESCUE_SCORE_TICKS, SceneKind, attract_cycle, attract_scene,
        demo_stage_for_tick, high_score_scene, legend_text_entries_for_stage, logo_scene,
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
        let (stage, local_tick) = demo_stage_for_tick(laser_tick);

        assert_eq!(stage, AttractDemoStage::RescueLaser);
        assert_eq!(
            super::demo_cue_for_stage(stage, local_tick),
            SoundCue::PlayerShot
        );
    }

    #[test]
    fn legend_text_waits_for_text_process_refresh() {
        let table_start = RESCUE_DESCENT_TICKS
            + RESCUE_ASCENT_TICKS
            + RESCUE_LASER_TICKS
            + RESCUE_FALL_TICKS
            + RESCUE_SCORE_TICKS
            + RESCUE_RETURN_TICKS;
        let reveal_start =
            table_start + LEGEND_APPROACH_TICKS + LEGEND_LASER_TICKS + LEGEND_TRANSFER_TICKS;

        let (reveal_stage, reveal_tick_zero) = demo_stage_for_tick(reveal_start);
        let (_, reveal_tick_one) = demo_stage_for_tick(reveal_start + 1);

        assert_eq!(reveal_stage, AttractDemoStage::LegendReveal(0));
        assert_eq!(
            legend_text_entries_for_stage(reveal_stage, reveal_tick_zero),
            0
        );
        assert_eq!(
            legend_text_entries_for_stage(reveal_stage, reveal_tick_one),
            1
        );
    }

    #[test]
    fn scanner_snapshot_updates_on_four_tick_cadence() {
        let frame_three = scripted_attract_frame_for_tick(3);
        let frame_four = scripted_attract_frame_for_tick(4);

        let live_three = frame_three
            .objects
            .iter()
            .find(|object| object.kind == EntityKind::Lander)
            .expect("live frame should contain the descending lander");
        let scanner_three = frame_three
            .scanner_objects
            .iter()
            .find(|object| object.kind == EntityKind::Lander)
            .expect("scanner snapshot should contain the descending lander");
        let scanner_four = frame_four
            .scanner_objects
            .iter()
            .find(|object| object.kind == EntityKind::Lander)
            .expect("scanner snapshot should refresh on the fourth tick");
        let live_four = frame_four
            .objects
            .iter()
            .find(|object| object.kind == EntityKind::Lander)
            .expect("live frame should still contain the descending lander");

        assert_ne!(live_three.y16, scanner_three.y16);
        assert_eq!(scanner_three.y16, ATTRACT_LANDER_Y16);
        assert_eq!(scanner_four.y16, live_four.y16);
    }

    #[test]
    fn rescue_explosion_keeps_the_lander_kind() {
        let rescue_phase_end = RESCUE_DESCENT_TICKS + RESCUE_ASCENT_TICKS + RESCUE_LASER_TICKS;
        let frame = scripted_attract_frame_for_tick(rescue_phase_end);

        let explosion = frame
            .objects
            .iter()
            .find(|object| object.visual == AttractVisual::Explosion)
            .expect("rescue fall should begin with an explosion object");

        assert_eq!(explosion.kind, EntityKind::Lander);
        assert_eq!(explosion.visual_tick, 0);
    }

    #[test]
    fn legend_entry_keeps_the_source_enemy_alive_for_the_full_laser_window() {
        let table_start = RESCUE_DESCENT_TICKS
            + RESCUE_ASCENT_TICKS
            + RESCUE_LASER_TICKS
            + RESCUE_FALL_TICKS
            + RESCUE_SCORE_TICKS
            + RESCUE_RETURN_TICKS;
        let laser_frame = scripted_attract_frame_for_tick(table_start + LEGEND_APPROACH_TICKS + 1);

        let source_enemy = laser_frame
            .objects
            .iter()
            .find(|object| {
                object.kind == EntityKind::Lander && object.visual == AttractVisual::Sprite
            })
            .expect("legend laser dwell should keep the source enemy visible");

        assert_eq!(source_enemy.x16, 0x1F00);
    }

    #[test]
    fn legend_entry_switches_to_explosion_and_materialize_on_the_same_transfer_tick() {
        let table_start = RESCUE_DESCENT_TICKS
            + RESCUE_ASCENT_TICKS
            + RESCUE_LASER_TICKS
            + RESCUE_FALL_TICKS
            + RESCUE_SCORE_TICKS
            + RESCUE_RETURN_TICKS;
        let transfer_frame = scripted_attract_frame_for_tick(
            table_start + LEGEND_APPROACH_TICKS + LEGEND_LASER_TICKS,
        );

        let explosion = transfer_frame
            .objects
            .iter()
            .find(|object| {
                object.kind == EntityKind::Lander && object.visual == AttractVisual::Explosion
            })
            .expect("legend transfer should show an explosion phase");
        let materialize = transfer_frame
            .objects
            .iter()
            .find(|object| {
                object.kind == EntityKind::Lander && object.visual == AttractVisual::Materialize
            })
            .expect("legend transfer should show a materialize phase");

        assert_eq!(explosion.kind, EntityKind::Lander);
        assert_eq!(materialize.kind, EntityKind::Lander);
        assert_eq!(explosion.visual_tick, 0);
        assert_eq!(materialize.visual_tick, 0);
    }
}
