#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SpriteAssetPixel {
    x: u8,
    y: u8,
    tint: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SpriteAssetImageSpec {
    asset_label: &'static str,
    rows: u8,
    bytes_per_row: u8,
}

fn sprite_asset_pixels(spec: SpriteAssetImageSpec) -> Vec<SpriteAssetPixel> {
    let bytes = sprite_asset_image_bytes(spec.asset_label);
    let expected_byte_count = usize::from(spec.rows) * usize::from(spec.bytes_per_row);
    if bytes.len() != expected_byte_count {
        return Vec::new();
    }
    let mut pixels = Vec::new();
    for column in 0..usize::from(spec.bytes_per_row) {
        let source_column = column * usize::from(spec.rows);
        for row in 0..usize::from(spec.rows) {
            let value = bytes[source_column + row];
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

fn sprite_asset_image_bytes(asset_label: &'static str) -> Vec<u8> {
    for line in OBJECT_IMAGES_TSV.lines().skip(1) {
        let mut columns = line.split('\t');
        let Some(image_label) = columns.next() else {
            continue;
        };
        let _address = columns.next();
        let Some(hex_bytes) = columns.next() else {
            continue;
        };
        if image_label == asset_label {
            return decode_sprite_asset_hex_bytes(asset_label, hex_bytes);
        }
    }
    Vec::new()
}

fn decode_sprite_asset_hex_bytes(asset_label: &'static str, hex_bytes: &str) -> Vec<u8> {
    assert!(
        hex_bytes.len().is_multiple_of(2),
        "sprite asset image {asset_label} hex byte string must be even length"
    );
    (0..hex_bytes.len())
        .step_by(2)
        .map(|start| {
            u8::from_str_radix(&hex_bytes[start..start + 2], 16).unwrap_or_else(|error| {
                panic!("sprite asset image {asset_label} hex must parse: {error}")
            })
        })
        .collect()
}

fn sprite_asset_nibble_tint(index: u8) -> Option<Color> {
    match index {
        0x0 => None,
        0x1 | 0xA | 0xC | 0xD | 0xE | 0xF => Some(Color::WHITE),
        0x2..=0x9 => Some(source_pseudo_color_tint(
            NORMAL_PALETTE_BYTES[usize::from(index)],
        )),
        0xB => Some(Color::from_rgba(170, 170, 186, 0xFF)),
        _ => None,
    }
}

fn expanded_object_uses_pixel_cloud(detail: &ExpandedObjectDetailSnapshot) -> bool {
    match detail.kind {
        ExpandedObjectKind::Appearance => {
            source_appearance_size_scale(detail.size).is_some()
                && pixel_cloud_sprite_asset(detail).is_some()
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
    let mut explosion = ExplosionSnapshot::source_spawn(kind, position);
    explosion.source_center = cloud_center;
    explosion.source_size = growth_size;
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
    sprite_frame_label: &'static str,
    picture_size: (u8, u8),
    mapped_sprite: SpriteId,
    growth_size: u16,
) -> bool {
    let detail = ExpandedObjectDetailSnapshot {
        kind: ExpandedObjectKind::Appearance,
        size: growth_size,
        sprite_frame_label: Some(sprite_frame_label),
        picture_size: Some(picture_size),
        mapped_sprite: Some(mapped_sprite),
        center: Some(source_appearance_center(position, picture_size)),
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
    let Some(scale) = source_explosion_size_scale(detail.size) else {
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
    let Some(scale) = source_appearance_size_scale(detail.size) else {
        return;
    };
    let tick = u32::from(source_appearance_tick(detail.size));
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
    sprite_frame_label: &'static str,
    image: SpriteAssetImageSpec,
}

const LANDER_FIRST_SPRITE_FRAME_LABEL: &str = "LNDP1"; // original: LNDP1
const LANDER_SECOND_SPRITE_FRAME_LABEL: &str = "LNDP2"; // original: LNDP2
const LANDER_THIRD_SPRITE_FRAME_LABEL: &str = "LNDP3"; // original: LNDP3
const MUTANT_SPRITE_FRAME_LABEL: &str = "SCZP1"; // original: SCZP1
const BOMBER_FIRST_SPRITE_FRAME_LABEL: &str = "TIEP1"; // original: TIEP1
const BOMBER_SECOND_SPRITE_FRAME_LABEL: &str = "TIEP2"; // original: TIEP2
const BOMBER_THIRD_SPRITE_FRAME_LABEL: &str = "TIEP3"; // original: TIEP3
const BOMBER_FOURTH_SPRITE_FRAME_LABEL: &str = "TIEP4"; // original: TIEP4
const POD_SPRITE_FRAME_LABEL: &str = "PRBP1"; // original: PRBP1
const BAITER_FIRST_SPRITE_FRAME_LABEL: &str = "UFOP1"; // original: UFOP1
const BAITER_SECOND_SPRITE_FRAME_LABEL: &str = "UFOP2"; // original: UFOP2
const BAITER_THIRD_SPRITE_FRAME_LABEL: &str = "UFOP3"; // original: UFOP3
const SWARMER_SPRITE_FRAME_LABEL: &str = "SWPIC1"; // original: SWPIC1
const SWARMER_EXPLOSION_SPRITE_FRAME_LABEL: &str = "SWXP1"; // original: SWXP1
const TERRAIN_EXPLOSION_SPRITE_FRAME_LABEL: &str = "TEREX"; // original: TEREX

const LANDER_FIRST_PIXEL_CLOUD_ASSET_LABEL: &str = "LND10"; // original: LND10
const LANDER_SECOND_PIXEL_CLOUD_ASSET_LABEL: &str = "LND20"; // original: LND20
const LANDER_THIRD_PIXEL_CLOUD_ASSET_LABEL: &str = "LND30"; // original: LND30
const MUTANT_PIXEL_CLOUD_ASSET_LABEL: &str = "SCZD10"; // original: SCZD10
const BOMBER_FIRST_PIXEL_CLOUD_ASSET_LABEL: &str = "TIED10"; // original: TIED10
const BOMBER_SECOND_PIXEL_CLOUD_ASSET_LABEL: &str = "TIED20"; // original: TIED20
const BOMBER_THIRD_PIXEL_CLOUD_ASSET_LABEL: &str = "TIED30"; // original: TIED30
const BOMBER_FOURTH_PIXEL_CLOUD_ASSET_LABEL: &str = "TIED40"; // original: TIED40
const POD_PIXEL_CLOUD_ASSET_LABEL: &str = "PRBD10"; // original: PRBD10
const BAITER_FIRST_PIXEL_CLOUD_ASSET_LABEL: &str = "UFOD10"; // original: UFOD10
const BAITER_SECOND_PIXEL_CLOUD_ASSET_LABEL: &str = "UFOD20"; // original: UFOD20
const BAITER_THIRD_PIXEL_CLOUD_ASSET_LABEL: &str = "UFOD30"; // original: UFOD30
const SWARMER_PIXEL_CLOUD_ASSET_LABEL: &str = "SWMD10"; // original: SWMD10
const SWARMER_EXPLOSION_PIXEL_CLOUD_ASSET_LABEL: &str = "SWXD10"; // original: SWXD10
const TERRAIN_EXPLOSION_PIXEL_CLOUD_ASSET_LABEL: &str = "TERX0"; // original: TERX0

const PIXEL_CLOUD_SPRITE_ASSETS: &[PixelCloudAsset] = &[
    PixelCloudAsset {
        sprite_frame_label: LANDER_FIRST_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: LANDER_FIRST_PIXEL_CLOUD_ASSET_LABEL,
            rows: 8,
            bytes_per_row: 5,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: LANDER_SECOND_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: LANDER_SECOND_PIXEL_CLOUD_ASSET_LABEL,
            rows: 8,
            bytes_per_row: 5,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: LANDER_THIRD_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: LANDER_THIRD_PIXEL_CLOUD_ASSET_LABEL,
            rows: 8,
            bytes_per_row: 5,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: MUTANT_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: MUTANT_PIXEL_CLOUD_ASSET_LABEL,
            rows: 8,
            bytes_per_row: 5,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: BOMBER_FIRST_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: BOMBER_FIRST_PIXEL_CLOUD_ASSET_LABEL,
            rows: 8,
            bytes_per_row: 4,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: BOMBER_SECOND_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: BOMBER_SECOND_PIXEL_CLOUD_ASSET_LABEL,
            rows: 8,
            bytes_per_row: 4,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: BOMBER_THIRD_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: BOMBER_THIRD_PIXEL_CLOUD_ASSET_LABEL,
            rows: 8,
            bytes_per_row: 4,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: BOMBER_FOURTH_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: BOMBER_FOURTH_PIXEL_CLOUD_ASSET_LABEL,
            rows: 8,
            bytes_per_row: 4,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: POD_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: POD_PIXEL_CLOUD_ASSET_LABEL,
            rows: 8,
            bytes_per_row: 4,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: BAITER_FIRST_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: BAITER_FIRST_PIXEL_CLOUD_ASSET_LABEL,
            rows: 4,
            bytes_per_row: 6,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: BAITER_SECOND_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: BAITER_SECOND_PIXEL_CLOUD_ASSET_LABEL,
            rows: 4,
            bytes_per_row: 6,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: BAITER_THIRD_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: BAITER_THIRD_PIXEL_CLOUD_ASSET_LABEL,
            rows: 4,
            bytes_per_row: 6,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: SWARMER_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: SWARMER_PIXEL_CLOUD_ASSET_LABEL,
            rows: 4,
            bytes_per_row: 3,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: SWARMER_EXPLOSION_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: SWARMER_EXPLOSION_PIXEL_CLOUD_ASSET_LABEL,
            rows: 8,
            bytes_per_row: 4,
        },
    },
    PixelCloudAsset {
        sprite_frame_label: TERRAIN_EXPLOSION_SPRITE_FRAME_LABEL,
        image: SpriteAssetImageSpec {
            asset_label: TERRAIN_EXPLOSION_PIXEL_CLOUD_ASSET_LABEL,
            rows: 6,
            bytes_per_row: 8,
        },
    },
];

fn pixel_cloud_sprite_asset(detail: &ExpandedObjectDetailSnapshot) -> Option<SpriteAssetImageSpec> {
    let sprite_frame_label = detail.sprite_frame_label?;
    PIXEL_CLOUD_SPRITE_ASSETS
        .iter()
        .find(|asset| asset.sprite_frame_label == sprite_frame_label)
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
    let y_offset_raw = center_y - top_left_y;
    let y_flavor = y_offset_raw & 1;
    let y_offset = y_offset_raw / 2;
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

fn pixel_cloud_tint(source_tint: Color, tick: u32, index: usize) -> Color {
    if tick < 2 && index.is_multiple_of(3) {
        return cycling_palette_tint(index);
    }
    source_tint
}

fn cycling_palette_tint(phase: usize) -> Color {
    source_pseudo_color_tint(COLTAB_COLOR_BYTES[phase % COLTAB_ACTIVE_BYTES])
}

const EXPLOSION_RENDER_MAX_SCALE: u8 = 3; // original: SOURCE_EXPLOSION_RENDER_MAX_SCALE

pub(crate) fn source_explosion_render_scale(size: u16) -> Option<u16> {
    source_explosion_size_scale(size).map(|scale| u16::from(scale.min(EXPLOSION_RENDER_MAX_SCALE)))
}

pub(crate) fn source_explosion_size_scale(size: u16) -> Option<u8> {
    let high = size.to_be_bytes()[0] & 0x7F;
    if high == 0 || high > EXPLOSION_KILL_SIZE_HIGH {
        return None;
    }
    Some(high)
}

fn source_appearance_size_scale(size: u16) -> Option<u8> {
    if size & 0x8000 == 0 {
        return None;
    }
    let scale = size.to_be_bytes()[0] & 0x7F;
    (scale > 0).then_some(scale)
}

fn source_appearance_tick(size: u16) -> u8 {
    let start = APPEARANCE_INITIAL_SIZE.to_be_bytes()[0];
    let current = size.to_be_bytes()[0];
    start.saturating_sub(current)
}

pub(crate) fn source_explosion_frame_index(size: u16) -> Option<u8> {
    if source_explosion_size_scale(size).is_none() || size < EXPLOSION_INITIAL_SIZE {
        return None;
    }
    let offset = size.wrapping_sub(EXPLOSION_INITIAL_SIZE);
    if !offset.is_multiple_of(EXPLOSION_SIZE_DELTA) {
        return None;
    }
    u8::try_from(offset / EXPLOSION_SIZE_DELTA).ok()
}
