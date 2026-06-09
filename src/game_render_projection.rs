fn attract_title_reference_sample_index(page_frame: u16) -> usize {
    usize::from(page_frame / ATTRACT_TITLE_REFERENCE_SAMPLE_STEP_FRAMES).saturating_sub(1)
}

fn williams_color_byte_tint(value: u8) -> Color {
    if value == 0 {
        return Color::from_rgba(0, 0, 0, 0);
    }

    Color::from_rgba(
        WILLIAMS_RED_GREEN_LEVELS[usize::from(value & 0x07)],
        WILLIAMS_RED_GREEN_LEVELS[usize::from((value >> 3) & 0x07)],
        WILLIAMS_BLUE_LEVELS[usize::from((value >> 6) & 0x03)],
        0xFF,
    )
}

pub(crate) fn arcade_wave_landscape_tint(wave: u16) -> Color {
    let wave = wave.max(1);
    let index = usize::from((wave - 1) % WAVE_LANDSCAPE_COLOR_BYTES.len() as u16);
    williams_color_byte_tint(WAVE_LANDSCAPE_COLOR_BYTES[index])
}

pub(crate) fn terrain_blow_flash_tint(elapsed: u16) -> Color {
    let color = TERRAIN_BLOW_FLASH_WINDOWS
        .iter()
        .find_map(|(start, end, color)| (*start <= elapsed && elapsed <= *end).then_some(*color))
        .unwrap_or(0);
    williams_color_byte_tint(color)
}

fn video_palette_index_tint(index: u8) -> Color {
    williams_color_byte_tint(NORMAL_PALETTE_BYTES[usize::from(index & 0x0F)])
}

