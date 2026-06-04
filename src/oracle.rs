//! Temporary gameplay oracle backed by the current implementation.
//!
//! This module is the explicit boundary where the clean rewrite can compare
//! against the existing behavior without letting converted implementation names
//! leak into new production contracts.

#[cfg(test)]
use crate::accepted::AcceptedGameplayMachine;
use crate::accepted::{
    AcceptedDirection, AcceptedEvent, AcceptedExpandedObjectDetail, AcceptedExpandedObjectEvidence,
    AcceptedExpandedObjectKind, AcceptedFrame, AcceptedGameOverState, AcceptedHighScoreEntry,
    AcceptedHighScoreInitials, AcceptedHighScoreSubmission, AcceptedHighScoreTableEntry,
    AcceptedHighScoreTables, AcceptedObjectEvidence, AcceptedObjectEvidenceDetail,
    AcceptedObjectEvidenceList, AcceptedPhase, AcceptedSnapshot, AcceptedWaveProfile,
};

use crate::game::{
    ATTRACT_WILLIAMS_LOGO_REVEAL_FRAMES, AttractPresentationPage, AttractPresentationSnapshot,
    Direction, EXPANDED_OBJECT_DETAIL_LIMIT, ExpandedObjectDetailSnapshot,
    ExpandedObjectEvidenceSnapshot, ExpandedObjectKind, GameEvent, GameEvents, GameFrame,
    GameOverSnapshot, GamePhase, GameState, HIGH_SCORE_TABLE_ENTRIES, HighScoreEntrySnapshot,
    HighScoreSubmissionSnapshot, HighScoreTableEntrySnapshot, HighScoreTablesSnapshot,
    OBJECT_EVIDENCE_DETAIL_LIMIT, ObjectEvidenceDetailSnapshot, ObjectEvidenceList,
    ObjectEvidenceSnapshot, PlayerSnapshot, PlayerStockSnapshot, SOURCE_VISUAL_STATE,
    ScannerRadarSnapshot, ScoreSnapshot, SoundEvent, WaveProfileSnapshot, WorldSnapshot,
    WorldVector, attract_credit_text_tint, attract_defender_wordmark_appearance_tick,
    expanded_object_sprite_geometry, push_scanner_radar_sprites,
};
use crate::renderer::{
    Color, RenderLayer, RenderScene, SceneSprite, SpriteId, SurfaceSize,
    push_source_controlled_message_sprites, push_source_message_sprites,
    push_source_text_bytes_sprites, source_attract_defender_appearance_pixels,
    source_attract_williams_logo_pixel_path, source_message_text, source_screen_position,
    source_screen_position_with_offset,
};
#[cfg(test)]
use crate::{game::GameInput, systems::GameSimulation};

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct GameplayOracle {
    machine: AcceptedGameplayMachine,
}

#[cfg(test)]
impl GameplayOracle {
    pub(crate) fn new() -> Self {
        Self {
            machine: AcceptedGameplayMachine::new(),
        }
    }

    pub(crate) fn snapshot(&self) -> GameState {
        adapt_snapshot(self.machine.snapshot())
    }

    pub(crate) fn step(&mut self, input: GameInput) -> GameFrame {
        adapt_frame_output(self.machine.step(input))
    }
}

#[cfg(test)]
impl Default for GameplayOracle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl GameSimulation for GameplayOracle {
    fn state(&self) -> GameState {
        self.snapshot()
    }

    fn step(&mut self, input: GameInput) -> GameFrame {
        GameplayOracle::step(self, input)
    }
}

fn adapt_frame_output(output: AcceptedFrame) -> GameFrame {
    let state = adapt_snapshot(output.snapshot);
    let scene = adapt_scene(&state, output.visual_signature);

    GameFrame {
        state,
        events: GameEvents::new(
            output.events.into_iter().map(adapt_event).collect(),
            output
                .sound_commands
                .into_iter()
                .map(adapt_sound_command)
                .collect(),
        ),
        scene,
    }
}

fn adapt_snapshot(snapshot: AcceptedSnapshot) -> GameState {
    let phase = adapt_phase(snapshot.phase);
    let game_over = adapt_game_over(snapshot.game_over);
    let attract = if phase == GamePhase::Attract && game_over.hall_of_fame_stall_remaining.is_none()
    {
        AttractPresentationSnapshot::for_page_frame(
            u16::try_from(snapshot.frame).unwrap_or(u16::MAX),
        )
    } else {
        AttractPresentationSnapshot::INACTIVE
    };

    let player = PlayerSnapshot {
        position: (
            WorldVector::from_subpixels(snapshot.player.x_subpixels),
            WorldVector::from_subpixels(snapshot.player.y_subpixels),
        ),
        velocity: (
            WorldVector::from_subpixels(snapshot.player.x_velocity_subpixels),
            WorldVector::from_subpixels(snapshot.player.y_velocity_subpixels),
        ),
        direction: adapt_direction(snapshot.player.direction),
        lives: snapshot.player.lives,
        smart_bombs: snapshot.player.smart_bombs,
    };
    let object_evidence = adapt_object_evidence(snapshot.object_evidence);
    let scanner = ScannerRadarSnapshot::for_world(
        phase,
        snapshot.frame,
        player.position.0,
        player.position,
        &object_evidence,
    );

    GameState {
        frame: snapshot.frame,
        phase,
        credits: snapshot.credits,
        current_player: snapshot.current_player,
        player_count: snapshot.player_count,
        wave: snapshot.wave,
        wave_profile: adapt_wave_profile(snapshot.wave_profile, snapshot.wave),
        player,
        player_stocks: snapshot.player_stocks.map(adapt_player_stock),
        scores: ScoreSnapshot {
            player_one: snapshot.scores.player_one,
            player_two: snapshot.scores.player_two,
            high_score: snapshot.scores.high_score,
            next_bonus: snapshot.scores.next_bonus,
        },
        high_score_initials: adapt_high_score_initials(snapshot.high_score_initials),
        high_score_entry: snapshot.high_score_entry.map(adapt_high_score_entry),
        high_score_submission: snapshot
            .high_score_submission
            .map(adapt_high_score_submission),
        high_score_tables: adapt_high_score_tables(snapshot.high_score_tables),
        attract,
        post_game_playfield: None,
        game_over,
        world: WorldSnapshot {
            object_evidence,
            expanded_objects: adapt_expanded_objects(snapshot.expanded_objects),
            player_explosion: snapshot.player_explosion,
            terrain_blow: snapshot.terrain_blow,
            scanner,
            ..WorldSnapshot::default()
        },
    }
}

fn adapt_player_stock(stock: crate::accepted::AcceptedPlayerStock) -> PlayerStockSnapshot {
    PlayerStockSnapshot {
        lives: stock.lives,
        smart_bombs: stock.smart_bombs,
    }
}

fn adapt_high_score_initials(
    initials: AcceptedHighScoreInitials,
) -> crate::systems::HighScoreInitialsState {
    crate::systems::HighScoreInitialsState {
        initials: initials.initials,
        cursor: initials.cursor.min(3),
    }
}

fn adapt_wave_profile(profile: AcceptedWaveProfile, wave: u8) -> WaveProfileSnapshot {
    let raw = WaveProfileSnapshot {
        landers: profile.landers,
        bombers: profile.bombers,
        pods: profile.pods,
        mutants: profile.mutants,
        swarmers: profile.swarmers,
        lander_x_velocity: profile.lander_x_velocity,
        lander_y_velocity_msb: profile.lander_y_velocity_msb,
        lander_y_velocity_lsb: profile.lander_y_velocity_lsb,
        mutant_random_y: profile.mutant_random_y,
        mutant_y_velocity_msb: profile.mutant_y_velocity_msb,
        mutant_y_velocity_lsb: profile.mutant_y_velocity_lsb,
        mutant_x_velocity: profile.mutant_x_velocity,
        swarmer_x_velocity: profile.swarmer_x_velocity,
        wave_time: profile.wave_time,
        wave_size: profile.wave_size,
        lander_shot_time: profile.lander_shot_time,
        bomber_x_velocity: profile.bomber_x_velocity,
        mutant_shot_time: profile.mutant_shot_time,
        swarmer_shot_time: profile.swarmer_shot_time,
        swarmer_acceleration_mask: profile.swarmer_acceleration_mask,
        baiter_delay: profile.baiter_delay,
        baiter_shot_time: profile.baiter_shot_time,
        baiter_seek_probability: profile.baiter_seek_probability,
    };
    WaveProfileSnapshot::from_raw_source_profile(wave, raw)
}

fn adapt_high_score_entry(entry: AcceptedHighScoreEntry) -> HighScoreEntrySnapshot {
    HighScoreEntrySnapshot {
        score: entry.score,
        rank: entry.rank,
    }
}

fn adapt_high_score_submission(
    submission: AcceptedHighScoreSubmission,
) -> HighScoreSubmissionSnapshot {
    HighScoreSubmissionSnapshot {
        player: submission.player,
        score: submission.score,
    }
}

fn adapt_high_score_tables(tables: AcceptedHighScoreTables) -> HighScoreTablesSnapshot {
    HighScoreTablesSnapshot {
        all_time: tables.all_time.map(adapt_high_score_table_entry),
        todays_greatest: tables.todays_greatest.map(adapt_high_score_table_entry),
    }
}

fn adapt_high_score_table_entry(entry: AcceptedHighScoreTableEntry) -> HighScoreTableEntrySnapshot {
    HighScoreTableEntrySnapshot {
        rank: entry.rank,
        score: entry.score,
        initials: entry.initials,
    }
}

fn adapt_game_over(state: AcceptedGameOverState) -> GameOverSnapshot {
    GameOverSnapshot {
        player_death_sleep_remaining: state.player_death_sleep_remaining,
        player_switch_sleep_remaining: state.player_switch_sleep_remaining,
        player_switch_from: state.player_switch_from,
        player_switch_to: state.player_switch_to,
        no_entry_delay_remaining: state.no_entry_delay_remaining,
        hall_of_fame_stall_remaining: state.hall_of_fame_stall_remaining,
    }
}

fn adapt_object_evidence(state: AcceptedObjectEvidence) -> ObjectEvidenceSnapshot {
    let mut evidence = ObjectEvidenceSnapshot {
        active_count: state.active_count,
        inactive_count: state.inactive_count,
        projectile_count: state.projectile_count,
        visible_count: state.visible_count,
        evidence_crc32: Some(state.evidence_crc32),
        detail_count: state.detail_count,
        details: [ObjectEvidenceDetailSnapshot::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT],
    };
    for index in 0..usize::from(state.detail_count).min(OBJECT_EVIDENCE_DETAIL_LIMIT) {
        evidence.details[index] = adapt_object_evidence_detail(state.details[index]);
    }
    evidence
}

fn adapt_object_evidence_detail(
    detail: AcceptedObjectEvidenceDetail,
) -> ObjectEvidenceDetailSnapshot {
    ObjectEvidenceDetailSnapshot {
        list: match detail.list {
            AcceptedObjectEvidenceList::Active => ObjectEvidenceList::Active,
            AcceptedObjectEvidenceList::Inactive => ObjectEvidenceList::Inactive,
            AcceptedObjectEvidenceList::Projectile => ObjectEvidenceList::Projectile,
        },
        object_category: None,
        address: Some(detail.address),
        slot: Some(detail.slot),
        screen_position: Some(crate::systems::ScreenPosition::new(
            detail.screen_x,
            detail.screen_y,
        )),
        world_position: Some((detail.world_x, detail.world_y)),
        velocity: Some((detail.velocity_x, detail.velocity_y)),
        picture_address: Some(detail.picture_address),
        picture_label: detail.picture_label,
        picture_size: detail.picture_size,
        primary_image_address: detail.primary_image_address,
        alternate_image_address: detail.alternate_image_address,
        mapped_sprite: detail.mapped_sprite.map(SpriteId),
        object_type: Some(detail.object_type),
        scanner_color: Some(detail.scanner_color),
    }
}

fn adapt_expanded_objects(state: AcceptedExpandedObjectEvidence) -> ExpandedObjectEvidenceSnapshot {
    let mut evidence = ExpandedObjectEvidenceSnapshot {
        active_count: state.active_count,
        last_slot_address: state.last_slot_address,
        detail_count: state.detail_count,
        details: [ExpandedObjectDetailSnapshot::EMPTY; EXPANDED_OBJECT_DETAIL_LIMIT],
    };
    for index in 0..usize::from(state.detail_count).min(EXPANDED_OBJECT_DETAIL_LIMIT) {
        evidence.details[index] = adapt_expanded_object_detail(state.details[index]);
    }
    evidence
}

fn adapt_expanded_object_detail(
    detail: AcceptedExpandedObjectDetail,
) -> ExpandedObjectDetailSnapshot {
    ExpandedObjectDetailSnapshot {
        kind: match detail.kind {
            AcceptedExpandedObjectKind::Appearance => ExpandedObjectKind::Appearance,
            AcceptedExpandedObjectKind::Explosion => ExpandedObjectKind::Explosion,
            AcceptedExpandedObjectKind::ScorePopup => ExpandedObjectKind::ScorePopup,
        },
        slot_address: Some(detail.slot_address),
        size: detail.size,
        descriptor_address: Some(detail.descriptor_address),
        picture_label: detail.picture_label,
        picture_size: detail.picture_size,
        mapped_sprite: detail.mapped_sprite.map(SpriteId),
        erase_address: Some(detail.erase_address),
        center: Some(crate::systems::ScreenPosition::new(
            detail.center_x,
            detail.center_y,
        )),
        top_left: Some(crate::systems::ScreenPosition::new(
            detail.top_left_x,
            detail.top_left_y,
        )),
        object_address: detail.object_address,
        score_popup_lifetime_ticks: detail.score_popup_lifetime_ticks,
        score_popup_value: detail.score_popup_value,
        explosion_frame: detail.explosion_frame,
        explosion_lifetime_frames: detail.explosion_lifetime_frames,
    }
}

