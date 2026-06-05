//! High-level README media generation facade.
//!
//! This feature-gated tool facade lives with the archived tooling, but it
//! captures the clean game and renderer-owned sprite atlas instead of the
//! accepted machine.

use crate::{
    game::{AttractPresentationPage, Game, GameFrame, GameInput, ReferenceCaptureSteer},
    renderer::{
        AtlasRegion, Color, NativeRendererResources, RenderLayer, RenderScene, SceneRaster,
        SceneSprite, SpriteId, SurfaceSize, TextureAtlas, ViewportLayout,
    },
    systems::FrameRate,
};

pub const FRAME_RATE_MILLIHZ: u32 = FrameRate::CABINET.millihz();

// MAME native snapshots are 4:3 output; clean sprites still use red-label
// source screen coordinates, whose visible window starts on display row 7.
const DEFENDER_NATIVE_MEDIA_SURFACE: SurfaceSize = SurfaceSize::new(320, 240);
const DEFENDER_MEDIA_VISIBLE_Y_ORIGIN: f32 = 7.0;

pub struct ReadmeMediaFrameSource {
    game: Game,
    current_frame: GameFrame,
    renderer: ReadmeMediaRenderer,
}

impl ReadmeMediaFrameSource {
    pub fn new(output_width: u32, output_height: u32) -> Self {
        Self::new_with_first_input(output_width, output_height, GameInput::NONE)
    }

    pub fn new_with_first_input(output_width: u32, output_height: u32, input: GameInput) -> Self {
        let mut game = Game::new();
        let current_frame = game.step(input);

        Self {
            game,
            current_frame,
            renderer: ReadmeMediaRenderer::new(SurfaceSize::new(output_width, output_height)),
        }
    }

    pub fn step(&mut self) {
        self.step_with_input(GameInput::NONE);
    }

    pub fn step_with_input(&mut self, input: GameInput) {
        self.current_frame = self.game.step(input);
    }

    pub fn frame(&self) -> u64 {
        self.current_frame.state.frame
    }

    pub fn attract_page(&self) -> AttractPresentationPage {
        self.current_frame.state.attract.page
    }

    pub fn sound_events(&self) -> &[crate::SoundEvent] {
        self.current_frame.events.sounds()
    }

    pub fn seed_reference_capture_window(&mut self, steer: ReferenceCaptureSteer) {
        self.current_frame = self.game.seed_reference_capture_window(steer);
    }

    pub fn render_frame(&mut self) -> Result<ReadmeMediaFrame, ReadmeMediaError> {
        self.renderer.render_scene(&self.current_frame.scene)
    }
}

struct ReadmeMediaRenderer {
    target: SurfaceSize,
    atlas: TextureAtlas,
}

impl ReadmeMediaRenderer {
    fn new(target: SurfaceSize) -> Self {
        Self {
            target,
            atlas: NativeRendererResources::default().atlas,
        }
    }

