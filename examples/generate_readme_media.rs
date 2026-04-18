use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gif::{Encoder, Frame, Repeat};
use png::{BitDepth, ColorType, Compression, Encoder as PngEncoder};

use defender::{
    attract::{AttractBeat, attract_cycle},
    demo::gameplay_demo_cycle,
    game::{Entity, EntityKind, World},
    high_scores::HighScoreTable,
    video::{RenderedImage, Renderer, Screen},
};

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
const README_GIF_SAMPLE_MS: u64 = 300;
const README_GIF_REALTIME_PERCENT: u64 = 40;
const README_GIF_MIN_DELAY_CS: u16 = 4;

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

    let screenshot = render_gameplay_screenshot();
    let sequence = build_sequence();
    let screenshot = compose_windowed_screenshot(&screenshot);
    write_png(&screenshot_path, &screenshot)?;
    write_gif(&gif_path, &sequence)?;

    println!("wrote {}", screenshot_path.display());
    println!("wrote {}", gif_path.display());
    Ok(())
}

fn build_sequence() -> Vec<(RgbaImage, u16)> {
    let mut renderer = Renderer::with_size(1_024, 768);
    let todays = HighScoreTable::default();
    let all_time = HighScoreTable::default();
    let cycle = attract_cycle();
    let cycle_ms = cycle.iter().map(|beat| beat.hold_ms).sum::<u64>();
    let frame_delay = scaled_centiseconds(README_GIF_SAMPLE_MS);
    let mut frames = Vec::new();
    let mut elapsed_ms = 0;

    while elapsed_ms < cycle_ms {
        let image = render_attract_sample(&mut renderer, &todays, &all_time, &cycle, elapsed_ms);
        frames.push((image.into(), frame_delay));
        elapsed_ms += README_GIF_SAMPLE_MS;
    }

    collapse_identical_frames(frames)
}

fn gameplay_screenshot_world() -> World {
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
    world
}

fn render_attract_sample(
    renderer: &mut Renderer,
    todays: &HighScoreTable,
    all_time: &HighScoreTable,
    cycle: &[AttractBeat],
    elapsed_ms: u64,
) -> RenderedImage {
    let (beat_index, beat_start_ms, beat) = cycle_entry_for_elapsed(cycle, elapsed_ms);
    let offset_ms = elapsed_ms.saturating_sub(beat_start_ms);
    let next_beat = cycle[(beat_index + 1) % cycle.len()];

    match beat.kind {
        defender::attract::SceneKind::Logo => renderer
            .render(Screen::Logo {
                palette_phase: beat.palette_phase,
                trace_points: beat.logo_trace_points,
                show_title_text: beat.logo_show_title_text,
                visible_defender_chunks: beat.logo_visible_defender_chunks,
                show_copyright: beat.logo_show_copyright,
            })
            .clone(),
        defender::attract::SceneKind::Attract => {
            let mut world = World::bootstrap();
            let world_steps = interpolated_world_steps(beat, next_beat, offset_ms);
            for _ in 0..world_steps {
                world.step();
            }
            renderer
                .render(Screen::Attract {
                    world: &world,
                    revealed_score_entries: beat.revealed_score_entries,
                    palette_phase: beat.palette_phase,
                })
                .clone()
        }
        defender::attract::SceneKind::HighScore => renderer
            .render(Screen::HighScores {
                todays,
                all_time,
                palette_phase: beat.palette_phase,
            })
            .clone(),
    }
}

fn cycle_entry_for_elapsed(cycle: &[AttractBeat], elapsed_ms: u64) -> (usize, u64, AttractBeat) {
    let cycle_ms = cycle.iter().map(|beat| beat.hold_ms).sum::<u64>();
    let mut beat_start_ms = 0;
    let mut remaining_ms = if cycle_ms == 0 {
        0
    } else {
        elapsed_ms % cycle_ms
    };

    for (index, beat) in cycle.iter().copied().enumerate() {
        if remaining_ms < beat.hold_ms {
            return (index, beat_start_ms, beat);
        }
        remaining_ms -= beat.hold_ms;
        beat_start_ms += beat.hold_ms;
    }

    (0, 0, cycle[0])
}

