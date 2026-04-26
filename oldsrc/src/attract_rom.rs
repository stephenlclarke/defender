//! Decodes ROM-derived attract-page data from the red-label source tables.

use std::sync::OnceLock;

use crate::video::RenderedImage;

mod data {
    include!("attract_rom_data.rs");
}

pub const WILLIAMS_TRACE_POINT_COUNT: usize = 661;
pub const DEFENDER_LOGO_CHUNK_COUNT: usize = 15;

const DEFENDER_NATIVE_CHUNK_WIDTH: usize = 8;
const DEFENDER_NATIVE_HEIGHT: usize = 24;
const DEFENDER_FACE: [u8; 4] = [112, 255, 52, 255];
const DEFENDER_SHADOW: [u8; 4] = [255, 48, 48, 255];

pub struct AttractRom {
    williams_points: Vec<(u16, u16)>,
    williams_point_prefixes: Vec<usize>,
    defender_chunks: Vec<RenderedImage>,
}

pub fn attract_rom() -> &'static AttractRom {
    static ATTRACT_ROM: OnceLock<AttractRom> = OnceLock::new();
    ATTRACT_ROM.get_or_init(AttractRom::decode)
}

impl AttractRom {
    pub fn williams_points(&self) -> &[(u16, u16)] {
        &self.williams_points
    }

    pub fn defender_chunks(&self) -> &[RenderedImage] {
        &self.defender_chunks
    }

    pub fn williams_point_prefixes(&self) -> &[usize] {
        &self.williams_point_prefixes
    }

    fn decode() -> Self {
        Self {
            williams_points: decode_williams_points(),
            williams_point_prefixes: decode_williams_point_prefixes(),
            defender_chunks: decode_defender_chunks(),
        }
    }
}

fn decode_williams_points() -> Vec<(u16, u16)> {
    // This ports the `LOGO` cursor-walk in `amode1.src`, preserving the ROM
    // path data from `LGOTAB` rather than replaying a hand-authored spline.
    let mut points = Vec::new();
    let mut cursor_x = 0u16;
    let mut cursor_y = 0u16;
    let mut index = 0;

    while index < data::WILLIAMS_TRACE_DATA.len() {
        let value = data::WILLIAMS_TRACE_DATA[index];
        index += 1;

        if value <= 0xAA {
            let mut accumulator = value;
            loop {
                if accumulator & 0x80 != 0 {
                    cursor_x = cursor_x.saturating_sub(1);
                }
                accumulator <<= 1;

                if accumulator & 0x80 != 0 {
                    cursor_x = cursor_x.saturating_add(1);
                }
                accumulator <<= 1;

                if accumulator & 0x80 != 0 {
                    cursor_y = cursor_y.saturating_sub(1);
                }
                accumulator <<= 1;

                if accumulator & 0x80 != 0 {
                    cursor_y = cursor_y.saturating_add(1);
                }
                accumulator <<= 1;

                points.push((cursor_x, cursor_y));

                if accumulator == 0 {
                    break;
                }
            }
            continue;
        }

        let instruction = (!value).wrapping_sub(1);
        if instruction != 0 {
            break;
        }

        let next_x = data::WILLIAMS_TRACE_DATA[index];
        let next_y = data::WILLIAMS_TRACE_DATA[index + 1];
        cursor_x = u16::from(next_x);
        cursor_y = u16::from(next_y);
        index += 2;
    }

    points
}

fn decode_williams_point_prefixes() -> Vec<usize> {
    let mut prefixes = Vec::with_capacity(data::WILLIAMS_TRACE_DATA.len());
    let mut points = 0usize;
    let mut index = 0usize;

    while index < data::WILLIAMS_TRACE_DATA.len() {
        let value = data::WILLIAMS_TRACE_DATA[index];
        index += 1;

        if value <= 0xAA {
            let mut accumulator = value;
            loop {
                accumulator <<= 4;
                points += 1;
                if accumulator == 0 {
                    break;
                }
            }
            prefixes.push(points);
            continue;
        }

        let instruction = (!value).wrapping_sub(1);
        if instruction != 0 {
            break;
        }
        index += 2;
        prefixes.push(points);
    }

    prefixes
}

