use crate::arcade_assets::{
    MESSAGE_GLYPH_ASSETS, ObjectBitmapId, SCORE_DIGIT_ASSETS, object_bitmap_bytes,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NativeRenderPipeline {
    TemporaryRaster,
    Terrain,
    Starfield,
    Sprites,
    Projectiles,
    Explosions,
    HudText,
    DebugOverlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasRegion {
    pub sprite: SpriteId,
    pub origin: [u32; 2],
    pub size: [u32; 2],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureAtlas {
    pub surface: SurfaceSize,
    pub regions: Vec<AtlasRegion>,
    pixels: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EmbeddedSprite {
    surface: SurfaceSize,
    pixels: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpriteAtlasSourceRegion {
    origin: [u32; 2],
    size: [u32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectPictureGrid {
    rows: u8,
    bytes_per_row: u8,
    bytes: &'static [u8],
    palette: ObjectPicturePalette,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectPicturePalette {
    one: [u8; 4],
    a: [u8; 4],
    c: [u8; 4],
    d: [u8; 4],
    e: [u8; 4],
    f: [u8; 4],
}

impl ObjectPicturePalette {
    const fn white() -> Self {
        Self {
            one: WHITE_RGBA,
            a: WHITE_RGBA,
            c: WHITE_RGBA,
            d: WHITE_RGBA,
            e: WHITE_RGBA,
            f: WHITE_RGBA,
        }
    }

    fn defender_logo() -> Self {
        Self {
            one: WHITE_RGBA,
            a: WHITE_RGBA,
            c: pseudo_color_rgba(0x3F),
            d: WHITE_RGBA,
            e: WHITE_RGBA,
            f: WHITE_RGBA,
        }
    }

    fn ship() -> Self {
        let white = pseudo_color_rgba(PICTURE_COLOR_TABLE[9]);
        Self {
            one: white,
            a: white,
            c: PURPLE_RGBA,
            d: PURPLE_RGBA,
            e: GRAY_RGBA,
            f: WHITE_RGBA,
        }
    }

    const fn player_shot() -> Self {
        Self {
            one: WHITE_RGBA,
            a: WHITE_RGBA,
            c: WHITE_RGBA,
            d: WHITE_RGBA,
            e: WHITE_RGBA,
            f: PALE_YELLOW_RGBA,
        }
    }

    fn bomb(phase: usize) -> Self {
        let color = object_cycle_palette_color(phase);
        Self {
            one: WHITE_RGBA,
            a: color,
            c: WHITE_RGBA,
            d: WHITE_RGBA,
            e: WHITE_RGBA,
            f: WHITE_RGBA,
        }
    }

    fn tie(phase: usize) -> Self {
        let offset = (phase % 3) * 3;
        Self {
            one: WHITE_RGBA,
            a: WHITE_RGBA,
            c: WHITE_RGBA,
            d: pseudo_color_rgba(BOMBER_COLOR_SEQUENCE[offset]),
            e: pseudo_color_rgba(BOMBER_COLOR_SEQUENCE[offset + 1]),
            f: pseudo_color_rgba(BOMBER_COLOR_SEQUENCE[offset + 2]),
        }
    }

    const fn score_250(phase: usize) -> Self {
        const CYCLE: [[u8; 4]; 3] = [BLUE_RGBA, YELLOW_RGBA, WHITE_RGBA];
        Self {
            one: CYCLE[phase % CYCLE.len()],
            a: WHITE_RGBA,
            c: WHITE_RGBA,
            d: WHITE_RGBA,
            e: WHITE_RGBA,
            f: WHITE_RGBA,
        }
    }

    const fn score_500(phase: usize) -> Self {
        const CYCLE: [[u8; 4]; 3] = [RED_RGBA, BLUE_RGBA, YELLOW_RGBA];
        Self {
            one: WHITE_RGBA,
            a: WHITE_RGBA,
            c: WHITE_RGBA,
            d: CYCLE[phase % CYCLE.len()],
            e: CYCLE[(phase + 1) % CYCLE.len()],
            f: CYCLE[(phase + 2) % CYCLE.len()],
        }
    }

    const fn burst() -> Self {
        Self {
            one: WHITE_RGBA,
            a: WHITE_RGBA,
            c: YELLOW_RGBA,
            d: RED_RGBA,
            e: BLUE_RGBA,
            f: WHITE_RGBA,
        }
    }
}

const WHITE_RGBA: [u8; 4] = [255, 255, 255, 255];
const YELLOW_RGBA: [u8; 4] = [255, 188, 0, 255];
const RED_RGBA: [u8; 4] = [255, 80, 80, 255];
const BLUE_RGBA: [u8; 4] = [40, 56, 220, 255];
const GRAY_RGBA: [u8; 4] = [170, 170, 186, 255];
const PURPLE_RGBA: [u8; 4] = [182, 48, 255, 255];
const PALE_YELLOW_RGBA: [u8; 4] = [255, 234, 128, 255];
const TRANSPARENT_RGBA: [u8; 4] = [0, 0, 0, 0];
const PICTURE_COLOR_TABLE: [u8; 16] = [
    0x00, 0x00, 0x07, 0x28, 0x2F, 0x81, 0xA4, 0x15, 0xC7, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const BOMBER_COLOR_SEQUENCE: [u8; 9] = [0x81, 0x81, 0x2F, 0x81, 0x2F, 0x07, 0x2F, 0x81, 0x07]; // original: SOURCE_TIE_COLOR_TABLE
const OBJECT_COLOR_SEQUENCE: [u8; 37] = [
    0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x37, 0x2F, 0x27, 0x1F, 0x17, 0x47, 0x47, 0x87,
    0x87, 0xC7, 0xC7, 0xC6, 0xC5, 0xCC, 0xCB, 0xCA, 0xDA, 0xE8, 0xF8, 0xF9, 0xFA, 0xFB, 0xFD, 0xFF,
    0xBF, 0x3F, 0x3E, 0x3C, 0x00,
]; // original: SOURCE_OBJECT_COLOR_TABLE
const WILLIAMS_RED_GREEN_DAC_LEVELS: [u8; 8] = [0, 38, 81, 118, 137, 174, 217, 255]; // original: SOURCE_WILLIAMS_RED_GREEN_LEVELS
const WILLIAMS_BLUE_DAC_LEVELS: [u8; 4] = [0, 95, 160, 255]; // original: SOURCE_WILLIAMS_BLUE_LEVELS
const FONT_SHEET_PNG: &[u8] = include_bytes!("../assets/sprites/font-sheet.png");
const SCORE_DIGIT_PIXEL_SIZE: [u32; 2] = [6, 8]; // original: SOURCE_SCORE_DIGIT_SIZE
const MESSAGE_GLYPH_ATLAS_START: [u32; 2] = [0, 104];
const MESSAGE_GLYPH_ATLAS_ROW_STEP: u32 = 16;
const MESSAGE_GLYPH_ATLAS_GAP: u32 = 2;
const SCREEN_COLUMN_WIDTH_PIXELS_U8: u8 = 2; // original: SOURCE_SCREEN_COLUMN_PIXELS_U8
const SCREEN_COLUMN_WIDTH_PIXELS: f32 = SCREEN_COLUMN_WIDTH_PIXELS_U8 as f32; // original: SOURCE_SCREEN_COLUMN_PIXELS
const MESSAGE_LINE_SPACING_ROWS: u8 = 0x0A; // original: SOURCE_MESSAGE_LINE_SPACING
const ASTRONAUT_EXPLOSION_BYTES: [u8; 32] = [
    0x00, 0x00, 0x0D, 0x6C, 0x6C, 0x0D, 0x00, 0x00, 0x06, 0xE6, 0xC8, 0x83, 0x82, 0xC8, 0xEC, 0x06,
    0x60, 0x6D, 0x8C, 0x28, 0x28, 0x8C, 0x6D, 0x60, 0x00, 0x00, 0xE0, 0xC6, 0xC6, 0xE0, 0x00, 0x00,
];
const NULL_OBJECT_BYTES: [u8; 1] = [0x00];
const TERRAIN_EXPLOSION_BYTES: [u8; 48] = [
    0x1C, 0x0D, 0x7F, 0xE7, 0x70, 0x00, 0x0F, 0x71, 0x71, 0x07, 0xDC, 0x77, 0x7C, 0x0D, 0x71, 0xC7,
    0x77, 0xDE, 0x07, 0x71, 0x17, 0x17, 0xDE, 0xF7, 0x71, 0x17, 0x71, 0x7C, 0xDE, 0xF0, 0x07, 0x77,
    0xC7, 0x71, 0x17, 0x70, 0x70, 0x7C, 0xD7, 0x77, 0x77, 0x70, 0x01, 0xCD, 0xFF, 0xD7, 0x70, 0xF0,
];
const ASTRONAUT_EXPLOSION_GRID: ObjectPictureGrid = ObjectPictureGrid {
    rows: 8,
    bytes_per_row: 4,
    bytes: &ASTRONAUT_EXPLOSION_BYTES,
    palette: ObjectPicturePalette::burst(),
};
const NULL_OBJECT_GRID: ObjectPictureGrid = ObjectPictureGrid {
    rows: 1,
    bytes_per_row: 1,
    bytes: &NULL_OBJECT_BYTES,
    palette: ObjectPicturePalette::white(),
};
const TERRAIN_EXPLOSION_GRID: ObjectPictureGrid = ObjectPictureGrid {
    rows: 6,
    bytes_per_row: 8,
    bytes: &TERRAIN_EXPLOSION_BYTES,
    palette: ObjectPicturePalette::burst(),
};
const HALL_OF_FAME_DEFENDER_LOGO_COLUMNS: u8 = 0x3C; // original: SOURCE_DEFENDER_LOGO_COLUMNS
const HALL_OF_FAME_DEFENDER_LOGO_ROWS: u8 = 0x18; // original: SOURCE_DEFENDER_LOGO_ROWS
const HALL_OF_FAME_DEFENDER_LOGO_BYTE_COUNT: usize =
    HALL_OF_FAME_DEFENDER_LOGO_COLUMNS as usize * HALL_OF_FAME_DEFENDER_LOGO_ROWS as usize; // original: SOURCE_DEFENDER_LOGO_BYTES
pub(crate) const ATTRACT_DEFENDER_WORDMARK_BLOCK_COLUMNS: usize = 15;
pub(crate) const ATTRACT_DEFENDER_WORDMARK_BLOCK_ROWS: usize = 1;
pub(crate) const ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT: usize =
    ATTRACT_DEFENDER_WORDMARK_BLOCK_COLUMNS * ATTRACT_DEFENDER_WORDMARK_BLOCK_ROWS;
const COPYRIGHT_STRIP_COLUMNS: u8 = 40; // original: SOURCE_ATTRACT_COPYRIGHT_COLUMNS
const COPYRIGHT_STRIP_ROWS: u8 = 8; // original: SOURCE_ATTRACT_COPYRIGHT_ROWS
const COPYRIGHT_STRIP_BYTES: [u8; 80] = [
    0x3E, 0x41, 0x41, 0x22, 0x00, 0x3E, 0x41, 0x41, 0x3E, 0x00, 0x7F, 0x09, 0x09, 0x06, 0x00, 0x03,
    0x04, 0x78, 0x04, 0x03, 0x00, 0x7F, 0x09, 0x19, 0x66, 0x00, 0x41, 0x7F, 0x41, 0x00, 0x3E, 0x41,
    0x49, 0x3A, 0x00, 0x7F, 0x08, 0x08, 0x7F, 0x00, 0x01, 0x01, 0x7F, 0x01, 0x01, 0x00, 0x1C, 0x22,
    0x5D, 0x63, 0x55, 0x22, 0x1C, 0x22, 0x7F, 0x4B, 0x45, 0x22, 0x1C, 0x00, 0x00, 0x00, 0x42, 0x7F,
    0x40, 0x00, 0x26, 0x49, 0x49, 0x3E, 0x00, 0x36, 0x49, 0x49, 0x36, 0x00, 0x3E, 0x41, 0x41, 0x3E,
]; // original: SOURCE_ATTRACT_COPYRIGHT_BYTES
const WILLIAMS_LOGO_COLUMNS: u8 = 46; // original: SOURCE_ATTRACT_WILLIAMS_LOGO_COLUMNS
const WILLIAMS_LOGO_ROWS: u8 = 19; // original: SOURCE_ATTRACT_WILLIAMS_LOGO_ROWS
const WILLIAMS_LOGO_PIXEL_COUNT: usize =
    WILLIAMS_LOGO_COLUMNS as usize * 2 * WILLIAMS_LOGO_ROWS as usize; // original: SOURCE_ATTRACT_WILLIAMS_LOGO_PIXELS
const WILLIAMS_LOGO_FIRST_SCREEN_COLUMN: u8 = 0x36; // original: SOURCE_ATTRACT_WILLIAMS_LOGO_FIRST_COLUMN
const WILLIAMS_LOGO_FIRST_SCREEN_ROW: u8 = 0x3C; // original: SOURCE_ATTRACT_WILLIAMS_LOGO_FIRST_ROW
const WILLIAMS_LOGO_DRAW_PROGRAM_BYTES: [u8; 351] = [
    0xFE, 0x74, 0x40, 0x11, 0x11, 0x85, 0x81, 0x81, 0x81, 0x88, 0x82, 0x82, 0x22, 0x24, 0x22, 0x42,
    0x24, 0x24, 0x24, 0x44, 0x24, 0x44, 0x49, 0x44, 0x94, 0x41, 0x88, 0x14, 0x41, 0x88, 0x14, 0x41,
    0x88, 0x94, 0x41, 0x88, 0x94, 0x49, 0x88, 0x14, 0x98, 0x58, 0x94, 0x98, 0x18, 0x94, 0x46, 0x66,
    0x62, 0x42, 0x42, 0x42, 0x42, 0x25, 0x24, 0x24, 0x68, 0x24, 0x24, 0x24, 0x26, 0x11, 0x18, 0x18,
    0x58, 0x18, 0x58, 0x81, 0x44, 0x98, 0x81, 0x44, 0x98, 0x81, 0x44, 0x98, 0x14, 0x94, 0x94, 0x16,
    0x22, 0x24, 0x24, 0xA4, 0x24, 0xA4, 0x24, 0x24, 0x24, 0x24, 0x24, 0xFE, 0x81, 0x4A, 0x42, 0x42,
    0x42, 0x42, 0x44, 0x99, 0x99, 0x41, 0x88, 0x14, 0x41, 0x88, 0x14, 0x46, 0x24, 0x24, 0x24, 0x24,
    0x24, 0x24, 0xA4, 0x24, 0x24, 0xA4, 0x22, 0x42, 0x4A, 0x42, 0x42, 0x44, 0x99, 0x19, 0x91, 0x19,
    0x91, 0x91, 0x81, 0x81, 0x41, 0x81, 0x49, 0x46, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x24, 0x22,
    0x42, 0x62, 0x62, 0x42, 0x24, 0x49, 0x19, 0x91, 0x91, 0x91, 0x91, 0x91, 0x85, 0x88, 0x14, 0x94,
    0x14, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0xA4, 0x24, 0x24, 0x41, 0x81, 0x81, 0x18, 0x18, 0x94,
    0x41, 0x88, 0x14, 0x14, 0x24, 0x42, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x44, 0x98, 0x18,
    0x18, 0x18, 0x58, 0x89, 0x44, 0x18, 0x85, 0x14, 0x24, 0x14, 0x24, 0xA4, 0x24, 0x24, 0x24, 0xA4,
    0x24, 0x28, 0x24, 0x44, 0x18, 0x19, 0x19, 0x81, 0x41, 0x81, 0x14, 0x24, 0x24, 0x24, 0x24, 0x22,
    0x42, 0x42, 0x64, 0x41, 0x85, 0x81, 0x81, 0x18, 0x19, 0x41, 0x89, 0x44, 0x42, 0x22, 0x42, 0x24,
    0x24, 0x24, 0x24, 0x24, 0x44, 0x18, 0x14, 0x98, 0x11, 0x81, 0x81, 0x41, 0x89, 0x44, 0x42, 0x22,
    0x42, 0x24, 0x24, 0x24, 0x24, 0x24, 0x44, 0x18, 0x94, 0x41, 0x88, 0x89, 0x44, 0x49, 0x88, 0x14,
    0x41, 0x88, 0x14, 0x14, 0x24, 0x24, 0x24, 0x26, 0x62, 0x66, 0x26, 0x24, 0x18, 0x91, 0x91, 0x19,
    0x18, 0x14, 0x18, 0x14, 0x14, 0x24, 0x14, 0x2A, 0x45, 0x24, 0x68, 0x88, 0x24, 0x44, 0x42, 0x18,
    0xA8, 0x82, 0x44, 0xA8, 0x22, 0x20, 0xFE, 0x87, 0x40, 0x44, 0x11, 0x88, 0x24, 0xFE, 0x9A, 0x3F,
    0x44, 0x11, 0x88, 0x24, 0xFE, 0xC1, 0x3F, 0x44, 0x44, 0x44, 0x11, 0x11, 0x11, 0x11, 0x88, 0x88,
    0x88, 0x22, 0x22, 0x22, 0x20, 0xFE, 0xC3, 0x45, 0x22, 0x22, 0x44, 0x11, 0x81, 0x50, 0xFD,
]; // original: SOURCE_ATTRACT_WILLIAMS_LOGO_TABLE
const HALL_OF_FAME_DEFENDER_LOGO_COMPRESSED_BYTES: [u8; 366] = [
    0x10, 0xD1, 0xBD, 0x29, 0xC2, 0x9C, 0x29, 0xCB, 0xEA, 0xC2, 0x8C, 0x29, 0xC2, 0x81, 0x0D, 0x10,
    0xC2, 0x8D, 0x29, 0xC2, 0x9C, 0x29, 0xCB, 0xEA, 0x42, 0x94, 0x29, 0x42, 0x81, 0x0C, 0x3F, 0x29,
    0xC2, 0x94, 0xC2, 0x9C, 0x29, 0xC1, 0x8D, 0xA4, 0x29, 0x42, 0x94, 0x29, 0x3F, 0x3E, 0x29, 0x42,
    0xA4, 0x29, 0x4C, 0x29, 0xC1, 0x8D, 0xA4, 0x2A, 0x42, 0x94, 0x29, 0x3E, 0x3D, 0xB6, 0xB4, 0xA2,
    0x4A, 0x17, 0xCA, 0x16, 0xC1, 0x9C, 0xB4, 0xA7, 0xA4, 0xB1, 0x7A, 0x7A, 0x3D, 0x3C, 0xB6, 0xB4,
    0xB1, 0x71, 0x81, 0x6B, 0x16, 0xC1, 0xAC, 0xA4, 0xB6, 0xB4, 0xA2, 0x4A, 0x6B, 0x3C, 0x2F, 0xB6,
    0xB4, 0x29, 0x62, 0x85, 0xC2, 0x85, 0xC1, 0xAC, 0xA4, 0xB6, 0xB4, 0x28, 0x62, 0xA2, 0xF2, 0xEB,
    0x61, 0x84, 0x29, 0x62, 0x8E, 0x28, 0xE2, 0xA4, 0xB7, 0xB4, 0x28, 0x62, 0xA2, 0xE2, 0xDB, 0x7B,
    0x42, 0x96, 0x28, 0x4E, 0x28, 0xE2, 0xB4, 0xB6, 0xB4, 0x29, 0x62, 0x92, 0xE2, 0xCB, 0x7B, 0x52,
    0x96, 0x28, 0x4E, 0x28, 0xEB, 0x41, 0xA4, 0xB7, 0xB4, 0x28, 0x62, 0x92, 0xE1, 0xFB, 0x7B, 0x5B,
    0x24, 0xB1, 0x6D, 0x18, 0x14, 0xEB, 0x51, 0x94, 0xB7, 0xB4, 0x18, 0x17, 0x29, 0x2D, 0x1E, 0xB1,
    0x4B, 0x4B, 0x25, 0xB1, 0x6D, 0xB1, 0x5E, 0xB5, 0x1A, 0x4B, 0x61, 0x84, 0xB2, 0x4B, 0x41, 0x82,
    0xC1, 0xDB, 0x14, 0xB5, 0x18, 0x17, 0x18, 0x16, 0xD1, 0x81, 0x4E, 0xB6, 0x19, 0x4B, 0x61, 0x84,
    0x18, 0x24, 0xB4, 0x18, 0x1F, 0x1C, 0x38, 0x53, 0x84, 0xB1, 0x6E, 0x2B, 0xCB, 0x61, 0x94, 0x38,
    0x42, 0xB4, 0x18, 0x41, 0x81, 0xEF, 0x39, 0x43, 0x85, 0xB1, 0x6E, 0x2B, 0xCB, 0x71, 0x84, 0x38,
    0x43, 0x84, 0xB6, 0x18, 0x1C, 0xE3, 0x95, 0x38, 0x41, 0x81, 0x6D, 0x38, 0xCB, 0xC6, 0x19, 0x42,
    0xB5, 0x38, 0x4B, 0x61, 0x8F, 0xD3, 0x95, 0x38, 0x5B, 0x51, 0xF3, 0x8C, 0xBD, 0x61, 0x84, 0x2A,
    0x63, 0x85, 0xB6, 0x18, 0xED, 0x38, 0x53, 0x94, 0x18, 0x51, 0xF3, 0x8C, 0xBD, 0x7B, 0x42, 0x91,
    0x42, 0xB5, 0x18, 0x7B, 0xDC, 0x21, 0x51, 0xF3, 0x4C, 0x7E, 0x30, 0x6C, 0xC2, 0x14, 0x2C, 0x34,
    0xC7, 0xE1, 0x07, 0xC1, 0x35, 0xCC, 0x21, 0x42, 0xC3, 0x4C, 0x7F, 0x10, 0x6C, 0x13, 0x5C, 0xC1,
    0x35, 0xC1, 0x52, 0xC3, 0x4C, 0x7F, 0x15, 0xC2, 0x7D, 0x34, 0xC1, 0x5C, 0x17, 0xCC, 0x36, 0xC3,
    0x5C, 0x14, 0x2D, 0x34, 0xC7, 0x1C, 0x14, 0xC2, 0x6E, 0x34, 0xD1, 0x4E, 0x15, 0xCC, 0x36, 0xC3,
    0x5C, 0x14, 0x2D, 0x34, 0xC7, 0x1C, 0x14, 0xC2, 0x51, 0xC2, 0x7D, 0x14, 0xF1, 0x4C,
]; // original: SOURCE_DEFENDER_LOGO_COMPRESSED

impl TextureAtlas {
    pub fn new(surface: SurfaceSize, regions: Vec<AtlasRegion>) -> Self {
        let pixels = transparent_rgba_pixels(surface).unwrap_or_default();
        Self {
            surface,
            regions,
            pixels,
        }
    }

    pub fn with_rgba(
        surface: SurfaceSize,
        regions: Vec<AtlasRegion>,
        pixels: Vec<u8>,
    ) -> Result<Self, SceneRasterError> {
        let Some(expected) = surface.rgba_len() else {
            return Err(SceneRasterError::PixelBufferTooLarge { surface });
        };
        if pixels.len() != expected {
            return Err(SceneRasterError::PixelBufferLength {
                expected,
                actual: pixels.len(),
            });
        }

        Ok(Self {
            surface,
            regions,
            pixels,
        })
    }

    pub fn default_sprites() -> Self {
        let surface = SurfaceSize::new(128, 192);
        let mut regions = vec![
            AtlasRegion {
                sprite: SpriteId::PLAYER_SHIP,
                origin: [0, 0],
                size: [16, 6],
            },
            AtlasRegion {
                sprite: SpriteId::PLAYER_SHIP_LEFT,
                origin: [16, 0],
                size: [16, 6],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_TEXT,
                origin: [0, 16],
                size: [80, 8],
            },
            AtlasRegion {
                sprite: SpriteId::STATUS_TEXT,
                origin: [0, 32],
                size: [96, 8],
            },
            AtlasRegion {
                sprite: SpriteId::PLAYER_PROJECTILE,
                origin: [0, 48],
                size: [16, 1],
            },
            AtlasRegion {
                sprite: SpriteId::TERRAIN_TILE,
                origin: [0, 64],
                size: [2, 2],
            },
            AtlasRegion {
                sprite: SpriteId::TERRAIN_TILE_ALT,
                origin: [2, 64],
                size: [2, 2],
            },
            AtlasRegion {
                sprite: SpriteId::STAR,
                origin: [4, 64],
                size: [1, 1],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_LANDER,
                origin: [24, 64],
                size: [10, 8],
            },
            AtlasRegion {
                sprite: SpriteId::HUMAN,
                origin: [40, 64],
                size: [4, 8],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_MUTANT,
                origin: [56, 64],
                size: [10, 8],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_BAITER,
                origin: [72, 64],
                size: [12, 4],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_BOMBER,
                origin: [88, 64],
                size: [8, 8],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_POD,
                origin: [104, 64],
                size: [8, 8],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_SWARMER,
                origin: [116, 64],
                size: [6, 4],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_BOMB,
                origin: [0, 80],
                size: [4, 3],
            },
            AtlasRegion {
                sprite: SpriteId::BOMB_EXPLOSION,
                origin: [8, 80],
                size: [8, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SWARMER_EXPLOSION,
                origin: [24, 80],
                size: [8, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_POPUP_250,
                origin: [40, 80],
                size: [12, 6],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_POPUP_500,
                origin: [56, 80],
                size: [12, 6],
            },
            AtlasRegion {
                sprite: SpriteId::PLAYER_LIFE_STOCK,
                origin: [72, 80],
                size: [10, 4],
            },
            AtlasRegion {
                sprite: SpriteId::SMART_BOMB_STOCK,
                origin: [88, 80],
                size: [6, 3],
            },
            AtlasRegion {
                sprite: SpriteId::ASTRONAUT_EXPLOSION,
                origin: [0, 96],
                size: [8, 8],
            },
            AtlasRegion {
                sprite: SpriteId::NULL_OBJECT,
                origin: [20, 96],
                size: [2, 1],
            },
            AtlasRegion {
                sprite: SpriteId::TERRAIN_EXPLOSION,
                origin: [24, 96],
                size: [16, 6],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_DIGIT_0,
                origin: [0, 112],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_DIGIT_1,
                origin: [8, 112],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_DIGIT_2,
                origin: [16, 112],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_DIGIT_3,
                origin: [24, 112],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_DIGIT_4,
                origin: [32, 112],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_DIGIT_5,
                origin: [40, 112],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_DIGIT_6,
                origin: [48, 112],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_DIGIT_7,
                origin: [56, 112],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_DIGIT_8,
                origin: [64, 112],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_DIGIT_9,
                origin: [72, 112],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::HALL_OF_FAME_UNDERLINE_WORD,
                origin: [80, 112],
                size: [2, 2],
            },
            AtlasRegion {
                sprite: SpriteId::HALL_OF_FAME_DEFENDER_LOGO,
                origin: [0, 128],
                size: [120, 24],
            },
            AtlasRegion {
                sprite: SpriteId::ATTRACT_COPYRIGHT_STRIP,
                origin: [0, 152],
                size: [80, 8],
            },
            AtlasRegion {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO,
                origin: [0, 160],
                size: [92, 19],
            },
            AtlasRegion {
                sprite: SpriteId::TOP_DISPLAY_BORDER_WORD,
                origin: [96, 160],
                size: [2, 2],
            },
            AtlasRegion {
                sprite: SpriteId::SCANNER_OBJECT_BLIP,
                origin: [100, 160],
                size: [2, 2],
            },
            AtlasRegion {
                sprite: SpriteId::SCANNER_PLAYER_BLIP,
                origin: [104, 160],
                size: [3, 2],
            },
            AtlasRegion {
                sprite: SpriteId::PLAYER_EXPLOSION_PIXEL,
                origin: [108, 160],
                size: [4, 2],
            },
            AtlasRegion {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
                origin: [112, 160],
                size: [1, 1],
            },
            AtlasRegion {
                sprite: SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL,
                origin: [114, 160],
                size: [1, 1],
            },
        ];
        regions.extend(attract_defender_wordmark_block_regions());
        regions.extend(message_glyph_atlas_regions(surface));
        let pixels = default_sprite_atlas_pixels(surface, &regions);

        Self {
            surface,
            regions,
            pixels,
        }
    }

    pub fn contains(&self, sprite: SpriteId) -> bool {
        self.region(sprite).is_some()
    }

    pub fn region(&self, sprite: SpriteId) -> Option<AtlasRegion> {
        self.regions
            .iter()
            .copied()
            .find(|region| region.sprite == sprite)
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn is_non_blank(&self) -> bool {
        self.pixels
            .chunks_exact(4)
            .any(|pixel| pixel != [0, 0, 0, 0].as_slice())
    }
}

fn transparent_rgba_pixels(surface: SurfaceSize) -> Option<Vec<u8>> {
    surface.rgba_len().map(|len| vec![0; len])
}

fn message_glyph_atlas_regions(surface: SurfaceSize) -> Vec<AtlasRegion> {
    let mut regions = Vec::with_capacity(SpriteId::MESSAGE_GLYPHS.len());
    let mut origin = MESSAGE_GLYPH_ATLAS_START;
    for (_, sprite, size) in SpriteId::MESSAGE_GLYPH_SPECS {
        if origin[0] != MESSAGE_GLYPH_ATLAS_START[0] && origin[0] + size[0] > surface.width {
            origin[0] = MESSAGE_GLYPH_ATLAS_START[0];
            origin[1] += MESSAGE_GLYPH_ATLAS_ROW_STEP;
        }
        assert!(
            origin[0] + size[0] <= surface.width && origin[1] + size[1] <= surface.height,
            "message glyph atlas regions must fit in the default atlas"
        );
        regions.push(AtlasRegion {
            sprite,
            origin,
            size,
        });
        origin[0] += size[0] + MESSAGE_GLYPH_ATLAS_GAP;
    }
    regions
}

fn attract_defender_wordmark_block_regions() -> Vec<AtlasRegion> {
    let mut regions = Vec::with_capacity(ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT);
    for index in 0..ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT {
        let column = index % ATTRACT_DEFENDER_WORDMARK_BLOCK_COLUMNS;
        let row = index / ATTRACT_DEFENDER_WORDMARK_BLOCK_COLUMNS;
        regions.push(AtlasRegion {
            sprite: SpriteId::attract_defender_wordmark_block(index)
                .expect("Defender wordmark block sprite"),
            origin: [
                u32::try_from(column).expect("Defender wordmark block column fits") * 8,
                128 + u32::try_from(row).expect("Defender wordmark block row fits") * 24,
            ],
            size: [8, 24],
        });
    }
    regions
}

fn default_sprite_atlas_pixels(surface: SurfaceSize, regions: &[AtlasRegion]) -> Vec<u8> {
    let mut pixels = transparent_rgba_pixels(surface).unwrap_or_default();

    let player_ship = decode_object_picture_asset_rgba(
        ObjectBitmapId::PlayerShipRightPrimary,
        6,
        8,
        ObjectPicturePalette::ship(),
    );
    let player_ship_left = decode_object_picture_asset_rgba(
        ObjectBitmapId::PlayerShipLeftPrimary,
        6,
        8,
        ObjectPicturePalette::ship(),
    );
    let player_projectile = decode_object_picture_asset_rgba(
        ObjectBitmapId::PlayerLaser,
        1,
        8,
        ObjectPicturePalette::player_shot(),
    );
    let enemy_lander = decode_object_picture_asset_rgba(
        ObjectBitmapId::LanderFrame1Primary,
        8,
        5,
        ObjectPicturePalette::white(),
    );
    let human = decode_object_picture_asset_rgba(
        ObjectBitmapId::HumanStandingPrimary,
        8,
        2,
        ObjectPicturePalette::white(),
    );
    let enemy_mutant = decode_object_picture_asset_rgba(
        ObjectBitmapId::MutantPrimary,
        8,
        5,
        ObjectPicturePalette::white(),
    );
    let enemy_baiter = decode_object_picture_asset_rgba(
        ObjectBitmapId::BaiterFrame1Primary,
        4,
        6,
        ObjectPicturePalette::white(),
    );
    let enemy_bomber = decode_object_picture_asset_rgba(
        ObjectBitmapId::BomberFrame1Primary,
        8,
        4,
        ObjectPicturePalette::tie(0),
    );
    let enemy_pod = decode_object_picture_asset_rgba(
        ObjectBitmapId::PodPrimary,
        8,
        4,
        ObjectPicturePalette::white(),
    );
    let enemy_swarmer = decode_object_picture_asset_rgba(
        ObjectBitmapId::SwarmerPrimary,
        4,
        3,
        ObjectPicturePalette::white(),
    );
    let enemy_bomb = decode_object_picture_asset_rgba(
        ObjectBitmapId::EnemyBombFrame1Primary,
        3,
        2,
        ObjectPicturePalette::bomb(0),
    );
    let bomb_explosion = decode_object_picture_asset_rgba(
        ObjectBitmapId::BombExplosion,
        8,
        4,
        ObjectPicturePalette::burst(),
    );
    let swarmer_explosion = decode_object_picture_asset_rgba(
        ObjectBitmapId::SwarmerExplosion,
        8,
        4,
        ObjectPicturePalette::burst(),
    );
    let score_popup_250 = decode_object_picture_asset_rgba(
        ObjectBitmapId::Score250Primary,
        6,
        6,
        ObjectPicturePalette::score_250(0),
    );
    let score_popup_500 = decode_object_picture_asset_rgba(
        ObjectBitmapId::Score500Primary,
        6,
        6,
        ObjectPicturePalette::score_500(0),
    );
    let player_life_stock = decode_object_picture_asset_rgba(
        ObjectBitmapId::PlayerLifeStock,
        4,
        5,
        ObjectPicturePalette::ship(),
    );
    let smart_bomb_stock = decode_object_picture_asset_rgba(
        ObjectBitmapId::SmartBombStock,
        3,
        3,
        ObjectPicturePalette::white(),
    );
    let astronaut_explosion = decode_picture_grid_rgba("ASXP1", ASTRONAUT_EXPLOSION_GRID);
    let null_object = decode_picture_grid_rgba("NULOB", NULL_OBJECT_GRID);
    let terrain_explosion = decode_picture_grid_rgba("TEREX", TERRAIN_EXPLOSION_GRID);
    let score_digits = decode_score_digit_sprites();
    let message_glyphs = decode_message_glyph_sprites();
    let hall_of_fame_logo = decode_source_defender_logo_rgba();
    let attract_copyright_strip = decode_source_attract_copyright_strip_rgba();
    let attract_williams_logo = decode_source_attract_williams_logo_rgba();
    let terrain_word_7007 = decode_source_terrain_word_rgba(0x7007);
    let terrain_word_0770 = decode_source_terrain_word_rgba(0x0770);
    let font_sheet = decode_embedded_png_rgba("font-sheet.png", FONT_SHEET_PNG);

    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::PLAYER_SHIP,
        &player_ship,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::PLAYER_SHIP_LEFT,
        &player_ship_left,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::PLAYER_PROJECTILE,
        &player_projectile,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ENEMY_LANDER,
        &enemy_lander,
    );
    blit_default_region(&mut pixels, surface, regions, SpriteId::HUMAN, &human);
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ENEMY_MUTANT,
        &enemy_mutant,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ENEMY_BAITER,
        &enemy_baiter,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ENEMY_BOMBER,
        &enemy_bomber,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ENEMY_POD,
        &enemy_pod,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ENEMY_SWARMER,
        &enemy_swarmer,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ENEMY_BOMB,
        &enemy_bomb,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::BOMB_EXPLOSION,
        &bomb_explosion,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::SWARMER_EXPLOSION,
        &swarmer_explosion,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::SCORE_POPUP_250,
        &score_popup_250,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::SCORE_POPUP_500,
        &score_popup_500,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::PLAYER_LIFE_STOCK,
        &player_life_stock,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::SMART_BOMB_STOCK,
        &smart_bomb_stock,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ASTRONAUT_EXPLOSION,
        &astronaut_explosion,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::NULL_OBJECT,
        &null_object,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::TERRAIN_EXPLOSION,
        &terrain_explosion,
    );
    for (digit, sprite) in score_digits.iter().enumerate() {
        blit_default_region(
            &mut pixels,
            surface,
            regions,
            SpriteId::score_digit(u8::try_from(digit).expect("score digit index fits"))
                .expect("score digit sprite"),
            sprite,
        );
    }
    fill_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::HALL_OF_FAME_UNDERLINE_WORD,
        WHITE_RGBA,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::HALL_OF_FAME_DEFENDER_LOGO,
        &hall_of_fame_logo,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ATTRACT_COPYRIGHT_STRIP,
        &attract_copyright_strip,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ATTRACT_WILLIAMS_LOGO,
        &attract_williams_logo,
    );
    fill_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::TOP_DISPLAY_BORDER_WORD,
        WHITE_RGBA,
    );
    fill_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::SCANNER_OBJECT_BLIP,
        WHITE_RGBA,
    );
    fill_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::SCANNER_PLAYER_BLIP,
        WHITE_RGBA,
    );
    fill_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::PLAYER_EXPLOSION_PIXEL,
        WHITE_RGBA,
    );
    fill_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
        WHITE_RGBA,
    );
    fill_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL,
        WHITE_RGBA,
    );
    for (character, sprite) in &message_glyphs {
        if let Some(sprite_id) = SpriteId::message_glyph(*character) {
            blit_default_region(&mut pixels, surface, regions, sprite_id, sprite);
        }
    }
    blit_default_region_from_source(
        &mut pixels,
        surface,
        regions,
        SpriteId::SCORE_TEXT,
        &font_sheet,
        SpriteAtlasSourceRegion {
            origin: [0, 0],
            size: [64, 8],
        },
    );
    blit_default_region_from_source(
        &mut pixels,
        surface,
        regions,
        SpriteId::STATUS_TEXT,
        &font_sheet,
        SpriteAtlasSourceRegion {
            origin: [0, 8],
            size: [64, 8],
        },
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::TERRAIN_TILE,
        &terrain_word_7007,
    );
    blit_default_region(
        &mut pixels,
        surface,
        regions,
        SpriteId::TERRAIN_TILE_ALT,
        &terrain_word_0770,
    );
    blit_star_region(&mut pixels, surface, regions, &font_sheet);

    pixels
}

fn decode_source_terrain_word_rgba(word: u16) -> EmbeddedSprite {
    const WIDTH: u32 = 2;
    const HEIGHT: u32 = 2;
    let mut pixels = vec![0; (WIDTH * HEIGHT * 4) as usize];

    for (row, byte) in word.to_be_bytes().into_iter().enumerate() {
        for column in 0..2 {
            let nibble = if column == 0 { byte >> 4 } else { byte & 0x0F };
            let color = picture_palette_color(nibble, ObjectPicturePalette::white());
            let start = (row * WIDTH as usize + column) * 4;
            pixels[start..start + 4].copy_from_slice(&color);
        }
    }

    EmbeddedSprite {
        surface: SurfaceSize::new(WIDTH, HEIGHT),
        pixels,
    }
}

fn decode_embedded_png_rgba(name: &'static str, bytes: &[u8]) -> EmbeddedSprite {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder
        .read_info()
        .unwrap_or_else(|error| panic!("embedded sprite asset {name} must decode: {error}"));
    let mut pixels = vec![
        0;
        reader.output_buffer_size().unwrap_or_else(|| panic!(
            "embedded sprite asset {name} output buffer must fit"
        ))
    ];
    let info = reader.next_frame(&mut pixels).unwrap_or_else(|error| {
        panic!("embedded sprite asset {name} must contain a frame: {error}")
    });

    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        panic!(
            "embedded sprite asset {name} must be 8-bit RGBA, got {:?} {:?}",
            info.color_type, info.bit_depth
        );
    }

    pixels.truncate(info.buffer_size());
    EmbeddedSprite {
        surface: SurfaceSize::new(info.width, info.height),
        pixels,
    }
}

fn decode_picture_grid_rgba(name: &'static str, grid: ObjectPictureGrid) -> EmbeddedSprite {
    decode_picture_bytes_rgba(
        name,
        grid.rows,
        grid.bytes_per_row,
        grid.bytes,
        grid.palette,
    )
}

fn decode_object_picture_asset_rgba(
    bitmap: ObjectBitmapId,
    rows: u8,
    bytes_per_row: u8,
    palette: ObjectPicturePalette,
) -> EmbeddedSprite {
    let bytes = object_bitmap_bytes(bitmap);
    decode_picture_bytes_rgba("object bitmap", rows, bytes_per_row, bytes, palette)
}

fn decode_picture_bytes_rgba(
    name: &'static str,
    rows: u8,
    bytes_per_row: u8,
    bytes: &[u8],
    palette: ObjectPicturePalette,
) -> EmbeddedSprite {
    let expected = usize::from(rows) * usize::from(bytes_per_row);
    assert_eq!(
        bytes.len(),
        expected,
        "object picture {name} byte grid must match its declared dimensions"
    );
    let surface = SurfaceSize::new(u32::from(bytes_per_row) * 2, u32::from(rows));
    let mut pixels = transparent_rgba_pixels(surface).unwrap_or_default();

    for column in 0..usize::from(bytes_per_row) {
        let source_column = column * usize::from(rows);
        for row in 0..usize::from(rows) {
            let value = bytes[source_column + row];
            let left = picture_palette_color(value >> 4, palette);
            let right = picture_palette_color(value & 0x0F, palette);
            let offset = ((row * surface.width as usize) + column * 2) * 4;
            pixels[offset..offset + 4].copy_from_slice(&left);
            pixels[offset + 4..offset + 8].copy_from_slice(&right);
        }
    }

    EmbeddedSprite { surface, pixels }
}

fn decode_score_digit_sprites() -> Vec<EmbeddedSprite> {
    let mut rows = SCORE_DIGIT_ASSETS
        .iter()
        .map(|asset| {
            (
                asset.digit,
                decode_score_digit_rgba(
                    "score digit",
                    asset.digit,
                    asset.columns,
                    asset.rows,
                    asset.bytes,
                ),
            )
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|(digit, _)| *digit);
    assert_eq!(rows.len(), 10, "embedded score digits must define ten sprites");
    for (expected, (digit, _)) in rows.iter().enumerate() {
        assert_eq!(
            usize::from(*digit),
            expected,
            "embedded score digit rows must cover 0 through 9 once"
        );
    }
    rows.into_iter().map(|(_, sprite)| sprite).collect()
}

fn decode_score_digit_rgba(
    name: &'static str,
    digit: u8,
    columns: u8,
    rows: u8,
    bytes: &[u8],
) -> EmbeddedSprite {
    let expected = usize::from(columns) * usize::from(rows);
    assert_eq!(
        bytes.len(),
        expected,
        "{name} digit {digit} byte grid must match its declared dimensions"
    );
    let surface = SurfaceSize::new(u32::from(columns) * 2, u32::from(rows));
    let mut pixels = transparent_rgba_pixels(surface).unwrap_or_default();
    let palette = ObjectPicturePalette::white();

    for column in 0..usize::from(columns) {
        let source_column = column * usize::from(rows);
        for row in 0..usize::from(rows) {
            let value = bytes[source_column + row];
            let left = picture_palette_color(value >> 4, palette);
            let right = picture_palette_color(value & 0x0F, palette);
            let offset = ((row * surface.width as usize) + column * 2) * 4;
            pixels[offset..offset + 4].copy_from_slice(&left);
            pixels[offset + 4..offset + 8].copy_from_slice(&right);
        }
    }

    EmbeddedSprite { surface, pixels }
}

fn decode_message_glyph_sprites() -> Vec<(char, EmbeddedSprite)> {
    let rows = MESSAGE_GLYPH_ASSETS
        .iter()
        .map(|asset| {
            (
                asset.character,
                decode_message_glyph_rgba(
                    "message glyph",
                    asset.character,
                    asset.columns,
                    asset.rows,
                    asset.bytes,
                ),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        SpriteId::MESSAGE_GLYPHS.len(),
        "embedded message glyphs must define the clean renderer glyph set"
    );
    for (character, _) in &rows {
        assert!(
            SpriteId::message_glyph(*character).is_some(),
            "embedded message glyph `{character}` must have a clean sprite mapping"
        );
    }
    rows
}

fn decode_message_glyph_rgba(
    name: &'static str,
    character: char,
    columns: u8,
    rows: u8,
    bytes: &[u8],
) -> EmbeddedSprite {
    let expected = usize::from(columns) * usize::from(rows);
    assert_eq!(
        bytes.len(),
        expected,
        "{name} glyph `{character}` byte grid must match its declared dimensions"
    );
    let surface = SurfaceSize::new(u32::from(columns) * 2, u32::from(rows));
    let mut pixels = transparent_rgba_pixels(surface).unwrap_or_default();
    let palette = ObjectPicturePalette::white();

    for column in 0..usize::from(columns) {
        let source_column = column * usize::from(rows);
        for row in 0..usize::from(rows) {
            let value = bytes[source_column + row];
            let left = picture_palette_color(value >> 4, palette);
            let right = picture_palette_color(value & 0x0F, palette);
            let offset = ((row * surface.width as usize) + column * 2) * 4;
            pixels[offset..offset + 4].copy_from_slice(&left);
            pixels[offset + 4..offset + 8].copy_from_slice(&right);
        }
    }

    EmbeddedSprite { surface, pixels }
}

fn picture_palette_color(index: u8, palette: ObjectPicturePalette) -> [u8; 4] {
    match index {
        0x0 => TRANSPARENT_RGBA,
        0x1 => palette.one,
        0x2..=0x9 => pseudo_color_rgba(PICTURE_COLOR_TABLE[index as usize]),
        0xA => palette.a,
        0xB => GRAY_RGBA,
        0xC => palette.c,
        0xD => palette.d,
        0xE => palette.e,
        0xF => palette.f,
        _ => unreachable!(),
    }
}

fn decode_source_defender_logo_rgba() -> EmbeddedSprite {
    let bytes = expand_source_defender_logo_bytes();
    let surface = SurfaceSize::new(
        u32::from(HALL_OF_FAME_DEFENDER_LOGO_COLUMNS) * 2,
        u32::from(HALL_OF_FAME_DEFENDER_LOGO_ROWS),
    );
    let mut pixels = transparent_rgba_pixels(surface).unwrap_or_default();
    let palette = ObjectPicturePalette::defender_logo();

    for column in 0..usize::from(HALL_OF_FAME_DEFENDER_LOGO_COLUMNS) {
        let source_column = column * usize::from(HALL_OF_FAME_DEFENDER_LOGO_ROWS);
        for row in 0..usize::from(HALL_OF_FAME_DEFENDER_LOGO_ROWS) {
            let value = bytes[source_column + row];
            let left = picture_palette_color(value >> 4, palette);
            let right = picture_palette_color(value & 0x0F, palette);
            let offset = ((row * surface.width as usize) + column * 2) * 4;
            pixels[offset..offset + 4].copy_from_slice(&left);
            pixels[offset + 4..offset + 8].copy_from_slice(&right);
        }
    }

    EmbeddedSprite { surface, pixels }
}

fn decode_source_attract_copyright_strip_rgba() -> EmbeddedSprite {
    let surface = SurfaceSize::new(
        u32::from(COPYRIGHT_STRIP_COLUMNS) * 2,
        u32::from(COPYRIGHT_STRIP_ROWS),
    );
    let mut pixels = transparent_rgba_pixels(surface).unwrap_or_default();

    for column in 0..usize::from(COPYRIGHT_STRIP_COLUMNS) {
        let left_bits = COPYRIGHT_STRIP_BYTES[column * 2];
        let right_bits = COPYRIGHT_STRIP_BYTES[column * 2 + 1];
        for row in 0..usize::from(COPYRIGHT_STRIP_ROWS) {
            let mask = 1u8 << row;
            let offset = ((row * surface.width as usize) + column * 2) * 4;
            if left_bits & mask != 0 {
                pixels[offset..offset + 4].copy_from_slice(&WHITE_RGBA);
            }
            if right_bits & mask != 0 {
                pixels[offset + 4..offset + 8].copy_from_slice(&WHITE_RGBA);
            }
        }
    }

    EmbeddedSprite { surface, pixels }
}

fn decode_source_attract_williams_logo_rgba() -> EmbeddedSprite {
    let surface = SurfaceSize::new(
        u32::from(WILLIAMS_LOGO_COLUMNS) * 2,
        u32::from(WILLIAMS_LOGO_ROWS),
    );
    let mut pixels = transparent_rgba_pixels(surface).unwrap_or_default();

    for [pixel_x, pixel_y] in source_attract_williams_logo_pixel_path() {
        let offset = ((usize::from(pixel_y) * surface.width as usize) + usize::from(pixel_x)) * 4;
        pixels[offset..offset + 4].copy_from_slice(&WHITE_RGBA);
    }

    EmbeddedSprite { surface, pixels }
}

pub(crate) fn source_attract_williams_logo_pixel_path() -> Vec<[u8; 2]> {
    source_attract_williams_logo_walk().pixels
}

pub(crate) fn source_attract_williams_logo_operation_pixel_counts() -> Vec<usize> {
    source_attract_williams_logo_walk().operation_pixel_counts
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceAttractWilliamsLogoWalk {
    pixels: Vec<[u8; 2]>,
    operation_pixel_counts: Vec<usize>,
}

fn source_attract_williams_logo_walk() -> SourceAttractWilliamsLogoWalk {
    let mut path = Vec::new();
    let mut operation_pixel_counts = Vec::new();
    let mut seen = [false; WILLIAMS_LOGO_PIXEL_COUNT];
    let mut pointer = 0usize;
    let mut cursor = 0u16;

    while let Some(opcode) = WILLIAMS_LOGO_DRAW_PROGRAM_BYTES.get(pointer).copied() {
        pointer += 1;
        if opcode > 0xAA {
            let complemented = !opcode;
            if complemented == 0 {
                continue;
            }
            if complemented.wrapping_sub(1) != 0 {
                break;
            }
            let Some(cursor_high) = WILLIAMS_LOGO_DRAW_PROGRAM_BYTES.get(pointer).copied() else {
                break;
            };
            let Some(cursor_low) = WILLIAMS_LOGO_DRAW_PROGRAM_BYTES.get(pointer + 1).copied()
            else {
                break;
            };
            pointer += 2;
            cursor = u16::from_be_bytes([cursor_high, cursor_low]);
            push_source_attract_williams_logo_pixel(&mut path, &mut seen, cursor);
            operation_pixel_counts.push(path.len());
        } else {
            let mut accumulator = opcode;
            loop {
                accumulator =
                    source_attract_williams_logo_horizontal_bit(accumulator, &mut cursor, true);
                accumulator =
                    source_attract_williams_logo_horizontal_bit(accumulator, &mut cursor, false);
                accumulator =
                    source_attract_williams_logo_vertical_bit(accumulator, &mut cursor, true);
                accumulator =
                    source_attract_williams_logo_vertical_bit(accumulator, &mut cursor, false);
                push_source_attract_williams_logo_pixel(&mut path, &mut seen, cursor);
                if accumulator == 0 {
                    break;
                }
            }
            operation_pixel_counts.push(path.len());
        }
    }

    SourceAttractWilliamsLogoWalk {
        pixels: path,
        operation_pixel_counts,
    }
}

fn source_attract_williams_logo_horizontal_bit(
    mut accumulator: u8,
    cursor: &mut u16,
    decrement: bool,
) -> u8 {
    let carry = accumulator & 0x80 != 0;
    accumulator = accumulator.wrapping_shl(1);
    if carry {
        let [x, y] = cursor.to_be_bytes();
        let x = if decrement {
            x.wrapping_sub(1)
        } else {
            x.wrapping_add(1)
        };
        *cursor = u16::from_be_bytes([x, y]);
    }
    accumulator
}

fn source_attract_williams_logo_vertical_bit(
    mut accumulator: u8,
    cursor: &mut u16,
    decrement: bool,
) -> u8 {
    let carry = accumulator & 0x80 != 0;
    accumulator = accumulator.wrapping_shl(1);
    if carry {
        let [x, y] = cursor.to_be_bytes();
        let y = if decrement {
            y.wrapping_sub(1)
        } else {
            y.wrapping_add(1)
        };
        *cursor = u16::from_be_bytes([x, y]);
    }
    accumulator
}

fn push_source_attract_williams_logo_pixel(
    path: &mut Vec<[u8; 2]>,
    seen: &mut [bool],
    cursor: u16,
) {
    let Some([pixel_x, pixel_y]) = source_attract_williams_logo_pixel(cursor) else {
        return;
    };
    let index =
        usize::from(pixel_y) * usize::from(WILLIAMS_LOGO_COLUMNS) * 2 + usize::from(pixel_x);
    if !seen[index] {
        seen[index] = true;
        path.push([pixel_x, pixel_y]);
    }
}

fn source_attract_williams_logo_pixel(cursor: u16) -> Option<[u8; 2]> {
    let [x, y] = cursor.to_be_bytes();
    let column = x >> 1;
    let relative_column = column.checked_sub(WILLIAMS_LOGO_FIRST_SCREEN_COLUMN)?;
    let relative_row = y.checked_sub(WILLIAMS_LOGO_FIRST_SCREEN_ROW)?;
    if relative_column >= WILLIAMS_LOGO_COLUMNS || relative_row >= WILLIAMS_LOGO_ROWS {
        return None;
    }

    let pixel_x = usize::from(relative_column) * 2 + usize::from(x & 1);
    Some([
        u8::try_from(pixel_x).expect("Williams logo pixel x fits in u8"),
        relative_row,
    ])
}

fn expand_source_defender_logo_bytes() -> [u8; HALL_OF_FAME_DEFENDER_LOGO_BYTE_COUNT] {
    let mut output = [0; HALL_OF_FAME_DEFENDER_LOGO_BYTE_COUNT];
    let mut cursor = 0usize;
    let mut length = 0u8;
    let mut right_pixel_next = false;

    for byte in HALL_OF_FAME_DEFENDER_LOGO_COMPRESSED_BYTES {
        for nibble in [byte >> 4, byte & 0x0F] {
            if nibble & 0x0C == 0 {
                length = nibble.wrapping_add(length).wrapping_shl(2);
                continue;
            }

            length = (nibble & 0x03).wrapping_add(length);
            let color = source_defender_logo_color_byte(nibble);
            if cursor >= HALL_OF_FAME_DEFENDER_LOGO_BYTE_COUNT {
                cursor = cursor + 1 - HALL_OF_FAME_DEFENDER_LOGO_BYTE_COUNT;
            }

            if right_pixel_next {
                output[cursor] = (output[cursor] & 0xF0) | (color & 0x0F);
                cursor += usize::from(HALL_OF_FAME_DEFENDER_LOGO_ROWS);
                length = length.wrapping_sub(1);
                if length & 0x80 != 0 {
                    right_pixel_next = false;
                    length = 0;
                    continue;
                }
            } else {
                right_pixel_next = true;
            }

            loop {
                output[cursor] = color;
                length = length.wrapping_sub(1);
                if length & 0x80 != 0 {
                    break;
                }

                cursor += usize::from(HALL_OF_FAME_DEFENDER_LOGO_ROWS);
                length = length.wrapping_sub(1);
                if length & 0x80 != 0 {
                    right_pixel_next = false;
                    break;
                }
            }
            length = 0;
        }
    }

    output
}

pub(crate) fn source_attract_defender_appearance_pixels(
    surface: SurfaceSize,
    appearance_tick: u8,
) -> Vec<SourceAttractDefenderAppearancePixel> {
    const LOGO_LEFT_BYTE: i32 = 0x30;
    const LOGO_TOP_SCANLINE: i32 = 0x90;
    const CHUNK_WIDTH_BYTES: i32 = 4;
    const CHUNK_WIDTH_PIXELS: i32 = 8;
    const CHUNK_ROW_PAIRS: i32 = 12;
    const CHUNK_CENTER_X_BYTES: i32 = 2;
    const CHUNK_CENTER_Y_SCANLINES: i32 = 12;
    const DEFENDER_WORDMARK_FINAL_APPEARANCE_TICK: u8 = 0x2E;

    let source = expand_source_defender_logo_bytes();
    let mut pixels = BTreeMap::new();
    let size =
        i32::from(DEFENDER_WORDMARK_FINAL_APPEARANCE_TICK.saturating_sub(appearance_tick)).max(1);
    let row_pair_step = size * 2;

    for chunk_index in 0..ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT {
        let chunk_index = i32::try_from(chunk_index).expect("Defender chunk index fits in i32");
        let logo_left_byte = LOGO_LEFT_BYTE + chunk_index * CHUNK_WIDTH_BYTES;
        let start_x_byte = logo_left_byte + CHUNK_CENTER_X_BYTES - CHUNK_CENTER_X_BYTES * size;
        let start_y =
            LOGO_TOP_SCANLINE + CHUNK_CENTER_Y_SCANLINES - CHUNK_CENTER_Y_SCANLINES * size;

        for byte_column in 0..CHUNK_WIDTH_BYTES {
            let target_x_byte = start_x_byte + byte_column * size;
            let source_x = chunk_index * CHUNK_WIDTH_PIXELS + byte_column * 2;

            for row_pair in 0..CHUNK_ROW_PAIRS {
                let target_y = start_y + row_pair * row_pair_step;
                let source_y = row_pair * 2;
                draw_source_defender_logo_word(
                    &mut pixels,
                    surface,
                    &source,
                    source_x,
                    source_y,
                    target_x_byte,
                    target_y,
                );
            }
        }

        clear_source_defender_logo_word(
            &mut pixels,
            logo_left_byte + CHUNK_CENTER_X_BYTES,
            LOGO_TOP_SCANLINE + CHUNK_CENTER_Y_SCANLINES,
        );
    }

    pixels
        .into_iter()
        .map(
            |([native_x, native_y], color)| SourceAttractDefenderAppearancePixel {
                position: [
                    u16::try_from(native_x).expect("Defender appearance x fits in u16"),
                    u16::try_from(native_y).expect("Defender appearance y fits in u16"),
                ],
                color,
            },
        )
        .collect()
}

fn draw_source_defender_logo_word(
    pixels: &mut BTreeMap<[u32; 2], [u8; 4]>,
    surface: SurfaceSize,
    source: &[u8; HALL_OF_FAME_DEFENDER_LOGO_BYTE_COUNT],
    source_x: i32,
    source_y: i32,
    target_x_byte: i32,
    target_y: i32,
) {
    let palette = ObjectPicturePalette::defender_logo();
    for dy in 0..2 {
        for dx in 0..2 {
            let Some(nibble) = source_defender_logo_nibble(source, source_x + dx, source_y + dy)
            else {
                continue;
            };
            if nibble == 0 {
                continue;
            }
            let color = picture_palette_color(nibble, palette);
            write_native_rgba_pixel(
                pixels,
                surface,
                target_x_byte * i32::from(SCREEN_COLUMN_WIDTH_PIXELS_U8) + dx,
                target_y + dy,
                color,
            );
        }
    }
}

fn clear_source_defender_logo_word(
    pixels: &mut BTreeMap<[u32; 2], [u8; 4]>,
    target_x_byte: i32,
    target_y: i32,
) {
    for dy in 0..2 {
        for dx in 0..2 {
            let native_x = target_x_byte * i32::from(SCREEN_COLUMN_WIDTH_PIXELS_U8) + dx;
            let native_y = target_y + dy;
            if native_x < 0 || native_y < 0 {
                continue;
            }
            let position = [
                u32::try_from(native_x).expect("native x is non-negative"),
                u32::try_from(native_y).expect("native y is non-negative"),
            ];
            pixels.remove(&position);
        }
    }
}

fn source_defender_logo_nibble(
    source: &[u8; HALL_OF_FAME_DEFENDER_LOGO_BYTE_COUNT],
    native_x: i32,
    native_y: i32,
) -> Option<u8> {
    if native_x < 0
        || native_y < 0
        || native_x >= i32::from(HALL_OF_FAME_DEFENDER_LOGO_COLUMNS) * 2
        || native_y >= i32::from(HALL_OF_FAME_DEFENDER_LOGO_ROWS)
    {
        return None;
    }

    let byte_column = usize::try_from(native_x / 2).ok()?;
    let row = usize::try_from(native_y).ok()?;
    let packed = source[byte_column * usize::from(HALL_OF_FAME_DEFENDER_LOGO_ROWS) + row];
    if native_x % 2 == 0 {
        Some(packed >> 4)
    } else {
        Some(packed & 0x0F)
    }
}

fn write_native_rgba_pixel(
    pixels: &mut BTreeMap<[u32; 2], [u8; 4]>,
    surface: SurfaceSize,
    native_x: i32,
    native_y: i32,
    color: [u8; 4],
) {
    if native_x < 0
        || native_y < 0
        || native_x >= surface.width as i32
        || native_y >= surface.height as i32
    {
        return;
    }

    let position = [
        u32::try_from(native_x).expect("native x is non-negative"),
        u32::try_from(native_y).expect("native y is non-negative"),
    ];
    pixels.insert(position, color);
}

const fn source_defender_logo_color_byte(nibble: u8) -> u8 {
    match (nibble & 0x0C) >> 2 {
        1 => 0x22,
        2 => 0xCC,
        3 => 0x00,
        _ => 0x00,
    }
}

fn pseudo_color_rgba(value: u8) -> [u8; 4] {
    if value == 0 {
        return TRANSPARENT_RGBA;
    }

    [
        WILLIAMS_RED_GREEN_DAC_LEVELS[usize::from(value & 0x07)],
        WILLIAMS_RED_GREEN_DAC_LEVELS[usize::from((value >> 3) & 0x07)],
        WILLIAMS_BLUE_DAC_LEVELS[usize::from((value >> 6) & 0x03)],
        255,
    ]
}

fn object_cycle_palette_color(phase: usize) -> [u8; 4] {
    pseudo_color_rgba(OBJECT_COLOR_SEQUENCE[phase % (OBJECT_COLOR_SEQUENCE.len() - 1)])
}

fn blit_default_region(
    atlas_pixels: &mut [u8],
    atlas_surface: SurfaceSize,
    regions: &[AtlasRegion],
    sprite: SpriteId,
    source: &EmbeddedSprite,
) {
    blit_default_region_from_source(
        atlas_pixels,
        atlas_surface,
        regions,
        sprite,
        source,
        SpriteAtlasSourceRegion {
            origin: [0, 0],
            size: [source.surface.width, source.surface.height],
        },
    );
}

fn blit_default_region_from_source(
    atlas_pixels: &mut [u8],
    atlas_surface: SurfaceSize,
    regions: &[AtlasRegion],
    sprite: SpriteId,
    source: &EmbeddedSprite,
    source_region: SpriteAtlasSourceRegion,
) {
    let Some(region) = regions
        .iter()
        .copied()
        .find(|region| region.sprite == sprite)
    else {
        return;
    };
    blit_scaled_region(atlas_pixels, atlas_surface, region, source, source_region);
}

fn fill_default_region(
    atlas_pixels: &mut [u8],
    atlas_surface: SurfaceSize,
    regions: &[AtlasRegion],
    sprite: SpriteId,
    color: [u8; 4],
) {
    let Some(region) = regions
        .iter()
        .copied()
        .find(|region| region.sprite == sprite)
    else {
        return;
    };
    let width = region.size[0].min(atlas_surface.width.saturating_sub(region.origin[0]));
    let height = region.size[1].min(atlas_surface.height.saturating_sub(region.origin[1]));
    for dest_y in 0..height {
        for dest_x in 0..width {
            let start = atlas_pixel_offset(
                atlas_surface,
                [region.origin[0] + dest_x, region.origin[1] + dest_y],
            );
            let end = start + 4;
            if end <= atlas_pixels.len() {
                atlas_pixels[start..end].copy_from_slice(&color);
            }
        }
    }
}

fn blit_scaled_region(
    atlas_pixels: &mut [u8],
    atlas_surface: SurfaceSize,
    region: AtlasRegion,
    source: &EmbeddedSprite,
    source_region: SpriteAtlasSourceRegion,
) {
    if region.size[0] == 0
        || region.size[1] == 0
        || source_region.size[0] == 0
        || source_region.size[1] == 0
    {
        return;
    }

    let width = region.size[0].min(atlas_surface.width.saturating_sub(region.origin[0]));
    let height = region.size[1].min(atlas_surface.height.saturating_sub(region.origin[1]));
    for dest_y in 0..height {
        let src_y = source_region.origin[1]
            .saturating_add(dest_y.saturating_mul(source_region.size[1]) / region.size[1]);
        for dest_x in 0..width {
            let src_x = source_region.origin[0]
                .saturating_add(dest_x.saturating_mul(source_region.size[0]) / region.size[0]);
            copy_source_pixel(
                atlas_pixels,
                atlas_surface,
                [region.origin[0] + dest_x, region.origin[1] + dest_y],
                source,
                [src_x, src_y],
            );
        }
    }
}

fn blit_star_region(
    atlas_pixels: &mut [u8],
    atlas_surface: SurfaceSize,
    regions: &[AtlasRegion],
    source: &EmbeddedSprite,
) {
    let Some(region) = regions
        .iter()
        .copied()
        .find(|region| region.sprite == SpriteId::STAR)
    else {
        return;
    };
    let Some(pixel) = first_visible_pixel(source) else {
        return;
    };
    let start = atlas_pixel_offset(atlas_surface, region.origin);
    let end = start + 4;
    if end <= atlas_pixels.len() {
        atlas_pixels[start..end].copy_from_slice(pixel);
    }
}

fn first_visible_pixel(source: &EmbeddedSprite) -> Option<&[u8]> {
    source.pixels.chunks_exact(4).find(|pixel| pixel[3] != 0)
}

fn copy_source_pixel(
    atlas_pixels: &mut [u8],
    atlas_surface: SurfaceSize,
    atlas_position: [u32; 2],
    source: &EmbeddedSprite,
    source_position: [u32; 2],
) {
    if source_position[0] >= source.surface.width || source_position[1] >= source.surface.height {
        return;
    }
    let source_start = atlas_pixel_offset(source.surface, source_position);
    let source_end = source_start + 4;
    let dest_start = atlas_pixel_offset(atlas_surface, atlas_position);
    let dest_end = dest_start + 4;
    if source_end <= source.pixels.len() && dest_end <= atlas_pixels.len() {
        atlas_pixels[dest_start..dest_end]
            .copy_from_slice(&source.pixels[source_start..source_end]);
    }
}

fn atlas_pixel_offset(surface: SurfaceSize, position: [u32; 2]) -> usize {
    ((position[1] as usize * surface.width as usize) + position[0] as usize) * 4
}