fn interpolated_world_steps(beat: AttractBeat, next_beat: AttractBeat, offset_ms: u64) -> usize {
    if beat.kind != defender::attract::SceneKind::Attract
        || next_beat.kind != defender::attract::SceneKind::Attract
        || beat.hold_ms == 0
    {
        return beat.world_steps;
    }

    let progress = (offset_ms as f32 / beat.hold_ms as f32).clamp(0.0, 1.0);
    let start = beat.world_steps as f32;
    let end = next_beat.world_steps as f32;
    (start + (end - start) * progress).round().max(0.0) as usize
}

fn scaled_centiseconds(duration_ms: u64) -> u16 {
    let scaled_ms = duration_ms
        .saturating_mul(README_GIF_REALTIME_PERCENT)
        .checked_div(100)
        .unwrap_or(duration_ms);
    let centiseconds = ((scaled_ms + 5) / 10).max(u64::from(README_GIF_MIN_DELAY_CS));
    centiseconds.min(u16::MAX as u64) as u16
}

fn collapse_identical_frames(frames: Vec<(RgbaImage, u16)>) -> Vec<(RgbaImage, u16)> {
    let mut collapsed: Vec<(RgbaImage, u16)> = Vec::new();

    for (image, delay) in frames {
        if let Some((previous, previous_delay)) = collapsed.last_mut()
            && previous == &image
        {
            *previous_delay = previous_delay.saturating_add(delay);
        } else {
            collapsed.push((image, delay));
        }
    }

    collapsed
}

fn render_gameplay_screenshot() -> RgbaImage {
    let world = gameplay_screenshot_world();
    let mut renderer = Renderer::with_size(1_600, 900);
    renderer
        .render(Screen::Playing {
            world: &world,
            xyzzy_active: false,
            invincible: false,
            auto_fire: false,
        })
        .clone()
        .into()
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

fn write_gif(path: &Path, frames: &[(RgbaImage, u16)]) -> Result<()> {
    let image = &frames[0].0;
    let file = File::create(path).with_context(|| format!("creating gif {}", path.display()))?;
    let mut encoder = Encoder::new(file, image.width as u16, image.height as u16, &[])
        .with_context(|| format!("creating gif encoder for {}", path.display()))?;
    encoder
        .set_repeat(Repeat::Infinite)
        .context("setting gif repeat mode")?;

    for (image, delay) in frames {
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

#[derive(Clone, PartialEq, Eq)]
struct RgbaImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
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

impl From<RenderedImage> for RgbaImage {
    fn from(image: RenderedImage) -> Self {
        Self {
            width: image.width,
            height: image.height,
            pixels: image.pixels,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        README_GIF_MIN_DELAY_CS, README_GIF_SAMPLE_MS, RgbaImage, collapse_identical_frames,
        scaled_centiseconds,
    };

    #[test]
    fn scaled_centiseconds_speeds_up_realtime_for_readme_media() {
        let delay = scaled_centiseconds(README_GIF_SAMPLE_MS);

        assert!(delay < (README_GIF_SAMPLE_MS / 10) as u16);
        assert!(delay >= README_GIF_MIN_DELAY_CS);
    }

    #[test]
    fn collapse_identical_frames_merges_static_runs() {
        let black = RgbaImage::new(4, 4, [0, 0, 0, 255]);
        let white = RgbaImage::new(4, 4, [255, 255, 255, 255]);
        let frames = vec![
            (black.clone(), 4),
            (black, 4),
            (white.clone(), 5),
            (white, 6),
        ];

        let collapsed = collapse_identical_frames(frames);

        assert_eq!(collapsed.len(), 2);
        assert_eq!(collapsed[0].1, 8);
        assert_eq!(collapsed[1].1, 11);
    }
}
