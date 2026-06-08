fn attract_title_reference_sample_index(page_frame: u16) -> usize {
    usize::from(page_frame / ATTRACT_TITLE_REFERENCE_SAMPLE_STEP_FRAMES).saturating_sub(1)
}

fn source_pseudo_color_tint(value: u8) -> Color {
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

pub(crate) fn source_wave_landscape_tint(wave: u16) -> Color {
    let wave = wave.max(1);
    let index = usize::from((wave - 1) % WAVE_LANDSCAPE_COLOR_BYTES.len() as u16);
    source_pseudo_color_tint(WAVE_LANDSCAPE_COLOR_BYTES[index])
}

pub(crate) fn source_terrain_blow_flash_tint(elapsed: u16) -> Color {
    let color = TERRAIN_BLOW_FLASH_WINDOWS
        .iter()
        .find_map(|(start, end, color)| (*start <= elapsed && elapsed <= *end).then_some(*color))
        .unwrap_or(0);
    source_pseudo_color_tint(color)
}

fn source_video_palette_index_tint(index: u8) -> Color {
    source_pseudo_color_tint(NORMAL_PALETTE_BYTES[usize::from(index & 0x0F)])
}

fn source_video_word_tint(word: u16) -> Color {
    source_video_palette_index_tint((word & 0x000F) as u8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceTerrainFlavorRecord {
    offset: u8,
    word: u16,
}

impl SourceTerrainFlavorRecord {
    const EMPTY: Self = Self { offset: 0, word: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceTerrainDrawRecord {
    screen_address: u16,
    word: u16,
}

impl SourceTerrainDrawRecord {
    const EMPTY: Self = Self {
        screen_address: 0,
        word: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceScannerTerrainRecord {
    screen_address: u16,
    word: u16,
}

impl SourceScannerTerrainRecord {
    const EMPTY: Self = Self {
        screen_address: 0,
        word: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceTerrainBitState {
    data_index: usize,
    data_pointer: u16,
    data_byte: u8,
    bit_counter: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceTerrainGenerationState {
    left: SourceTerrainBitState,
    right: SourceTerrainBitState,
    left_offset: u8,
    right_offset: u8,
    background_left: u16,
    terrain_left: u16,
    flavor_0_pointer: usize,
    flavor_1_pointer: usize,
}

pub(crate) fn push_source_bgout_terrain_sprites(
    scene: &mut RenderScene,
    background_left: u16,
    tint: Color,
) {
    for record in source_bgout_terrain_records(background_left) {
        scene.push_sprite(SceneSprite {
            sprite: source_terrain_word_sprite(record.word),
            layer: RenderLayer::Terrain,
            position: source_screen_position(record.screen_address),
            size: TERRAIN_WORD_SIZE,
            tint,
        });
    }
}

fn source_terrain_word_sprite(word: u16) -> SpriteId {
    if word == TERRAIN_WORD_0770 {
        SpriteId::TERRAIN_TILE_ALT
    } else {
        SpriteId::TERRAIN_TILE
    }
}

fn source_bgout_default_terrain_records() -> &'static [SourceTerrainDrawRecord; TERRAIN_SCREEN_WORDS]
{
    static RECORDS: OnceLock<[SourceTerrainDrawRecord; TERRAIN_SCREEN_WORDS]> = OnceLock::new();
    RECORDS.get_or_init(|| source_generate_bgout_terrain_records(0))
}

fn source_bgout_terrain_records(
    background_left: u16,
) -> [SourceTerrainDrawRecord; TERRAIN_SCREEN_WORDS] {
    let terrain_left = source_bgout_terrain_left(background_left);
    if terrain_left == 0 {
        return *source_bgout_default_terrain_records();
    }
    source_generate_bgout_terrain_records(terrain_left)
}

fn source_scanner_mini_terrain_records_for_scan_left(
    scan_left: u16,
) -> [SourceScannerTerrainRecord; SCANNER_TERRAIN_RECORDS] {
    source_generate_scanner_mini_terrain_records(scan_left)
}

fn source_generate_bgout_terrain_records(
    terrain_left: u16,
) -> [SourceTerrainDrawRecord; TERRAIN_SCREEN_WORDS] {
    let data = source_tdata_bytes();
    let terrain_left = source_bgout_terrain_left(terrain_left);
    let (flavor_0, flavor_1, state) = source_initialize_terrain_flavor_tables(data, terrain_left);
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

    let mut records = [SourceTerrainDrawRecord::EMPTY; TERRAIN_SCREEN_WORDS];
    for (entry_index, record) in records.iter_mut().enumerate() {
        let source_record =
            selected_flavor[(selected_pointer + entry_index) % selected_flavor.len()];
        *record = SourceTerrainDrawRecord {
            screen_address: u16::from_be_bytes([
                0x98u8.wrapping_sub(
                    u8::try_from(entry_index).expect("BGOUT terrain entry index fits in u8"),
                ),
                source_record.offset,
            ]),
            word: source_record.word,
        };
    }
    records
}

const fn source_bgout_terrain_left(background_left: u16) -> u16 {
    background_left & 0xFFE0
}

fn source_generate_scanner_mini_terrain_records(
    scan_left: u16,
) -> [SourceScannerTerrainRecord; SCANNER_TERRAIN_RECORDS] {
    let bytes = source_mterr_bytes();
    let first_record = usize::from(scan_left.to_be_bytes()[0] >> 2);
    assert!(
        first_record + SCANNER_TERRAIN_RECORDS <= SCANNER_MINI_TERRAIN_RECORDS,
        "MTERR slice must contain 64 source scanner terrain records"
    );

    let mut records = [SourceScannerTerrainRecord::EMPTY; SCANNER_TERRAIN_RECORDS];
    let mut source_column = SCANNER_OBJECT_BASE_SCREEN.to_be_bytes()[0];
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

fn source_initialize_terrain_flavor_tables(
    data: &[u8; TERRAIN_TDATA_BYTES],
    terrain_left: u16,
) -> (
    [SourceTerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
    [SourceTerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
    SourceTerrainGenerationState,
) {
    let (right, right_offset) = source_alinit_final_terrain_state(data);
    let mut generation_left = terrain_left.wrapping_add(0x2610);
    let mut left = SourceTerrainBitState {
        data_index: data.len() - 1,
        data_pointer: TERRAIN_TDATA_ADDRESS.wrapping_sub(1),
        data_byte: 0,
        bit_counter: 0,
    };
    let mut left_offset = 0xE0;
    source_advance_terrain_right_state(&mut left, data);

    let mut scan_x = 0x0010u16;
    for _ in 0..=0x0800 {
        if scan_x == generation_left {
            break;
        }
        left_offset = source_terrain_altitude_step(left_offset, left.data_byte);
        source_advance_terrain_right_state(&mut left, data);
        scan_x = scan_x.wrapping_add(0x20);
    }
    assert_eq!(
        scan_x, generation_left,
        "BGINIT terrain stream must align to BGLX 0x{generation_left:04X}"
    );

    let saved_right = left;
    let saved_right_offset = left_offset;
    let mut flavor_0 = [SourceTerrainFlavorRecord::EMPTY; TERRAIN_FLAVOR_RECORDS];
    let mut flavor_1 = [SourceTerrainFlavorRecord::EMPTY; TERRAIN_FLAVOR_RECORDS];
    let mut state = SourceTerrainGenerationState {
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
        source_add_left_terrain_pixel(&mut state, data, &mut flavor_0, &mut flavor_1);
    }

    state.right = saved_right;
    state.right_offset = saved_right_offset;
    (flavor_0, flavor_1, state)
}

fn source_add_left_terrain_pixel(
    state: &mut SourceTerrainGenerationState,
    data: &[u8; TERRAIN_TDATA_BYTES],
    flavor_0: &mut [SourceTerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
    flavor_1: &mut [SourceTerrainFlavorRecord; TERRAIN_FLAVOR_RECORDS],
) {
    source_advance_terrain_left_state(&mut state.right, data);
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

    source_advance_terrain_left_state(&mut state.left, data);
    let (offset, word) = if state.left.data_byte & 0x80 == 0 {
        state.left_offset = state.left_offset.wrapping_sub(1);
        (state.left_offset, TERRAIN_WORD_7007)
    } else {
        let offset = state.left_offset;
        state.left_offset = state.left_offset.wrapping_add(1);
        (offset, TERRAIN_WORD_0770)
    };

    let record = SourceTerrainFlavorRecord { offset, word };
    if flavor_0_selected {
        flavor_0[record_index] = record;
        state.flavor_0_pointer = (record_index + 1) % TERRAIN_FLAVOR_RECORDS;
    } else {
        flavor_1[record_index] = record;
        state.flavor_1_pointer = (record_index + 1) % TERRAIN_FLAVOR_RECORDS;
    }
}

fn source_alinit_final_terrain_state(
    data: &[u8; TERRAIN_TDATA_BYTES],
) -> (SourceTerrainBitState, u8) {
    let mut state = SourceTerrainBitState {
        data_index: 0,
        data_pointer: TERRAIN_TDATA_ADDRESS,
        data_byte: data[0],
        bit_counter: 7,
    };
    let mut offset = 0xE0;
    for _ in 0..0x0400 {
        offset = source_terrain_altitude_step(offset, state.data_byte);
        source_advance_terrain_right_state(&mut state, data);
        offset = source_terrain_altitude_step(offset, state.data_byte);
        source_advance_terrain_right_state(&mut state, data);
    }
    (state, offset)
}

fn source_terrain_altitude_step(offset: u8, data_byte: u8) -> u8 {
    if data_byte & 0x80 != 0 {
        offset.wrapping_sub(1)
    } else {
        offset.wrapping_add(1)
    }
}

fn source_advance_terrain_right_state(
    state: &mut SourceTerrainBitState,
    data: &[u8; TERRAIN_TDATA_BYTES],
) {
    if state.bit_counter == 0 {
        state.data_index = (state.data_index + 1) % data.len();
        state.data_pointer = TERRAIN_TDATA_ADDRESS
            .wrapping_add(u16::try_from(state.data_index).expect("TDATA index fits in u16"));
        state.bit_counter = 7;
        state.data_byte = data[state.data_index];
    } else {
        state.bit_counter -= 1;
        let carry = u8::from(state.data_byte & 0x80 != 0);
        state.data_byte = state.data_byte.wrapping_shl(1).wrapping_add(carry);
    }
}

fn source_advance_terrain_left_state(
    state: &mut SourceTerrainBitState,
    data: &[u8; TERRAIN_TDATA_BYTES],
) {
    if state.bit_counter == 7 {
        state.data_index = if state.data_index == 0 {
            data.len() - 1
        } else {
            state.data_index - 1
        };
        state.data_pointer = TERRAIN_TDATA_ADDRESS
            .wrapping_add(u16::try_from(state.data_index).expect("TDATA index fits in u16"));
        state.bit_counter = 0;
        state.data_byte = source_rotate_terrain_right_byte(data[state.data_index]);
    } else {
        state.bit_counter += 1;
        state.data_byte = source_rotate_terrain_right_byte(state.data_byte);
    }
}

fn source_rotate_terrain_right_byte(data_byte: u8) -> u8 {
    (data_byte >> 1).wrapping_add(if data_byte & 1 == 0 { 0 } else { 0x80 })
}

fn source_tdata_bytes() -> &'static [u8; TERRAIN_TDATA_BYTES] {
    static TDATA: OnceLock<[u8; TERRAIN_TDATA_BYTES]> = OnceLock::new();
    TDATA.get_or_init(parse_source_tdata_bytes)
}

fn source_mterr_bytes() -> &'static [u8; MAIN_TERRAIN_RECORD_BYTE_COUNT] {
    static MTERR: OnceLock<[u8; MAIN_TERRAIN_RECORD_BYTE_COUNT]> = OnceLock::new();
    MTERR.get_or_init(parse_source_mterr_bytes)
}

fn parse_source_tdata_bytes() -> [u8; TERRAIN_TDATA_BYTES] {
    let mut output = [0; TERRAIN_TDATA_BYTES];
    for (line_index, line) in TERRAIN_DATA_TSV.lines().enumerate().skip(1) {
        let mut fields = line.split('\t');
        let label = fields.next().unwrap_or_default();
        let address = fields.next().unwrap_or_default();
        let bytes = fields.next().unwrap_or_default();
        if label != TERRAIN_TDATA_LABEL {
            continue;
        }
        assert_eq!(
            address,
            "0xC350",
            "terrain-data line {} must preserve TDATA source address",
            line_index + 1
        );
        assert_eq!(
            bytes.len(),
            TERRAIN_TDATA_BYTES * 2,
            "TDATA hex payload must contain exactly 0x100 bytes"
        );
        for index in 0..TERRAIN_TDATA_BYTES {
            output[index] = parse_source_hex_byte(&bytes[index * 2..index * 2 + 2]);
        }
        return output;
    }

    panic!("terrain-data.tsv must contain the TDATA record")
}

fn parse_source_mterr_bytes() -> [u8; MAIN_TERRAIN_RECORD_BYTE_COUNT] {
    let mut output = [0; MAIN_TERRAIN_RECORD_BYTE_COUNT];
    for (line_index, line) in TERRAIN_DATA_TSV.lines().enumerate().skip(1) {
        let mut fields = line.split('\t');
        let label = fields.next().unwrap_or_default();
        let address = fields.next().unwrap_or_default();
        let bytes = fields.next().unwrap_or_default();
        if label != MAIN_TERRAIN_RECORD_LABEL {
            continue;
        }
        let expected_address = format!("0x{MAIN_TERRAIN_RECORD_ADDRESS:04X}");
        assert_eq!(
            address,
            expected_address.as_str(),
            "terrain-data line {} must preserve MTERR source address",
            line_index + 1
        );
        assert_eq!(
            bytes.len(),
            MAIN_TERRAIN_RECORD_BYTE_COUNT * 2,
            "MTERR hex payload must contain exactly 0x180 bytes"
        );
        for index in 0..MAIN_TERRAIN_RECORD_BYTE_COUNT {
            output[index] = parse_source_hex_byte(&bytes[index * 2..index * 2 + 2]);
        }
        return output;
    }

    panic!("terrain-data.tsv must contain the MTERR record")
}

fn parse_source_hex_byte(value: &str) -> u8 {
    u8::from_str_radix(value, 16).expect("source terrain byte must be hexadecimal")
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
            blip.screen_address,
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
            player_blip.screen_address,
            player_blip.body_word,
        );
        push_scanner_byte_pixels(
            scene,
            SpriteId::SCANNER_PLAYER_BLIP,
            player_blip.screen_address.wrapping_add(2),
            player_blip.tail_byte,
        );
        push_scanner_byte_pixels(
            scene,
            SpriteId::SCANNER_PLAYER_BLIP,
            player_blip.screen_address.wrapping_sub(0x00FF),
            player_blip.upper_byte,
        );
    }
}

fn push_scanner_terrain_sprites(scene: &mut RenderScene, scan_left: u16) {
    for record in source_scanner_mini_terrain_records_for_scan_left(scan_left) {
        let origin = source_screen_position(record.screen_address);
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
    screen_address: u16,
    word: u16,
) {
    let [top, bottom] = word.to_be_bytes();
    push_scanner_byte_pixels(scene, sprite, screen_address, top);
    push_scanner_byte_pixels(scene, sprite, screen_address.wrapping_add(1), bottom);
}

fn push_scanner_byte_pixels(
    scene: &mut RenderScene,
    sprite: SpriteId,
    screen_address: u16,
    byte: u8,
) {
    let base = source_screen_position(screen_address);
    for (x_offset, palette_index) in [(0.0, byte >> 4), (1.0, byte & 0x0F)] {
        let tint = source_video_palette_index_tint(palette_index);
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
