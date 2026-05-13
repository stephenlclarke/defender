use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use defender::compatibility::{
    input::CabinetInput,
    machine::{ArcadeMachine, FRAME_RATE_MILLIHZ},
    video::{RenderedImage, Renderer},
};
use gif::{Encoder, Frame, Repeat};

const OUTPUT_WIDTH: u32 = 768;
const OUTPUT_HEIGHT: u32 = 576;
const SAMPLE_STEP_FRAMES: u64 = 8;
const WILLIAMS_START_FRAME: u64 = 1;
const HIGH_SCORE_START_FRAME: u64 = 3_000;
const FULL_ATTRACT_START_FRAME: u64 = 4_200;
const WILLIAMS_SECONDS: u64 = 8;
const HIGH_SCORE_SECONDS: u64 = 8;
const FULL_ATTRACT_SECONDS: u64 = 30;

fn main() -> Result<()> {
    let gif_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs/start-sequence.gif"));

    ensure_parent_dir(&gif_path)?;
    let sequence = build_start_sequence()?;
    write_gif(&gif_path, &sequence)?;

    println!("wrote {}", gif_path.display());
    Ok(())
}

fn build_start_sequence() -> Result<Vec<(RgbaImage, u16)>> {
    let mut machine = ArcadeMachine::new();
    let mut renderer = Renderer::with_size(OUTPUT_WIDTH, OUTPUT_HEIGHT);
    let mut machine_frame = 0;
    let mut delay = DelayAccumulator::new();
    let mut frames = Vec::new();

    for segment in readme_segments() {
        step_until_frame(&mut machine, &mut machine_frame, segment.start_frame);
        capture_segment(
            &mut machine,
            &mut renderer,
            &mut machine_frame,
            &mut delay,
            segment,
            &mut frames,
        )?;
    }

    collapse_identical_frames(frames)
}

fn readme_segments() -> [ReadmeSegment; 3] {
    [
        ReadmeSegment::new(
            WILLIAMS_START_FRAME,
            frame_count_for_seconds(WILLIAMS_SECONDS),
        ),
        ReadmeSegment::new(
            HIGH_SCORE_START_FRAME,
            frame_count_for_seconds(HIGH_SCORE_SECONDS),
        ),
        ReadmeSegment::new(
            FULL_ATTRACT_START_FRAME,
            frame_count_for_seconds(FULL_ATTRACT_SECONDS),
        ),
    ]
}

fn capture_segment(
    machine: &mut ArcadeMachine,
    renderer: &mut Renderer,
    machine_frame: &mut u64,
    delay: &mut DelayAccumulator,
    segment: ReadmeSegment,
    frames: &mut Vec<(RgbaImage, u16)>,
) -> Result<()> {
    let segment_end = segment
        .start_frame
        .checked_add(segment.frame_count)
        .context("README media segment frame range overflowed")?;
    while *machine_frame < segment_end {
        let step_frames = SAMPLE_STEP_FRAMES.min(segment_end - *machine_frame);
        frames.push((
            render_machine_frame(machine, renderer)?,
            delay.centiseconds_for_frames(step_frames),
        ));
        step_for_frames(machine, machine_frame, step_frames);
    }

    Ok(())
}

fn render_machine_frame(machine: &mut ArcadeMachine, renderer: &mut Renderer) -> Result<RgbaImage> {
    machine
        .red_label_copy_color_mapping_to_palette_ram()
        .map_err(anyhow::Error::msg)?;
    let native_frame = machine
        .red_label_visible_rgba_image()
        .context("red-label visible frame is unavailable")?;
    Ok(renderer.render_cabinet_frame(&native_frame).into())
}

fn step_until_frame(machine: &mut ArcadeMachine, machine_frame: &mut u64, target_frame: u64) {
    if target_frame > *machine_frame {
        step_for_frames(machine, machine_frame, target_frame - *machine_frame);
    }
}

fn step_for_frames(machine: &mut ArcadeMachine, machine_frame: &mut u64, frames: u64) {
    for _ in 0..frames {
        machine.step(CabinetInput::NONE);
        *machine_frame += 1;
    }
}

const fn frame_count_for_seconds(seconds: u64) -> u64 {
    (seconds * FRAME_RATE_MILLIHZ as u64).div_ceil(1_000)
}

fn collapse_identical_frames(frames: Vec<(RgbaImage, u16)>) -> Result<Vec<(RgbaImage, u16)>> {
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

    if collapsed.is_empty() {
        bail!("README media sequence did not produce any frames");
    }

    Ok(collapsed)
}

fn write_gif(path: &Path, frames: &[(RgbaImage, u16)]) -> Result<()> {
    let first = &frames[0].0;
    let file = File::create(path).with_context(|| format!("creating GIF {}", path.display()))?;
    let mut encoder = Encoder::new(file, first.width as u16, first.height as u16, &[])
        .with_context(|| format!("creating GIF encoder for {}", path.display()))?;
    encoder
        .set_repeat(Repeat::Infinite)
        .context("setting GIF repeat mode")?;

    for (image, delay) in frames {
        let mut pixels = image.pixels.clone();
        let mut frame =
            Frame::from_rgba_speed(image.width as u16, image.height as u16, &mut pixels, 30);
        frame.delay = *delay;
        encoder.write_frame(&frame).context("writing GIF frame")?;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReadmeSegment {
    start_frame: u64,
    frame_count: u64,
}

impl ReadmeSegment {
    const fn new(start_frame: u64, frame_count: u64) -> Self {
        Self {
            start_frame,
            frame_count,
        }
    }
}

struct DelayAccumulator {
    remainder: u64,
}

impl DelayAccumulator {
    const fn new() -> Self {
        Self { remainder: 0 }
    }

    fn centiseconds_for_frames(&mut self, frames: u64) -> u16 {
        let total = frames * 100_000 + self.remainder;
        let rate = u64::from(FRAME_RATE_MILLIHZ);
        let centiseconds = total / rate;
        self.remainder = total % rate;
        u16::try_from(centiseconds.max(1)).unwrap_or(u16::MAX)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct RgbaImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl From<&RenderedImage> for RgbaImage {
    fn from(image: &RenderedImage) -> Self {
        Self {
            width: image.width,
            height: image.height,
            pixels: image.pixels.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DelayAccumulator, FULL_ATTRACT_SECONDS, HIGH_SCORE_SECONDS, ReadmeSegment,
        SAMPLE_STEP_FRAMES, WILLIAMS_SECONDS, frame_count_for_seconds, readme_segments,
    };

    #[test]
    fn readme_segments_hold_first_two_screens_for_eight_seconds() {
        assert_eq!(
            readme_segments()[0],
            ReadmeSegment::new(1, frame_count_for_seconds(WILLIAMS_SECONDS))
        );
        assert_eq!(
            readme_segments()[1],
            ReadmeSegment::new(3_000, frame_count_for_seconds(HIGH_SCORE_SECONDS))
        );
        assert_eq!(WILLIAMS_SECONDS, 8);
        assert_eq!(HIGH_SCORE_SECONDS, 8);
        assert_eq!(FULL_ATTRACT_SECONDS, 30);
    }

    #[test]
    fn delay_accumulator_preserves_cabinet_frame_rate_over_samples() {
        let mut delay = DelayAccumulator::new();
        let delays: Vec<u16> = (0..10)
            .map(|_| delay.centiseconds_for_frames(SAMPLE_STEP_FRAMES))
            .collect();

        assert!(delays.iter().all(|delay| *delay >= 13 && *delay <= 14));
        assert_eq!(delays.iter().copied().map(u32::from).sum::<u32>(), 133);
    }
}
