//! High-level README media generation facade.

use crate::{
    input::CabinetInput,
    machine::{ArcadeMachine, FRAME_RATE_MILLIHZ as MACHINE_FRAME_RATE_MILLIHZ},
    video::Renderer,
};

pub const FRAME_RATE_MILLIHZ: u32 = MACHINE_FRAME_RATE_MILLIHZ;

pub struct ReadmeMediaFrameSource {
    machine: ArcadeMachine,
    renderer: Renderer,
}

impl ReadmeMediaFrameSource {
    pub fn new(output_width: u32, output_height: u32) -> Self {
        Self {
            machine: ArcadeMachine::new(),
            renderer: Renderer::with_size(output_width, output_height),
        }
    }

    pub fn step(&mut self) {
        self.machine.step(CabinetInput::NONE);
    }

    pub fn render_frame(&mut self) -> Result<ReadmeMediaFrame, ReadmeMediaError> {
        self.machine
            .red_label_copy_color_mapping_to_palette_ram()
            .map_err(ReadmeMediaError::PaletteCopy)?;
        let native_frame = self
            .machine
            .red_label_visible_rgba_image()
            .ok_or(ReadmeMediaError::FrameUnavailable)?;
        let rendered = self.renderer.render_cabinet_frame(&native_frame);

        Ok(ReadmeMediaFrame {
            width: rendered.width,
            height: rendered.height,
            pixels: rendered.pixels.clone(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadmeMediaFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadmeMediaError {
    FrameUnavailable,
    PaletteCopy(String),
}

impl std::fmt::Display for ReadmeMediaError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FrameUnavailable => write!(formatter, "README media frame is unavailable"),
            Self::PaletteCopy(message) => write!(
                formatter,
                "copying the README media palette before rendering failed: {message}"
            ),
        }
    }
}

impl std::error::Error for ReadmeMediaError {}

#[cfg(test)]
mod tests {
    use super::{FRAME_RATE_MILLIHZ, ReadmeMediaFrameSource};

    #[test]
    fn frame_rate_matches_arcade_refresh_contract() {
        assert_eq!(FRAME_RATE_MILLIHZ, 60_100);
    }

    #[test]
    fn source_renders_scaled_rgba_frames() {
        let mut source = ReadmeMediaFrameSource::new(320, 240);

        source.step();
        let frame = source.render_frame().expect("README media frame");

        assert_eq!(frame.width, 320);
        assert_eq!(frame.height, 240);
        assert_eq!(frame.pixels.len(), 320 * 240 * 4);
    }
}
