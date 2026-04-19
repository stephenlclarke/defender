//! Decodes gameplay object pictures from the red-label `defb6.src` tables.

use crate::{
    object_rom_data::{CRTAB, RomPictureData, TCTAB},
    video::RenderedImage,
};

const RED: [u8; 4] = [255, 80, 80, 255];
const GREEN: [u8; 4] = [0, 190, 0, 255];
const YELLOW: [u8; 4] = [255, 188, 0, 255];
const BLUE: [u8; 4] = [40, 56, 220, 255];
const GRAY: [u8; 4] = [170, 170, 186, 255];
const BROWN: [u8; 4] = [176, 112, 48, 255];
const PURPLE: [u8; 4] = [182, 48, 255, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

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
    const CYCLE: [[u8; 4]; 4] = [PURPLE, YELLOW, RED, WHITE];
    let color = CYCLE[phase % CYCLE.len()];
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
    const CYCLE: [[u8; 4]; 4] = [PURPLE, RED, YELLOW, WHITE];
    PaletteOverrides::new(
        WHITE,
        WHITE,
        CYCLE[phase % CYCLE.len()],
        CYCLE[(phase + 1) % CYCLE.len()],
        CYCLE[(phase + 2) % CYCLE.len()],
        CYCLE[(phase + 3) % CYCLE.len()],
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
    crtab_color(CRTAB[index as usize])
}

fn crtab_color(value: u8) -> [u8; 4] {
    match value {
        0x07 => RED,
        0x28 => GREEN,
        0x2F => YELLOW,
        0x81 => BLUE,
        0xA4 => GRAY,
        0x15 => BROWN,
        0xC7 => PURPLE,
        0xFF => WHITE,
        _ => TRANSPARENT,
    }
}

#[cfg(test)]
mod tests {
    use super::{render_picture, score_500_palette, ship_palette, tie_palette};
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
}