fn video_word_tint(word: u16) -> Color {
    video_palette_index_tint((word & 0x000F) as u8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerrainFlavorRecord {
    offset: u8,
    word: u16,
}

impl TerrainFlavorRecord {
    const EMPTY: Self = Self { offset: 0, word: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerrainDrawRecord {
    screen_cell: crate::ScreenAddress,
    word: u16,
}

impl TerrainDrawRecord {
    const EMPTY: Self = Self {
        screen_cell: crate::ScreenAddress::new(0),
        word: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScannerMiniTerrainRecord {
    screen_cell: crate::ScreenAddress,
    word: u16,
}

impl ScannerMiniTerrainRecord {
    const EMPTY: Self = Self {
        screen_cell: crate::ScreenAddress::new(0),
        word: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerrainBitState {
    data_index: usize,
    data_pointer: u16,
    data_byte: u8,
    bit_counter: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerrainGenerationState {
    left: TerrainBitState,
    right: TerrainBitState,
    left_offset: u8,
    right_offset: u8,
    background_left: u16,
    terrain_left: u16,
    flavor_0_pointer: usize,
    flavor_1_pointer: usize,
}

pub(crate) fn push_background_terrain_sprites(
    scene: &mut RenderScene,
    background_left: u16,
    tint: Color,
) {
    for record in background_terrain_records(background_left) {
        scene.push_sprite(SceneSprite {
            sprite: terrain_word_sprite(record.word),
            layer: RenderLayer::Terrain,
            position: screen_position_from_cell(record.screen_cell),
            size: TERRAIN_WORD_SIZE,
            tint,
        });
    }
}

fn terrain_word_sprite(word: u16) -> SpriteId {
    if word == TERRAIN_WORD_0770 {
        SpriteId::TERRAIN_TILE_ALT
    } else {
        SpriteId::TERRAIN_TILE
    }
}

fn default_background_terrain_records() -> &'static [TerrainDrawRecord; TERRAIN_SCREEN_WORDS] {
    static RECORDS: OnceLock<[TerrainDrawRecord; TERRAIN_SCREEN_WORDS]> = OnceLock::new();
    RECORDS.get_or_init(|| generate_background_terrain_records(0))
}

fn background_terrain_records(background_left: u16) -> [TerrainDrawRecord; TERRAIN_SCREEN_WORDS] {
    let terrain_left = background_terrain_left(background_left);
    if terrain_left == 0 {
        return *default_background_terrain_records();
    }
    generate_background_terrain_records(terrain_left)
}

fn scanner_mini_terrain_records_for_scan_left(
    scan_left: u16,
) -> [ScannerMiniTerrainRecord; SCANNER_TERRAIN_RECORDS] {
    generate_scanner_mini_terrain_records(scan_left)
}

fn generate_background_terrain_records(terrain_left: u16) -> [TerrainDrawRecord; TERRAIN_SCREEN_WORDS] {
    let data = terrain_pattern_bytes();
    let terrain_left = background_terrain_left(terrain_left);
    let (flavor_0, flavor_1, state) = initialize_terrain_flavor_tables(data, terrain_left);
    let selected_flavor = if state.terrain_left.to_be_bytes()[1] & 0x20 == 0 {
        &flavor_1
    } else {
        &flavor_0
    };
    let selected_pointer = if state.terrain_left.to_be_bytes()[1] & 0x20 == 0 {
        state.flavor_1_pointer
    } else {
        state.flavor_0_pointer
    };

    let mut records = [TerrainDrawRecord::EMPTY; TERRAIN_SCREEN_WORDS];
    for (entry_index, record) in records.iter_mut().enumerate() {
        let terrain_record =
            selected_flavor[(selected_pointer + entry_index) % selected_flavor.len()];
        *record = TerrainDrawRecord {
            screen_cell: crate::ScreenAddress::from_bytes(
                0x98u8.wrapping_sub(
                    u8::try_from(entry_index)
                        .expect("background terrain entry index fits in u8"),
                ),
                terrain_record.offset,
            ),
            word: terrain_record.word,
        };
    }
    records
}

const fn background_terrain_left(background_left: u16) -> u16 {
    background_left & 0xFFE0
}

fn generate_scanner_mini_terrain_records(
    scan_left: u16,
) -> [ScannerMiniTerrainRecord; SCANNER_TERRAIN_RECORDS] {
    let bytes = main_terrain_record_bytes();
    let first_record = usize::from(scan_left.to_be_bytes()[0] >> 2);
    assert!(
        first_record + SCANNER_TERRAIN_RECORDS <= SCANNER_MINI_TERRAIN_RECORDS,
        "main terrain slice must contain 64 scanner terrain records"
    );

    let mut records = [ScannerMiniTerrainRecord::EMPTY; SCANNER_TERRAIN_RECORDS];
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

fn initialize_terrain_flavor_tables(
    data: &[u8; TERRAIN_TDATA_BYTES],
    terrain_left: u16,
) -> (
    [TerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
    [TerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
    TerrainGenerationState,
) {
    let (right, right_offset) = initialize_right_terrain_state(data);
    let mut generation_left = terrain_left.wrapping_add(0x2610);
    let mut left = TerrainBitState {
        data_index: data.len() - 1,
        data_pointer: TERRAIN_PATTERN_STREAM_BASE.wrapping_sub(1),
        data_byte: 0,
        bit_counter: 0,
    };
    let mut left_offset = 0xE0;
    advance_terrain_right_state(&mut left, data);

    let mut scan_x = 0x0010u16;
    for _ in 0..=0x0800 {
        if scan_x == generation_left {
            break;
        }
        left_offset = terrain_altitude_step(left_offset, left.data_byte);
        advance_terrain_right_state(&mut left, data);
        scan_x = scan_x.wrapping_add(0x20);
    }
    assert_eq!(
        scan_x, generation_left,
        "background terrain stream must align to 0x{generation_left:04X}"
    );

    let saved_right = left;
    let saved_right_offset = left_offset;
    let mut flavor_0 = [TerrainFlavorRecord::EMPTY; TERRAIN_FLAVOR_RECORDS];
    let mut flavor_1 = [TerrainFlavorRecord::EMPTY; TERRAIN_FLAVOR_RECORDS];
    let mut state = TerrainGenerationState {
        left,
        right,
        left_offset,
        right_offset,
        background_left: generation_left,
        terrain_left,
        flavor_0_pointer: 0,
        flavor_1_pointer: 0,
    };

    loop {
        generation_left = generation_left.wrapping_sub(0x20);
        state.background_left = generation_left;
        if generation_left.wrapping_sub(state.terrain_left) & 0x8000 != 0 {
            break;
        }
        add_left_terrain_pixel(&mut state, data, &mut flavor_0, &mut flavor_1);
    }

    state.right = saved_right;
    state.right_offset = saved_right_offset;
    (flavor_0, flavor_1, state)
}

fn add_left_terrain_pixel(
    state: &mut TerrainGenerationState,
    data: &[u8; TERRAIN_TDATA_BYTES],
    flavor_0: &mut [TerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
    flavor_1: &mut [TerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
) {
    advance_terrain_left_state(&mut state.right, data);
    state.right_offset = if state.right.data_byte & 0x80 == 0 {
        state.right_offset.wrapping_sub(1)
    } else {
        state.right_offset.wrapping_add(1)
    };

    let flavor_0_selected = state.background_left.to_be_bytes()[1] & 0x20 != 0;
    let record_index = if flavor_0_selected {
        state.flavor_0_pointer
    } else {
        state.flavor_1_pointer
    };

    advance_terrain_left_state(&mut state.left, data);
    let (offset, word) = if state.left.data_byte & 0x80 == 0 {
        state.left_offset = state.left_offset.wrapping_sub(1);
        (state.left_offset, TERRAIN_WORD_7007)
    } else {
        let offset = state.left_offset;
        state.left_offset = state.left_offset.wrapping_add(1);
        (offset, TERRAIN_WORD_0770)
    };

    let record = TerrainFlavorRecord { offset, word };
    if flavor_0_selected {
        flavor_0[record_index] = record;
        state.flavor_0_pointer = (record_index + 1) % TERRAIN_FLAVOR_RECORDS;
    } else {
        flavor_1[record_index] = record;
        state.flavor_1_pointer = (record_index + 1) % TERRAIN_FLAVOR_RECORDS;
    }
}

fn initialize_right_terrain_state(data: &[u8; TERRAIN_TDATA_BYTES]) -> (TerrainBitState, u8) {
    let mut state = TerrainBitState {
        data_index: 0,
        data_pointer: TERRAIN_PATTERN_STREAM_BASE,
        data_byte: data[0],
        bit_counter: 7,
    };
    let mut offset = 0xE0;
    for _ in 0..0x0400 {
        offset = terrain_altitude_step(offset, state.data_byte);
        advance_terrain_right_state(&mut state, data);
        offset = terrain_altitude_step(offset, state.data_byte);
        advance_terrain_right_state(&mut state, data);
    }
    (state, offset)
}

fn terrain_altitude_step(offset: u8, data_byte: u8) -> u8 {
    if data_byte & 0x80 != 0 {
        offset.wrapping_sub(1)
    } else {
        offset.wrapping_add(1)
    }
}

fn advance_terrain_right_state(
    state: &mut TerrainBitState,
    data: &[u8; TERRAIN_TDATA_BYTES],
) {
    if state.bit_counter == 0 {
        state.data_index = (state.data_index + 1) % data.len();
        state.data_pointer = TERRAIN_PATTERN_STREAM_BASE
            .wrapping_add(u16::try_from(state.data_index).expect("TDATA index fits in u16"));
        state.bit_counter = 7;
        state.data_byte = data[state.data_index];
    } else {
        state.bit_counter -= 1;
        let carry = u8::from(state.data_byte & 0x80 != 0);
        state.data_byte = state.data_byte.wrapping_shl(1).wrapping_add(carry);
    }
}

fn advance_terrain_left_state(
    state: &mut TerrainBitState,
    data: &[u8; TERRAIN_TDATA_BYTES],
) {
    if state.bit_counter == 7 {
        state.data_index = if state.data_index == 0 {
            data.len() - 1
        } else {
            state.data_index - 1
        };
        state.data_pointer = TERRAIN_PATTERN_STREAM_BASE
            .wrapping_add(u16::try_from(state.data_index).expect("TDATA index fits in u16"));
        state.bit_counter = 0;
        state.data_byte = rotate_terrain_right_byte(data[state.data_index]);
    } else {
        state.bit_counter += 1;
        state.data_byte = rotate_terrain_right_byte(state.data_byte);
    }
}

fn rotate_terrain_right_byte(data_byte: u8) -> u8 {
    (data_byte >> 1).wrapping_add(if data_byte & 1 == 0 { 0 } else { 0x80 })
}

fn terrain_pattern_bytes() -> &'static [u8; TERRAIN_TDATA_BYTES] {
    crate::arcade_assets::TERRAIN_PATTERN_BYTES
}

fn main_terrain_record_bytes() -> &'static [u8; MAIN_TERRAIN_RECORD_BYTE_COUNT] {
    crate::arcade_assets::MAIN_TERRAIN_BYTES
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SpriteAssetPixel {
    x: u8,
    y: u8,
    tint: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SpriteAssetImageSpec {
    bitmap: crate::arcade_assets::ObjectBitmapId,
    rows: u8,
    byte_columns: u8,
}

fn sprite_asset_pixels(spec: SpriteAssetImageSpec) -> Vec<SpriteAssetPixel> {
    let bytes = crate::arcade_assets::object_bitmap_bytes(spec.bitmap);
    let expected_byte_count = usize::from(spec.rows) * usize::from(spec.byte_columns);
    if bytes.len() != expected_byte_count {
        return Vec::new();
    }
    let mut pixels = Vec::new();
    for column in 0..usize::from(spec.byte_columns) {
        let byte_column_offset = column * usize::from(spec.rows);
        for row in 0..usize::from(spec.rows) {
            let value = bytes[byte_column_offset + row];
            if let Some(tint) = sprite_asset_nibble_tint(value >> 4) {
                pixels.push(SpriteAssetPixel {
                    x: (column * 2) as u8,
                    y: row as u8,
                    tint,
                });
            }
            if let Some(tint) = sprite_asset_nibble_tint(value & 0x0F) {
                pixels.push(SpriteAssetPixel {
                    x: (column * 2 + 1) as u8,
                    y: row as u8,
                    tint,
                });
            }
        }
    }
    pixels
}

fn sprite_asset_nibble_tint(index: u8) -> Option<Color> {
    match index {
        0x0 => None,
        0x1 | 0xA | 0xC | 0xD | 0xE | 0xF => Some(Color::WHITE),
        0x2..=0x9 => Some(williams_color_byte_tint(
            NORMAL_PALETTE_BYTES[usize::from(index)],
        )),
        0xB => Some(Color::from_rgba(170, 170, 186, 0xFF)),
        _ => None,
    }
}

pub(crate) fn push_scanner_radar_sprites(scene: &mut RenderScene, scanner: &ScannerRadarSnapshot) {
    if !scanner.enabled {
        return;
    }

    if scanner.terrain_enabled
        && let Some(scan_left) = scanner.scan_left
    {
        push_scanner_terrain_sprites(scene, scan_left);
    }

    let blip_count = usize::from(scanner.blip_count).min(SCANNER_RADAR_BLIP_LIMIT);
    for blip in &scanner.blips[..blip_count] {
        if VISUAL_STATE.scanner_object_blip_tint(blip.color_word).rgba[3] == 0 {
            continue;
        }
        push_scanner_word_pixels(
            scene,
            SpriteId::SCANNER_OBJECT_BLIP,
            blip.screen_cell,
            blip.color_word,
        );
    }

    let Some(player_blip) = scanner.player_blip else {
        return;
    };
    if VISUAL_STATE
        .scanner_player_blip_tint(player_blip.body_word)
        .rgba[3]
        != 0
    {
        push_scanner_word_pixels(
            scene,
            SpriteId::SCANNER_PLAYER_BLIP,
            player_blip.screen_cell,
            player_blip.body_word,
        );
        push_scanner_byte_pixels(
            scene,
            SpriteId::SCANNER_PLAYER_BLIP,
            player_blip.screen_cell.wrapping_add(2),
            player_blip.tail_byte,
        );
        push_scanner_byte_pixels(
            scene,
            SpriteId::SCANNER_PLAYER_BLIP,
            player_blip.screen_cell.wrapping_sub(0x00FF),
            player_blip.upper_byte,
        );
    }
}

fn push_scanner_terrain_sprites(scene: &mut RenderScene, scan_left: u16) {
    for record in scanner_mini_terrain_records_for_scan_left(scan_left) {
        let origin = screen_position_from_cell(record.screen_cell);
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
                    size: SCANNER_TERRAIN_PIXEL_SIZE,
                    tint: SCANNER_TERRAIN_TINT,
                });
            }
        }
    }
}

fn push_scanner_word_pixels(
    scene: &mut RenderScene,
    sprite: SpriteId,
    screen_cell: crate::ScreenAddress,
    word: u16,
) {
    let [top, bottom] = word.to_be_bytes();
    push_scanner_byte_pixels(scene, sprite, screen_cell, top);
    push_scanner_byte_pixels(scene, sprite, screen_cell.wrapping_add(1), bottom);
}

fn push_scanner_byte_pixels(
    scene: &mut RenderScene,
    sprite: SpriteId,
    screen_cell: crate::ScreenAddress,
    byte: u8,
) {
    let base = screen_position_from_cell(screen_cell);
    for (x_offset, palette_index) in [(0.0, byte >> 4), (1.0, byte & 0x0F)] {
        let tint = video_palette_index_tint(palette_index);
        if tint.rgba[3] == 0 {
            continue;
        }
        scene.push_sprite(SceneSprite {
            sprite,
            layer: RenderLayer::Hud,
            position: [base[0] + x_offset, base[1]],
            size: [1.0, 1.0],
            tint,
        });
    }
}

fn expanded_object_uses_pixel_cloud(detail: &ExpandedObjectDetailSnapshot) -> bool {
    match detail.kind {
        ExpandedObjectKind::Appearance => {
            appearance_growth_scale(detail.size).is_some() && pixel_cloud_sprite_asset(detail).is_some()
        }
        ExpandedObjectKind::Explosion => matches!(
            detail.mapped_sprite,
            Some(
                SpriteId::ENEMY_LANDER
                    | SpriteId::ENEMY_MUTANT
                    | SpriteId::ENEMY_BOMBER
                    | SpriteId::ENEMY_POD
                    | SpriteId::ENEMY_BAITER
                    | SpriteId::SWARMER_EXPLOSION
                    | SpriteId::TERRAIN_EXPLOSION
            )
        ),
        ExpandedObjectKind::ScorePopup => false,
    }
}

pub(crate) fn push_explosion_cloud_pixels(
    scene: &mut RenderScene,
    kind: ExplosionKind,
    position: ScreenPosition,
    cloud_center: Option<ScreenPosition>,
    growth_size: u16,
) -> bool {
    let mut explosion = ExplosionSnapshot::spawn(kind, position);
    explosion.explosion_anchor = cloud_center;
    explosion.growth_size = growth_size;
    let detail = explosion.expanded_object_detail();
    if !expanded_object_uses_pixel_cloud(&detail) {
        return false;
    }

    push_expanded_object_explosion_pixels(scene, &detail);
    true
}

pub(crate) fn push_appearance_cloud_pixels(
    scene: &mut RenderScene,
    position: ScreenPosition,
    sprite_asset_label: &'static str,
    picture_size: (u8, u8),
    mapped_sprite: SpriteId,
    growth_size: u16,
) -> bool {
    let detail = ExpandedObjectDetailSnapshot {
        kind: ExpandedObjectKind::Appearance,
        size: growth_size,
        sprite_asset_label: Some(sprite_asset_label),
        picture_size: Some(picture_size),
        mapped_sprite: Some(mapped_sprite),
        center: Some(appearance_center(position, picture_size)),
        top_left: Some(position),
        ..ExpandedObjectDetailSnapshot::EMPTY
    };
    if !expanded_object_uses_pixel_cloud(&detail) {
        return false;
    }

    push_expanded_object_appearance_pixels(scene, &detail);
    true
}

fn push_expanded_object_explosion_pixels(
    scene: &mut RenderScene,
    detail: &ExpandedObjectDetailSnapshot,
) {
    let Some(top_left) = detail.top_left else {
        return;
    };
    let Some(center) = detail.center else {
        return;
    };
    let Some(spec) = pixel_cloud_sprite_asset(detail) else {
        return;
    };
    let Some(scale) = explosion_growth_scale(detail.size) else {
        return;
    };
    let Some(explosion_frame) = detail.explosion_frame else {
        return;
    };
    if explosion_frame < PIXEL_CLOUD_EXPLOSION_FIRST_VISIBLE_FRAME {
        return;
    }
    let tick = u32::from(explosion_frame);
    push_expanded_object_pixel_cloud(
        scene,
        spec,
        top_left,
        center,
        scale,
        tick,
        RenderLayer::Objects,
    );
}

fn push_expanded_object_appearance_pixels(
    scene: &mut RenderScene,
    detail: &ExpandedObjectDetailSnapshot,
) {
    let Some(top_left) = detail.top_left else {
        return;
    };
    let Some(center) = detail.center else {
        return;
    };
    let Some(spec) = pixel_cloud_sprite_asset(detail) else {
        return;
    };
    let Some(scale) = appearance_growth_scale(detail.size) else {
        return;
    };
    let tick = u32::from(appearance_growth_tick(detail.size));
    push_expanded_object_pixel_cloud(
        scene,
        spec,
        top_left,
        center,
        scale,
        tick,
        RenderLayer::Objects,
    );
}

const PIXEL_CLOUD_EXPLOSION_FIRST_VISIBLE_FRAME: u8 = 2; // original: SOURCE_EXPANDED_OBJECT_EXPLOSION_VISIBLE_FRAME

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PixelCloudAsset {
    sprite_asset_label: &'static str,
    image: SpriteAssetImageSpec,
}

const LANDER_FIRST_SPRITE_ASSET_LABEL: &str = "LNDP1"; // original: LNDP1
const LANDER_SECOND_SPRITE_ASSET_LABEL: &str = "LNDP2"; // original: LNDP2
const LANDER_THIRD_SPRITE_ASSET_LABEL: &str = "LNDP3"; // original: LNDP3
const MUTANT_SPRITE_ASSET_LABEL: &str = "SCZP1"; // original: SCZP1
const BOMBER_FIRST_SPRITE_ASSET_LABEL: &str = "TIEP1"; // original: TIEP1
const BOMBER_SECOND_SPRITE_ASSET_LABEL: &str = "TIEP2"; // original: TIEP2
const BOMBER_THIRD_SPRITE_ASSET_LABEL: &str = "TIEP3"; // original: TIEP3
const BOMBER_FOURTH_SPRITE_ASSET_LABEL: &str = "TIEP4"; // original: TIEP4
const POD_SPRITE_ASSET_LABEL: &str = "PRBP1"; // original: PRBP1
const BAITER_FIRST_SPRITE_ASSET_LABEL: &str = "UFOP1"; // original: UFOP1
const BAITER_SECOND_SPRITE_ASSET_LABEL: &str = "UFOP2"; // original: UFOP2
const BAITER_THIRD_SPRITE_ASSET_LABEL: &str = "UFOP3"; // original: UFOP3
const SWARMER_SPRITE_ASSET_LABEL: &str = "SWPIC1"; // original: SWPIC1
const SWARMER_EXPLOSION_SPRITE_ASSET_LABEL: &str = "SWXP1"; // original: SWXP1
const TERRAIN_EXPLOSION_SPRITE_ASSET_LABEL: &str = "TEREX"; // original: TEREX

const PIXEL_CLOUD_SPRITE_ASSETS: &[PixelCloudAsset] = &[
    PixelCloudAsset {
        sprite_asset_label: LANDER_FIRST_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::LanderFrame1Primary,
            rows: 8,
            byte_columns: 5,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: LANDER_SECOND_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::LanderFrame2Primary,
            rows: 8,
            byte_columns: 5,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: LANDER_THIRD_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::LanderFrame3Primary,
            rows: 8,
            byte_columns: 5,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: MUTANT_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::MutantPrimary,
            rows: 8,
            byte_columns: 5,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: BOMBER_FIRST_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::BomberFrame1Primary,
            rows: 8,
            byte_columns: 4,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: BOMBER_SECOND_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::BomberFrame2Primary,
            rows: 8,
            byte_columns: 4,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: BOMBER_THIRD_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::BomberFrame3Primary,
            rows: 8,
            byte_columns: 4,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: BOMBER_FOURTH_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::BomberFrame4Primary,
            rows: 8,
            byte_columns: 4,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: POD_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::PodPrimary,
            rows: 8,
            byte_columns: 4,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: BAITER_FIRST_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::BaiterFrame1Primary,
            rows: 4,
            byte_columns: 6,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: BAITER_SECOND_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::BaiterFrame2Primary,
            rows: 4,
            byte_columns: 6,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: BAITER_THIRD_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::BaiterFrame3Primary,
            rows: 4,
            byte_columns: 6,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: SWARMER_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::SwarmerPrimary,
            rows: 4,
            byte_columns: 3,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: SWARMER_EXPLOSION_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::SwarmerExplosion,
            rows: 8,
            byte_columns: 4,
        },
    },
    PixelCloudAsset {
        sprite_asset_label: TERRAIN_EXPLOSION_SPRITE_ASSET_LABEL,
        image: SpriteAssetImageSpec {
            bitmap: ObjectBitmapId::TerrainExplosion,
            rows: 6,
            byte_columns: 8,
        },
    },
];

fn pixel_cloud_sprite_asset(detail: &ExpandedObjectDetailSnapshot) -> Option<SpriteAssetImageSpec> {
    let sprite_asset_label = detail.sprite_asset_label?;
    PIXEL_CLOUD_SPRITE_ASSETS
        .iter()
        .find(|asset| asset.sprite_asset_label == sprite_asset_label)
        .map(|asset| asset.image)
}

fn push_expanded_object_pixel_cloud(
    scene: &mut RenderScene,
    spec: SpriteAssetImageSpec,
    top_left: ScreenPosition,
    center: ScreenPosition,
    scale: u8,
    tick: u32,
    layer: RenderLayer,
) {
    let pixels = sprite_asset_pixels(spec);
    if pixels.is_empty() {
        return;
    }

    let scale = i32::from(scale);
    let top_left_x = i32::from(top_left.x);
    let top_left_y = i32::from(top_left.y);
    let center_x = i32::from(center.x);
    let center_y = i32::from(center.y);
    let x_start = center_x - scale * (center_x - top_left_x);
    let vertical_delta = center_y - top_left_y;
    let y_flavor = vertical_delta & 1;
    let y_offset = vertical_delta / 2;
    let y_start = center_y - (scale * 2 * y_offset) - y_flavor;

    for (index, pixel) in pixels.into_iter().enumerate() {
        let target_x = x_start + i32::from(pixel.x / 2) * scale * 2 + i32::from(pixel.x % 2);
        let target_y = y_start + i32::from(pixel.y / 2) * scale * 2 + i32::from(pixel.y % 2);
        if target_x < 0
            || target_y < 0
            || target_x >= scene.surface.width as i32
            || target_y >= scene.surface.height as i32
        {
            continue;
        }

        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_EXPLOSION_PIXEL,
            layer,
            position: [target_x as f32, target_y as f32],
            size: [1.0, 1.0],
            tint: pixel_cloud_tint(pixel.tint, tick, index),
        });
    }
}

fn pixel_cloud_tint(base_tint: Color, tick: u32, index: usize) -> Color {
    if tick < 2 && index.is_multiple_of(3) {
        return cycling_palette_tint(index);
    }
    base_tint
}

fn cycling_palette_tint(phase: usize) -> Color {
    williams_color_byte_tint(COLTAB_COLOR_BYTES[phase % COLTAB_ACTIVE_BYTES])
}

const EXPLOSION_RENDER_MAX_SCALE: u8 = 3; // original: SOURCE_EXPLOSION_RENDER_MAX_SCALE

pub(crate) fn explosion_render_scale(size: u16) -> Option<u16> {
    explosion_growth_scale(size).map(|scale| u16::from(scale.min(EXPLOSION_RENDER_MAX_SCALE)))
}

pub(crate) fn explosion_growth_scale(size: u16) -> Option<u8> {
    let high = size.to_be_bytes()[0] & 0x7F;
    if high == 0 || high > EXPLOSION_KILL_SIZE_HIGH {
        return None;
    }
    Some(high)
}

fn appearance_growth_scale(size: u16) -> Option<u8> {
    if size & 0x8000 == 0 {
        return None;
    }
    let scale = size.to_be_bytes()[0] & 0x7F;
    (scale > 0).then_some(scale)
}

fn appearance_growth_tick(size: u16) -> u8 {
    let start = APPEARANCE_INITIAL_SIZE.to_be_bytes()[0];
    let current = size.to_be_bytes()[0];
    start.saturating_sub(current)
}

pub(crate) fn explosion_frame_index(size: u16) -> Option<u8> {
    if explosion_growth_scale(size).is_none() || size < EXPLOSION_INITIAL_SIZE {
        return None;
    }
    let offset = size.wrapping_sub(EXPLOSION_INITIAL_SIZE);
    if !offset.is_multiple_of(EXPLOSION_SIZE_DELTA) {
        return None;
    }
    u8::try_from(offset / EXPLOSION_SIZE_DELTA).ok()
}
