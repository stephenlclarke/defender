//! Wgpu-oriented scene contracts.

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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneSprite {
    pub sprite: SpriteId,
    pub layer: RenderLayer,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub tint: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderScene {
    pub frame: u64,
    pub surface: SurfaceSize,
    pub clear_color: Color,
    pub visual_hash: Option<u32>,
    pub sprites: Vec<SceneSprite>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderSceneSummary {
    pub frame: u64,
    pub surface: SurfaceSize,
    pub visual_hash: Option<u32>,
    pub sprite_count: usize,
    pub layers: RenderLayerCounts,
}

impl RenderScene {
    pub fn empty(frame: u64, surface: SurfaceSize) -> Self {
        Self {
            frame,
            surface,
            clear_color: Color { rgba: [0; 4] },
            visual_hash: None,
            sprites: Vec::new(),
        }
    }

    pub fn push_sprite(&mut self, sprite: SceneSprite) {
        self.sprites.push(sprite);
    }

    pub fn summary(&self) -> RenderSceneSummary {
        let mut layers = RenderLayerCounts::default();
        for sprite in &self.sprites {
            layers.add(sprite.layer);
        }

        RenderSceneSummary {
            frame: self.frame,
            surface: self.surface,
            visual_hash: self.visual_hash,
            sprite_count: self.sprites.len(),
            layers,
        }
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
        Color, GpuRendererSettings, RenderLayer, RenderLayerCounts, RenderScene, SceneSprite,
        SpriteId, SurfaceSize,
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
    fn render_scene_summary_counts_layers_and_visual_hash() {
        let mut scene = RenderScene::empty(12, SurfaceSize::new(292, 240));
        scene.visual_hash = Some(0xCAFE_BABE);
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

        assert_eq!(summary.visual_hash, Some(0xCAFE_BABE));
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
}
