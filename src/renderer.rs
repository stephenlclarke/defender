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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub rgba: [u8; 4],
}

impl Color {
    pub const WHITE: Self = Self {
        rgba: [0xFF, 0xFF, 0xFF, 0xFF],
    };
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
}

impl TextureAtlas {
    pub fn new(surface: SurfaceSize, regions: Vec<AtlasRegion>) -> Self {
        Self { surface, regions }
    }

    pub fn default_sprites() -> Self {
        Self {
            surface: SurfaceSize::new(128, 128),
            regions: vec![
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
            ],
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

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteDrawBatch {
    pub pipeline: NativeRenderPipeline,
    pub layer: RenderLayer,
    pub instances: Vec<SpriteDrawInstance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneDrawPlan {
    pub frame: u64,
    pub surface: SurfaceSize,
    pub pipelines: Vec<NativeRenderPipeline>,
    pub sprite_instances: usize,
    pub missing_sprite_regions: usize,
    pub sprite_batches: Vec<SpriteDrawBatch>,
    pub layer_counts: RenderLayerCounts,
    pub raster_upload: Option<SceneRasterUpload>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeSceneRenderer {
    pub resources: NativeRendererResources,
}

impl NativeSceneRenderer {
    pub fn new(resources: NativeRendererResources) -> Self {
        Self { resources }
    }

    pub fn prepare(&self, scene: &RenderScene) -> SceneDrawPlan {
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

        SceneDrawPlan {
            frame: scene.frame,
            surface: scene.surface,
            pipelines,
            sprite_instances,
            missing_sprite_regions,
            sprite_batches,
            layer_counts,
            raster_upload: scene.raster.as_ref().map(|raster| SceneRasterUpload {
                surface: raster.surface,
                byte_len: raster.pixels.len(),
                visual_signature: scene.visual_signature,
                non_blank: raster.is_non_blank(),
            }),
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
        NativeSceneRenderer, RenderLayer, RenderLayerCounts, RenderScene, SceneRaster,
        SceneRasterError, SceneSprite, SpriteDrawBatch, SpriteDrawInstance, SpriteId, SurfaceSize,
        TextureAtlas,
    };

    #[test]
    fn surface_size_reports_empty_edges() {
        assert!(SurfaceSize::new(0, 240).is_empty());
        assert!(SurfaceSize::new(320, 0).is_empty());
        assert!(!SurfaceSize::new(320, 240).is_empty());
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
        assert_eq!(
            atlas.region(SpriteId(42)),
            Some(AtlasRegion {
                sprite: SpriteId(42),
                origin: [1, 2],
                size: [3, 4],
            })
        );
        assert_eq!(atlas.region(SpriteId::PLAYER_SHIP), None);
        assert!(TextureAtlas::default_sprites().contains(SpriteId::STATUS_TEXT));
        assert!(TextureAtlas::default_sprites().contains(SpriteId::PLAYER_PROJECTILE));
        assert!(TextureAtlas::default_sprites().contains(SpriteId::TERRAIN_TILE));
        assert!(TextureAtlas::default_sprites().contains(SpriteId::STAR));
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
    }
}
