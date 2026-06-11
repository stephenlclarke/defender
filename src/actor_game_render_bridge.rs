use super::*;

pub(in crate::actor_game) const TOP_DISPLAY_LEFT_SCANNER_MARKER_CELL: u16 = 0x4C07;
pub(in crate::actor_game) const TOP_DISPLAY_RIGHT_SCANNER_MARKER_CELL: u16 = 0x4C28;
pub(in crate::actor_game) const ACTOR_RENDER_WILLIAMS_RED_GREEN_CHANNEL_MASK: u8 = 0x07;
pub(in crate::actor_game) const ACTOR_RENDER_WILLIAMS_GREEN_CHANNEL_SHIFT: u8 = 3;
pub(in crate::actor_game) const ACTOR_RENDER_WILLIAMS_BLUE_CHANNEL_SHIFT: u8 = 6;
pub(in crate::actor_game) const ACTOR_RENDER_WILLIAMS_BLUE_CHANNEL_MASK: u8 = 0x03;
pub(in crate::actor_game) const ACTOR_RENDER_OPAQUE_ALPHA: u8 = 0xFF;
pub(in crate::actor_game) const ACTOR_RENDER_TRANSPARENT: Color = Color::from_rgba(0, 0, 0, 0);
pub(in crate::actor_game) const ACTOR_RENDER_SCREEN_SIGN_BIT: u16 = 0x8000;
pub(in crate::actor_game) const HUMAN_CARRIED_TINT: Color =
    Color::from_rgba(0xFF, 0xF8, 0x80, ACTOR_RENDER_OPAQUE_ALPHA);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorRenderSceneBridge {
    pub(in crate::actor_game) surface: SurfaceSize,
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
        let state = (report.phase == Phase::Playing && report.player_start.is_none())
            .then(|| report.game_state());
        if report.phase == Phase::Playing && report.smart_bomb_flash_steps_remaining > 0 {
            scene.clear_color = Color::WHITE;
        }
        if report.phase == Phase::Playing
            && let Some(terrain_blow) = report.terrain_blow
            && terrain_blow.terrain_erased()
        {
            let flash_tint = terrain_blow_flash_tint(terrain_blow.elapsed_ticks);
            if flash_tint.rgba[3] != 0 {
                scene.clear_color = flash_tint;
            }
        }
        if report.phase == Phase::Playing
            && report.player_start.is_none()
            && report.terrain_blow.is_none()
        {
            push_background_terrain_sprites(
                &mut scene,
                report.background_left,
                wave_tuning_landscape_tint(report.wave),
            );
        }
        if let Some(state) = &state {
            push_actor_playing_top_display_border(&mut scene, report.wave);
            push_scanner_radar_sprites(&mut scene, &state.world.scanner);
        }
        push_actor_playing_hud_sprites(&mut scene, report);
        for draw in &report.draws {
            self.push_draw(&mut scene, report, draw);
        }
        push_actor_player_switch_prompt_sprites(&mut scene, report);
        push_actor_final_game_over_prompt_sprites(&mut scene, report);
        push_actor_player_start_prompt_sprites(&mut scene, report);
        push_actor_wave_completion_status_sprites(&mut scene, report);
        push_actor_survivor_bonus_icon_sprites(&mut scene, report);
        scene
    }

    fn push_draw(&self, scene: &mut RenderScene, report: &StepReport, draw: &DrawCommand) {
        if let Some(text) = &draw.text {
            let layer = if report.phase == Phase::Attract {
                RenderLayer::Overlay
            } else {
                RenderLayer::Hud
            };
            if let VisualEffect::MessageText {
                screen_cell,
                visual_offset,
            } = draw.effect
            {
                push_controlled_message_sprites_with_offset(
                    scene,
                    text,
                    screen_cell,
                    layer,
                    visual_offset,
                );
                return;
            }
            push_message_text_bytes_sprites(
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
                color_step,
            } => self.push_williams_reveal(scene, draw.position, stroke_step, color_step),
            VisualEffect::DefenderCoalescence { slot, row_pair } => {
                self.push_defender_coalescence(scene, slot, row_pair)
            }
            VisualEffect::AttractScoringSurface { scoring_tick } => {
                self.push_attract_scoring_surface(scene, scoring_tick)
            }
            VisualEffect::ExplosionCloud {
                kind,
                age,
                explosion_anchor,
            } => self.push_explosion_sprite(scene, draw.position, kind, age, explosion_anchor),
            VisualEffect::Static
            | VisualEffect::MessageText { .. }
            | VisualEffect::LanderSpriteFrame { .. }
            | VisualEffect::BomberSpriteFrame { .. }
            | VisualEffect::PodSprite
            | VisualEffect::BaiterSpriteFrame { .. }
            | VisualEffect::HumanSpriteFrame { .. } => self.push_static_sprite(scene, report, draw),
        }
    }

    fn push_williams_reveal(
        &self,
        scene: &mut RenderScene,
        position: Point,
        stroke_step: u16,
        color_step: u16,
    ) {
        let tint = VISUAL_STATE.attract_williams_logo_tint_for_step(color_step);
        let pixel_path = attract_williams_logo_pixel_path();
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
                .saturating_mul(u16::from(ATTRACT_DEFENDER_APPEARANCE_FINAL_TICK))
                .checked_div(total_steps)
                .unwrap_or(0)
        };
        let appearance_tick = u8::try_from(appearance_tick)
            .expect("Defender appearance tick fits")
            .min(ATTRACT_DEFENDER_APPEARANCE_FINAL_TICK);

        for pixel in attract_defender_appearance_pixels(scene.surface, appearance_tick) {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
                layer: RenderLayer::Overlay,
                position: [f32::from(pixel.position[0]), f32::from(pixel.position[1])],
                size: [1.0, 1.0],
                tint: Color { rgba: pixel.color },
            });
        }
    }

    fn push_attract_scoring_surface(&self, scene: &mut RenderScene, scoring_tick: TimelineStep) {
        push_background_terrain_sprites(scene, 0, wave_tuning_landscape_tint(1));
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
        explosion_anchor: Option<Point>,
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
        let growth_size = actor_explosion_growth_size_for_kind(kind, age);
        if let Some(screen_position) = try_screen_position(position)
            && push_explosion_cloud_pixels(
                scene,
                clean_explosion_kind(kind),
                screen_position,
                explosion_anchor.and_then(try_screen_position),
                growth_size,
            )
        {
            return;
        }

        let scale = actor_explosion_render_scale(growth_size);
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

    fn push_static_sprite(&self, scene: &mut RenderScene, report: &StepReport, draw: &DrawCommand) {
        let Some(position) = actor_draw_screen_position(report, draw) else {
            return;
        };
        let Some(sprite) = actor_scene_sprite(draw.sprite, position) else {
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

pub(in crate::actor_game) fn push_actor_wave_completion_status_sprites(
    scene: &mut RenderScene,
    report: &StepReport,
) {
    if !should_show_actor_wave_completion_status(report) {
        return;
    }

    for (message, screen_cell) in WAVE_COMPLETION_STATUS_LINES {
        let text = crate::arcade_assets::message_text(*message);
        push_message_text_bytes_sprites(
            scene,
            text.as_bytes(),
            screen_position_from_cell(*screen_cell),
            RenderLayer::Overlay,
        );
    }

    let (wave_digits, wave_digit_count) = actor_visible_decimal_digits(clean_wave(report.wave));
    push_message_text_bytes_sprites(
        scene,
        &wave_digits[..wave_digit_count],
        screen_position_from_cell(WAVE_COMPLETION_WAVE_NUMBER_SCREEN_CELL),
        RenderLayer::Overlay,
    );

    let multiplier = clean_wave(report.wave).min(5);
    let (multiplier_digits, multiplier_digit_count) = actor_visible_decimal_digits(multiplier);
    push_message_text_bytes_sprites(
        scene,
        &multiplier_digits[..multiplier_digit_count],
        screen_position_from_cell(WAVE_COMPLETION_MULTIPLIER_NUMBER_SCREEN_CELL),
        RenderLayer::Overlay,
    );
}

pub(in crate::actor_game) fn push_actor_survivor_bonus_icon_sprites(
    scene: &mut RenderScene,
    report: &StepReport,
) {
    if !should_show_actor_wave_completion_status(report) {
        return;
    }

    for index in 0..actor_visible_survivor_bonus_icon_count(report) {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::HUMAN,
            layer: RenderLayer::Overlay,
            position: screen_position_from_cell_with_offset(
                SURVIVOR_BONUS_FIRST_HUMAN_SCREEN_CELL,
                (index as u8) * SURVIVOR_BONUS_HUMAN_STEP,
                0,
            ),
            size: SURVIVOR_BONUS_HUMAN_SIZE,
            tint: Color::WHITE,
        });
    }
}

pub(in crate::actor_game) fn should_show_actor_wave_completion_status(report: &StepReport) -> bool {
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

pub(in crate::actor_game) fn push_actor_player_switch_prompt_sprites(
    scene: &mut RenderScene,
    report: &StepReport,
) {
    let Some(player_switch) = report.player_switch else {
        return;
    };

    push_actor_message_sprites(
        scene,
        player_message(player_switch.from_player),
        PLAYER_SWITCH_LABEL_SCREEN_CELL,
        RenderLayer::Overlay,
    );
    push_actor_message_sprites(
        scene,
        MessageId::GameOver,
        PLAYER_SWITCH_GAME_OVER_SCREEN_CELL,
        RenderLayer::Overlay,
    );
}

pub(in crate::actor_game) fn push_actor_final_game_over_prompt_sprites(
    scene: &mut RenderScene,
    report: &StepReport,
) {
    if report.player_death_sleep_remaining.is_none() {
        return;
    }

    push_actor_message_sprites(
        scene,
        MessageId::GameOver,
        FINAL_GAME_OVER_SCREEN_CELL,
        RenderLayer::Overlay,
    );
}

pub(in crate::actor_game) fn push_actor_player_start_prompt_sprites(
    scene: &mut RenderScene,
    report: &StepReport,
) {
    let Some(player_start) = report.player_start else {
        return;
    };
    if report.player_count <= 1 {
        return;
    }

    push_actor_message_sprites(
        scene,
        player_message(player_start.player),
        PLAYER_START_PROMPT_SCREEN_CELL,
        RenderLayer::Overlay,
    );
}

pub(in crate::actor_game) fn push_actor_message_sprites(
    scene: &mut RenderScene,
    message: MessageId,
    screen_cell: ScreenAddress,
    layer: RenderLayer,
) {
    push_controlled_message_sprites(
        scene,
        crate::arcade_assets::message_text(message),
        screen_cell,
        layer,
    );
}

pub(in crate::actor_game) const TWO_DIGIT_DISPLAY_MAX: u8 = 99;
pub(in crate::actor_game) const DECIMAL_RADIX: u8 = 10;
pub(in crate::actor_game) const SCORE_DIGIT_START_PLACE: u32 = 100_000;

pub(in crate::actor_game) fn actor_visible_survivor_bonus_icon_count(report: &StepReport) -> usize {
    report
        .survivor_bonus
        .map(|bonus| usize::from(bonus.visible_icons).min(SURVIVOR_BONUS_HUMAN_LIMIT))
        .unwrap_or(0)
}

pub(in crate::actor_game) fn actor_visible_decimal_digits(value: u8) -> ([u8; 2], usize) {
    let value = value.min(TWO_DIGIT_DISPLAY_MAX);
    if value < DECIMAL_RADIX {
        ([b'0' + value, b' '], 1)
    } else {
        (
            [b'0' + value / DECIMAL_RADIX, b'0' + value % DECIMAL_RADIX],
            2,
        )
    }
}

pub(in crate::actor_game) fn push_controlled_message_sprites_with_offset(
    scene: &mut RenderScene,
    text: &str,
    screen_cell: ScreenAddress,
    layer: RenderLayer,
    visual_offset: Point,
) {
    let first_sprite = scene.sprites.len();
    push_controlled_message_sprites(scene, text, screen_cell, layer);
    offset_new_sprites(scene, first_sprite, point_position(visual_offset));
}

pub(in crate::actor_game) fn push_actor_playing_top_display_border(
    scene: &mut RenderScene,
    wave: u16,
) {
    for (screen_cell_word, size) in TOP_DISPLAY_BORDER_SEGMENTS {
        let screen_cell = crate::ScreenAddress::new(screen_cell_word);
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::TOP_DISPLAY_BORDER_WORD,
            layer: RenderLayer::Hud,
            position: screen_position_from_cell(screen_cell),
            size,
            tint: if matches!(
                screen_cell.word(),
                TOP_DISPLAY_LEFT_SCANNER_MARKER_CELL | TOP_DISPLAY_RIGHT_SCANNER_MARKER_CELL
            ) {
                VISUAL_STATE.top_display_scanner_marker_tint()
            } else {
                wave_tuning_landscape_tint(wave)
            },
        });
    }
}

