//! Loads embedded ROM-derived branding art for Defender attract screens.

use std::{
    io::Cursor,
    sync::{Arc, OnceLock},
};

use anyhow::Context;
use png::{ColorType, Decoder, Transformations};

use crate::video::RenderedImage;

#[derive(Clone, Debug)]
pub struct ArcadeBranding {
    logo_page: Arc<RenderedImage>,
    defender_logo: Arc<RenderedImage>,
}

pub fn arcade_branding() -> &'static ArcadeBranding {
    static BRANDING: OnceLock<ArcadeBranding> = OnceLock::new();
    BRANDING.get_or_init(ArcadeBranding::new)
}

impl ArcadeBranding {
    fn new() -> Self {
        Self {
            logo_page: load_embedded_png(include_bytes!("../assets/arcade/logo-page.png")),
            defender_logo: load_embedded_png(include_bytes!("../assets/arcade/defender-logo.png")),
        }
    }

    pub fn logo_page(&self) -> &RenderedImage {
        self.logo_page.as_ref()
    }

    pub fn defender_logo(&self) -> &RenderedImage {
        self.defender_logo.as_ref()
    }
}

fn load_embedded_png(bytes: &'static [u8]) -> Arc<RenderedImage> {
    Arc::new(decode_png_image(bytes).expect("embedded branding asset should decode"))
}

#[cfg(test)]
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn decode_png_image(bytes: &[u8]) -> anyhow::Result<RenderedImage> {
    let cursor = Cursor::new(bytes);
    let mut decoder = Decoder::new(cursor);
    decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .context("reading embedded branding png header")?;
    let out_size = reader
        .output_buffer_size()
        .expect("expanded PNG should report an output size");
    let mut buffer = vec![0; out_size];
    let info = reader
        .next_frame(&mut buffer)
        .context("decoding embedded branding png frame")?;
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
    use super::{arcade_branding, fnv1a64};

    #[test]
    fn logo_page_decodes_with_visible_pixels() {
        let branding = arcade_branding();
        let logo_page = branding.logo_page();

        assert!(logo_page.width > 0);
        assert!(logo_page.height > 0);
        assert!(logo_page.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
    }

    #[test]
    fn defender_logo_is_wide_enough_for_hall_of_fame_title() {
        let branding = arcade_branding();
        let logo = branding.defender_logo();

        assert!(logo.width > logo.height * 4);
    }

    #[test]
    fn logo_page_matches_rom_derived_layout() {
        let branding = arcade_branding();

        assert_eq!(fnv1a64(&branding.logo_page().pixels), 0x1486_86f7_920d_c186);
    }

    #[test]
    fn defender_logo_matches_rom_derived_wordmark() {
        let branding = arcade_branding();

        assert_eq!(
            fnv1a64(&branding.defender_logo().pixels),
            0x1322_29d3_4e9c_ab3b
        );
    }
}