    fn render_scene(&self, scene: &RenderScene) -> Result<ReadmeMediaFrame, ReadmeMediaError> {
        let Some(len) = self.target.rgba_len() else {
            return Err(ReadmeMediaError::TargetTooLarge(self.target));
        };
        let mut pixels = vec![0; len];
        fill_target(&mut pixels, scene.clear_color);

        let projection = ReadmeMediaProjection::defender_native(self.target);
        if scene.surface.is_empty() || projection.viewport.is_empty() {
            return Err(ReadmeMediaError::EmptyViewport {
                scene: scene.surface,
                target: self.target,
            });
        }

        if let Some(raster) = scene.raster() {
            blit_raster(&mut pixels, self.target, raster, projection);
        }

        for layer in [
            RenderLayer::Terrain,
            RenderLayer::Starfield,
            RenderLayer::Objects,
            RenderLayer::Projectiles,
            RenderLayer::Hud,
            RenderLayer::Overlay,
        ] {
            for sprite in scene
                .sprites
                .iter()
                .copied()
                .filter(|sprite| sprite.layer == layer)
            {
                let region = self
                    .atlas
                    .region(sprite.sprite)
                    .ok_or(ReadmeMediaError::MissingSprite(sprite.sprite))?;
                blit_sprite(
                    &mut pixels,
                    self.target,
                    &self.atlas,
                    region,
                    sprite,
                    projection,
                );
            }
        }

        Ok(ReadmeMediaFrame {
            width: self.target.width,
            height: self.target.height,
            pixels,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ReadmeMediaProjection {
    viewport: ViewportLayout,
    scene_origin: [f32; 2],
}

impl ReadmeMediaProjection {
    fn defender_native(target: SurfaceSize) -> Self {
        Self {
            viewport: ViewportLayout::fit(DEFENDER_NATIVE_MEDIA_SURFACE, target),
            scene_origin: [0.0, DEFENDER_MEDIA_VISIBLE_Y_ORIGIN],
        }
    }

    fn target_to_scene(&self, target_axis: u32, axis: usize) -> f32 {
        (target_axis as f32 + 0.5 - self.viewport.origin[axis] as f32) / self.viewport.scale
            + self.scene_origin[axis]
    }

    fn scene_to_target(&self, scene_axis: f32, axis: usize) -> f32 {
        self.viewport.origin[axis] as f32
            + (scene_axis - self.scene_origin[axis]) * self.viewport.scale
    }
}

fn fill_target(pixels: &mut [u8], clear_color: Color) {
    let mut color = clear_color.rgba;
    if color[3] == 0 {
        color = [0, 0, 0, 0xFF];
    }
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.copy_from_slice(&color);
    }
}

fn blit_raster(
    target_pixels: &mut [u8],
    target: SurfaceSize,
    raster: &SceneRaster,
    projection: ReadmeMediaProjection,
) {
    let viewport = projection.viewport;
    for target_y in viewport.origin[1]..viewport.origin[1] + viewport.size.height {
        for target_x in viewport.origin[0]..viewport.origin[0] + viewport.size.width {
            let Some(scene_x) =
                scaled_target_to_scene(&projection, target_x, 0, raster.surface.width)
            else {
                continue;
            };
            let Some(scene_y) =
                scaled_target_to_scene(&projection, target_y, 1, raster.surface.height)
            else {
                continue;
            };
            let source_index =
                pixel_offset(raster.surface, scene_x, scene_y).min(raster.pixels().len());
            if source_index + 4 <= raster.pixels().len() {
                let source = &raster.pixels()[source_index..source_index + 4];
                let target_index = pixel_offset(target, target_x, target_y);
                alpha_blend(target_pixels, target_index, source);
            }
        }
    }
}

fn blit_sprite(
    target_pixels: &mut [u8],
    target: SurfaceSize,
    atlas: &TextureAtlas,
    region: AtlasRegion,
    sprite: SceneSprite,
    projection: ReadmeMediaProjection,
) {
    if sprite.size[0] <= 0.0 || sprite.size[1] <= 0.0 {
        return;
    }

    let [min_x, min_y, max_x, max_y] = sprite_target_bounds(sprite, projection, target);
    if min_x >= max_x || min_y >= max_y {
        return;
    }

    for target_y in min_y..max_y {
        for target_x in min_x..max_x {
            let scene_x = projection.target_to_scene(target_x, 0);
            let scene_y = projection.target_to_scene(target_y, 1);
            let local_x = ((scene_x - sprite.position[0]) / sprite.size[0]).clamp(0.0, 0.999_999);
            let local_y = ((scene_y - sprite.position[1]) / sprite.size[1]).clamp(0.0, 0.999_999);
            let atlas_x = region.origin[0] + (local_x * region.size[0] as f32) as u32;
            let atlas_y = region.origin[1] + (local_y * region.size[1] as f32) as u32;
            let atlas_index = pixel_offset(atlas.surface, atlas_x, atlas_y);
            if atlas_index + 4 > atlas.pixels().len() {
                continue;
            }

            let tinted = tint_rgba(&atlas.pixels()[atlas_index..atlas_index + 4], sprite.tint);
            if tinted[3] == 0 {
                continue;
            }
            let target_index = pixel_offset(target, target_x, target_y);
            alpha_blend(target_pixels, target_index, &tinted);
        }
    }
}

fn sprite_target_bounds(
    sprite: SceneSprite,
    projection: ReadmeMediaProjection,
    target: SurfaceSize,
) -> [u32; 4] {
    let x0 = projection.scene_to_target(sprite.position[0], 0);
    let y0 = projection.scene_to_target(sprite.position[1], 1);
    let x1 = projection.scene_to_target(sprite.position[0] + sprite.size[0], 0);
    let y1 = projection.scene_to_target(sprite.position[1] + sprite.size[1], 1);

    [
        x0.floor().max(0.0).min(target.width as f32) as u32,
        y0.floor().max(0.0).min(target.height as f32) as u32,
        x1.ceil().max(0.0).min(target.width as f32) as u32,
        y1.ceil().max(0.0).min(target.height as f32) as u32,
    ]
}

fn scaled_target_to_scene(
    projection: &ReadmeMediaProjection,
    target_axis: u32,
    axis: usize,
    scene_extent: u32,
) -> Option<u32> {
    let scene_axis = projection.target_to_scene(target_axis, axis).floor();
    if !(0.0..scene_extent as f32).contains(&scene_axis) {
        return None;
    }
    Some(scene_axis as u32)
}

fn tint_rgba(source: &[u8], tint: Color) -> [u8; 4] {
    [
        multiply_channel(source[0], tint.rgba[0]),
        multiply_channel(source[1], tint.rgba[1]),
        multiply_channel(source[2], tint.rgba[2]),
        multiply_channel(source[3], tint.rgba[3]),
    ]
}

fn multiply_channel(left: u8, right: u8) -> u8 {
    ((u16::from(left) * u16::from(right)) / 0xFF) as u8
}

fn alpha_blend(target_pixels: &mut [u8], target_index: usize, source: &[u8]) {
    if target_index + 4 > target_pixels.len() {
        return;
    }
    let alpha = u16::from(source[3]);
    let inverse_alpha = 0xFF - alpha;
    for channel in 0..3 {
        let source_channel = u16::from(source[channel]);
        let target_channel = u16::from(target_pixels[target_index + channel]);
        target_pixels[target_index + channel] =
            ((source_channel * alpha + target_channel * inverse_alpha) / 0xFF) as u8;
    }
    target_pixels[target_index + 3] = 0xFF;
}

fn pixel_offset(surface: SurfaceSize, x: u32, y: u32) -> usize {
    ((y as usize * surface.width as usize) + x as usize) * 4
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadmeMediaFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadmeMediaError {
    EmptyViewport {
        scene: SurfaceSize,
        target: SurfaceSize,
    },
    MissingSprite(SpriteId),
    TargetTooLarge(SurfaceSize),
}

impl std::fmt::Display for ReadmeMediaError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyViewport { scene, target } => write!(
                formatter,
                "README media viewport is empty for scene {}x{} into target {}x{}",
                scene.width, scene.height, target.width, target.height
            ),
            Self::MissingSprite(sprite) => write!(
                formatter,
                "README media clean sprite atlas is missing sprite {:?}",
                sprite
            ),
            Self::TargetTooLarge(surface) => write!(
                formatter,
                "README media target is too large: {}x{}",
                surface.width, surface.height
            ),
        }
    }
}

impl std::error::Error for ReadmeMediaError {}

#[cfg(test)]
mod tests {
    use super::{
        FRAME_RATE_MILLIHZ, ReadmeMediaFrame, ReadmeMediaFrameSource, ReadmeMediaProjection,
        sprite_target_bounds,
    };
    use crate::game::{ATTRACT_WILLIAMS_LOGO_REVEAL_FRAMES, AttractPresentationPage};
    use crate::renderer::{Color, RenderLayer, SceneSprite, SpriteId, SurfaceSize};