pub(in crate::actor_game) fn williams_color_byte_tint(value: u8) -> Color {
    if value == 0 {
        return ACTOR_RENDER_TRANSPARENT;
    }
    Color::from_rgba(
        WILLIAMS_RED_GREEN_LEVELS
            [usize::from(value & ACTOR_RENDER_WILLIAMS_RED_GREEN_CHANNEL_MASK)],
        WILLIAMS_RED_GREEN_LEVELS[usize::from(
            (value >> ACTOR_RENDER_WILLIAMS_GREEN_CHANNEL_SHIFT)
                & ACTOR_RENDER_WILLIAMS_RED_GREEN_CHANNEL_MASK,
        )],
        WILLIAMS_BLUE_LEVELS[usize::from(
            (value >> ACTOR_RENDER_WILLIAMS_BLUE_CHANNEL_SHIFT)
                & ACTOR_RENDER_WILLIAMS_BLUE_CHANNEL_MASK,
        )],
        ACTOR_RENDER_OPAQUE_ALPHA,
    )
}

pub(in crate::actor_game) fn scanner_mini_terrain_records()
-> &'static [ScannerMiniTerrainRecord; SCANNER_TERRAIN_RECORDS] {
    static RECORDS: OnceLock<[ScannerMiniTerrainRecord; SCANNER_TERRAIN_RECORDS]> = OnceLock::new();
    RECORDS.get_or_init(|| {
        generate_scanner_mini_terrain_records(0u16.wrapping_sub(SCANNER_SCAN_CENTER_OFFSET))
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::actor_game) struct ScannerMiniTerrainRecord {
    pub(in crate::actor_game) screen_cell: crate::ScreenAddress,
    pub(in crate::actor_game) word: u16,
}

pub(in crate::actor_game) fn generate_scanner_mini_terrain_records(
    scan_left: u16,
) -> [ScannerMiniTerrainRecord; SCANNER_TERRAIN_RECORDS] {
    let bytes = main_terrain_record_bytes();
    let first_record = usize::from(scan_left.to_be_bytes()[0] >> 2);
    assert!(
        first_record + SCANNER_TERRAIN_RECORDS <= SCANNER_MINI_TERRAIN_RECORDS,
        "main terrain slice must contain 64 scanner terrain records"
    );

    let mut records = [ScannerMiniTerrainRecord::default(); SCANNER_TERRAIN_RECORDS];
    let mut screen_column = SCANNER_OBJECT_BASE_SCREEN.to_be_bytes()[0];
    for (index, record) in records.iter_mut().enumerate() {
        let record_byte_index = (first_record + index) * 3;
        *record = ScannerMiniTerrainRecord {
            screen_cell: crate::ScreenAddress::from_bytes(screen_column, bytes[record_byte_index]),
            word: u16::from_be_bytes([bytes[record_byte_index + 1], bytes[record_byte_index + 2]]),
        };
        screen_column = screen_column.wrapping_add(1);
    }

    records
}

pub(in crate::actor_game) fn main_terrain_record_bytes()
-> &'static [u8; MAIN_TERRAIN_RECORD_BYTE_COUNT] {
    crate::arcade_assets::MAIN_TERRAIN_BYTES
}

