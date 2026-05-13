//! Wraps the Kitty graphics protocol so rendered frames can be pushed into compatible terminals.

use std::io::{IsTerminal, Write};

use anyhow::{Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use crossterm::{cursor::MoveTo, queue};
use png::{BitDepth, ColorType, Compression, Encoder};

use crate::video::RenderedImage;

const IMAGE_ID: u32 = 1_981;
const SECONDARY_IMAGE_ID: u32 = IMAGE_ID + 1;
const PLACEMENT_ID: u32 = 1;
const CHUNK_SIZE: usize = 4_096;
const ESCAPE_BEGIN: &str = "\x1b_G";
const ESCAPE_END: &str = "\x1b\\";

pub struct KittyGraphics {
    placement_cols: u16,
    placement_rows: u16,
    visible_image_id: Option<u32>,
    next_image_id: u32,
    png_buffer: Vec<u8>,
    base64_buffer: String,
}

impl KittyGraphics {
    pub fn new(placement_cols: u16, placement_rows: u16) -> Self {
        Self {
            placement_cols,
            placement_rows,
            visible_image_id: None,
            next_image_id: IMAGE_ID,
            png_buffer: Vec::new(),
            base64_buffer: String::new(),
        }
    }

    pub fn ensure_supported() -> Result<()> {
        let term = std::env::var("TERM").unwrap_or_default();
        let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
        let force = std::env::var("DEFENDER_FORCE_KITTY").unwrap_or_default();
        let kitty_window = std::env::var_os("KITTY_WINDOW_ID").is_some();

        if force == "1" || kitty_window || is_known_kitty_graphics_terminal(&term, &term_program) {
            return Ok(());
        }

        validate_environment(&term, std::io::stdout().is_terminal())
    }

    pub fn resize(&mut self, placement_cols: u16, placement_rows: u16) {
        self.placement_cols = placement_cols;
        self.placement_rows = placement_rows;
    }

    pub fn draw_frame<W: Write>(&mut self, stdout: &mut W, image: &RenderedImage) -> Result<()> {
        encode_png_into(image, &mut self.png_buffer)?;
        self.base64_buffer.clear();
        STANDARD.encode_string(&self.png_buffer, &mut self.base64_buffer);
        let chunk_count = self.base64_buffer.len().div_ceil(CHUNK_SIZE);
        let image_id = self.next_image_id;
        let previous_image_id = self.visible_image_id;

        queue!(stdout, MoveTo(0, 0))?;

        for (index, chunk) in self.base64_buffer.as_bytes().chunks(CHUNK_SIZE).enumerate() {
            let more = if index + 1 == chunk_count { 0 } else { 1 };
            if index == 0 {
                write!(
                    stdout,
                    "{ESCAPE_BEGIN}a=T,f=100,i={image_id},p={PLACEMENT_ID},q=2,C=1,c={},r={},z=-1,m={more};",
                    self.placement_cols, self.placement_rows
                )?;
            } else {
                write!(stdout, "{ESCAPE_BEGIN}m={more};")?;
            }

            stdout.write_all(chunk)?;
            write!(stdout, "{ESCAPE_END}")?;
        }

        if let Some(previous_image_id) = previous_image_id {
            write_delete_image(stdout, previous_image_id)?;
        }
        self.visible_image_id = Some(image_id);
        self.next_image_id = alternate_image_id(image_id);

        Ok(())
    }

    pub fn clear<W: Write>(&mut self, stdout: &mut W) -> Result<()> {
        write_delete_image(stdout, IMAGE_ID)?;
        write_delete_image(stdout, SECONDARY_IMAGE_ID)?;
        self.visible_image_id = None;
        self.next_image_id = IMAGE_ID;
        Ok(())
    }
}

fn alternate_image_id(image_id: u32) -> u32 {
    if image_id == IMAGE_ID {
        SECONDARY_IMAGE_ID
    } else {
        IMAGE_ID
    }
}

fn write_delete_image<W: Write>(stdout: &mut W, image_id: u32) -> Result<()> {
    write!(stdout, "{ESCAPE_BEGIN}a=d,d=I,i={image_id},q=2{ESCAPE_END}")?;
    Ok(())
}

fn is_known_kitty_graphics_terminal(term: &str, term_program: &str) -> bool {
    term == "xterm-kitty"
        || term == "xterm-ghostty"
        || term_program == "ghostty"
        || term_program == "kitty"
        || term_program == "WarpTerminal"
}

fn validate_environment(term: &str, is_terminal: bool) -> Result<()> {
    if !is_terminal {
        bail!(
            "Kitty graphics output requires an interactive terminal on stdout. \
             Run inside kitty or another compatible terminal."
        );
    }

    if term.is_empty() || term == "dumb" {
        bail!(
            "TERM={term:?} does not expose the interactive terminal capabilities needed for \
             Kitty graphics. Run inside kitty or set DEFENDER_FORCE_KITTY=1 to bypass this \
             basic check."
        );
    }

    Ok(())
}

#[cfg(test)]
fn encode_png(image: &RenderedImage) -> Result<Vec<u8>> {
    let mut encoded = Vec::new();
    encode_png_into(image, &mut encoded)?;
    Ok(encoded)
}

fn encode_png_into(image: &RenderedImage, encoded: &mut Vec<u8>) -> Result<()> {
    encoded.clear();
    let mut encoder = Encoder::new(encoded, image.width, image.height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(Compression::Fast);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&image.pixels)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        sync::{Mutex, OnceLock},
    };

    use super::{
        CHUNK_SIZE, IMAGE_ID, KittyGraphics, PLACEMENT_ID, SECONDARY_IMAGE_ID, encode_png,
        is_known_kitty_graphics_terminal, validate_environment,
    };
    use crate::video::RenderedImage;

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_env_vars<T>(vars: &[(&str, Option<&str>)], f: impl FnOnce() -> T) -> T {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let previous = vars
            .iter()
            .map(|(key, _)| ((*key).to_string(), env::var_os(key)))
            .collect::<Vec<_>>();
        for (key, value) in vars {
            match value {
                Some(value) => unsafe { env::set_var(key, value) },
                None => unsafe { env::remove_var(key) },
            }
        }
        let result = f();
        for (key, value) in previous {
            match value {
                Some(value) => unsafe { env::set_var(&key, value) },
                None => unsafe { env::remove_var(&key) },
            }
        }
        result
    }

    #[test]
    fn png_encoder_writes_signature() {
        let image = RenderedImage {
            width: 2,
            height: 2,
            pixels: vec![
                0, 0, 0, 255, 255, 255, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255,
            ],
        };

        let png = encode_png(&image).expect("png encoding should succeed");
        assert!(png.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
    }

    #[test]
    fn chunk_size_matches_protocol_limit() {
        assert_eq!(CHUNK_SIZE, 4_096);
    }

    #[test]
    fn draw_frame_keeps_previous_image_visible_while_transmitting_replacement() {
        let mut graphics = KittyGraphics::new(10, 20);
        let image = RenderedImage::new_blank(2, 2, [12, 34, 56, 255]);
        let mut first = Vec::new();
        graphics
            .draw_frame(&mut first, &image)
            .expect("first Kitty frame");
        let first = String::from_utf8(first).expect("Kitty escapes are UTF-8");

        assert!(first.contains(&format!("a=T,f=100,i={IMAGE_ID},p={PLACEMENT_ID}")));
        assert!(!first.contains("a=d,d=I"));
        assert!(!first.contains("\x1b[2J"));

        let mut second = Vec::new();
        graphics
            .draw_frame(&mut second, &image)
            .expect("second Kitty frame");
        let second = String::from_utf8(second).expect("Kitty escapes are UTF-8");
        let transmit = second
            .find(&format!(
                "a=T,f=100,i={SECONDARY_IMAGE_ID},p={PLACEMENT_ID}"
            ))
            .expect("second frame should transmit the secondary image first");
        let delete = second
            .find(&format!("a=d,d=I,i={IMAGE_ID}"))
            .expect("second frame should delete the previous image");

        assert!(
            transmit < delete,
            "previous image must be deleted only after the replacement is transmitted"
        );
        assert!(!second.contains("\x1b[2J"));
    }

    #[test]
    fn draw_frame_double_buffers_images_between_frames() {
        let mut graphics = KittyGraphics::new(10, 20);
        let image = RenderedImage::new_blank(2, 2, [90, 12, 34, 255]);
        let mut output = Vec::new();

        graphics
            .draw_frame(&mut output, &image)
            .expect("first frame");
        graphics
            .draw_frame(&mut output, &image)
            .expect("second frame");
        graphics
            .draw_frame(&mut output, &image)
            .expect("third frame");

        let output = String::from_utf8(output).expect("Kitty escapes are UTF-8");
        assert!(output.contains(&format!("a=T,f=100,i={IMAGE_ID},p={PLACEMENT_ID}")));
        assert!(output.contains(&format!(
            "a=T,f=100,i={SECONDARY_IMAGE_ID},p={PLACEMENT_ID}"
        )));
        assert!(output.contains(&format!("a=d,d=I,i={IMAGE_ID}")));
        assert!(output.contains(&format!("a=d,d=I,i={SECONDARY_IMAGE_ID}")));
    }

    #[test]
    fn clear_deletes_both_frame_buffers_and_resets_draw_order() {
        let mut graphics = KittyGraphics::new(10, 20);
        let image = RenderedImage::new_blank(2, 2, [1, 2, 3, 255]);
        let mut output = Vec::new();

        graphics
            .draw_frame(&mut output, &image)
            .expect("first frame");
        graphics.clear(&mut output).expect("clear frame buffers");
        let clear_output = String::from_utf8(output).expect("Kitty escapes are UTF-8");
        assert!(clear_output.contains(&format!("a=d,d=I,i={IMAGE_ID}")));
        assert!(clear_output.contains(&format!("a=d,d=I,i={SECONDARY_IMAGE_ID}")));

        let mut next = Vec::new();
        graphics
            .draw_frame(&mut next, &image)
            .expect("frame after clear");
        let next = String::from_utf8(next).expect("Kitty escapes are UTF-8");
        assert!(next.contains(&format!("a=T,f=100,i={IMAGE_ID},p={PLACEMENT_ID}")));
        assert!(!next.contains("a=d,d=I"));
    }

    #[test]
    fn environment_check_allows_interactive_terminals() {
        validate_environment("xterm-256color", true).expect("interactive terminals should pass");
    }

    #[test]
    fn environment_check_rejects_dumb_terminals() {
        assert!(validate_environment("dumb", true).is_err());
    }

    #[test]
    fn known_terminals_include_kitty() {
        assert!(is_known_kitty_graphics_terminal("xterm-kitty", ""));
        assert!(is_known_kitty_graphics_terminal("xterm-ghostty", "ghostty"));
        assert!(is_known_kitty_graphics_terminal("", "WarpTerminal"));
    }

    #[test]
    fn force_flag_bypasses_terminal_detection() {
        with_env_vars(
            &[
                ("DEFENDER_FORCE_KITTY", Some("1")),
                ("TERM", Some("dumb")),
                ("TERM_PROGRAM", None),
            ],
            || {
                assert!(KittyGraphics::ensure_supported().is_ok());
            },
        );
    }

    #[test]
    fn kitty_window_id_bypasses_terminal_detection() {
        with_env_vars(
            &[
                ("DEFENDER_FORCE_KITTY", None),
                ("KITTY_WINDOW_ID", Some("123")),
                ("TERM", Some("dumb")),
                ("TERM_PROGRAM", None),
            ],
            || {
                assert!(KittyGraphics::ensure_supported().is_ok());
            },
        );
    }

    #[test]
    fn resize_updates_placement_dimensions() {
        let mut graphics = KittyGraphics::new(10, 20);
        graphics.resize(30, 40);

        assert_eq!(graphics.placement_cols, 30);
        assert_eq!(graphics.placement_rows, 40);
    }
}
