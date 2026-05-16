//! Wgpu-oriented scene contracts.

use std::collections::BTreeSet;

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
        let surface = SurfaceSize::new(128, 128);
        let regions = vec![
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
        ];
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

fn default_sprite_atlas_pixels(surface: SurfaceSize, regions: &[AtlasRegion]) -> Vec<u8> {
    let mut pixels = transparent_rgba_pixels(surface).unwrap_or_default();
    for region in regions {
        fill_atlas_region(
            &mut pixels,
            surface,
            *region,
            default_sprite_region_color(region.sprite),
        );
    }
    pixels
}

fn default_sprite_region_color(sprite: SpriteId) -> [u8; 4] {
    match sprite {
        SpriteId::PLAYER_SHIP => [0xFF, 0xFF, 0xFF, 0xFF],
        SpriteId::SCORE_TEXT => [0xFF, 0xFF, 0xFF, 0xFF],
        SpriteId::STATUS_TEXT => [0x26, 0xAE, 0x00, 0xFF],
        SpriteId::PLAYER_PROJECTILE => [0xFF, 0xF8, 0x80, 0xFF],
        SpriteId::TERRAIN_TILE => [0x26, 0xAE, 0x00, 0xFF],
        SpriteId::STAR => [0xFF, 0xFF, 0xFF, 0xFF],
        SpriteId::ENEMY_LANDER => [0xF4, 0x5B, 0x5B, 0xFF],
        SpriteId::HUMAN => [0x7C, 0xD7, 0xFF, 0xFF],
        _ => [0xD9, 0x51, 0xFF, 0xFF],
    }
}

fn fill_atlas_region(pixels: &mut [u8], surface: SurfaceSize, region: AtlasRegion, color: [u8; 4]) {
    let max_x = region.origin[0]
        .saturating_add(region.size[0])
        .min(surface.width);
    let max_y = region.origin[1]
        .saturating_add(region.size[1])
        .min(surface.height);
    for y in region.origin[1]..max_y {
        for x in region.origin[0]..max_x {
            let start = ((y as usize * surface.width as usize) + x as usize) * 4;
            let end = start + 4;
            if end <= pixels.len() {
                pixels[start..end].copy_from_slice(&color);
            }
        }
    }
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
        AtlasRegion, Color, GpuRendererSettings, NativeRenderPipeline, NativeRendererResources,
        NativeSceneRenderer, RenderLayer, RenderLayerCounts, RenderScene, SceneDrawPlan,
        SceneProjectionUniformUpload, SceneProjectionUniforms, SceneRaster, SceneRasterError,
        SceneRasterUpload, SceneSprite, SpriteAtlasTextureUpload, SpriteBindGroupLayoutPlan,
        SpriteBindGroupRole, SpriteBufferRole, SpriteBufferUpload, SpriteBufferUploadPlan,
        SpriteDrawBatch, SpriteDrawCommand, SpriteDrawInstance, SpriteId, SpriteIndexBufferBinding,
        SpriteInstanceBuffer, SpriteInstanceBufferRecord, SpriteInstanceUpload,
        SpritePipelineLayoutBindGroup, SpritePipelineLayoutPlan, SpritePipelinePlan,
        SpriteQuadGeometry, SpriteQuadVertex, SpriteRenderPassDraw, SpriteRenderPassEncoderCommand,
        SpriteRenderPassEncoderPlan, SpriteRenderPassPlan, SpriteRenderPipelineDescriptorPlan,
        SpriteResourceBindingPlan, SpriteResourceBindingRole, SpriteSamplerBindingPlan,
        SpriteShaderPlan, SpriteTextureBindingPlan, SpriteVertexBufferBinding,
        SpriteVertexBufferLayoutPlan, SurfaceSize, TextureAtlas, ViewportLayout, WgpuFrameCommand,
        WgpuFramePlan, WgpuPassPlan, WgpuViewportCommand,
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
                surface: SurfaceSize::new(128, 128),
                mip_level_count: 1,
                sample_count: 1,
                depth_or_array_layers: 1,
                bytes_per_row: 128 * 4,
                rows_per_image: 128,
                byte_len: 128 * 128 * 4,
                bytes: atlas.pixels().to_vec(),
                non_blank: true,
            }
        );
        assert_eq!(
            upload.extent(),
            wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            }
        );
        let copy_layout = upload.copy_layout();
        assert_eq!(copy_layout.offset, 0);
        assert_eq!(copy_layout.bytes_per_row, Some(128 * 4));
        assert_eq!(copy_layout.rows_per_image, Some(128));
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
        assert_eq!(descriptor.layout_bind_group_count, 2);
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
        assert_eq!(default_atlas.pixels().len(), 128 * 128 * 4);
        assert!(default_atlas.is_non_blank());
        assert!(default_atlas.contains(SpriteId::STATUS_TEXT));
        assert!(default_atlas.contains(SpriteId::PLAYER_PROJECTILE));
        assert!(default_atlas.contains(SpriteId::TERRAIN_TILE));
        assert!(default_atlas.contains(SpriteId::STAR));
        assert!(default_atlas.contains(SpriteId::ENEMY_LANDER));
        assert!(default_atlas.contains(SpriteId::HUMAN));
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
        assert_eq!(
            super::default_sprite_region_color(SpriteId(99)),
            [0xD9, 0x51, 0xFF, 0xFF]
        );
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
                        atlas_uv_size: [0.125, 0.0625],
                        tint: [1.0, 1.0, 1.0, 1.0],
                    }],
                },
                SpriteInstanceBuffer {
                    pipeline: NativeRenderPipeline::HudText,
                    layer: RenderLayer::Hud,
                    records: vec![SpriteInstanceBufferRecord {
                        scene_origin: [0.0, 0.0],
                        scene_size: [80.0, 8.0],
                        atlas_uv_origin: [0.0, 0.125],
                        atlas_uv_size: [0.625, 0.0625],
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