pub(in crate::actor_game) fn offset_new_sprites(
    scene: &mut RenderScene,
    first_sprite: usize,
    offset: [f32; 2],
) {
    if offset == [0.0, 0.0] {
        return;
    }
    for sprite in &mut scene.sprites[first_sprite..] {
        sprite.position = offset_f32_position(sprite.position, offset);
    }
}

pub(in crate::actor_game) fn offset_f32_position(position: [f32; 2], offset: [f32; 2]) -> [f32; 2] {
    [position[0] + offset[0], position[1] + offset[1]]
}

pub(in crate::actor_game) fn actor_explosion_render_scale(growth_size: u16) -> f32 {
    explosion_render_scale(growth_size)
        .map(f32::from)
        .unwrap_or(1.0)
}

pub(in crate::actor_game) fn actor_explosion_growth_size_for_kind(
    kind: ExplosionKind,
    age: u16,
) -> u16 {
    if kind == ExplosionKind::Terrain {
        return terrain_explosion_growth_size_for_age(
            u8::try_from(age).unwrap_or(TERRAIN_EXPLOSION_LIFETIME_STEPS),
        );
    }

    explosion_growth_size_for_age(age)
}

pub(in crate::actor_game) fn point_position(point: Point) -> [f32; 2] {
    [f32::from(point.x), f32::from(point.y)]
}

