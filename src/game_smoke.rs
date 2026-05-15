//! Clean gameplay smoke runner.
//!
//! This command path exercises the domain game and native draw planning without
//! entering the windowed live presenter.

use std::collections::BTreeSet;

use anyhow::bail;

use crate::{
    Game, GameFrame, GameInput, GamePhase, NativeSceneRenderer, SceneDrawPlan, SurfaceSize,
};

const SMOKE_FRAMES: u32 = 24;
const REQUIRED_INPUTS: [&str; 9] = [
    "coin",
    "start_one",
    "altitude_up",
    "thrust",
    "fire",
    "reverse",
    "smart_bomb",
    "hyperspace",
    "altitude_down",
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct GameSmokeReport {
    pub(crate) frames: u32,
    pub(crate) first_frame_size: Option<(u32, u32)>,
    pub(crate) distinct_scene_signatures: usize,
    pub(crate) saw_attract: bool,
    pub(crate) saw_credit: bool,
    pub(crate) saw_playing: bool,
    pub(crate) attract_frames: u32,
    pub(crate) credited_frames: u32,
    pub(crate) playing_frames: u32,
    pub(crate) sprite_frames: u32,
    pub(crate) sprite_instances: usize,
    pub(crate) sprite_draw_commands: usize,
    pub(crate) raster_frames: u32,
    pub(crate) missing_sprite_regions: usize,
    pub(crate) injected_inputs: Vec<String>,
    pub(crate) clean_exit: bool,
}

impl GameSmokeReport {
    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        if self.frames == 0 {
            bail!("clean game smoke did not advance any frames");
        }
        if self.first_frame_size.is_none() {
            bail!("clean game smoke did not record a frame size");
        }
        if self.distinct_scene_signatures < 3 {
            bail!("clean game smoke did not produce dynamic scene signatures");
        }
        if !self.saw_attract || self.attract_frames == 0 {
            bail!("clean game smoke did not observe attract frames");
        }
        if !self.saw_credit || self.credited_frames == 0 {
            bail!("clean game smoke did not observe a credited attract frame");
        }
        if !self.saw_playing || self.playing_frames == 0 {
            bail!("clean game smoke did not observe playing frames");
        }
        if self.sprite_frames != self.frames {
            bail!("clean game smoke did not produce sprites for every frame");
        }
        if self.sprite_instances == 0 || self.sprite_draw_commands == 0 {
            bail!("clean game smoke did not produce sprite draw plans");
        }
        if self.raster_frames != 0 {
            bail!("clean game smoke unexpectedly produced raster payloads");
        }
        if self.missing_sprite_regions != 0 {
            bail!("clean game smoke had missing sprite atlas regions");
        }
        for required in REQUIRED_INPUTS {
            if !self.injected_inputs.iter().any(|input| input == required) {
                bail!("clean game smoke did not inject {required}");
            }
        }
        if !self.clean_exit {
            bail!("clean game smoke did not exit cleanly");
        }

        Ok(())
    }

    pub(crate) fn to_text(&self) -> String {
        let frame_size = self
            .first_frame_size
            .map(|(width, height)| format!("{width}x{height}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        format!(
            "clean game smoke passed\n  frames: {}\n  first_frame_size: {}\n  distinct_scene_signatures: {}\n  saw_attract: {} (frames: {})\n  saw_credit: {} (frames: {})\n  saw_playing: {} (frames: {})\n  sprite_frames: {}\n  sprite_instances: {}\n  sprite_draw_commands: {}\n  raster_frames: {}\n  missing_sprite_regions: {}\n  injected_inputs: {}\n  clean_exit: {}\n",
            self.frames,
            frame_size,
            self.distinct_scene_signatures,
            self.saw_attract,
            self.attract_frames,
            self.saw_credit,
            self.credited_frames,
            self.saw_playing,
            self.playing_frames,
            self.sprite_frames,
            self.sprite_instances,
            self.sprite_draw_commands,
            self.raster_frames,
            self.missing_sprite_regions,
            self.injected_inputs.join(","),
            self.clean_exit
        )
    }
}

pub(crate) fn run() -> anyhow::Result<()> {
    let report = smoke_report(SMOKE_FRAMES)?;

    print!("{}", report.to_text());
    Ok(())
}

pub(crate) fn smoke_report(frames: u32) -> anyhow::Result<GameSmokeReport> {
    if frames == 0 {
        bail!("clean game smoke frame count must be positive");
    }

    let mut game = Game::new();
    let renderer = NativeSceneRenderer::default();
    let mut report = GameSmokeReport {
        clean_exit: true,
        ..GameSmokeReport::default()
    };
    let mut signatures = BTreeSet::new();

    for frame_index in 0..frames {
        let input = smoke_input(frame_index);
        if let Some(label) = input.label {
            record_input(&mut report.injected_inputs, label);
        }

        let frame = game.step(input.value);
        let plan = renderer.prepare(&frame.scene);
        observe_frame(&mut report, &mut signatures, &frame, &plan);
    }

    report.distinct_scene_signatures = signatures.len();
    report.validate()?;
    Ok(report)
}

fn observe_frame(
    report: &mut GameSmokeReport,
    signatures: &mut BTreeSet<u64>,
    frame: &GameFrame,
    plan: &SceneDrawPlan,
) {
    report.frames = report.frames.saturating_add(1);
    report
        .first_frame_size
        .get_or_insert(surface_tuple(frame.scene.surface));
    report.saw_attract |= frame.state.phase == GamePhase::Attract;
    report.saw_credit |= frame.state.phase == GamePhase::Attract && frame.state.credits > 0;
    report.saw_playing |= frame.state.phase == GamePhase::Playing;

    if frame.state.phase == GamePhase::Attract {
        report.attract_frames = report.attract_frames.saturating_add(1);
        if frame.state.credits > 0 {
            report.credited_frames = report.credited_frames.saturating_add(1);
        }
    }
    if frame.state.phase == GamePhase::Playing {
        report.playing_frames = report.playing_frames.saturating_add(1);
    }

    let summary = frame.scene.summary();
    if summary.sprite_count > 0 {
        report.sprite_frames = report.sprite_frames.saturating_add(1);
    }
    if summary.raster_count > 0 {
        report.raster_frames = report.raster_frames.saturating_add(1);
    }
    report.sprite_instances = report
        .sprite_instances
        .saturating_add(plan.sprite_instances);
    report.sprite_draw_commands = report
        .sprite_draw_commands
        .saturating_add(plan.sprite_draw_commands.len());
    report.missing_sprite_regions = report
        .missing_sprite_regions
        .saturating_add(plan.missing_sprite_regions);
    signatures.insert(scene_signature(frame, plan));
}

fn surface_tuple(surface: SurfaceSize) -> (u32, u32) {
    (surface.width, surface.height)
}

fn scene_signature(frame: &GameFrame, plan: &SceneDrawPlan) -> u64 {
    let mut signature = 0x6A09_E667_F3BC_C909_u64;
    signature = mix_signature(signature, frame.state.frame);
    signature = mix_signature(signature, phase_code(frame.state.phase));
    signature = mix_signature(signature, u64::from(frame.state.credits));
    signature = mix_signature(signature, u64::from(frame.state.wave));
    signature = mix_signature(signature, frame.scene.summary().sprite_count as u64);
    signature = mix_signature(signature, plan.sprite_instances as u64);
    signature = mix_signature(signature, plan.sprite_draw_commands.len() as u64);
    signature = mix_signature(signature, plan.pipelines.len() as u64);
    signature
}

fn mix_signature(current: u64, value: u64) -> u64 {
    current
        ^ value
            .wrapping_add(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(current << 6)
            .wrapping_add(current >> 2)
}

fn phase_code(phase: GamePhase) -> u64 {
    match phase {
        GamePhase::Attract => 1,
        GamePhase::Playing => 2,
        GamePhase::GameOver => 3,
        GamePhase::HighScoreEntry => 4,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScriptedInput {
    value: GameInput,
    label: Option<&'static str>,
}

fn smoke_input(frame_index: u32) -> ScriptedInput {
    let (value, label) = match frame_index {
        1 => (
            GameInput {
                coin: true,
                ..GameInput::NONE
            },
            Some("coin"),
        ),
        3 => (
            GameInput {
                start_one: true,
                ..GameInput::NONE
            },
            Some("start_one"),
        ),
        5 => (
            GameInput {
                altitude_up: true,
                ..GameInput::NONE
            },
            Some("altitude_up"),
        ),
        7 => (
            GameInput {
                thrust: true,
                ..GameInput::NONE
            },
            Some("thrust"),
        ),
        9 => (
            GameInput {
                fire: true,
                ..GameInput::NONE
            },
            Some("fire"),
        ),
        11 => (
            GameInput {
                reverse: true,
                ..GameInput::NONE
            },
            Some("reverse"),
        ),
        13 => (
            GameInput {
                smart_bomb: true,
                ..GameInput::NONE
            },
            Some("smart_bomb"),
        ),
        15 => (
            GameInput {
                hyperspace: true,
                ..GameInput::NONE
            },
            Some("hyperspace"),
        ),
        17 => (
            GameInput {
                altitude_down: true,
                ..GameInput::NONE
            },
            Some("altitude_down"),
        ),
        _ => (GameInput::NONE, None),
    };

    ScriptedInput { value, label }
}

fn record_input(inputs: &mut Vec<String>, label: &str) {
    if !inputs.iter().any(|input| input == label) {
        inputs.push(String::from(label));
    }
}

#[cfg(test)]
mod tests {
    use super::{GameSmokeReport, smoke_input, smoke_report};

    #[test]
    fn smoke_report_exercises_clean_game_and_native_draw_plans() {
        let report = smoke_report(24).expect("clean game smoke report");

        assert_eq!(report.frames, 24);
        assert_eq!(report.first_frame_size, Some((292, 240)));
        assert!(report.distinct_scene_signatures >= 3);
        assert!(report.saw_attract);
        assert!(report.saw_credit);
        assert!(report.saw_playing);
        assert!(report.attract_frames > 0);
        assert!(report.credited_frames > 0);
        assert!(report.playing_frames > 0);
        assert_eq!(report.sprite_frames, report.frames);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert_eq!(report.raster_frames, 0);
        assert_eq!(report.missing_sprite_regions, 0);
        assert_eq!(
            report.injected_inputs,
            vec![
                "coin",
                "start_one",
                "altitude_up",
                "thrust",
                "fire",
                "reverse",
                "smart_bomb",
                "hyperspace",
                "altitude_down",
            ]
        );
        assert!(report.clean_exit);
    }

    #[test]
    fn smoke_report_rejects_zero_frames() {
        let error = smoke_report(0).expect_err("zero-frame smoke should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke frame count must be positive"
        );
    }

    #[test]
    fn smoke_report_validates_required_play_states() {
        let error = GameSmokeReport {
            frames: 3,
            first_frame_size: Some((292, 240)),
            distinct_scene_signatures: 3,
            saw_attract: true,
            saw_credit: true,
            attract_frames: 2,
            credited_frames: 1,
            sprite_frames: 3,
            sprite_instances: 3,
            sprite_draw_commands: 3,
            clean_exit: true,
            injected_inputs: vec![
                String::from("coin"),
                String::from("start_one"),
                String::from("altitude_up"),
                String::from("thrust"),
                String::from("fire"),
                String::from("reverse"),
                String::from("smart_bomb"),
                String::from("hyperspace"),
                String::from("altitude_down"),
            ],
            ..GameSmokeReport::default()
        }
        .validate()
        .expect_err("missing playing evidence should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not observe playing frames"
        );
    }

    #[test]
    fn smoke_report_formats_current_cli_output() {
        let report = GameSmokeReport {
            frames: 2,
            first_frame_size: Some((292, 240)),
            distinct_scene_signatures: 2,
            saw_attract: true,
            saw_credit: true,
            saw_playing: false,
            attract_frames: 2,
            credited_frames: 1,
            playing_frames: 0,
            sprite_frames: 2,
            sprite_instances: 4,
            sprite_draw_commands: 2,
            raster_frames: 0,
            missing_sprite_regions: 0,
            injected_inputs: vec![String::from("coin")],
            clean_exit: true,
        };

        assert_eq!(
            report.to_text(),
            concat!(
                "clean game smoke passed\n",
                "  frames: 2\n",
                "  first_frame_size: 292x240\n",
                "  distinct_scene_signatures: 2\n",
                "  saw_attract: true (frames: 2)\n",
                "  saw_credit: true (frames: 1)\n",
                "  saw_playing: false (frames: 0)\n",
                "  sprite_frames: 2\n",
                "  sprite_instances: 4\n",
                "  sprite_draw_commands: 2\n",
                "  raster_frames: 0\n",
                "  missing_sprite_regions: 0\n",
                "  injected_inputs: coin\n",
                "  clean_exit: true\n",
            )
        );
    }

    #[test]
    fn smoke_script_uses_release_frames_between_edge_inputs() {
        assert!(smoke_input(1).value.coin);
        assert_eq!(smoke_input(2).value, crate::GameInput::NONE);
        assert!(smoke_input(3).value.start_one);
        assert_eq!(smoke_input(4).value, crate::GameInput::NONE);
    }
}
