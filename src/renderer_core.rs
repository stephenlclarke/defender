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
pub(crate) struct AttractDefenderAppearancePixel {
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
    pub const PLAYER_SHIP_LEFT: Self = Self(91);
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
            "PLAPIC" => Some(Self::PLAYER_SHIP),
            "PLBPIC" => Some(Self::PLAYER_SHIP_LEFT),
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

pub fn render_scene_to_rgba(
    scene: &RenderScene,
    target: SurfaceSize,
) -> Result<SceneRaster, SceneRasterError> {
    let atlas = TextureAtlas::default_sprites();
    render_scene_with_atlas_to_rgba(scene, target, &atlas)
}

pub fn render_scene_with_atlas_to_rgba(
    scene: &RenderScene,
    target: SurfaceSize,
    atlas: &TextureAtlas,
) -> Result<SceneRaster, SceneRasterError> {
    let Some(pixel_buffer_length) = target.rgba_len() else {
        return Err(SceneRasterError::PixelBufferTooLarge { surface: target });
    };

    let clear_color = opaque_scene_clear_color(scene);
    let mut pixels = Vec::with_capacity(pixel_buffer_length);
    for _ in 0..pixel_buffer_length / 4 {
        pixels.extend_from_slice(&clear_color);
    }

    let layout = ViewportLayout::fit(scene.surface, target);
    if !layout.is_empty() {
        if let Some(raster) = scene.raster() {
            blit_scene_raster_nearest(&mut pixels, target, layout, raster);
        }

        let mut indexed_sprites: Vec<_> = scene.sprites.iter().enumerate().collect();
        indexed_sprites.sort_by_key(|(index, sprite)| (render_layer_order(sprite.layer), *index));

        for (_, sprite) in indexed_sprites {
            blit_scene_sprite_nearest(&mut pixels, target, layout, atlas, sprite);
        }
    }

    SceneRaster::from_rgba(target, pixels)
}

fn opaque_scene_clear_color(scene: &RenderScene) -> [u8; 4] {
    match scene.clear_color.rgba {
        [_, _, _, 0] => [0, 0, 0, 255],
        color => color,
    }
}

fn render_layer_order(layer: RenderLayer) -> u8 {
    match layer {
        RenderLayer::Terrain => 0,
        RenderLayer::Starfield => 1,
        RenderLayer::Objects => 2,
        RenderLayer::Projectiles => 3,
        RenderLayer::Hud => 4,
        RenderLayer::Overlay => 5,
    }
}

fn blit_scene_raster_nearest(
    pixels: &mut [u8],
    target: SurfaceSize,
    layout: ViewportLayout,
    raster: &SceneRaster,
) {
    if raster.surface.is_empty() {
        return;
    }

    for target_y in layout.origin[1]..layout.origin[1] + layout.size.height {
        let viewport_y = target_y - layout.origin[1];
        let source_y =
            nearest_source_coordinate(viewport_y, layout.size.height, raster.surface.height);
        for target_x in layout.origin[0]..layout.origin[0] + layout.size.width {
            let viewport_x = target_x - layout.origin[0];
            let source_x =
                nearest_source_coordinate(viewport_x, layout.size.width, raster.surface.width);
            let source_index =
                rgba_index(raster.surface, source_x, source_y).expect("raster index in bounds");
            let target_index =
                rgba_index(target, target_x, target_y).expect("target index in bounds");
            blend_rgba(
                &mut pixels[target_index..target_index + 4],
                &raster.pixels()[source_index..source_index + 4],
            );
        }
    }
}

fn blit_scene_sprite_nearest(
    pixels: &mut [u8],
    target: SurfaceSize,
    layout: ViewportLayout,
    atlas: &TextureAtlas,
    sprite: &SceneSprite,
) {
    if sprite.size[0] <= 0.0 || sprite.size[1] <= 0.0 {
        return;
    }

    let Some(region) = atlas.region(sprite.sprite) else {
        return;
    };

    let target_x = layout.origin[0] as i32 + (sprite.position[0] * layout.scale).round() as i32;
    let target_y = layout.origin[1] as i32 + (sprite.position[1] * layout.scale).round() as i32;
    let target_width = scaled_sprite_extent(sprite.size[0], layout.scale);
    let target_height = scaled_sprite_extent(sprite.size[1], layout.scale);
    let clip_left = target_x.max(0) as u32;
    let clip_top = target_y.max(0) as u32;
    let clip_right = (target_x + target_width as i32).clamp(0, target.width as i32) as u32;
    let clip_bottom = (target_y + target_height as i32).clamp(0, target.height as i32) as u32;

    if clip_left >= clip_right || clip_top >= clip_bottom {
        return;
    }

    for y in clip_top..clip_bottom {
        let local_y = (y as i32 - target_y) as u32;
        let source_y = nearest_source_coordinate(local_y, target_height, region.size[1]);
        for x in clip_left..clip_right {
            let local_x = (x as i32 - target_x) as u32;
            let source_x = nearest_source_coordinate(local_x, target_width, region.size[0]);
            let atlas_x = region.origin[0] + source_x;
            let atlas_y = region.origin[1] + source_y;
            let Some(source_index) = rgba_index(atlas.surface, atlas_x, atlas_y) else {
                continue;
            };
            let tinted = tint_rgba(&atlas.pixels()[source_index..source_index + 4], sprite.tint);
            if tinted[3] == 0 {
                continue;
            }
            let target_index = rgba_index(target, x, y).expect("target index in bounds");
            blend_rgba(&mut pixels[target_index..target_index + 4], &tinted);
        }
    }
}

fn scaled_sprite_extent(extent: f32, scale: f32) -> u32 {
    (extent * scale).round().max(1.0) as u32
}

fn nearest_source_coordinate(
    target_coordinate: u32,
    target_extent: u32,
    source_extent: u32,
) -> u32 {
    if target_extent == 0 || source_extent == 0 {
        return 0;
    }

    ((u64::from(target_coordinate) * u64::from(source_extent)) / u64::from(target_extent))
        .min(u64::from(source_extent.saturating_sub(1))) as u32
}

fn rgba_index(surface: SurfaceSize, x: u32, y: u32) -> Option<usize> {
    if x >= surface.width || y >= surface.height {
        return None;
    }

    let row_offset = (y as usize).checked_mul(surface.width as usize)?;
    let pixel_offset = row_offset.checked_add(x as usize)?;
    pixel_offset.checked_mul(4)
}

fn tint_rgba(source: &[u8], tint: Color) -> [u8; 4] {
    [
        multiply_u8(source[0], tint.rgba[0]),
        multiply_u8(source[1], tint.rgba[1]),
        multiply_u8(source[2], tint.rgba[2]),
        multiply_u8(source[3], tint.rgba[3]),
    ]
}

fn multiply_u8(left: u8, right: u8) -> u8 {
    ((u16::from(left) * u16::from(right) + 127) / 255) as u8
}

fn blend_rgba(destination: &mut [u8], source: &[u8]) {
    let alpha = u16::from(source[3]);
    let inverse_alpha = 255 - alpha;
    for channel in 0..3 {
        destination[channel] = ((u16::from(source[channel]) * alpha
            + u16::from(destination[channel]) * inverse_alpha
            + 127)
            / 255) as u8;
    }
    destination[3] = 255;
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

pub fn screen_position_from_address(screen_address: u16) -> [f32; 2] {
    let [column, row] = screen_address.to_be_bytes();
    [
        f32::from(column) * SCREEN_COLUMN_WIDTH_PIXELS,
        f32::from(row),
    ]
}

pub fn screen_position_from_address_with_offset(
    top_left_screen_address: u16,
    horizontal: u8,
    vertical: u8,
) -> [f32; 2] {
    let [column, row] = top_left_screen_address.to_be_bytes();
    screen_position_from_address(u16::from_be_bytes([
        column.wrapping_add(horizontal),
        row.wrapping_add(vertical),
    ]))
}

pub fn push_message_sprites(
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
        cursor_x += size[0] as f32 + SCREEN_COLUMN_WIDTH_PIXELS;
    }
}

pub fn push_message_text_bytes_sprites(
    scene: &mut RenderScene,
    bytes: &[u8],
    origin: [f32; 2],
    layer: RenderLayer,
) {
    let mut cursor_x = origin[0];
    for byte in bytes {
        let Some((sprite, size)) = message_text_byte_sprite(*byte) else {
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
        cursor_x += size[0] as f32 + SCREEN_COLUMN_WIDTH_PIXELS;
    }
}

pub fn push_arcade_controlled_message_sprites(
    scene: &mut RenderScene,
    text: &str,
    top_left_screen_address: u16,
    layer: RenderLayer,
) {
    let mut layout = ArcadeMessageTextLayout {
        top_left: top_left_screen_address,
        cursor: top_left_screen_address,
        line_spacing: MESSAGE_LINE_SPACING_ROWS,
    };

    for word in text.split_whitespace() {
        if let Some(control) = arcade_message_control(word) {
            layout.apply(control);
            continue;
        }
        if arcade_message_control_body(word).is_some() {
            continue;
        }

        let bytes = word.as_bytes();
        push_message_text_bytes_sprites(scene, bytes, screen_position_from_address(layout.cursor), layer);
        layout.cursor = message_text_cursor_after_bytes(layout.cursor, bytes);
        layout.cursor = message_text_cursor_after_bytes(layout.cursor, b" ");
    }
}

fn message_text_byte_sprite(byte: u8) -> Option<(Option<SpriteId>, [u32; 2])> {
    if byte == b' ' {
        return Some((None, SpriteId::message_glyph_size(' ')?));
    }

    if byte.is_ascii_digit() {
        return Some((SpriteId::score_digit(byte - b'0'), SCORE_DIGIT_PIXEL_SIZE));
    }

    let character = char::from(byte);
    let size = SpriteId::message_glyph_size(character)?;
    Some((SpriteId::message_glyph(character), size))
}

fn message_text_cursor_after_bytes(mut cursor: u16, bytes: &[u8]) -> u16 {
    for byte in bytes {
        let Some((_sprite, size)) = message_text_byte_sprite(*byte) else {
            continue;
        };
        cursor = message_text_cursor_advance(cursor, size[0]);
    }
    cursor
}

fn message_text_cursor_advance(cursor: u16, width_pixels: u32) -> u16 {
    let [column, row] = cursor.to_be_bytes();
    let width_columns = u8::try_from(width_pixels / u32::from(SCREEN_COLUMN_WIDTH_PIXELS_U8))
        .expect("message glyph width fits in u8");
    u16::from_be_bytes([column.wrapping_add(width_columns).wrapping_add(1), row])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArcadeMessageTextLayout {
    top_left: u16,
    cursor: u16,
    line_spacing: u8,
}

impl ArcadeMessageTextLayout {
    fn apply(&mut self, control: ArcadeMessageControl) {
        match control {
            ArcadeMessageControl::HorizontalFromTopLeft(delta) => {
                let [top_x, cursor_y] =
                    [self.top_left.to_be_bytes()[0], self.cursor.to_be_bytes()[1]];
                self.cursor = u16::from_be_bytes([top_x.wrapping_add(delta), cursor_y]);
            }
            ArcadeMessageControl::HorizontalFromCursor(delta) => {
                let [cursor_x, cursor_y] = self.cursor.to_be_bytes();
                self.cursor = u16::from_be_bytes([cursor_x.wrapping_add(delta), cursor_y]);
            }
            ArcadeMessageControl::VerticalFromTopLeft(delta) => {
                let [cursor_x, _cursor_y] = self.cursor.to_be_bytes();
                let top_y = self.top_left.to_be_bytes()[1];
                self.cursor = u16::from_be_bytes([cursor_x, top_y.wrapping_add(delta)]);
            }
            ArcadeMessageControl::VerticalFromCursor(delta) => {
                let [cursor_x, cursor_y] = self.cursor.to_be_bytes();
                self.cursor = u16::from_be_bytes([cursor_x, cursor_y.wrapping_add(delta)]);
            }
            ArcadeMessageControl::ResetTopLeftAndCursor(address) => {
                self.top_left = address;
                self.cursor = address;
            }
            ArcadeMessageControl::ReturnLineFeed => {
                let [top_x, _top_y] = self.top_left.to_be_bytes();
                let cursor_y = self.cursor.to_be_bytes()[1];
                self.cursor = u16::from_be_bytes([top_x, cursor_y.wrapping_add(self.line_spacing)]);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArcadeMessageControl {
    HorizontalFromTopLeft(u8),
    HorizontalFromCursor(u8),
    VerticalFromTopLeft(u8),
    VerticalFromCursor(u8),
    ResetTopLeftAndCursor(u16),
    ReturnLineFeed,
}

fn arcade_message_control(word: &str) -> Option<ArcadeMessageControl> {
    let body = arcade_message_control_body(word)?;
    let (name, arguments) = body.split_once(':').unwrap_or((body, ""));
    match name {
        "HMT" => Some(ArcadeMessageControl::HorizontalFromTopLeft(
            arcade_message_control_byte(arguments)?,
        )),
        "HMC" => Some(ArcadeMessageControl::HorizontalFromCursor(
            arcade_message_control_byte(arguments)?,
        )),
        "VMT" => Some(ArcadeMessageControl::VerticalFromTopLeft(
            arcade_message_control_byte(arguments)?,
        )),
        "VMC" => Some(ArcadeMessageControl::VerticalFromCursor(
            arcade_message_control_byte(arguments)?,
        )),
        "RTC" => {
            let (x, y) = arguments.split_once(',')?;
            Some(ArcadeMessageControl::ResetTopLeftAndCursor(
                u16::from_be_bytes([
                    arcade_message_control_byte(x)?,
                    arcade_message_control_byte(y)?,
                ]),
            ))
        }
        "RLF" if arguments.is_empty() => Some(ArcadeMessageControl::ReturnLineFeed),
        _ => None,
    }
}

fn arcade_message_control_body(word: &str) -> Option<&str> {
    word.strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
}

fn arcade_message_control_byte(value: &str) -> Option<u8> {
    let hex = value.strip_prefix("0x")?;
    u8::from_str_radix(hex, 16).ok()
}
