fn push_attract_scoring_top_display_border(scene: &mut RenderScene) {
    for (screen_address, size) in TOP_DISPLAY_BORDER_SEGMENTS {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::TOP_DISPLAY_BORDER_WORD,
            layer: RenderLayer::Hud,
            position: offset_f32_position(
                screen_position_from_address(screen_address),
                point_position(ATTRACT_SCORING_VISUAL_OFFSET),
            ),
            size,
            tint: ATTRACT_SCORING_SCANNER_BORDER_TINT,
        });
    }
}

fn push_attract_scoring_scanner_terrain(scene: &mut RenderScene) {
    for record in scanner_mini_terrain_records() {
        let origin = offset_f32_position(
            screen_position_from_address(record.screen_address),
            point_position(ATTRACT_SCORING_VISUAL_OFFSET),
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
        push_actor_attract_scoring_bonus(scene, bonus, frame.display_step);
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
    table_world_x: i32,
    table_world_y: i32,
    scanner_color_word: u16,
}

const ACTOR_ATTRACT_SCORING_LEGEND: [ActorAttractScoringLegendEntry;
    ATTRACT_SCORING_LEGEND_ENTRIES as usize] = [
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Lander,
        table_world_x: 0x07A0,
        table_world_y: 0x5900,
        scanner_color_word: 0x4433,
    },
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Mutant,
        table_world_x: 0x0FA0,
        table_world_y: 0x5900,
        scanner_color_word: 0xCC33,
    },
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Baiter,
        table_world_x: 0x1820,
        table_world_y: 0x5B00,
        scanner_color_word: 0x3333,
    },
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Bomber,
        table_world_x: 0x0800,
        table_world_y: 0x9100,
        scanner_color_word: 0x8888,
    },
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Pod,
        table_world_x: 0x1000,
        table_world_y: 0x9100,
        scanner_color_word: 0xCCCC,
    },
    ActorAttractScoringLegendEntry {
        enemy: ActorAttractScoringEnemyKind::Swarmer,
        table_world_x: 0x1880,
        table_world_y: 0x9300,
        scanner_color_word: 0x2424,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorAttractScoringBonus {
    sprite: SpriteId,
    world_x: i32,
    world_y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorAttractScoringObject {
    kind: ActorAttractScoringObjectKind,
    world_x: i32,
    world_y: i32,
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
        return ATTRACT_SCORING_SEQUENCE_START_STEP;
    };

    let reveal_display_step = actor_attract_scoring_display_step_for_stage(
        ActorAttractScoringStage::LegendReveal(legend_index),
        0,
    );
    ATTRACT_SCORING_SEQUENCE_START_STEP
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
                ATTRACT_SCORING_LANDER_WORLD_X,
                ATTRACT_SCORING_LANDER_WORLD_Y + i32::from(local_step) * 0x00A0,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::Human,
                ATTRACT_SCORING_HUMAN_WORLD_X,
                ATTRACT_SCORING_HUMAN_WORLD_Y,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShip,
                ATTRACT_SCORING_PLAYER_WORLD_X,
                ATTRACT_SCORING_PLAYER_WORLD_Y,
            ));
        }
        ActorAttractScoringStage::RescueAscend | ActorAttractScoringStage::RescueLaser => {
            let rise_step = if stage == ActorAttractScoringStage::RescueAscend {
                local_step
            } else {
                ATTRACT_SCORING_RESCUE_ASCENT_STEPS + local_step
            };
            let lander_y = if stage == ActorAttractScoringStage::RescueLaser {
                actor_attract_scoring_laser_aligned_enemy_world_y(
                    ActorAttractScoringEnemyKind::Lander,
                    ATTRACT_SCORING_PLAYER_WORLD_Y,
                )
            } else {
                ATTRACT_SCORING_LANDER_WORLD_Y
                    + i32::from(ATTRACT_SCORING_RESCUE_DESCENT_STEPS) * 0x00A0
                    - i32::from(rise_step) * 0x00B0
            };
            let human_y = ATTRACT_SCORING_HUMAN_WORLD_Y - i32::from(rise_step) * 0x00B0;
            objects.push(actor_attract_scoring_enemy_object(
                ActorAttractScoringEnemyKind::Lander,
                ATTRACT_SCORING_LANDER_WORLD_X,
                lander_y,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::Human,
                ATTRACT_SCORING_HUMAN_WORLD_X,
                human_y,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShip,
                ATTRACT_SCORING_PLAYER_WORLD_X,
                ATTRACT_SCORING_PLAYER_WORLD_Y,
            ));
            if stage == ActorAttractScoringStage::RescueLaser {
                objects.push(actor_attract_scoring_object(
                    ActorAttractScoringObjectKind::PlayerShot,
                    ATTRACT_SCORING_LANDER_WORLD_X,
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
                let lander_y = actor_attract_scoring_laser_aligned_enemy_world_y(
                    ActorAttractScoringEnemyKind::Lander,
                    ATTRACT_SCORING_PLAYER_WORLD_Y,
                );
                objects.push(actor_attract_scoring_visual_enemy_object(
                    ActorAttractScoringEnemyKind::Lander,
                    ATTRACT_SCORING_LANDER_WORLD_X,
                    lander_y,
                    ActorAttractScoringVisual::Explosion,
                    local_step,
                ));
            }
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::Human,
                ATTRACT_SCORING_HUMAN_WORLD_X,
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
                ATTRACT_SCORING_CAUGHT_HUMAN_WORLD_X,
                human_y,
            ));
        }
        ActorAttractScoringStage::RescueReturn => {
            let (ship_x, ship_y, _) =
                actor_attract_scoring_drop_state(ATTRACT_SCORING_RESCUE_SCORE_STEPS);
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShip,
                ship_x + i32::from(local_step) * ATTRACT_SCORING_RESCUE_RETURN_WORLD_X_VELOCITY,
                ship_y + i32::from(local_step) * ATTRACT_SCORING_RESCUE_RETURN_WORLD_Y_VELOCITY,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::Human,
                ATTRACT_SCORING_CAUGHT_HUMAN_WORLD_X,
                ATTRACT_SCORING_GROUNDED_HUMAN_WORLD_Y,
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
                ATTRACT_SCORING_CAUGHT_HUMAN_WORLD_X,
                ATTRACT_SCORING_GROUNDED_HUMAN_WORLD_Y,
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
    let mut ship_x = ATTRACT_SCORING_PLAYER_WORLD_X;
    let mut ship_y = ATTRACT_SCORING_PLAYER_WORLD_Y;
    let mut human_y = ATTRACT_SCORING_HUMAN_WORLD_Y
        - i32::from(ATTRACT_SCORING_RESCUE_ASCENT_STEPS + ATTRACT_SCORING_RESCUE_LASER_STEPS)
            * 0x00B0;
    let mut elapsed = 0;
    let mut human_velocity = 0;
    for _ in 0..(ATTRACT_SCORING_RESCUE_FALL_STEPS / 2) {
        human_velocity += ATTRACT_SCORING_RESCUE_HUMAN_WORLD_ACCELERATION;
        for _ in 0..2 {
            if elapsed >= fall_step {
                return (ship_x, ship_y, human_y);
            }
            ship_x += ATTRACT_SCORING_RESCUE_SHIP_WORLD_X_VELOCITY;
            ship_y += ATTRACT_SCORING_RESCUE_SHIP_WORLD_Y_VELOCITY;
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
        ship_y + i32::from(score_step) * ATTRACT_SCORING_RESCUE_DROP_WORLD_Y_VELOCITY,
        ATTRACT_SCORING_CAUGHT_HUMAN_WORLD_Y + i32::from(score_step) * ATTRACT_SCORING_RESCUE_DROP_WORLD_Y_VELOCITY,
    )
}

fn actor_attract_scoring_legend_player_position() -> (i32, i32) {
    let (ship_x, ship_y, _) = actor_attract_scoring_drop_state(ATTRACT_SCORING_RESCUE_SCORE_STEPS);
    (
        ship_x
            + i32::from(ATTRACT_SCORING_RESCUE_RETURN_STEPS) * ATTRACT_SCORING_RESCUE_RETURN_WORLD_X_VELOCITY,
        ship_y
            + i32::from(ATTRACT_SCORING_RESCUE_RETURN_STEPS) * ATTRACT_SCORING_RESCUE_RETURN_WORLD_Y_VELOCITY,
    )
}

fn append_actor_attract_scoring_legend_objects(
    objects: &mut Vec<ActorAttractScoringObject>,
    stage: ActorAttractScoringStage,
    local_step: u16,
    player_world_x: i32,
    player_world_y: i32,
) {
    for entry in ACTOR_ATTRACT_SCORING_LEGEND
        .iter()
        .take(actor_attract_scoring_revealed_legend_entries(stage))
    {
        objects.push(actor_attract_scoring_enemy_object(
            entry.enemy,
            entry.table_world_x,
            entry.table_world_y,
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
    let legend_enemy_world_y = actor_attract_scoring_legend_enemy_world_y(entry.enemy, player_world_y);
    match stage {
        ActorAttractScoringStage::LegendApproach(_) => {
            let enemy_y = actor_attract_scoring_lerp_world_y(
                ATTRACT_SCORING_LEGEND_ORIGIN_START_WORLD_Y,
                legend_enemy_world_y,
                local_step,
                ATTRACT_SCORING_LEGEND_APPROACH_STEPS,
            );
            objects.push(actor_attract_scoring_enemy_object(
                entry.enemy,
                ATTRACT_SCORING_LEGEND_ORIGIN_WORLD_X,
                enemy_y,
            ));
        }
        ActorAttractScoringStage::LegendLaser(_) => {
            objects.push(actor_attract_scoring_enemy_object(
                entry.enemy,
                ATTRACT_SCORING_LEGEND_ORIGIN_WORLD_X,
                legend_enemy_world_y,
            ));
            objects.push(actor_attract_scoring_object(
                ActorAttractScoringObjectKind::PlayerShot,
                player_world_x,
                player_world_y,
            ));
        }
        ActorAttractScoringStage::LegendTransfer(_) => {
            objects.push(actor_attract_scoring_visual_enemy_object(
                entry.enemy,
                ATTRACT_SCORING_LEGEND_ORIGIN_WORLD_X,
                legend_enemy_world_y,
                ActorAttractScoringVisual::Explosion,
                local_step,
            ));
            objects.push(actor_attract_scoring_visual_enemy_object(
                entry.enemy,
                entry.table_world_x,
                entry.table_world_y,
                ActorAttractScoringVisual::Materialize,
                local_step,
            ));
        }
        ActorAttractScoringStage::LegendReveal(_) => {
            objects.push(actor_attract_scoring_enemy_object(
                entry.enemy,
                entry.table_world_x,
                entry.table_world_y,
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

fn actor_attract_scoring_legend_enemy_world_y(
    enemy: ActorAttractScoringEnemyKind,
    player_world_y: i32,
) -> i32 {
    actor_attract_scoring_laser_aligned_enemy_world_y(enemy, player_world_y)
}

fn actor_attract_scoring_laser_aligned_enemy_world_y(
    enemy: ActorAttractScoringEnemyKind,
    player_world_y: i32,
) -> i32 {
    let player_top_y = actor_attract_scoring_scene_position(0, player_world_y)[1];
    let ship_anchor_y = player_top_y + PLAYER_SHIP_SCENE_SIZE[1] / 2.0 + 1.0;
    let target_top_y = ship_anchor_y - actor_attract_scoring_enemy_size(enemy)[1] / 2.0;
    let native_y = target_top_y - ATTRACT_SCORING_OBJECT_REFERENCE_OFFSET[1];
    (native_y.round() as i32) << 8
}

fn actor_attract_scoring_lerp_world_y(start_world_y: i32, end_world_y: i32, step: u16, steps: u16) -> i32 {
    let denominator = i64::from(steps.saturating_sub(1).max(1));
    let numerator = i64::from(step.min(steps.saturating_sub(1)));
    let start = i64::from(start_world_y);
    let delta = i64::from(end_world_y - start_world_y);
    (start + delta * numerator / denominator) as i32
}

fn actor_attract_scoring_enemy_object(
    enemy: ActorAttractScoringEnemyKind,
    world_x: i32,
    world_y: i32,
) -> ActorAttractScoringObject {
    actor_attract_scoring_object(ActorAttractScoringObjectKind::Enemy(enemy), world_x, world_y)
}

fn actor_attract_scoring_visual_enemy_object(
    enemy: ActorAttractScoringEnemyKind,
    world_x: i32,
    world_y: i32,
    visual: ActorAttractScoringVisual,
    visual_step: u16,
) -> ActorAttractScoringObject {
    ActorAttractScoringObject {
        kind: ActorAttractScoringObjectKind::Enemy(enemy),
        world_x,
        world_y,
        visual,
        visual_step,
    }
}

fn actor_attract_scoring_object(
    kind: ActorAttractScoringObjectKind,
    world_x: i32,
    world_y: i32,
) -> ActorAttractScoringObject {
    ActorAttractScoringObject {
        kind,
        world_x,
        world_y,
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
            world_x: ATTRACT_SCORING_SCORE_500_WORLD_X,
            world_y: ATTRACT_SCORING_SCORE_500_WORLD_Y,
        }),
        ActorAttractScoringStage::RescueReturn => Some(ActorAttractScoringBonus {
            sprite: SpriteId::SCORE_POPUP_500,
            world_x: ATTRACT_SCORING_SCORE_500_DROP_WORLD_X,
            world_y: ATTRACT_SCORING_SCORE_500_DROP_WORLD_Y + i32::from(local_step / 2) * 0x0010,
        }),
        ActorAttractScoringStage::LegendTransfer(index) if local_step == 0 => {
            let entry = ACTOR_ATTRACT_SCORING_LEGEND[index];
            Some(ActorAttractScoringBonus {
                sprite: SpriteId::SCORE_POPUP_250,
                world_x: entry.table_world_x,
                world_y: entry.table_world_y,
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
        position: actor_attract_scoring_scene_position(object.world_x, object.world_y),
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
        tint: williams_color_byte_tint((color_word & 0x00FF) as u8),
    });
}

fn push_actor_attract_scoring_laser_beam(
    scene: &mut RenderScene,
    player_ship: ActorAttractScoringObject,
    target: ActorAttractScoringObject,
    display_step: u16,
) {
    let start = actor_attract_scoring_laser_ship_anchor(actor_attract_scoring_scene_position(
        player_ship.world_x,
        player_ship.world_y,
    ));
    let target_position = actor_attract_scoring_scene_position(target.world_x, target.world_y);
    let end = match target.kind {
        ActorAttractScoringObjectKind::Enemy(enemy) => {
            actor_attract_scoring_laser_enemy_anchor(enemy, target_position)
        }
        _ => target_position,
    };
    push_actor_scoring_sparse_laser(scene, start[0], start[1], end[0], display_step);
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
    [position[0], position[1] + size[1] / 2.0]
}

fn push_actor_attract_scoring_bonus(
    scene: &mut RenderScene,
    bonus: ActorAttractScoringBonus,
    display_step: u16,
) {
    let position = actor_attract_scoring_scene_position(bonus.world_x, bonus.world_y);
    if bonus.sprite == SpriteId::SCORE_POPUP_500 {
        push_actor_attract_scoring_score_500_pixels(scene, position, display_step);
        return;
    }

    scene.push_sprite(SceneSprite {
        sprite: bonus.sprite,
        layer: RenderLayer::Objects,
        position,
        size: SCORE_POPUP_SCENE_SIZE,
        tint: Color::WHITE,
    });
}

fn push_actor_attract_scoring_score_500_pixels(
    scene: &mut RenderScene,
    position: [f32; 2],
    display_step: u16,
) {
    let bytes = crate::arcade_assets::object_bitmap_bytes(SCORE_POPUP_500_PIXEL_BITMAP);
    let rows = 6_usize;
    let bytes_per_row = 6_usize;
    if bytes.len() != rows * bytes_per_row {
        return;
    }

    let phase = usize::from((display_step / 5) % SCORE_POPUP_500_COLOR_CYCLE.len() as u16);
    for column in 0..bytes_per_row {
        let column_start = column * rows;
        for row in 0..rows {
            let byte = bytes[column_start + row];
            if let Some(tint) = actor_score_500_nibble_tint(byte >> 4, phase) {
                push_actor_fragment_pixel(
                    scene,
                    [position[0] + (column * 2) as f32, position[1] + row as f32],
                    tint,
                );
            }
            if let Some(tint) = actor_score_500_nibble_tint(byte & 0x0F, phase) {
                push_actor_fragment_pixel(
                    scene,
                    [
                        position[0] + (column * 2 + 1) as f32,
                        position[1] + row as f32,
                    ],
                    tint,
                );
            }
        }
    }
}

const SCORE_POPUP_500_PIXEL_BITMAP: crate::arcade_assets::ObjectBitmapId =
    crate::arcade_assets::ObjectBitmapId::Score500Primary; // original: C5D10

fn actor_score_500_nibble_tint(nibble: u8, phase: usize) -> Option<Color> {
    match nibble {
        0x0 => None,
        0xD => Some(SCORE_POPUP_500_COLOR_CYCLE[phase % SCORE_POPUP_500_COLOR_CYCLE.len()]),
        0xE => Some(SCORE_POPUP_500_COLOR_CYCLE[(phase + 1) % SCORE_POPUP_500_COLOR_CYCLE.len()]),
        0xF => Some(SCORE_POPUP_500_COLOR_CYCLE[(phase + 2) % SCORE_POPUP_500_COLOR_CYCLE.len()]),
        _ => actor_sprite_asset_nibble_tint(nibble),
    }
}

fn push_actor_scoring_sparse_laser(
    scene: &mut RenderScene,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    display_step: u16,
) {
    let left = start_x.min(end_x).round() as i32;
    let right = start_x.max(end_x).round() as i32;
    if right <= left {
        return;
    }

    let direction = if end_x >= start_x { 1 } else { -1 };
    let visible_left = if direction > 0 { left } else { left + 1 };
    let visible_right = if direction > 0 { right - 1 } else { right };
    if visible_right < visible_left {
        return;
    }
    let y = start_y.round() as i32;
    let head_x = if direction > 0 {
        visible_right
    } else {
        visible_left
    };
    let mut x = left;
    let mut cell = 0_i32;
    while x <= right {
        let cells_from_head = if direction > 0 {
            (head_x - x).div_euclid(LASER_BYTE_PIXELS)
        } else {
            (x - head_x).div_euclid(LASER_BYTE_PIXELS)
        }
        .max(0);
        let (byte, tint) = if cells_from_head == 0 {
            let byte = if x >= right {
                LASER_TIP_BYTE & 0xF0
            } else {
                LASER_TIP_BYTE
            };
            (byte, LASER_TIP_TINT)
        } else if cells_from_head <= LASER_BODY_CELLS {
            (LASER_BODY_BYTE, LASER_BODY_TINT)
        } else {
            let fizzle_seed = i32::from(display_step) + cell * 7 + x;
            let byte = if fizzle_seed.rem_euclid(5) == 0 {
                LASER_BODY_BYTE
            } else {
                actor_laser_fizzle_byte(fizzle_seed as u8)
            };
            (byte, LASER_FIZZLE_TINT)
        };
        push_actor_scoring_laser_byte(scene, x, y, byte, tint, visible_left, visible_right);
        x += LASER_BYTE_PIXELS;
        cell += 1;
    }
}

fn push_actor_scoring_laser_byte(
    scene: &mut RenderScene,
    x: i32,
    y: i32,
    byte: u8,
    tint: Color,
    visible_left: i32,
    visible_right: i32,
) {
    if byte & 0xF0 != 0 {
        push_actor_scoring_laser_pixel(scene, x, y, tint, visible_left, visible_right);
    }
    if byte & 0x0F != 0 {
        push_actor_scoring_laser_pixel(scene, x + 1, y, tint, visible_left, visible_right);
    }
}

fn push_actor_scoring_laser_pixel(
    scene: &mut RenderScene,
    x: i32,
    y: i32,
    tint: Color,
    visible_left: i32,
    visible_right: i32,
) {
    if x < visible_left
        || x > visible_right
        || x < 0
        || y < 0
        || x >= scene.surface.width as i32
        || y >= scene.surface.height as i32
    {
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

const fn actor_laser_fizzle_byte(seed: u8) -> u8 {
    (seed & 0x01) | ((seed & 0x02) << 3)
}

fn push_actor_attract_scoring_fragment_pixels(
    scene: &mut RenderScene,
    object: ActorAttractScoringObject,
) {
    let ActorAttractScoringObjectKind::Enemy(enemy) = object.kind else {
        return;
    };
    let position = actor_attract_scoring_scene_position(object.world_x, object.world_y);
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

fn push_actor_attract_scoring_materialize_pixels(
    scene: &mut RenderScene,
    enemy: ActorAttractScoringEnemyKind,
    position: [f32; 2],
    visual_step: u16,
) {
    let Some(position) = try_screen_position_from_scene_position(position) else {
        return;
    };
    let descriptor = actor_attract_scoring_enemy_sprite_frame_descriptor(enemy);
    let appearance_age = actor_attract_scoring_materialize_age(visual_step);
    let growth_size = appearance_growth_size_for_age(appearance_age);
    let _ = push_appearance_cloud_pixels(
        scene,
        position,
        descriptor.sprite_asset_label,
        descriptor.picture_size,
        descriptor.sprite,
        growth_size,
    );
}

fn push_actor_attract_scoring_explosion_pixels(
    scene: &mut RenderScene,
    enemy: ActorAttractScoringEnemyKind,
    position: [f32; 2],
    visual_step: u16,
) {
    let Some(position) = try_screen_position_from_scene_position(position) else {
        return;
    };
    let growth_size = explosion_growth_size_for_age(visual_step.saturating_add(2));
    let _ = push_explosion_cloud_pixels(
        scene,
        clean_explosion_kind(actor_attract_scoring_enemy_explosion_kind(enemy)),
        position,
        None,
        growth_size,
    );
}

fn push_actor_fragment_pixel(scene: &mut RenderScene, position: [f32; 2], tint: Color) {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActorAttractScoringSpriteFrameDescriptor {
    sprite_asset_label: &'static str,
    picture_size: (u8, u8),
    sprite: SpriteId,
}

const ATTRACT_LANDER_SPRITE_ASSET_LABEL: &str = "LNDP1"; // original: LNDP1
const ATTRACT_MUTANT_SPRITE_ASSET_LABEL: &str = "SCZP1"; // original: SCZP1
const ATTRACT_BAITER_SPRITE_ASSET_LABEL: &str = "UFOP1"; // original: UFOP1
const ATTRACT_BOMBER_SPRITE_ASSET_LABEL: &str = "TIEP1"; // original: TIEP1
const ATTRACT_POD_SPRITE_ASSET_LABEL: &str = "PRBP1"; // original: PRBP1
const ATTRACT_SWARMER_SPRITE_ASSET_LABEL: &str = "SWPIC1"; // original: SWPIC1

fn actor_attract_scoring_enemy_sprite_frame_descriptor(
    enemy: ActorAttractScoringEnemyKind,
) -> ActorAttractScoringSpriteFrameDescriptor {
    match enemy {
        ActorAttractScoringEnemyKind::Lander => ActorAttractScoringSpriteFrameDescriptor {
            sprite_asset_label: ATTRACT_LANDER_SPRITE_ASSET_LABEL,
            picture_size: (5, 8),
            sprite: SpriteId::ENEMY_LANDER,
        },
        ActorAttractScoringEnemyKind::Mutant => ActorAttractScoringSpriteFrameDescriptor {
            sprite_asset_label: ATTRACT_MUTANT_SPRITE_ASSET_LABEL,
            picture_size: (5, 8),
            sprite: SpriteId::ENEMY_MUTANT,
        },
        ActorAttractScoringEnemyKind::Baiter => ActorAttractScoringSpriteFrameDescriptor {
            sprite_asset_label: ATTRACT_BAITER_SPRITE_ASSET_LABEL,
            picture_size: (6, 4),
            sprite: SpriteId::ENEMY_BAITER,
        },
        ActorAttractScoringEnemyKind::Bomber => ActorAttractScoringSpriteFrameDescriptor {
            sprite_asset_label: ATTRACT_BOMBER_SPRITE_ASSET_LABEL,
            picture_size: (4, 8),
            sprite: SpriteId::ENEMY_BOMBER,
        },
        ActorAttractScoringEnemyKind::Pod => ActorAttractScoringSpriteFrameDescriptor {
            sprite_asset_label: ATTRACT_POD_SPRITE_ASSET_LABEL,
            picture_size: (4, 8),
            sprite: SpriteId::ENEMY_POD,
        },
        ActorAttractScoringEnemyKind::Swarmer => ActorAttractScoringSpriteFrameDescriptor {
            sprite_asset_label: ATTRACT_SWARMER_SPRITE_ASSET_LABEL,
            picture_size: (3, 4),
            sprite: SpriteId::ENEMY_SWARMER,
        },
    }
}

fn actor_attract_scoring_enemy_explosion_kind(
    enemy: ActorAttractScoringEnemyKind,
) -> ExplosionKind {
    match enemy {
        ActorAttractScoringEnemyKind::Lander => ExplosionKind::Lander,
        ActorAttractScoringEnemyKind::Mutant => ExplosionKind::Mutant,
        ActorAttractScoringEnemyKind::Baiter => ExplosionKind::Baiter,
        ActorAttractScoringEnemyKind::Bomber => ExplosionKind::Bomber,
        ActorAttractScoringEnemyKind::Pod => ExplosionKind::Pod,
        ActorAttractScoringEnemyKind::Swarmer => ExplosionKind::Swarmer,
    }
}

fn actor_attract_scoring_materialize_age(visual_step: u16) -> u16 {
    let final_age = 0x2C_u32;
    let step = u32::from(visual_step.min(ATTRACT_SCORING_LEGEND_TRANSFER_STEPS.saturating_sub(1)));
    let denominator = u32::from(
        ATTRACT_SCORING_LEGEND_TRANSFER_STEPS
            .saturating_sub(1)
            .max(1),
    );
    u16::try_from(step * final_age / denominator).expect("materialize age fits in u16")
}

fn try_screen_position_from_scene_position(position: [f32; 2]) -> Option<ScreenPosition> {
    if !position[0].is_finite() || !position[1].is_finite() {
        return None;
    }
    let x = position[0].round();
    let y = position[1].round();
    if x < 0.0 || y < 0.0 || x > f32::from(u8::MAX) || y > f32::from(u8::MAX) {
        return None;
    }
    Some(ScreenPosition::new(x as u8, y as u8))
}

fn actor_sprite_asset_nibble_tint(nibble: u8) -> Option<Color> {
    match nibble {
        0x0 => None,
        0x1 | 0xA | 0xC | 0xD | 0xE | 0xF => Some(Color::WHITE),
        0x2..=0x9 => Some(williams_color_byte_tint(
            NORMAL_PALETTE_BYTES[usize::from(nibble)],
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

fn actor_attract_scoring_scene_position(world_x: i32, world_y: i32) -> [f32; 2] {
    offset_f32_position(
        actor_attract_scoring_native_position(world_x, world_y),
        ATTRACT_SCORING_OBJECT_REFERENCE_OFFSET,
    )
}

fn actor_attract_scoring_scanner_position(object: ActorAttractScoringObject) -> [f32; 2] {
    let [native_x, native_y] = actor_attract_scoring_native_position(object.world_x, object.world_y);
    offset_f32_position(
        [
            ATTRACT_SCORING_SCANNER_ORIGIN[0]
                + native_x * ATTRACT_SCORING_SCANNER_SIZE[0] / ATTRACT_SCORING_PLAYFIELD_SIZE[0],
            ATTRACT_SCORING_SCANNER_ORIGIN[1]
                + native_y * ATTRACT_SCORING_SCANNER_SIZE[1] / ATTRACT_SCORING_PLAYFIELD_SIZE[1],
        ],
        point_position(ATTRACT_SCORING_VISUAL_OFFSET),
    )
}

fn actor_attract_scoring_native_position(world_x: i32, world_y: i32) -> [f32; 2] {
    [
        ((world_x + 0x10) >> 5).clamp(0, 319) as f32,
        ((world_y + 0x80) >> 8).clamp(0, 255) as f32,
    ]
}
