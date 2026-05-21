use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use defender::AttractPresentationPage;
use defender::readme_media::{FRAME_RATE_MILLIHZ, ReadmeMediaFrame, ReadmeMediaFrameSource};
use gif::{ColorOutput, DisposalMethod, Encoder, Frame, Repeat};

const OUTPUT_WIDTH: u32 = 768;
const OUTPUT_HEIGHT: u32 = 576;
const SAMPLE_STEP_FRAMES: u64 = 8;
const PAGE_SEARCH_LIMIT_FRAMES: u64 = 10_000;
const WILLIAMS_SECONDS: u64 = 8;
const HIGH_SCORE_SECONDS: u64 = 8;
const FULL_ATTRACT_SECONDS: u64 = 30;
const PROTECTED_REFERENCE_GIF: &str = "docs/start-sequence.gif";
const DEFAULT_CANDIDATE_GIF: &str = "target/readme-media/start-sequence-candidate.gif";
const ALLOW_REFERENCE_OVERWRITE_ENV: &str = "DEFENDER_ALLOW_REFERENCE_MEDIA_OVERWRITE";
const VISUAL_SAMPLE_STRIDE: u32 = 8;
const RGBA_CHANNELS: usize = 4;
const RGB_CHANNELS: usize = 3;

fn main() -> Result<()> {
    let gif_path = gif_path_from_args(std::env::args_os().skip(1));
    ensure_reference_overwrite_allowed(&gif_path)?;

    ensure_parent_dir(&gif_path)?;
    let sequence = build_start_sequence()?;
    write_gif(&gif_path, &sequence)?;

    println!("wrote {}", gif_path.display());
    print_reference_comparison(&gif_path)?;
    Ok(())
}

fn gif_path_from_args(mut args: impl Iterator<Item = std::ffi::OsString>) -> PathBuf {
    args.next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CANDIDATE_GIF))
}

fn ensure_reference_overwrite_allowed(path: &Path) -> Result<()> {
    if is_protected_reference_path(path)
        && std::env::var_os(ALLOW_REFERENCE_OVERWRITE_ENV).as_deref() != Some("1".as_ref())
    {
        bail!(
            "{} is the protected B13 visual reference; write a candidate path \
             instead, or set {ALLOW_REFERENCE_OVERWRITE_ENV}=1 after owner \
             approval",
            PROTECTED_REFERENCE_GIF
        );
    }

    Ok(())
}

fn is_protected_reference_path(path: &Path) -> bool {
    normalized_path(path) == normalized_path(Path::new(PROTECTED_REFERENCE_GIF))
}

fn normalized_path(path: &Path) -> PathBuf {
    path.components()
        .filter(|component| !matches!(component, std::path::Component::CurDir))
        .collect()
}

fn build_start_sequence() -> Result<Vec<(RgbaImage, u16)>> {
    let mut source = ReadmeMediaFrameSource::new(OUTPUT_WIDTH, OUTPUT_HEIGHT);
    let mut delay = DelayAccumulator::new();
    let mut frames = Vec::new();

    for segment in readme_segments() {
        step_until_segment_start(&mut source, segment.start)?;
        capture_segment(&mut source, &mut delay, segment, &mut frames)?;
    }

    preserve_sampled_cadence(frames)
}

fn readme_segments() -> [ReadmeSegment; 3] {
    [
        ReadmeSegment::new(
            ReadmeSegmentStart::Current,
            frame_count_for_seconds(WILLIAMS_SECONDS),
        ),
        ReadmeSegment::new(
            ReadmeSegmentStart::AttractPage(AttractPresentationPage::HallOfFame),
            frame_count_for_seconds(HIGH_SCORE_SECONDS),
        ),
        ReadmeSegment::new(
            ReadmeSegmentStart::AttractPage(AttractPresentationPage::ScoringSequence),
            frame_count_for_seconds(FULL_ATTRACT_SECONDS),
        ),
    ]
}

fn step_until_segment_start(
    source: &mut ReadmeMediaFrameSource,
    start: ReadmeSegmentStart,
) -> Result<()> {
    match start {
        ReadmeSegmentStart::Current => Ok(()),
        ReadmeSegmentStart::AttractPage(page) => step_until_attract_page(source, page),
    }
}

