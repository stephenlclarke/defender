//! Runtime-facing WGPU live launch facade.
//!
//! This module is the single clean quarantine point for the temporary live
//! presenter bridge while the platform event loop remains parked outside the
//! active source tree.

use std::path::Path;

use crate::audio::LiveAudioMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiveInputProfile {
    Planetoid,
    Cabinet,
    Test,
}

impl From<LiveInputProfile> for crate::input::InputProfile {
    fn from(profile: LiveInputProfile) -> Self {
        match profile {
            LiveInputProfile::Planetoid => Self::Planetoid,
            LiveInputProfile::Cabinet => Self::Cabinet,
            LiveInputProfile::Test => Self::Test,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LiveSmokeReport {
    pub(crate) window_created: bool,
    pub(crate) rendered_frames: u32,
    pub(crate) first_frame_size: Option<(u32, u32)>,
    pub(crate) distinct_frame_signatures: usize,
    pub(crate) saw_non_blank_frame: bool,
    pub(crate) saw_attract: bool,
    pub(crate) saw_credit: bool,
    pub(crate) saw_playing: bool,
    pub(crate) attract_visual_frames: u32,
    pub(crate) credit_visual_frames: u32,
    pub(crate) playing_visual_frames: u32,
    pub(crate) attract_distinct_frame_signatures: usize,
    pub(crate) credit_distinct_frame_signatures: usize,
    pub(crate) playing_distinct_frame_signatures: usize,
    pub(crate) injected_inputs: Vec<String>,
    pub(crate) clean_exit: bool,
}

impl LiveSmokeReport {
    pub(crate) fn to_text(&self) -> String {
        let frame_size = self
            .first_frame_size
            .map(|(width, height)| format!("{width}x{height}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        format!(
            "wgpu live smoke passed\n  window_created: {}\n  rendered_frames: {}\n  first_frame_size: {}\n  distinct_frame_signatures: {}\n  saw_non_blank_frame: {}\n  saw_attract: {} (visual_frames: {}, visual_signatures: {})\n  saw_credit: {} (visual_frames: {}, visual_signatures: {})\n  saw_playing: {} (visual_frames: {}, visual_signatures: {})\n  injected_inputs: {}\n  clean_exit: {}\n",
            self.window_created,
            self.rendered_frames,
            frame_size,
            self.distinct_frame_signatures,
            self.saw_non_blank_frame,
            self.saw_attract,
            self.attract_visual_frames,
            self.attract_distinct_frame_signatures,
            self.saw_credit,
            self.credit_visual_frames,
            self.credit_distinct_frame_signatures,
            self.saw_playing,
            self.playing_visual_frames,
            self.playing_distinct_frame_signatures,
            self.injected_inputs.join(","),
            self.clean_exit
        )
    }
}

impl From<crate::wgpu_presenter::WgpuSmokeReport> for LiveSmokeReport {
    fn from(report: crate::wgpu_presenter::WgpuSmokeReport) -> Self {
        Self {
            window_created: report.window_created,
            rendered_frames: report.rendered_frames,
            first_frame_size: report.first_frame_size,
            distinct_frame_signatures: report.distinct_frame_signatures,
            saw_non_blank_frame: report.saw_non_blank_frame,
            saw_attract: report.saw_attract,
            saw_credit: report.saw_credit,
            saw_playing: report.saw_playing,
            attract_visual_frames: report.attract_visual_frames,
            credit_visual_frames: report.credit_visual_frames,
            playing_visual_frames: report.playing_visual_frames,
            attract_distinct_frame_signatures: report.attract_distinct_frame_signatures,
            credit_distinct_frame_signatures: report.credit_distinct_frame_signatures,
            playing_distinct_frame_signatures: report.playing_distinct_frame_signatures,
            injected_inputs: report.injected_inputs,
            clean_exit: report.clean_exit,
        }
    }
}

pub(crate) fn run(
    input_profile: LiveInputProfile,
    audio_mode: LiveAudioMode,
    cmos_path: Option<&Path>,
) -> anyhow::Result<()> {
    crate::wgpu_presenter::run_wgpu_live(input_profile.into(), audio_mode, cmos_path)
}

pub(crate) fn run_smoke(
    input_profile: LiveInputProfile,
    cmos_path: Option<&Path>,
) -> anyhow::Result<LiveSmokeReport> {
    crate::wgpu_presenter::run_wgpu_live_smoke(input_profile.into(), cmos_path)
        .map(LiveSmokeReport::from)
}

#[cfg(test)]
mod tests {
    use crate::input::InputProfile;

    use super::{LiveInputProfile, LiveSmokeReport};

    #[test]
    fn live_input_profiles_map_to_bridge_profiles() {
        assert_eq!(
            InputProfile::from(LiveInputProfile::Planetoid),
            InputProfile::Planetoid
        );
        assert_eq!(
            InputProfile::from(LiveInputProfile::Cabinet),
            InputProfile::Cabinet
        );
        assert_eq!(
            InputProfile::from(LiveInputProfile::Test),
            InputProfile::Test
        );
    }

    #[test]
    fn live_smoke_report_formats_current_cli_output() {
        let report = LiveSmokeReport {
            window_created: true,
            rendered_frames: 3,
            first_frame_size: Some((640, 480)),
            distinct_frame_signatures: 2,
            saw_non_blank_frame: true,
            saw_attract: true,
            saw_credit: true,
            saw_playing: true,
            attract_visual_frames: 1,
            credit_visual_frames: 1,
            playing_visual_frames: 1,
            attract_distinct_frame_signatures: 1,
            credit_distinct_frame_signatures: 1,
            playing_distinct_frame_signatures: 1,
            injected_inputs: vec![String::from("coin"), String::from("start_one")],
            clean_exit: true,
        };

        assert_eq!(
            report.to_text(),
            concat!(
                "wgpu live smoke passed\n",
                "  window_created: true\n",
                "  rendered_frames: 3\n",
                "  first_frame_size: 640x480\n",
                "  distinct_frame_signatures: 2\n",
                "  saw_non_blank_frame: true\n",
                "  saw_attract: true (visual_frames: 1, visual_signatures: 1)\n",
                "  saw_credit: true (visual_frames: 1, visual_signatures: 1)\n",
                "  saw_playing: true (visual_frames: 1, visual_signatures: 1)\n",
                "  injected_inputs: coin,start_one\n",
                "  clean_exit: true\n",
            )
        );
    }

    #[test]
    fn live_smoke_report_adapts_bridge_report() {
        let report = crate::wgpu_presenter::WgpuSmokeReport {
            window_created: true,
            rendered_frames: 7,
            first_frame_size: Some((320, 240)),
            distinct_frame_signatures: 4,
            saw_non_blank_frame: true,
            saw_attract: true,
            saw_credit: true,
            saw_playing: true,
            attract_visual_frames: 2,
            credit_visual_frames: 3,
            playing_visual_frames: 4,
            attract_distinct_frame_signatures: 1,
            credit_distinct_frame_signatures: 2,
            playing_distinct_frame_signatures: 3,
            injected_inputs: vec![String::from("fire")],
            clean_exit: true,
        };

        assert_eq!(
            LiveSmokeReport::from(report),
            LiveSmokeReport {
                window_created: true,
                rendered_frames: 7,
                first_frame_size: Some((320, 240)),
                distinct_frame_signatures: 4,
                saw_non_blank_frame: true,
                saw_attract: true,
                saw_credit: true,
                saw_playing: true,
                attract_visual_frames: 2,
                credit_visual_frames: 3,
                playing_visual_frames: 4,
                attract_distinct_frame_signatures: 1,
                credit_distinct_frame_signatures: 2,
                playing_distinct_frame_signatures: 3,
                injected_inputs: vec![String::from("fire")],
                clean_exit: true,
            }
        );
    }
}