fn adapt_scene(state: &GameState, visual_signature: Option<u32>) -> RenderScene {
    let (width, height) = crate::accepted::native_visible_size();
    let mut scene = RenderScene::empty(
        state.frame,
        SurfaceSize::new(u32::from(width), u32::from(height)),
    );
    scene.visual_signature = visual_signature;
    push_score_sprites(&mut scene, state.scores, state.player_count);
    push_top_display_border_sprites(&mut scene, state);
    push_attract_credit_sprites(&mut scene, state);
    push_attract_presents_sprites(&mut scene, state);
    push_attract_instruction_text_sprites(&mut scene, state);
    push_attract_williams_logo_sprite(&mut scene, state);
    push_attract_defender_wordmark_sprite(&mut scene, state);
    push_attract_copyright_strip_sprite(&mut scene, state);
    push_final_game_over_prompt_sprites(&mut scene, state.game_over);
    push_player_switch_prompt_sprites(&mut scene, state.game_over);
    push_player_start_prompt_sprites(&mut scene, state);
    push_wave_completion_status_sprites(&mut scene, state);
    push_survivor_bonus_icon_sprites(&mut scene, state);
    push_high_score_entry_prompt_sprites(&mut scene, state);
    push_hall_of_fame_display_sprites(&mut scene, state);
    crate::game::push_player_explosion_cloud_sprites(
        &mut scene,
        state.world.player_explosion.as_ref(),
    );

    if state.phase == GamePhase::Playing {
        push_stock_sprites(&mut scene, state.player_count, state.player_stocks);
        push_scanner_radar_sprites(&mut scene, &state.world.scanner);
        push_source_object_detail_sprites(&mut scene, &state.world.object_evidence);
        push_expanded_object_detail_sprites(&mut scene, &state.world.expanded_objects);

        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [
                world_vector_pixels(state.player.position.0),
                world_vector_pixels(state.player.position.1),
            ],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });
    }

    scene
}

fn push_source_message_sprites_with_tint(
    scene: &mut RenderScene,
    text: &str,
    origin: [f32; 2],
    layer: RenderLayer,
    tint: Color,
) {
    let first_sprite = scene.sprites.len();
    push_source_message_sprites(scene, text, origin, layer);
    tint_new_sprites(scene, first_sprite, tint);
}

fn push_source_text_bytes_sprites_with_tint(
    scene: &mut RenderScene,
    bytes: &[u8],
    origin: [f32; 2],
    layer: RenderLayer,
    tint: Color,
) {
    let first_sprite = scene.sprites.len();
    push_source_text_bytes_sprites(scene, bytes, origin, layer);
    tint_new_sprites(scene, first_sprite, tint);
}

fn push_source_controlled_message_sprites_with_tint(
    scene: &mut RenderScene,
    text: &str,
    top_left_screen_address: u16,
    layer: RenderLayer,
    tint: Color,
) {
    let first_sprite = scene.sprites.len();
    push_source_controlled_message_sprites(scene, text, top_left_screen_address, layer);
    tint_new_sprites(scene, first_sprite, tint);
}

fn tint_new_sprites(scene: &mut RenderScene, first_sprite: usize, tint: Color) {
    for sprite in &mut scene.sprites[first_sprite..] {
        sprite.tint = tint;
    }
}

fn push_top_display_border_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Playing {
        return;
    }

    for (screen_address, size) in SOURCE_TOP_DISPLAY_BORDER_SEGMENTS {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::TOP_DISPLAY_BORDER_WORD,
            layer: RenderLayer::Hud,
            position: source_screen_position(*screen_address),
            size: *size,
            tint: top_display_border_segment_tint(*screen_address),
        });
    }
}

fn top_display_border_segment_tint(screen_address: u16) -> Color {
    if matches!(screen_address, 0x4C07 | 0x4C28) {
        SOURCE_VISUAL_STATE.top_display_scanner_marker_tint()
    } else {
        SOURCE_VISUAL_STATE.top_display_border_tint()
    }
}

fn push_source_object_detail_sprites(scene: &mut RenderScene, evidence: &ObjectEvidenceSnapshot) {
    let detail_count = usize::from(evidence.detail_count).min(OBJECT_EVIDENCE_DETAIL_LIMIT);
    for detail in &evidence.details[..detail_count] {
        let Some(layer) = source_object_detail_render_layer(detail.list) else {
            continue;
        };
        let Some(sprite) = detail.mapped_sprite else {
            continue;
        };
        if sprite == SpriteId::NULL_OBJECT {
            continue;
        }
        let Some(position) = detail.screen_position else {
            continue;
        };
        let Some((width, height)) = detail.picture_size else {
            continue;
        };
        if width == 0 || height == 0 {
            continue;
        }

        scene.push_sprite(SceneSprite {
            sprite,
            layer,
            position: [f32::from(position.x), f32::from(position.y)],
            size: [f32::from(width), f32::from(height)],
            tint: Color::WHITE,
        });
    }
}

fn push_expanded_object_detail_sprites(
    scene: &mut RenderScene,
    evidence: &ExpandedObjectEvidenceSnapshot,
) {
    let detail_count = usize::from(evidence.detail_count).min(EXPANDED_OBJECT_DETAIL_LIMIT);
    for detail in &evidence.details[..detail_count] {
        let Some(sprite) = detail.mapped_sprite else {
            continue;
        };
        if sprite == SpriteId::NULL_OBJECT {
            continue;
        }
        let Some((position, size)) = expanded_object_sprite_geometry(detail) else {
            continue;
        };
        if size[0] == 0.0 || size[1] == 0.0 {
            continue;
        }

        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Objects,
            position,
            size,
            tint: Color::WHITE,
        });
    }
}

fn source_object_detail_render_layer(list: ObjectEvidenceList) -> Option<RenderLayer> {
    match list {
        ObjectEvidenceList::Active => Some(RenderLayer::Objects),
        ObjectEvidenceList::Projectile => Some(RenderLayer::Projectiles),
        ObjectEvidenceList::Inactive => None,
    }
}

fn push_attract_credit_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if state.credits == 0
        && !matches!(
            state.attract.page,
            AttractPresentationPage::HallOfFame | AttractPresentationPage::ScoringSequence
        )
    {
        return;
    }

    if let Some(text) = source_message_text("CREDV") {
        push_source_message_sprites_with_tint(
            scene,
            text,
            source_screen_position(SOURCE_ATTRACT_CREDITS_LABEL_SCREEN),
            RenderLayer::Overlay,
            attract_credit_text_tint(state),
        );
    }

    let (digits, digit_count) = attract_credit_digits(state.credits);
    push_source_text_bytes_sprites_with_tint(
        scene,
        &digits[..digit_count],
        source_screen_position(SOURCE_ATTRACT_CREDITS_NUMBER_SCREEN),
        RenderLayer::Overlay,
        attract_credit_text_tint(state),
    );
}

fn attract_credit_digits(credits: u8) -> ([u8; 2], usize) {
    let credits = credits.min(99);
    if credits < 10 {
        ([b'0' + credits, b' '], 1)
    } else {
        ([b'0' + credits / 10, b'0' + credits % 10], 2)
    }
}

fn push_attract_presents_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if !state.attract.shows_presents_text() {
        return;
    }

    if let Some(text) = source_message_text("ELECV") {
        push_source_controlled_message_sprites_with_tint(
            scene,
            text,
            SOURCE_ATTRACT_PRESENTS_ELECTRONICS_SCREEN,
            RenderLayer::Overlay,
            SOURCE_VISUAL_STATE.attract_title_text_tint_for_frame(state.attract.page_frame),
        );
    }
}

fn push_attract_instruction_text_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if !state.attract.shows_instruction_text() {
        return;
    }

    for (label, screen_address) in SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES {
        if let Some(text) = source_message_text(label) {
            push_source_controlled_message_sprites_with_tint(
                scene,
                text,
                *screen_address,
                RenderLayer::Overlay,
                SOURCE_VISUAL_STATE.attract_instruction_text_tint(*screen_address),
            );
        }
    }
}

fn push_attract_williams_logo_sprite(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if !state.attract.shows_williams_logo() {
        return;
    }
    if !SOURCE_VISUAL_STATE.attract_williams_logo_should_render() {
        return;
    }

    let tint = SOURCE_VISUAL_STATE.attract_williams_logo_tint_for_frame(state.attract.page_frame);
    let pixel_path = source_attract_williams_logo_pixel_path();
    let visible_pixel_count =
        attract_williams_logo_visible_pixel_count(&state.attract, pixel_path.len());
    if visible_pixel_count < pixel_path.len() {
        let origin = source_screen_position(SOURCE_ATTRACT_WILLIAMS_LOGO_SCREEN);
        for [pixel_x, pixel_y] in pixel_path.iter().take(visible_pixel_count) {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
                layer: RenderLayer::Overlay,
                position: [
                    origin[0] + f32::from(*pixel_x),
                    origin[1] + f32::from(*pixel_y),
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
        position: source_screen_position(SOURCE_ATTRACT_WILLIAMS_LOGO_SCREEN),
        size: SOURCE_ATTRACT_WILLIAMS_LOGO_SIZE,
        tint,
    });
}

fn attract_williams_logo_visible_pixel_count(
    attract: &AttractPresentationSnapshot,
    total_pixels: usize,
) -> usize {
    if total_pixels == 0 {
        return 0;
    }
    if attract.page != AttractPresentationPage::WilliamsLogo
        || attract.page_frame >= ATTRACT_WILLIAMS_LOGO_REVEAL_FRAMES
    {
        return total_pixels;
    }

    let reveal_frames = usize::from(ATTRACT_WILLIAMS_LOGO_REVEAL_FRAMES);
    let page_frame = usize::from(attract.page_frame.max(1));
    (total_pixels * page_frame / reveal_frames).clamp(1, total_pixels)
}

fn push_attract_defender_wordmark_sprite(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if !state.attract.shows_defender_wordmark() {
        return;
    }

    if let Some(appearance_tick) = attract_defender_wordmark_appearance_tick(&state.attract) {
        for pixel in source_attract_defender_appearance_pixels(scene.surface, appearance_tick) {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
                layer: RenderLayer::Overlay,
                position: [f32::from(pixel.position[0]), f32::from(pixel.position[1])],
                size: [1.0, 1.0],
                tint: Color { rgba: pixel.color },
            });
        }
        return;
    }

    scene.push_sprite(SceneSprite {
        sprite: SpriteId::HALL_OF_FAME_DEFENDER_LOGO,
        layer: RenderLayer::Overlay,
        position: source_screen_position(SOURCE_ATTRACT_DEFENDER_WORDMARK_SCREEN),
        size: SOURCE_DEFENDER_WORDMARK_SIZE,
        tint: SOURCE_VISUAL_STATE.attract_defender_wordmark_tint(),
    });
}

fn push_attract_copyright_strip_sprite(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::Attract || state.game_over.hall_of_fame_stall_remaining.is_some() {
        return;
    }
    if !state.attract.shows_copyright() {
        return;
    }

    scene.push_sprite(SceneSprite {
        sprite: SpriteId::ATTRACT_COPYRIGHT_STRIP,
        layer: RenderLayer::Overlay,
        position: source_screen_position(SOURCE_ATTRACT_COPYRIGHT_STRIP_SCREEN),
        size: SOURCE_ATTRACT_COPYRIGHT_STRIP_SIZE,
        tint: SOURCE_VISUAL_STATE.attract_copyright_tint(),
    });
}

fn push_final_game_over_prompt_sprites(scene: &mut RenderScene, game_over: GameOverSnapshot) {
    if game_over.player_death_sleep_remaining.is_none() {
        return;
    }

    if let Some(text) = source_message_text("GO") {
        push_source_message_sprites(
            scene,
            text,
            source_screen_position(SOURCE_FINAL_GAME_OVER_SCREEN),
            RenderLayer::Overlay,
        );
    }
}

fn push_player_switch_prompt_sprites(scene: &mut RenderScene, game_over: GameOverSnapshot) {
    if game_over.player_switch_sleep_remaining.is_none() {
        return;
    }

    let player_label = if game_over.player_switch_from == Some(2) {
        "PLYR2"
    } else {
        "PLYR1"
    };
    if let Some(text) = source_message_text(player_label) {
        push_source_message_sprites(
            scene,
            text,
            source_screen_position(SOURCE_PLAYER_SWITCH_LABEL_SCREEN),
            RenderLayer::Overlay,
        );
    }
    if let Some(text) = source_message_text("GO") {
        push_source_message_sprites(
            scene,
            text,
            source_screen_position(SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN),
            RenderLayer::Overlay,
        );
    }
}

fn push_player_start_prompt_sprites(scene: &mut RenderScene, state: &GameState) {
    if !should_show_player_start_prompt(state) {
        return;
    }

    let player_label = if state.current_player == 2 {
        "PLYR2"
    } else {
        "PLYR1"
    };
    if let Some(text) = source_message_text(player_label) {
        push_source_message_sprites(
            scene,
            text,
            source_screen_position(SOURCE_PLAYER_START_PROMPT_SCREEN),
            RenderLayer::Overlay,
        );
    }
}

fn should_show_player_start_prompt(state: &GameState) -> bool {
    state.phase == GamePhase::Playing
        && state.player_count > 1
        && state.player.position == (WorldVector::default(), WorldVector::default())
        && state.world == WorldSnapshot::default()
}

fn push_wave_completion_status_sprites(scene: &mut RenderScene, state: &GameState) {
    if !should_show_wave_completion_status(state) {
        return;
    }

    for (label, screen_address) in SOURCE_WAVE_COMPLETION_STATUS_LINES {
        if let Some(text) = source_message_text(label) {
            push_source_message_sprites(
                scene,
                text,
                source_screen_position(*screen_address),
                RenderLayer::Overlay,
            );
        }
    }

    let (wave_digits, wave_digit_count) = source_visible_decimal_digits(state.wave);
    push_source_text_bytes_sprites(
        scene,
        &wave_digits[..wave_digit_count],
        source_screen_position(SOURCE_WAVE_COMPLETION_WAVE_NUMBER_SCREEN),
        RenderLayer::Overlay,
    );

    let (multiplier_digits, multiplier_digit_count) =
        source_visible_decimal_digits(state.wave.min(5));
    push_source_text_bytes_sprites(
        scene,
        &multiplier_digits[..multiplier_digit_count],
        source_screen_position(SOURCE_WAVE_COMPLETION_MULTIPLIER_NUMBER_SCREEN),
        RenderLayer::Overlay,
    );
}

