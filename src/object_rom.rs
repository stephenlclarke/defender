//! Decodes gameplay object pictures from the red-label `defb6.src` tables.

use crate::{
    object_rom_data::{CRTAB, RomPictureData, TCTAB},
    video::RenderedImage,
};

const RED: [u8; 4] = [255, 80, 80, 255];
const YELLOW: [u8; 4] = [255, 188, 0, 255];
const BLUE: [u8; 4] = [40, 56, 220, 255];
const GRAY: [u8; 4] = [170, 170, 186, 255];
const PURPLE: [u8; 4] = [182, 48, 255, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
const ROM_COLTAB: [u8; 37] = [
    0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x37, 0x2F, 0x27, 0x1F, 0x17, 0x47, 0x47, 0x87,
    0x87, 0xC7, 0xC7, 0xC6, 0xC5, 0xCC, 0xCB, 0xCA, 0xDA, 0xE8, 0xF8, 0xF9, 0xFA, 0xFB, 0xFD, 0xFF,
    0xBF, 0x3F, 0x3E, 0x3C, 0x00,
];

#[derive(Clone, Copy, Debug)]
pub struct PaletteOverrides {
    pub one: [u8; 4],
    pub a: [u8; 4],
    pub c: [u8; 4],
    pub d: [u8; 4],
    pub e: [u8; 4],
    pub f: [u8; 4],
}

impl PaletteOverrides {
    pub const fn new(
        one: [u8; 4],
        a: [u8; 4],
        c: [u8; 4],
        d: [u8; 4],
        e: [u8; 4],
        f: [u8; 4],
    ) -> Self {
        Self { one, a, c, d, e, f }
    }
}

pub fn render_picture(data: &RomPictureData, overrides: PaletteOverrides) -> RenderedImage {
    let width = u32::from(data.bytes_per_row) * 2;
    let height = u32::from(data.rows);
    let mut pixels = vec![0; width as usize * height as usize * 4];

    for row in 0..usize::from(data.rows) {
        for byte_index in 0..usize::from(data.bytes_per_row) {
            let source = data.bytes[row * usize::from(data.bytes_per_row) + byte_index];
            let left = palette_color(source >> 4, overrides);
            let right = palette_color(source & 0x0F, overrides);
            let left_index = (row * width as usize + byte_index * 2) * 4;
            pixels[left_index..left_index + 4].copy_from_slice(&left);
            pixels[left_index + 4..left_index + 8].copy_from_slice(&right);
        }
    }

    RenderedImage {
        width,
        height,
        pixels,
    }
}

pub fn ship_palette() -> PaletteOverrides {
    PaletteOverrides::new(
        crtab_color_index(9),
        crtab_color_index(9),
        PURPLE,
        PURPLE,
        GRAY,
        WHITE,
    )
}

pub fn player_shot_palette() -> PaletteOverrides {
    PaletteOverrides::new(WHITE, WHITE, WHITE, WHITE, WHITE, [255, 234, 128, 255])
}

pub fn bomb_palette(phase: usize) -> PaletteOverrides {
    let color = coltab_color(phase);
    PaletteOverrides::new(WHITE, color, WHITE, WHITE, WHITE, WHITE)
}

pub fn human_palette() -> PaletteOverrides {
    PaletteOverrides::new(WHITE, WHITE, WHITE, WHITE, WHITE, WHITE)
}

pub fn lander_palette() -> PaletteOverrides {
    PaletteOverrides::new(WHITE, WHITE, WHITE, WHITE, WHITE, WHITE)
}

pub fn bomber_palette() -> PaletteOverrides {
    PaletteOverrides::new(WHITE, WHITE, WHITE, WHITE, WHITE, WHITE)
}

pub fn swarmer_palette() -> PaletteOverrides {
    PaletteOverrides::new(WHITE, WHITE, WHITE, WHITE, WHITE, WHITE)
}

pub fn tie_palette(phase: usize) -> PaletteOverrides {
    let offset = (phase % 3) * 3;
    let d = crtab_color(TCTAB[offset]);
    let e = crtab_color(TCTAB[offset + 1]);
    let f = crtab_color(TCTAB[offset + 2]);
    PaletteOverrides::new(WHITE, WHITE, WHITE, d, e, f)
}

pub fn cycler_palette(phase: usize) -> PaletteOverrides {
    PaletteOverrides::new(
        WHITE,
        WHITE,
        coltab_color(phase),
        coltab_color(phase + 1),
        coltab_color(phase + 2),
        coltab_color(phase + 3),
    )
}

pub fn score_250_palette(phase: usize) -> PaletteOverrides {
    const CYCLE: [[u8; 4]; 3] = [BLUE, YELLOW, WHITE];
    PaletteOverrides::new(
        CYCLE[phase % CYCLE.len()],
        WHITE,
        WHITE,
        WHITE,
        WHITE,
        WHITE,
    )
}

pub fn score_500_palette(phase: usize) -> PaletteOverrides {
    const CYCLE: [[u8; 4]; 3] = [RED, BLUE, YELLOW];
    PaletteOverrides::new(
        WHITE,
        WHITE,
        WHITE,
        CYCLE[phase % CYCLE.len()],
        CYCLE[(phase + 1) % CYCLE.len()],
        CYCLE[(phase + 2) % CYCLE.len()],
    )
}

fn palette_color(index: u8, overrides: PaletteOverrides) -> [u8; 4] {
    match index {
        0x0 => TRANSPARENT,
        0x1 => overrides.one,
        0x2..=0x9 => crtab_color_index(index),
        0xA => overrides.a,
        0xB => GRAY,
        0xC => overrides.c,
        0xD => overrides.d,
        0xE => overrides.e,
        0xF => overrides.f,
        _ => unreachable!(),
    }
}

fn crtab_color_index(index: u8) -> [u8; 4] {
    pseudo_color_rgba(CRTAB[index as usize])
}

pub fn pseudo_color_rgba(value: u8) -> [u8; 4] {
    if value == 0 {
        return TRANSPARENT;
    }

    // Williams packs pseudo-color RAM as 3 red bits, 3 green bits, and 2 blue
    // bits. Decode those channels directly instead of keeping a hand-written
    // lookup for only the subset we happened to use first.
    let red = scale_channel(value & 0x07, 7);
    let green = scale_channel((value >> 3) & 0x07, 7);
    let blue = scale_channel((value >> 6) & 0x03, 3);
    [red, green, blue, 255]
}

fn crtab_color(value: u8) -> [u8; 4] {
    pseudo_color_rgba(value)
}

pub fn coltab_color(phase: usize) -> [u8; 4] {
    let color = ROM_COLTAB[phase % (ROM_COLTAB.len() - 1)];
    pseudo_color_rgba(color)
}

fn scale_channel(value: u8, max: u8) -> u8 {
    ((u16::from(value) * 255) / u16::from(max.max(1))) as u8
}

#[cfg(test)]
mod tests {
    use super::{
        coltab_color, pseudo_color_rgba, render_picture, score_500_palette, ship_palette,
        tie_palette,
    };
    use crate::object_rom_data::{C5P1, PLAPIC, TIEP1};

    #[test]
    fn ship_picture_renders_to_expected_rom_dimensions() {
        let image = render_picture(&PLAPIC, ship_palette());

        assert_eq!(image.width, 12);
        assert_eq!(image.height, 8);
        assert!(image.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
    }

    #[test]
    fn tie_palette_cycles_rom_special_colors() {
        let first = render_picture(&TIEP1, tie_palette(0));
        let next = render_picture(&TIEP1, tie_palette(1));

        assert_ne!(first.pixels, next.pixels);
    }

    #[test]
    fn score_palette_cycles_rom_special_colors() {
        let first = render_picture(&C5P1, score_500_palette(0));
        let next = render_picture(&C5P1, score_500_palette(1));

        assert_ne!(first.pixels, next.pixels);
    }

    #[test]
    fn pseudo_color_decoder_matches_known_primary_examples() {
        let red = pseudo_color_rgba(0x07);
        let green = pseudo_color_rgba(0x28);
        let blue = pseudo_color_rgba(0x81);

        assert!(red[0] > red[1] && red[0] > red[2]);
        assert!(green[1] > green[0] && green[1] > green[2]);
        assert!(blue[2] > blue[0] && blue[2] > blue[1]);
    }

    #[test]
    fn coltab_cycle_uses_rom_color_bytes() {
        let first = coltab_color(0);
        let later = coltab_color(8);

        assert_ne!(first, later);
        assert!(first[1] > 0 || first[2] > 0);
        assert!(later[0] > 0 || later[1] > 0 || later[2] > 0);
    }
}
