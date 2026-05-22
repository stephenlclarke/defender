//! Wgpu-oriented scene contracts.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

impl SurfaceSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn rgba_len(self) -> Option<usize> {
        let pixels = (self.width as usize).checked_mul(self.height as usize)?;
        pixels.checked_mul(4)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportLayout {
    pub target: SurfaceSize,
    pub scene: SurfaceSize,
    pub origin: [u32; 2],
    pub size: SurfaceSize,
    pub scale: f32,
}

impl ViewportLayout {
    pub fn fit(scene: SurfaceSize, target: SurfaceSize) -> Self {
        if scene.is_empty() || target.is_empty() {
            return Self::empty(scene, target);
        }

        let scale = (f64::from(target.width) / f64::from(scene.width))
            .min(f64::from(target.height) / f64::from(scene.height));
        let width = scaled_viewport_dimension(scene.width, scale, target.width);
        let height = scaled_viewport_dimension(scene.height, scale, target.height);

        Self {
            target,
            scene,
            origin: [
                target.width.saturating_sub(width) / 2,
                target.height.saturating_sub(height) / 2,
            ],
            size: SurfaceSize::new(width, height),
            scale: scale as f32,
        }
    }

    pub const fn empty(scene: SurfaceSize, target: SurfaceSize) -> Self {
        Self {
            target,
            scene,
            origin: [0, 0],
            size: SurfaceSize::new(0, 0),
            scale: 0.0,
        }
    }