    #[test]
    fn frame_rate_matches_clean_cabinet_refresh_contract() {
        assert_eq!(FRAME_RATE_MILLIHZ, 60_100);
    }

    #[test]
    fn source_renders_clean_scaled_rgba_frames() {
        let mut source = ReadmeMediaFrameSource::new(320, 240);

        let frame = render_first_visible_frame(&mut source);

        assert_eq!(frame.width, 320);
        assert_eq!(frame.height, 240);
        assert_eq!(frame.pixels.len(), 320 * 240 * 4);
        assert!(frame_has_visible_pixels(&frame));
    }

    #[test]
    fn source_reference_media_uses_defender_native_visible_y_projection() {
        let target = SurfaceSize::new(768, 576);
        let projection = ReadmeMediaProjection::defender_native(target);
        let sprite = SceneSprite {
            sprite: SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL,
            layer: RenderLayer::Hud,
            position: [96.0, 28.0],
            size: [1.0, 1.0],
            tint: Color::WHITE,
        };

        assert_eq!(
            sprite_target_bounds(sprite, projection, target),
            [230, 50, 233, 53]
        );
    }

    #[test]
    fn source_follows_clean_attract_acceptance_order() {
        let mut source = ReadmeMediaFrameSource::new(320, 240);

        assert_eq!(source.attract_page(), AttractPresentationPage::WilliamsLogo);
        step_until_page(&mut source, AttractPresentationPage::HallOfFame);
        step_until_page(&mut source, AttractPresentationPage::ScoringSequence);
    }

    fn step_until_page(source: &mut ReadmeMediaFrameSource, page: AttractPresentationPage) {
        for _ in 0..10_000 {
            if source.attract_page() == page {
                return;
            }
            source.step();
        }
        panic!("clean README media source did not reach {page:?}");
    }

    fn render_first_visible_frame(source: &mut ReadmeMediaFrameSource) -> ReadmeMediaFrame {
        for _ in 0..=ATTRACT_WILLIAMS_LOGO_REVEAL_FRAMES {
            let frame = source.render_frame().expect("README media frame");
            if frame_has_visible_pixels(&frame) {
                return frame;
            }
            source.step();
        }
        panic!("README media source did not render a visible Williams logo frame");
    }

    fn frame_has_visible_pixels(frame: &ReadmeMediaFrame) -> bool {
        frame
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel != [0, 0, 0, 0xFF])
    }
}