fn push_survivor_bonus_icon_sprites(scene: &mut RenderScene, state: &GameState) {
    if !should_show_wave_completion_status(state) {
        return;
    }

    for index in 0..state
        .world
        .humans
        .len()
        .min(SOURCE_SURVIVOR_BONUS_HUMAN_LIMIT)
    {
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

fn should_show_wave_completion_status(state: &GameState) -> bool {
    state.phase == GamePhase::Playing
        && state.player.position != (WorldVector::default(), WorldVector::default())
        && state.world.enemies.is_empty()
}

fn source_visible_decimal_digits(value: u8) -> ([u8; 2], usize) {
    let value = value.min(99);
    if value < 10 {
        ([b'0' + value, b' '], 1)
    } else {
        ([b'0' + value / 10, b'0' + value % 10], 2)
    }
}

fn push_high_score_entry_prompt_sprites(scene: &mut RenderScene, state: &GameState) {
    if state.phase != GamePhase::HighScoreEntry {
        return;
    }

    let player_label = if state.current_player == 2 {
        "PLYR2"
    } else {
        "PLYR1"
    };
    if let Some(text) = source_message_text(player_label) {
        push_source_message_sprites_with_tint(
            scene,
            text,
            source_screen_position(SOURCE_HALL_OF_FAME_PLAYER_LABEL_SCREEN),
            RenderLayer::Overlay,
            SOURCE_VISUAL_STATE.hall_of_fame_entry_text_tint(),
        );
    }

    for (index, message_label) in SOURCE_HALL_OF_FAME_INSTRUCTION_MESSAGES.iter().enumerate() {
        if let Some(text) = source_message_text(message_label) {
            push_source_message_sprites_with_tint(
                scene,
                text,
                source_screen_position_with_offset(
                    SOURCE_HALL_OF_FAME_INSTRUCTIONS_TOP_LEFT,
                    0,
                    SOURCE_HALL_OF_FAME_LINE_VERTICAL_OFFSETS[index],
                ),
                RenderLayer::Overlay,
                SOURCE_VISUAL_STATE.hall_of_fame_entry_text_tint(),
            );
        }
    }

    for (index, initial) in state.high_score_initials.initials.iter().enumerate() {
        let Some(initial) = initial else {
            continue;
        };
        let Some(sprite) = SpriteId::message_glyph(*initial) else {
            continue;
        };
        let Some(size) = SpriteId::message_glyph_size(*initial) else {
            continue;
        };
        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Overlay,
            position: source_screen_position_with_offset(
                SOURCE_HALL_OF_FAME_INITIALS_SCREEN,
                SOURCE_HALL_OF_FAME_INITIAL_HORIZONTAL_OFFSETS[index],
                0,
            ),
            size: [size[0] as f32, size[1] as f32],
            tint: SOURCE_VISUAL_STATE.hall_of_fame_blink_text_tint(),
        });
    }

    push_high_score_entry_underline_sprites(
        scene,
        usize::from(state.high_score_initials.cursor).min(
            SOURCE_HALL_OF_FAME_INITIAL_HORIZONTAL_OFFSETS
                .len()
                .saturating_sub(1),
        ),
    );
}

fn push_high_score_entry_underline_sprites(scene: &mut RenderScene, active_initial: usize) {
    for initial_index in 0..SOURCE_HALL_OF_FAME_INITIAL_HORIZONTAL_OFFSETS.len() {
        let initial_offset = u8::try_from(initial_index).expect("high-score initial index fits")
            * SOURCE_HALL_OF_FAME_UNDERLINE_INITIAL_STEP;
        let tint = if initial_index == active_initial {
            SOURCE_VISUAL_STATE.hall_of_fame_active_underline_tint()
        } else {
            SOURCE_VISUAL_STATE.hall_of_fame_normal_underline_tint()
        };
        for word_offset in SOURCE_HALL_OF_FAME_UNDERLINE_WORD_HORIZONTAL_OFFSETS {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::HALL_OF_FAME_UNDERLINE_WORD,
                layer: RenderLayer::Overlay,
                position: source_screen_position_with_offset(
                    SOURCE_HALL_OF_FAME_UNDERLINE_SCREEN,
                    initial_offset.wrapping_add(word_offset),
                    0,
                ),
                size: SOURCE_HALL_OF_FAME_UNDERLINE_WORD_SIZE,
                tint,
            });
        }
    }
}

fn push_hall_of_fame_display_sprites(scene: &mut RenderScene, state: &GameState) {
    let shows_attract_hall_of_fame =
        state.phase == GamePhase::Attract && state.attract.shows_hall_of_fame();
    if state.game_over.hall_of_fame_stall_remaining.is_none() && !shows_attract_hall_of_fame {
        return;
    }

    for (message_label, screen_address) in SOURCE_HALL_OF_FAME_DISPLAY_HEADINGS {
        if let Some(text) = source_message_text(message_label) {
            push_source_message_sprites_with_tint(
                scene,
                text,
                source_screen_position(screen_address),
                RenderLayer::Overlay,
                SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint(),
            );
        }
    }

    push_hall_of_fame_display_underline_sprites(scene);
    push_hall_of_fame_defender_logo_sprite(scene);
    push_hall_of_fame_table_sprites(
        scene,
        state.high_score_tables.todays_greatest,
        SOURCE_HALL_OF_FAME_TODAYS_TABLE_SCREEN,
    );
    push_hall_of_fame_table_sprites(
        scene,
        state.high_score_tables.all_time,
        SOURCE_HALL_OF_FAME_ALL_TIME_TABLE_SCREEN,
    );
}

fn push_hall_of_fame_display_underline_sprites(scene: &mut RenderScene) {
    for word_offset in (SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_RIGHT_END
        ..=SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_RIGHT_START)
        .rev()
        .chain((0..=SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_LEFT_START).rev())
    {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::HALL_OF_FAME_UNDERLINE_WORD,
            layer: RenderLayer::Overlay,
            position: source_screen_position_with_offset(
                SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_SCREEN,
                word_offset,
                0,
            ),
            size: SOURCE_HALL_OF_FAME_UNDERLINE_WORD_SIZE,
            tint: SOURCE_VISUAL_STATE.hall_of_fame_normal_underline_tint(),
        });
    }
}

fn push_hall_of_fame_defender_logo_sprite(scene: &mut RenderScene) {
    scene.push_sprite(SceneSprite {
        sprite: SpriteId::HALL_OF_FAME_DEFENDER_LOGO,
        layer: RenderLayer::Overlay,
        position: source_screen_position(SOURCE_HALL_OF_FAME_LOGO_SCREEN),
        size: SOURCE_HALL_OF_FAME_LOGO_SIZE,
        tint: SOURCE_VISUAL_STATE.hall_of_fame_logo_tint(),
    });
}

fn push_hall_of_fame_table_sprites(
    scene: &mut RenderScene,
    entries: [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES],
    top_left_screen_address: u16,
) {
    for (index, entry) in entries.iter().enumerate() {
        let vertical_offset = u8::try_from(index).expect("high-score table index fits in u8")
            * SOURCE_HALL_OF_FAME_TABLE_ROW_STEP;
        let row_rank = b'1' + u8::try_from(index).expect("high-score table index fits in u8");
        push_source_text_bytes_sprites_with_tint(
            scene,
            &[row_rank],
            source_screen_position_with_offset(top_left_screen_address, 0, vertical_offset),
            RenderLayer::Overlay,
            SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint(),
        );
        push_source_text_bytes_sprites_with_tint(
            scene,
            &high_score_initials_text(entry.initials),
            source_screen_position_with_offset(
                top_left_screen_address,
                SOURCE_HALL_OF_FAME_TABLE_INITIALS_OFFSET,
                vertical_offset,
            ),
            RenderLayer::Overlay,
            SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint(),
        );
        push_source_text_bytes_sprites_with_tint(
            scene,
            &hall_of_fame_score_text(entry.score),
            source_screen_position_with_offset(
                top_left_screen_address,
                SOURCE_HALL_OF_FAME_TABLE_SCORE_OFFSET,
                vertical_offset,
            ),
            RenderLayer::Overlay,
            SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint(),
        );
    }
}

fn high_score_initials_text(initials: [Option<char>; 3]) -> [u8; 3] {
    initials.map(|initial| {
        initial
            .filter(|character| character.is_ascii_alphabetic())
            .map(|character| character.to_ascii_uppercase() as u8)
            .unwrap_or(b' ')
    })
}

fn hall_of_fame_score_text(score: u32) -> [u8; SOURCE_HALL_OF_FAME_SCORE_TEXT_LEN] {
    let mut text = [b' '; SOURCE_HALL_OF_FAME_SCORE_TEXT_LEN];
    let mut place = 100_000;
    let mut seen_non_zero = false;
    for byte in &mut text {
        let digit = ((score.min(SCORE_DISPLAY_MAX) / place) % 10) as u8;
        if digit != 0 || seen_non_zero {
            seen_non_zero = true;
            *byte = b'0' + digit;
        }
        place /= 10;
    }
    text
}

fn push_score_sprites(scene: &mut RenderScene, scores: ScoreSnapshot, player_count: u8) {
    push_player_score_sprites(scene, scores.player_one, PLAYER_ONE_SCORE_ORIGIN);
    if player_count > 1 {
        push_player_score_sprites(scene, scores.player_two, PLAYER_TWO_SCORE_ORIGIN);
    }
}

fn push_player_score_sprites(scene: &mut RenderScene, score: u32, origin: [f32; 2]) {
    for (index, digit) in visible_score_digits(score).iter().enumerate() {
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
                origin[0] + SCORE_DIGIT_STEP[0] * index as f32,
                origin[1] + SCORE_DIGIT_STEP[1] * index as f32,
            ],
            size: SCORE_DIGIT_SIZE,
            tint: SOURCE_VISUAL_STATE.hud_tint(),
        });
    }
}

fn visible_score_digits(score: u32) -> [Option<u8>; SCORE_DIGIT_DISPLAY_COUNT] {
    let score = score.min(SCORE_DISPLAY_MAX);
    let mut place = 100_000;
    let mut digits = [None; SCORE_DIGIT_DISPLAY_COUNT];
    let mut non_zero_seen = false;

    for (index, digit) in digits.iter_mut().enumerate() {
        let value = ((score / place) % 10) as u8;
        let counter = SCORE_DIGIT_DISPLAY_COUNT - index;
        if value == 0 && counter > 2 && !non_zero_seen {
            *digit = None;
        } else {
            non_zero_seen = true;
            *digit = Some(value);
        }
        place /= 10;
    }

    digits
}

fn push_stock_sprites(
    scene: &mut RenderScene,
    player_count: u8,
    player_stocks: [PlayerStockSnapshot; 2],
) {
    push_player_stock_sprites(
        scene,
        player_stocks[0],
        PLAYER_ONE_LIFE_STOCK_ORIGIN,
        PLAYER_ONE_SMART_BOMB_STOCK_ORIGIN,
    );
    if player_count > 1 {
        push_player_stock_sprites(
            scene,
            player_stocks[1],
            PLAYER_TWO_LIFE_STOCK_ORIGIN,
            PLAYER_TWO_SMART_BOMB_STOCK_ORIGIN,
        );
    }
}

fn push_player_stock_sprites(
    scene: &mut RenderScene,
    stock: PlayerStockSnapshot,
    life_origin: [f32; 2],
    smart_bomb_origin: [f32; 2],
) {
    push_stock_sprite_series(
        scene,
        SpriteId::PLAYER_LIFE_STOCK,
        stock.lives.min(PLAYER_LIFE_STOCK_DISPLAY_LIMIT),
        life_origin,
        PLAYER_LIFE_STOCK_STEP,
        PLAYER_LIFE_STOCK_SIZE,
    );
    push_stock_sprite_series(
        scene,
        SpriteId::SMART_BOMB_STOCK,
        stock.smart_bombs.min(SMART_BOMB_STOCK_DISPLAY_LIMIT),
        smart_bomb_origin,
        SMART_BOMB_STOCK_STEP,
        SMART_BOMB_STOCK_SIZE,
    );
}

fn push_stock_sprite_series(
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
            tint: SOURCE_VISUAL_STATE.hud_tint(),
        });
    }
}

