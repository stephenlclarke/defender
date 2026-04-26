//! Self-contained RGBA rendering for the clean-slate arcade core.

use std::io::Cursor;

use anyhow::Context;
use png::{ColorType, Decoder, Transformations};

use crate::{
    assets::ARCADE_LOGO_PAGE_PNG,
    machine::{GamePhase, MachineSnapshot, VISIBLE_HEIGHT, VISIBLE_WIDTH},
    terminal::TerminalGeometry,
};

pub const WILLIAMS_VIDEO_PAIR_STRIDE: usize = 256;
pub const DEFENDER_VISIBLE_X_START: u16 = 12;
pub const DEFENDER_VISIBLE_Y_START: u16 = 7;
pub const DEFENDER_VISIBLE_X_END: u16 = DEFENDER_VISIBLE_X_START + VISIBLE_WIDTH - 1;
pub const DEFENDER_VISIBLE_Y_END: u16 = DEFENDER_VISIBLE_Y_START + VISIBLE_HEIGHT - 1;
pub const WILLIAMS_PALETTE_SIZE: usize = 256;
pub const WILLIAMS_RED_GREEN_RESISTORS: [f64; 3] = [1200.0, 560.0, 330.0];
pub const WILLIAMS_BLUE_RESISTORS: [f64; 2] = [560.0, 330.0];

const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const GRID: [u8; 4] = [18, 44, 70, 255];
const GREEN: [u8; 4] = [88, 255, 88, 255];
const AMBER: [u8; 4] = [255, 184, 48, 255];
const BLUE: [u8; 4] = [80, 160, 255, 255];
const RED: [u8; 4] = [255, 72, 72, 255];
const WHITE: [u8; 4] = [244, 244, 244, 255];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl RenderedImage {
    pub fn new_blank(width: u32, height: u32, color: [u8; 4]) -> Self {
        let mut image = Self {
            width,
            height,
            pixels: vec![0; width as usize * height as usize * 4],
        };
        image.clear(color);
        image
    }

    pub fn resize(&mut self, width: u32, height: u32, color: [u8; 4]) {
        self.width = width;
        self.height = height;
        self.pixels.resize(width as usize * height as usize * 4, 0);
        self.clear(color);
    }

    pub fn clear(&mut self, color: [u8; 4]) {
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let offset = ((y as u32 * self.width + x as u32) * 4) as usize;
        self.pixels[offset..offset + 4].copy_from_slice(&color);
    }
}

pub struct Renderer {
    image_width: u32,
    image_height: u32,
    target: RenderedImage,
    logo: RenderedImage,
}

impl Renderer {
    pub fn new(geometry: TerminalGeometry) -> Self {
        let (width, height) = raster_size(geometry);
        Self::with_size(width, height)
    }

    pub fn with_size(image_width: u32, image_height: u32) -> Self {
        Self {
            image_width,
            image_height,
            target: RenderedImage::new_blank(image_width, image_height, BACKGROUND),
            logo: decode_png_image(ARCADE_LOGO_PAGE_PNG).expect("embedded logo page should decode"),
        }
    }

    pub fn resize(&mut self, geometry: TerminalGeometry) {
        let (width, height) = raster_size(geometry);
        self.image_width = width;
        self.image_height = height;
        self.target.resize(width, height, BACKGROUND);
    }

    pub fn render_cabinet_frame(&mut self, native_frame: &RenderedImage) -> &RenderedImage {
        self.target.clear(BACKGROUND);
        let rect = fit_image_rect(
            self.image_width,
            self.image_height,
            native_frame.width,
            native_frame.height,
        );
        draw_scaled_image(
            &mut self.target,
            native_frame,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
        );
        &self.target
    }

    pub fn render(&mut self, snapshot: MachineSnapshot) -> &RenderedImage {
        self.render_scaffold(snapshot)
    }

    pub fn render_scaffold(&mut self, snapshot: MachineSnapshot) -> &RenderedImage {
        self.target.clear(BACKGROUND);
        self.draw_reference_frame();
        self.draw_logo();
        self.draw_status(snapshot);
        &self.target
    }

