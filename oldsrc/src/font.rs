//! Loads the red-label Defender text glyphs used by the Kitty renderer.
//!
//! The glyph shapes and per-character widths come from the `CHRTBL` and
//! `CHARACTERS` tables in `mess0.src` from the red-label source release
//! referenced in the README (`https://github.com/mwenge/defender`). The
//! bundled `assets/arcade/font-sheet.png` is a rasterized copy of those
//! ROM-backed glyphs so compile and runtime stay self-contained.

use std::{
    io::Cursor,
    sync::{Arc, OnceLock},
};

use anyhow::Context;
use png::{ColorType, Decoder, Transformations};

use crate::video::RenderedImage;

const FONT_SHEET_BYTES: &[u8] = include_bytes!("../assets/arcade/font-sheet.png");
const GLYPH_CELL_WIDTH: u32 = 8;
const GLYPH_CELL_HEIGHT: u32 = 8;
const FONT_COLUMNS: u32 = 8;

const FONT_LAYOUT: &[(char, u32)] = &[
    (' ', 2),
    ('!', 2),
    (',', 2),
    ('.', 2),
    ('0', 6),
    ('1', 6),
    ('2', 6),
    ('3', 6),
    ('4', 6),
    ('5', 6),
    ('6', 6),
    ('7', 6),
    ('8', 6),
    ('9', 6),
    (':', 2),
    ('?', 6),
    ('A', 6),
    ('B', 6),
    ('C', 6),
    ('D', 6),
    ('E', 6),
    ('F', 6),
    ('G', 6),
    ('H', 6),
    ('I', 4),
    ('J', 6),
    ('K', 6),
    ('L', 6),
    ('M', 8),
    ('N', 6),
    ('O', 6),
    ('P', 6),
    ('Q', 6),
    ('R', 6),
    ('S', 6),
    ('T', 6),
    ('U', 6),
    ('V', 6),
    ('W', 8),
    ('X', 6),
    ('Y', 6),
    ('Z', 6),
    // These two are synthetic extensions for repo-specific control/help
    // overlays. They are not part of the original ROM text table.
    ('-', 6),
    ('/', 6),
];

#[derive(Clone, Debug)]
pub struct Glyph {
    image: Arc<RenderedImage>,
    advance: u32,
}

#[derive(Clone, Debug)]
pub struct ArcadeFont {
    glyphs: Vec<Glyph>,
}

pub fn arcade_font() -> &'static ArcadeFont {
    static FONT: OnceLock<ArcadeFont> = OnceLock::new();
    FONT.get_or_init(ArcadeFont::load)
}

impl ArcadeFont {
    fn load() -> Self {
        let sheet = decode_png_image(FONT_SHEET_BYTES).expect("embedded font sheet should decode");
        let glyphs = FONT_LAYOUT
            .iter()
            .enumerate()
            .map(|(index, (_, width_pixels))| {
                let column = index as u32 % FONT_COLUMNS;
                let row = index as u32 / FONT_COLUMNS;
                let image = crop_image(
                    &sheet,
                    column * GLYPH_CELL_WIDTH,
                    row * GLYPH_CELL_HEIGHT,
                    GLYPH_CELL_WIDTH,
                    GLYPH_CELL_HEIGHT,
                );
                Glyph {
                    image: Arc::new(image),
                    advance: width_pixels + 1,
                }
            })
            .collect();
        Self { glyphs }
    }

    pub fn glyph_for_char(&self, ch: char) -> &Glyph {
        let ch = ch.to_ascii_uppercase();
        let index = FONT_LAYOUT
            .iter()
            .position(|(glyph, _)| *glyph == ch)
            .unwrap_or_else(|| {
                FONT_LAYOUT
                    .iter()
                    .position(|(glyph, _)| *glyph == '?')
                    .expect("font layout should always include a question mark glyph")
            });
        &self.glyphs[index]
    }

    pub fn text_width(&self, text: &str, scale: i32) -> i32 {
        let scale = scale.max(1) as u32;
        let total_advance = text
            .chars()
            .map(|ch| self.glyph_for_char(ch).advance)
            .sum::<u32>();
        if total_advance == 0 {
            0
        } else {
            ((total_advance - 1) * scale) as i32
        }
    }
}

impl Glyph {
    pub fn image(&self) -> &RenderedImage {
        self.image.as_ref()
    }

    pub fn advance(&self) -> i32 {
        self.advance as i32
    }
}

fn crop_image(image: &RenderedImage, x: u32, y: u32, width: u32, height: u32) -> RenderedImage {
    let mut pixels = vec![0; (width * height * 4) as usize];
    for row in 0..height {
        let src_start = (((y + row) * image.width + x) * 4) as usize;
        let src_end = src_start + (width * 4) as usize;
        let dst_start = (row * width * 4) as usize;
        pixels[dst_start..dst_start + (width * 4) as usize]
            .copy_from_slice(&image.pixels[src_start..src_end]);
    }
    RenderedImage {
        width,
        height,
        pixels,
    }
}

fn decode_png_image(bytes: &[u8]) -> anyhow::Result<RenderedImage> {
    let cursor = Cursor::new(bytes);
    let mut decoder = Decoder::new(cursor);
    decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .context("reading embedded font header")?;
    let out_size = reader
        .output_buffer_size()
        .expect("expanded PNG should report an output size");
    let mut buffer = vec![0; out_size];
    let info = reader
        .next_frame(&mut buffer)
        .context("decoding embedded font frame")?;
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

#[cfg(test)]
mod tests {
    use super::arcade_font;

    #[test]
    fn rom_backed_font_sheet_decodes_visible_pixels() {
        let font = arcade_font();
        let glyph = font.glyph_for_char('A').image();

        assert!(glyph.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
    }

    #[test]
    fn narrow_i_and_wide_m_keep_their_original_widths() {
        let font = arcade_font();

        assert!(font.glyph_for_char('I').advance() < font.glyph_for_char('M').advance());
    }

    #[test]
    fn lowercase_maps_to_the_uppercase_rom_glyphs() {
        let font = arcade_font();

        assert_eq!(
            font.glyph_for_char('a').image().pixels,
            font.glyph_for_char('A').image().pixels
        );
    }
}