fn step_until_attract_page(
    source: &mut ReadmeMediaFrameSource,
    page: AttractPresentationPage,
) -> Result<()> {
    for _ in 0..PAGE_SEARCH_LIMIT_FRAMES {
        if source.attract_page() == page {
            return Ok(());
        }
        source.step();
    }

    bail!("README media source did not reach {page:?} within {PAGE_SEARCH_LIMIT_FRAMES} frames")
}

fn capture_segment(
    source: &mut ReadmeMediaFrameSource,
    delay: &mut DelayAccumulator,
    segment: ReadmeSegment,
    frames: &mut Vec<(RgbaImage, u16)>,
) -> Result<()> {
    let mut remaining = segment.frame_count;
    while remaining > 0 {
        let step_frames = SAMPLE_STEP_FRAMES.min(remaining);
        frames.push((
            source
                .render_frame()
                .context("rendering README media frame")?
                .into(),
            delay.centiseconds_for_frames(step_frames),
        ));
        step_for_frames(source, step_frames);
        remaining -= step_frames;
    }

    Ok(())
}

fn step_for_frames(source: &mut ReadmeMediaFrameSource, frames: u64) {
    for _ in 0..frames {
        source.step();
    }
}

const fn frame_count_for_seconds(seconds: u64) -> u64 {
    (seconds * FRAME_RATE_MILLIHZ as u64).div_ceil(1_000)
}