pub(in crate::actor_game) fn actor_draw_screen_position(
    report: &StepReport,
    draw: &DrawCommand,
) -> Option<Point> {
    if report.phase != Phase::Playing {
        return Some(draw.position);
    }

    let Some(snapshot) = report
        .snapshots
        .iter()
        .find(|snapshot| snapshot.id == draw.actor && snapshot.alive)
    else {
        return Some(draw.position);
    };

    actor_project_actor_state_draw(snapshot, draw.position, report.background_left)
}

pub(in crate::actor_game) fn actor_project_actor_state_draw(
    snapshot: &ActorSnapshot,
    draw_position: Point,
    background_left: u16,
) -> Option<Point> {
    let Some(x_fraction) = actor_actor_state_x_fraction(snapshot) else {
        return Some(draw_position);
    };
    if draw_position != snapshot.position && snapshot.kind != ActorKind::Human {
        return Some(draw_position);
    }
    actor_screen_position_from_world(draw_position, x_fraction, background_left)
}

pub(in crate::actor_game) fn actor_actor_state_x_fraction(snapshot: &ActorSnapshot) -> Option<u8> {
    snapshot.actor_x_fraction()
}

pub(in crate::actor_game) fn actor_screen_position_from_world(
    position: Point,
    x_fraction: u8,
    background_left: u16,
) -> Option<Point> {
    let world_x_word = absolute_world_x(position, x_fraction);
    let active_left = background_left.wrapping_sub(OBJECT_ACTIVE_LEFT_MARGIN);
    if world_x_word.wrapping_sub(active_left) >= OBJECT_ACTIVE_WORLD_WIDTH {
        return None;
    }
    let screen_word = world_x_word.wrapping_sub(background_left);
    if screen_word & ACTOR_RENDER_SCREEN_SIGN_BIT != 0 {
        return None;
    }
    let screen_x = screen_word >> OBJECT_WORLD_TO_SCREEN_SHIFT;
    if screen_x >= OBJECT_VISIBLE_SCREEN_WIDTH {
        return None;
    }
    Some(Point::new(screen_x as i16, position.y))
}