    fn draw_reference_frame(&mut self) {
        let left = 32;
        let top = 34;
        let width = self.image_width as i32 - 64;
        let height = self.image_height as i32 - 68;
        self.draw_rect_outline(left, top, width, height, GREEN);
        self.draw_rect_outline(left + 10, top + 34, width - 20, 52, BLUE);
        for index in 0..=8 {
            let x = left + (width * index) / 8;
            self.draw_vertical_line(x, top, top + height, GRID);
        }
        for index in 0..=6 {
            let y = top + (height * index) / 6;
            self.draw_horizontal_line(left, left + width, y, GRID);
        }
    }

    fn draw_logo(&mut self) {
        let logo_height = (self.image_height / 3).clamp(80, 220);
        let scale = logo_height as f32 / self.logo.height as f32;
        let logo_width = (self.logo.width as f32 * scale).round() as i32;
        let x = self.image_width as i32 / 2 - logo_width / 2;
        let y = 48;
        draw_scaled_image(
            &mut self.target,
            &self.logo,
            x,
            y,
            logo_width,
            logo_height as i32,
        );
    }

    fn draw_status(&mut self, snapshot: MachineSnapshot) {
        let baseline = self.image_height as i32 - 72;
        let phase_color = match snapshot.phase {
            GamePhase::Attract => BLUE,
            GamePhase::Playing => GREEN,
            GamePhase::GameOver => RED,
            GamePhase::HighScoreEntry => AMBER,
        };
        self.draw_horizontal_line(48, self.image_width as i32 - 48, baseline - 22, phase_color);

        let player_x = 72 + ((snapshot.player.x.raw() >> 16).rem_euclid(180) * 3);
        let player_y = baseline - 12 + ((snapshot.player.y.raw() >> 16).rem_euclid(18));
        self.draw_ship_marker(player_x, player_y, phase_color);

        let scanner_width = self.image_width as i32 - 144;
        let scanner_left = 72;
        let scanner_y = baseline + 20;
        self.draw_rect_outline(scanner_left, scanner_y, scanner_width, 20, AMBER);
        let blip_x = scanner_left
            + ((snapshot.frame as i32 + snapshot.rng.seed as i32) % scanner_width.max(1));
        self.fill_rect(blip_x - 2, scanner_y + 7, 5, 5, WHITE);
    }

    fn draw_ship_marker(&mut self, x: i32, y: i32, color: [u8; 4]) {
        self.fill_rect(x - 8, y - 2, 16, 5, color);
        self.fill_rect(x + 4, y - 7, 5, 15, color);
    }

    fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: [u8; 4]) {
        for yy in y..y + height {
            for xx in x..x + width {
                self.target.put_pixel(xx, yy, color);
            }
        }
    }

    fn draw_rect_outline(&mut self, x: i32, y: i32, width: i32, height: i32, color: [u8; 4]) {
        self.draw_horizontal_line(x, x + width, y, color);
        self.draw_horizontal_line(x, x + width, y + height, color);
        self.draw_vertical_line(x, y, y + height, color);
        self.draw_vertical_line(x + width, y, y + height, color);
    }

    fn draw_horizontal_line(&mut self, x0: i32, x1: i32, y: i32, color: [u8; 4]) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.target.put_pixel(x, y, color);
        }
    }

    fn draw_vertical_line(&mut self, x: i32, y0: i32, y1: i32, color: [u8; 4]) {
        for y in y0.min(y1)..=y0.max(y1) {
            self.target.put_pixel(x, y, color);
        }
    }
}

