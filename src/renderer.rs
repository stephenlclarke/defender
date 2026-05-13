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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteId(pub u16);

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
    pub sprites: Vec<SceneSprite>,
}

impl RenderScene {
    pub fn empty(frame: u64, surface: SurfaceSize) -> Self {
        Self {
            frame,
            surface,
            clear_color: Color { rgba: [0; 4] },
            sprites: Vec::new(),
        }
    }

    pub fn push_sprite(&mut self, sprite: SceneSprite) {
        self.sprites.push(sprite);
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
        Color, GpuRendererSettings, RenderLayer, RenderScene, SceneSprite, SpriteId, SurfaceSize,
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
    fn renderer_settings_are_wgpu_native() {
        let settings = GpuRendererSettings::default();

        assert_eq!(settings.texture_format, wgpu::TextureFormat::Rgba8UnormSrgb);
        assert_eq!(settings.present_mode, wgpu::PresentMode::AutoVsync);
    }
}