fn preserve_sampled_cadence(mut frames: Vec<(RgbaImage, u16)>) -> Result<Vec<(RgbaImage, u16)>> {
    if frames.is_empty() {
        bail!("README media sequence did not produce any frames");
    }

    if frames.len() > 1 {
        let second = frames.remove(1);
        frames[0].1 = frames[0].1.saturating_add(second.1);
    }

    Ok(frames)
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

fn print_reference_comparison(candidate_path: &Path) -> Result<()> {
    let reference_path = Path::new(PROTECTED_REFERENCE_GIF);
    if is_protected_reference_path(candidate_path) || !reference_path.exists() {
        return Ok(());
    }

    let comparison = compare_gifs(reference_path, candidate_path).with_context(|| {
        format!(
            "comparing candidate {} against protected reference {}",
            candidate_path.display(),
            reference_path.display()
        )
    })?;

    println!("comparison reference {}", reference_path.display());
    println!(
        "  reference: {} frames, {}x{}, {}cs",
        comparison.reference.frame_count,
        comparison.reference.width,
        comparison.reference.height,
        comparison.reference.total_delay_cs
    );
    println!(
        "  candidate: {} frames, {}x{}, {}cs",
        comparison.candidate.frame_count,
        comparison.candidate.width,
        comparison.candidate.height,
        comparison.candidate.total_delay_cs
    );
    println!(
        "  deltas: frames {:+}, delay {:+}cs",
        comparison.frame_count_delta, comparison.total_delay_delta_cs
    );
    println!("  sampled rms:");
    println!("    full: {:.2}", comparison.full_rms);
    for region in &comparison.regions {
        println!("    {}: {:.2}", region.name, region.rms);
    }

    Ok(())
}

fn compare_gifs(reference_path: &Path, candidate_path: &Path) -> Result<GifComparison> {
    compare_summaries(
        decode_gif_summary(reference_path)
            .with_context(|| format!("decoding reference GIF {}", reference_path.display()))?,
        decode_gif_summary(candidate_path)
            .with_context(|| format!("decoding candidate GIF {}", candidate_path.display()))?,
    )
}

fn compare_summaries(reference: GifSummary, candidate: GifSummary) -> Result<GifComparison> {
    if reference.frame_count == 0 || candidate.frame_count == 0 {
        bail!("cannot compare empty GIF summaries");
    }

    let full_rms = sampled_rms(&reference, &candidate, None)
        .context("GIF summaries have no shared sampled pixels")?;
    let regions = VISUAL_REGIONS
        .iter()
        .copied()
        .filter_map(|region| {
            sampled_rms(&reference, &candidate, Some(region)).map(|rms| RegionComparison {
                name: region.name,
                rms,
            })
        })
        .collect();

    Ok(GifComparison {
        frame_count_delta: candidate.frame_count as i64 - reference.frame_count as i64,
        total_delay_delta_cs: candidate.total_delay_cs as i64 - reference.total_delay_cs as i64,
        full_rms,
        regions,
        reference,
        candidate,
    })
}

fn decode_gif_summary(path: &Path) -> Result<GifSummary> {
    let file = File::open(path).with_context(|| format!("opening GIF {}", path.display()))?;
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(ColorOutput::RGBA);
    let mut decoder = options
        .read_info(file)
        .with_context(|| format!("reading GIF header {}", path.display()))?;
    let width = decoder.width();
    let height = decoder.height();
    let mut canvas = vec![0; usize::from(width) * usize::from(height) * RGBA_CHANNELS];
    for pixel in canvas.chunks_exact_mut(RGBA_CHANNELS) {
        pixel.copy_from_slice(&[0, 0, 0, 0xFF]);
    }

    let mut frames = Vec::new();
    let mut total_delay_cs = 0_u64;

    while let Some(frame) = decoder
        .read_next_frame()
        .with_context(|| format!("reading GIF frame {}", path.display()))?
    {
        let previous_canvas = if frame.dispose == DisposalMethod::Previous {
            Some(canvas.clone())
        } else {
            None
        };
        blit_gif_frame(&mut canvas, width, height, frame);
        total_delay_cs += u64::from(frame.delay);
        frames.push(GifFrameSample {
            pixels: sample_canvas(&canvas, width, height),
        });
        dispose_gif_frame(&mut canvas, width, height, frame, previous_canvas);
    }

    Ok(GifSummary {
        width,
        height,
        frame_count: frames.len(),
        total_delay_cs,
        sample_width: sample_extent(u32::from(width)),
        sample_height: sample_extent(u32::from(height)),
        frames,
    })
}

fn blit_gif_frame(canvas: &mut [u8], canvas_width: u16, canvas_height: u16, frame: &Frame<'_>) {
    for frame_y in 0..frame.height {
        let canvas_y = u32::from(frame.top) + u32::from(frame_y);
        if canvas_y >= u32::from(canvas_height) {
            continue;
        }
        for frame_x in 0..frame.width {
            let canvas_x = u32::from(frame.left) + u32::from(frame_x);
            if canvas_x >= u32::from(canvas_width) {
                continue;
            }

            let source_index = (usize::from(frame_y) * usize::from(frame.width)
                + usize::from(frame_x))
                * RGBA_CHANNELS;
            if source_index + RGBA_CHANNELS > frame.buffer.len() {
                continue;
            }
            let source = &frame.buffer[source_index..source_index + RGBA_CHANNELS];
            if source[3] == 0 {
                continue;
            }

            let target_index =
                (canvas_y as usize * usize::from(canvas_width) + canvas_x as usize) * RGBA_CHANNELS;
            canvas[target_index..target_index + RGBA_CHANNELS]
                .copy_from_slice(&[source[0], source[1], source[2], 0xFF]);
        }
    }
}

fn dispose_gif_frame(
    canvas: &mut [u8],
    canvas_width: u16,
    canvas_height: u16,
    frame: &Frame<'_>,
    previous_canvas: Option<Vec<u8>>,
) {
    match frame.dispose {
        DisposalMethod::Background => {
            clear_gif_frame_rect(canvas, canvas_width, canvas_height, frame)
        }
        DisposalMethod::Previous => {
            if let Some(previous_canvas) = previous_canvas {
                canvas.copy_from_slice(&previous_canvas);
            }
        }
        DisposalMethod::Any | DisposalMethod::Keep => {}
    }
}

fn clear_gif_frame_rect(
    canvas: &mut [u8],
    canvas_width: u16,
    canvas_height: u16,
    frame: &Frame<'_>,
) {
    for frame_y in 0..frame.height {
        let canvas_y = u32::from(frame.top) + u32::from(frame_y);
        if canvas_y >= u32::from(canvas_height) {
            continue;
        }
        for frame_x in 0..frame.width {
            let canvas_x = u32::from(frame.left) + u32::from(frame_x);
            if canvas_x >= u32::from(canvas_width) {
                continue;
            }
            let target_index =
                (canvas_y as usize * usize::from(canvas_width) + canvas_x as usize) * RGBA_CHANNELS;
            canvas[target_index..target_index + RGBA_CHANNELS].copy_from_slice(&[0, 0, 0, 0xFF]);
        }
    }
}

fn sample_canvas(canvas: &[u8], width: u16, height: u16) -> Vec<[u8; RGBA_CHANNELS]> {
    let mut samples = Vec::with_capacity(
        sample_extent(u32::from(width)) as usize * sample_extent(u32::from(height)) as usize,
    );
    for sample_y in 0..sample_extent(u32::from(height)) {
        let y = sampled_pixel_axis(sample_y, u32::from(height));
        for sample_x in 0..sample_extent(u32::from(width)) {
            let x = sampled_pixel_axis(sample_x, u32::from(width));
            let index = (y as usize * usize::from(width) + x as usize) * RGBA_CHANNELS;
            samples.push([
                canvas[index],
                canvas[index + 1],
                canvas[index + 2],
                canvas[index + 3],
            ]);
        }
    }
    samples
}

const fn sample_extent(axis_extent: u32) -> u32 {
    axis_extent.div_ceil(VISUAL_SAMPLE_STRIDE)
}

fn sampled_pixel_axis(sample_axis: u32, axis_extent: u32) -> u32 {
    (sample_axis * VISUAL_SAMPLE_STRIDE + VISUAL_SAMPLE_STRIDE / 2)
        .min(axis_extent.saturating_sub(1))
}

fn sampled_rms(
    reference: &GifSummary,
    candidate: &GifSummary,
    region: Option<VisualRegion>,
) -> Option<f64> {
    let frame_count = reference.frame_count.min(candidate.frame_count);
    let sample_width = reference.sample_width.min(candidate.sample_width);
    let sample_height = reference.sample_height.min(candidate.sample_height);
    let mut sum_squared = 0_u64;
    let mut channel_count = 0_u64;

    for frame_index in 0..frame_count {
        for sample_y in 0..sample_height {
            let y = sampled_pixel_axis(sample_y, u32::from(reference.height.min(candidate.height)));
            for sample_x in 0..sample_width {
                let x =
                    sampled_pixel_axis(sample_x, u32::from(reference.width.min(candidate.width)));
                if region.is_some_and(|region| !region.contains(x, y)) {
                    continue;
                }
                let reference_index =
                    sample_y as usize * reference.sample_width as usize + sample_x as usize;
                let candidate_index =
                    sample_y as usize * candidate.sample_width as usize + sample_x as usize;
                let reference_pixel = reference.frames[frame_index].pixels[reference_index];
                let candidate_pixel = candidate.frames[frame_index].pixels[candidate_index];
                for channel in 0..RGB_CHANNELS {
                    let diff =
                        i16::from(reference_pixel[channel]) - i16::from(candidate_pixel[channel]);
                    sum_squared += u64::from(diff.unsigned_abs()).pow(2);
                    channel_count += 1;
                }
            }
        }
    }

    (channel_count > 0).then(|| (sum_squared as f64 / channel_count as f64).sqrt())
}

const VISUAL_REGIONS: [VisualRegion; 6] = [
    VisualRegion::new("title", 0, 0, OUTPUT_WIDTH, 220),
    VisualRegion::new("hall_of_fame", 120, 48, 528, 360),
    VisualRegion::new("numeric_glyphs", 0, 0, OUTPUT_WIDTH, 120),
    VisualRegion::new("sprites", 0, 120, OUTPUT_WIDTH, 300),
    VisualRegion::new("terrain", 0, 390, OUTPUT_WIDTH, 186),
    VisualRegion::new("scoring", 0, 96, OUTPUT_WIDTH, 360),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VisualRegion {
    name: &'static str,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl VisualRegion {
    const fn new(name: &'static str, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            name,
            x,
            y,
            width,
            height,
        }
    }

    const fn contains(self, x: u32, y: u32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
}

#[derive(Debug, Clone, PartialEq)]
struct GifComparison {
    reference: GifSummary,
    candidate: GifSummary,
    frame_count_delta: i64,
    total_delay_delta_cs: i64,
    full_rms: f64,
    regions: Vec<RegionComparison>,
}

#[derive(Debug, Clone, PartialEq)]
struct RegionComparison {
    name: &'static str,
    rms: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GifSummary {
    width: u16,
    height: u16,
    frame_count: usize,
    total_delay_cs: u64,
    sample_width: u32,
    sample_height: u32,
    frames: Vec<GifFrameSample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GifFrameSample {
    pixels: Vec<[u8; RGBA_CHANNELS]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadmeSegmentStart {
    Current,
    AttractPage(AttractPresentationPage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReadmeSegment {
    start: ReadmeSegmentStart,
    frame_count: u64,
}

impl ReadmeSegment {
    const fn new(start: ReadmeSegmentStart, frame_count: u64) -> Self {
        Self { start, frame_count }
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

impl From<ReadmeMediaFrame> for RgbaImage {
    fn from(image: ReadmeMediaFrame) -> Self {
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
        AttractPresentationPage, DEFAULT_CANDIDATE_GIF, DelayAccumulator, FULL_ATTRACT_SECONDS,
        GifFrameSample, GifSummary, HIGH_SCORE_SECONDS, PROTECTED_REFERENCE_GIF, RGBA_CHANNELS,
        ReadmeSegment, ReadmeSegmentStart, RgbaImage, SAMPLE_STEP_FRAMES, VISUAL_REGIONS,
        WILLIAMS_SECONDS, compare_summaries, frame_count_for_seconds, gif_path_from_args,
        is_protected_reference_path, preserve_sampled_cadence, readme_segments,
    };
    use std::{ffi::OsString, path::Path};

    #[test]
    fn readme_segments_follow_clean_acceptance_order() {
        assert_eq!(
            readme_segments()[0],
            ReadmeSegment::new(
                ReadmeSegmentStart::Current,
                frame_count_for_seconds(WILLIAMS_SECONDS)
            )
        );
        assert_eq!(
            readme_segments()[1],
            ReadmeSegment::new(
                ReadmeSegmentStart::AttractPage(AttractPresentationPage::HallOfFame),
                frame_count_for_seconds(HIGH_SCORE_SECONDS)
            )
        );
        assert_eq!(
            readme_segments()[2],
            ReadmeSegment::new(
                ReadmeSegmentStart::AttractPage(AttractPresentationPage::ScoringSequence),
                frame_count_for_seconds(FULL_ATTRACT_SECONDS)
            )
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

    #[test]
    fn sampled_cadence_merges_initial_reference_hold_only() {
        let blank = RgbaImage {
            width: 2,
            height: 1,
            pixels: vec![0; 8],
        };
        let first_visible = RgbaImage {
            width: 2,
            height: 1,
            pixels: vec![0xFF; 8],
        };

        let frames = preserve_sampled_cadence(vec![
            (blank.clone(), 13),
            (blank.clone(), 13),
            (first_visible.clone(), 14),
            (first_visible.clone(), 13),
        ])
        .expect("cadence frames");

        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].1, 26);
        assert_eq!(frames[1].1, 14);
        assert_eq!(frames[2].1, 13);
    }

    #[test]
    fn default_output_path_is_candidate_not_protected_reference() {
        let args = Vec::<OsString>::new();

        assert_eq!(
            gif_path_from_args(args.into_iter()),
            Path::new(DEFAULT_CANDIDATE_GIF)
        );
        assert!(!is_protected_reference_path(Path::new(
            DEFAULT_CANDIDATE_GIF
        )));
    }

    #[test]
    fn protected_reference_path_allows_dot_prefix_detection() {
        assert!(is_protected_reference_path(Path::new(
            PROTECTED_REFERENCE_GIF
        )));
        assert!(is_protected_reference_path(Path::new(
            "./docs/start-sequence.gif"
        )));
    }

    #[test]
    fn visual_regions_track_reopened_b13_failure_areas() {
        let names: Vec<&str> = VISUAL_REGIONS.iter().map(|region| region.name).collect();

        assert!(names.contains(&"title"));
        assert!(names.contains(&"numeric_glyphs"));
        assert!(names.contains(&"sprites"));
        assert!(names.contains(&"terrain"));
        assert!(names.contains(&"scoring"));
    }

    #[test]
    fn summary_comparison_reports_frame_count_delay_and_region_metrics() {
        let reference = single_color_summary(2, 10, [0, 0, 0, 0xFF]);
        let candidate = single_color_summary(1, 15, [0xFF, 0, 0, 0xFF]);

        let comparison = compare_summaries(reference, candidate).expect("comparison");

        assert_eq!(comparison.frame_count_delta, -1);
        assert_eq!(comparison.total_delay_delta_cs, 5);
        assert!(comparison.full_rms > 0.0);
        assert!(
            comparison
                .regions
                .iter()
                .any(|region| region.name == "numeric_glyphs" && region.rms > 0.0)
        );
    }

    fn single_color_summary(
        frame_count: usize,
        total_delay_cs: u64,
        color: [u8; RGBA_CHANNELS],
    ) -> GifSummary {
        GifSummary {
            width: 768,
            height: 576,
            frame_count,
            total_delay_cs,
            sample_width: 96,
            sample_height: 72,
            frames: (0..frame_count)
                .map(|_| GifFrameSample {
                    pixels: vec![color; 96 * 72],
                })
                .collect(),
        }
    }
}
