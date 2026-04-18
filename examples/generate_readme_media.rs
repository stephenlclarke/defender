use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use gif::{Encoder, Frame, Repeat};
use png::{BitDepth, ColorType, Compression, Encoder as PngEncoder};

use defender::{
    attract::{AttractBeat, Scene, attract_cycle},
    demo::gameplay_demo_cycle,
    game::{Entity, EntityKind},
    render,
};

const CELL_SIZE: u32 = 8;
const GIF_SCALE: u32 = 3;
const SCREENSHOT_SCALE: u32 = 6;
const PADDING: u32 = 16;
const BACKGROUND: [u8; 4] = [4, 8, 14, 255];
const FOREGROUND: [u8; 4] = [80, 255, 140, 255];
const OUTPUT_WIDTH: u32 = 3456;
const OUTPUT_HEIGHT: u32 = 1864;
const WINDOW_X: u32 = 112;
const WINDOW_Y: u32 = 76;
const WINDOW_WIDTH: u32 = 3232;
const WINDOW_HEIGHT: u32 = 1650;
const WINDOW_RADIUS: u32 = 42;
const CONTENT_X: u32 = 160;
const CONTENT_Y: u32 = 204;
const CONTENT_WIDTH: u32 = 3135;
const CONTENT_HEIGHT: u32 = 1444;
const WINDOW_COLOR: [u8; 4] = [30, 30, 45, 255];
const WINDOW_SHADOW: [u8; 4] = [0, 0, 0, 52];
const CONTENT_BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const TRAFFIC_RED: [u8; 4] = [255, 95, 87, 255];
const TRAFFIC_YELLOW: [u8; 4] = [254, 188, 46, 255];
const TRAFFIC_GREEN: [u8; 4] = [40, 200, 64, 255];
const GHOST_BADGE: [u8; 4] = [240, 240, 248, 255];
const GHOST_EYE: [u8; 4] = [18, 18, 26, 255];

fn main() -> Result<()> {
    let screenshot_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs/defender.png"));
    let gif_path = std::env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs/start-sequence.gif"));

    ensure_parent_dir(&screenshot_path)?;
    ensure_parent_dir(&gif_path)?;

    let screenshot_scene = gameplay_screenshot_scene();
    let sequence = build_sequence();
    let (screenshot_cols, screenshot_rows) = scene_bounds(std::slice::from_ref(&screenshot_scene));
    let (gif_cols, gif_rows) = scene_bounds_from_frames(&sequence);

    let screenshot = rasterize_scene(
        &screenshot_scene,
        screenshot_cols,
        screenshot_rows,
        SCREENSHOT_SCALE,
    );
    let screenshot = compose_windowed_screenshot(&screenshot);
    write_png(&screenshot_path, &screenshot)?;
    write_gif(&gif_path, &sequence, gif_cols, gif_rows)?;

    println!("wrote {}", screenshot_path.display());
    println!("wrote {}", gif_path.display());
    Ok(())
}

fn build_sequence() -> Vec<(Scene, u16)> {
    attract_cycle()
        .into_iter()
        .map(|beat| (beat.scene(), centiseconds_for_beat(beat)))
        .collect()
}

fn gameplay_screenshot_scene() -> Scene {
    let mut world = gameplay_demo_cycle()[4].world.clone();
    world.add_score(375);
    world.set_lives(2);
    world.set_smart_bombs(1);
    world.spawn_entity(Entity::new(EntityKind::Lander, 18, 5, -1, 1));
    world.spawn_entity(Entity::new(EntityKind::Lander, 26, 8, -1, 0));
    world.spawn_entity(Entity::new(EntityKind::Mutant, 22, 6, 1, -1));
    world.spawn_entity(Entity::new(EntityKind::EnemyShot, 20, 7, 1, -1));
    world.spawn_entity(Entity::new(EntityKind::PlayerShot, 15, 6, 2, 0));
    world.spawn_entity(Entity::new(
        EntityKind::Human,
        20,
        world.safe_altitude_at_world_x(20),
        0,
        0,
    ));

    Scene {
        kind: defender::attract::SceneKind::Attract,
        lines: render::render_grid(&world),
    }
}

fn scene_bounds(scenes: &[Scene]) -> (usize, usize) {
    let mut max_cols = 0;
    let mut max_rows = 0;

    for scene in scenes {
        max_rows = max_rows.max(scene.lines.len());
        for line in &scene.lines {
            max_cols = max_cols.max(line.len());
        }
    }

    (max_cols, max_rows)
}

fn scene_bounds_from_frames(frames: &[(Scene, u16)]) -> (usize, usize) {
    let mut max_cols = 0;
    let mut max_rows = 0;

    for (scene, _) in frames {
        max_rows = max_rows.max(scene.lines.len());
        for line in &scene.lines {
            max_cols = max_cols.max(line.len());
        }
    }

    (max_cols, max_rows)
}

fn centiseconds_for_beat(beat: AttractBeat) -> u16 {
    let centiseconds = (beat.hold_ms / 10).max(1);
    centiseconds.min(u16::MAX as u64) as u16
}