    pub const fn is_empty(self) -> bool {
        self.size.is_empty() || self.scale == 0.0
    }
}

fn scaled_viewport_dimension(scene_extent: u32, scale: f64, target_extent: u32) -> u32 {
    (f64::from(scene_extent) * scale)
        .round()
        .clamp(1.0, f64::from(target_extent)) as u32
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WgpuViewportCommand {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl WgpuViewportCommand {
    pub fn from_layout(layout: ViewportLayout) -> Option<Self> {
        if layout.is_empty() {
            return None;
        }

        Some(Self {
            x: layout.origin[0] as f32,
            y: layout.origin[1] as f32,
            width: layout.size.width as f32,
            height: layout.size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SceneProjectionUniforms {
    pub scale: [f32; 2],
    pub translate: [f32; 2],
}

impl SceneProjectionUniforms {
    pub const FLOAT_COMPONENTS: usize = 4;
    pub const BYTE_SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;

    pub fn for_surface(surface: SurfaceSize) -> Option<Self> {
        if surface.is_empty() {
            return None;
        }

        Some(Self {
            scale: [2.0 / surface.width as f32, -2.0 / surface.height as f32],
            translate: [-1.0, 1.0],
        })
    }

    pub fn project_point(self, point: [f32; 2]) -> [f32; 2] {
        [
            point[0].mul_add(self.scale[0], self.translate[0]),
            point[1].mul_add(self.scale[1], self.translate[1]),
        ]
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WgpuPassPlan {
    pub clear_color: wgpu::Color,
    pub viewport: Option<WgpuViewportCommand>,
    pub scene_projection: Option<SceneProjectionUniforms>,
}

impl WgpuPassPlan {
    pub fn from_scene(scene: &RenderScene, viewport: ViewportLayout) -> Self {
        Self {
            clear_color: scene.clear_color.to_wgpu(),
            viewport: WgpuViewportCommand::from_layout(viewport),
            scene_projection: SceneProjectionUniforms::for_surface(scene.surface),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub rgba: [u8; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceAttractDefenderAppearancePixel {
    pub position: [u16; 2],
    pub color: [u8; 4],
}

impl Color {
    pub const WHITE: Self = Self {
        rgba: [0xFF, 0xFF, 0xFF, 0xFF],
    };

    pub const fn from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            rgba: [red, green, blue, alpha],
        }
    }

    pub fn to_wgpu(self) -> wgpu::Color {
        wgpu::Color {
            r: f64::from(self.rgba[0]) / 255.0,
            g: f64::from(self.rgba[1]) / 255.0,
            b: f64::from(self.rgba[2]) / 255.0,
            a: f64::from(self.rgba[3]) / 255.0,
        }
    }

    pub fn to_normalized_rgba(self) -> [f32; 4] {
        [
            f32::from(self.rgba[0]) / 255.0,
            f32::from(self.rgba[1]) / 255.0,
            f32::from(self.rgba[2]) / 255.0,
            f32::from(self.rgba[3]) / 255.0,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderLayer {
    Terrain,
    Starfield,
    Objects,
    Projectiles,
    Hud,
    Overlay,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderLayerCounts {
    pub terrain: usize,
    pub starfield: usize,
    pub objects: usize,
    pub projectiles: usize,
    pub hud: usize,
    pub overlay: usize,
}

impl RenderLayerCounts {
    fn add(&mut self, layer: RenderLayer) {
        match layer {
            RenderLayer::Terrain => self.terrain += 1,
            RenderLayer::Starfield => self.starfield += 1,
            RenderLayer::Objects => self.objects += 1,
            RenderLayer::Projectiles => self.projectiles += 1,
            RenderLayer::Hud => self.hud += 1,
            RenderLayer::Overlay => self.overlay += 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteId(pub u16);

impl SpriteId {
    pub const PLAYER_SHIP: Self = Self(1);
    pub const SCORE_TEXT: Self = Self(2);
    pub const STATUS_TEXT: Self = Self(3);
    pub const PLAYER_PROJECTILE: Self = Self(4);
    pub const TERRAIN_TILE: Self = Self(5);
    pub const STAR: Self = Self(6);
    pub const ENEMY_LANDER: Self = Self(7);
    pub const HUMAN: Self = Self(8);
    pub const ENEMY_MUTANT: Self = Self(9);
    pub const ENEMY_BAITER: Self = Self(10);
    pub const ENEMY_BOMBER: Self = Self(11);
    pub const ENEMY_POD: Self = Self(12);
    pub const ENEMY_SWARMER: Self = Self(13);
    pub const ENEMY_BOMB: Self = Self(14);
    pub const BOMB_EXPLOSION: Self = Self(15);
    pub const SWARMER_EXPLOSION: Self = Self(16);
    pub const SCORE_POPUP_250: Self = Self(17);
    pub const SCORE_POPUP_500: Self = Self(18);
    pub const PLAYER_LIFE_STOCK: Self = Self(19);
    pub const SMART_BOMB_STOCK: Self = Self(20);
    pub const ASTRONAUT_EXPLOSION: Self = Self(21);
    pub const NULL_OBJECT: Self = Self(22);
    pub const TERRAIN_EXPLOSION: Self = Self(23);
    pub const SCORE_DIGIT_0: Self = Self(24);
    pub const SCORE_DIGIT_1: Self = Self(25);
    pub const SCORE_DIGIT_2: Self = Self(26);
    pub const SCORE_DIGIT_3: Self = Self(27);
    pub const SCORE_DIGIT_4: Self = Self(28);
    pub const SCORE_DIGIT_5: Self = Self(29);
    pub const SCORE_DIGIT_6: Self = Self(30);
    pub const SCORE_DIGIT_7: Self = Self(31);
    pub const SCORE_DIGIT_8: Self = Self(32);
    pub const SCORE_DIGIT_9: Self = Self(33);
    pub const MESSAGE_GLYPH_SPACE: Self = Self(34);
    pub const MESSAGE_GLYPH_COMMA: Self = Self(35);
    pub const MESSAGE_GLYPH_PERIOD: Self = Self(36);
    pub const MESSAGE_GLYPH_COLON: Self = Self(37);
    pub const MESSAGE_GLYPH_QUESTION: Self = Self(38);
    pub const MESSAGE_GLYPH_A: Self = Self(39);
    pub const MESSAGE_GLYPH_B: Self = Self(40);
    pub const MESSAGE_GLYPH_C: Self = Self(41);
    pub const MESSAGE_GLYPH_D: Self = Self(42);
    pub const MESSAGE_GLYPH_E: Self = Self(43);
    pub const MESSAGE_GLYPH_F: Self = Self(44);
    pub const MESSAGE_GLYPH_G: Self = Self(45);
    pub const MESSAGE_GLYPH_H: Self = Self(46);
    pub const MESSAGE_GLYPH_I: Self = Self(47);
    pub const MESSAGE_GLYPH_J: Self = Self(48);
    pub const MESSAGE_GLYPH_K: Self = Self(49);
    pub const MESSAGE_GLYPH_L: Self = Self(50);
    pub const MESSAGE_GLYPH_M: Self = Self(51);
    pub const MESSAGE_GLYPH_N: Self = Self(52);
    pub const MESSAGE_GLYPH_O: Self = Self(53);
    pub const MESSAGE_GLYPH_P: Self = Self(54);
    pub const MESSAGE_GLYPH_Q: Self = Self(55);
    pub const MESSAGE_GLYPH_R: Self = Self(56);
    pub const MESSAGE_GLYPH_S: Self = Self(57);
    pub const MESSAGE_GLYPH_T: Self = Self(58);
    pub const MESSAGE_GLYPH_U: Self = Self(59);
    pub const MESSAGE_GLYPH_V: Self = Self(60);
    pub const MESSAGE_GLYPH_W: Self = Self(61);
    pub const MESSAGE_GLYPH_X: Self = Self(62);
    pub const MESSAGE_GLYPH_Y: Self = Self(63);
    pub const MESSAGE_GLYPH_Z: Self = Self(64);
    pub const HALL_OF_FAME_UNDERLINE_WORD: Self = Self(65);
    pub const HALL_OF_FAME_DEFENDER_LOGO: Self = Self(66);
    pub const ATTRACT_COPYRIGHT_STRIP: Self = Self(67);
    pub const ATTRACT_WILLIAMS_LOGO: Self = Self(68);
    pub const TOP_DISPLAY_BORDER_WORD: Self = Self(69);
    pub const SCANNER_OBJECT_BLIP: Self = Self(70);
    pub const SCANNER_PLAYER_BLIP: Self = Self(71);
    pub const PLAYER_EXPLOSION_PIXEL: Self = Self(72);
    pub const ATTRACT_WILLIAMS_LOGO_PIXEL: Self = Self(73);
    pub const ATTRACT_DEFENDER_WORDMARK_BLOCK_BASE: Self = Self(74);
    pub const TERRAIN_TILE_ALT: Self = Self(89);
    pub const ATTRACT_SCANNER_TERRAIN_PIXEL: Self = Self(90);
    pub const SCORE_DIGITS: [Self; 10] = [
        Self::SCORE_DIGIT_0,
        Self::SCORE_DIGIT_1,
        Self::SCORE_DIGIT_2,
        Self::SCORE_DIGIT_3,
        Self::SCORE_DIGIT_4,
        Self::SCORE_DIGIT_5,
        Self::SCORE_DIGIT_6,
        Self::SCORE_DIGIT_7,
        Self::SCORE_DIGIT_8,
        Self::SCORE_DIGIT_9,
    ];
    pub const MESSAGE_GLYPHS: [Self; 31] = [
        Self::MESSAGE_GLYPH_SPACE,
        Self::MESSAGE_GLYPH_COMMA,
        Self::MESSAGE_GLYPH_PERIOD,
        Self::MESSAGE_GLYPH_COLON,
        Self::MESSAGE_GLYPH_QUESTION,
        Self::MESSAGE_GLYPH_A,
        Self::MESSAGE_GLYPH_B,
        Self::MESSAGE_GLYPH_C,
        Self::MESSAGE_GLYPH_D,
        Self::MESSAGE_GLYPH_E,
        Self::MESSAGE_GLYPH_F,
        Self::MESSAGE_GLYPH_G,
        Self::MESSAGE_GLYPH_H,
        Self::MESSAGE_GLYPH_I,
        Self::MESSAGE_GLYPH_J,
        Self::MESSAGE_GLYPH_K,
        Self::MESSAGE_GLYPH_L,
        Self::MESSAGE_GLYPH_M,
        Self::MESSAGE_GLYPH_N,
        Self::MESSAGE_GLYPH_O,
        Self::MESSAGE_GLYPH_P,
        Self::MESSAGE_GLYPH_Q,
        Self::MESSAGE_GLYPH_R,
        Self::MESSAGE_GLYPH_S,
        Self::MESSAGE_GLYPH_T,
        Self::MESSAGE_GLYPH_U,
        Self::MESSAGE_GLYPH_V,
        Self::MESSAGE_GLYPH_W,
        Self::MESSAGE_GLYPH_X,
        Self::MESSAGE_GLYPH_Y,
        Self::MESSAGE_GLYPH_Z,
    ];

    pub(crate) fn attract_defender_wordmark_block(index: usize) -> Option<Self> {
        if index < ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT {
            Some(Self(
                Self::ATTRACT_DEFENDER_WORDMARK_BLOCK_BASE.0
                    + u16::try_from(index).expect("Defender wordmark block index fits in u16"),
            ))
        } else {
            None
        }
    }

    const MESSAGE_GLYPH_SPECS: [(char, Self, [u32; 2]); 31] = [
        (' ', Self::MESSAGE_GLYPH_SPACE, [2, 8]),
        (',', Self::MESSAGE_GLYPH_COMMA, [2, 8]),
        ('.', Self::MESSAGE_GLYPH_PERIOD, [2, 8]),
        (':', Self::MESSAGE_GLYPH_COLON, [2, 8]),
        ('?', Self::MESSAGE_GLYPH_QUESTION, [6, 8]),
        ('A', Self::MESSAGE_GLYPH_A, [6, 8]),
        ('B', Self::MESSAGE_GLYPH_B, [6, 8]),
        ('C', Self::MESSAGE_GLYPH_C, [6, 8]),
        ('D', Self::MESSAGE_GLYPH_D, [6, 8]),
        ('E', Self::MESSAGE_GLYPH_E, [6, 8]),
        ('F', Self::MESSAGE_GLYPH_F, [6, 8]),
        ('G', Self::MESSAGE_GLYPH_G, [6, 8]),
        ('H', Self::MESSAGE_GLYPH_H, [6, 8]),
        ('I', Self::MESSAGE_GLYPH_I, [4, 8]),
        ('J', Self::MESSAGE_GLYPH_J, [6, 8]),
        ('K', Self::MESSAGE_GLYPH_K, [6, 8]),
        ('L', Self::MESSAGE_GLYPH_L, [6, 8]),
        ('M', Self::MESSAGE_GLYPH_M, [8, 8]),
        ('N', Self::MESSAGE_GLYPH_N, [6, 8]),
        ('O', Self::MESSAGE_GLYPH_O, [6, 8]),
        ('P', Self::MESSAGE_GLYPH_P, [6, 8]),
        ('Q', Self::MESSAGE_GLYPH_Q, [6, 8]),
        ('R', Self::MESSAGE_GLYPH_R, [6, 8]),
        ('S', Self::MESSAGE_GLYPH_S, [6, 8]),
        ('T', Self::MESSAGE_GLYPH_T, [6, 8]),
        ('U', Self::MESSAGE_GLYPH_U, [6, 8]),
        ('V', Self::MESSAGE_GLYPH_V, [6, 8]),
        ('W', Self::MESSAGE_GLYPH_W, [8, 8]),
        ('X', Self::MESSAGE_GLYPH_X, [6, 8]),
        ('Y', Self::MESSAGE_GLYPH_Y, [6, 8]),
        ('Z', Self::MESSAGE_GLYPH_Z, [6, 8]),
    ];

    pub fn score_digit(digit: u8) -> Option<Self> {
        Self::SCORE_DIGITS.get(usize::from(digit)).copied()
    }

    pub fn message_glyph(character: char) -> Option<Self> {
        Self::MESSAGE_GLYPH_SPECS
            .iter()
            .find(|(glyph_character, _, _)| *glyph_character == character)
            .map(|(_, sprite, _)| *sprite)
    }

    pub fn message_glyph_size(character: char) -> Option<[u32; 2]> {
        Self::MESSAGE_GLYPH_SPECS
            .iter()
            .find(|(glyph_character, _, _)| *glyph_character == character)
            .map(|(_, _, size)| *size)
    }

    pub fn for_object_picture_label(label: &str) -> Option<Self> {
        match label {
            "PLAPIC" | "PLBPIC" => Some(Self::PLAYER_SHIP),
            "LNDP1" | "LNDP2" | "LNDP3" => Some(Self::ENEMY_LANDER),
            "ASTP1" | "ASTP2" | "ASTP3" | "ASTP4" => Some(Self::HUMAN),
            "LASP1" => Some(Self::PLAYER_PROJECTILE),
            "SCZP1" => Some(Self::ENEMY_MUTANT),
            "UFOP1" | "UFOP2" | "UFOP3" => Some(Self::ENEMY_BAITER),
            "TIEP1" | "TIEP2" | "TIEP3" | "TIEP4" => Some(Self::ENEMY_BOMBER),
            "PRBP1" => Some(Self::ENEMY_POD),
            "SWPIC1" => Some(Self::ENEMY_SWARMER),
            "BMBP1" | "BMBP2" => Some(Self::ENEMY_BOMB),
            "BXPIC" => Some(Self::BOMB_EXPLOSION),
            "SWXP1" => Some(Self::SWARMER_EXPLOSION),
            "C25P1" => Some(Self::SCORE_POPUP_250),
            "C5P1" => Some(Self::SCORE_POPUP_500),
            "PLAMIN" => Some(Self::PLAYER_LIFE_STOCK),
            "SBPIC" => Some(Self::SMART_BOMB_STOCK),
            "ASXP1" => Some(Self::ASTRONAUT_EXPLOSION),
            "NULOB" => Some(Self::NULL_OBJECT),
            "TEREX" => Some(Self::TERRAIN_EXPLOSION),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneSprite {
    pub sprite: SpriteId,
    pub layer: RenderLayer,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub tint: Color,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneRaster {
    pub surface: SurfaceSize,
    pixels: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneRasterError {
    PixelBufferTooLarge { surface: SurfaceSize },
    PixelBufferLength { expected: usize, actual: usize },
}

impl SceneRaster {
    pub fn from_rgba(surface: SurfaceSize, pixels: Vec<u8>) -> Result<Self, SceneRasterError> {
        let Some(expected) = surface.rgba_len() else {
            return Err(SceneRasterError::PixelBufferTooLarge { surface });
        };
        if pixels.len() != expected {
            return Err(SceneRasterError::PixelBufferLength {
                expected,
                actual: pixels.len(),
            });
        }

        Ok(Self { surface, pixels })
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn into_pixels(self) -> Vec<u8> {
        self.pixels
    }

    pub fn is_non_blank(&self) -> bool {
        self.pixels
            .chunks_exact(4)
            .any(|pixel| pixel != [0, 0, 0, 255].as_slice())
    }
}

impl std::fmt::Display for SceneRasterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PixelBufferTooLarge { surface } => write!(
                formatter,
                "rgba buffer is too large for {}x{} surface",
                surface.width, surface.height
            ),
            Self::PixelBufferLength { expected, actual } => write!(
                formatter,
                "rgba buffer length mismatch: expected {expected} bytes, got {actual}"
            ),
        }
    }
}

impl std::error::Error for SceneRasterError {}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderScene {
    pub frame: u64,
    pub surface: SurfaceSize,
    pub clear_color: Color,
    pub visual_signature: Option<u32>,
    pub sprites: Vec<SceneSprite>,
    raster: Option<SceneRaster>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderSceneSummary {
    pub frame: u64,
    pub surface: SurfaceSize,
    pub visual_signature: Option<u32>,
    pub raster_count: usize,
    pub sprite_count: usize,
    pub layers: RenderLayerCounts,
}

impl RenderScene {
    pub fn empty(frame: u64, surface: SurfaceSize) -> Self {
        Self {
            frame,
            surface,
            clear_color: Color { rgba: [0; 4] },
            visual_signature: None,
            sprites: Vec::new(),
            raster: None,
        }
    }

    pub fn from_rgba(
        frame: u64,
        surface: SurfaceSize,
        pixels: Vec<u8>,
        visual_signature: Option<u32>,
    ) -> Result<Self, SceneRasterError> {
        let raster = SceneRaster::from_rgba(surface, pixels)?;
        Ok(Self {
            frame,
            surface,
            clear_color: Color { rgba: [0; 4] },
            visual_signature,
            sprites: Vec::new(),
            raster: Some(raster),
        })
    }

    pub fn push_sprite(&mut self, sprite: SceneSprite) {
        self.sprites.push(sprite);
    }

    pub fn raster(&self) -> Option<&SceneRaster> {
        self.raster.as_ref()
    }

    pub fn set_raster(&mut self, raster: SceneRaster) {
        self.surface = raster.surface;
        self.raster = Some(raster);
    }

    pub fn summary(&self) -> RenderSceneSummary {
        let mut layers = RenderLayerCounts::default();
        for sprite in &self.sprites {
            layers.add(sprite.layer);
        }

        RenderSceneSummary {
            frame: self.frame,
            surface: self.surface,
            visual_signature: self.visual_signature,
            raster_count: usize::from(self.raster.is_some()),
            sprite_count: self.sprites.len(),
            layers,
        }
    }
}

pub fn source_message_text(label: &str) -> Option<&'static str> {
    SOURCE_MESSAGES_TSV.lines().skip(1).find_map(|line| {
        if line.trim().is_empty() {
            return None;
        }
        let mut fields = line.split('\t');
        let row_label = fields.next()?;
        let _vector_address = fields.next()?;
        let words = fields.next()?;
        if row_label == label {
            Some(words)
        } else {
            None
        }
    })
}

pub fn source_screen_position(screen_address: u16) -> [f32; 2] {
    let [column, row] = screen_address.to_be_bytes();
    [
        f32::from(column) * SOURCE_SCREEN_COLUMN_PIXELS,
        f32::from(row),
    ]
}

pub fn source_screen_position_with_offset(
    top_left_screen_address: u16,
    horizontal: u8,
    vertical: u8,
) -> [f32; 2] {
    let [column, row] = top_left_screen_address.to_be_bytes();
    source_screen_position(u16::from_be_bytes([
        column.wrapping_add(horizontal),
        row.wrapping_add(vertical),
    ]))
}

pub fn push_source_message_sprites(
    scene: &mut RenderScene,
    text: &str,
    origin: [f32; 2],
    layer: RenderLayer,
) {
    let mut cursor_x = origin[0];
    for character in text.chars() {
        let Some(size) = SpriteId::message_glyph_size(character) else {
            continue;
        };
        if character != ' '
            && let Some(sprite) = SpriteId::message_glyph(character)
        {
            scene.push_sprite(SceneSprite {
                sprite,
                layer,
                position: [cursor_x, origin[1]],
                size: [size[0] as f32, size[1] as f32],
                tint: Color::WHITE,
            });
        }
        cursor_x += size[0] as f32 + SOURCE_SCREEN_COLUMN_PIXELS;
    }
}

pub fn push_source_text_bytes_sprites(
    scene: &mut RenderScene,
    bytes: &[u8],
    origin: [f32; 2],
    layer: RenderLayer,
) {
    let mut cursor_x = origin[0];
    for byte in bytes {
        let Some((sprite, size)) = source_text_byte_sprite(*byte) else {
            continue;
        };
        if let Some(sprite) = sprite {
            scene.push_sprite(SceneSprite {
                sprite,
                layer,
                position: [cursor_x, origin[1]],
                size: [size[0] as f32, size[1] as f32],
                tint: Color::WHITE,
            });
        }
        cursor_x += size[0] as f32 + SOURCE_SCREEN_COLUMN_PIXELS;
    }
}

pub fn push_source_controlled_message_sprites(
    scene: &mut RenderScene,
    text: &str,
    top_left_screen_address: u16,
    layer: RenderLayer,
) {
    let mut layout = SourceMessageTextLayout {
        top_left: top_left_screen_address,
        cursor: top_left_screen_address,
        line_spacing: SOURCE_MESSAGE_LINE_SPACING,
    };

    for word in text.split_whitespace() {
        if let Some(control) = source_message_control(word) {
            layout.apply(control);
            continue;
        }
        if source_message_control_body(word).is_some() {
            continue;
        }

        let bytes = word.as_bytes();
        push_source_text_bytes_sprites(scene, bytes, source_screen_position(layout.cursor), layer);
        layout.cursor = source_text_cursor_after_bytes(layout.cursor, bytes);
        layout.cursor = source_text_cursor_after_bytes(layout.cursor, b" ");
    }
}

fn source_text_byte_sprite(byte: u8) -> Option<(Option<SpriteId>, [u32; 2])> {
    if byte == b' ' {
        return Some((None, SpriteId::message_glyph_size(' ')?));
    }

    if byte.is_ascii_digit() {
        return Some((SpriteId::score_digit(byte - b'0'), SOURCE_SCORE_DIGIT_SIZE));
    }

    let character = char::from(byte);
    let size = SpriteId::message_glyph_size(character)?;
    Some((SpriteId::message_glyph(character), size))
}

fn source_text_cursor_after_bytes(mut cursor: u16, bytes: &[u8]) -> u16 {
    for byte in bytes {
        let Some((_sprite, size)) = source_text_byte_sprite(*byte) else {
            continue;
        };
        cursor = source_text_cursor_advance(cursor, size[0]);
    }
    cursor
}

fn source_text_cursor_advance(cursor: u16, width_pixels: u32) -> u16 {
    let [column, row] = cursor.to_be_bytes();
    let width_columns = u8::try_from(width_pixels / u32::from(SOURCE_SCREEN_COLUMN_PIXELS_U8))
        .expect("source glyph width fits in u8");
    u16::from_be_bytes([column.wrapping_add(width_columns).wrapping_add(1), row])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceMessageTextLayout {
    top_left: u16,
    cursor: u16,
    line_spacing: u8,
}

impl SourceMessageTextLayout {
    fn apply(&mut self, control: SourceMessageControl) {
        match control {
            SourceMessageControl::HorizontalFromTopLeft(delta) => {
                let [top_x, cursor_y] =
                    [self.top_left.to_be_bytes()[0], self.cursor.to_be_bytes()[1]];
                self.cursor = u16::from_be_bytes([top_x.wrapping_add(delta), cursor_y]);
            }
            SourceMessageControl::HorizontalFromCursor(delta) => {
                let [cursor_x, cursor_y] = self.cursor.to_be_bytes();
                self.cursor = u16::from_be_bytes([cursor_x.wrapping_add(delta), cursor_y]);
            }
            SourceMessageControl::VerticalFromTopLeft(delta) => {
                let [cursor_x, _cursor_y] = self.cursor.to_be_bytes();
                let top_y = self.top_left.to_be_bytes()[1];
                self.cursor = u16::from_be_bytes([cursor_x, top_y.wrapping_add(delta)]);
            }
            SourceMessageControl::VerticalFromCursor(delta) => {
                let [cursor_x, cursor_y] = self.cursor.to_be_bytes();
                self.cursor = u16::from_be_bytes([cursor_x, cursor_y.wrapping_add(delta)]);
            }
            SourceMessageControl::ResetTopLeftAndCursor(address) => {
                self.top_left = address;
                self.cursor = address;
            }
            SourceMessageControl::ReturnLineFeed => {
                let [top_x, _top_y] = self.top_left.to_be_bytes();
                let cursor_y = self.cursor.to_be_bytes()[1];
                self.cursor = u16::from_be_bytes([top_x, cursor_y.wrapping_add(self.line_spacing)]);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceMessageControl {
    HorizontalFromTopLeft(u8),
    HorizontalFromCursor(u8),
    VerticalFromTopLeft(u8),
    VerticalFromCursor(u8),
    ResetTopLeftAndCursor(u16),
    ReturnLineFeed,
}

fn source_message_control(word: &str) -> Option<SourceMessageControl> {
    let body = source_message_control_body(word)?;
    let (name, arguments) = body.split_once(':').unwrap_or((body, ""));
    match name {
        "HMT" => Some(SourceMessageControl::HorizontalFromTopLeft(
            source_message_control_byte(arguments)?,
        )),
        "HMC" => Some(SourceMessageControl::HorizontalFromCursor(
            source_message_control_byte(arguments)?,
        )),
        "VMT" => Some(SourceMessageControl::VerticalFromTopLeft(
            source_message_control_byte(arguments)?,
        )),
        "VMC" => Some(SourceMessageControl::VerticalFromCursor(
            source_message_control_byte(arguments)?,
        )),
        "RTC" => {
            let (x, y) = arguments.split_once(',')?;
            Some(SourceMessageControl::ResetTopLeftAndCursor(
                u16::from_be_bytes([
                    source_message_control_byte(x)?,
                    source_message_control_byte(y)?,
                ]),
            ))
        }
        "RLF" if arguments.is_empty() => Some(SourceMessageControl::ReturnLineFeed),
        _ => None,
    }
}

fn source_message_control_body(word: &str) -> Option<&str> {
    word.strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
}

fn source_message_control_byte(value: &str) -> Option<u8> {
    let hex = value.strip_prefix("0x")?;
    u8::from_str_radix(hex, 16).ok()
}

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
struct SpriteAssetSource {
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
        let color = source_object_cycle_color(phase);
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
            d: pseudo_color_rgba(SOURCE_TIE_COLOR_TABLE[offset]),
            e: pseudo_color_rgba(SOURCE_TIE_COLOR_TABLE[offset + 1]),
            f: pseudo_color_rgba(SOURCE_TIE_COLOR_TABLE[offset + 2]),
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
const SOURCE_TIE_COLOR_TABLE: [u8; 9] = [0x81, 0x81, 0x2F, 0x81, 0x2F, 0x07, 0x2F, 0x81, 0x07];
const SOURCE_OBJECT_COLOR_TABLE: [u8; 37] = [
    0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x37, 0x2F, 0x27, 0x1F, 0x17, 0x47, 0x47, 0x87,
    0x87, 0xC7, 0xC7, 0xC6, 0xC5, 0xCC, 0xCB, 0xCA, 0xDA, 0xE8, 0xF8, 0xF9, 0xFA, 0xFB, 0xFD, 0xFF,
    0xBF, 0x3F, 0x3E, 0x3C, 0x00,
];
const SOURCE_WILLIAMS_RED_GREEN_LEVELS: [u8; 8] = [0, 38, 81, 118, 137, 174, 217, 255];
const SOURCE_WILLIAMS_BLUE_LEVELS: [u8; 4] = [0, 95, 160, 255];
const FONT_SHEET_PNG: &[u8] = include_bytes!("../assets/sprites/font-sheet.png");
const ARCADE_SCORE_DIGITS_TSV: &str = include_str!("../assets/red-label/score-digits.tsv");
const SOURCE_MESSAGE_GLYPHS_TSV: &str = include_str!("../assets/red-label/message-glyphs.tsv");
const SOURCE_MESSAGES_TSV: &str = include_str!("../assets/red-label/messages.tsv");
const SOURCE_OBJECT_IMAGES_TSV: &str = include_str!("../assets/red-label/object-images.tsv");
const SOURCE_SCORE_DIGIT_SIZE: [u32; 2] = [6, 8];
const MESSAGE_GLYPH_ATLAS_START: [u32; 2] = [0, 104];
const MESSAGE_GLYPH_ATLAS_ROW_STEP: u32 = 16;
const MESSAGE_GLYPH_ATLAS_GAP: u32 = 2;
const SOURCE_SCREEN_COLUMN_PIXELS_U8: u8 = 2;
const SOURCE_SCREEN_COLUMN_PIXELS: f32 = SOURCE_SCREEN_COLUMN_PIXELS_U8 as f32;
const SOURCE_MESSAGE_LINE_SPACING: u8 = 0x0A;
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
    rows: 4,
    bytes_per_row: 8,
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
    rows: 8,
    bytes_per_row: 6,
    bytes: &TERRAIN_EXPLOSION_BYTES,
    palette: ObjectPicturePalette::burst(),
};
const SOURCE_DEFENDER_LOGO_COLUMNS: u8 = 0x3C;
const SOURCE_DEFENDER_LOGO_ROWS: u8 = 0x18;
const SOURCE_DEFENDER_LOGO_BYTES: usize =
    SOURCE_DEFENDER_LOGO_COLUMNS as usize * SOURCE_DEFENDER_LOGO_ROWS as usize;
pub(crate) const ATTRACT_DEFENDER_WORDMARK_BLOCK_COLUMNS: usize = 15;
pub(crate) const ATTRACT_DEFENDER_WORDMARK_BLOCK_ROWS: usize = 1;
pub(crate) const ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT: usize =
    ATTRACT_DEFENDER_WORDMARK_BLOCK_COLUMNS * ATTRACT_DEFENDER_WORDMARK_BLOCK_ROWS;
const SOURCE_ATTRACT_COPYRIGHT_COLUMNS: u8 = 40;
const SOURCE_ATTRACT_COPYRIGHT_ROWS: u8 = 8;
const SOURCE_ATTRACT_COPYRIGHT_BYTES: [u8; 80] = [
    0x3E, 0x41, 0x41, 0x22, 0x00, 0x3E, 0x41, 0x41, 0x3E, 0x00, 0x7F, 0x09, 0x09, 0x06, 0x00, 0x03,
    0x04, 0x78, 0x04, 0x03, 0x00, 0x7F, 0x09, 0x19, 0x66, 0x00, 0x41, 0x7F, 0x41, 0x00, 0x3E, 0x41,
    0x49, 0x3A, 0x00, 0x7F, 0x08, 0x08, 0x7F, 0x00, 0x01, 0x01, 0x7F, 0x01, 0x01, 0x00, 0x1C, 0x22,
    0x5D, 0x63, 0x55, 0x22, 0x1C, 0x22, 0x7F, 0x4B, 0x45, 0x22, 0x1C, 0x00, 0x00, 0x00, 0x42, 0x7F,
    0x40, 0x00, 0x26, 0x49, 0x49, 0x3E, 0x00, 0x36, 0x49, 0x49, 0x36, 0x00, 0x3E, 0x41, 0x41, 0x3E,
];
const SOURCE_ATTRACT_WILLIAMS_LOGO_COLUMNS: u8 = 46;
const SOURCE_ATTRACT_WILLIAMS_LOGO_ROWS: u8 = 19;
const SOURCE_ATTRACT_WILLIAMS_LOGO_PIXELS: usize =
    SOURCE_ATTRACT_WILLIAMS_LOGO_COLUMNS as usize * 2 * SOURCE_ATTRACT_WILLIAMS_LOGO_ROWS as usize;
const SOURCE_ATTRACT_WILLIAMS_LOGO_FIRST_COLUMN: u8 = 0x36;
const SOURCE_ATTRACT_WILLIAMS_LOGO_FIRST_ROW: u8 = 0x3C;
const SOURCE_ATTRACT_WILLIAMS_LOGO_TABLE: [u8; 351] = [
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
];
const SOURCE_DEFENDER_LOGO_COMPRESSED: [u8; 366] = [
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
];

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
                size: [16, 8],
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
                size: [8, 2],
            },
            AtlasRegion {
                sprite: SpriteId::TERRAIN_TILE,
                origin: [0, 64],
                size: [8, 8],
            },
            AtlasRegion {
                sprite: SpriteId::TERRAIN_TILE_ALT,
                origin: [8, 64],
                size: [8, 8],
            },
            AtlasRegion {
                sprite: SpriteId::STAR,
                origin: [16, 64],
                size: [1, 1],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_LANDER,
                origin: [24, 64],
                size: [12, 8],
            },
            AtlasRegion {
                sprite: SpriteId::HUMAN,
                origin: [40, 64],
                size: [6, 8],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_MUTANT,
                origin: [56, 64],
                size: [12, 10],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_BAITER,
                origin: [72, 64],
                size: [14, 6],
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
                size: [6, 6],
            },
            AtlasRegion {
                sprite: SpriteId::ENEMY_BOMB,
                origin: [0, 80],
                size: [4, 6],
            },
            AtlasRegion {
                sprite: SpriteId::BOMB_EXPLOSION,
                origin: [8, 80],
                size: [10, 10],
            },
            AtlasRegion {
                sprite: SpriteId::SWARMER_EXPLOSION,
                origin: [24, 80],
                size: [10, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_POPUP_250,
                origin: [40, 80],
                size: [12, 8],
            },
            AtlasRegion {
                sprite: SpriteId::SCORE_POPUP_500,
                origin: [56, 80],
                size: [12, 6],
            },
            AtlasRegion {
                sprite: SpriteId::PLAYER_LIFE_STOCK,
                origin: [72, 80],
                size: [12, 6],
            },
            AtlasRegion {
                sprite: SpriteId::SMART_BOMB_STOCK,
                origin: [88, 80],
                size: [8, 6],
            },
            AtlasRegion {
                sprite: SpriteId::ASTRONAUT_EXPLOSION,
                origin: [0, 96],
                size: [16, 4],
            },
            AtlasRegion {
                sprite: SpriteId::NULL_OBJECT,
                origin: [20, 96],
                size: [2, 1],
            },
            AtlasRegion {
                sprite: SpriteId::TERRAIN_EXPLOSION,
                origin: [24, 96],
                size: [12, 8],
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

    let player_ship = decode_source_object_image_rgba("PLD10", 6, 8, ObjectPicturePalette::ship());
    let player_projectile =
        decode_source_object_image_rgba("LASD10", 1, 8, ObjectPicturePalette::player_shot());
    let enemy_lander =
        decode_source_object_image_rgba("LND10", 8, 5, ObjectPicturePalette::white());
    let human = decode_source_object_image_rgba("ASTD10", 8, 2, ObjectPicturePalette::white());
    let enemy_mutant =
        decode_source_object_image_rgba("SCZD10", 8, 5, ObjectPicturePalette::white());
    let enemy_baiter =
        decode_source_object_image_rgba("UFOD10", 4, 6, ObjectPicturePalette::white());
    let enemy_bomber =
        decode_source_object_image_rgba("TIED10", 8, 4, ObjectPicturePalette::tie(0));
    let enemy_pod = decode_source_object_image_rgba("PRBD10", 8, 4, ObjectPicturePalette::white());
    let enemy_swarmer =
        decode_source_object_image_rgba("SWMD10", 4, 3, ObjectPicturePalette::white());
    let enemy_bomb = decode_source_object_image_rgba("BMBD10", 3, 2, ObjectPicturePalette::bomb(0));
    let bomb_explosion =
        decode_source_object_image_rgba("BXD10", 8, 4, ObjectPicturePalette::burst());
    let swarmer_explosion =
        decode_source_object_image_rgba("SWXD10", 8, 4, ObjectPicturePalette::burst());
    let score_popup_250 =
        decode_source_object_image_rgba("C25D10", 6, 6, ObjectPicturePalette::score_250(0));
    let score_popup_500 =
        decode_source_object_image_rgba("C5D10", 6, 6, ObjectPicturePalette::score_500(0));
    let player_life_stock =
        decode_source_object_image_rgba("PLAM0", 4, 5, ObjectPicturePalette::ship());
    let smart_bomb_stock =
        decode_source_object_image_rgba("SBD10", 3, 3, ObjectPicturePalette::white());
    let astronaut_explosion = decode_picture_grid_rgba("ASXP1", ASTRONAUT_EXPLOSION_GRID);
    let null_object = decode_picture_grid_rgba("NULOB", NULL_OBJECT_GRID);
    let terrain_explosion = decode_picture_grid_rgba("TEREX", TERRAIN_EXPLOSION_GRID);
    let score_digits = decode_score_digit_sprites("score-digits.tsv", ARCADE_SCORE_DIGITS_TSV);
    let message_glyphs =
        decode_message_glyph_sprites("message-glyphs.tsv", SOURCE_MESSAGE_GLYPHS_TSV);
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
        SpriteAssetSource {
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
        SpriteAssetSource {
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
    const WIDTH: u32 = 8;
    const HEIGHT: u32 = 8;
    let mut pixels = vec![0; (WIDTH * HEIGHT * 4) as usize];

    for source_pixel in 0..4 {
        let shift = 12 - source_pixel * 4;
        let nibble = ((word >> shift) & 0x000F) as u8;
        let color = picture_palette_color(nibble, ObjectPicturePalette::white());
        for y in 0..HEIGHT {
            for repeat_x in 0..2 {
                let x = source_pixel as u32 * 2 + repeat_x;
                let start = ((y * WIDTH + x) * 4) as usize;
                pixels[start..start + 4].copy_from_slice(&color);
            }
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

fn decode_source_object_image_rgba(
    label: &'static str,
    rows: u8,
    bytes_per_row: u8,
    palette: ObjectPicturePalette,
) -> EmbeddedSprite {
    let bytes = source_object_image_bytes(label);
    decode_picture_bytes_rgba(label, rows, bytes_per_row, &bytes, palette)
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

    for row in 0..usize::from(rows) {
        for byte_index in 0..usize::from(bytes_per_row) {
            let value = bytes[row * usize::from(bytes_per_row) + byte_index];
            let left = picture_palette_color(value >> 4, palette);
            let right = picture_palette_color(value & 0x0F, palette);
            let offset = ((row * surface.width as usize) + byte_index * 2) * 4;
            pixels[offset..offset + 4].copy_from_slice(&left);
            pixels[offset + 4..offset + 8].copy_from_slice(&right);
        }
    }

    EmbeddedSprite { surface, pixels }
}

fn source_object_image_bytes(label: &'static str) -> Vec<u8> {
    for line in SOURCE_OBJECT_IMAGES_TSV.lines().skip(1) {
        let mut columns = line.split('\t');
        let Some(image_label) = columns.next() else {
            continue;
        };
        let _address = columns.next();
        let Some(hex_bytes) = columns.next() else {
            continue;
        };
        if image_label == label {
            return decode_source_hex_bytes(label, hex_bytes);
        }
    }

    panic!("source object image {label} must exist in object-images.tsv");
}

fn decode_source_hex_bytes(label: &'static str, hex_bytes: &str) -> Vec<u8> {
    assert!(
        hex_bytes.len().is_multiple_of(2),
        "source object image {label} hex byte string must be even length"
    );

    (0..hex_bytes.len())
        .step_by(2)
        .map(|start| {
            u8::from_str_radix(&hex_bytes[start..start + 2], 16).unwrap_or_else(|error| {
                panic!("source object image {label} hex must parse: {error}")
            })
        })
        .collect()
}

fn decode_score_digit_sprites(name: &'static str, tsv: &str) -> Vec<EmbeddedSprite> {
    let mut rows = Vec::new();
    for (line_index, line) in tsv.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        rows.push(decode_score_digit_sprite_row(name, line_index + 1, line));
    }
    rows.sort_by_key(|(digit, _)| *digit);
    assert_eq!(rows.len(), 10, "{name} must define ten score digit sprites");
    for (expected, (digit, _)) in rows.iter().enumerate() {
        assert_eq!(
            usize::from(*digit),
            expected,
            "{name} score digit rows must cover 0 through 9 once"
        );
    }
    rows.into_iter().map(|(_, sprite)| sprite).collect()
}

fn decode_score_digit_sprite_row(
    name: &'static str,
    line_index: usize,
    line: &str,
) -> (u8, EmbeddedSprite) {
    let fields = line.split('\t').collect::<Vec<_>>();
    assert_eq!(
        fields.len(),
        7,
        "{name}:{line_index} score digit row must have seven TSV fields"
    );
    let digit = fields[1]
        .parse::<u8>()
        .unwrap_or_else(|error| panic!("{name}:{line_index} digit must parse: {error}"));
    assert!(
        digit < 10,
        "{name}:{line_index} digit must be in the range 0..=9"
    );
    let columns = fields[3]
        .parse::<u8>()
        .unwrap_or_else(|error| panic!("{name}:{line_index} width must parse: {error}"));
    let rows = fields[4]
        .parse::<u8>()
        .unwrap_or_else(|error| panic!("{name}:{line_index} height must parse: {error}"));
    let hex = fields[5].trim();
    let expected_hex_len = usize::from(columns) * usize::from(rows) * 2;
    assert_eq!(
        hex.len(),
        expected_hex_len,
        "{name}:{line_index} digit byte string must match the declared dimensions"
    );
    let mut bytes = Vec::with_capacity(expected_hex_len / 2);
    for pair in hex.as_bytes().chunks_exact(2) {
        let pair = std::str::from_utf8(pair)
            .unwrap_or_else(|error| panic!("{name}:{line_index} hex pair must parse: {error}"));
        bytes
            .push(u8::from_str_radix(pair, 16).unwrap_or_else(|error| {
                panic!("{name}:{line_index} hex byte must parse: {error}")
            }));
    }
    (
        digit,
        decode_score_digit_rgba(name, digit, columns, rows, &bytes),
    )
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

fn decode_message_glyph_sprites(name: &'static str, tsv: &str) -> Vec<(char, EmbeddedSprite)> {
    let mut rows = Vec::new();
    for (line_index, line) in tsv.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        rows.push(decode_message_glyph_sprite_row(name, line_index + 1, line));
    }
    assert_eq!(
        rows.len(),
        SpriteId::MESSAGE_GLYPHS.len(),
        "{name} must define the clean renderer message glyph set"
    );
    for (character, _) in &rows {
        assert!(
            SpriteId::message_glyph(*character).is_some(),
            "{name} glyph `{character}` must have a clean sprite mapping"
        );
    }
    rows
}

fn decode_message_glyph_sprite_row(
    name: &'static str,
    line_index: usize,
    line: &str,
) -> (char, EmbeddedSprite) {
    let fields = line.split('\t').collect::<Vec<_>>();
    assert_eq!(
        fields.len(),
        7,
        "{name}:{line_index} message glyph row must have seven TSV fields"
    );
    let character = match fields[1] {
        "SPACE" => ' ',
        value => {
            let mut chars = value.chars();
            let character = chars
                .next()
                .unwrap_or_else(|| panic!("{name}:{line_index} glyph character must not be empty"));
            assert!(
                chars.next().is_none(),
                "{name}:{line_index} glyph character must be one character"
            );
            character
        }
    };
    let width = fields[3]
        .parse::<u8>()
        .unwrap_or_else(|error| panic!("{name}:{line_index} glyph width must parse: {error}"));
    let height = fields[4]
        .parse::<u8>()
        .unwrap_or_else(|error| panic!("{name}:{line_index} glyph height must parse: {error}"));
    let hex = fields[5].trim();
    let expected_hex_len = usize::from(width) * usize::from(height) * 2;
    assert_eq!(
        hex.len(),
        expected_hex_len,
        "{name}:{line_index} glyph byte string must match the declared dimensions"
    );
    let mut bytes = Vec::with_capacity(expected_hex_len / 2);
    for pair in hex.as_bytes().chunks_exact(2) {
        let pair = std::str::from_utf8(pair)
            .unwrap_or_else(|error| panic!("{name}:{line_index} hex pair must parse: {error}"));
        bytes
            .push(u8::from_str_radix(pair, 16).unwrap_or_else(|error| {
                panic!("{name}:{line_index} hex byte must parse: {error}")
            }));
    }
    (
        character,
        decode_message_glyph_rgba(name, character, width, height, &bytes),
    )
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
        u32::from(SOURCE_DEFENDER_LOGO_COLUMNS) * 2,
        u32::from(SOURCE_DEFENDER_LOGO_ROWS),
    );
    let mut pixels = transparent_rgba_pixels(surface).unwrap_or_default();
    let palette = ObjectPicturePalette::defender_logo();

    for column in 0..usize::from(SOURCE_DEFENDER_LOGO_COLUMNS) {
        let source_column = column * usize::from(SOURCE_DEFENDER_LOGO_ROWS);
        for row in 0..usize::from(SOURCE_DEFENDER_LOGO_ROWS) {
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
        u32::from(SOURCE_ATTRACT_COPYRIGHT_COLUMNS) * 2,
        u32::from(SOURCE_ATTRACT_COPYRIGHT_ROWS),
    );
    let mut pixels = transparent_rgba_pixels(surface).unwrap_or_default();

    for column in 0..usize::from(SOURCE_ATTRACT_COPYRIGHT_COLUMNS) {
        let left_bits = SOURCE_ATTRACT_COPYRIGHT_BYTES[column * 2];
        let right_bits = SOURCE_ATTRACT_COPYRIGHT_BYTES[column * 2 + 1];
        for row in 0..usize::from(SOURCE_ATTRACT_COPYRIGHT_ROWS) {
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
        u32::from(SOURCE_ATTRACT_WILLIAMS_LOGO_COLUMNS) * 2,
        u32::from(SOURCE_ATTRACT_WILLIAMS_LOGO_ROWS),
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
    let mut seen = [false; SOURCE_ATTRACT_WILLIAMS_LOGO_PIXELS];
    let mut pointer = 0usize;
    let mut cursor = 0u16;

    while let Some(opcode) = SOURCE_ATTRACT_WILLIAMS_LOGO_TABLE.get(pointer).copied() {
        pointer += 1;
        if opcode > 0xAA {
            let complemented = !opcode;
            if complemented == 0 {
                continue;
            }
            if complemented.wrapping_sub(1) != 0 {
                break;
            }
            let Some(cursor_high) = SOURCE_ATTRACT_WILLIAMS_LOGO_TABLE.get(pointer).copied() else {
                break;
            };
            let Some(cursor_low) = SOURCE_ATTRACT_WILLIAMS_LOGO_TABLE.get(pointer + 1).copied()
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
    let index = usize::from(pixel_y) * usize::from(SOURCE_ATTRACT_WILLIAMS_LOGO_COLUMNS) * 2
        + usize::from(pixel_x);
    if !seen[index] {
        seen[index] = true;
        path.push([pixel_x, pixel_y]);
    }
}

fn source_attract_williams_logo_pixel(cursor: u16) -> Option<[u8; 2]> {
    let [x, y] = cursor.to_be_bytes();
    let column = x >> 1;
    let relative_column = column.checked_sub(SOURCE_ATTRACT_WILLIAMS_LOGO_FIRST_COLUMN)?;
    let relative_row = y.checked_sub(SOURCE_ATTRACT_WILLIAMS_LOGO_FIRST_ROW)?;
    if relative_column >= SOURCE_ATTRACT_WILLIAMS_LOGO_COLUMNS
        || relative_row >= SOURCE_ATTRACT_WILLIAMS_LOGO_ROWS
    {
        return None;
    }

    let pixel_x = usize::from(relative_column) * 2 + usize::from(x & 1);
    Some([
        u8::try_from(pixel_x).expect("Williams logo pixel x fits in u8"),
        relative_row,
    ])
}

fn expand_source_defender_logo_bytes() -> [u8; SOURCE_DEFENDER_LOGO_BYTES] {
    let mut output = [0; SOURCE_DEFENDER_LOGO_BYTES];
    let mut cursor = 0usize;
    let mut length = 0u8;
    let mut right_pixel_next = false;

    for byte in SOURCE_DEFENDER_LOGO_COMPRESSED {
        for nibble in [byte >> 4, byte & 0x0F] {
            if nibble & 0x0C == 0 {
                length = nibble.wrapping_add(length).wrapping_shl(2);
                continue;
            }

            length = (nibble & 0x03).wrapping_add(length);
            let color = source_defender_logo_color_byte(nibble);
            if cursor >= SOURCE_DEFENDER_LOGO_BYTES {
                cursor = cursor + 1 - SOURCE_DEFENDER_LOGO_BYTES;
            }

            if right_pixel_next {
                output[cursor] = (output[cursor] & 0xF0) | (color & 0x0F);
                cursor += usize::from(SOURCE_DEFENDER_LOGO_ROWS);
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

                cursor += usize::from(SOURCE_DEFENDER_LOGO_ROWS);
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
    const SOURCE_APPEARANCE_FINAL_TICK: u8 = 0x2E;

    let source = expand_source_defender_logo_bytes();
    let mut pixels = BTreeMap::new();
    let size = i32::from(SOURCE_APPEARANCE_FINAL_TICK.saturating_sub(appearance_tick)).max(1);
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
    source: &[u8; SOURCE_DEFENDER_LOGO_BYTES],
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
                target_x_byte * i32::from(SOURCE_SCREEN_COLUMN_PIXELS_U8) + dx,
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
            let native_x = target_x_byte * i32::from(SOURCE_SCREEN_COLUMN_PIXELS_U8) + dx;
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
    source: &[u8; SOURCE_DEFENDER_LOGO_BYTES],
    native_x: i32,
    native_y: i32,
) -> Option<u8> {
    if native_x < 0
        || native_y < 0
        || native_x >= i32::from(SOURCE_DEFENDER_LOGO_COLUMNS) * 2
        || native_y >= i32::from(SOURCE_DEFENDER_LOGO_ROWS)
    {
        return None;
    }

    let byte_column = usize::try_from(native_x / 2).ok()?;
    let row = usize::try_from(native_y).ok()?;
    let packed = source[byte_column * usize::from(SOURCE_DEFENDER_LOGO_ROWS) + row];
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
        SOURCE_WILLIAMS_RED_GREEN_LEVELS[usize::from(value & 0x07)],
        SOURCE_WILLIAMS_RED_GREEN_LEVELS[usize::from((value >> 3) & 0x07)],
        SOURCE_WILLIAMS_BLUE_LEVELS[usize::from((value >> 6) & 0x03)],
        255,
    ]
}

fn source_object_cycle_color(phase: usize) -> [u8; 4] {
    pseudo_color_rgba(SOURCE_OBJECT_COLOR_TABLE[phase % (SOURCE_OBJECT_COLOR_TABLE.len() - 1)])
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
        SpriteAssetSource {
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
    source_region: SpriteAssetSource,
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
    source_region: SpriteAssetSource,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteResource {
    pub colors: Vec<Color>,
}

impl PaletteResource {
    pub fn defender_default() -> Self {
        Self {
            colors: vec![
                Color {
                    rgba: [0, 0, 0, 255],
                },
                Color::WHITE,
                Color {
                    rgba: [217, 81, 255, 255],
                },
                Color {
                    rgba: [38, 174, 0, 255],
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontAtlas {
    pub glyph_size: [u32; 2],
    pub glyph_count: u16,
}

impl Default for FontAtlas {
    fn default() -> Self {
        Self {
            glyph_size: [8, 8],
            glyph_count: 96,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeRendererResources {
    pub atlas: TextureAtlas,
    pub palette: PaletteResource,
    pub font: FontAtlas,
    pub pipelines: BTreeSet<NativeRenderPipeline>,
}

impl Default for NativeRendererResources {
    fn default() -> Self {
        let pipelines = [
            NativeRenderPipeline::TemporaryRaster,
            NativeRenderPipeline::Terrain,
            NativeRenderPipeline::Starfield,
            NativeRenderPipeline::Sprites,
            NativeRenderPipeline::Projectiles,
            NativeRenderPipeline::Explosions,
            NativeRenderPipeline::HudText,
            NativeRenderPipeline::DebugOverlay,
        ]
        .into_iter()
        .collect();

        Self {
            atlas: TextureAtlas::default_sprites(),
            palette: PaletteResource::defender_default(),
            font: FontAtlas::default(),
            pipelines,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneRasterUpload {
    pub surface: SurfaceSize,
    pub byte_len: usize,
    pub visual_signature: Option<u32>,
    pub non_blank: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteQuadVertex {
    pub unit_position: [f32; 2],
    pub unit_uv: [f32; 2],
}

impl SpriteQuadVertex {
    pub const FLOAT_COMPONENTS: usize = 4;
    pub const BYTE_SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
    pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        5 => Float32x2,
        6 => Float32x2,
    ];

    pub const fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::BYTE_SIZE,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VERTEX_ATTRIBUTES,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

const SPRITE_QUAD_VERTICES: [SpriteQuadVertex; 4] = [
    SpriteQuadVertex {
        unit_position: [0.0, 0.0],
        unit_uv: [0.0, 0.0],
    },
    SpriteQuadVertex {
        unit_position: [1.0, 0.0],
        unit_uv: [1.0, 0.0],
    },
    SpriteQuadVertex {
        unit_position: [0.0, 1.0],
        unit_uv: [0.0, 1.0],
    },
    SpriteQuadVertex {
        unit_position: [1.0, 1.0],
        unit_uv: [1.0, 1.0],
    },
];

const SPRITE_QUAD_INDICES: [u16; 6] = [0, 2, 1, 2, 3, 1];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteQuadGeometry;

impl SpriteQuadGeometry {
    pub const VERTICES: [SpriteQuadVertex; 4] = SPRITE_QUAD_VERTICES;
    pub const INDICES: [u16; 6] = SPRITE_QUAD_INDICES;
    pub const VERTEX_COUNT: u32 = SPRITE_QUAD_VERTICES.len() as u32;
    pub const INDEX_COUNT: u32 = SPRITE_QUAD_INDICES.len() as u32;
    pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

    pub const fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        SpriteQuadVertex::vertex_buffer_layout()
    }

    pub fn vertices() -> &'static [SpriteQuadVertex] {
        &SPRITE_QUAD_VERTICES
    }

    pub fn indices() -> &'static [u16] {
        &SPRITE_QUAD_INDICES
    }

    pub fn vertex_upload_bytes() -> &'static [u8] {
        bytemuck::cast_slice(&SPRITE_QUAD_VERTICES)
    }

    pub fn index_upload_bytes() -> &'static [u8] {
        bytemuck::cast_slice(&SPRITE_QUAD_INDICES)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteDrawInstance {
    pub sprite: SpriteId,
    pub atlas_origin: [u32; 2],
    pub atlas_size: [u32; 2],
    pub layer: RenderLayer,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub tint: Color,
}

impl SpriteDrawInstance {
    fn from_sprite(sprite: SceneSprite, region: AtlasRegion) -> Self {
        Self {
            sprite: sprite.sprite,
            atlas_origin: region.origin,
            atlas_size: region.size,
            layer: sprite.layer,
            position: sprite.position,
            size: sprite.size,
            tint: sprite.tint,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstanceBufferRecord {
    pub scene_origin: [f32; 2],
    pub scene_size: [f32; 2],
    pub atlas_uv_origin: [f32; 2],
    pub atlas_uv_size: [f32; 2],
    pub tint: [f32; 4],
}

impl SpriteInstanceBufferRecord {
    pub const FLOAT_COMPONENTS: usize = 12;
    pub const BYTE_SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
    pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x2,
        4 => Float32x4,
    ];

    pub fn from_instance(instance: SpriteDrawInstance, atlas_surface: SurfaceSize) -> Option<Self> {
        if atlas_surface.is_empty() {
            return None;
        }

        Some(Self {
            scene_origin: instance.position,
            scene_size: instance.size,
            atlas_uv_origin: [
                instance.atlas_origin[0] as f32 / atlas_surface.width as f32,
                instance.atlas_origin[1] as f32 / atlas_surface.height as f32,
            ],
            atlas_uv_size: [
                instance.atlas_size[0] as f32 / atlas_surface.width as f32,
                instance.atlas_size[1] as f32 / atlas_surface.height as f32,
            ],
            tint: instance.tint.to_normalized_rgba(),
        })
    }

    pub const fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::BYTE_SIZE,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::VERTEX_ATTRIBUTES,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteDrawBatch {
    pub pipeline: NativeRenderPipeline,
    pub layer: RenderLayer,
    pub instances: Vec<SpriteDrawInstance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteInstanceBuffer {
    pub pipeline: NativeRenderPipeline,
    pub layer: RenderLayer,
    pub records: Vec<SpriteInstanceBufferRecord>,
}

impl SpriteInstanceBuffer {
    fn from_batch(batch: &SpriteDrawBatch, atlas_surface: SurfaceSize) -> Option<Self> {
        let records = batch
            .instances
            .iter()
            .copied()
            .filter_map(|instance| {
                SpriteInstanceBufferRecord::from_instance(instance, atlas_surface)
            })
            .collect::<Vec<_>>();

        if records.is_empty() {
            return None;
        }

        Some(Self {
            pipeline: batch.pipeline,
            layer: batch.layer,
            records,
        })
    }

    pub fn upload_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.records)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteInstanceUpload {
    pub records: Vec<SpriteInstanceBufferRecord>,
}

impl SpriteInstanceUpload {
    fn from_instance_buffers(buffers: &[SpriteInstanceBuffer]) -> Option<Self> {
        let records = buffers
            .iter()
            .flat_map(|buffer| buffer.records.iter().copied())
            .collect::<Vec<_>>();

        if records.is_empty() {
            return None;
        }

        Some(Self { records })
    }

    pub fn instance_count(&self) -> usize {
        self.records.len()
    }

    pub fn byte_len(&self) -> wgpu::BufferAddress {
        self.upload_bytes().len() as wgpu::BufferAddress
    }

    pub fn upload_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.records)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteBufferRole {
    QuadVertices,
    QuadIndices,
    Instances,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteBufferUpload {
    pub role: SpriteBufferRole,
    pub label: &'static str,
    pub usage: wgpu::BufferUsages,
    pub byte_len: wgpu::BufferAddress,
    pub bytes: Vec<u8>,
}

impl SpriteBufferUpload {
    fn quad_vertices() -> Self {
        let bytes = SpriteQuadGeometry::vertex_upload_bytes().to_vec();
        Self {
            role: SpriteBufferRole::QuadVertices,
            label: "defender.sprite.quad.vertices",
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            byte_len: bytes.len() as wgpu::BufferAddress,
            bytes,
        }
    }

    fn quad_indices() -> Self {
        let bytes = SpriteQuadGeometry::index_upload_bytes().to_vec();
        Self {
            role: SpriteBufferRole::QuadIndices,
            label: "defender.sprite.quad.indices",
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            byte_len: bytes.len() as wgpu::BufferAddress,
            bytes,
        }
    }

    fn instances(upload: &SpriteInstanceUpload) -> Self {
        let bytes = upload.upload_bytes().to_vec();
        Self {
            role: SpriteBufferRole::Instances,
            label: "defender.sprite.instances",
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            byte_len: bytes.len() as wgpu::BufferAddress,
            bytes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteBufferUploadPlan {
    pub quad_vertices: SpriteBufferUpload,
    pub quad_indices: SpriteBufferUpload,
    pub instances: SpriteBufferUpload,
}

impl SpriteBufferUploadPlan {
    fn from_instance_upload(upload: &SpriteInstanceUpload) -> Self {
        Self {
            quad_vertices: SpriteBufferUpload::quad_vertices(),
            quad_indices: SpriteBufferUpload::quad_indices(),
            instances: SpriteBufferUpload::instances(upload),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteVertexBufferBinding {
    pub role: SpriteBufferRole,
    pub slot: u32,
    pub byte_offset: wgpu::BufferAddress,
    pub byte_len: wgpu::BufferAddress,
}

impl SpriteVertexBufferBinding {
    pub const QUAD_VERTEX_SLOT: u32 = 0;
    pub const INSTANCE_SLOT: u32 = 1;

    fn quad_vertices(upload: &SpriteBufferUpload) -> Self {
        Self {
            role: upload.role,
            slot: Self::QUAD_VERTEX_SLOT,
            byte_offset: 0,
            byte_len: upload.byte_len,
        }
    }

    fn instances(upload: &SpriteBufferUpload) -> Self {
        Self {
            role: upload.role,
            slot: Self::INSTANCE_SLOT,
            byte_offset: 0,
            byte_len: upload.byte_len,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteIndexBufferBinding {
    pub role: SpriteBufferRole,
    pub index_format: wgpu::IndexFormat,
    pub byte_offset: wgpu::BufferAddress,
    pub byte_len: wgpu::BufferAddress,
}

impl SpriteIndexBufferBinding {
    fn quad_indices(upload: &SpriteBufferUpload) -> Self {
        Self {
            role: upload.role,
            index_format: SpriteQuadGeometry::INDEX_FORMAT,
            byte_offset: 0,
            byte_len: upload.byte_len,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteRenderPassDraw {
    pub pipeline: NativeRenderPipeline,
    pub layer: RenderLayer,
    pub indices: std::ops::Range<u32>,
    pub base_vertex: i32,
    pub instances: std::ops::Range<u32>,
    pub instance_buffer_byte_offset: wgpu::BufferAddress,
    pub instance_buffer_byte_len: wgpu::BufferAddress,
}

impl SpriteRenderPassDraw {
    fn from_command(command: SpriteDrawCommand) -> Self {
        Self {
            pipeline: command.pipeline,
            layer: command.layer,
            indices: command.first_index..command.first_index + command.index_count,
            base_vertex: command.base_vertex,
            instances: command.first_instance..command.first_instance + command.instance_count,
            instance_buffer_byte_offset: command.instance_buffer_byte_offset,
            instance_buffer_byte_len: command.instance_buffer_byte_len,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteRenderPassPlan {
    pub quad_vertices: SpriteVertexBufferBinding,
    pub instances: SpriteVertexBufferBinding,
    pub indices: SpriteIndexBufferBinding,
    pub draws: Vec<SpriteRenderPassDraw>,
}

impl SpriteRenderPassPlan {
    fn from_uploads_and_commands(
        uploads: &SpriteBufferUploadPlan,
        commands: &[SpriteDrawCommand],
    ) -> Option<Self> {
        if commands.is_empty() {
            return None;
        }

        Some(Self {
            quad_vertices: SpriteVertexBufferBinding::quad_vertices(&uploads.quad_vertices),
            instances: SpriteVertexBufferBinding::instances(&uploads.instances),
            indices: SpriteIndexBufferBinding::quad_indices(&uploads.quad_indices),
            draws: commands
                .iter()
                .copied()
                .map(SpriteRenderPassDraw::from_command)
                .collect(),
        })
    }

    pub fn draw_count(&self) -> usize {
        self.draws.len()
    }

    pub fn instance_count(&self) -> u32 {
        self.draws
            .iter()
            .map(|draw| draw.instances.end - draw.instances.start)
            .sum()
    }
}

const SPRITE_SHADER_SOURCE: &str = r#"
struct SceneProjection {
    scale: vec2<f32>,
    translate: vec2<f32>,
};

@group(0) @binding(0) var<uniform> scene_projection: SceneProjection;
@group(1) @binding(0) var sprite_atlas: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

struct SpriteInstance {
    @location(0) scene_origin: vec2<f32>,
    @location(1) scene_size: vec2<f32>,
    @location(2) atlas_uv_origin: vec2<f32>,
    @location(3) atlas_uv_size: vec2<f32>,
    @location(4) tint: vec4<f32>,
};

struct SpriteVertex {
    @location(5) unit_position: vec2<f32>,
    @location(6) unit_uv: vec2<f32>,
};

struct SpriteVertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) atlas_uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn sprite_vs(instance: SpriteInstance, vertex: SpriteVertex) -> SpriteVertexOut {
    let scene_position = instance.scene_origin + vertex.unit_position * instance.scene_size;

    var out: SpriteVertexOut;
    out.position = vec4<f32>(
        scene_position * scene_projection.scale + scene_projection.translate,
        0.0,
        1.0,
    );
    out.atlas_uv = instance.atlas_uv_origin + vertex.unit_uv * instance.atlas_uv_size;
    out.tint = instance.tint;
    return out;
}

@fragment
fn sprite_fs(in: SpriteVertexOut) -> @location(0) vec4<f32> {
    return textureSample(sprite_atlas, sprite_sampler, in.atlas_uv) * in.tint;
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteShaderPlan {
    pub label: &'static str,
    pub source: &'static str,
    pub vertex_entry: &'static str,
    pub fragment_entry: &'static str,
}

impl SpriteShaderPlan {
    pub fn shader_module_descriptor(&self) -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: Some(self.label),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(self.source)),
        }
    }
}

impl Default for SpriteShaderPlan {
    fn default() -> Self {
        Self {
            label: "defender.sprite.shader",
            source: SPRITE_SHADER_SOURCE,
            vertex_entry: "sprite_vs",
            fragment_entry: "sprite_fs",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteVertexBufferLayoutPlan {
    pub role: SpriteBufferRole,
    pub slot: u32,
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: &'static [wgpu::VertexAttribute],
}

impl SpriteVertexBufferLayoutPlan {
    fn quad_vertices() -> Self {
        let layout = SpriteQuadGeometry::vertex_buffer_layout();
        Self {
            role: SpriteBufferRole::QuadVertices,
            slot: SpriteVertexBufferBinding::QUAD_VERTEX_SLOT,
            array_stride: layout.array_stride,
            step_mode: layout.step_mode,
            attributes: layout.attributes,
        }
    }

    fn instances() -> Self {
        let layout = SpriteInstanceBufferRecord::vertex_buffer_layout();
        Self {
            role: SpriteBufferRole::Instances,
            slot: SpriteVertexBufferBinding::INSTANCE_SLOT,
            array_stride: layout.array_stride,
            step_mode: layout.step_mode,
            attributes: layout.attributes,
        }
    }

    pub const fn vertex_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: self.step_mode,
            attributes: self.attributes,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpritePipelinePlan {
    pub label: &'static str,
    pub shader: SpriteShaderPlan,
    pub vertex_buffers: [SpriteVertexBufferLayoutPlan; 2],
    pub primitive: wgpu::PrimitiveState,
    pub color_target: wgpu::ColorTargetState,
    pub multisample: wgpu::MultisampleState,
}

impl SpritePipelinePlan {
    fn for_settings(settings: GpuRendererSettings) -> Self {
        Self {
            label: "defender.sprite.pipeline",
            shader: SpriteShaderPlan::default(),
            vertex_buffers: [
                SpriteVertexBufferLayoutPlan::quad_vertices(),
                SpriteVertexBufferLayoutPlan::instances(),
            ],
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..wgpu::PrimitiveState::default()
            },
            color_target: wgpu::ColorTargetState {
                format: settings.texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            },
            multisample: wgpu::MultisampleState::default(),
        }
    }

    pub fn vertex_buffer_layouts(&self) -> [wgpu::VertexBufferLayout<'static>; 2] {
        self.vertex_buffers
            .map(|buffer| buffer.vertex_buffer_layout())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteBindGroupRole {
    SceneProjection,
    SpriteAtlas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteResourceBindingRole {
    SceneProjectionUniform,
    SpriteAtlasTexture,
    SpriteAtlasSampler,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneProjectionUniformUpload {
    pub role: SpriteResourceBindingRole,
    pub label: &'static str,
    pub usage: wgpu::BufferUsages,
    pub byte_len: wgpu::BufferAddress,
    pub bytes: Vec<u8>,
}

impl SceneProjectionUniformUpload {
    fn from_projection(projection: SceneProjectionUniforms) -> Self {
        let bytes = projection.as_bytes().to_vec();
        Self {
            role: SpriteResourceBindingRole::SceneProjectionUniform,
            label: "defender.sprite.scene_projection.uniform",
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            byte_len: bytes.len() as wgpu::BufferAddress,
            bytes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteBindGroupLayoutPlan {
    pub role: SpriteBindGroupRole,
    pub label: &'static str,
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl SpriteBindGroupLayoutPlan {
    pub const SCENE_PROJECTION_GROUP: u32 = 0;
    pub const SPRITE_ATLAS_GROUP: u32 = 1;
    pub const SCENE_PROJECTION_BINDING: u32 = 0;
    pub const ATLAS_TEXTURE_BINDING: u32 = 0;
    pub const ATLAS_SAMPLER_BINDING: u32 = 1;

    fn scene_projection() -> Self {
        Self {
            role: SpriteBindGroupRole::SceneProjection,
            label: "defender.sprite.scene_projection.bind_group_layout",
            entries: vec![wgpu::BindGroupLayoutEntry {
                binding: Self::SCENE_PROJECTION_BINDING,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(SceneProjectionUniforms::BYTE_SIZE),
                },
                count: None,
            }],
        }
    }

    fn sprite_atlas() -> Self {
        Self {
            role: SpriteBindGroupRole::SpriteAtlas,
            label: "defender.sprite.atlas.bind_group_layout",
            entries: vec![
                wgpu::BindGroupLayoutEntry {
                    binding: Self::ATLAS_TEXTURE_BINDING,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::ATLAS_SAMPLER_BINDING,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        }
    }

    pub const fn group_index(&self) -> u32 {
        match self.role {
            SpriteBindGroupRole::SceneProjection => Self::SCENE_PROJECTION_GROUP,
            SpriteBindGroupRole::SpriteAtlas => Self::SPRITE_ATLAS_GROUP,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteTextureBindingPlan {
    pub role: SpriteResourceBindingRole,
    pub label: &'static str,
    pub binding: u32,
    pub visibility: wgpu::ShaderStages,
    pub sample_type: wgpu::TextureSampleType,
    pub view_dimension: wgpu::TextureViewDimension,
    pub multisampled: bool,
    pub surface: SurfaceSize,
}

impl SpriteTextureBindingPlan {
    fn atlas(surface: SurfaceSize) -> Self {
        Self {
            role: SpriteResourceBindingRole::SpriteAtlasTexture,
            label: "defender.sprite.atlas.texture_view",
            binding: SpriteBindGroupLayoutPlan::ATLAS_TEXTURE_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
            surface,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteSamplerBindingPlan {
    pub role: SpriteResourceBindingRole,
    pub label: &'static str,
    pub binding: u32,
    pub visibility: wgpu::ShaderStages,
    pub sampler_binding: wgpu::SamplerBindingType,
}

impl SpriteSamplerBindingPlan {
    fn atlas() -> Self {
        Self {
            role: SpriteResourceBindingRole::SpriteAtlasSampler,
            label: "defender.sprite.atlas.sampler",
            binding: SpriteBindGroupLayoutPlan::ATLAS_SAMPLER_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            sampler_binding: wgpu::SamplerBindingType::Filtering,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteAtlasTextureUpload {
    pub role: SpriteResourceBindingRole,
    pub label: &'static str,
    pub usage: wgpu::TextureUsages,
    pub format: wgpu::TextureFormat,
    pub dimension: wgpu::TextureDimension,
    pub surface: SurfaceSize,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub depth_or_array_layers: u32,
    pub bytes_per_row: u32,
    pub rows_per_image: u32,
    pub byte_len: usize,
    pub bytes: Vec<u8>,
    pub non_blank: bool,
}

impl SpriteAtlasTextureUpload {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    pub const MIP_LEVEL_COUNT: u32 = 1;
    pub const SAMPLE_COUNT: u32 = 1;
    pub const DEPTH_OR_ARRAY_LAYERS: u32 = 1;

    fn from_atlas(atlas: &TextureAtlas) -> Option<Self> {
        if atlas.surface.is_empty() {
            return None;
        }
        if atlas.pixels.is_empty() || atlas.pixels.len() != atlas.surface.rgba_len()? {
            return None;
        }

        Some(Self {
            role: SpriteResourceBindingRole::SpriteAtlasTexture,
            label: "defender.sprite.atlas.texture",
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            format: Self::FORMAT,
            dimension: wgpu::TextureDimension::D2,
            surface: atlas.surface,
            mip_level_count: Self::MIP_LEVEL_COUNT,
            sample_count: Self::SAMPLE_COUNT,
            depth_or_array_layers: Self::DEPTH_OR_ARRAY_LAYERS,
            bytes_per_row: atlas.surface.width.checked_mul(4)?,
            rows_per_image: atlas.surface.height,
            byte_len: atlas.pixels.len(),
            bytes: atlas.pixels.clone(),
            non_blank: atlas.is_non_blank(),
        })
    }

    pub const fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.surface.width,
            height: self.surface.height,
            depth_or_array_layers: self.depth_or_array_layers,
        }
    }

    pub fn copy_layout(&self) -> wgpu::TexelCopyBufferLayout {
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(self.bytes_per_row),
            rows_per_image: Some(self.rows_per_image),
        }
    }

    pub fn texture_descriptor(&self) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some(self.label),
            size: self.extent(),
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension,
            format: self.format,
            usage: self.usage,
            view_formats: &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteResourceBindingPlan {
    pub atlas_upload: SpriteAtlasTextureUpload,
    pub projection_upload: SceneProjectionUniformUpload,
    pub projection_layout: SpriteBindGroupLayoutPlan,
    pub atlas_layout: SpriteBindGroupLayoutPlan,
    pub atlas_texture: SpriteTextureBindingPlan,
    pub atlas_sampler: SpriteSamplerBindingPlan,
}

impl SpriteResourceBindingPlan {
    pub const BIND_GROUP_COUNT: usize = 2;
    pub const BINDING_ENTRY_COUNT: usize = 3;

    fn from_projection_and_atlas(
        projection: SceneProjectionUniforms,
        atlas: &TextureAtlas,
    ) -> Option<Self> {
        let atlas_upload = SpriteAtlasTextureUpload::from_atlas(atlas)?;

        Some(Self {
            atlas_upload,
            projection_upload: SceneProjectionUniformUpload::from_projection(projection),
            projection_layout: SpriteBindGroupLayoutPlan::scene_projection(),
            atlas_layout: SpriteBindGroupLayoutPlan::sprite_atlas(),
            atlas_texture: SpriteTextureBindingPlan::atlas(atlas.surface),
            atlas_sampler: SpriteSamplerBindingPlan::atlas(),
        })
    }

    pub fn bind_group_count(&self) -> usize {
        Self::BIND_GROUP_COUNT
    }

    pub fn binding_entry_count(&self) -> usize {
        self.projection_layout.entries.len() + self.atlas_layout.entries.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpritePipelineLayoutBindGroup {
    pub role: SpriteBindGroupRole,
    pub group_index: u32,
    pub layout_label: &'static str,
    pub entry_count: usize,
}

impl SpritePipelineLayoutBindGroup {
    fn from_layout(layout: &SpriteBindGroupLayoutPlan) -> Self {
        Self {
            role: layout.role,
            group_index: layout.group_index(),
            layout_label: layout.label,
            entry_count: layout.entries.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpritePipelineLayoutPlan {
    pub label: &'static str,
    pub bind_groups: Vec<SpritePipelineLayoutBindGroup>,
    pub immediate_size: u32,
}

impl SpritePipelineLayoutPlan {
    pub const BIND_GROUP_COUNT: usize = SpriteResourceBindingPlan::BIND_GROUP_COUNT;
    pub const BINDING_ENTRY_COUNT: usize = SpriteResourceBindingPlan::BINDING_ENTRY_COUNT;
    pub const IMMEDIATE_SIZE: u32 = 0;

    fn from_resource_bindings(bindings: &SpriteResourceBindingPlan) -> Self {
        Self {
            label: "defender.sprite.pipeline_layout",
            bind_groups: vec![
                SpritePipelineLayoutBindGroup::from_layout(&bindings.projection_layout),
                SpritePipelineLayoutBindGroup::from_layout(&bindings.atlas_layout),
            ],
            immediate_size: Self::IMMEDIATE_SIZE,
        }
    }

    pub fn bind_group_count(&self) -> usize {
        self.bind_groups.len()
    }

    pub fn binding_entry_count(&self) -> usize {
        self.bind_groups
            .iter()
            .map(|bind_group| bind_group.entry_count)
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteRenderPipelineDescriptorPlan {
    pub label: &'static str,
    pub layout_label: &'static str,
    pub layout_bind_group_count: usize,
    pub immediate_size: u32,
    pub shader_label: &'static str,
    pub vertex_entry: &'static str,
    pub fragment_entry: &'static str,
    pub vertex_buffers: [SpriteVertexBufferLayoutPlan; 2],
    pub primitive: wgpu::PrimitiveState,
    pub color_target: wgpu::ColorTargetState,
    pub multisample: wgpu::MultisampleState,
}

impl SpriteRenderPipelineDescriptorPlan {
    pub const LAYOUT_BIND_GROUP_COUNT: usize = SpritePipelineLayoutPlan::BIND_GROUP_COUNT;
    pub const VERTEX_BUFFER_COUNT: usize = 2;
    pub const COLOR_TARGET_COUNT: usize = 1;

    fn from_pipeline_and_layout(
        pipeline: &SpritePipelinePlan,
        layout: &SpritePipelineLayoutPlan,
    ) -> Self {
        Self {
            label: pipeline.label,
            layout_label: layout.label,
            layout_bind_group_count: layout.bind_group_count(),
            immediate_size: layout.immediate_size,
            shader_label: pipeline.shader.label,
            vertex_entry: pipeline.shader.vertex_entry,
            fragment_entry: pipeline.shader.fragment_entry,
            vertex_buffers: pipeline.vertex_buffers,
            primitive: pipeline.primitive,
            color_target: pipeline.color_target.clone(),
            multisample: pipeline.multisample,
        }
    }

    pub fn vertex_buffer_layouts(&self) -> [wgpu::VertexBufferLayout<'static>; 2] {
        self.vertex_buffers
            .map(|buffer| buffer.vertex_buffer_layout())
    }

    pub fn layout_bind_group_count(&self) -> usize {
        self.layout_bind_group_count
    }

    pub fn vertex_buffer_count(&self) -> usize {
        self.vertex_buffers.len()
    }

    pub fn color_target_count(&self) -> usize {
        Self::COLOR_TARGET_COUNT
    }

    pub fn color_targets(&self) -> [Option<wgpu::ColorTargetState>; 1] {
        [Some(self.color_target.clone())]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpriteRenderPassEncoderCommand {
    SetPipeline {
        label: &'static str,
    },
    SetBindGroup {
        role: SpriteBindGroupRole,
        group_index: u32,
        layout_label: &'static str,
    },
    SetVertexBuffer {
        role: SpriteBufferRole,
        slot: u32,
        byte_offset: wgpu::BufferAddress,
        byte_len: wgpu::BufferAddress,
    },
    SetIndexBuffer {
        role: SpriteBufferRole,
        index_format: wgpu::IndexFormat,
        byte_offset: wgpu::BufferAddress,
        byte_len: wgpu::BufferAddress,
    },
    DrawIndexed {
        draw: SpriteRenderPassDraw,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteRenderPassEncoderPlan {
    pub label: &'static str,
    pub commands: Vec<SpriteRenderPassEncoderCommand>,
}

impl SpriteRenderPassEncoderPlan {
    pub const SET_PIPELINE_COMMAND_COUNT: usize = 1;
    pub const SET_BIND_GROUP_COMMAND_COUNT: usize = SpritePipelineLayoutPlan::BIND_GROUP_COUNT;
    pub const SET_VERTEX_BUFFER_COMMAND_COUNT: usize =
        SpriteRenderPipelineDescriptorPlan::VERTEX_BUFFER_COUNT;
    pub const SET_INDEX_BUFFER_COMMAND_COUNT: usize = 1;

    fn from_render_pass_layout_and_descriptor(
        render_pass: &SpriteRenderPassPlan,
        layout: &SpritePipelineLayoutPlan,
        descriptor: &SpriteRenderPipelineDescriptorPlan,
    ) -> Self {
        let mut commands =
            Vec::with_capacity(1 + layout.bind_group_count() + 3 + render_pass.draws.len());
        commands.push(SpriteRenderPassEncoderCommand::SetPipeline {
            label: descriptor.label,
        });
        commands.extend(layout.bind_groups.iter().map(|bind_group| {
            SpriteRenderPassEncoderCommand::SetBindGroup {
                role: bind_group.role,
                group_index: bind_group.group_index,
                layout_label: bind_group.layout_label,
            }
        }));
        commands.push(SpriteRenderPassEncoderCommand::SetVertexBuffer {
            role: render_pass.quad_vertices.role,
            slot: render_pass.quad_vertices.slot,
            byte_offset: render_pass.quad_vertices.byte_offset,
            byte_len: render_pass.quad_vertices.byte_len,
        });
        commands.push(SpriteRenderPassEncoderCommand::SetVertexBuffer {
            role: render_pass.instances.role,
            slot: render_pass.instances.slot,
            byte_offset: render_pass.instances.byte_offset,
            byte_len: render_pass.instances.byte_len,
        });
        commands.push(SpriteRenderPassEncoderCommand::SetIndexBuffer {
            role: render_pass.indices.role,
            index_format: render_pass.indices.index_format,
            byte_offset: render_pass.indices.byte_offset,
            byte_len: render_pass.indices.byte_len,
        });
        commands.extend(
            render_pass
                .draws
                .iter()
                .cloned()
                .map(|draw| SpriteRenderPassEncoderCommand::DrawIndexed { draw }),
        );

        Self {
            label: "defender.sprite.render_pass.encoder",
            commands,
        }
    }

    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    pub fn set_pipeline_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, SpriteRenderPassEncoderCommand::SetPipeline { .. }))
            .count()
    }

    pub fn set_bind_group_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| {
                matches!(command, SpriteRenderPassEncoderCommand::SetBindGroup { .. })
            })
            .count()
    }

    pub fn set_vertex_buffer_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    SpriteRenderPassEncoderCommand::SetVertexBuffer { .. }
                )
            })
            .count()
    }

    pub fn set_index_buffer_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    SpriteRenderPassEncoderCommand::SetIndexBuffer { .. }
                )
            })
            .count()
    }

    pub fn draw_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, SpriteRenderPassEncoderCommand::DrawIndexed { .. }))
            .count()
    }

    pub fn instance_count(&self) -> usize {
        self.commands
            .iter()
            .map(|command| match command {
                SpriteRenderPassEncoderCommand::DrawIndexed { draw } => {
                    (draw.instances.end - draw.instances.start) as usize
                }
                _ => 0,
            })
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WgpuFrameCommand {
    BeginRenderPass {
        clear_color: wgpu::Color,
    },
    SetViewport {
        viewport: WgpuViewportCommand,
    },
    UploadSceneProjection {
        byte_len: wgpu::BufferAddress,
    },
    UploadTemporaryRaster {
        upload: SceneRasterUpload,
    },
    ExecuteSpriteRenderPass {
        encoder_label: &'static str,
        command_count: usize,
        draw_count: usize,
        instance_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct WgpuFramePlan {
    pub label: &'static str,
    pub commands: Vec<WgpuFrameCommand>,
}

impl WgpuFramePlan {
    fn from_pass_raster_and_sprite_encoder(
        pass: &WgpuPassPlan,
        raster_upload: Option<SceneRasterUpload>,
        sprite_encoder: Option<&SpriteRenderPassEncoderPlan>,
    ) -> Self {
        let mut commands = Vec::new();
        commands.push(WgpuFrameCommand::BeginRenderPass {
            clear_color: pass.clear_color,
        });
        if let Some(viewport) = pass.viewport {
            commands.push(WgpuFrameCommand::SetViewport { viewport });
        }
        if let Some(projection) = pass.scene_projection {
            commands.push(WgpuFrameCommand::UploadSceneProjection {
                byte_len: projection.as_bytes().len() as wgpu::BufferAddress,
            });
        }
        if let Some(upload) = raster_upload {
            commands.push(WgpuFrameCommand::UploadTemporaryRaster { upload });
        }
        if let Some(encoder) = sprite_encoder {
            commands.push(WgpuFrameCommand::ExecuteSpriteRenderPass {
                encoder_label: encoder.label,
                command_count: encoder.command_count(),
                draw_count: encoder.draw_count(),
                instance_count: encoder.instance_count(),
            });
        }

        Self {
            label: "defender.frame.commands",
            commands,
        }
    }

    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    pub fn has_ordered_sprite_only_commands(&self) -> bool {
        matches!(
            self.commands.as_slice(),
            [
                WgpuFrameCommand::BeginRenderPass { .. },
                WgpuFrameCommand::SetViewport { .. },
                WgpuFrameCommand::UploadSceneProjection { .. },
                WgpuFrameCommand::ExecuteSpriteRenderPass { .. },
            ]
        )
    }

    pub fn sprite_pass_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, WgpuFrameCommand::ExecuteSpriteRenderPass { .. }))
            .count()
    }

    pub fn temporary_raster_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, WgpuFrameCommand::UploadTemporaryRaster { .. }))
            .count()
    }

    pub fn begin_render_pass_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, WgpuFrameCommand::BeginRenderPass { .. }))
            .count()
    }

    pub fn viewport_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, WgpuFrameCommand::SetViewport { .. }))
            .count()
    }

    pub fn scene_projection_upload_byte_len(&self) -> wgpu::BufferAddress {
        self.commands
            .iter()
            .map(|command| match command {
                WgpuFrameCommand::UploadSceneProjection { byte_len } => *byte_len,
                _ => 0,
            })
            .sum()
    }

    pub fn sprite_encoder_command_count(&self) -> usize {
        self.commands
            .iter()
            .map(|command| match command {
                WgpuFrameCommand::ExecuteSpriteRenderPass { command_count, .. } => *command_count,
                _ => 0,
            })
            .sum()
    }

    pub fn sprite_draw_count(&self) -> usize {
        self.commands
            .iter()
            .map(|command| match command {
                WgpuFrameCommand::ExecuteSpriteRenderPass { draw_count, .. } => *draw_count,
                _ => 0,
            })
            .sum()
    }

    pub fn sprite_instance_count(&self) -> usize {
        self.commands
            .iter()
            .map(|command| match command {
                WgpuFrameCommand::ExecuteSpriteRenderPass { instance_count, .. } => *instance_count,
                _ => 0,
            })
            .sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteDrawCommand {
    pub pipeline: NativeRenderPipeline,
    pub layer: RenderLayer,
    pub vertex_count: u32,
    pub index_count: u32,
    pub index_format: wgpu::IndexFormat,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
    pub instance_count: u32,
    pub vertex_buffer_byte_len: wgpu::BufferAddress,
    pub index_buffer_byte_len: wgpu::BufferAddress,
    pub instance_buffer_byte_offset: wgpu::BufferAddress,
    pub instance_buffer_byte_len: wgpu::BufferAddress,
}

impl SpriteDrawCommand {
    fn from_instance_buffer(buffer: &SpriteInstanceBuffer, first_instance: u32) -> Option<Self> {
        if buffer.records.is_empty() {
            return None;
        }

        let instance_count = u32::try_from(buffer.records.len()).ok()?;
        let instance_buffer_byte_offset =
            u64::from(first_instance) * SpriteInstanceBufferRecord::BYTE_SIZE;
        let instance_buffer_byte_len =
            u64::from(instance_count) * SpriteInstanceBufferRecord::BYTE_SIZE;

        Some(Self {
            pipeline: buffer.pipeline,
            layer: buffer.layer,
            vertex_count: SpriteQuadGeometry::VERTEX_COUNT,
            index_count: SpriteQuadGeometry::INDEX_COUNT,
            index_format: SpriteQuadGeometry::INDEX_FORMAT,
            first_index: 0,
            base_vertex: 0,
            first_instance,
            instance_count,
            vertex_buffer_byte_len: SpriteQuadGeometry::vertex_upload_bytes().len()
                as wgpu::BufferAddress,
            index_buffer_byte_len: SpriteQuadGeometry::index_upload_bytes().len()
                as wgpu::BufferAddress,
            instance_buffer_byte_offset,
            instance_buffer_byte_len,
        })
    }
}

fn sprite_draw_commands_from_instance_buffers(
    buffers: &[SpriteInstanceBuffer],
) -> Vec<SpriteDrawCommand> {
    let mut first_instance = 0;
    let mut commands = Vec::new();

    for buffer in buffers {
        let Some(command) = SpriteDrawCommand::from_instance_buffer(buffer, first_instance) else {
            continue;
        };
        commands.push(command);
        first_instance += command.instance_count;
    }

    commands
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneDrawPlan {
    pub frame: u64,
    pub surface: SurfaceSize,
    pub viewport: ViewportLayout,
    pub gpu_pass: WgpuPassPlan,
    pub frame_plan: WgpuFramePlan,
    pub pipelines: Vec<NativeRenderPipeline>,
    pub sprite_instances: usize,
    pub missing_sprite_regions: usize,
    pub sprite_batches: Vec<SpriteDrawBatch>,
    pub sprite_instance_buffers: Vec<SpriteInstanceBuffer>,
    pub sprite_instance_upload: Option<SpriteInstanceUpload>,
    pub sprite_buffer_uploads: Option<SpriteBufferUploadPlan>,
    pub sprite_draw_commands: Vec<SpriteDrawCommand>,
    pub sprite_render_pass: Option<SpriteRenderPassPlan>,
    pub sprite_pipeline: Option<SpritePipelinePlan>,
    pub sprite_resource_bindings: Option<SpriteResourceBindingPlan>,
    pub sprite_pipeline_layout: Option<SpritePipelineLayoutPlan>,
    pub sprite_render_pipeline_descriptor: Option<SpriteRenderPipelineDescriptorPlan>,
    pub sprite_render_pass_encoder: Option<SpriteRenderPassEncoderPlan>,
    pub layer_counts: RenderLayerCounts,
    pub raster_upload: Option<SceneRasterUpload>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeSceneRenderer {
    pub resources: NativeRendererResources,
    pub settings: GpuRendererSettings,
}

impl NativeSceneRenderer {
    pub fn new(resources: NativeRendererResources) -> Self {
        Self {
            resources,
            settings: GpuRendererSettings::default(),
        }
    }

    pub fn with_settings(
        resources: NativeRendererResources,
        settings: GpuRendererSettings,
    ) -> Self {
        Self {
            resources,
            settings,
        }
    }

    pub fn prepare(&self, scene: &RenderScene) -> SceneDrawPlan {
        self.prepare_for_target(scene, scene.surface)
    }

    pub fn prepare_for_target(&self, scene: &RenderScene, target: SurfaceSize) -> SceneDrawPlan {
        let mut requested = BTreeSet::new();
        if scene.raster.is_some() {
            requested.insert(NativeRenderPipeline::TemporaryRaster);
        }

        let mut layer_counts = RenderLayerCounts::default();
        let mut missing_sprite_regions = 0;
        let mut sprite_batches = Vec::new();
        for sprite in &scene.sprites {
            layer_counts.add(sprite.layer);
            let pipeline = pipeline_for_layer(sprite.layer);
            let Some(region) = self.resources.atlas.region(sprite.sprite) else {
                missing_sprite_regions += 1;
                continue;
            };
            if self.resources.pipelines.contains(&pipeline) {
                requested.insert(pipeline);
                push_sprite_instance(
                    &mut sprite_batches,
                    pipeline,
                    sprite.layer,
                    SpriteDrawInstance::from_sprite(*sprite, region),
                );
            }
        }

        let pipelines = requested
            .into_iter()
            .filter(|pipeline| self.resources.pipelines.contains(pipeline))
            .collect();
        let sprite_instances = sprite_batches
            .iter()
            .map(|batch: &SpriteDrawBatch| batch.instances.len())
            .sum();
        let sprite_instance_buffers = sprite_batches
            .iter()
            .filter_map(|batch| {
                SpriteInstanceBuffer::from_batch(batch, self.resources.atlas.surface)
            })
            .collect::<Vec<_>>();
        let sprite_instance_upload =
            SpriteInstanceUpload::from_instance_buffers(&sprite_instance_buffers);
        let sprite_buffer_uploads = sprite_instance_upload
            .as_ref()
            .map(SpriteBufferUploadPlan::from_instance_upload);
        let sprite_draw_commands =
            sprite_draw_commands_from_instance_buffers(&sprite_instance_buffers);
        let sprite_render_pass = sprite_buffer_uploads.as_ref().and_then(|uploads| {
            SpriteRenderPassPlan::from_uploads_and_commands(uploads, &sprite_draw_commands)
        });
        let sprite_pipeline = sprite_render_pass
            .as_ref()
            .map(|_| SpritePipelinePlan::for_settings(self.settings));
        let viewport = ViewportLayout::fit(scene.surface, target);
        let gpu_pass = WgpuPassPlan::from_scene(scene, viewport);
        let sprite_resource_bindings = sprite_pipeline.as_ref().and_then(|_| {
            gpu_pass.scene_projection.and_then(|projection| {
                SpriteResourceBindingPlan::from_projection_and_atlas(
                    projection,
                    &self.resources.atlas,
                )
            })
        });
        let sprite_pipeline_layout = match (
            sprite_pipeline.as_ref(),
            sprite_resource_bindings.as_ref(),
            gpu_pass.viewport,
        ) {
            (Some(_), Some(bindings), Some(_)) => {
                Some(SpritePipelineLayoutPlan::from_resource_bindings(bindings))
            }
            _ => None,
        };
        let sprite_render_pipeline_descriptor = match (
            sprite_render_pass.as_ref(),
            sprite_pipeline.as_ref(),
            sprite_resource_bindings.as_ref(),
            sprite_pipeline_layout.as_ref(),
            gpu_pass.viewport,
        ) {
            (Some(_), Some(pipeline), Some(_), Some(layout), Some(_)) => {
                Some(SpriteRenderPipelineDescriptorPlan::from_pipeline_and_layout(pipeline, layout))
            }
            _ => None,
        };
        let sprite_render_pass_encoder = match (
            sprite_render_pass.as_ref(),
            sprite_resource_bindings.as_ref(),
            sprite_pipeline_layout.as_ref(),
            sprite_render_pipeline_descriptor.as_ref(),
            gpu_pass.viewport,
        ) {
            (Some(render_pass), Some(_), Some(layout), Some(descriptor), Some(_)) => Some(
                SpriteRenderPassEncoderPlan::from_render_pass_layout_and_descriptor(
                    render_pass,
                    layout,
                    descriptor,
                ),
            ),
            _ => None,
        };
        let raster_upload = scene.raster.as_ref().map(|raster| SceneRasterUpload {
            surface: raster.surface,
            byte_len: raster.pixels.len(),
            visual_signature: scene.visual_signature,
            non_blank: raster.is_non_blank(),
        });
        let frame_plan = WgpuFramePlan::from_pass_raster_and_sprite_encoder(
            &gpu_pass,
            raster_upload,
            sprite_render_pass_encoder.as_ref(),
        );

        SceneDrawPlan {
            frame: scene.frame,
            surface: scene.surface,
            viewport,
            gpu_pass,
            frame_plan,
            pipelines,
            sprite_instances,
            missing_sprite_regions,
            sprite_batches,
            sprite_instance_buffers,
            sprite_instance_upload,
            sprite_buffer_uploads,
            sprite_draw_commands,
            sprite_render_pass,
            sprite_pipeline,
            sprite_resource_bindings,
            sprite_pipeline_layout,
            sprite_render_pipeline_descriptor,
            sprite_render_pass_encoder,
            layer_counts,
            raster_upload,
        }
    }
}

fn push_sprite_instance(
    batches: &mut Vec<SpriteDrawBatch>,
    pipeline: NativeRenderPipeline,
    layer: RenderLayer,
    instance: SpriteDrawInstance,
) {
    if let Some(batch) = batches
        .iter_mut()
        .find(|batch| batch.pipeline == pipeline && batch.layer == layer)
    {
        batch.instances.push(instance);
        return;
    }

    batches.push(SpriteDrawBatch {
        pipeline,
        layer,
        instances: vec![instance],
    });
}

fn pipeline_for_layer(layer: RenderLayer) -> NativeRenderPipeline {
    match layer {
        RenderLayer::Terrain => NativeRenderPipeline::Terrain,
        RenderLayer::Starfield => NativeRenderPipeline::Starfield,
        RenderLayer::Objects => NativeRenderPipeline::Sprites,
        RenderLayer::Projectiles => NativeRenderPipeline::Projectiles,
        RenderLayer::Hud => NativeRenderPipeline::HudText,
        RenderLayer::Overlay => NativeRenderPipeline::DebugOverlay,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuRendererSettings {
    pub texture_format: wgpu::TextureFormat,
    pub present_mode: wgpu::PresentMode,
    pub alpha_mode: wgpu::CompositeAlphaMode,
}

impl Default for GpuRendererSettings {
    fn default() -> Self {
        Self {
            texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT, AtlasRegion, Color, GpuRendererSettings,
        NativeRenderPipeline, NativeRendererResources, NativeSceneRenderer, ObjectPicturePalette,
        PALE_YELLOW_RGBA, PICTURE_COLOR_TABLE, PURPLE_RGBA, RenderLayer, RenderLayerCounts,
        RenderScene, SceneDrawPlan, SceneProjectionUniformUpload, SceneProjectionUniforms,
        SceneRaster, SceneRasterError, SceneRasterUpload, SceneSprite, SpriteAtlasTextureUpload,
        SpriteBindGroupLayoutPlan, SpriteBindGroupRole, SpriteBufferRole, SpriteBufferUpload,
        SpriteBufferUploadPlan, SpriteDrawBatch, SpriteDrawCommand, SpriteDrawInstance, SpriteId,
        SpriteIndexBufferBinding, SpriteInstanceBuffer, SpriteInstanceBufferRecord,
        SpriteInstanceUpload, SpritePipelineLayoutBindGroup, SpritePipelineLayoutPlan,
        SpritePipelinePlan, SpriteQuadGeometry, SpriteQuadVertex, SpriteRenderPassDraw,
        SpriteRenderPassEncoderCommand, SpriteRenderPassEncoderPlan, SpriteRenderPassPlan,
        SpriteRenderPipelineDescriptorPlan, SpriteResourceBindingPlan, SpriteResourceBindingRole,
        SpriteSamplerBindingPlan, SpriteShaderPlan, SpriteTextureBindingPlan,
        SpriteVertexBufferBinding, SpriteVertexBufferLayoutPlan, SurfaceSize, TextureAtlas,
        ViewportLayout, WgpuFrameCommand, WgpuFramePlan, WgpuPassPlan, WgpuViewportCommand,
        decode_source_object_image_rgba, pseudo_color_rgba, push_source_controlled_message_sprites,
        push_source_text_bytes_sprites, source_message_text, source_screen_position,
        source_screen_position_with_offset,
    };
    use crate::renderer::{
        EmbeddedSprite, WHITE_RGBA, decode_source_attract_williams_logo_rgba,
        source_attract_williams_logo_operation_pixel_counts,
        source_attract_williams_logo_pixel_path,
    };

    #[test]
    fn surface_size_reports_empty_edges() {
        assert!(SurfaceSize::new(0, 240).is_empty());
        assert!(SurfaceSize::new(320, 0).is_empty());
        assert!(!SurfaceSize::new(320, 240).is_empty());
    }

    #[test]
    fn viewport_layout_preserves_scene_aspect_and_centers() {
        let scene = SurfaceSize::new(292, 240);

        assert_eq!(
            ViewportLayout::fit(scene, SurfaceSize::new(640, 480)),
            ViewportLayout {
                scene,
                target: SurfaceSize::new(640, 480),
                origin: [28, 0],
                size: SurfaceSize::new(584, 480),
                scale: 2.0,
            }
        );
        assert_eq!(
            ViewportLayout::fit(scene, SurfaceSize::new(800, 600)),
            ViewportLayout {
                scene,
                target: SurfaceSize::new(800, 600),
                origin: [35, 0],
                size: SurfaceSize::new(730, 600),
                scale: 2.5,
            }
        );
        assert_eq!(
            ViewportLayout::fit(scene, SurfaceSize::new(320, 240)),
            ViewportLayout {
                scene,
                target: SurfaceSize::new(320, 240),
                origin: [14, 0],
                size: SurfaceSize::new(292, 240),
                scale: 1.0,
            }
        );
    }

    #[test]
    fn viewport_layout_reports_empty_scene_or_target() {
        let empty_target =
            ViewportLayout::fit(SurfaceSize::new(292, 240), SurfaceSize::new(0, 480));
        let empty_scene = ViewportLayout::fit(SurfaceSize::new(0, 240), SurfaceSize::new(640, 480));

        assert_eq!(
            empty_target,
            ViewportLayout {
                scene: SurfaceSize::new(292, 240),
                target: SurfaceSize::new(0, 480),
                origin: [0, 0],
                size: SurfaceSize::new(0, 0),
                scale: 0.0,
            }
        );
        assert!(empty_target.is_empty());
        assert_eq!(
            empty_scene,
            ViewportLayout {
                scene: SurfaceSize::new(0, 240),
                target: SurfaceSize::new(640, 480),
                origin: [0, 0],
                size: SurfaceSize::new(0, 0),
                scale: 0.0,
            }
        );
        assert!(empty_scene.is_empty());
    }

    #[test]
    fn color_normalizes_to_wgpu_clear_color() {
        let color = Color {
            rgba: [128, 64, 255, 0],
        };

        assert_eq!(
            color.to_wgpu(),
            wgpu::Color {
                r: 128.0 / 255.0,
                g: 64.0 / 255.0,
                b: 1.0,
                a: 0.0,
            }
        );
        assert_eq!(
            color.to_normalized_rgba(),
            [128.0 / 255.0, 64.0 / 255.0, 1.0, 0.0]
        );
    }

    #[test]
    fn sprite_instance_buffer_record_normalizes_atlas_and_tint() {
        let record = SpriteInstanceBufferRecord::from_instance(
            SpriteDrawInstance {
                sprite: SpriteId::PLAYER_SHIP,
                atlas_origin: [16, 32],
                atlas_size: [8, 16],
                layer: RenderLayer::Objects,
                position: [12.0, 34.0],
                size: [16.0, 8.0],
                tint: Color {
                    rgba: [255, 128, 64, 32],
                },
            },
            SurfaceSize::new(128, 64),
        )
        .expect("instance buffer record");

        assert_eq!(record.scene_origin, [12.0, 34.0]);
        assert_eq!(record.scene_size, [16.0, 8.0]);
        assert_eq!(record.atlas_uv_origin, [0.125, 0.5]);
        assert_eq!(record.atlas_uv_size, [0.0625, 0.25]);
        assert_eq!(
            record.tint,
            [1.0, 128.0 / 255.0, 64.0 / 255.0, 32.0 / 255.0]
        );
        assert_eq!(
            SpriteInstanceBufferRecord::from_instance(
                SpriteDrawInstance {
                    sprite: SpriteId::PLAYER_SHIP,
                    atlas_origin: [0, 0],
                    atlas_size: [16, 8],
                    layer: RenderLayer::Objects,
                    position: [0.0, 0.0],
                    size: [16.0, 8.0],
                    tint: Color::WHITE,
                },
                SurfaceSize::new(0, 64),
            ),
            None
        );
    }

    #[test]
    fn sprite_quad_vertex_declares_stable_gpu_layout() {
        let layout = SpriteQuadGeometry::vertex_buffer_layout();
        let expected_attributes = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 5,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 6,
            },
        ];

        assert_eq!(SpriteQuadVertex::FLOAT_COMPONENTS, 4);
        assert_eq!(SpriteQuadVertex::BYTE_SIZE, 16);
        assert_eq!(std::mem::size_of::<SpriteQuadVertex>(), 16);
        assert_eq!(
            std::mem::align_of::<SpriteQuadVertex>(),
            std::mem::align_of::<f32>()
        );
        assert_eq!(layout.array_stride, 16);
        assert_eq!(layout.step_mode, wgpu::VertexStepMode::Vertex);
        assert_eq!(layout.attributes, expected_attributes);
        assert!(SpriteQuadVertex::VERTEX_ATTRIBUTES.iter().all(|quad| {
            !SpriteInstanceBufferRecord::VERTEX_ATTRIBUTES
                .iter()
                .any(|instance| instance.shader_location == quad.shader_location)
        }));
    }

    #[test]
    fn sprite_quad_geometry_exposes_upload_bytes_without_repacking() {
        assert_eq!(SpriteQuadGeometry::VERTEX_COUNT, 4);
        assert_eq!(SpriteQuadGeometry::INDEX_COUNT, 6);
        assert_eq!(SpriteQuadGeometry::INDEX_FORMAT, wgpu::IndexFormat::Uint16);
        assert_eq!(
            SpriteQuadGeometry::vertices(),
            SpriteQuadGeometry::VERTICES.as_slice()
        );
        assert_eq!(
            SpriteQuadGeometry::indices(),
            SpriteQuadGeometry::INDICES.as_slice()
        );
        assert_eq!(
            SpriteQuadGeometry::vertex_upload_bytes().len(),
            SpriteQuadGeometry::vertices().len() * SpriteQuadVertex::BYTE_SIZE as usize
        );
        assert_eq!(
            SpriteQuadGeometry::index_upload_bytes().len(),
            std::mem::size_of_val(SpriteQuadGeometry::indices())
        );
        assert_eq!(
            SpriteQuadGeometry::vertex_upload_bytes(),
            bytemuck::cast_slice::<SpriteQuadVertex, u8>(SpriteQuadGeometry::vertices())
        );
        assert_eq!(
            SpriteQuadGeometry::index_upload_bytes(),
            bytemuck::cast_slice::<u16, u8>(SpriteQuadGeometry::indices())
        );
        assert_eq!(
            SpriteQuadGeometry::vertices()[0].as_bytes(),
            &SpriteQuadGeometry::vertex_upload_bytes()[..SpriteQuadVertex::BYTE_SIZE as usize]
        );
        assert_eq!(
            bytemuck::cast_slice::<SpriteQuadVertex, f32>(SpriteQuadGeometry::vertices()),
            &[
                0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            ]
        );
        assert_eq!(SpriteQuadGeometry::indices(), &[0, 2, 1, 2, 3, 1]);
    }

    #[test]
    fn sprite_quad_indices_are_front_facing_after_scene_projection() {
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(1, 1)).expect("projection");
        let clip_vertices = SpriteQuadGeometry::vertices()
            .iter()
            .map(|vertex| projection.project_point(vertex.unit_position))
            .collect::<Vec<_>>();

        for triangle in SpriteQuadGeometry::indices().chunks_exact(3) {
            let points = [
                clip_vertices[triangle[0] as usize],
                clip_vertices[triangle[1] as usize],
                clip_vertices[triangle[2] as usize],
            ];

            assert!(
                triangle_signed_area(points) > 0.0,
                "quad triangle {triangle:?} was not counter-clockwise in clip space"
            );
        }
    }

    #[test]
    fn sprite_instance_buffer_record_declares_stable_gpu_layout() {
        let layout = SpriteInstanceBufferRecord::vertex_buffer_layout();
        let expected_attributes = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16,
                shader_location: 2,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 24,
                shader_location: 3,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 4,
            },
        ];

        assert_eq!(SpriteInstanceBufferRecord::FLOAT_COMPONENTS, 12);
        assert_eq!(SpriteInstanceBufferRecord::BYTE_SIZE, 48);
        assert_eq!(std::mem::size_of::<SpriteInstanceBufferRecord>(), 48);
        assert_eq!(
            std::mem::align_of::<SpriteInstanceBufferRecord>(),
            std::mem::align_of::<f32>()
        );
        assert_eq!(layout.array_stride, 48);
        assert_eq!(layout.step_mode, wgpu::VertexStepMode::Instance);
        assert_eq!(layout.attributes, expected_attributes);
    }

    #[test]
    fn sprite_instance_buffer_exposes_upload_bytes_without_repacking() {
        let records = vec![
            SpriteInstanceBufferRecord {
                scene_origin: [1.0, 2.0],
                scene_size: [3.0, 4.0],
                atlas_uv_origin: [0.125, 0.25],
                atlas_uv_size: [0.5, 0.75],
                tint: [1.0, 0.5, 0.25, 0.125],
            },
            SpriteInstanceBufferRecord {
                scene_origin: [5.0, 6.0],
                scene_size: [7.0, 8.0],
                atlas_uv_origin: [0.0, 0.5],
                atlas_uv_size: [0.25, 0.125],
                tint: [0.25, 0.5, 0.75, 1.0],
            },
        ];
        let buffer = SpriteInstanceBuffer {
            pipeline: NativeRenderPipeline::Sprites,
            layer: RenderLayer::Objects,
            records,
        };

        assert_eq!(
            buffer.upload_bytes().len(),
            buffer.records.len() * SpriteInstanceBufferRecord::BYTE_SIZE as usize
        );
        assert_eq!(
            buffer.upload_bytes(),
            bytemuck::cast_slice::<SpriteInstanceBufferRecord, u8>(&buffer.records)
        );
        assert_eq!(
            buffer.records[0].as_bytes(),
            &buffer.upload_bytes()[..SpriteInstanceBufferRecord::BYTE_SIZE as usize]
        );
        assert_eq!(
            bytemuck::cast_slice::<SpriteInstanceBufferRecord, f32>(&buffer.records),
            &[
                1.0, 2.0, 3.0, 4.0, 0.125, 0.25, 0.5, 0.75, 1.0, 0.5, 0.25, 0.125, 5.0, 6.0, 7.0,
                8.0, 0.0, 0.5, 0.25, 0.125, 0.25, 0.5, 0.75, 1.0,
            ]
        );
    }

    #[test]
    fn sprite_instance_upload_flattens_buffers_without_repacking() {
        let first =
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2);
        let empty = test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 0);
        let second =
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 1);

        let upload =
            SpriteInstanceUpload::from_instance_buffers(&[first.clone(), empty, second.clone()])
                .expect("sprite instance upload");
        let expected_records = first
            .records
            .iter()
            .chain(&second.records)
            .copied()
            .collect::<Vec<_>>();

        assert_eq!(upload.instance_count(), 3);
        assert_eq!(upload.records, expected_records);
        assert_eq!(upload.byte_len(), 3 * SpriteInstanceBufferRecord::BYTE_SIZE);
        assert_eq!(
            upload.upload_bytes(),
            bytemuck::cast_slice::<SpriteInstanceBufferRecord, u8>(&upload.records)
        );
        assert_eq!(SpriteInstanceUpload::from_instance_buffers(&[]), None);
        assert_eq!(
            SpriteInstanceUpload::from_instance_buffers(&[test_sprite_instance_buffer(
                NativeRenderPipeline::DebugOverlay,
                RenderLayer::Overlay,
                0,
            )]),
            None
        );
    }

    #[test]
    fn sprite_buffer_upload_plan_describes_wgpu_buffers_and_bytes() {
        let buffer =
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2);
        let upload =
            SpriteInstanceUpload::from_instance_buffers(&[buffer]).expect("instance upload");

        let plan = SpriteBufferUploadPlan::from_instance_upload(&upload);

        assert_eq!(
            plan.quad_vertices,
            SpriteBufferUpload {
                role: SpriteBufferRole::QuadVertices,
                label: "defender.sprite.quad.vertices",
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                byte_len: SpriteQuadGeometry::vertex_upload_bytes().len() as wgpu::BufferAddress,
                bytes: SpriteQuadGeometry::vertex_upload_bytes().to_vec(),
            }
        );
        assert_eq!(
            plan.quad_indices,
            SpriteBufferUpload {
                role: SpriteBufferRole::QuadIndices,
                label: "defender.sprite.quad.indices",
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                byte_len: SpriteQuadGeometry::index_upload_bytes().len() as wgpu::BufferAddress,
                bytes: SpriteQuadGeometry::index_upload_bytes().to_vec(),
            }
        );
        assert_eq!(
            plan.instances,
            SpriteBufferUpload {
                role: SpriteBufferRole::Instances,
                label: "defender.sprite.instances",
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                byte_len: upload.byte_len(),
                bytes: upload.upload_bytes().to_vec(),
            }
        );
    }

    #[test]
    fn sprite_render_pass_plan_describes_bindings_and_indexed_draws() {
        let buffers = vec![
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2),
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 1),
        ];
        let upload = SpriteInstanceUpload::from_instance_buffers(&buffers).expect("upload");
        let uploads = SpriteBufferUploadPlan::from_instance_upload(&upload);
        let commands = super::sprite_draw_commands_from_instance_buffers(&buffers);

        let plan = SpriteRenderPassPlan::from_uploads_and_commands(&uploads, &commands)
            .expect("sprite render pass plan");

        assert_eq!(
            plan.quad_vertices,
            SpriteVertexBufferBinding {
                role: SpriteBufferRole::QuadVertices,
                slot: SpriteVertexBufferBinding::QUAD_VERTEX_SLOT,
                byte_offset: 0,
                byte_len: uploads.quad_vertices.byte_len,
            }
        );
        assert_eq!(
            plan.instances,
            SpriteVertexBufferBinding {
                role: SpriteBufferRole::Instances,
                slot: SpriteVertexBufferBinding::INSTANCE_SLOT,
                byte_offset: 0,
                byte_len: uploads.instances.byte_len,
            }
        );
        assert_eq!(
            plan.indices,
            SpriteIndexBufferBinding {
                role: SpriteBufferRole::QuadIndices,
                index_format: wgpu::IndexFormat::Uint16,
                byte_offset: 0,
                byte_len: uploads.quad_indices.byte_len,
            }
        );
        assert_eq!(plan.draw_count(), 2);
        assert_eq!(plan.instance_count(), 3);
        assert_eq!(
            plan.draws,
            vec![
                SpriteRenderPassDraw {
                    pipeline: NativeRenderPipeline::Sprites,
                    layer: RenderLayer::Objects,
                    indices: 0..SpriteQuadGeometry::INDEX_COUNT,
                    base_vertex: 0,
                    instances: 0..2,
                    instance_buffer_byte_offset: 0,
                    instance_buffer_byte_len: 2 * SpriteInstanceBufferRecord::BYTE_SIZE,
                },
                SpriteRenderPassDraw {
                    pipeline: NativeRenderPipeline::HudText,
                    layer: RenderLayer::Hud,
                    indices: 0..SpriteQuadGeometry::INDEX_COUNT,
                    base_vertex: 0,
                    instances: 2..3,
                    instance_buffer_byte_offset: 2 * SpriteInstanceBufferRecord::BYTE_SIZE,
                    instance_buffer_byte_len: SpriteInstanceBufferRecord::BYTE_SIZE,
                },
            ]
        );
        assert_eq!(
            SpriteRenderPassPlan::from_uploads_and_commands(&uploads, &[]),
            None
        );
    }

    #[test]
    fn sprite_render_pass_encoder_plan_orders_wgpu_commands() {
        let buffers = vec![
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2),
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 1),
        ];
        let upload = SpriteInstanceUpload::from_instance_buffers(&buffers).expect("upload");
        let uploads = SpriteBufferUploadPlan::from_instance_upload(&upload);
        let draw_commands = super::sprite_draw_commands_from_instance_buffers(&buffers);
        let render_pass = SpriteRenderPassPlan::from_uploads_and_commands(&uploads, &draw_commands)
            .expect("sprite render pass plan");
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");
        let atlas = TextureAtlas::default_sprites();
        let bindings = SpriteResourceBindingPlan::from_projection_and_atlas(projection, &atlas)
            .expect("sprite resource binding plan");
        let layout = SpritePipelineLayoutPlan::from_resource_bindings(&bindings);
        let pipeline = SpritePipelinePlan::for_settings(GpuRendererSettings::default());
        let descriptor =
            SpriteRenderPipelineDescriptorPlan::from_pipeline_and_layout(&pipeline, &layout);

        let plan = SpriteRenderPassEncoderPlan::from_render_pass_layout_and_descriptor(
            &render_pass,
            &layout,
            &descriptor,
        );

        assert_eq!(plan.label, "defender.sprite.render_pass.encoder");
        assert_eq!(plan.command_count(), 8);
        assert_eq!(
            plan.set_pipeline_command_count(),
            SpriteRenderPassEncoderPlan::SET_PIPELINE_COMMAND_COUNT
        );
        assert_eq!(
            plan.set_bind_group_command_count(),
            SpriteRenderPassEncoderPlan::SET_BIND_GROUP_COMMAND_COUNT
        );
        assert_eq!(
            plan.set_vertex_buffer_command_count(),
            SpriteRenderPassEncoderPlan::SET_VERTEX_BUFFER_COMMAND_COUNT
        );
        assert_eq!(
            plan.set_index_buffer_command_count(),
            SpriteRenderPassEncoderPlan::SET_INDEX_BUFFER_COMMAND_COUNT
        );
        assert_eq!(plan.draw_count(), 2);
        assert_eq!(plan.instance_count(), 3);
        assert_eq!(
            plan.commands,
            vec![
                SpriteRenderPassEncoderCommand::SetPipeline {
                    label: "defender.sprite.pipeline",
                },
                SpriteRenderPassEncoderCommand::SetBindGroup {
                    role: SpriteBindGroupRole::SceneProjection,
                    group_index: 0,
                    layout_label: "defender.sprite.scene_projection.bind_group_layout",
                },
                SpriteRenderPassEncoderCommand::SetBindGroup {
                    role: SpriteBindGroupRole::SpriteAtlas,
                    group_index: 1,
                    layout_label: "defender.sprite.atlas.bind_group_layout",
                },
                SpriteRenderPassEncoderCommand::SetVertexBuffer {
                    role: SpriteBufferRole::QuadVertices,
                    slot: SpriteVertexBufferBinding::QUAD_VERTEX_SLOT,
                    byte_offset: 0,
                    byte_len: SpriteQuadGeometry::vertex_upload_bytes().len()
                        as wgpu::BufferAddress,
                },
                SpriteRenderPassEncoderCommand::SetVertexBuffer {
                    role: SpriteBufferRole::Instances,
                    slot: SpriteVertexBufferBinding::INSTANCE_SLOT,
                    byte_offset: 0,
                    byte_len: 3 * SpriteInstanceBufferRecord::BYTE_SIZE,
                },
                SpriteRenderPassEncoderCommand::SetIndexBuffer {
                    role: SpriteBufferRole::QuadIndices,
                    index_format: wgpu::IndexFormat::Uint16,
                    byte_offset: 0,
                    byte_len: SpriteQuadGeometry::index_upload_bytes().len() as wgpu::BufferAddress,
                },
                SpriteRenderPassEncoderCommand::DrawIndexed {
                    draw: render_pass.draws[0].clone(),
                },
                SpriteRenderPassEncoderCommand::DrawIndexed {
                    draw: render_pass.draws[1].clone(),
                },
            ]
        );
    }

    #[test]
    fn wgpu_frame_plan_orders_pass_raster_and_sprite_commands() {
        let pass = WgpuPassPlan {
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.1,
                b: 0.2,
                a: 1.0,
            },
            viewport: Some(WgpuViewportCommand {
                x: 28.0,
                y: 0.0,
                width: 584.0,
                height: 480.0,
                min_depth: 0.0,
                max_depth: 1.0,
            }),
            scene_projection: SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)),
        };
        let raster_upload = SceneRasterUpload {
            surface: SurfaceSize::new(292, 240),
            byte_len: 292 * 240 * 4,
            visual_signature: Some(0xCAFE_BABE),
            non_blank: true,
        };
        let sprite_encoder = SpriteRenderPassEncoderPlan {
            label: "defender.sprite.render_pass.encoder",
            commands: vec![
                SpriteRenderPassEncoderCommand::SetPipeline {
                    label: "defender.sprite.pipeline",
                },
                SpriteRenderPassEncoderCommand::DrawIndexed {
                    draw: SpriteRenderPassDraw {
                        pipeline: NativeRenderPipeline::Sprites,
                        layer: RenderLayer::Objects,
                        indices: 0..SpriteQuadGeometry::INDEX_COUNT,
                        base_vertex: 0,
                        instances: 0..2,
                        instance_buffer_byte_offset: 0,
                        instance_buffer_byte_len: 2 * SpriteInstanceBufferRecord::BYTE_SIZE,
                    },
                },
            ],
        };

        let plan = WgpuFramePlan::from_pass_raster_and_sprite_encoder(
            &pass,
            Some(raster_upload),
            Some(&sprite_encoder),
        );

        assert_eq!(plan.label, "defender.frame.commands");
        assert_eq!(plan.command_count(), 5);
        assert!(!plan.has_ordered_sprite_only_commands());
        assert_eq!(plan.temporary_raster_count(), 1);
        assert_eq!(plan.sprite_pass_count(), 1);
        assert_eq!(plan.begin_render_pass_count(), 1);
        assert_eq!(plan.viewport_command_count(), 1);
        assert_eq!(
            plan.scene_projection_upload_byte_len(),
            SceneProjectionUniforms::BYTE_SIZE
        );
        assert_eq!(plan.sprite_encoder_command_count(), 2);
        assert_eq!(plan.sprite_draw_count(), 1);
        assert_eq!(plan.sprite_instance_count(), 2);
        assert_eq!(
            plan.commands,
            vec![
                WgpuFrameCommand::BeginRenderPass {
                    clear_color: pass.clear_color,
                },
                WgpuFrameCommand::SetViewport {
                    viewport: pass.viewport.expect("viewport"),
                },
                WgpuFrameCommand::UploadSceneProjection {
                    byte_len: SceneProjectionUniforms::BYTE_SIZE,
                },
                WgpuFrameCommand::UploadTemporaryRaster {
                    upload: raster_upload,
                },
                WgpuFrameCommand::ExecuteSpriteRenderPass {
                    encoder_label: "defender.sprite.render_pass.encoder",
                    command_count: 2,
                    draw_count: 1,
                    instance_count: 2,
                },
            ]
        );

        let sprite_only_plan =
            WgpuFramePlan::from_pass_raster_and_sprite_encoder(&pass, None, Some(&sprite_encoder));

        assert_eq!(sprite_only_plan.command_count(), 4);
        assert!(sprite_only_plan.has_ordered_sprite_only_commands());
    }

    #[test]
    fn sprite_shader_plan_exposes_wgsl_descriptor_and_entries() {
        let shader = SpriteShaderPlan::default();

        assert_eq!(shader.label, "defender.sprite.shader");
        assert_eq!(shader.vertex_entry, "sprite_vs");
        assert_eq!(shader.fragment_entry, "sprite_fs");
        assert!(shader.source.contains("@vertex"));
        assert!(shader.source.contains("@fragment"));
        assert!(shader.source.contains("textureSample(sprite_atlas"));
        assert!(shader.source.contains("@location(0) scene_origin"));
        assert!(shader.source.contains("@location(6) unit_uv"));

        let descriptor = shader.shader_module_descriptor();
        assert_eq!(descriptor.label, Some("defender.sprite.shader"));
        match descriptor.source {
            wgpu::ShaderSource::Wgsl(source) => assert_eq!(source.as_ref(), shader.source),
            _ => panic!("sprite shader descriptor must use WGSL"),
        }
    }

    #[test]
    fn sprite_pipeline_plan_describes_wgpu_state_and_vertex_layouts() {
        let settings = GpuRendererSettings {
            texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            ..GpuRendererSettings::default()
        };

        let plan = SpritePipelinePlan::for_settings(settings);

        assert_eq!(plan.label, "defender.sprite.pipeline");
        assert_eq!(plan.shader, SpriteShaderPlan::default());
        assert_eq!(
            plan.vertex_buffers,
            [
                SpriteVertexBufferLayoutPlan {
                    role: SpriteBufferRole::QuadVertices,
                    slot: SpriteVertexBufferBinding::QUAD_VERTEX_SLOT,
                    array_stride: SpriteQuadVertex::BYTE_SIZE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &SpriteQuadVertex::VERTEX_ATTRIBUTES,
                },
                SpriteVertexBufferLayoutPlan {
                    role: SpriteBufferRole::Instances,
                    slot: SpriteVertexBufferBinding::INSTANCE_SLOT,
                    array_stride: SpriteInstanceBufferRecord::BYTE_SIZE,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &SpriteInstanceBufferRecord::VERTEX_ATTRIBUTES,
                },
            ]
        );

        let layouts = plan.vertex_buffer_layouts();
        assert_eq!(layouts[0].array_stride, SpriteQuadVertex::BYTE_SIZE);
        assert_eq!(layouts[0].step_mode, wgpu::VertexStepMode::Vertex);
        assert_eq!(layouts[0].attributes, SpriteQuadVertex::VERTEX_ATTRIBUTES);
        assert_eq!(
            layouts[1].array_stride,
            SpriteInstanceBufferRecord::BYTE_SIZE
        );
        assert_eq!(layouts[1].step_mode, wgpu::VertexStepMode::Instance);
        assert_eq!(
            layouts[1].attributes,
            SpriteInstanceBufferRecord::VERTEX_ATTRIBUTES
        );
        assert_eq!(
            plan.primitive.topology,
            wgpu::PrimitiveTopology::TriangleList
        );
        assert_eq!(plan.primitive.front_face, wgpu::FrontFace::Ccw);
        assert_eq!(plan.primitive.cull_mode, None);
        assert_eq!(
            plan.color_target,
            wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            }
        );
        assert_eq!(plan.multisample, wgpu::MultisampleState::default());
    }

    #[test]
    fn sprite_atlas_texture_upload_describes_wgpu_texture_copy() {
        let atlas = TextureAtlas::default_sprites();
        let upload = SpriteAtlasTextureUpload::from_atlas(&atlas).expect("atlas upload");

        assert_eq!(
            upload,
            SpriteAtlasTextureUpload {
                role: SpriteResourceBindingRole::SpriteAtlasTexture,
                label: "defender.sprite.atlas.texture",
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                dimension: wgpu::TextureDimension::D2,
                surface: SurfaceSize::new(128, 192),
                mip_level_count: 1,
                sample_count: 1,
                depth_or_array_layers: 1,
                bytes_per_row: 128 * 4,
                rows_per_image: 192,
                byte_len: 128 * 192 * 4,
                bytes: atlas.pixels().to_vec(),
                non_blank: true,
            }
        );
        assert_eq!(
            upload.extent(),
            wgpu::Extent3d {
                width: 128,
                height: 192,
                depth_or_array_layers: 1,
            }
        );
        let copy_layout = upload.copy_layout();
        assert_eq!(copy_layout.offset, 0);
        assert_eq!(copy_layout.bytes_per_row, Some(128 * 4));
        assert_eq!(copy_layout.rows_per_image, Some(192));
        let descriptor = upload.texture_descriptor();
        assert_eq!(descriptor.label, Some("defender.sprite.atlas.texture"));
        assert_eq!(descriptor.size, upload.extent());
        assert_eq!(descriptor.mip_level_count, 1);
        assert_eq!(descriptor.sample_count, 1);
        assert_eq!(descriptor.dimension, wgpu::TextureDimension::D2);
        assert_eq!(descriptor.format, wgpu::TextureFormat::Rgba8UnormSrgb);
        assert_eq!(
            descriptor.usage,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
        );
        assert_eq!(descriptor.view_formats, &[]);

        assert_eq!(
            SpriteAtlasTextureUpload::from_atlas(&TextureAtlas::new(
                SurfaceSize::new(0, 128),
                Vec::new()
            )),
            None
        );
        let missing_pixels = TextureAtlas {
            surface: SurfaceSize::new(2, 2),
            regions: Vec::new(),
            pixels: Vec::new(),
        };
        assert_eq!(SpriteAtlasTextureUpload::from_atlas(&missing_pixels), None);
    }

    #[test]
    fn sprite_resource_binding_plan_describes_uniform_and_atlas_bindings() {
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");
        let atlas = TextureAtlas::with_rgba(
            SurfaceSize::new(128, 64),
            Vec::new(),
            vec![0x80; 128 * 64 * 4],
        )
        .expect("atlas");

        let plan = SpriteResourceBindingPlan::from_projection_and_atlas(projection, &atlas)
            .expect("sprite resource binding plan");

        assert_eq!(
            plan.atlas_upload,
            SpriteAtlasTextureUpload {
                role: SpriteResourceBindingRole::SpriteAtlasTexture,
                label: "defender.sprite.atlas.texture",
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                dimension: wgpu::TextureDimension::D2,
                surface: SurfaceSize::new(128, 64),
                mip_level_count: 1,
                sample_count: 1,
                depth_or_array_layers: 1,
                bytes_per_row: 128 * 4,
                rows_per_image: 64,
                byte_len: 128 * 64 * 4,
                bytes: atlas.pixels().to_vec(),
                non_blank: true,
            }
        );
        assert_eq!(
            plan.projection_upload,
            SceneProjectionUniformUpload {
                role: SpriteResourceBindingRole::SceneProjectionUniform,
                label: "defender.sprite.scene_projection.uniform",
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                byte_len: SceneProjectionUniforms::BYTE_SIZE,
                bytes: projection.as_bytes().to_vec(),
            }
        );
        assert_eq!(plan.projection_layout.group_index(), 0);
        assert_eq!(
            plan.bind_group_count(),
            SpriteResourceBindingPlan::BIND_GROUP_COUNT
        );
        assert_eq!(
            plan.binding_entry_count(),
            SpriteResourceBindingPlan::BINDING_ENTRY_COUNT
        );
        assert_eq!(
            plan.projection_layout,
            SpriteBindGroupLayoutPlan {
                role: SpriteBindGroupRole::SceneProjection,
                label: "defender.sprite.scene_projection.bind_group_layout",
                entries: vec![wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(SceneProjectionUniforms::BYTE_SIZE),
                    },
                    count: None,
                }],
            }
        );
        assert_eq!(plan.atlas_layout.group_index(), 1);
        assert_eq!(
            plan.atlas_layout,
            SpriteBindGroupLayoutPlan {
                role: SpriteBindGroupRole::SpriteAtlas,
                label: "defender.sprite.atlas.bind_group_layout",
                entries: vec![
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            }
        );
        assert_eq!(
            plan.atlas_texture,
            SpriteTextureBindingPlan {
                role: SpriteResourceBindingRole::SpriteAtlasTexture,
                label: "defender.sprite.atlas.texture_view",
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
                surface: SurfaceSize::new(128, 64),
            }
        );
        assert_eq!(
            plan.atlas_sampler,
            SpriteSamplerBindingPlan {
                role: SpriteResourceBindingRole::SpriteAtlasSampler,
                label: "defender.sprite.atlas.sampler",
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                sampler_binding: wgpu::SamplerBindingType::Filtering,
            }
        );
        assert_eq!(
            SpriteResourceBindingPlan::from_projection_and_atlas(
                projection,
                &TextureAtlas::new(SurfaceSize::new(0, 64), Vec::new())
            ),
            None
        );
    }

    #[test]
    fn sprite_pipeline_layout_plan_orders_resource_bind_groups() {
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");
        let atlas = TextureAtlas::default_sprites();
        let bindings = SpriteResourceBindingPlan::from_projection_and_atlas(projection, &atlas)
            .expect("sprite resource binding plan");

        let plan = SpritePipelineLayoutPlan::from_resource_bindings(&bindings);

        assert_eq!(plan.label, "defender.sprite.pipeline_layout");
        assert_eq!(plan.immediate_size, 0);
        assert_eq!(
            plan.bind_group_count(),
            SpritePipelineLayoutPlan::BIND_GROUP_COUNT
        );
        assert_eq!(
            plan.binding_entry_count(),
            SpritePipelineLayoutPlan::BINDING_ENTRY_COUNT
        );
        assert_eq!(
            plan.bind_groups,
            vec![
                SpritePipelineLayoutBindGroup {
                    role: SpriteBindGroupRole::SceneProjection,
                    group_index: 0,
                    layout_label: "defender.sprite.scene_projection.bind_group_layout",
                    entry_count: 1,
                },
                SpritePipelineLayoutBindGroup {
                    role: SpriteBindGroupRole::SpriteAtlas,
                    group_index: 1,
                    layout_label: "defender.sprite.atlas.bind_group_layout",
                    entry_count: 2,
                },
            ]
        );
    }

    #[test]
    fn sprite_render_pipeline_descriptor_plan_combines_pipeline_and_layout() {
        let settings = GpuRendererSettings {
            texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            ..GpuRendererSettings::default()
        };
        let pipeline = SpritePipelinePlan::for_settings(settings);
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");
        let atlas = TextureAtlas::default_sprites();
        let bindings = SpriteResourceBindingPlan::from_projection_and_atlas(projection, &atlas)
            .expect("sprite resource binding plan");
        let layout = SpritePipelineLayoutPlan::from_resource_bindings(&bindings);

        let descriptor =
            SpriteRenderPipelineDescriptorPlan::from_pipeline_and_layout(&pipeline, &layout);

        assert_eq!(descriptor.label, "defender.sprite.pipeline");
        assert_eq!(descriptor.layout_label, "defender.sprite.pipeline_layout");
        assert_eq!(
            descriptor.layout_bind_group_count(),
            SpriteRenderPipelineDescriptorPlan::LAYOUT_BIND_GROUP_COUNT
        );
        assert_eq!(
            descriptor.vertex_buffer_count(),
            SpriteRenderPipelineDescriptorPlan::VERTEX_BUFFER_COUNT
        );
        assert_eq!(
            descriptor.color_target_count(),
            SpriteRenderPipelineDescriptorPlan::COLOR_TARGET_COUNT
        );
        assert_eq!(descriptor.immediate_size, 0);
        assert_eq!(descriptor.shader_label, "defender.sprite.shader");
        assert_eq!(descriptor.vertex_entry, "sprite_vs");
        assert_eq!(descriptor.fragment_entry, "sprite_fs");
        assert_eq!(descriptor.vertex_buffers, pipeline.vertex_buffers);
        assert_eq!(descriptor.primitive, pipeline.primitive);
        assert_eq!(descriptor.color_target, pipeline.color_target);
        assert_eq!(descriptor.multisample, pipeline.multisample);
        assert_eq!(
            descriptor.vertex_buffer_layouts(),
            pipeline.vertex_buffer_layouts()
        );
        assert_eq!(
            descriptor.color_targets(),
            [Some(pipeline.color_target.clone())]
        );
    }

    #[test]
    fn sprite_shader_bindings_match_resource_binding_plan() {
        let shader = SpriteShaderPlan::default();
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");
        let atlas = TextureAtlas::default_sprites();
        let bindings = SpriteResourceBindingPlan::from_projection_and_atlas(projection, &atlas)
            .expect("sprite resource binding plan");
        let pipeline_layout = SpritePipelineLayoutPlan::from_resource_bindings(&bindings);

        assert!(shader.source.contains("@group(0) @binding(0)"));
        assert_eq!(bindings.projection_layout.group_index(), 0);
        assert_eq!(bindings.projection_layout.entries[0].binding, 0);
        assert_eq!(pipeline_layout.bind_groups[0].group_index, 0);
        assert!(shader.source.contains("@group(1) @binding(0)"));
        assert!(shader.source.contains("@group(1) @binding(1)"));
        assert_eq!(bindings.atlas_layout.group_index(), 1);
        assert_eq!(bindings.atlas_texture.binding, 0);
        assert_eq!(bindings.atlas_sampler.binding, 1);
        assert_eq!(pipeline_layout.bind_groups[1].group_index, 1);
    }

    #[test]
    fn sprite_draw_command_uses_quad_geometry_and_instance_buffer_metadata() {
        let buffer =
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2);
        let empty_buffer =
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 0);

        assert_eq!(
            SpriteDrawCommand::from_instance_buffer(&buffer, 7),
            Some(expected_sprite_draw_command(
                NativeRenderPipeline::Sprites,
                RenderLayer::Objects,
                7,
                2,
            ))
        );
        assert_eq!(
            SpriteDrawCommand::from_instance_buffer(&empty_buffer, 7),
            None
        );
    }

    #[test]
    fn sprite_draw_commands_track_cumulative_instance_ranges() {
        let buffers = vec![
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2),
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 0),
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 1),
            test_sprite_instance_buffer(
                NativeRenderPipeline::Projectiles,
                RenderLayer::Projectiles,
                3,
            ),
        ];

        assert_eq!(
            super::sprite_draw_commands_from_instance_buffers(&buffers),
            vec![
                expected_sprite_draw_command(
                    NativeRenderPipeline::Sprites,
                    RenderLayer::Objects,
                    0,
                    2,
                ),
                expected_sprite_draw_command(NativeRenderPipeline::HudText, RenderLayer::Hud, 2, 1),
                expected_sprite_draw_command(
                    NativeRenderPipeline::Projectiles,
                    RenderLayer::Projectiles,
                    3,
                    3,
                ),
            ]
        );
    }

    #[test]
    fn wgpu_viewport_command_matches_non_empty_layout() {
        let layout = ViewportLayout::fit(SurfaceSize::new(292, 240), SurfaceSize::new(640, 480));

        assert_eq!(
            WgpuViewportCommand::from_layout(layout),
            Some(WgpuViewportCommand {
                x: 28.0,
                y: 0.0,
                width: 584.0,
                height: 480.0,
                min_depth: 0.0,
                max_depth: 1.0,
            })
        );
        assert_eq!(
            WgpuViewportCommand::from_layout(ViewportLayout::fit(
                SurfaceSize::new(292, 240),
                SurfaceSize::new(0, 480)
            )),
            None
        );
    }

    #[test]
    fn scene_projection_uniforms_map_scene_points_to_clip_space() {
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");

        assert_eq!(SceneProjectionUniforms::FLOAT_COMPONENTS, 4);
        assert_eq!(SceneProjectionUniforms::BYTE_SIZE, 16);
        assert_eq!(std::mem::size_of::<SceneProjectionUniforms>(), 16);
        assert_eq!(
            std::mem::align_of::<SceneProjectionUniforms>(),
            std::mem::align_of::<f32>()
        );
        assert_eq!(projection.scale, [2.0 / 292.0, -2.0 / 240.0]);
        assert_eq!(projection.translate, [-1.0, 1.0]);
        assert_eq!(projection.project_point([0.0, 0.0]), [-1.0, 1.0]);
        assert_clip_point_near(projection.project_point([146.0, 120.0]), [0.0, 0.0]);
        assert_clip_point_near(projection.project_point([292.0, 240.0]), [1.0, -1.0]);
        assert_eq!(
            bytemuck::cast_slice::<SceneProjectionUniforms, f32>(&[projection]),
            &[2.0 / 292.0, -2.0 / 240.0, -1.0, 1.0]
        );
        assert_eq!(projection.as_bytes(), bytemuck::bytes_of(&projection));
        assert_eq!(
            SceneProjectionUniforms::for_surface(SurfaceSize::new(0, 240)),
            None
        );
    }

    fn assert_clip_point_near(actual: [f32; 2], expected: [f32; 2]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() <= f32::EPSILON,
                "clip point component {actual} was not near {expected}"
            );
        }
    }

    fn triangle_signed_area(points: [[f32; 2]; 3]) -> f32 {
        let [a, b, c] = points;
        ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) * 0.5
    }

    fn test_sprite_instance_buffer_record(scene_origin: [f32; 2]) -> SpriteInstanceBufferRecord {
        SpriteInstanceBufferRecord {
            scene_origin,
            scene_size: [8.0, 4.0],
            atlas_uv_origin: [0.125, 0.25],
            atlas_uv_size: [0.5, 0.75],
            tint: [1.0, 0.5, 0.25, 1.0],
        }
    }

    fn test_sprite_instance_buffer(
        pipeline: NativeRenderPipeline,
        layer: RenderLayer,
        record_count: usize,
    ) -> SpriteInstanceBuffer {
        SpriteInstanceBuffer {
            pipeline,
            layer,
            records: (0..record_count)
                .map(|index| test_sprite_instance_buffer_record([index as f32, index as f32]))
                .collect(),
        }
    }

    fn expected_sprite_draw_command(
        pipeline: NativeRenderPipeline,
        layer: RenderLayer,
        first_instance: u32,
        instance_count: u32,
    ) -> SpriteDrawCommand {
        SpriteDrawCommand {
            pipeline,
            layer,
            vertex_count: SpriteQuadGeometry::VERTEX_COUNT,
            index_count: SpriteQuadGeometry::INDEX_COUNT,
            index_format: SpriteQuadGeometry::INDEX_FORMAT,
            first_index: 0,
            base_vertex: 0,
            first_instance,
            instance_count,
            vertex_buffer_byte_len: SpriteQuadGeometry::vertex_upload_bytes().len()
                as wgpu::BufferAddress,
            index_buffer_byte_len: SpriteQuadGeometry::index_upload_bytes().len()
                as wgpu::BufferAddress,
            instance_buffer_byte_offset: u64::from(first_instance)
                * SpriteInstanceBufferRecord::BYTE_SIZE,
            instance_buffer_byte_len: u64::from(instance_count)
                * SpriteInstanceBufferRecord::BYTE_SIZE,
        }
    }

    fn expected_sprite_render_pass(plan: &SceneDrawPlan) -> Option<SpriteRenderPassPlan> {
        plan.sprite_buffer_uploads.as_ref().and_then(|uploads| {
            SpriteRenderPassPlan::from_uploads_and_commands(uploads, &plan.sprite_draw_commands)
        })
    }

    fn expected_sprite_pipeline(
        plan: &SceneDrawPlan,
        settings: GpuRendererSettings,
    ) -> Option<SpritePipelinePlan> {
        plan.sprite_render_pass
            .as_ref()
            .map(|_| SpritePipelinePlan::for_settings(settings))
    }

    fn expected_sprite_resource_bindings(
        plan: &SceneDrawPlan,
    ) -> Option<SpriteResourceBindingPlan> {
        plan.sprite_pipeline.as_ref().and_then(|_| {
            plan.gpu_pass.scene_projection.and_then(|projection| {
                SpriteResourceBindingPlan::from_projection_and_atlas(
                    projection,
                    &TextureAtlas::default_sprites(),
                )
            })
        })
    }

    fn expected_sprite_pipeline_layout(plan: &SceneDrawPlan) -> Option<SpritePipelineLayoutPlan> {
        match (
            plan.sprite_pipeline.as_ref(),
            plan.sprite_resource_bindings.as_ref(),
            plan.gpu_pass.viewport,
        ) {
            (Some(_), Some(bindings), Some(_)) => {
                Some(SpritePipelineLayoutPlan::from_resource_bindings(bindings))
            }
            _ => None,
        }
    }

    fn expected_sprite_render_pipeline_descriptor(
        plan: &SceneDrawPlan,
    ) -> Option<SpriteRenderPipelineDescriptorPlan> {
        match (
            plan.sprite_render_pass.as_ref(),
            plan.sprite_pipeline.as_ref(),
            plan.sprite_resource_bindings.as_ref(),
            plan.sprite_pipeline_layout.as_ref(),
            plan.gpu_pass.viewport,
        ) {
            (Some(_), Some(pipeline), Some(_), Some(layout), Some(_)) => {
                Some(SpriteRenderPipelineDescriptorPlan::from_pipeline_and_layout(pipeline, layout))
            }
            _ => None,
        }
    }

    fn expected_sprite_render_pass_encoder(
        plan: &SceneDrawPlan,
    ) -> Option<SpriteRenderPassEncoderPlan> {
        match (
            plan.sprite_render_pass.as_ref(),
            plan.sprite_resource_bindings.as_ref(),
            plan.sprite_pipeline_layout.as_ref(),
            plan.sprite_render_pipeline_descriptor.as_ref(),
            plan.gpu_pass.viewport,
        ) {
            (Some(render_pass), Some(_), Some(layout), Some(descriptor), Some(_)) => Some(
                SpriteRenderPassEncoderPlan::from_render_pass_layout_and_descriptor(
                    render_pass,
                    layout,
                    descriptor,
                ),
            ),
            _ => None,
        }
    }

    fn expected_frame_plan(plan: &SceneDrawPlan) -> WgpuFramePlan {
        WgpuFramePlan::from_pass_raster_and_sprite_encoder(
            &plan.gpu_pass,
            plan.raster_upload,
            plan.sprite_render_pass_encoder.as_ref(),
        )
    }

    #[test]
    fn render_scene_collects_sprites_in_order() {
        let mut scene = RenderScene::empty(7, SurfaceSize::new(320, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId(3),
            layer: RenderLayer::Objects,
            position: [12.0, 24.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        assert_eq!(scene.frame, 7);
        assert_eq!(scene.sprites[0].sprite, SpriteId(3));
    }

    #[test]
    fn render_scene_summary_counts_layers_and_visual_signature() {
        let mut scene = RenderScene::empty(12, SurfaceSize::new(292, 240));
        scene.visual_signature = Some(0xCAFE_BABE);
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::SCORE_TEXT,
            layer: RenderLayer::Hud,
            position: [0.0, 0.0],
            size: [80.0, 8.0],
            tint: Color::WHITE,
        });

        let summary = scene.summary();

        assert_eq!(summary.visual_signature, Some(0xCAFE_BABE));
        assert_eq!(summary.raster_count, 0);
        assert_eq!(summary.sprite_count, 2);
        assert_eq!(
            summary.layers,
            RenderLayerCounts {
                objects: 1,
                hud: 1,
                ..RenderLayerCounts::default()
            }
        );
    }

    #[test]
    fn scene_raster_validates_rgba_payload_length() {
        let surface = SurfaceSize::new(2, 2);
        let pixels = vec![0; 15];

        assert_eq!(
            SceneRaster::from_rgba(surface, pixels).expect_err("invalid length"),
            SceneRasterError::PixelBufferLength {
                expected: 16,
                actual: 15
            }
        );
    }

    #[test]
    fn scene_raster_reports_oversized_surfaces_and_formats_errors() {
        let error = SceneRaster::from_rgba(SurfaceSize::new(u32::MAX, u32::MAX), Vec::new())
            .expect_err("oversized surface");

        assert_eq!(
            error,
            SceneRasterError::PixelBufferTooLarge {
                surface: SurfaceSize::new(u32::MAX, u32::MAX)
            }
        );
        assert_eq!(
            error.to_string(),
            "rgba buffer is too large for 4294967295x4294967295 surface"
        );
        assert_eq!(
            SceneRasterError::PixelBufferLength {
                expected: 8,
                actual: 4
            }
            .to_string(),
            "rgba buffer length mismatch: expected 8 bytes, got 4"
        );
    }

    #[test]
    fn render_scene_from_raster_keeps_temporary_fidelity_payload() {
        let scene = RenderScene::from_rgba(
            21,
            SurfaceSize::new(2, 1),
            vec![0, 0, 0, 255, 10, 20, 30, 255],
            Some(0x1234_5678),
        )
        .expect("raster scene");

        let raster = scene.raster().expect("scene raster");
        assert_eq!(raster.surface, SurfaceSize::new(2, 1));
        assert!(raster.is_non_blank());
        assert_eq!(scene.summary().raster_count, 1);
        assert_eq!(scene.summary().visual_signature, Some(0x1234_5678));
    }

    #[test]
    fn render_scene_can_replace_raster_payload_and_move_pixels_out() {
        let mut scene = RenderScene::empty(22, SurfaceSize::new(1, 1));
        let raster =
            SceneRaster::from_rgba(SurfaceSize::new(2, 1), vec![1, 2, 3, 255, 4, 5, 6, 255])
                .expect("replacement raster");

        scene.set_raster(raster.clone());

        assert_eq!(scene.surface, SurfaceSize::new(2, 1));
        assert_eq!(scene.summary().raster_count, 1);
        assert_eq!(raster.into_pixels(), vec![1, 2, 3, 255, 4, 5, 6, 255]);
    }

    #[test]
    fn render_scene_summary_counts_every_layer() {
        let mut scene = RenderScene::empty(13, SurfaceSize::new(292, 240));
        for (index, layer) in [
            RenderLayer::Terrain,
            RenderLayer::Starfield,
            RenderLayer::Objects,
            RenderLayer::Projectiles,
            RenderLayer::Hud,
            RenderLayer::Overlay,
        ]
        .into_iter()
        .enumerate()
        {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId(index as u16),
                layer,
                position: [index as f32, 0.0],
                size: [1.0, 1.0],
                tint: Color::WHITE,
            });
        }

        assert_eq!(
            scene.summary().layers,
            RenderLayerCounts {
                terrain: 1,
                starfield: 1,
                objects: 1,
                projectiles: 1,
                hud: 1,
                overlay: 1,
            }
        );
    }

    #[test]
    fn renderer_settings_are_wgpu_native() {
        let settings = GpuRendererSettings::default();

        assert_eq!(settings.texture_format, wgpu::TextureFormat::Rgba8UnormSrgb);
        assert_eq!(settings.present_mode, wgpu::PresentMode::AutoVsync);
    }

    #[test]
    fn native_scene_renderer_uses_settings_for_sprite_pipeline_plan() {
        let settings = GpuRendererSettings {
            texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            ..GpuRendererSettings::default()
        };
        let renderer =
            NativeSceneRenderer::with_settings(NativeRendererResources::default(), settings);
        let mut scene = RenderScene::empty(31, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = renderer.prepare(&scene);

        assert_eq!(
            plan.sprite_pipeline,
            Some(SpritePipelinePlan::for_settings(settings))
        );
        assert_eq!(
            plan.sprite_resource_bindings,
            expected_sprite_resource_bindings(&plan)
        );
        assert_eq!(
            plan.sprite_pipeline_layout,
            expected_sprite_pipeline_layout(&plan)
        );
        assert_eq!(
            plan.sprite_render_pipeline_descriptor,
            expected_sprite_render_pipeline_descriptor(&plan)
        );
        assert_eq!(
            plan.sprite_render_pass_encoder,
            expected_sprite_render_pass_encoder(&plan)
        );
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(
            plan.sprite_pipeline_layout
                .as_ref()
                .map(SpritePipelineLayoutPlan::bind_group_count),
            Some(2)
        );
        assert_eq!(
            plan.sprite_pipeline
                .as_ref()
                .map(|pipeline| pipeline.color_target.format),
            Some(wgpu::TextureFormat::Bgra8UnormSrgb)
        );
        assert_eq!(
            plan.sprite_render_pipeline_descriptor
                .as_ref()
                .map(|descriptor| descriptor.color_target.format),
            Some(wgpu::TextureFormat::Bgra8UnormSrgb)
        );
        assert_eq!(
            plan.sprite_render_pass_encoder
                .as_ref()
                .map(SpriteRenderPassEncoderPlan::draw_count),
            Some(1)
        );
        assert_eq!(plan.frame_plan.sprite_pass_count(), 1);
    }

    #[test]
    fn texture_atlas_owns_sprite_regions() {
        let atlas = TextureAtlas::new(
            SurfaceSize::new(16, 16),
            vec![AtlasRegion {
                sprite: SpriteId(42),
                origin: [1, 2],
                size: [3, 4],
            }],
        );

        assert!(atlas.contains(SpriteId(42)));
        assert!(!atlas.contains(SpriteId::PLAYER_SHIP));
        assert_eq!(atlas.pixels().len(), 16 * 16 * 4);
        assert!(!atlas.is_non_blank());
        assert_eq!(
            atlas.region(SpriteId(42)),
            Some(AtlasRegion {
                sprite: SpriteId(42),
                origin: [1, 2],
                size: [3, 4],
            })
        );
        assert_eq!(atlas.region(SpriteId::PLAYER_SHIP), None);
        let default_atlas = TextureAtlas::default_sprites();
        assert_eq!(default_atlas.pixels().len(), 128 * 192 * 4);
        assert!(default_atlas.is_non_blank());
        assert!(default_atlas.contains(SpriteId::STATUS_TEXT));
        assert!(default_atlas.contains(SpriteId::PLAYER_PROJECTILE));
        assert!(default_atlas.contains(SpriteId::TERRAIN_TILE));
        assert!(default_atlas.contains(SpriteId::TERRAIN_TILE_ALT));
        assert!(default_atlas.contains(SpriteId::STAR));
        assert!(default_atlas.contains(SpriteId::ENEMY_LANDER));
        assert!(default_atlas.contains(SpriteId::HUMAN));
        assert!(default_atlas.contains(SpriteId::ENEMY_MUTANT));
        assert!(default_atlas.contains(SpriteId::ENEMY_BAITER));
        assert!(default_atlas.contains(SpriteId::ENEMY_BOMBER));
        assert!(default_atlas.contains(SpriteId::ENEMY_POD));
        assert!(default_atlas.contains(SpriteId::ENEMY_SWARMER));
        assert!(default_atlas.contains(SpriteId::ENEMY_BOMB));
        assert!(default_atlas.contains(SpriteId::BOMB_EXPLOSION));
        assert!(default_atlas.contains(SpriteId::SWARMER_EXPLOSION));
        assert!(default_atlas.contains(SpriteId::SCORE_POPUP_250));
        assert!(default_atlas.contains(SpriteId::SCORE_POPUP_500));
        assert!(default_atlas.contains(SpriteId::PLAYER_LIFE_STOCK));
        assert!(default_atlas.contains(SpriteId::SMART_BOMB_STOCK));
        assert!(default_atlas.contains(SpriteId::ASTRONAUT_EXPLOSION));
        assert!(default_atlas.contains(SpriteId::NULL_OBJECT));
        assert!(default_atlas.contains(SpriteId::TERRAIN_EXPLOSION));
        for sprite in SpriteId::SCORE_DIGITS {
            assert!(default_atlas.contains(sprite));
        }
        assert!(default_atlas.contains(SpriteId::HALL_OF_FAME_UNDERLINE_WORD));
        assert!(default_atlas.contains(SpriteId::HALL_OF_FAME_DEFENDER_LOGO));
        assert!(default_atlas.contains(SpriteId::ATTRACT_COPYRIGHT_STRIP));
        assert!(default_atlas.contains(SpriteId::TOP_DISPLAY_BORDER_WORD));
        assert!(default_atlas.contains(SpriteId::SCANNER_OBJECT_BLIP));
        assert!(default_atlas.contains(SpriteId::SCANNER_PLAYER_BLIP));
        assert!(default_atlas.contains(SpriteId::PLAYER_EXPLOSION_PIXEL));
        assert_eq!(
            TextureAtlas::with_rgba(SurfaceSize::new(2, 2), Vec::new(), vec![0; 15]),
            Err(SceneRasterError::PixelBufferLength {
                expected: 16,
                actual: 15,
            })
        );
        assert_eq!(
            TextureAtlas::with_rgba(SurfaceSize::new(u32::MAX, u32::MAX), Vec::new(), Vec::new()),
            Err(SceneRasterError::PixelBufferTooLarge {
                surface: SurfaceSize::new(u32::MAX, u32::MAX),
            })
        );
    }

    #[test]
    fn default_sprite_atlas_uses_source_backed_runtime_regions() {
        let atlas = TextureAtlas::default_sprites();

        assert_non_placeholder_region(&atlas, SpriteId::PLAYER_SHIP);
        assert_non_placeholder_region(&atlas, SpriteId::SCORE_TEXT);
        assert_non_placeholder_region(&atlas, SpriteId::STATUS_TEXT);
        assert_visible_region(&atlas, SpriteId::PLAYER_PROJECTILE);
        assert_non_placeholder_region(&atlas, SpriteId::TERRAIN_TILE);
        assert_non_placeholder_region(&atlas, SpriteId::TERRAIN_TILE_ALT);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_LANDER);
        assert_non_placeholder_region(&atlas, SpriteId::HUMAN);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_MUTANT);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_BAITER);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_BOMBER);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_POD);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_SWARMER);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_BOMB);
        assert_non_placeholder_region(&atlas, SpriteId::BOMB_EXPLOSION);
        assert_non_placeholder_region(&atlas, SpriteId::SWARMER_EXPLOSION);
        assert_non_placeholder_region(&atlas, SpriteId::SCORE_POPUP_250);
        assert_non_placeholder_region(&atlas, SpriteId::SCORE_POPUP_500);
        assert_non_placeholder_region(&atlas, SpriteId::PLAYER_LIFE_STOCK);
        assert_non_placeholder_region(&atlas, SpriteId::SMART_BOMB_STOCK);
        assert_visible_region(&atlas, SpriteId::STAR);
    }

    #[test]
    fn default_sprite_atlas_decodes_source_terrain_word_patterns() {
        let atlas = TextureAtlas::default_sprites();
        let terrain_7007 = atlas_region_pixels(&atlas, SpriteId::TERRAIN_TILE);
        let terrain_0770 = atlas_region_pixels(&atlas, SpriteId::TERRAIN_TILE_ALT);
        let is_visible = |pixels: &[[u8; 4]], x: usize| pixels[x][3] != 0;

        assert_eq!(
            atlas.region(SpriteId::TERRAIN_TILE).expect("terrain").size,
            [8, 8]
        );
        assert_eq!(
            atlas
                .region(SpriteId::TERRAIN_TILE_ALT)
                .expect("terrain alt")
                .size,
            [8, 8]
        );
        assert_eq!(
            (0..8)
                .map(|x| is_visible(&terrain_7007, x))
                .collect::<Vec<_>>(),
            vec![true, true, false, false, false, false, true, true]
        );
        assert_eq!(
            (0..8)
                .map(|x| is_visible(&terrain_0770, x))
                .collect::<Vec<_>>(),
            vec![false, false, true, true, true, true, false, false]
        );
        assert_eq!(terrain_7007[0], pseudo_color_rgba(PICTURE_COLOR_TABLE[7]));
    }

    #[test]
    fn source_object_images_decode_arcade_bytes_and_palettes() {
        let ship = decode_source_object_image_rgba("PLD10", 6, 8, ObjectPicturePalette::ship());
        let shot =
            decode_source_object_image_rgba("LASD10", 1, 8, ObjectPicturePalette::player_shot());
        let human = decode_source_object_image_rgba("ASTD10", 8, 2, ObjectPicturePalette::white());

        assert_eq!(ship.surface, SurfaceSize::new(16, 6));
        assert!(
            ship.pixels
                .chunks_exact(4)
                .any(|pixel| pixel == PURPLE_RGBA.as_slice())
        );
        assert_eq!(shot.surface, SurfaceSize::new(16, 1));
        assert!(
            shot.pixels
                .chunks_exact(4)
                .all(|pixel| pixel == PALE_YELLOW_RGBA.as_slice())
        );
        assert_eq!(human.surface, SurfaceSize::new(4, 8));
        assert!(human.pixels.chunks_exact(4).any(|pixel| pixel[3] != 0));
    }

    #[test]
    fn default_sprite_atlas_uses_object_picture_grid_regions() {
        let atlas = TextureAtlas::default_sprites();

        assert_visible_region(&atlas, SpriteId::ASTRONAUT_EXPLOSION);
        assert_transparent_region(&atlas, SpriteId::NULL_OBJECT);
        assert_visible_region(&atlas, SpriteId::TERRAIN_EXPLOSION);
    }

    #[test]
    fn default_sprite_atlas_uses_score_digit_regions() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(SpriteId::score_digit(0), Some(SpriteId::SCORE_DIGIT_0));
        assert_eq!(SpriteId::score_digit(9), Some(SpriteId::SCORE_DIGIT_9));
        assert_eq!(SpriteId::score_digit(10), None);
        for (index, sprite) in SpriteId::SCORE_DIGITS.iter().enumerate() {
            assert_eq!(
                atlas.region(*sprite),
                Some(AtlasRegion {
                    sprite: *sprite,
                    origin: [u32::try_from(index).expect("digit index fits") * 8, 112],
                    size: [6, 8],
                })
            );
            assert_visible_region(&atlas, *sprite);
        }
    }

    #[test]
    fn default_sprite_atlas_decodes_score_digits_in_source_column_order() {
        let atlas = TextureAtlas::default_sprites();
        let pixels = atlas_region_pixels(&atlas, SpriteId::SCORE_DIGIT_0);
        let is_visible = |x: usize, y: usize| pixels[y * 6 + x][3] != 0;

        assert_eq!(pixels[1], WHITE_RGBA);
        assert_eq!(
            (0..6).map(|x| is_visible(x, 0)).collect::<Vec<_>>(),
            vec![false, true, true, true, true, true]
        );
        assert_eq!(
            (0..6).map(|x| is_visible(x, 1)).collect::<Vec<_>>(),
            vec![false, true, false, false, true, true]
        );
        assert_eq!(
            (0..6).map(|x| is_visible(x, 6)).collect::<Vec<_>>(),
            vec![false, true, true, true, true, true]
        );
        assert!((0..6).all(|x| !is_visible(x, 7)));
    }

    #[test]
    fn default_sprite_atlas_uses_high_score_underline_word_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::HALL_OF_FAME_UNDERLINE_WORD),
            Some(AtlasRegion {
                sprite: SpriteId::HALL_OF_FAME_UNDERLINE_WORD,
                origin: [80, 112],
                size: [2, 2],
            })
        );
        assert_visible_region(&atlas, SpriteId::HALL_OF_FAME_UNDERLINE_WORD);
    }

    #[test]
    fn default_sprite_atlas_uses_hall_of_fame_defender_logo_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::HALL_OF_FAME_DEFENDER_LOGO),
            Some(AtlasRegion {
                sprite: SpriteId::HALL_OF_FAME_DEFENDER_LOGO,
                origin: [0, 128],
                size: [120, 24],
            })
        );
        assert_visible_region(&atlas, SpriteId::HALL_OF_FAME_DEFENDER_LOGO);
    }

    #[test]
    fn default_sprite_atlas_uses_defender_wordmark_block_regions() {
        let atlas = TextureAtlas::default_sprites();
        let first = SpriteId::attract_defender_wordmark_block(0).expect("first block sprite");
        let center = SpriteId::attract_defender_wordmark_block(7).expect("center block sprite");
        let last = SpriteId::attract_defender_wordmark_block(
            ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT.saturating_sub(1),
        )
        .expect("last block sprite");

        assert_eq!(
            atlas.region(first),
            Some(AtlasRegion {
                sprite: first,
                origin: [0, 128],
                size: [8, 24],
            })
        );
        assert_eq!(
            atlas.region(center),
            Some(AtlasRegion {
                sprite: center,
                origin: [56, 128],
                size: [8, 24],
            })
        );
        assert_eq!(
            atlas.region(last),
            Some(AtlasRegion {
                sprite: last,
                origin: [112, 128],
                size: [8, 24],
            })
        );
        assert_visible_region(&atlas, first);
        assert_visible_region(&atlas, center);
    }

    #[test]
    fn default_sprite_atlas_uses_attract_copyright_strip_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::ATTRACT_COPYRIGHT_STRIP),
            Some(AtlasRegion {
                sprite: SpriteId::ATTRACT_COPYRIGHT_STRIP,
                origin: [0, 152],
                size: [80, 8],
            })
        );
        assert_visible_region(&atlas, SpriteId::ATTRACT_COPYRIGHT_STRIP);
    }

    #[test]
    fn default_sprite_atlas_uses_attract_williams_logo_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::ATTRACT_WILLIAMS_LOGO),
            Some(AtlasRegion {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO,
                origin: [0, 160],
                size: [92, 19],
            })
        );
        assert_visible_region(&atlas, SpriteId::ATTRACT_WILLIAMS_LOGO);
    }

    #[test]
    fn default_sprite_atlas_uses_top_display_border_word_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::TOP_DISPLAY_BORDER_WORD),
            Some(AtlasRegion {
                sprite: SpriteId::TOP_DISPLAY_BORDER_WORD,
                origin: [96, 160],
                size: [2, 2],
            })
        );
        assert_visible_region(&atlas, SpriteId::TOP_DISPLAY_BORDER_WORD);
    }

    #[test]
    fn default_sprite_atlas_uses_scanner_blip_regions() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::SCANNER_OBJECT_BLIP),
            Some(AtlasRegion {
                sprite: SpriteId::SCANNER_OBJECT_BLIP,
                origin: [100, 160],
                size: [2, 2],
            })
        );
        assert_eq!(
            atlas.region(SpriteId::SCANNER_PLAYER_BLIP),
            Some(AtlasRegion {
                sprite: SpriteId::SCANNER_PLAYER_BLIP,
                origin: [104, 160],
                size: [3, 2],
            })
        );
        assert_visible_region(&atlas, SpriteId::SCANNER_OBJECT_BLIP);
        assert_visible_region(&atlas, SpriteId::SCANNER_PLAYER_BLIP);
    }

    #[test]
    fn default_sprite_atlas_uses_player_explosion_pixel_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::PLAYER_EXPLOSION_PIXEL),
            Some(AtlasRegion {
                sprite: SpriteId::PLAYER_EXPLOSION_PIXEL,
                origin: [108, 160],
                size: [4, 2],
            })
        );
        assert_visible_region(&atlas, SpriteId::PLAYER_EXPLOSION_PIXEL);
    }

    #[test]
    fn default_sprite_atlas_uses_attract_williams_logo_pixel_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL),
            Some(AtlasRegion {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
                origin: [112, 160],
                size: [1, 1],
            })
        );
        assert_visible_region(&atlas, SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL);
    }

    #[test]
    fn default_sprite_atlas_uses_attract_scanner_terrain_pixel_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL),
            Some(AtlasRegion {
                sprite: SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL,
                origin: [114, 160],
                size: [1, 1],
            })
        );
        assert_visible_region(&atlas, SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL);
    }

    #[test]
    fn source_attract_williams_logo_decodes_table_pixels() {
        let sprite = decode_source_attract_williams_logo_rgba();
        let path = source_attract_williams_logo_pixel_path();
        let operation_counts = source_attract_williams_logo_operation_pixel_counts();

        assert_eq!(sprite.surface, SurfaceSize::new(92, 19));
        assert_eq!(path.len(), 660);
        assert!(!operation_counts.is_empty());
        assert_eq!(operation_counts.last().copied(), Some(path.len()));
        assert!(
            operation_counts
                .windows(2)
                .all(|window| window[0] <= window[1])
        );
        assert!(
            operation_counts
                .get(2)
                .is_some_and(|count| *count < path.len())
        );
        assert_eq!(path.first().copied(), Some([8, 4]));
        assert!(path.contains(&[8, 4]));
        assert!(path.contains(&[89, 9]));
        assert_eq!(
            sprite
                .pixels
                .chunks_exact(4)
                .filter(|pixel| pixel[3] != 0)
                .count(),
            660
        );
        assert_eq!(source_sprite_pixel(&sprite, 8, 4), WHITE_RGBA);
        assert_eq!(source_sprite_pixel(&sprite, 89, 9), WHITE_RGBA);
        assert_eq!(source_sprite_pixel(&sprite, 0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn default_sprite_atlas_uses_message_glyph_regions() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(source_message_text("PLYR1"), Some("PLAYER ONE"));
        assert_eq!(source_message_text("PLYR2"), Some("PLAYER TWO"));
        assert_eq!(source_message_text("GO"), Some("GAME OVER"));
        assert_eq!(source_message_text("MISSING"), None);
        assert_eq!(source_screen_position(0x3C78), [120.0, 120.0]);
        assert_eq!(
            source_screen_position_with_offset(0x1458, 0, 0x0A),
            [40.0, 98.0]
        );
        assert_eq!(
            SpriteId::message_glyph('P'),
            Some(SpriteId::MESSAGE_GLYPH_P)
        );
        assert_eq!(
            SpriteId::message_glyph('W'),
            Some(SpriteId::MESSAGE_GLYPH_W)
        );
        assert_eq!(SpriteId::message_glyph('a'), None);
        assert_eq!(SpriteId::message_glyph_size('P'), Some([6, 8]));
        assert_eq!(SpriteId::message_glyph_size('W'), Some([8, 8]));

        for sprite in SpriteId::MESSAGE_GLYPHS {
            assert!(atlas.contains(sprite));
        }
        assert_transparent_region(&atlas, SpriteId::MESSAGE_GLYPH_SPACE);
        assert_visible_region(&atlas, SpriteId::MESSAGE_GLYPH_P);
        assert_visible_region(&atlas, SpriteId::MESSAGE_GLYPH_G);
        assert_visible_region(&atlas, SpriteId::MESSAGE_GLYPH_W);
    }

    #[test]
    fn source_text_bytes_render_mixed_score_digits_and_message_glyphs() {
        let mut scene = RenderScene::empty(0, SurfaceSize::new(292, 240));

        push_source_text_bytes_sprites(
            &mut scene,
            b" 2A",
            source_screen_position(0x2B86),
            RenderLayer::Overlay,
        );

        assert_eq!(scene.sprites.len(), 2);
        assert_eq!(scene.sprites[0].sprite, SpriteId::SCORE_DIGIT_2);
        assert_eq!(scene.sprites[0].position, [90.0, 134.0]);
        assert_eq!(scene.sprites[0].size, [6.0, 8.0]);
        assert_eq!(scene.sprites[1].sprite, SpriteId::MESSAGE_GLYPH_A);
        assert_eq!(scene.sprites[1].position, [98.0, 134.0]);
        assert_eq!(scene.sprites[1].size, [6.0, 8.0]);
        assert!(
            scene
                .sprites
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Overlay)
        );
    }

    #[test]
    fn source_controlled_message_sprites_apply_source_cursor_controls() {
        let mut scene = RenderScene::empty(0, SurfaceSize::new(292, 240));
        let text = source_message_text("ELECV").expect("ELECV message text");

        push_source_controlled_message_sprites(&mut scene, text, 0x3258, RenderLayer::Overlay);

        assert_eq!(scene.sprites.len(), 23);
        assert!(scene.sprites.iter().all(|sprite| {
            sprite.layer == RenderLayer::Overlay
                && SpriteId::MESSAGE_GLYPHS.contains(&sprite.sprite)
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E
                && sprite.position == [100.0, 88.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_I
                && sprite.position == [190.0, 88.0]
                && sprite.size == [4.0, 8.0]
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_PERIOD
                && sprite.position == [212.0, 88.0]
                && sprite.size == [2.0, 8.0]
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.position == [124.0, 108.0]
                && sprite.size == [6.0, 8.0]
        }));
    }

    #[test]
    fn object_picture_labels_map_reclassified_clean_sprite_assets() {
        for label in ["PLAPIC", "PLBPIC"] {
            assert_eq!(
                SpriteId::for_object_picture_label(label),
                Some(SpriteId::PLAYER_SHIP)
            );
        }
        for label in ["LNDP1", "LNDP2", "LNDP3"] {
            assert_eq!(
                SpriteId::for_object_picture_label(label),
                Some(SpriteId::ENEMY_LANDER)
            );
        }
        for label in ["ASTP1", "ASTP2", "ASTP3", "ASTP4"] {
            assert_eq!(
                SpriteId::for_object_picture_label(label),
                Some(SpriteId::HUMAN)
            );
        }
        assert_eq!(
            SpriteId::for_object_picture_label("LASP1"),
            Some(SpriteId::PLAYER_PROJECTILE)
        );
        assert_eq!(
            SpriteId::for_object_picture_label("SCZP1"),
            Some(SpriteId::ENEMY_MUTANT)
        );
        for label in ["UFOP1", "UFOP2", "UFOP3"] {
            assert_eq!(
                SpriteId::for_object_picture_label(label),
                Some(SpriteId::ENEMY_BAITER)
            );
        }
        for label in ["TIEP1", "TIEP2", "TIEP3", "TIEP4"] {
            assert_eq!(
                SpriteId::for_object_picture_label(label),
                Some(SpriteId::ENEMY_BOMBER)
            );
        }
        assert_eq!(
            SpriteId::for_object_picture_label("PRBP1"),
            Some(SpriteId::ENEMY_POD)
        );
        assert_eq!(
            SpriteId::for_object_picture_label("SWPIC1"),
            Some(SpriteId::ENEMY_SWARMER)
        );
        for label in ["BMBP1", "BMBP2"] {
            assert_eq!(
                SpriteId::for_object_picture_label(label),
                Some(SpriteId::ENEMY_BOMB)
            );
        }
        assert_eq!(
            SpriteId::for_object_picture_label("BXPIC"),
            Some(SpriteId::BOMB_EXPLOSION)
        );
        assert_eq!(
            SpriteId::for_object_picture_label("SWXP1"),
            Some(SpriteId::SWARMER_EXPLOSION)
        );
        assert_eq!(
            SpriteId::for_object_picture_label("C25P1"),
            Some(SpriteId::SCORE_POPUP_250)
        );
        assert_eq!(
            SpriteId::for_object_picture_label("C5P1"),
            Some(SpriteId::SCORE_POPUP_500)
        );
        assert_eq!(
            SpriteId::for_object_picture_label("PLAMIN"),
            Some(SpriteId::PLAYER_LIFE_STOCK)
        );
        assert_eq!(
            SpriteId::for_object_picture_label("SBPIC"),
            Some(SpriteId::SMART_BOMB_STOCK)
        );
        assert_eq!(
            SpriteId::for_object_picture_label("ASXP1"),
            Some(SpriteId::ASTRONAUT_EXPLOSION)
        );
        assert_eq!(
            SpriteId::for_object_picture_label("NULOB"),
            Some(SpriteId::NULL_OBJECT)
        );
        assert_eq!(
            SpriteId::for_object_picture_label("TEREX"),
            Some(SpriteId::TERRAIN_EXPLOSION)
        );
        assert_eq!(SpriteId::for_object_picture_label("UNKNOWN"), None);
    }

    #[test]
    fn embedded_png_decoder_rejects_truncated_or_unsupported_assets() {
        let rgba_png = encode_test_png(png::ColorType::Rgba, &[255, 0, 0, 255]);
        let truncated = truncate_first_idat_payload(rgba_png);

        assert_panic_message(
            || {
                let _ = super::decode_embedded_png_rgba("truncated-test", &truncated);
            },
            "must contain a frame",
        );

        let rgb_png = encode_test_png(png::ColorType::Rgb, &[255, 0, 0]);
        assert_panic_message(
            || {
                let _ = super::decode_embedded_png_rgba("rgb-test", &rgb_png);
            },
            "must be 8-bit RGBA",
        );
    }

    #[test]
    fn default_sprite_blitters_skip_missing_or_empty_sources() {
        let atlas_surface = SurfaceSize::new(2, 2);
        let source = super::EmbeddedSprite {
            surface: SurfaceSize::new(1, 1),
            pixels: vec![10, 20, 30, 255],
        };
        let mut atlas_pixels = vec![0; atlas_surface.rgba_len().expect("atlas byte length")];
        let unchanged = atlas_pixels.clone();

        super::blit_default_region_from_source(
            &mut atlas_pixels,
            atlas_surface,
            &[],
            SpriteId::PLAYER_SHIP,
            &source,
            super::SpriteAssetSource {
                origin: [0, 0],
                size: [1, 1],
            },
        );
        assert_eq!(atlas_pixels, unchanged);

        super::blit_scaled_region(
            &mut atlas_pixels,
            atlas_surface,
            AtlasRegion {
                sprite: SpriteId::PLAYER_SHIP,
                origin: [0, 0],
                size: [0, 1],
            },
            &source,
            super::SpriteAssetSource {
                origin: [0, 0],
                size: [1, 1],
            },
        );
        assert_eq!(atlas_pixels, unchanged);

        super::copy_source_pixel(&mut atlas_pixels, atlas_surface, [0, 0], &source, [1, 0]);
        assert_eq!(atlas_pixels, unchanged);
    }

    #[test]
    fn star_blitter_skips_missing_region_or_transparent_source() {
        let atlas_surface = SurfaceSize::new(2, 2);
        let source = super::EmbeddedSprite {
            surface: SurfaceSize::new(1, 1),
            pixels: vec![10, 20, 30, 255],
        };
        let mut atlas_pixels = vec![0; atlas_surface.rgba_len().expect("atlas byte length")];
        let unchanged = atlas_pixels.clone();

        super::blit_star_region(&mut atlas_pixels, atlas_surface, &[], &source);
        assert_eq!(atlas_pixels, unchanged);

        let transparent_source = super::EmbeddedSprite {
            surface: SurfaceSize::new(1, 1),
            pixels: vec![10, 20, 30, 0],
        };
        super::blit_star_region(
            &mut atlas_pixels,
            atlas_surface,
            &[AtlasRegion {
                sprite: SpriteId::STAR,
                origin: [0, 0],
                size: [1, 1],
            }],
            &transparent_source,
        );
        assert_eq!(atlas_pixels, unchanged);
    }

    fn encode_test_png(color_type: png::ColorType, pixels: &[u8]) -> Vec<u8> {
        let mut png_bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_bytes, 1, 1);
            encoder.set_color(color_type);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("write png header");
            writer.write_image_data(pixels).expect("write png data");
        }
        png_bytes
    }

    fn truncate_first_idat_payload(mut png_bytes: Vec<u8>) -> Vec<u8> {
        let mut offset = 8;
        while offset + 8 <= png_bytes.len() {
            let length = u32::from_be_bytes(
                png_bytes[offset..offset + 4]
                    .try_into()
                    .expect("png chunk length"),
            ) as usize;
            let kind = &png_bytes[offset + 4..offset + 8];
            if kind == b"IDAT" {
                assert!(length > 1);
                png_bytes.truncate(offset + 8 + length - 1);
                return png_bytes;
            }
            offset += 12 + length;
        }
        panic!("test png must contain IDAT");
    }

    fn assert_panic_message(run: impl FnOnce() + std::panic::UnwindSafe, expected: &str) {
        let panic = std::panic::catch_unwind(run).expect_err("expected panic");
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&'static str>().copied())
            .expect("panic message");
        assert!(
            message.contains(expected),
            "panic message {message:?} must contain {expected:?}"
        );
    }

    fn assert_non_placeholder_region(atlas: &TextureAtlas, sprite: SpriteId) {
        let pixels = atlas_region_pixels(atlas, sprite);
        assert!(
            pixels.iter().any(|pixel| pixel[3] != 0),
            "sprite {:?} region must contain visible pixels",
            sprite
        );
        let distinct_pixels = pixels
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            distinct_pixels.len() > 1,
            "sprite {:?} region must not be one solid placeholder color",
            sprite
        );
    }

    fn assert_visible_region(atlas: &TextureAtlas, sprite: SpriteId) {
        assert!(
            atlas_region_pixels(atlas, sprite)
                .iter()
                .any(|pixel| pixel[3] != 0),
            "sprite {:?} region must contain visible pixels",
            sprite
        );
    }

    fn assert_transparent_region(atlas: &TextureAtlas, sprite: SpriteId) {
        assert!(
            atlas_region_pixels(atlas, sprite)
                .iter()
                .all(|pixel| pixel[3] == 0),
            "sprite {:?} region must stay transparent",
            sprite
        );
    }

    fn source_sprite_pixel(sprite: &EmbeddedSprite, x: u32, y: u32) -> [u8; 4] {
        let start = ((y as usize * sprite.surface.width as usize) + x as usize) * 4;
        let pixel = &sprite.pixels[start..start + 4];
        [pixel[0], pixel[1], pixel[2], pixel[3]]
    }

    fn atlas_region_pixels(atlas: &TextureAtlas, sprite: SpriteId) -> Vec<[u8; 4]> {
        let region = atlas.region(sprite).expect("sprite region");
        let mut pixels = Vec::new();
        for y in region.origin[1]..region.origin[1] + region.size[1] {
            for x in region.origin[0]..region.origin[0] + region.size[0] {
                let start = ((y as usize * atlas.surface.width as usize) + x as usize) * 4;
                let pixel = &atlas.pixels()[start..start + 4];
                pixels.push([pixel[0], pixel[1], pixel[2], pixel[3]]);
            }
        }
        pixels
    }

    #[test]
    fn native_scene_renderer_builds_draw_plan_from_scene_layers() {
        let mut scene = RenderScene::empty(34, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::SCORE_TEXT,
            layer: RenderLayer::Hud,
            position: [0.0, 0.0],
            size: [80.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = NativeSceneRenderer::default().prepare(&scene);

        assert_eq!(plan.frame, 34);
        assert_eq!(
            plan.viewport,
            ViewportLayout {
                scene: SurfaceSize::new(292, 240),
                target: SurfaceSize::new(292, 240),
                origin: [0, 0],
                size: SurfaceSize::new(292, 240),
                scale: 1.0,
            }
        );
        assert_eq!(
            plan.gpu_pass,
            WgpuPassPlan {
                clear_color: wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
                viewport: Some(WgpuViewportCommand {
                    x: 0.0,
                    y: 0.0,
                    width: 292.0,
                    height: 240.0,
                    min_depth: 0.0,
                    max_depth: 1.0,
                }),
                scene_projection: SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)),
            }
        );
        assert_eq!(plan.sprite_instances, 2);
        assert_eq!(plan.missing_sprite_regions, 0);
        assert_eq!(
            plan.pipelines,
            vec![NativeRenderPipeline::Sprites, NativeRenderPipeline::HudText]
        );
        assert_eq!(
            plan.layer_counts,
            RenderLayerCounts {
                objects: 1,
                hud: 1,
                ..RenderLayerCounts::default()
            }
        );
        assert_eq!(
            plan.sprite_batches,
            vec![
                SpriteDrawBatch {
                    pipeline: NativeRenderPipeline::Sprites,
                    layer: RenderLayer::Objects,
                    instances: vec![SpriteDrawInstance {
                        sprite: SpriteId::PLAYER_SHIP,
                        atlas_origin: [0, 0],
                        atlas_size: [16, 8],
                        layer: RenderLayer::Objects,
                        position: [128.0, 96.0],
                        size: [16.0, 8.0],
                        tint: Color::WHITE,
                    }],
                },
                SpriteDrawBatch {
                    pipeline: NativeRenderPipeline::HudText,
                    layer: RenderLayer::Hud,
                    instances: vec![SpriteDrawInstance {
                        sprite: SpriteId::SCORE_TEXT,
                        atlas_origin: [0, 16],
                        atlas_size: [80, 8],
                        layer: RenderLayer::Hud,
                        position: [0.0, 0.0],
                        size: [80.0, 8.0],
                        tint: Color::WHITE,
                    }],
                },
            ]
        );
        assert_eq!(
            plan.sprite_instance_buffers,
            vec![
                SpriteInstanceBuffer {
                    pipeline: NativeRenderPipeline::Sprites,
                    layer: RenderLayer::Objects,
                    records: vec![SpriteInstanceBufferRecord {
                        scene_origin: [128.0, 96.0],
                        scene_size: [16.0, 8.0],
                        atlas_uv_origin: [0.0, 0.0],
                        atlas_uv_size: [0.125, 0.041666668],
                        tint: [1.0, 1.0, 1.0, 1.0],
                    }],
                },
                SpriteInstanceBuffer {
                    pipeline: NativeRenderPipeline::HudText,
                    layer: RenderLayer::Hud,
                    records: vec![SpriteInstanceBufferRecord {
                        scene_origin: [0.0, 0.0],
                        scene_size: [80.0, 8.0],
                        atlas_uv_origin: [0.0, 0.083333336],
                        atlas_uv_size: [0.625, 0.041666668],
                        tint: [1.0, 1.0, 1.0, 1.0],
                    }],
                },
            ]
        );
        assert_eq!(
            plan.sprite_instance_upload,
            Some(SpriteInstanceUpload {
                records: plan
                    .sprite_instance_buffers
                    .iter()
                    .flat_map(|buffer| buffer.records.iter().copied())
                    .collect(),
            })
        );
        assert_eq!(
            plan.sprite_buffer_uploads,
            plan.sprite_instance_upload
                .as_ref()
                .map(SpriteBufferUploadPlan::from_instance_upload)
        );
        assert_eq!(
            plan.sprite_draw_commands,
            vec![
                expected_sprite_draw_command(
                    NativeRenderPipeline::Sprites,
                    RenderLayer::Objects,
                    0,
                    1,
                ),
                expected_sprite_draw_command(NativeRenderPipeline::HudText, RenderLayer::Hud, 1, 1),
            ]
        );
        assert_eq!(plan.sprite_render_pass, expected_sprite_render_pass(&plan));
        assert_eq!(
            plan.sprite_pipeline,
            expected_sprite_pipeline(&plan, GpuRendererSettings::default())
        );
        assert_eq!(
            plan.sprite_resource_bindings,
            expected_sprite_resource_bindings(&plan)
        );
        assert_eq!(
            plan.sprite_pipeline_layout,
            expected_sprite_pipeline_layout(&plan)
        );
        assert_eq!(
            plan.sprite_render_pipeline_descriptor,
            expected_sprite_render_pipeline_descriptor(&plan)
        );
        assert_eq!(
            plan.sprite_render_pass_encoder,
            expected_sprite_render_pass_encoder(&plan)
        );
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(
            plan.sprite_render_pipeline_descriptor
                .as_ref()
                .map(|descriptor| descriptor.vertex_buffer_layouts().len()),
            Some(2)
        );
        assert_eq!(
            plan.sprite_render_pass_encoder
                .as_ref()
                .map(SpriteRenderPassEncoderPlan::command_count),
            Some(8)
        );
        assert_eq!(plan.frame_plan.command_count(), 4);
        assert_eq!(plan.frame_plan.sprite_pass_count(), 1);
        assert_eq!(plan.frame_plan.temporary_raster_count(), 0);
        assert_eq!(plan.raster_upload, None);
    }

    #[test]
    fn native_scene_renderer_respects_available_resource_pipelines() {
        let mut resources = NativeRendererResources::default();
        resources
            .pipelines
            .remove(&NativeRenderPipeline::TemporaryRaster);
        let renderer = NativeSceneRenderer::new(resources);
        let scene = RenderScene::from_rgba(1, SurfaceSize::new(1, 1), vec![0, 0, 0, 255], None)
            .expect("raster scene");

        let plan = renderer.prepare(&scene);

        assert_eq!(plan.pipelines, Vec::<NativeRenderPipeline>::new());
        assert!(plan.raster_upload.is_some());
        assert_eq!(
            plan.sprite_instance_buffers,
            Vec::<SpriteInstanceBuffer>::new()
        );
        assert_eq!(plan.sprite_instance_upload, None);
        assert_eq!(plan.sprite_buffer_uploads, None);
        assert_eq!(plan.sprite_draw_commands, Vec::<SpriteDrawCommand>::new());
        assert_eq!(plan.sprite_render_pass, None);
        assert_eq!(plan.sprite_pipeline, None);
        assert_eq!(plan.sprite_resource_bindings, None);
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
        assert_eq!(plan.frame_plan.temporary_raster_count(), 1);
    }

    #[test]
    fn native_scene_renderer_skips_sprite_commands_for_unavailable_sprite_pipelines() {
        let mut resources = NativeRendererResources::default();
        resources.pipelines.remove(&NativeRenderPipeline::Sprites);
        let renderer = NativeSceneRenderer::new(resources);
        let mut scene = RenderScene::empty(35, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = renderer.prepare(&scene);

        assert_eq!(plan.sprite_instances, 0);
        assert_eq!(plan.sprite_batches, Vec::<SpriteDrawBatch>::new());
        assert_eq!(
            plan.sprite_instance_buffers,
            Vec::<SpriteInstanceBuffer>::new()
        );
        assert_eq!(plan.sprite_instance_upload, None);
        assert_eq!(plan.sprite_buffer_uploads, None);
        assert_eq!(plan.sprite_draw_commands, Vec::<SpriteDrawCommand>::new());
        assert_eq!(plan.sprite_render_pass, None);
        assert_eq!(plan.sprite_pipeline, None);
        assert_eq!(plan.sprite_resource_bindings, None);
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
        assert_eq!(plan.pipelines, Vec::<NativeRenderPipeline>::new());
        assert_eq!(
            plan.layer_counts,
            RenderLayerCounts {
                objects: 1,
                ..RenderLayerCounts::default()
            }
        );
    }

    #[test]
    fn native_scene_renderer_maps_all_domain_layers_to_pipelines() {
        let mut scene = RenderScene::empty(80, SurfaceSize::new(292, 240));
        for (layer, sprite) in [
            (RenderLayer::Terrain, SpriteId::TERRAIN_TILE),
            (RenderLayer::Starfield, SpriteId::STAR),
            (RenderLayer::Projectiles, SpriteId::PLAYER_PROJECTILE),
        ] {
            scene.push_sprite(SceneSprite {
                sprite,
                layer,
                position: [0.0, 0.0],
                size: [1.0, 1.0],
                tint: Color::WHITE,
            });
        }

        let plan = NativeSceneRenderer::default().prepare(&scene);

        assert_eq!(
            plan.pipelines,
            vec![
                NativeRenderPipeline::Terrain,
                NativeRenderPipeline::Starfield,
                NativeRenderPipeline::Projectiles
            ]
        );
    }

    #[test]
    fn native_scene_renderer_counts_missing_sprite_atlas_regions() {
        let mut scene = RenderScene::empty(81, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId(900),
            layer: RenderLayer::Objects,
            position: [12.0, 34.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = NativeSceneRenderer::default().prepare(&scene);

        assert_eq!(plan.sprite_instances, 0);
        assert_eq!(plan.missing_sprite_regions, 1);
        assert_eq!(plan.sprite_batches, Vec::<SpriteDrawBatch>::new());
        assert_eq!(
            plan.sprite_instance_buffers,
            Vec::<SpriteInstanceBuffer>::new()
        );
        assert_eq!(plan.sprite_instance_upload, None);
        assert_eq!(plan.sprite_buffer_uploads, None);
        assert_eq!(plan.sprite_draw_commands, Vec::<SpriteDrawCommand>::new());
        assert_eq!(plan.sprite_render_pass, None);
        assert_eq!(plan.sprite_pipeline, None);
        assert_eq!(plan.sprite_resource_bindings, None);
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
        assert_eq!(plan.pipelines, Vec::<NativeRenderPipeline>::new());
        assert_eq!(
            plan.layer_counts,
            RenderLayerCounts {
                objects: 1,
                ..RenderLayerCounts::default()
            }
        );
    }

    #[test]
    fn native_scene_renderer_batches_multiple_sprites_by_pipeline_and_layer() {
        let mut scene = RenderScene::empty(82, SurfaceSize::new(292, 240));
        for position in [[2.0, 4.0], [10.0, 4.0]] {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::PLAYER_PROJECTILE,
                layer: RenderLayer::Projectiles,
                position,
                size: [8.0, 2.0],
                tint: Color::WHITE,
            });
        }

        let plan = NativeSceneRenderer::default().prepare(&scene);

        assert_eq!(plan.sprite_instances, 2);
        assert_eq!(plan.sprite_batches.len(), 1);
        assert_eq!(
            plan.sprite_batches[0].pipeline,
            NativeRenderPipeline::Projectiles
        );
        assert_eq!(plan.sprite_batches[0].layer, RenderLayer::Projectiles);
        assert_eq!(plan.sprite_batches[0].instances.len(), 2);
        assert_eq!(plan.sprite_batches[0].instances[0].atlas_origin, [0, 48]);
        assert_eq!(plan.sprite_batches[0].instances[0].atlas_size, [8, 2]);
        assert_eq!(plan.sprite_batches[0].instances[1].position, [10.0, 4.0]);
        assert_eq!(plan.sprite_instance_buffers.len(), 1);
        assert_eq!(plan.sprite_instance_buffers[0].records.len(), 2);
        assert_eq!(
            plan.sprite_instance_buffers[0].records[1].scene_origin,
            [10.0, 4.0]
        );
        assert_eq!(
            plan.sprite_instance_upload,
            Some(SpriteInstanceUpload {
                records: plan.sprite_instance_buffers[0].records.clone(),
            })
        );
        assert_eq!(
            plan.sprite_buffer_uploads,
            plan.sprite_instance_upload
                .as_ref()
                .map(SpriteBufferUploadPlan::from_instance_upload)
        );
        assert_eq!(
            plan.sprite_draw_commands,
            vec![expected_sprite_draw_command(
                NativeRenderPipeline::Projectiles,
                RenderLayer::Projectiles,
                0,
                2,
            )]
        );
        assert_eq!(plan.sprite_render_pass, expected_sprite_render_pass(&plan));
        assert_eq!(
            plan.sprite_pipeline,
            expected_sprite_pipeline(&plan, GpuRendererSettings::default())
        );
        assert_eq!(
            plan.sprite_resource_bindings,
            expected_sprite_resource_bindings(&plan)
        );
        assert_eq!(
            plan.sprite_pipeline_layout,
            expected_sprite_pipeline_layout(&plan)
        );
        assert_eq!(
            plan.sprite_render_pipeline_descriptor,
            expected_sprite_render_pipeline_descriptor(&plan)
        );
        assert_eq!(
            plan.sprite_render_pass_encoder,
            expected_sprite_render_pass_encoder(&plan)
        );
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 1);
    }

    #[test]
    fn native_scene_renderer_skips_instance_buffers_when_atlas_surface_is_empty() {
        let resources = NativeRendererResources {
            atlas: TextureAtlas::new(
                SurfaceSize::new(0, 128),
                vec![AtlasRegion {
                    sprite: SpriteId::PLAYER_SHIP,
                    origin: [0, 0],
                    size: [16, 8],
                }],
            ),
            ..NativeRendererResources::default()
        };
        let mut scene = RenderScene::empty(86, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = NativeSceneRenderer::new(resources).prepare(&scene);

        assert_eq!(plan.sprite_instances, 1);
        assert_eq!(plan.sprite_batches.len(), 1);
        assert_eq!(
            plan.sprite_instance_buffers,
            Vec::<SpriteInstanceBuffer>::new()
        );
        assert_eq!(plan.sprite_instance_upload, None);
        assert_eq!(plan.sprite_buffer_uploads, None);
        assert_eq!(plan.sprite_draw_commands, Vec::<SpriteDrawCommand>::new());
        assert_eq!(plan.sprite_render_pass, None);
        assert_eq!(plan.sprite_pipeline, None);
        assert_eq!(plan.sprite_resource_bindings, None);
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
    }

    #[test]
    fn native_scene_renderer_uses_target_viewport_for_sprite_and_raster_plans() {
        let mut sprite_scene = RenderScene::empty(83, SurfaceSize::new(292, 240));
        sprite_scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });
        let mut pixels = vec![0; 292 * 240 * 4];
        pixels[0] = 1;
        pixels[3] = 255;
        let raster_scene =
            RenderScene::from_rgba(84, SurfaceSize::new(292, 240), pixels, Some(0xCAFE_BABE))
                .expect("raster scene");
        let target = SurfaceSize::new(640, 480);
        let expected = ViewportLayout {
            scene: SurfaceSize::new(292, 240),
            target,
            origin: [28, 0],
            size: SurfaceSize::new(584, 480),
            scale: 2.0,
        };
        let renderer = NativeSceneRenderer::default();

        let sprite_plan = renderer.prepare_for_target(&sprite_scene, target);
        let raster_plan = renderer.prepare_for_target(&raster_scene, target);

        assert_eq!(sprite_plan.viewport, expected);
        assert_eq!(raster_plan.viewport, expected);
        assert_eq!(
            sprite_plan.gpu_pass.viewport,
            Some(WgpuViewportCommand {
                x: 28.0,
                y: 0.0,
                width: 584.0,
                height: 480.0,
                min_depth: 0.0,
                max_depth: 1.0,
            })
        );
        assert_eq!(sprite_plan.gpu_pass, raster_plan.gpu_pass);
        assert_eq!(sprite_plan.sprite_instances, 1);
        assert_eq!(sprite_plan.sprite_instance_buffers.len(), 1);
        assert_eq!(
            sprite_plan.sprite_instance_upload,
            Some(SpriteInstanceUpload {
                records: sprite_plan.sprite_instance_buffers[0].records.clone(),
            })
        );
        assert_eq!(
            sprite_plan.sprite_buffer_uploads,
            sprite_plan
                .sprite_instance_upload
                .as_ref()
                .map(SpriteBufferUploadPlan::from_instance_upload)
        );
        assert_eq!(
            sprite_plan.sprite_draw_commands,
            vec![expected_sprite_draw_command(
                NativeRenderPipeline::Sprites,
                RenderLayer::Objects,
                0,
                1,
            )]
        );
        assert_eq!(
            sprite_plan.sprite_render_pass,
            expected_sprite_render_pass(&sprite_plan)
        );
        assert_eq!(
            sprite_plan.sprite_pipeline,
            expected_sprite_pipeline(&sprite_plan, GpuRendererSettings::default())
        );
        assert_eq!(
            sprite_plan.sprite_resource_bindings,
            expected_sprite_resource_bindings(&sprite_plan)
        );
        assert_eq!(
            sprite_plan.sprite_pipeline_layout,
            expected_sprite_pipeline_layout(&sprite_plan)
        );
        assert_eq!(
            sprite_plan.sprite_render_pipeline_descriptor,
            expected_sprite_render_pipeline_descriptor(&sprite_plan)
        );
        assert_eq!(
            sprite_plan.sprite_render_pass_encoder,
            expected_sprite_render_pass_encoder(&sprite_plan)
        );
        assert_eq!(sprite_plan.frame_plan, expected_frame_plan(&sprite_plan));
        assert_eq!(sprite_plan.frame_plan.sprite_pass_count(), 1);
        assert_eq!(
            raster_plan.sprite_instance_buffers,
            Vec::<SpriteInstanceBuffer>::new()
        );
        assert_eq!(raster_plan.sprite_instance_upload, None);
        assert_eq!(raster_plan.sprite_buffer_uploads, None);
        assert_eq!(
            raster_plan.sprite_draw_commands,
            Vec::<SpriteDrawCommand>::new()
        );
        assert_eq!(raster_plan.sprite_render_pass, None);
        assert_eq!(raster_plan.sprite_pipeline, None);
        assert_eq!(raster_plan.sprite_resource_bindings, None);
        assert_eq!(raster_plan.sprite_pipeline_layout, None);
        assert_eq!(raster_plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(raster_plan.sprite_render_pass_encoder, None);
        assert_eq!(raster_plan.frame_plan, expected_frame_plan(&raster_plan));
        assert_eq!(raster_plan.frame_plan.sprite_pass_count(), 0);
        assert_eq!(raster_plan.frame_plan.temporary_raster_count(), 1);
        assert_eq!(
            raster_plan.raster_upload,
            Some(super::SceneRasterUpload {
                surface: SurfaceSize::new(292, 240),
                byte_len: 292 * 240 * 4,
                visual_signature: Some(0xCAFE_BABE),
                non_blank: true,
            })
        );
    }

    #[test]
    fn native_scene_renderer_omits_gpu_viewport_for_empty_target() {
        let scene = RenderScene::empty(85, SurfaceSize::new(292, 240));

        let plan =
            NativeSceneRenderer::default().prepare_for_target(&scene, SurfaceSize::new(0, 0));

        assert!(plan.viewport.is_empty());
        assert_eq!(plan.gpu_pass.viewport, None);
        assert_eq!(plan.sprite_render_pass, None);
        assert_eq!(plan.sprite_pipeline, None);
        assert_eq!(plan.sprite_resource_bindings, None);
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
        assert_eq!(
            plan.gpu_pass.scene_projection,
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240))
        );
    }

    #[test]
    fn native_scene_renderer_omits_sprite_pipeline_layout_for_empty_target() {
        let mut scene = RenderScene::empty(87, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        let plan =
            NativeSceneRenderer::default().prepare_for_target(&scene, SurfaceSize::new(0, 0));

        assert!(plan.viewport.is_empty());
        assert_eq!(plan.gpu_pass.viewport, None);
        assert_eq!(plan.sprite_render_pass, expected_sprite_render_pass(&plan));
        assert_eq!(
            plan.sprite_pipeline,
            expected_sprite_pipeline(&plan, GpuRendererSettings::default())
        );
        assert_eq!(
            plan.sprite_resource_bindings,
            expected_sprite_resource_bindings(&plan)
        );
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
    }

    #[test]
    fn native_scene_renderer_keeps_raster_upload_separate_from_sprites() {
        let mut scene = RenderScene::from_rgba(
            55,
            SurfaceSize::new(1, 1),
            vec![1, 2, 3, 255],
            Some(0xFEED_FACE),
        )
        .expect("raster scene");
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::STATUS_TEXT,
            layer: RenderLayer::Overlay,
            position: [0.0, 0.0],
            size: [8.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = NativeSceneRenderer::default().prepare(&scene);

        assert_eq!(
            plan.pipelines,
            vec![
                NativeRenderPipeline::TemporaryRaster,
                NativeRenderPipeline::DebugOverlay
            ]
        );
        assert_eq!(
            plan.raster_upload,
            Some(super::SceneRasterUpload {
                surface: SurfaceSize::new(1, 1),
                byte_len: 4,
                visual_signature: Some(0xFEED_FACE),
                non_blank: true,
            })
        );
        assert_eq!(plan.sprite_instances, 1);
        assert_eq!(plan.missing_sprite_regions, 0);
        assert_eq!(plan.sprite_batches.len(), 1);
        assert_eq!(
            plan.sprite_batches[0].pipeline,
            NativeRenderPipeline::DebugOverlay
        );
        assert_eq!(
            plan.sprite_batches[0].instances[0].sprite,
            SpriteId::STATUS_TEXT
        );
        assert_eq!(
            plan.sprite_instance_upload,
            Some(SpriteInstanceUpload {
                records: plan.sprite_instance_buffers[0].records.clone(),
            })
        );
        assert_eq!(
            plan.sprite_buffer_uploads,
            plan.sprite_instance_upload
                .as_ref()
                .map(SpriteBufferUploadPlan::from_instance_upload)
        );
        assert_eq!(
            plan.sprite_draw_commands,
            vec![expected_sprite_draw_command(
                NativeRenderPipeline::DebugOverlay,
                RenderLayer::Overlay,
                0,
                1,
            )]
        );
        assert_eq!(plan.sprite_render_pass, expected_sprite_render_pass(&plan));
        assert_eq!(
            plan.sprite_pipeline,
            expected_sprite_pipeline(&plan, GpuRendererSettings::default())
        );
        assert_eq!(
            plan.sprite_resource_bindings,
            expected_sprite_resource_bindings(&plan)
        );
        assert_eq!(
            plan.sprite_pipeline_layout,
            expected_sprite_pipeline_layout(&plan)
        );
        assert_eq!(
            plan.sprite_render_pipeline_descriptor,
            expected_sprite_render_pipeline_descriptor(&plan)
        );
        assert_eq!(
            plan.sprite_render_pass_encoder,
            expected_sprite_render_pass_encoder(&plan)
        );
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 1);
        assert_eq!(plan.frame_plan.temporary_raster_count(), 1);
    }
}