fn decode_defender_chunks() -> Vec<RenderedImage> {
    let full_logo = decode_defender_logo_native();
    let mut chunks = Vec::with_capacity(DEFENDER_LOGO_CHUNK_COUNT);

    for chunk_index in 0..DEFENDER_LOGO_CHUNK_COUNT {
        let start_x = chunk_index * DEFENDER_NATIVE_CHUNK_WIDTH;
        let mut native_pixels = vec![0; DEFENDER_NATIVE_CHUNK_WIDTH * DEFENDER_NATIVE_HEIGHT * 4];

        for y in 0..DEFENDER_NATIVE_HEIGHT {
            for x in 0..DEFENDER_NATIVE_CHUNK_WIDTH {
                let source = ((y * full_logo.width as usize) + start_x + x) * 4;
                let destination = ((y * DEFENDER_NATIVE_CHUNK_WIDTH) + x) * 4;
                native_pixels[destination..destination + 4]
                    .copy_from_slice(&full_logo.pixels[source..source + 4]);
            }
        }

        chunks.push(scale_chunk_to_display(RenderedImage {
            width: DEFENDER_NATIVE_CHUNK_WIDTH as u32,
            height: DEFENDER_NATIVE_HEIGHT as u32,
            pixels: native_pixels,
        }));
    }

    chunks
}

fn scale_chunk_to_display(chunk: RenderedImage) -> RenderedImage {
    chunk
}

fn decode_defender_logo_native() -> RenderedImage {
    // This follows the `DEFNNN` nibble-expansion logic from `amode1.src`,
    // turning `DEFDAT` into the native 120x24 `DEFENDER` bitmap used by the
    // attract page and hall-of-fame logo.
    let mut buffer = [0u8; 60 * 24];
    let color_table = [None, Some(0x22u8), Some(0xCCu8), Some(0x00u8)];
    let mut cursor = 0usize;
    let mut odd_nibble = false;
    let mut run_length = 0u8;

    for byte in data::DEFENDER_LOGO_DATA {
        for nibble in [byte >> 4, byte & 0x0F] {
            if nibble & 0x0C == 0 {
                run_length = nibble.wrapping_add(run_length).wrapping_mul(4);
                continue;
            }

            run_length = (nibble & 0x03).wrapping_add(run_length);
            let color = color_table[((nibble & 0x0C) >> 2) as usize].unwrap_or(0x00);
            if cursor >= buffer.len() {
                cursor = cursor + 1 - buffer.len();
            }

            if odd_nibble {
                if cursor < buffer.len() {
                    buffer[cursor] = (buffer[cursor] & 0xF0) | (color & 0x0F);
                }
                cursor += 24;
                run_length = run_length.wrapping_sub(1);
                if run_length & 0x80 != 0 {
                    odd_nibble = false;
                    run_length = 0;
                    continue;
                }
            } else {
                odd_nibble = true;
            }

            loop {
                if cursor < buffer.len() {
                    buffer[cursor] = color;
                }
                run_length = run_length.wrapping_sub(1);
                if run_length & 0x80 != 0 {
                    break;
                }
                cursor += 24;
                run_length = run_length.wrapping_sub(1);
                if run_length & 0x80 == 0 {
                    continue;
                }
                odd_nibble = false;
                break;
            }

            run_length = 0;
        }
    }

    let mut pixels = vec![0; 120 * 24 * 4];
    for x_byte in 0..60usize {
        for y in 0..24usize {
            let packed = buffer[x_byte * 24 + y];
            let left = (packed >> 4) & 0x0F;
            let right = packed & 0x0F;

            if left != 0 {
                let destination = ((y * 120) + x_byte * 2) * 4;
                pixels[destination..destination + 4].copy_from_slice(if left == 0x02 {
                    &DEFENDER_SHADOW
                } else {
                    &DEFENDER_FACE
                });
            }

            if right != 0 {
                let destination = ((y * 120) + x_byte * 2 + 1) * 4;
                pixels[destination..destination + 4].copy_from_slice(if right == 0x02 {
                    &DEFENDER_SHADOW
                } else {
                    &DEFENDER_FACE
                });
            }
        }
    }

    RenderedImage {
        width: 120,
        height: 24,
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFENDER_LOGO_CHUNK_COUNT, WILLIAMS_TRACE_POINT_COUNT, attract_rom};

    #[test]
    fn rom_trace_points_match_the_red_label_logo_path() {
        let rom = attract_rom();

        assert_eq!(rom.williams_points().len(), WILLIAMS_TRACE_POINT_COUNT);
        assert!(!rom.williams_points().is_empty());
        assert_eq!(
            rom.williams_point_prefixes().last().copied(),
            Some(WILLIAMS_TRACE_POINT_COUNT)
        );
    }

    #[test]
    fn rom_defender_chunks_match_the_red_label_chunk_count() {
        let rom = attract_rom();

        assert_eq!(rom.defender_chunks().len(), DEFENDER_LOGO_CHUNK_COUNT);
        assert!(
            rom.defender_chunks()
                .iter()
                .all(|chunk| chunk.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0))
        );
    }
}