const PLAYER_LIFE_STOCK_DISPLAY_LIMIT: u8 = 5;
const SMART_BOMB_STOCK_DISPLAY_LIMIT: u8 = 3;
const SCORE_DIGIT_DISPLAY_COUNT: usize = 6;
const SCORE_DISPLAY_MAX: u32 = 999_999;
const PLAYER_ONE_SCORE_ORIGIN: [f32; 2] = [18.0, 21.0];
const PLAYER_TWO_SCORE_ORIGIN: [f32; 2] = [214.0, 21.0];
const SCORE_DIGIT_STEP: [f32; 2] = [8.0, 0.0];
const SCORE_DIGIT_SIZE: [f32; 2] = [6.0, 8.0];
const PLAYER_ONE_LIFE_STOCK_ORIGIN: [f32; 2] = [18.0, 13.0];
const PLAYER_TWO_LIFE_STOCK_ORIGIN: [f32; 2] = [214.0, 13.0];
const PLAYER_LIFE_STOCK_STEP: [f32; 2] = [12.0, 0.0];
const PLAYER_LIFE_STOCK_SIZE: [f32; 2] = [10.0, 4.0];
const PLAYER_ONE_SMART_BOMB_STOCK_ORIGIN: [f32; 2] = [70.0, 20.0];
const PLAYER_TWO_SMART_BOMB_STOCK_ORIGIN: [f32; 2] = [266.0, 20.0];
const SMART_BOMB_STOCK_STEP: [f32; 2] = [0.0, 4.0];
const SMART_BOMB_STOCK_SIZE: [f32; 2] = [6.0, 3.0];
const SOURCE_TOP_DISPLAY_BORDER_SEGMENTS: &[(u16, [f32; 2])] = &[
    (0x0028, [312.0, 2.0]),
    (0x2F08, [2.0, 32.0]),
    (0x7008, [2.0, 32.0]),
    (0x2F07, [130.0, 1.0]),
    (0x4C07, [16.0, 2.0]),
    (0x4C28, [16.0, 2.0]),
];
const SOURCE_ATTRACT_CREDITS_LABEL_SCREEN: u16 = 0x28E5;
const SOURCE_ATTRACT_CREDITS_NUMBER_SCREEN: u16 = 0x48E5;
const SOURCE_ATTRACT_PRESENTS_ELECTRONICS_SCREEN: u16 = 0x3258;
const SOURCE_ATTRACT_INSTRUCTION_TEXT_LINES: &[(&str, u16)] = &[
    ("SCANV", 0x4330),
    ("LANDV", 0x1C70),
    ("MUTV", 0x3C70),
    ("BAITV", 0x5F70),
    ("BOMBV", 0x1CA8),
    ("SWRMPV", 0x40A8),
    ("SWARMV", 0x5CA8),
];
const SOURCE_ATTRACT_WILLIAMS_LOGO_SCREEN: u16 = 0x363C;
const SOURCE_ATTRACT_WILLIAMS_LOGO_SIZE: [f32; 2] = [92.0, 19.0];
const SOURCE_ATTRACT_DEFENDER_WORDMARK_SCREEN: u16 = 0x3090;
const SOURCE_DEFENDER_WORDMARK_SIZE: [f32; 2] = [120.0, 24.0];
const SOURCE_ATTRACT_COPYRIGHT_STRIP_SCREEN: u16 = 0x3BD0;
const SOURCE_ATTRACT_COPYRIGHT_STRIP_SIZE: [f32; 2] = [80.0, 8.0];
const SOURCE_FINAL_GAME_OVER_SCREEN: u16 = 0x3E80;
const SOURCE_PLAYER_START_PROMPT_SCREEN: u16 = 0x3C80;
const SOURCE_PLAYER_SWITCH_LABEL_SCREEN: u16 = 0x3C78;
const SOURCE_PLAYER_SWITCH_GAME_OVER_SCREEN: u16 = 0x3E88;
const SOURCE_WAVE_COMPLETION_STATUS_LINES: &[(&str, u16)] =
    &[("ATWV", 0x3850), ("COMPV", 0x3D60), ("BONSX", 0x3C90)];
const SOURCE_WAVE_COMPLETION_WAVE_NUMBER_SCREEN: u16 = 0x6550;
const SOURCE_WAVE_COMPLETION_MULTIPLIER_NUMBER_SCREEN: u16 = 0x5890;
const SOURCE_SURVIVOR_BONUS_FIRST_HUMAN_SCREEN: u16 = 0x3CA0;
const SOURCE_SURVIVOR_BONUS_HUMAN_STEP: u8 = 0x04;
const SOURCE_SURVIVOR_BONUS_HUMAN_LIMIT: usize = 10;
const SOURCE_SURVIVOR_BONUS_HUMAN_SIZE: [f32; 2] = [4.0, 8.0];
const SOURCE_HALL_OF_FAME_PLAYER_LABEL_SCREEN: u16 = 0x3E38;
const SOURCE_HALL_OF_FAME_INSTRUCTIONS_TOP_LEFT: u16 = 0x1458;
const SOURCE_HALL_OF_FAME_INITIALS_SCREEN: u16 = 0x46AC;
const SOURCE_HALL_OF_FAME_UNDERLINE_SCREEN: u16 = 0x45B7;
const SOURCE_HALL_OF_FAME_LINE_VERTICAL_OFFSETS: [u8; 4] = [0x00, 0x0A, 0x1E, 0x32];
const SOURCE_HALL_OF_FAME_INITIAL_HORIZONTAL_OFFSETS: [u8; 3] = [0x00, 0x08, 0x10];
const SOURCE_HALL_OF_FAME_UNDERLINE_INITIAL_STEP: u8 = 0x08;
const SOURCE_HALL_OF_FAME_UNDERLINE_WORD_HORIZONTAL_OFFSETS: [u8; 4] = [0x04, 0x03, 0x02, 0x01];
const SOURCE_HALL_OF_FAME_UNDERLINE_WORD_SIZE: [f32; 2] = [2.0, 2.0];
const SOURCE_HALL_OF_FAME_INSTRUCTION_MESSAGES: [&str; 4] = ["HOFV1", "HOFV2", "HOFV3", "HOFV4"];
const SOURCE_HALL_OF_FAME_DISPLAY_HEADINGS: [(&str, u16); 5] = [
    ("HALLD_TITLE", 0x3854),
    ("HALLD_TODAYS", 0x2268),
    ("HALLD_ALL_TIME", 0x6068),
    ("HALLD_GREATEST", 0x1E72),
    ("HALLD_GREATEST", 0x5F72),
];
const SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_SCREEN: u16 = 0x1E7B;
const SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_RIGHT_START: u8 = 0x5F;
const SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_RIGHT_END: u8 = 0x41;
const SOURCE_HALL_OF_FAME_DISPLAY_UNDERLINE_LEFT_START: u8 = 0x1E;
const SOURCE_HALL_OF_FAME_LOGO_SCREEN: u16 = 0x3038;
const SOURCE_HALL_OF_FAME_LOGO_SIZE: [f32; 2] = SOURCE_DEFENDER_WORDMARK_SIZE;
const SOURCE_HALL_OF_FAME_TODAYS_TABLE_SCREEN: u16 = 0x1886;
const SOURCE_HALL_OF_FAME_ALL_TIME_TABLE_SCREEN: u16 = 0x5986;
const SOURCE_HALL_OF_FAME_TABLE_ROW_STEP: u8 = 0x0A;
const SOURCE_HALL_OF_FAME_TABLE_INITIALS_OFFSET: u8 = 0x05;
const SOURCE_HALL_OF_FAME_TABLE_SCORE_OFFSET: u8 = 0x13;
const SOURCE_HALL_OF_FAME_SCORE_TEXT_LEN: usize = 6;

fn world_vector_pixels(vector: WorldVector) -> f32 {
    vector.subpixels() as f32 / WorldVector::SUBPIXELS_PER_PIXEL as f32
}

fn adapt_phase(phase: AcceptedPhase) -> GamePhase {
    match phase {
        AcceptedPhase::Attract => GamePhase::Attract,
        AcceptedPhase::Playing => GamePhase::Playing,
        AcceptedPhase::GameOver => GamePhase::GameOver,
        AcceptedPhase::HighScoreEntry => GamePhase::HighScoreEntry,
    }
}

fn adapt_direction(direction: AcceptedDirection) -> Direction {
    match direction {
        AcceptedDirection::Left => Direction::Left,
        AcceptedDirection::Right => Direction::Right,
    }
}

fn adapt_event(event: AcceptedEvent) -> GameEvent {
    match event {
        AcceptedEvent::CreditAdded => GameEvent::CreditAdded,
        AcceptedEvent::GameStarted => GameEvent::GameStarted,
        AcceptedEvent::DiagnosticsSelected => GameEvent::DiagnosticsSelected,
        AcceptedEvent::AuditsSelected => GameEvent::AuditsSelected,
        AcceptedEvent::HighScoreReset => GameEvent::HighScoreReset,
        AcceptedEvent::ReversePressed => GameEvent::ReversePressed,
        AcceptedEvent::FirePressed => GameEvent::FirePressed,
        AcceptedEvent::SmartBombPressed => GameEvent::SmartBombPressed,
        AcceptedEvent::HyperspacePressed => GameEvent::HyperspacePressed,
        AcceptedEvent::BonusAwarded => GameEvent::BonusAwarded,
        AcceptedEvent::HighScoreEntryStarted => GameEvent::HighScoreEntryStarted,
        AcceptedEvent::HighScoreInitialAccepted => GameEvent::HighScoreInitialAccepted,
        AcceptedEvent::HighScoreSubmitted => GameEvent::HighScoreSubmitted,
    }
}