pub(in crate::actor_game) fn williams_reveal_visible_pixel_count(
    stroke_step: u16,
    total_pixels: usize,
) -> usize {
    if total_pixels == 0 {
        return 0;
    }
    if stroke_step >= WILLIAMS_REVEAL_STEPS {
        return total_pixels;
    }
    if stroke_step == 0 {
        return 0;
    }

    let operation_counts = attract_williams_logo_operation_pixel_counts();
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

pub(in crate::actor_game) fn push_actor_playing_hud_sprites(
    scene: &mut RenderScene,
    report: &StepReport,
) {
    if report.phase != Phase::Playing {
        return;
    }

    push_actor_player_score_sprites(
        scene,
        report.player_scores[0],
        ACTOR_HUD_PLAYER_ONE_SCORE_ORIGIN,
    );
    if report.player_count > 1 {
        push_actor_player_score_sprites(
            scene,
            report.player_scores[1],
            ACTOR_HUD_PLAYER_TWO_SCORE_ORIGIN,
        );
    }
    push_actor_player_stock_sprites(
        scene,
        report.player_stocks[0],
        ACTOR_HUD_PLAYER_ONE_LIFE_STOCK_ORIGIN,
        ACTOR_HUD_PLAYER_ONE_SMART_BOMB_STOCK_ORIGIN,
    );
    if report.player_count > 1 {
        push_actor_player_stock_sprites(
            scene,
            report.player_stocks[1],
            ACTOR_HUD_PLAYER_TWO_LIFE_STOCK_ORIGIN,
            ACTOR_HUD_PLAYER_TWO_SMART_BOMB_STOCK_ORIGIN,
        );
    }
}

pub(in crate::actor_game) fn push_actor_player_score_sprites(
    scene: &mut RenderScene,
    score: u32,
    origin: [f32; 2],
) {
    for (index, digit) in actor_visible_score_digits(score).iter().enumerate() {
        let Some(digit) = digit else {
            continue;
        };
        let Some(sprite) = SpriteId::score_digit(*digit) else {
            continue;
        };
        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Hud,
            position: [
                origin[0] + ACTOR_HUD_SCORE_DIGIT_STEP[0] * index as f32,
                origin[1] + ACTOR_HUD_SCORE_DIGIT_STEP[1] * index as f32,
            ],
            size: ACTOR_HUD_SCORE_DIGIT_SIZE,
            tint: VISUAL_STATE.hud_tint(),
        });
    }
}