struct RgbaImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

fn rasterize_scene(scene: &Scene, cols: usize, rows: usize, scale: u32) -> RgbaImage {
    let width = PADDING * 2 + cols as u32 * CELL_SIZE * scale;
    let height = PADDING * 2 + rows as u32 * CELL_SIZE * scale;
    let mut pixels = vec![0; (width * height * 4) as usize];

    for chunk in pixels.chunks_exact_mut(4) {
        chunk.copy_from_slice(&BACKGROUND);
    }

    for (row, line) in scene.lines.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            draw_char(
                &mut pixels,
                width,
                PADDING + col as u32 * CELL_SIZE * scale,
                PADDING + row as u32 * CELL_SIZE * scale,
                ch,
                scale,
            );
        }
    }

    RgbaImage {
        width,
        height,
        pixels,
    }
}

fn draw_char(pixels: &mut [u8], width: u32, origin_x: u32, origin_y: u32, ch: char, scale: u32) {
    let Some(glyph) = BASIC_FONTS.get(ch) else {
        return;
    };

    for (row_index, row_bits) in glyph.iter().enumerate() {
        for col_index in 0..8u32 {
            if row_bits & (1 << col_index) == 0 {
                continue;
            }

            for dy in 0..scale {
                for dx in 0..scale {
                    let x = origin_x + col_index * scale + dx;
                    let y = origin_y + row_index as u32 * scale + dy;
                    let pixel_index = ((y * width + x) * 4) as usize;
                    pixels[pixel_index..pixel_index + 4].copy_from_slice(&FOREGROUND);
                }
            }
        }
    }
}

fn compose_windowed_screenshot(image: &RgbaImage) -> RgbaImage {
    let mut framed = RgbaImage::new(OUTPUT_WIDTH, OUTPUT_HEIGHT, [0, 0, 0, 0]);

    draw_rounded_rect(
        &mut framed,
        WINDOW_X + 18,
        WINDOW_Y + 26,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WINDOW_RADIUS,
        WINDOW_SHADOW,
    );
    draw_rounded_rect(
        &mut framed,
        WINDOW_X,
        WINDOW_Y,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WINDOW_RADIUS,
        WINDOW_COLOR,
    );
    fill_rect(
        &mut framed,
        CONTENT_X,
        CONTENT_Y,
        CONTENT_WIDTH,
        CONTENT_HEIGHT,
        CONTENT_BACKGROUND,
    );

    let button_y = WINDOW_Y + 34;
    draw_circle(&mut framed, WINDOW_X + 34, button_y, 14, TRAFFIC_RED);
    draw_circle(&mut framed, WINDOW_X + 82, button_y, 14, TRAFFIC_YELLOW);
    draw_circle(&mut framed, WINDOW_X + 130, button_y, 14, TRAFFIC_GREEN);
    draw_ghost_badge(&mut framed, WINDOW_X + WINDOW_WIDTH / 2, WINDOW_Y + 42, 2);

    blit_scaled_to_fit(
        image,
        &mut framed,
        CONTENT_X,
        CONTENT_Y,
        CONTENT_WIDTH,
        CONTENT_HEIGHT,
    );
    framed
}

fn fill_rect(image: &mut RgbaImage, x: u32, y: u32, width: u32, height: u32, color: [u8; 4]) {
    for py in y..y + height {
        for px in x..x + width {
            blend_pixel(image, px, py, color);
        }
    }
}

fn draw_circle(image: &mut RgbaImage, center_x: u32, center_y: u32, radius: u32, color: [u8; 4]) {
    let radius = radius as i32;
    let center_x = center_x as i32;
    let center_y = center_y as i32;

    for py in center_y - radius..=center_y + radius {
        for px in center_x - radius..=center_x + radius {
            let dx = px - center_x;
            let dy = py - center_y;
            if dx * dx + dy * dy <= radius * radius
                && let (Ok(px), Ok(py)) = (u32::try_from(px), u32::try_from(py))
            {
                blend_pixel(image, px, py, color);
            }
        }
    }
}

fn draw_rounded_rect(
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
    color: [u8; 4],
) {
    for py in y..y + height {
        for px in x..x + width {
            if point_inside_rounded_rect(px, py, x, y, width, height, radius) {
                blend_pixel(image, px, py, color);
            }
        }
    }
}

fn point_inside_rounded_rect(
    px: u32,
    py: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
) -> bool {
    let px = px as i32;
    let py = py as i32;
    let x = x as i32;
    let y = y as i32;
    let width = width as i32;
    let height = height as i32;
    let radius = radius as i32;
    let max_x = x + width - 1;
    let max_y = y + height - 1;

    if px < x || px > max_x || py < y || py > max_y {
        return false;
    }

    if px >= x + radius && px <= max_x - radius {
        return true;
    }

    if py >= y + radius && py <= max_y - radius {
        return true;
    }

    let center_x = if px < x + radius {
        x + radius
    } else {
        max_x - radius
    };
    let center_y = if py < y + radius {
        y + radius
    } else {
        max_y - radius
    };
    let dx = px - center_x;
    let dy = py - center_y;

    dx * dx + dy * dy <= radius * radius
}

