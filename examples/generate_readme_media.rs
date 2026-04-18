use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use gif::{Encoder, Frame, Repeat};
use png::{BitDepth, ColorType, Compression, Encoder as PngEncoder};

use defender::{
    attract::{AttractBeat, Scene, attract_cycle},
    demo::gameplay_demo_cycle,
    render,
};

const CELL_SIZE: u32 = 8;
const SCALE: u32 = 3;
const PADDING: u32 = 16;
const BACKGROUND: [u8; 4] = [4, 8, 14, 255];
const FOREGROUND: [u8; 4] = [80, 255, 140, 255];

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

    let screenshot = rasterize_scene(&screenshot_scene, screenshot_cols, screenshot_rows);
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
    let gameplay = gameplay_demo_cycle();
    let world = &gameplay[gameplay.len() - 1].world;

    Scene {
        kind: defender::attract::SceneKind::Attract,
        lines: render::render_grid(world),
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

fn rasterize_scene(scene: &Scene, cols: usize, rows: usize) -> RgbaImage {
    let width = PADDING * 2 + cols as u32 * CELL_SIZE * SCALE;
    let height = PADDING * 2 + rows as u32 * CELL_SIZE * SCALE;
    let mut pixels = vec![0; (width * height * 4) as usize];

    for chunk in pixels.chunks_exact_mut(4) {
        chunk.copy_from_slice(&BACKGROUND);
    }

    for (row, line) in scene.lines.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            draw_char(
                &mut pixels,
                width,
                PADDING + col as u32 * CELL_SIZE * SCALE,
                PADDING + row as u32 * CELL_SIZE * SCALE,
                ch,
            );
        }
    }

    RgbaImage {
        width,
        height,
        pixels,
    }
}

fn draw_char(pixels: &mut [u8], width: u32, origin_x: u32, origin_y: u32, ch: char) {
    let Some(glyph) = BASIC_FONTS.get(ch) else {
        return;
    };

    for (row_index, row_bits) in glyph.iter().enumerate() {
        for col_index in 0..8u32 {
            if row_bits & (1 << col_index) == 0 {
                continue;
            }

            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    let x = origin_x + col_index * SCALE + dx;
                    let y = origin_y + row_index as u32 * SCALE + dy;
                    let pixel_index = ((y * width + x) * 4) as usize;
                    pixels[pixel_index..pixel_index + 4].copy_from_slice(&FOREGROUND);
                }
            }
        }
    }
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
    let image = rasterize_scene(&frames[0].0, cols, rows);
    let file = File::create(path).with_context(|| format!("creating gif {}", path.display()))?;
    let mut encoder = Encoder::new(file, image.width as u16, image.height as u16, &[])
        .with_context(|| format!("creating gif encoder for {}", path.display()))?;
    encoder
        .set_repeat(Repeat::Infinite)
        .context("setting gif repeat mode")?;

    for (scene, delay) in frames {
        let image = rasterize_scene(scene, cols, rows);
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