pub(in crate::actor_game) fn actor_visible_score_digits(
    score: u32,
) -> [Option<u8>; ACTOR_HUD_SCORE_DIGIT_DISPLAY_COUNT] {
    let score = score.min(ACTOR_HUD_SCORE_DISPLAY_MAX);
    let mut place = SCORE_DIGIT_START_PLACE;
    let mut digits = [None; ACTOR_HUD_SCORE_DIGIT_DISPLAY_COUNT];
    let mut non_zero_seen = false;

    for (index, digit) in digits.iter_mut().enumerate() {
        let value = ((score / place) % u32::from(DECIMAL_RADIX)) as u8;
        let counter = ACTOR_HUD_SCORE_DIGIT_DISPLAY_COUNT - index;
        if value == 0 && counter > 2 && !non_zero_seen {
            *digit = None;
        } else {
            non_zero_seen = true;
            *digit = Some(value);
        }
        place /= u32::from(DECIMAL_RADIX);
    }

    digits
}

pub(in crate::actor_game) fn push_actor_player_stock_sprites(
    scene: &mut RenderScene,
    stock: PlayerStockSnapshot,
    life_origin: [f32; 2],
    smart_bomb_origin: [f32; 2],
) {
    push_actor_stock_sprite_series(
        scene,
        SpriteId::PLAYER_LIFE_STOCK,
        stock.lives.min(ACTOR_HUD_PLAYER_LIFE_STOCK_DISPLAY_LIMIT),
        life_origin,
        ACTOR_HUD_PLAYER_LIFE_STOCK_STEP,
        ACTOR_HUD_PLAYER_LIFE_STOCK_SIZE,
    );
    push_actor_stock_sprite_series(
        scene,
        SpriteId::SMART_BOMB_STOCK,
        stock
            .smart_bombs
            .min(ACTOR_HUD_SMART_BOMB_STOCK_DISPLAY_LIMIT),
        smart_bomb_origin,
        ACTOR_HUD_SMART_BOMB_STOCK_STEP,
        ACTOR_HUD_SMART_BOMB_STOCK_SIZE,
    );
}

pub(in crate::actor_game) fn push_actor_stock_sprite_series(
    scene: &mut RenderScene,
    sprite: SpriteId,
    count: u8,
    origin: [f32; 2],
    step: [f32; 2],
    size: [f32; 2],
) {
    for index in 0..count {
        let index = f32::from(index);
        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Hud,
            position: [origin[0] + step[0] * index, origin[1] + step[1] * index],
            size,
            tint: VISUAL_STATE.hud_tint(),
        });
    }
}

pub(in crate::actor_game) fn actor_scene_sprite(
    sprite: SpriteKey,
    position: Point,
) -> Option<SceneSprite> {
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
            HUMAN_CARRIED_TINT,
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