fn draw_ghost_badge(image: &mut RgbaImage, center_x: u32, center_y: u32, scale: u32) {
    draw_circle(
        image,
        center_x,
        center_y - 5 * scale,
        8 * scale,
        GHOST_BADGE,
    );
    fill_rect(
        image,
        center_x - 8 * scale,
        center_y - 5 * scale,
        16 * scale,
        11 * scale,
        GHOST_BADGE,
    );
    draw_circle(
        image,
        center_x - 5 * scale,
        center_y + 5 * scale,
        3 * scale,
        GHOST_BADGE,
    );
    draw_circle(
        image,
        center_x,
        center_y + 6 * scale,
        3 * scale,
        GHOST_BADGE,
    );
    draw_circle(
        image,
        center_x + 5 * scale,
        center_y + 5 * scale,
        3 * scale,
        GHOST_BADGE,
    );
    draw_circle(
        image,
        center_x - 3 * scale,
        center_y - 3 * scale,
        scale,
        GHOST_EYE,
    );
    draw_circle(
        image,
        center_x + 3 * scale,
        center_y - 3 * scale,
        scale,
        GHOST_EYE,
    );
}

fn blit_scaled_to_fit(
    source: &RgbaImage,
    destination: &mut RgbaImage,
    frame_x: u32,
    frame_y: u32,
    frame_width: u32,
    frame_height: u32,
) {
    let width_ratio = frame_width as f32 / source.width as f32;
    let height_ratio = frame_height as f32 / source.height as f32;
    let scale = width_ratio.min(height_ratio);
    let target_width = (source.width as f32 * scale).round() as u32;
    let target_height = (source.height as f32 * scale).round() as u32;
    let offset_x = frame_x + (frame_width - target_width) / 2;
    let offset_y = frame_y + (frame_height - target_height) / 2;

    for y in 0..target_height {
        let source_y = (y as f32 * source.height as f32 / target_height as f32).floor() as u32;
        for x in 0..target_width {
            let source_x = (x as f32 * source.width as f32 / target_width as f32).floor() as u32;
            let source_index = ((source_y * source.width + source_x) * 4) as usize;
            blend_pixel(
                destination,
                offset_x + x,
                offset_y + y,
                source.pixels[source_index..source_index + 4]
                    .try_into()
                    .unwrap_or(CONTENT_BACKGROUND),
            );
        }
    }
}

fn blend_pixel(image: &mut RgbaImage, x: u32, y: u32, color: [u8; 4]) {
    if x >= image.width || y >= image.height {
        return;
    }

    let index = ((y * image.width + x) * 4) as usize;
    let destination = &mut image.pixels[index..index + 4];
    let alpha = color[3] as u32;
    let inverse = 255 - alpha;
    let destination_alpha = destination[3] as u32;

    for channel in 0..3 {
        destination[channel] = (((color[channel] as u32 * alpha)
            + (destination[channel] as u32 * inverse))
            / 255) as u8;
    }
    destination[3] = (alpha + (destination_alpha * inverse) / 255) as u8;
}

fn write_png(path: &Path, image: &RgbaImage) -> Result<()> {
    let file = File::create(path).with_context(|| format!("creating png {}", path.display()))?;
    let mut encoder = PngEncoder::new(file, image.width, image.height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(Compression::Fast);
    let mut writer = encoder.write_header().context("writing png header")?;
    writer
        .write_image_data(&image.pixels)
        .context("writing png data")?;
    Ok(())
}

fn write_gif(path: &Path, frames: &[(Scene, u16)], cols: usize, rows: usize) -> Result<()> {
    let image = rasterize_scene(&frames[0].0, cols, rows, GIF_SCALE);
    let file = File::create(path).with_context(|| format!("creating gif {}", path.display()))?;
    let mut encoder = Encoder::new(file, image.width as u16, image.height as u16, &[])
        .with_context(|| format!("creating gif encoder for {}", path.display()))?;
    encoder
        .set_repeat(Repeat::Infinite)
        .context("setting gif repeat mode")?;

    for (scene, delay) in frames {
        let image = rasterize_scene(scene, cols, rows, GIF_SCALE);
        let mut pixels = image.pixels.clone();
        let mut frame =
            Frame::from_rgba_speed(image.width as u16, image.height as u16, &mut pixels, 10);
        frame.delay = *delay;
        encoder.write_frame(&frame).context("writing gif frame")?;
    }

    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating directory {}", parent.display()))?;
    }
    Ok(())
}

impl RgbaImage {
    fn new(width: u32, height: u32, color: [u8; 4]) -> Self {
        let mut pixels = vec![0; (width * height * 4) as usize];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&color);
        }

        Self {
            width,
            height,
            pixels,
        }
    }
}