fn raster_size(geometry: TerminalGeometry) -> (u32, u32) {
    let width = u32::from(geometry.pixel_width).clamp(640, 1_280);
    let height = u32::from(geometry.pixel_height).clamp(480, 960);
    if geometry.pixel_width == 0 || geometry.pixel_height == 0 {
        (960, 720)
    } else {
        (width, height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImageRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn fit_image_rect(
    target_width: u32,
    target_height: u32,
    source_width: u32,
    source_height: u32,
) -> ImageRect {
    if target_width == 0 || target_height == 0 || source_width == 0 || source_height == 0 {
        return ImageRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
    }

    let scale_by_width = u64::from(target_width) * u64::from(source_height)
        <= u64::from(target_height) * u64::from(source_width);
    let (width, height) = if scale_by_width {
        let height = u64::from(target_width) * u64::from(source_height) / u64::from(source_width);
        (target_width, height as u32)
    } else {
        let width = u64::from(target_height) * u64::from(source_width) / u64::from(source_height);
        (width as u32, target_height)
    };

    ImageRect {
        x: ((target_width - width) / 2) as i32,
        y: ((target_height - height) / 2) as i32,
        width: width as i32,
        height: height as i32,
    }
}

fn draw_scaled_image(
    target: &mut RenderedImage,
    source: &RenderedImage,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    if width <= 0 || height <= 0 {
        return;
    }

    for dst_y in 0..height {
        let src_y = (dst_y as u32 * source.height / height as u32).min(source.height - 1);
        for dst_x in 0..width {
            let src_x = (dst_x as u32 * source.width / width as u32).min(source.width - 1);
            let source_offset = ((src_y * source.width + src_x) * 4) as usize;
            let color = [
                source.pixels[source_offset],
                source.pixels[source_offset + 1],
                source.pixels[source_offset + 2],
                source.pixels[source_offset + 3],
            ];
            if color[3] > 0 {
                target.put_pixel(x + dst_x, y + dst_y, color);
            }
        }
    }
}

fn decode_png_image(bytes: &[u8]) -> anyhow::Result<RenderedImage> {
    let cursor = Cursor::new(bytes);
    let mut decoder = Decoder::new(cursor);
    decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    let mut reader = decoder.read_info().context("reading embedded png header")?;
    let out_size = reader
        .output_buffer_size()
        .expect("expanded PNG should report an output size");
    let mut buffer = vec![0; out_size];
    let info = reader
        .next_frame(&mut buffer)
        .context("decoding embedded png frame")?;
    let pixels = &buffer[..info.buffer_size()];

    let mut rgba = Vec::with_capacity(info.width as usize * info.height as usize * 4);
    match info.color_type {
        ColorType::Rgba => rgba.extend_from_slice(pixels),
        ColorType::Rgb => {
            for chunk in pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
        }
        ColorType::GrayscaleAlpha => {
            for chunk in pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
            }
        }
        ColorType::Grayscale => {
            for value in pixels {
                rgba.extend_from_slice(&[*value, *value, *value, 255]);
            }
        }
        ColorType::Indexed => unreachable!("indexed pngs are expanded before decoding"),
    }

    Ok(RenderedImage {
        width: info.width,
        height: info.height,
        pixels: rgba,
    })
}

pub fn native_visible_size() -> (u16, u16) {
    (VISIBLE_WIDTH, VISIBLE_HEIGHT)
}

/// Returns the byte offset for a Williams bitmap pixel in absolute screen
/// coordinates. MAME documents the screen format as two 4-bit pixels per byte,
/// with successive pixel-pair columns separated by 256 bytes.
/// Source: <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams_v.cpp>.
pub fn williams_screen_byte_offset(screen_x: u16, screen_y: u16) -> usize {
    usize::from(screen_y) + (usize::from(screen_x / 2) * WILLIAMS_VIDEO_PAIR_STRIDE)
}

pub fn defender_visible_screen_coordinate(visible_x: u16, visible_y: u16) -> Option<(u16, u16)> {
    if visible_x < VISIBLE_WIDTH && visible_y < VISIBLE_HEIGHT {
        Some((
            DEFENDER_VISIBLE_X_START + visible_x,
            DEFENDER_VISIBLE_Y_START + visible_y,
        ))
    } else {
        None
    }
}

pub fn defender_visible_byte_offset(visible_x: u16, visible_y: u16) -> Option<usize> {
    defender_visible_screen_coordinate(visible_x, visible_y)
        .map(|(screen_x, screen_y)| williams_screen_byte_offset(screen_x, screen_y))
}

pub fn williams_pixel_nibble(video_ram: &[u8], screen_x: u16, screen_y: u16) -> Option<u8> {
    let byte = *video_ram.get(williams_screen_byte_offset(screen_x, screen_y))?;
    if screen_x & 1 == 0 {
        Some(byte >> 4)
    } else {
        Some(byte & 0x0F)
    }
}

pub fn defender_visible_pixel_nibble(
    video_ram: &[u8],
    visible_x: u16,
    visible_y: u16,
) -> Option<u8> {
    let (screen_x, screen_y) = defender_visible_screen_coordinate(visible_x, visible_y)?;
    williams_pixel_nibble(video_ram, screen_x, screen_y)
}

pub fn defender_visible_palette_index(
    video_ram: &[u8],
    palette_ram: &[u8; 16],
    visible_x: u16,
    visible_y: u16,
) -> Option<u8> {
    let nibble = defender_visible_pixel_nibble(video_ram, visible_x, visible_y)?;
    Some(palette_ram[usize::from(nibble)])
}

pub fn render_defender_visible_palette_indices(
    video_ram: &[u8],
    palette_ram: &[u8; 16],
) -> Option<Vec<u8>> {
    let mut pixels = Vec::with_capacity(usize::from(VISIBLE_WIDTH) * usize::from(VISIBLE_HEIGHT));
    for y in 0..VISIBLE_HEIGHT {
        for x in 0..VISIBLE_WIDTH {
            pixels.push(defender_visible_palette_index(
                video_ram,
                palette_ram,
                x,
                y,
            )?);
        }
    }
    Some(pixels)
}

/// Converts a Williams 8-bit palette value into RGBA using the same resistor
/// weights MAME uses for the first-generation Williams boards.
/// Source: <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams_v.cpp>.
/// Source: <https://github.com/mamedev/mame/blob/master/src/emu/video/resnet.cpp>.
pub fn williams_palette_rgba(palette_value: u8) -> [u8; 4] {
    let (red_green_weights, blue_weights) = williams_palette_weights();
    [
        combine_palette_weights(
            red_green_weights,
            [
                palette_value & 0x01 != 0,
                palette_value & 0x02 != 0,
                palette_value & 0x04 != 0,
            ],
        ),
        combine_palette_weights(
            red_green_weights,
            [
                palette_value & 0x08 != 0,
                palette_value & 0x10 != 0,
                palette_value & 0x20 != 0,
            ],
        ),
        combine_palette_weights(
            [blue_weights[0], blue_weights[1], 0.0],
            [palette_value & 0x40 != 0, palette_value & 0x80 != 0, false],
        ),
        255,
    ]
}

pub fn williams_palette_lookup() -> [[u8; 4]; WILLIAMS_PALETTE_SIZE] {
    let mut palette = [[0, 0, 0, 255]; WILLIAMS_PALETTE_SIZE];
    for index in u8::MIN..=u8::MAX {
        palette[usize::from(index)] = williams_palette_rgba(index);
    }
    palette
}

pub fn render_defender_visible_rgba(
    video_ram: &[u8],
    palette_ram: &[u8; 16],
) -> Option<RenderedImage> {
    let palette_lookup = williams_palette_lookup();
    let mut pixels =
        Vec::with_capacity(usize::from(VISIBLE_WIDTH) * usize::from(VISIBLE_HEIGHT) * 4);

    for palette_index in render_defender_visible_palette_indices(video_ram, palette_ram)? {
        pixels.extend_from_slice(&palette_lookup[usize::from(palette_index)]);
    }

    Some(RenderedImage {
        width: u32::from(VISIBLE_WIDTH),
        height: u32::from(VISIBLE_HEIGHT),
        pixels,
    })
}

fn williams_palette_weights() -> ([f64; 3], [f64; 2]) {
    let red_green_raw = raw_resistor_weights(WILLIAMS_RED_GREEN_RESISTORS);
    let blue_raw = raw_resistor_weights(WILLIAMS_BLUE_RESISTORS);
    let max_output = sum(red_green_raw).max(sum(blue_raw));
    let scale = 255.0 / max_output;

    (
        red_green_raw.map(|weight| weight * scale),
        blue_raw.map(|weight| weight * scale),
    )
}

fn raw_resistor_weights<const N: usize>(resistances: [f64; N]) -> [f64; N] {
    let mut weights = [0.0; N];
    for (driven, weight) in weights.iter_mut().enumerate() {
        let mut ground_conductance = 1.0e-12;
        let mut vcc_conductance = 1.0e-12;
        for (index, resistance) in resistances.iter().copied().enumerate() {
            if index == driven {
                vcc_conductance += 1.0 / resistance;
            } else {
                ground_conductance += 1.0 / resistance;
            }
        }

        let ground_resistance = 1.0 / ground_conductance;
        let vcc_resistance = 1.0 / vcc_conductance;
        *weight =
            (255.0 * ground_resistance / (vcc_resistance + ground_resistance)).clamp(0.0, 255.0);
    }
    weights
}

fn sum<const N: usize>(values: [f64; N]) -> f64 {
    values.into_iter().sum()
}

fn combine_palette_weights(weights: [f64; 3], bits: [bool; 3]) -> u8 {
    let value: f64 = weights
        .into_iter()
        .zip(bits)
        .map(|(weight, bit)| if bit { weight } else { 0.0 })
        .sum();
    (value + 0.5) as u8
}

#[cfg(test)]
mod tests {
    use super::{
        DEFENDER_VISIBLE_X_END, DEFENDER_VISIBLE_X_START, DEFENDER_VISIBLE_Y_END,
        DEFENDER_VISIBLE_Y_START, RenderedImage, Renderer, decode_png_image,
        defender_visible_byte_offset, defender_visible_palette_index,
        defender_visible_pixel_nibble, fit_image_rect, native_visible_size, raster_size,
        render_defender_visible_palette_indices, render_defender_visible_rgba,
        williams_palette_lookup, williams_palette_rgba, williams_screen_byte_offset,
    };
    use crate::{
        board::{MAIN_CPU_RAM_SIZE, PALETTE_RAM_SIZE},
        machine::{ArcadeMachine, VISIBLE_HEIGHT, VISIBLE_WIDTH},
        terminal::TerminalGeometry,
    };

    #[test]
    fn blank_image_initializes_every_pixel() {
        let image = RenderedImage::new_blank(2, 2, [1, 2, 3, 4]);
        assert_eq!(image.pixels, [1, 2, 3, 4].repeat(4));
    }

    #[test]
    fn resize_reinitializes_pixels() {
        let mut image = RenderedImage::new_blank(1, 1, [0, 0, 0, 255]);
        image.resize(2, 1, [5, 6, 7, 8]);
        assert_eq!(image.width, 2);
        assert_eq!(image.pixels, vec![5, 6, 7, 8, 5, 6, 7, 8]);
    }

    #[test]
    fn embedded_logo_decodes() {
        let logo =
            decode_png_image(crate::assets::ARCADE_LOGO_PAGE_PNG).expect("logo should decode");
        assert!(logo.width > 0);
        assert!(logo.height > 0);
        assert!(logo.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
    }

    #[test]
    fn raster_size_uses_terminal_pixels_when_available() {
        assert_eq!(
            raster_size(TerminalGeometry {
                cols: 80,
                rows: 24,
                pixel_width: 800,
                pixel_height: 600,
            }),
            (800, 600)
        );
        assert_eq!(
            raster_size(TerminalGeometry {
                cols: 80,
                rows: 24,
                pixel_width: 0,
                pixel_height: 0,
            }),
            (960, 720)
        );
    }

    #[test]
    fn renderer_outputs_non_blank_frame() {
        let machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(640, 480);
        let image = renderer.render(machine.snapshot());

        assert!(
            image
                .pixels
                .chunks_exact(4)
                .any(|pixel| pixel != [0, 0, 0, 255])
        );
    }

    #[test]
    fn cabinet_frame_fit_preserves_native_aspect_ratio() {
        assert_eq!(
            fit_image_rect(584, 480, 292, 240),
            super::ImageRect {
                x: 0,
                y: 0,
                width: 584,
                height: 480,
            }
        );
        assert_eq!(
            fit_image_rect(640, 480, 292, 240),
            super::ImageRect {
                x: 28,
                y: 0,
                width: 584,
                height: 480,
            }
        );
        assert_eq!(
            fit_image_rect(292, 300, 292, 240),
            super::ImageRect {
                x: 0,
                y: 30,
                width: 292,
                height: 240,
            }
        );
    }

    #[test]
    fn renderer_scales_native_cabinet_frames_with_letterboxing() {
        let native = RenderedImage {
            width: 2,
            height: 2,
            pixels: vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
        };
        let mut renderer = Renderer::with_size(6, 4);
        let image = renderer.render_cabinet_frame(&native);

        assert_eq!(image.width, 6);
        assert_eq!(image.height, 4);
        assert_eq!(rgba_at(image, 0, 0), [0, 0, 0, 255]);
        assert_eq!(rgba_at(image, 1, 0), [255, 0, 0, 255]);
        assert_eq!(rgba_at(image, 2, 0), [255, 0, 0, 255]);
        assert_eq!(rgba_at(image, 3, 0), [0, 255, 0, 255]);
        assert_eq!(rgba_at(image, 4, 0), [0, 255, 0, 255]);
        assert_eq!(rgba_at(image, 5, 0), [0, 0, 0, 255]);
        assert_eq!(rgba_at(image, 1, 2), [0, 0, 255, 255]);
        assert_eq!(rgba_at(image, 4, 3), [255, 255, 255, 255]);
    }

    fn rgba_at(image: &RenderedImage, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y * image.width + x) * 4) as usize;
        [
            image.pixels[offset],
            image.pixels[offset + 1],
            image.pixels[offset + 2],
            image.pixels[offset + 3],
        ]
    }

    #[test]
    fn native_visible_size_matches_mame_metadata() {
        assert_eq!(native_visible_size(), (292, 240));
    }

    #[test]
    fn williams_screen_format_offsets_match_mame_pair_stride() {
        assert_eq!(DEFENDER_VISIBLE_X_START, 12);
        assert_eq!(DEFENDER_VISIBLE_Y_START, 7);
        assert_eq!(DEFENDER_VISIBLE_X_END, 303);
        assert_eq!(DEFENDER_VISIBLE_Y_END, 246);
        assert_eq!(williams_screen_byte_offset(0, 0), 0);
        assert_eq!(williams_screen_byte_offset(1, 0), 0);
        assert_eq!(williams_screen_byte_offset(2, 0), 256);
        assert_eq!(williams_screen_byte_offset(12, 7), 1543);
        assert_eq!(
            defender_visible_byte_offset(0, 0),
            Some(williams_screen_byte_offset(12, 7))
        );
        assert_eq!(
            defender_visible_byte_offset(VISIBLE_WIDTH - 1, VISIBLE_HEIGHT - 1),
            Some(williams_screen_byte_offset(303, 246))
        );
        assert_eq!(defender_visible_byte_offset(VISIBLE_WIDTH, 0), None);
        assert_eq!(defender_visible_byte_offset(0, VISIBLE_HEIGHT), None);
    }

    #[test]
    fn defender_visible_pixel_nibbles_decode_high_then_low() {
        let mut video_ram = vec![0; MAIN_CPU_RAM_SIZE];
        video_ram[defender_visible_byte_offset(0, 0).expect("top-left offset")] = 0xAB;
        video_ram[defender_visible_byte_offset(2, 0).expect("next pair offset")] = 0xCD;
        video_ram[defender_visible_byte_offset(VISIBLE_WIDTH - 1, VISIBLE_HEIGHT - 1)
            .expect("bottom-right offset")] = 0xEF;

        assert_eq!(defender_visible_pixel_nibble(&video_ram, 0, 0), Some(0x0A));
        assert_eq!(defender_visible_pixel_nibble(&video_ram, 1, 0), Some(0x0B));
        assert_eq!(defender_visible_pixel_nibble(&video_ram, 2, 0), Some(0x0C));
        assert_eq!(defender_visible_pixel_nibble(&video_ram, 3, 0), Some(0x0D));
        assert_eq!(
            defender_visible_pixel_nibble(&video_ram, VISIBLE_WIDTH - 1, VISIBLE_HEIGHT - 1),
            Some(0x0F)
        );
    }

    #[test]
    fn defender_visible_palette_indices_use_palette_ram_entries() {
        let mut video_ram = vec![0; MAIN_CPU_RAM_SIZE];
        let mut palette_ram = [0; PALETTE_RAM_SIZE];
        palette_ram[0x0A] = 0x5A;
        palette_ram[0x0B] = 0xB5;
        video_ram[defender_visible_byte_offset(0, 0).expect("top-left offset")] = 0xAB;

        assert_eq!(
            defender_visible_palette_index(&video_ram, &palette_ram, 0, 0),
            Some(0x5A)
        );
        assert_eq!(
            defender_visible_palette_index(&video_ram, &palette_ram, 1, 0),
            Some(0xB5)
        );

        let pixels = render_defender_visible_palette_indices(&video_ram, &palette_ram)
            .expect("visible frame should render from full video RAM");
        assert_eq!(
            pixels.len(),
            usize::from(VISIBLE_WIDTH) * usize::from(VISIBLE_HEIGHT)
        );
        assert_eq!(pixels[0], 0x5A);
        assert_eq!(pixels[1], 0xB5);
    }

    #[test]
    fn williams_palette_rgba_matches_mame_resistor_weights() {
        assert_eq!(williams_palette_rgba(0x00), [0, 0, 0, 255]);
        assert_eq!(williams_palette_rgba(0x01), [38, 0, 0, 255]);
        assert_eq!(williams_palette_rgba(0x02), [81, 0, 0, 255]);
        assert_eq!(williams_palette_rgba(0x04), [137, 0, 0, 255]);
        assert_eq!(williams_palette_rgba(0x07), [255, 0, 0, 255]);
        assert_eq!(williams_palette_rgba(0x38), [0, 255, 0, 255]);
        assert_eq!(williams_palette_rgba(0x40), [0, 0, 95, 255]);
        assert_eq!(williams_palette_rgba(0x80), [0, 0, 160, 255]);
        assert_eq!(williams_palette_rgba(0xC0), [0, 0, 255, 255]);
        assert_eq!(williams_palette_rgba(0xFF), [255, 255, 255, 255]);
        assert_eq!(williams_palette_rgba(0xD6), [217, 81, 255, 255]);
        assert_eq!(williams_palette_rgba(0x29), [38, 174, 0, 255]);

        let lookup = williams_palette_lookup();
        assert_eq!(lookup[0xD6], [217, 81, 255, 255]);
    }

    #[test]
    fn defender_visible_rgba_uses_palette_resistor_conversion() {
        let mut video_ram = vec![0; MAIN_CPU_RAM_SIZE];
        let mut palette_ram = [0; PALETTE_RAM_SIZE];
        palette_ram[0x0A] = 0xD6;
        palette_ram[0x0B] = 0x29;
        video_ram[defender_visible_byte_offset(0, 0).expect("top-left offset")] = 0xAB;

        let image = render_defender_visible_rgba(&video_ram, &palette_ram)
            .expect("visible RGBA frame should render from full video RAM");
        assert_eq!(image.width, u32::from(VISIBLE_WIDTH));
        assert_eq!(image.height, u32::from(VISIBLE_HEIGHT));
        assert_eq!(&image.pixels[0..4], &[217, 81, 255, 255]);
        assert_eq!(&image.pixels[4..8], &[38, 174, 0, 255]);
    }
}