fn adapt_sound_command(command: u8) -> SoundEvent {
    match command {
        0xC0 => SoundEvent::Startup,
        0xE6 => SoundEvent::CreditAdded,
        0xF5 => SoundEvent::GameStarted,
        0xE9 => SoundEvent::ThrustStarted,
        0xF0 => SoundEvent::ThrustStopped,
        command => SoundEvent::UnmappedSoundCommand { command },
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::{GameEvent, GameFrame, GameInput, GameState, RenderScene, SoundEvent};
    use crate::accepted::{AcceptedEvent, AcceptedGameplayMachine, AcceptedSnapshot};

    #[derive(Debug)]
    pub(crate) struct ReferenceFrameProbe {
        machine: AcceptedGameplayMachine,
    }

    impl ReferenceFrameProbe {
        pub(crate) fn new() -> Self {
            Self {
                machine: AcceptedGameplayMachine::new(),
            }
        }

        pub(crate) fn step(&mut self, input: GameInput) -> GameFrame {
            super::adapt_frame_output(self.machine.step(input))
        }
    }

    pub(crate) fn adapt_accepted_snapshot(snapshot: AcceptedSnapshot) -> GameState {
        super::adapt_snapshot(snapshot)
    }

    pub(crate) fn adapt_accepted_event(event: AcceptedEvent) -> GameEvent {
        super::adapt_event(event)
    }

    pub(crate) fn adapt_accepted_scene(
        state: &GameState,
        visual_signature: Option<u32>,
    ) -> RenderScene {
        super::adapt_scene(state, visual_signature)
    }

    pub(crate) fn adapt_accepted_sound_command(command: u8) -> SoundEvent {
        super::adapt_sound_command(command)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        accepted::{
            AcceptedDirection, AcceptedEvent, AcceptedExpandedObjectDetail,
            AcceptedExpandedObjectEvidence, AcceptedExpandedObjectKind, AcceptedGameOverState,
            AcceptedHighScoreEntry, AcceptedHighScoreInitials, AcceptedHighScoreSubmission,
            AcceptedHighScoreTableEntry, AcceptedHighScoreTables, AcceptedObjectEvidence,
            AcceptedObjectEvidenceDetail, AcceptedObjectEvidenceList, AcceptedPhase,
            AcceptedPlayer, AcceptedPlayerStock, AcceptedScores, AcceptedSnapshot,
            AcceptedWaveProfile,
        },
        game::{
            ATTRACT_DEFENDER_WORDMARK_REVEAL_FRAMES, ATTRACT_DEFENDER_WORDMARK_START_FRAME,
            ATTRACT_SCORING_SEQUENCE_START_FRAME, AttractPresentationSnapshot,
            EXPANDED_OBJECT_DETAIL_LIMIT, ExpandedObjectDetailSnapshot,
            ExpandedObjectEvidenceSnapshot, ExpandedObjectKind, GameOverSnapshot,
            HighScoreEntrySnapshot, HighScoreSubmissionSnapshot, HighScoreTableEntrySnapshot,
            HighScoreTablesSnapshot, OBJECT_EVIDENCE_DETAIL_LIMIT, ObjectEvidenceDetailSnapshot,
            ObjectEvidenceList, ObjectEvidenceSnapshot, PlayerStockSnapshot,
            SOURCE_EXPLOSION_INITIAL_SIZE, SOURCE_EXPLOSION_LIFETIME_FRAMES,
            SOURCE_EXPLOSION_SIZE_DELTA, SOURCE_SCORE_POPUP_LIFETIME_TICKS, SOURCE_VISUAL_STATE,
            ScannerRadarSnapshot, TerrainBlowSnapshot, TerrainBlowStage, WaveProfileSnapshot,
            WorldVector,
        },
        renderer::{Color, RenderLayer, SpriteId},
        systems::{
            GameSimulation, HighScoreInitialsState, PlayerActionTriggers, advance_one_frame,
        },
    };

    use super::{
        Direction, GameEvent, GameInput, GamePhase, GameplayOracle, SoundEvent, adapt_direction,
        adapt_event, adapt_expanded_objects, adapt_game_over, adapt_high_score_entry,
        adapt_high_score_initials, adapt_high_score_submission, adapt_high_score_table_entry,
        adapt_high_score_tables, adapt_object_evidence, adapt_phase, adapt_sound_command,
        adapt_wave_profile,
    };

    #[test]
    fn oracle_starts_from_clean_attract_snapshot() {
        let oracle = GameplayOracle::new();
        let snapshot = oracle.snapshot();

        assert_eq!(snapshot.frame, 0);
        assert_eq!(snapshot.phase, GamePhase::Attract);
        assert_eq!(snapshot.current_player, 1);
    }

    #[test]
    fn oracle_steps_through_clean_frame_contract() {
        let mut oracle = GameplayOracle::new();
        let frame = oracle.step(GameInput::NONE);

        assert_eq!(frame.state.frame, 1);
        assert_eq!(frame.state.phase, GamePhase::Attract);
        assert_eq!(frame.scene.summary().frame, 1);
    }

    #[test]
    fn oracle_scene_projects_attract_credit_text_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::Attract;
        state.credits = 12;
        state.attract = AttractPresentationSnapshot::for_page_frame(
            ATTRACT_DEFENDER_WORDMARK_START_FRAME + ATTRACT_DEFENDER_WORDMARK_REVEAL_FRAMES,
        );

        let scene = super::adapt_scene(&state, Some(0xC0ED_0012));
        let overlay_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.layer == RenderLayer::Overlay)
            .collect::<Vec<_>>();

        assert_eq!(scene.visual_signature, Some(0xC0ED_0012));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO
                && sprite.position == [108.0, 60.0]
                && sprite.size == [92.0, 19.0]
                && sprite.tint
                    == crate::game::SOURCE_VISUAL_STATE
                        .attract_williams_logo_tint_for_frame(state.attract.page_frame)
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position == [96.0, 144.0]
                && sprite.size == [120.0, 24.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(
            !overlay_sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ATTRACT_COPYRIGHT_STRIP)
        );
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_C
                && sprite.position == [80.0, 229.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_COLON
                && sprite.position == [134.0, 229.0]
                && sprite.size == [2.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.position == [144.0, 229.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_2
                && sprite.position == [152.0, 229.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E
                && sprite.position == [100.0, 88.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.position == [124.0, 108.0]
                && sprite.size == [6.0, 8.0]
        }));

        state.attract =
            AttractPresentationSnapshot::for_page_frame(ATTRACT_DEFENDER_WORDMARK_START_FRAME);
        let early_defender_scene = super::adapt_scene(&state, Some(0xC0ED_2850));
        assert!(early_defender_scene.raster().is_none());
        assert!(early_defender_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(
            !early_defender_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO)
        );

        state.attract = AttractPresentationSnapshot::for_page_frame(488);
        let hall_scene = super::adapt_scene(&state, Some(0xC0ED_4410));
        let hall_sprites = hall_scene
            .sprites
            .iter()
            .filter(|sprite| sprite.layer == RenderLayer::Overlay)
            .collect::<Vec<_>>();
        assert!(hall_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position == [96.0, 56.0]
                && sprite.size == [120.0, 24.0]
        }));
        assert!(hall_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_H
                && sprite.position == [112.0, 84.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(hall_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_C
                && sprite.position == [80.0, 229.0]
                && sprite.tint == crate::game::SOURCE_VISUAL_STATE.hall_of_fame_display_text_tint()
        }));

        state.attract =
            AttractPresentationSnapshot::for_page_frame(ATTRACT_SCORING_SEQUENCE_START_FRAME);
        let scoring_scene = super::adapt_scene(&state, Some(0xC0ED_E370));
        let scoring_sprites = scoring_scene
            .sprites
            .iter()
            .filter(|sprite| sprite.layer == RenderLayer::Overlay)
            .collect::<Vec<_>>();
        assert!(scoring_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_S
                && sprite.position == [134.0, 48.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(scoring_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_L
                && sprite.position == [56.0, 112.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(!scoring_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO
                || sprite.sprite == SpriteId::ATTRACT_COPYRIGHT_STRIP
        }));
    }

    #[test]
    fn oracle_scene_projects_current_player_stock_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::Playing;
        state.player.lives = 6;
        state.player.smart_bombs = 4;
        state.player_stocks[0] = PlayerStockSnapshot::new(6, 4);

        let scene = super::adapt_scene(&state, Some(0x1234_5678));
        let life_stock = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::PLAYER_LIFE_STOCK)
            .collect::<Vec<_>>();
        let smart_bomb_stock = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::SMART_BOMB_STOCK)
            .collect::<Vec<_>>();

        assert_eq!(scene.summary().visual_signature, Some(0x1234_5678));
        assert_eq!(life_stock.len(), 5);
        assert_eq!(smart_bomb_stock.len(), 3);
        assert_eq!(life_stock[0].position, [18.0, 13.0]);
        assert_eq!(life_stock[4].position, [66.0, 13.0]);
        assert_eq!(life_stock[0].size, [10.0, 4.0]);
        assert_eq!(smart_bomb_stock[0].position, [70.0, 20.0]);
        assert_eq!(smart_bomb_stock[2].position, [70.0, 28.0]);
        assert_eq!(smart_bomb_stock[0].size, [6.0, 3.0]);
        assert!(
            life_stock
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Hud)
        );
        assert!(
            smart_bomb_stock
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Hud)
        );
    }

    #[test]
    fn oracle_scene_projects_source_top_display_border() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::Playing;

        let scene = super::adapt_scene(&state, Some(0xB0AD_EA11));
        let border_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD)
            .collect::<Vec<_>>();

        assert_eq!(scene.visual_signature, Some(0xB0AD_EA11));
        assert_eq!(border_sprites.len(), 6);
        assert!(
            border_sprites
                .iter()
                .filter(|sprite| {
                    sprite.position != [152.0, 7.0] && sprite.position != [152.0, 40.0]
                })
                .all(|sprite| {
                    sprite.layer == RenderLayer::Hud
                        && sprite.tint == SOURCE_VISUAL_STATE.top_display_border_tint()
                })
        );
        assert!(border_sprites.iter().any(|sprite| {
            sprite.position == [152.0, 7.0]
                && sprite.tint == SOURCE_VISUAL_STATE.top_display_scanner_marker_tint()
        }));
        assert!(border_sprites.iter().any(|sprite| {
            sprite.position == [152.0, 40.0]
                && sprite.tint == SOURCE_VISUAL_STATE.top_display_scanner_marker_tint()
        }));
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [0.0, 40.0] && sprite.size == [312.0, 2.0] })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [94.0, 8.0] && sprite.size == [2.0, 32.0] })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [224.0, 8.0] && sprite.size == [2.0, 32.0] })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [94.0, 7.0] && sprite.size == [130.0, 1.0] })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [152.0, 7.0] && sprite.size == [16.0, 2.0] })
        );
        assert!(
            border_sprites
                .iter()
                .any(|sprite| { sprite.position == [152.0, 40.0] && sprite.size == [16.0, 2.0] })
        );
    }

    #[test]
    fn oracle_scene_projects_scanner_radar_sprites() {
        let mut details = [ObjectEvidenceDetailSnapshot::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT];
        details[0] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Active,
            world_position: Some((0x2100, 0x5000)),
            scanner_color: Some(0x1234),
            ..ObjectEvidenceDetailSnapshot::EMPTY
        };
        let evidence = ObjectEvidenceSnapshot {
            active_count: 1,
            inactive_count: 0,
            projectile_count: 0,
            visible_count: 1,
            evidence_crc32: None,
            detail_count: 1,
            details,
        };
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::Playing;
        state.world.scanner = ScannerRadarSnapshot::for_world(
            GamePhase::Playing,
            4,
            WorldVector::from_subpixels(0x2000_i32 << 8),
            (
                WorldVector::from_subpixels(0x8000_i32 << 8),
                WorldVector::from_subpixels(0x4000_i32 << 8),
            ),
            &evidence,
        );

        let scene = super::adapt_scene(&state, Some(0x5CA4_4E11));

        assert_eq!(scene.visual_signature, Some(0x5CA4_4E11));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_OBJECT_BLIP
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [151.0, 17.0]
                && sprite.size == [1.0, 1.0]
                && sprite.tint == SOURCE_VISUAL_STATE.scanner_object_blip_tint(0x0002)
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_OBJECT_BLIP
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [150.0, 18.0]
                && sprite.size == [1.0, 1.0]
                && sprite.tint == SOURCE_VISUAL_STATE.scanner_object_blip_tint(0x0003)
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_OBJECT_BLIP
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [151.0, 18.0]
                && sprite.size == [1.0, 1.0]
                && sprite.tint == SOURCE_VISUAL_STATE.scanner_object_blip_tint(0x0004)
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_PLAYER_BLIP
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [166.0, 15.0]
                && sprite.size == [1.0, 1.0]
                && sprite.tint == SOURCE_VISUAL_STATE.scanner_player_blip_tint(0x0009)
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_PLAYER_BLIP
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [167.0, 16.0]
                && sprite.size == [1.0, 1.0]
                && sprite.tint == SOURCE_VISUAL_STATE.scanner_player_blip_tint(0x0009)
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_PLAYER_BLIP
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [166.0, 17.0]
                && sprite.size == [1.0, 1.0]
                && sprite.tint == SOURCE_VISUAL_STATE.scanner_player_blip_tint(0x0009)
        }));
    }

    #[test]
    fn oracle_scene_projects_source_object_detail_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::Playing;
        state.world.object_evidence = ObjectEvidenceSnapshot {
            active_count: 3,
            inactive_count: 1,
            projectile_count: 1,
            visible_count: 3,
            evidence_crc32: Some(0x0B1E_C7ED),
            detail_count: 4,
            details: {
                let mut details =
                    [ObjectEvidenceDetailSnapshot::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT];
                details[0] = ObjectEvidenceDetailSnapshot {
                    list: ObjectEvidenceList::Active,
                    screen_position: Some(crate::systems::ScreenPosition::new(40, 50)),
                    picture_size: Some((6, 4)),
                    mapped_sprite: Some(SpriteId::ENEMY_BAITER),
                    ..ObjectEvidenceDetailSnapshot::EMPTY
                };
                details[1] = ObjectEvidenceDetailSnapshot {
                    list: ObjectEvidenceList::Projectile,
                    screen_position: Some(crate::systems::ScreenPosition::new(60, 70)),
                    picture_size: Some((8, 1)),
                    mapped_sprite: Some(SpriteId::PLAYER_PROJECTILE),
                    ..ObjectEvidenceDetailSnapshot::EMPTY
                };
                details[2] = ObjectEvidenceDetailSnapshot {
                    list: ObjectEvidenceList::Inactive,
                    screen_position: Some(crate::systems::ScreenPosition::new(80, 90)),
                    picture_size: Some((2, 3)),
                    mapped_sprite: Some(SpriteId::ENEMY_BOMB),
                    ..ObjectEvidenceDetailSnapshot::EMPTY
                };
                details[3] = ObjectEvidenceDetailSnapshot {
                    list: ObjectEvidenceList::Active,
                    screen_position: Some(crate::systems::ScreenPosition::new(100, 110)),
                    picture_size: Some((1, 1)),
                    mapped_sprite: Some(SpriteId::NULL_OBJECT),
                    ..ObjectEvidenceDetailSnapshot::EMPTY
                };
                details
            },
        };

        let scene = super::adapt_scene(&state, Some(0x0B1E_C7ED));

        assert_eq!(scene.visual_signature, Some(0x0B1E_C7ED));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_BAITER
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [40.0, 50.0]
                && sprite.size == [6.0, 4.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.layer == RenderLayer::Projectiles
                && sprite.position == [60.0, 70.0]
                && sprite.size == [8.0, 1.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(
            !scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_BOMB)
        );
        assert!(
            !scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::NULL_OBJECT)
        );
    }

    #[test]
    fn oracle_scene_projects_source_expanded_object_detail_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::Playing;
        state.world.expanded_objects = ExpandedObjectEvidenceSnapshot {
            active_count: 5,
            last_slot_address: Some(0x9C40),
            detail_count: 5,
            details: {
                let mut details =
                    [ExpandedObjectDetailSnapshot::EMPTY; EXPANDED_OBJECT_DETAIL_LIMIT];
                details[0] = ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::Appearance,
                    slot_address: Some(0x9C00),
                    descriptor_address: Some(0xF9C1),
                    picture_label: Some("PLAPIC"),
                    picture_size: Some((8, 6)),
                    mapped_sprite: Some(SpriteId::PLAYER_SHIP),
                    top_left: Some(crate::systems::ScreenPosition::new(10, 20)),
                    object_address: Some(0xA23C),
                    ..ExpandedObjectDetailSnapshot::EMPTY
                };
                details[1] = ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::Explosion,
                    slot_address: Some(0x9C40),
                    size: SOURCE_EXPLOSION_INITIAL_SIZE + SOURCE_EXPLOSION_SIZE_DELTA * 2,
                    descriptor_address: Some(0xF951),
                    picture_label: Some("BXPIC"),
                    picture_size: Some((4, 8)),
                    mapped_sprite: Some(SpriteId::BOMB_EXPLOSION),
                    top_left: Some(crate::systems::ScreenPosition::new(30, 40)),
                    explosion_frame: Some(2),
                    explosion_lifetime_frames: Some(SOURCE_EXPLOSION_LIFETIME_FRAMES),
                    ..ExpandedObjectDetailSnapshot::EMPTY
                };
                details[2] = ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::Appearance,
                    picture_size: Some((1, 1)),
                    mapped_sprite: Some(SpriteId::NULL_OBJECT),
                    top_left: Some(crate::systems::ScreenPosition::new(50, 60)),
                    ..ExpandedObjectDetailSnapshot::EMPTY
                };
                details[3] = ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::Explosion,
                    mapped_sprite: Some(SpriteId::ENEMY_BOMB),
                    top_left: Some(crate::systems::ScreenPosition::new(70, 80)),
                    ..ExpandedObjectDetailSnapshot::EMPTY
                };
                details[4] = ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::ScorePopup,
                    picture_label: Some("C25P1"),
                    picture_size: Some((6, 6)),
                    mapped_sprite: Some(SpriteId::SCORE_POPUP_250),
                    top_left: Some(crate::systems::ScreenPosition::new(90, 100)),
                    score_popup_lifetime_ticks: Some(SOURCE_SCORE_POPUP_LIFETIME_TICKS),
                    score_popup_value: Some(250),
                    ..ExpandedObjectDetailSnapshot::EMPTY
                };
                details
            },
        };

        let scene = super::adapt_scene(&state, Some(0xE0B1_EC75));

        assert_eq!(scene.visual_signature, Some(0xE0B1_EC75));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_SHIP
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [10.0, 20.0]
                && sprite.size == [8.0, 6.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::BOMB_EXPLOSION
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [26.0, 36.0]
                && sprite.size == [16.0, 16.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_POPUP_250
                && sprite.layer == RenderLayer::Objects
                && sprite.position == [90.0, 100.0]
                && sprite.size == [6.0, 6.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(
            !scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::NULL_OBJECT)
        );
        assert!(!scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_BOMB && sprite.position == [70.0, 80.0]
        }));
    }

    #[test]
    fn oracle_scene_projects_score_digits_with_arcade_blanking() {
        let mut state = crate::game::Game::new().state();
        state.player_count = 2;
        state.scores.player_one = 987_654;
        state.scores.player_two = 90;

        let scene = super::adapt_scene(&state, Some(0x2468_1357));
        let score_digits = scene
            .sprites
            .iter()
            .filter(|sprite| {
                sprite.layer == RenderLayer::Hud && SpriteId::SCORE_DIGITS.contains(&sprite.sprite)
            })
            .collect::<Vec<_>>();

        assert_eq!(score_digits.len(), 8);
        assert_eq!(
            score_digits
                .iter()
                .map(|sprite| sprite.sprite)
                .collect::<Vec<_>>(),
            vec![
                SpriteId::SCORE_DIGIT_9,
                SpriteId::SCORE_DIGIT_8,
                SpriteId::SCORE_DIGIT_7,
                SpriteId::SCORE_DIGIT_6,
                SpriteId::SCORE_DIGIT_5,
                SpriteId::SCORE_DIGIT_4,
                SpriteId::SCORE_DIGIT_9,
                SpriteId::SCORE_DIGIT_0,
            ]
        );
        assert_eq!(score_digits[0].position, [18.0, 21.0]);
        assert_eq!(score_digits[5].position, [58.0, 21.0]);
        assert_eq!(score_digits[6].position, [246.0, 21.0]);
        assert_eq!(score_digits[7].position, [254.0, 21.0]);
        assert!(
            score_digits
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Hud && sprite.size == [6.0, 8.0])
        );
    }

    #[test]
    fn oracle_scene_projects_player_switch_prompt_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::GameOver;
        state.game_over = GameOverSnapshot {
            player_switch_sleep_remaining: Some(0x60),
            player_switch_from: Some(2),
            player_switch_to: Some(1),
            ..GameOverSnapshot::NONE
        };

        let scene = super::adapt_scene(&state, Some(0xAABB_CCDD));
        let prompt_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| SpriteId::MESSAGE_GLYPHS.contains(&sprite.sprite))
            .collect::<Vec<_>>();

        assert_eq!(scene.visual_signature, Some(0xAABB_CCDD));
        assert_eq!(prompt_sprites.len(), 17);
        assert!(
            prompt_sprites
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Overlay)
        );
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_W
                && sprite.position == [180.0, 120.0]
                && sprite.size == [8.0, 8.0]
        }));
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_G
                && sprite.position == [124.0, 136.0]
                && sprite.size == [6.0, 8.0]
        }));
    }

    #[test]
    fn oracle_scene_projects_player_start_prompt_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::Playing;
        state.current_player = 2;
        state.player_count = 2;

        let scene = super::adapt_scene(&state, Some(0x571A_2702));
        let prompt_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| SpriteId::MESSAGE_GLYPHS.contains(&sprite.sprite))
            .collect::<Vec<_>>();

        assert_eq!(scene.visual_signature, Some(0x571A_2702));
        assert_eq!(prompt_sprites.len(), 9);
        assert!(
            prompt_sprites
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Overlay)
        );
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.position == [120.0, 128.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_W
                && sprite.position == [180.0, 128.0]
                && sprite.size == [8.0, 8.0]
        }));
    }

    #[test]
    fn oracle_scene_projects_wave_completion_status_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::Playing;
        state.wave = 3;
        state.player.position = (
            crate::game::WorldVector::from_subpixels(1),
            crate::game::WorldVector::from_subpixels(1),
        );
        state.world.humans = vec![
            crate::game::HumanSnapshot::new(crate::systems::ScreenPosition::new(72, 216)),
            crate::game::HumanSnapshot::new(crate::systems::ScreenPosition::new(180, 218)),
        ];

        let scene = super::adapt_scene(&state, Some(0x2A7E_C003));
        let overlay_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.layer == RenderLayer::Overlay)
            .collect::<Vec<_>>();

        assert_eq!(scene.visual_signature, Some(0x2A7E_C003));
        assert_eq!(overlay_sprites.len(), 29);
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_A
                && sprite.position == [112.0, 80.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_3
                && sprite.position == [202.0, 80.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_C
                && sprite.position == [122.0, 96.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_B
                && sprite.position == [120.0, 144.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_3
                && sprite.position == [176.0, 144.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.position == [120.0, 160.0]
                && sprite.size == [4.0, 8.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.position == [128.0, 160.0]
                && sprite.size == [4.0, 8.0]
                && sprite.tint == Color::WHITE
        }));
    }

    #[test]
    fn oracle_scene_projects_final_game_over_prompt_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::GameOver;
        state.game_over = GameOverSnapshot {
            player_death_sleep_remaining: Some(40),
            ..GameOverSnapshot::NONE
        };

        let scene = super::adapt_scene(&state, Some(0x0BAD_CAFE));
        let prompt_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| SpriteId::MESSAGE_GLYPHS.contains(&sprite.sprite))
            .collect::<Vec<_>>();

        assert_eq!(scene.visual_signature, Some(0x0BAD_CAFE));
        assert_eq!(prompt_sprites.len(), 8);
        assert!(
            prompt_sprites
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Overlay)
        );
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_G
                && sprite.position == [124.0, 128.0]
                && sprite.size == [6.0, 8.0]
        }));
    }

    #[test]
    fn oracle_scene_projects_high_score_entry_prompt_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::HighScoreEntry;
        state.current_player = 2;
        state.high_score_entry = Some(HighScoreEntrySnapshot {
            score: 25_000,
            rank: 1,
        });
        state.high_score_initials = HighScoreInitialsState {
            initials: [Some('A'), None, Some('Z')],
            cursor: 2,
        };

        let scene = super::adapt_scene(&state, Some(0x51A7_EE00));
        let prompt_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| SpriteId::MESSAGE_GLYPHS.contains(&sprite.sprite))
            .collect::<Vec<_>>();
        let underline_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::HALL_OF_FAME_UNDERLINE_WORD)
            .collect::<Vec<_>>();

        assert_eq!(scene.visual_signature, Some(0x51A7_EE00));
        assert_eq!(prompt_sprites.len(), 103);
        assert_eq!(underline_sprites.len(), 12);
        assert!(
            prompt_sprites
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Overlay)
        );
        assert!(
            underline_sprites
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Overlay && sprite.size == [2.0, 2.0])
        );
        assert!(underline_sprites.iter().any(|sprite| {
            sprite.position == [140.0, 183.0]
                && sprite.tint == Color::from_rgba(0x66, 0x66, 0x66, 0xFF)
        }));
        assert!(
            underline_sprites
                .iter()
                .any(|sprite| { sprite.position == [172.0, 183.0] && sprite.tint == Color::WHITE })
        );
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_W
                && sprite.position == [184.0, 56.0]
                && sprite.size == [8.0, 8.0]
        }));
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_Y
                && sprite.position == [40.0, 88.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_T
                && sprite.position == [40.0, 98.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_S
                && sprite.position == [40.0, 118.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.position == [40.0, 138.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_A
                && sprite.position == [140.0, 172.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(prompt_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_Z
                && sprite.position == [172.0, 172.0]
                && sprite.size == [6.0, 8.0]
        }));
    }

    #[test]
    fn oracle_scene_projects_hall_of_fame_display_text_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::Attract;
        state.game_over = GameOverSnapshot {
            hall_of_fame_stall_remaining: Some(60),
            ..GameOverSnapshot::NONE
        };
        let mut tables = HighScoreTablesSnapshot::EMPTY;
        tables.todays_greatest[0] = HighScoreTableEntrySnapshot {
            rank: 1,
            score: 5_000,
            initials: [Some('A'), Some('C'), Some('E')],
        };
        tables.all_time[0] = HighScoreTableEntrySnapshot {
            rank: 1,
            score: 987_654,
            initials: [Some('Z'), Some('E'), Some('D')],
        };
        state.high_score_tables = tables;

        let scene = super::adapt_scene(&state, Some(0x4A11_0F00));
        let overlay_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.layer == RenderLayer::Overlay)
            .collect::<Vec<_>>();
        let underline_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::HALL_OF_FAME_UNDERLINE_WORD)
            .collect::<Vec<_>>();

        assert_eq!(scene.visual_signature, Some(0x4A11_0F00));
        assert_eq!(underline_sprites.len(), 62);
        assert!(
            underline_sprites
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Overlay && sprite.size == [2.0, 2.0])
        );
        assert!(underline_sprites.iter().any(|sprite| {
            sprite.position == [250.0, 123.0]
                && sprite.tint == Color::from_rgba(0x66, 0x66, 0x66, 0xFF)
        }));
        assert!(underline_sprites.iter().any(|sprite| {
            sprite.position == [60.0, 123.0]
                && sprite.tint == Color::from_rgba(0x66, 0x66, 0x66, 0xFF)
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_H
                && sprite.position == [112.0, 84.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position == [96.0, 56.0]
                && sprite.size == [120.0, 24.0]
                && sprite.tint == Color::WHITE
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_T
                && sprite.position == [68.0, 104.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_A
                && sprite.position == [192.0, 104.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.position == [48.0, 134.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_A
                && sprite.position == [58.0, 134.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_5
                && sprite.position == [94.0, 134.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_Z
                && sprite.position == [188.0, 134.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(overlay_sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_9
                && sprite.position == [216.0, 134.0]
                && sprite.size == [6.0, 8.0]
        }));
    }

    #[test]
    fn oracle_scene_projects_two_player_stock_sprites() {
        let mut state = crate::game::Game::new().state();
        state.phase = GamePhase::Playing;
        state.player_count = 2;
        state.player_stocks = [
            PlayerStockSnapshot::new(5, 3),
            PlayerStockSnapshot::new(2, 2),
        ];

        let scene = super::adapt_scene(&state, Some(0x8765_4321));
        let life_stock = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::PLAYER_LIFE_STOCK)
            .collect::<Vec<_>>();
        let smart_bomb_stock = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.sprite == SpriteId::SMART_BOMB_STOCK)
            .collect::<Vec<_>>();

        assert_eq!(scene.summary().visual_signature, Some(0x8765_4321));
        assert_eq!(life_stock.len(), 7);
        assert_eq!(smart_bomb_stock.len(), 5);
        assert_eq!(life_stock[0].position, [18.0, 13.0]);
        assert_eq!(life_stock[4].position, [66.0, 13.0]);
        assert_eq!(life_stock[5].position, [214.0, 13.0]);
        assert_eq!(life_stock[6].position, [226.0, 13.0]);
        assert_eq!(smart_bomb_stock[0].position, [70.0, 20.0]);
        assert_eq!(smart_bomb_stock[2].position, [70.0, 28.0]);
        assert_eq!(smart_bomb_stock[3].position, [266.0, 20.0]);
        assert_eq!(smart_bomb_stock[4].position, [266.0, 24.0]);
        assert_eq!(scene.summary().layers.hud, 22);
    }

    #[test]
    fn oracle_maps_all_accepted_phase_contracts() {
        assert_eq!(adapt_phase(AcceptedPhase::Attract), GamePhase::Attract);
        assert_eq!(adapt_phase(AcceptedPhase::Playing), GamePhase::Playing);
        assert_eq!(adapt_phase(AcceptedPhase::GameOver), GamePhase::GameOver);
        assert_eq!(
            adapt_phase(AcceptedPhase::HighScoreEntry),
            GamePhase::HighScoreEntry
        );
    }

    #[test]
    fn oracle_maps_all_accepted_direction_contracts() {
        assert_eq!(adapt_direction(AcceptedDirection::Left), Direction::Left);
        assert_eq!(adapt_direction(AcceptedDirection::Right), Direction::Right);
    }

    #[test]
    fn oracle_maps_accepted_high_score_initials_contract() {
        let initials = adapt_high_score_initials(AcceptedHighScoreInitials {
            initials: [Some('A'), None, Some('Z')],
            cursor: 9,
        });

        assert_eq!(
            initials,
            HighScoreInitialsState {
                initials: [Some('A'), None, Some('Z')],
                cursor: 3,
            }
        );
    }

    #[test]
    fn oracle_maps_accepted_high_score_entry_contract() {
        assert_eq!(
            adapt_high_score_entry(AcceptedHighScoreEntry {
                score: 31_250,
                rank: 3,
            }),
            HighScoreEntrySnapshot {
                score: 31_250,
                rank: 3,
            }
        );
    }

    #[test]
    fn oracle_maps_accepted_high_score_submission_contract() {
        assert_eq!(
            adapt_high_score_submission(AcceptedHighScoreSubmission {
                player: 2,
                score: 48_000,
            }),
            HighScoreSubmissionSnapshot {
                player: 2,
                score: 48_000,
            }
        );
    }

    #[test]
    fn oracle_maps_accepted_high_score_table_contract() {
        let entry = AcceptedHighScoreTableEntry {
            rank: 1,
            score: 50_000,
            initials: [Some('A'), Some('C'), Some('E')],
        };
        let tables = AcceptedHighScoreTables {
            all_time: [entry; crate::game::HIGH_SCORE_TABLE_ENTRIES],
            todays_greatest: [AcceptedHighScoreTableEntry {
                rank: 2,
                score: 40_000,
                initials: [Some('B'), Some('O'), Some('B')],
            }; crate::game::HIGH_SCORE_TABLE_ENTRIES],
        };

        assert_eq!(
            adapt_high_score_table_entry(entry),
            HighScoreTableEntrySnapshot {
                rank: 1,
                score: 50_000,
                initials: [Some('A'), Some('C'), Some('E')],
            }
        );
        assert_eq!(
            adapt_high_score_tables(tables).todays_greatest[0],
            HighScoreTableEntrySnapshot {
                rank: 2,
                score: 40_000,
                initials: [Some('B'), Some('O'), Some('B')],
            }
        );
    }

    #[test]
    fn oracle_snapshot_preserves_accepted_high_score_initials() {
        let snapshot = AcceptedSnapshot {
            frame: 7,
            phase: AcceptedPhase::HighScoreEntry,
            credits: 0,
            current_player: 1,
            player_count: 2,
            wave: 1,
            wave_profile: AcceptedWaveProfile {
                landers: 15,
                bombers: 0,
                pods: 0,
                mutants: 0,
                swarmers: 0,
                lander_x_velocity: 22,
                lander_y_velocity_msb: 0,
                lander_y_velocity_lsb: 112,
                mutant_random_y: 1,
                mutant_y_velocity_msb: 0,
                mutant_y_velocity_lsb: 98,
                mutant_x_velocity: 12,
                swarmer_x_velocity: 22,
                wave_time: 30,
                wave_size: 5,
                lander_shot_time: 74,
                bomber_x_velocity: 32,
                mutant_shot_time: 42,
                swarmer_shot_time: 25,
                swarmer_acceleration_mask: 31,
                baiter_delay: 212,
                baiter_shot_time: 15,
                baiter_seek_probability: 240,
            },
            player: AcceptedPlayer {
                x_subpixels: 0,
                y_subpixels: 0,
                x_velocity_subpixels: 0,
                y_velocity_subpixels: 0,
                direction: AcceptedDirection::Right,
                lives: 0,
                smart_bombs: 0,
            },
            player_stocks: [
                AcceptedPlayerStock {
                    lives: 4,
                    smart_bombs: 2,
                },
                AcceptedPlayerStock {
                    lives: 1,
                    smart_bombs: 3,
                },
            ],
            scores: AcceptedScores {
                player_one: 30_000,
                player_two: 0,
                high_score: 21_270,
                next_bonus: 10_000,
            },
            high_score_initials: AcceptedHighScoreInitials {
                initials: [Some('D'), Some('R'), None],
                cursor: 2,
            },
            high_score_entry: Some(AcceptedHighScoreEntry {
                score: 30_000,
                rank: 1,
            }),
            high_score_submission: Some(AcceptedHighScoreSubmission {
                player: 1,
                score: 30_000,
            }),
            high_score_tables: AcceptedHighScoreTables {
                all_time: [AcceptedHighScoreTableEntry {
                    rank: 1,
                    score: 30_000,
                    initials: [Some('D'), Some('R'), Some('J')],
                }; crate::game::HIGH_SCORE_TABLE_ENTRIES],
                todays_greatest: [AcceptedHighScoreTableEntry {
                    rank: 1,
                    score: 29_000,
                    initials: [Some('S'), Some('A'), Some('M')],
                }; crate::game::HIGH_SCORE_TABLE_ENTRIES],
            },
            game_over: AcceptedGameOverState {
                player_death_sleep_remaining: Some(40),
                player_switch_sleep_remaining: None,
                player_switch_from: None,
                player_switch_to: None,
                no_entry_delay_remaining: None,
                hall_of_fame_stall_remaining: None,
            },
            object_evidence: AcceptedObjectEvidence {
                active_count: 4,
                inactive_count: 7,
                projectile_count: 2,
                visible_count: 3,
                evidence_crc32: 0x1234_5678,
                detail_count: 1,
                details: {
                    let mut details =
                        [AcceptedObjectEvidenceDetail::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT];
                    details[0] = AcceptedObjectEvidenceDetail {
                        list: AcceptedObjectEvidenceList::Active,
                        address: 0xA23C,
                        slot: 0,
                        screen_x: 12,
                        screen_y: 34,
                        world_x: 0x2000,
                        world_y: 0x8000,
                        velocity_x: 0xFFF0,
                        velocity_y: 0x0010,
                        picture_address: 0x9000,
                        picture_label: Some("LNDP1"),
                        picture_size: Some((5, 8)),
                        primary_image_address: Some(0xCCE0),
                        alternate_image_address: Some(0xCD08),
                        mapped_sprite: Some(SpriteId::ENEMY_LANDER.0),
                        object_type: 0x11,
                        scanner_color: 0x4433,
                    };
                    details
                },
            },
            expanded_objects: AcceptedExpandedObjectEvidence {
                active_count: 1,
                last_slot_address: Some(0x9C00),
                detail_count: 1,
                details: {
                    let mut details =
                        [AcceptedExpandedObjectDetail::EMPTY; EXPANDED_OBJECT_DETAIL_LIMIT];
                    details[0] = AcceptedExpandedObjectDetail {
                        kind: AcceptedExpandedObjectKind::Explosion,
                        slot_address: 0x9C00,
                        size: 0x01AA,
                        descriptor_address: 0xF951,
                        picture_label: Some("BXPIC"),
                        picture_size: Some((4, 8)),
                        mapped_sprite: Some(SpriteId::BOMB_EXPLOSION.0),
                        erase_address: 0x9C40,
                        center_x: 0x14,
                        center_y: 0x05,
                        top_left_x: 0x0C,
                        top_left_y: 0x05,
                        object_address: None,
                        score_popup_lifetime_ticks: None,
                        score_popup_value: None,
                        explosion_frame: Some(1),
                        explosion_lifetime_frames: Some(SOURCE_EXPLOSION_LIFETIME_FRAMES),
                    };
                    details
                },
            },
            player_explosion: None,
            terrain_blow: Some(TerrainBlowSnapshot {
                stage: TerrainBlowStage::ExplosionPassSleeping,
                status_terrain_blown: true,
                source_elapsed_frames: 0,
                source_iteration: 0,
                source_iteration_limit: 16,
                source_sleep_remaining: Some(1),
                source_pseudo_color: 0,
                source_overload_counter: 8,
                terrain_erase_entries: 0x98,
                scanner_terrain_erase_entries: 0x40,
                terrain_words_remaining: 0,
                scanner_terrain_words_remaining: 0,
                explosions_per_pass: 2,
            }),
        };

        let state = super::adapt_snapshot(snapshot);

        assert_eq!(state.phase, GamePhase::HighScoreEntry);
        assert_eq!(state.player_count, 2);
        assert_eq!(
            state.player_stocks,
            [
                PlayerStockSnapshot {
                    lives: 4,
                    smart_bombs: 2,
                },
                PlayerStockSnapshot {
                    lives: 1,
                    smart_bombs: 3,
                },
            ]
        );
        assert_eq!(state.wave_profile.landers, 15);
        assert_eq!(state.wave_profile.baiter_delay, 192);
        assert_eq!(
            state.high_score_initials,
            HighScoreInitialsState {
                initials: [Some('D'), Some('R'), None],
                cursor: 2,
            }
        );
        assert_eq!(
            state.high_score_entry,
            Some(HighScoreEntrySnapshot {
                score: 30_000,
                rank: 1,
            })
        );
        assert_eq!(
            state.high_score_submission,
            Some(HighScoreSubmissionSnapshot {
                player: 1,
                score: 30_000,
            })
        );
        assert_eq!(state.high_score_tables.all_time[0].score, 30_000);
        assert_eq!(state.high_score_tables.todays_greatest[0].score, 29_000);
        assert_eq!(
            state.game_over,
            GameOverSnapshot {
                player_death_sleep_remaining: Some(40),
                player_switch_sleep_remaining: None,
                player_switch_from: None,
                player_switch_to: None,
                no_entry_delay_remaining: None,
                hall_of_fame_stall_remaining: None,
            }
        );
        assert_eq!(state.world.object_evidence.active_count, 4);
        assert_eq!(state.world.object_evidence.inactive_count, 7);
        assert_eq!(state.world.object_evidence.projectile_count, 2);
        assert_eq!(state.world.object_evidence.visible_count, 3);
        assert_eq!(
            state.world.object_evidence.evidence_crc32,
            Some(0x1234_5678)
        );
        assert_eq!(state.world.object_evidence.detail_count, 1);
        assert_eq!(
            state.world.object_evidence.details[0],
            ObjectEvidenceDetailSnapshot {
                list: ObjectEvidenceList::Active,
                object_category: None,
                address: Some(0xA23C),
                slot: Some(0),
                screen_position: Some(crate::systems::ScreenPosition::new(12, 34)),
                world_position: Some((0x2000, 0x8000)),
                velocity: Some((0xFFF0, 0x0010)),
                picture_address: Some(0x9000),
                picture_label: Some("LNDP1"),
                picture_size: Some((5, 8)),
                primary_image_address: Some(0xCCE0),
                alternate_image_address: Some(0xCD08),
                mapped_sprite: Some(SpriteId::ENEMY_LANDER),
                object_type: Some(0x11),
                scanner_color: Some(0x4433),
            }
        );
        assert_eq!(state.world.expanded_objects.active_count, 1);
        assert_eq!(state.world.expanded_objects.last_slot_address, Some(0x9C00));
        assert_eq!(state.world.expanded_objects.detail_count, 1);
        assert_eq!(
            state.world.expanded_objects.details[0],
            ExpandedObjectDetailSnapshot {
                kind: ExpandedObjectKind::Explosion,
                slot_address: Some(0x9C00),
                size: 0x01AA,
                descriptor_address: Some(0xF951),
                picture_label: Some("BXPIC"),
                picture_size: Some((4, 8)),
                mapped_sprite: Some(SpriteId::BOMB_EXPLOSION),
                erase_address: Some(0x9C40),
                center: Some(crate::systems::ScreenPosition::new(0x14, 0x05)),
                top_left: Some(crate::systems::ScreenPosition::new(0x0C, 0x05)),
                object_address: None,
                score_popup_lifetime_ticks: None,
                score_popup_value: None,
                explosion_frame: Some(1),
                explosion_lifetime_frames: Some(SOURCE_EXPLOSION_LIFETIME_FRAMES),
            }
        );
        assert_eq!(state.world.terrain_blow, snapshot.terrain_blow);
    }

    #[test]
    fn oracle_preserves_accepted_game_over_return_timing() {
        let state = AcceptedGameOverState {
            player_death_sleep_remaining: None,
            player_switch_sleep_remaining: None,
            player_switch_from: None,
            player_switch_to: None,
            no_entry_delay_remaining: Some(0xFF),
            hall_of_fame_stall_remaining: None,
        };

        assert_eq!(
            adapt_game_over(state),
            GameOverSnapshot {
                player_death_sleep_remaining: None,
                player_switch_sleep_remaining: None,
                player_switch_from: None,
                player_switch_to: None,
                no_entry_delay_remaining: Some(0xFF),
                hall_of_fame_stall_remaining: None,
            }
        );
    }

    #[test]
    fn oracle_preserves_accepted_player_switch_timing() {
        let state = AcceptedGameOverState {
            player_death_sleep_remaining: None,
            player_switch_sleep_remaining: Some(0x60),
            player_switch_from: Some(1),
            player_switch_to: Some(2),
            no_entry_delay_remaining: None,
            hall_of_fame_stall_remaining: None,
        };

        assert_eq!(
            adapt_game_over(state),
            GameOverSnapshot {
                player_death_sleep_remaining: None,
                player_switch_sleep_remaining: Some(0x60),
                player_switch_from: Some(1),
                player_switch_to: Some(2),
                no_entry_delay_remaining: None,
                hall_of_fame_stall_remaining: None,
            }
        );
    }

    #[test]
    fn oracle_maps_accepted_wave_profile_contract() {
        let accepted = AcceptedWaveProfile {
            landers: 20,
            bombers: 3,
            pods: 1,
            mutants: 0,
            swarmers: 0,
            lander_x_velocity: 30,
            lander_y_velocity_msb: 0,
            lander_y_velocity_lsb: 176,
            mutant_random_y: 1,
            mutant_y_velocity_msb: 0,
            mutant_y_velocity_lsb: 224,
            mutant_x_velocity: 28,
            swarmer_x_velocity: 30,
            wave_time: 25,
            wave_size: 5,
            lander_shot_time: 58,
            bomber_x_velocity: 40,
            mutant_shot_time: 34,
            swarmer_shot_time: 25,
            swarmer_acceleration_mask: 31,
            baiter_delay: 196,
            baiter_shot_time: 13,
            baiter_seek_probability: 220,
        };

        assert_eq!(
            adapt_wave_profile(accepted, 2),
            WaveProfileSnapshot::for_wave(2)
        );
    }

    #[test]
    fn oracle_maps_accepted_object_evidence_contract() {
        let accepted = AcceptedObjectEvidence {
            active_count: 5,
            inactive_count: 6,
            projectile_count: 1,
            visible_count: 4,
            evidence_crc32: 0xAABB_CCDD,
            detail_count: 5,
            details: {
                let mut details =
                    [AcceptedObjectEvidenceDetail::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT];
                details[0] = AcceptedObjectEvidenceDetail {
                    list: AcceptedObjectEvidenceList::Projectile,
                    address: 0xA267,
                    slot: 1,
                    screen_x: 20,
                    screen_y: 30,
                    world_x: 0x1111,
                    world_y: 0x2222,
                    velocity_x: 0x3333,
                    velocity_y: 0x4444,
                    picture_address: 0x9555,
                    picture_label: Some("PRBP1"),
                    picture_size: Some((4, 8)),
                    primary_image_address: Some(0xFA8B),
                    alternate_image_address: Some(0xFAAB),
                    mapped_sprite: Some(SpriteId::ENEMY_POD.0),
                    object_type: 0x02,
                    scanner_color: 0x2424,
                };
                details[1] = AcceptedObjectEvidenceDetail {
                    list: AcceptedObjectEvidenceList::Active,
                    address: 0xA2A7,
                    slot: 2,
                    screen_x: 40,
                    screen_y: 50,
                    world_x: 0x5555,
                    world_y: 0x6666,
                    velocity_x: 0x7777,
                    velocity_y: 0x8888,
                    picture_address: 0xF95B,
                    picture_label: Some("BMBP1"),
                    picture_size: Some((2, 3)),
                    primary_image_address: Some(0xCCB0),
                    alternate_image_address: Some(0xCCB6),
                    mapped_sprite: Some(SpriteId::ENEMY_BOMB.0),
                    object_type: 0x04,
                    scanner_color: 0x1111,
                };
                details[2] = AcceptedObjectEvidenceDetail {
                    list: AcceptedObjectEvidenceList::Active,
                    address: 0xA2E7,
                    slot: 3,
                    screen_x: 60,
                    screen_y: 70,
                    world_x: 0x9999,
                    world_y: 0xAAAA,
                    velocity_x: 0xBBBB,
                    velocity_y: 0xCCCC,
                    picture_address: 0xF8D8,
                    picture_label: Some("ASXP1"),
                    picture_size: Some((4, 8)),
                    primary_image_address: Some(0xFA4B),
                    alternate_image_address: Some(0xFA4B),
                    mapped_sprite: Some(SpriteId::ASTRONAUT_EXPLOSION.0),
                    object_type: 0x08,
                    scanner_color: 0x6666,
                };
                details[3] = AcceptedObjectEvidenceDetail {
                    list: AcceptedObjectEvidenceList::Inactive,
                    address: 0xA327,
                    slot: 4,
                    screen_x: 80,
                    screen_y: 90,
                    world_x: 0xDDDD,
                    world_y: 0xEEEE,
                    velocity_x: 0x1110,
                    velocity_y: 0x2220,
                    picture_address: 0xF8EC,
                    picture_label: Some("NULOB"),
                    picture_size: Some((1, 1)),
                    primary_image_address: Some(0xF8F6),
                    alternate_image_address: Some(0xF8F6),
                    mapped_sprite: Some(SpriteId::NULL_OBJECT.0),
                    object_type: 0x00,
                    scanner_color: 0,
                };
                details[4] = AcceptedObjectEvidenceDetail {
                    list: AcceptedObjectEvidenceList::Active,
                    address: 0xA367,
                    slot: 5,
                    screen_x: 100,
                    screen_y: 110,
                    world_x: 0x3330,
                    world_y: 0x4440,
                    velocity_x: 0x5550,
                    velocity_y: 0x6660,
                    picture_address: 0xF9F1,
                    picture_label: Some("TEREX"),
                    picture_size: Some((8, 6)),
                    primary_image_address: Some(0xCFCD),
                    alternate_image_address: Some(0xCFCD),
                    mapped_sprite: Some(SpriteId::TERRAIN_EXPLOSION.0),
                    object_type: 0x10,
                    scanner_color: 0,
                };
                details
            },
        };

        let evidence = adapt_object_evidence(accepted);

        assert_eq!(evidence.active_count, 5);
        assert_eq!(evidence.inactive_count, 6);
        assert_eq!(evidence.projectile_count, 1);
        assert_eq!(evidence.visible_count, 4);
        assert_eq!(evidence.evidence_crc32, Some(0xAABB_CCDD));
        assert_eq!(evidence.detail_count, 5);
        assert_eq!(
            evidence.details[0],
            ObjectEvidenceDetailSnapshot {
                list: ObjectEvidenceList::Projectile,
                object_category: None,
                address: Some(0xA267),
                slot: Some(1),
                screen_position: Some(crate::systems::ScreenPosition::new(20, 30)),
                world_position: Some((0x1111, 0x2222)),
                velocity: Some((0x3333, 0x4444)),
                picture_address: Some(0x9555),
                picture_label: Some("PRBP1"),
                picture_size: Some((4, 8)),
                primary_image_address: Some(0xFA8B),
                alternate_image_address: Some(0xFAAB),
                mapped_sprite: Some(SpriteId::ENEMY_POD),
                object_type: Some(0x02),
                scanner_color: Some(0x2424),
            }
        );
        assert_eq!(
            evidence.details[1],
            ObjectEvidenceDetailSnapshot {
                list: ObjectEvidenceList::Active,
                object_category: None,
                address: Some(0xA2A7),
                slot: Some(2),
                screen_position: Some(crate::systems::ScreenPosition::new(40, 50)),
                world_position: Some((0x5555, 0x6666)),
                velocity: Some((0x7777, 0x8888)),
                picture_address: Some(0xF95B),
                picture_label: Some("BMBP1"),
                picture_size: Some((2, 3)),
                primary_image_address: Some(0xCCB0),
                alternate_image_address: Some(0xCCB6),
                mapped_sprite: Some(SpriteId::ENEMY_BOMB),
                object_type: Some(0x04),
                scanner_color: Some(0x1111),
            }
        );
        assert_eq!(
            evidence.details[2],
            ObjectEvidenceDetailSnapshot {
                list: ObjectEvidenceList::Active,
                object_category: None,
                address: Some(0xA2E7),
                slot: Some(3),
                screen_position: Some(crate::systems::ScreenPosition::new(60, 70)),
                world_position: Some((0x9999, 0xAAAA)),
                velocity: Some((0xBBBB, 0xCCCC)),
                picture_address: Some(0xF8D8),
                picture_label: Some("ASXP1"),
                picture_size: Some((4, 8)),
                primary_image_address: Some(0xFA4B),
                alternate_image_address: Some(0xFA4B),
                mapped_sprite: Some(SpriteId::ASTRONAUT_EXPLOSION),
                object_type: Some(0x08),
                scanner_color: Some(0x6666),
            }
        );
        assert_eq!(
            evidence.details[3],
            ObjectEvidenceDetailSnapshot {
                list: ObjectEvidenceList::Inactive,
                object_category: None,
                address: Some(0xA327),
                slot: Some(4),
                screen_position: Some(crate::systems::ScreenPosition::new(80, 90)),
                world_position: Some((0xDDDD, 0xEEEE)),
                velocity: Some((0x1110, 0x2220)),
                picture_address: Some(0xF8EC),
                picture_label: Some("NULOB"),
                picture_size: Some((1, 1)),
                primary_image_address: Some(0xF8F6),
                alternate_image_address: Some(0xF8F6),
                mapped_sprite: Some(SpriteId::NULL_OBJECT),
                object_type: Some(0x00),
                scanner_color: Some(0),
            }
        );
        assert_eq!(
            evidence.details[4],
            ObjectEvidenceDetailSnapshot {
                list: ObjectEvidenceList::Active,
                object_category: None,
                address: Some(0xA367),
                slot: Some(5),
                screen_position: Some(crate::systems::ScreenPosition::new(100, 110)),
                world_position: Some((0x3330, 0x4440)),
                velocity: Some((0x5550, 0x6660)),
                picture_address: Some(0xF9F1),
                picture_label: Some("TEREX"),
                picture_size: Some((8, 6)),
                primary_image_address: Some(0xCFCD),
                alternate_image_address: Some(0xCFCD),
                mapped_sprite: Some(SpriteId::TERRAIN_EXPLOSION),
                object_type: Some(0x10),
                scanner_color: Some(0),
            }
        );
    }

    #[test]
    fn oracle_maps_accepted_expanded_object_evidence_contract() {
        let accepted = AcceptedExpandedObjectEvidence {
            active_count: 3,
            last_slot_address: Some(0x9C40),
            detail_count: 3,
            details: {
                let mut details =
                    [AcceptedExpandedObjectDetail::EMPTY; EXPANDED_OBJECT_DETAIL_LIMIT];
                details[0] = AcceptedExpandedObjectDetail {
                    kind: AcceptedExpandedObjectKind::Appearance,
                    slot_address: 0x9C00,
                    size: 0xAE00,
                    descriptor_address: 0xF9C1,
                    picture_label: Some("PLAPIC"),
                    picture_size: Some((8, 6)),
                    mapped_sprite: Some(SpriteId::PLAYER_SHIP.0),
                    erase_address: 0x9C30,
                    center_x: 0x08,
                    center_y: 0x47,
                    top_left_x: 0x08,
                    top_left_y: 0x44,
                    object_address: Some(0xA23C),
                    score_popup_lifetime_ticks: None,
                    score_popup_value: None,
                    explosion_frame: None,
                    explosion_lifetime_frames: None,
                };
                details[1] = AcceptedExpandedObjectDetail {
                    kind: AcceptedExpandedObjectKind::Explosion,
                    slot_address: 0x9C40,
                    size: 0x01AA,
                    descriptor_address: 0xF951,
                    picture_label: Some("BXPIC"),
                    picture_size: Some((4, 8)),
                    mapped_sprite: Some(SpriteId::BOMB_EXPLOSION.0),
                    erase_address: 0x9C80,
                    center_x: 0x14,
                    center_y: 0x05,
                    top_left_x: 0x0C,
                    top_left_y: 0x05,
                    object_address: None,
                    score_popup_lifetime_ticks: None,
                    score_popup_value: None,
                    explosion_frame: Some(1),
                    explosion_lifetime_frames: Some(SOURCE_EXPLOSION_LIFETIME_FRAMES),
                };
                details[2] = AcceptedExpandedObjectDetail {
                    kind: AcceptedExpandedObjectKind::ScorePopup,
                    slot_address: 0x9C80,
                    size: 0x8000,
                    descriptor_address: 0xF9E7,
                    picture_label: Some("C5P1"),
                    picture_size: Some((6, 6)),
                    mapped_sprite: Some(SpriteId::SCORE_POPUP_500.0),
                    erase_address: 0x9CC0,
                    center_x: 0x24,
                    center_y: 0x54,
                    top_left_x: 0x21,
                    top_left_y: 0x51,
                    object_address: Some(0xA253),
                    score_popup_lifetime_ticks: Some(SOURCE_SCORE_POPUP_LIFETIME_TICKS),
                    score_popup_value: Some(500),
                    explosion_frame: None,
                    explosion_lifetime_frames: None,
                };
                details
            },
        };

        let evidence = adapt_expanded_objects(accepted);

        assert_eq!(evidence.active_count, 3);
        assert_eq!(evidence.last_slot_address, Some(0x9C40));
        assert_eq!(evidence.detail_count, 3);
        assert_eq!(
            evidence.details[0],
            ExpandedObjectDetailSnapshot {
                kind: ExpandedObjectKind::Appearance,
                slot_address: Some(0x9C00),
                size: 0xAE00,
                descriptor_address: Some(0xF9C1),
                picture_label: Some("PLAPIC"),
                picture_size: Some((8, 6)),
                mapped_sprite: Some(SpriteId::PLAYER_SHIP),
                erase_address: Some(0x9C30),
                center: Some(crate::systems::ScreenPosition::new(0x08, 0x47)),
                top_left: Some(crate::systems::ScreenPosition::new(0x08, 0x44)),
                object_address: Some(0xA23C),
                score_popup_lifetime_ticks: None,
                score_popup_value: None,
                explosion_frame: None,
                explosion_lifetime_frames: None,
            }
        );
        assert_eq!(
            evidence.details[1],
            ExpandedObjectDetailSnapshot {
                kind: ExpandedObjectKind::Explosion,
                slot_address: Some(0x9C40),
                size: 0x01AA,
                descriptor_address: Some(0xF951),
                picture_label: Some("BXPIC"),
                picture_size: Some((4, 8)),
                mapped_sprite: Some(SpriteId::BOMB_EXPLOSION),
                erase_address: Some(0x9C80),
                center: Some(crate::systems::ScreenPosition::new(0x14, 0x05)),
                top_left: Some(crate::systems::ScreenPosition::new(0x0C, 0x05)),
                object_address: None,
                score_popup_lifetime_ticks: None,
                score_popup_value: None,
                explosion_frame: Some(1),
                explosion_lifetime_frames: Some(SOURCE_EXPLOSION_LIFETIME_FRAMES),
            }
        );
        assert_eq!(
            evidence.details[2],
            ExpandedObjectDetailSnapshot {
                kind: ExpandedObjectKind::ScorePopup,
                slot_address: Some(0x9C80),
                size: 0x8000,
                descriptor_address: Some(0xF9E7),
                picture_label: Some("C5P1"),
                picture_size: Some((6, 6)),
                mapped_sprite: Some(SpriteId::SCORE_POPUP_500),
                erase_address: Some(0x9CC0),
                center: Some(crate::systems::ScreenPosition::new(0x24, 0x54)),
                top_left: Some(crate::systems::ScreenPosition::new(0x21, 0x51)),
                object_address: Some(0xA253),
                score_popup_lifetime_ticks: Some(SOURCE_SCORE_POPUP_LIFETIME_TICKS),
                score_popup_value: Some(500),
                explosion_frame: None,
                explosion_lifetime_frames: None,
            }
        );
    }

    #[test]
    fn oracle_maps_all_accepted_event_contracts() {
        let pairs = [
            (AcceptedEvent::CreditAdded, GameEvent::CreditAdded),
            (AcceptedEvent::GameStarted, GameEvent::GameStarted),
            (
                AcceptedEvent::DiagnosticsSelected,
                GameEvent::DiagnosticsSelected,
            ),
            (AcceptedEvent::AuditsSelected, GameEvent::AuditsSelected),
            (AcceptedEvent::HighScoreReset, GameEvent::HighScoreReset),
            (AcceptedEvent::ReversePressed, GameEvent::ReversePressed),
            (AcceptedEvent::FirePressed, GameEvent::FirePressed),
            (AcceptedEvent::SmartBombPressed, GameEvent::SmartBombPressed),
            (
                AcceptedEvent::HyperspacePressed,
                GameEvent::HyperspacePressed,
            ),
            (AcceptedEvent::BonusAwarded, GameEvent::BonusAwarded),
            (
                AcceptedEvent::HighScoreEntryStarted,
                GameEvent::HighScoreEntryStarted,
            ),
            (
                AcceptedEvent::HighScoreInitialAccepted,
                GameEvent::HighScoreInitialAccepted,
            ),
            (
                AcceptedEvent::HighScoreSubmitted,
                GameEvent::HighScoreSubmitted,
            ),
        ];

        for (accepted, clean) in pairs {
            assert_eq!(adapt_event(accepted), clean);
        }
    }

    #[test]
    fn oracle_maps_accepted_sound_commands_to_clean_events() {
        assert_eq!(adapt_sound_command(0xC0), SoundEvent::Startup);
        assert_eq!(adapt_sound_command(0xE6), SoundEvent::CreditAdded);
        assert_eq!(adapt_sound_command(0xF5), SoundEvent::GameStarted);
        assert_eq!(adapt_sound_command(0xE9), SoundEvent::ThrustStarted);
        assert_eq!(adapt_sound_command(0xF0), SoundEvent::ThrustStopped);
        assert_eq!(
            adapt_sound_command(0x3E),
            SoundEvent::UnmappedSoundCommand { command: 0x3E }
        );
    }

    #[test]
    fn oracle_implements_clean_simulation_trait() {
        let mut oracle = GameplayOracle::new();

        let frame = advance_one_frame(&mut oracle, GameInput::NONE);

        assert_eq!(frame.state.frame, 1);
        assert_eq!(GameSimulation::state(&oracle).frame, 1);
    }

    #[test]
    fn player_action_triggers_none_is_empty() {
        assert!(!PlayerActionTriggers::NONE.any());
    }
}
