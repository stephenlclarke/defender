//! Fidelity trace schema for red-label equivalence tests.
//!
//! This module is infrastructure only. It records machine state without
//! interpreting arcade behavior, so future red-label routine translations can
//! be compared against MAME/source-derived golden traces. The gameplay columns
//! mirror the same red-label RAM fields sampled by the MAME trace script.

use std::fmt;

use crate::{
    input::{CabinetInput, DefenderInputPorts},
    machine::{
        ArcadeMachine, CompatibilityState, FrameOutput, GamePhase, MachineEvent, MachineSnapshot,
    },
    rom::crc32,
    sound::{SoundCommand, format_sound_command_list},
    video::RenderedImage,
};

pub fn trace_header() -> &'static str {
    crate::assets::RED_LABEL_TRACE_SCHEMA_TSV.trim_end_matches(&['\r', '\n'][..])
}

pub fn trace_scenarios() -> Result<Vec<TraceScenario>, String> {
    parse_trace_scenarios(crate::assets::RED_LABEL_TRACE_SCENARIOS_TSV)
}

pub fn parse_trace_scenarios(tsv: &str) -> Result<Vec<TraceScenario>, String> {
    let mut lines = tsv.lines();
    let Some(header) = lines.next() else {
        return Err(String::from("trace scenario TSV is empty"));
    };
    if header != "scenario\tframes\tinput_program\tdescription\tsource" {
        return Err(format!("unexpected trace scenario header: {header}"));
    }

    let mut scenarios = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 5 {
            return Err(format!(
                "trace scenario line {} has {} fields, expected 5",
                index + 2,
                fields.len()
            ));
        }

        let scenario = fields[0].trim();
        if scenario.is_empty() {
            return Err(format!("trace scenario line {} has empty name", index + 2));
        }
        if !scenario
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        {
            return Err(format!(
                "trace scenario line {} has invalid name `{scenario}`",
                index + 2
            ));
        }

        let frames = fields[1].parse::<usize>().map_err(|_| {
            format!(
                "trace scenario line {} has invalid frame count `{}`",
                index + 2,
                fields[1]
            )
        })?;
        if frames == 0 {
            return Err(format!(
                "trace scenario line {} frame count must be greater than zero",
                index + 2
            ));
        }

        let input_program = fields[2].trim();
        let expanded_frames = expand_trace_input_program(input_program)?;
        if expanded_frames.len() != frames {
            return Err(format!(
                "trace scenario `{scenario}` declares {frames} frame(s) but expands to {}",
                expanded_frames.len()
            ));
        }

        scenarios.push(TraceScenario {
            scenario: String::from(scenario),
            frames,
            input_program: String::from(input_program),
            description: String::from(fields[3]),
            source: String::from(fields[4]),
        });
    }

    if scenarios.is_empty() {
        return Err(String::from("trace scenario TSV has no scenarios"));
    }

    Ok(scenarios)
}

pub fn expand_trace_input_program(program: &str) -> Result<Vec<String>, String> {
    if program.trim().is_empty() {
        return Err(String::from("trace input program is empty"));
    }

    let mut frames = Vec::new();
    for (segment_index, segment) in program.split(';').enumerate() {
        let segment = segment.trim();
        if segment.is_empty() {
            return Err(format!(
                "trace input program segment {} is empty",
                segment_index + 1
            ));
        }

        let (frame, repeats) = parse_trace_program_segment(segment, segment_index + 1)?;
        parse_trace_input_frame(segment_index + 1, frame)?;
        frames.extend((0..repeats).map(|_| String::from(frame)));
    }

    Ok(frames)
}

pub fn expanded_trace_input_text(program: &str) -> Result<String, String> {
    let mut text = expand_trace_input_program(program)?.join(";");
    text.push('\n');
    Ok(text)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceComparison {
    pub frames: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceScenario {
    pub scenario: String,
    pub frames: usize,
    pub input_program: String,
    pub description: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceMismatch {
    pub line: usize,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

impl fmt::Display for TraceMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.expected, &self.actual) {
            (Some(expected), Some(actual)) => write!(
                formatter,
                "trace mismatch at line {}: expected `{expected}` got `{actual}`",
                self.line
            ),
            (Some(expected), None) => write!(
                formatter,
                "trace mismatch at line {}: expected `{expected}` got end of trace",
                self.line
            ),
            (None, Some(actual)) => write!(
                formatter,
                "trace mismatch at line {}: expected end of trace got `{actual}`",
                self.line
            ),
            (None, None) => write!(formatter, "trace mismatch at line {}", self.line),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceFrame {
    pub frame: u64,
    pub input_bits: u16,
    pub input_ports: DefenderInputPorts,
    pub phase: GamePhase,
    pub player_one_score: u32,
    pub player_two_score: u32,
    pub wave: u8,
    pub lives: u8,
    pub smart_bombs: u8,
    pub seed: u8,
    pub hseed: u8,
    pub lseed: u8,
    pub object_table_crc32: Option<u32>,
    pub process_table_crc32: Option<u32>,
    pub super_process_table_crc32: Option<u32>,
    pub shell_table_crc32: Option<u32>,
    pub video_crc32: Option<u32>,
    pub sound_commands: Vec<SoundCommand>,
    pub events: Vec<MachineEvent>,
}

impl TraceFrame {
    pub fn from_output(input: CabinetInput, output: &FrameOutput) -> Self {
        let mut frame = Self::from_snapshot(
            input.bits(),
            input.defender_input_ports(),
            output.snapshot,
            output.sound_commands().collect(),
            output.events().collect(),
        );
        frame.player_one_score = output.red_label_trace.player_one_score;
        frame.player_two_score = output.red_label_trace.player_two_score;
        frame.wave = output.red_label_trace.wave;
        frame.lives = output.red_label_trace.lives;
        frame.smart_bombs = output.red_label_trace.smart_bombs;
        frame.seed = output.red_label_trace.seed;
        frame.hseed = output.red_label_trace.hseed;
        frame.lseed = output.red_label_trace.lseed;
        frame.object_table_crc32 = output.object_table_crc32;
        frame.process_table_crc32 = output.process_table_crc32;
        frame.super_process_table_crc32 = output.super_process_table_crc32;
        frame.shell_table_crc32 = output.shell_table_crc32;
        frame.video_crc32 = output.video_crc32;
        frame
    }

    pub fn from_output_with_video(
        input: CabinetInput,
        output: &FrameOutput,
        video: &RenderedImage,
    ) -> Self {
        Self::from_output(input, output).with_video_frame(video)
    }

    pub fn from_snapshot(
        input_bits: u16,
        input_ports: DefenderInputPorts,
        snapshot: MachineSnapshot,
        sound_commands: Vec<SoundCommand>,
        events: Vec<MachineEvent>,
    ) -> Self {
        Self {
            frame: snapshot.frame,
            input_bits,
            input_ports,
            phase: snapshot.phase,
            player_one_score: snapshot.scores.player_one,
            player_two_score: snapshot.scores.player_two,
            wave: snapshot.wave,
            lives: snapshot.player.lives,
            smart_bombs: snapshot.player.smart_bombs,
            seed: snapshot.rng.seed,
            hseed: snapshot.rng.hseed,
            lseed: snapshot.rng.lseed,
            object_table_crc32: None,
            process_table_crc32: None,
            super_process_table_crc32: None,
            shell_table_crc32: None,
            video_crc32: None,
            sound_commands,
            events,
        }
    }

    pub fn with_object_table_bytes(mut self, object_table: &[u8]) -> Self {
        self.object_table_crc32 = Some(crc32(object_table));
        self
    }

    pub fn with_process_table_bytes(mut self, process_table: &[u8]) -> Self {
        self.process_table_crc32 = Some(crc32(process_table));
        self
    }

    pub fn with_super_process_table_bytes(mut self, super_process_table: &[u8]) -> Self {
        self.super_process_table_crc32 = Some(crc32(super_process_table));
        self
    }

    pub fn with_shell_table_bytes(mut self, shell_table: &[u8]) -> Self {
        self.shell_table_crc32 = Some(crc32(shell_table));
        self
    }

    pub fn with_video_frame(mut self, video: &RenderedImage) -> Self {
        self.video_crc32 = Some(crc32(&video.pixels));
        self
    }

    pub fn to_tsv_line(&self) -> String {
        format!(
            "{}\t0x{:04X}\t0x{:02X}\t0x{:02X}\t0x{:02X}\t{}\t{}\t{}\t{}\t{}\t{}\t0x{:02X}\t0x{:02X}\t0x{:02X}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.frame,
            self.input_bits,
            self.input_ports.in0,
            self.input_ports.in1,
            self.input_ports.in2,
            phase_label(self.phase),
            self.player_one_score,
            self.player_two_score,
            self.wave,
            self.lives,
            self.smart_bombs,
            self.seed,
            self.hseed,
            self.lseed,
            optional_crc32_label(self.object_table_crc32),
            optional_crc32_label(self.process_table_crc32),
            optional_crc32_label(self.super_process_table_crc32),
            optional_crc32_label(self.shell_table_crc32),
            optional_crc32_label(self.video_crc32),
            format_sound_command_list(&self.sound_commands),
            event_list(&self.events)
        )
    }
}

pub fn trace_output(input: CabinetInput, output: &FrameOutput) -> String {
    let frame = TraceFrame::from_output(input, output);
    format!("{}\n{}\n", trace_header(), frame.to_tsv_line())
}

pub fn trace_text_for_inputs(inputs: &[CabinetInput]) -> Result<String, String> {
    trace_text_for_inputs_with_compatibility(inputs, CompatibilityState::default())
}

fn trace_text_for_inputs_with_compatibility(
    inputs: &[CabinetInput],
    compatibility: CompatibilityState,
) -> Result<String, String> {
    if inputs.is_empty() {
        return Err(String::from("fidelity trace requires at least one frame"));
    }

    let mut machine = ArcadeMachine::new_cold_boot_trace();
    machine.set_compatibility(compatibility);
    let mut text = String::from(trace_header());
    text.push('\n');

    for input in inputs.iter().copied() {
        let output = machine.step(input);
        text.push_str(&TraceFrame::from_output(input, &output).to_tsv_line());
        text.push('\n');
    }

    Ok(text)
}

pub fn compare_trace_text(expected: &str, actual: &str) -> Result<TraceComparison, TraceMismatch> {
    let expected_lines = expected.lines().collect::<Vec<_>>();
    let actual_lines = actual.lines().collect::<Vec<_>>();
    let line_count = expected_lines.len().max(actual_lines.len());

    for index in 0..line_count {
        let expected = expected_lines.get(index).copied();
        let actual = actual_lines.get(index).copied();
        if expected != actual {
            return Err(TraceMismatch {
                line: index + 1,
                expected: expected.map(String::from),
                actual: actual.map(String::from),
            });
        }
    }

    Ok(TraceComparison {
        frames: expected_lines.len().saturating_sub(1),
    })
}

pub fn parse_trace_input_script(script: &str) -> Result<Vec<CabinetInput>, String> {
    if script.trim().is_empty() {
        return Err(String::from("trace input script is empty"));
    }

    script
        .split(';')
        .enumerate()
        .map(|(index, frame)| parse_trace_input_frame(index + 1, frame))
        .collect()
}

fn parse_trace_program_segment(
    segment: &str,
    segment_number: usize,
) -> Result<(&str, usize), String> {
    let Some((frame, repeats)) = segment.rsplit_once('*') else {
        return Ok((segment, 1));
    };
    let frame = frame.trim();
    if frame.is_empty() {
        return Err(format!(
            "trace input program segment {segment_number} has an empty frame before repeat"
        ));
    }

    let repeats = repeats.trim().parse::<usize>().map_err(|_| {
        format!("trace input program segment {segment_number} has invalid repeat count `{repeats}`")
    })?;
    if repeats == 0 {
        return Err(format!(
            "trace input program segment {segment_number} repeat count must be greater than zero"
        ));
    }

    Ok((frame, repeats))
}

fn phase_label(phase: GamePhase) -> &'static str {
    match phase {
        GamePhase::Attract => "attract",
        GamePhase::Playing => "playing",
        GamePhase::GameOver => "game_over",
        GamePhase::HighScoreEntry => "high_score_entry",
    }
}

fn event_label(event: MachineEvent) -> &'static str {
    match event {
        MachineEvent::CreditAdded => "credit_added",
        MachineEvent::GameStarted => "game_started",
        MachineEvent::DiagnosticsSelected => "diagnostics_selected",
        MachineEvent::AuditsSelected => "audits_selected",
        MachineEvent::HighScoreReset => "high_score_reset",
        MachineEvent::ReversePressed => "reverse_pressed",
        MachineEvent::FirePressed => "fire_pressed",
        MachineEvent::SmartBombPressed => "smart_bomb_pressed",
        MachineEvent::HyperspacePressed => "hyperspace_pressed",
        MachineEvent::BonusAwarded => "bonus_awarded",
        MachineEvent::HighScoreEntryStarted => "high_score_entry_started",
        MachineEvent::HighScoreInitialAccepted => "high_score_initial_accepted",
        MachineEvent::HighScoreSubmitted => "high_score_submitted",
    }
}

fn event_list(events: &[MachineEvent]) -> String {
    if events.is_empty() {
        return String::from("-");
    }

    events
        .iter()
        .copied()
        .map(event_label)
        .collect::<Vec<_>>()
        .join(",")
}

fn optional_crc32_label(crc32: Option<u32>) -> String {
    match crc32 {
        Some(crc) => format!("0x{crc:08X}"),
        None => String::from("-"),
    }
}

fn parse_trace_input_frame(frame_number: usize, frame: &str) -> Result<CabinetInput, String> {
    let frame = frame.trim();
    if frame.is_empty() {
        return Err(format!("trace input frame {frame_number} is empty"));
    }

    let mut input = CabinetInput::NONE;
    for action in frame.split(',') {
        apply_trace_input_action(frame_number, action.trim(), &mut input)?;
    }

    Ok(input)
}

fn apply_trace_input_action(
    frame_number: usize,
    action: &str,
    input: &mut CabinetInput,
) -> Result<(), String> {
    match action {
        "-" | "none" => {}
        "coin" | "coin_one" | "coin1" => input.coin = true,
        "coin_two" | "coin2" => input.coin_two = true,
        "coin_three" | "coin3" => input.coin_three = true,
        "start_one" | "start1" => input.start_one = true,
        "start_two" | "start2" => input.start_two = true,
        "altitude_up" | "up" => input.altitude_up = true,
        "altitude_down" | "down" => input.altitude_down = true,
        "reverse" => input.reverse = true,
        "thrust" => input.thrust = true,
        "fire" => input.fire = true,
        "smart_bomb" | "smartbomb" => input.smart_bomb = true,
        "hyperspace" => input.hyperspace = true,
        "auto_up_manual_down" => input.auto_up_manual_down = true,
        "service_advance" | "advance" => input.service_advance = true,
        "high_score_reset" => input.high_score_reset = true,
        "tilt" => input.tilt = true,
        "" => {
            return Err(format!(
                "trace input frame {frame_number} has an empty action"
            ));
        }
        _ => {
            return Err(format!(
                "unknown trace input action '{action}' in frame {frame_number}"
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        fidelity::{
            TraceFrame, compare_trace_text, expanded_trace_input_text, parse_trace_input_script,
            parse_trace_scenarios, trace_header, trace_output, trace_scenarios,
            trace_text_for_inputs,
        },
        input::CabinetInput,
        machine::{ArcadeMachine, CompatibilityState, GamePhase, MachineEvent},
        rom::crc32,
        sound::SoundCommand,
        video::RenderedImage,
    };

    #[test]
    fn trace_header_is_stable() {
        assert_eq!(
            trace_header(),
            "frame\tinput_bits\tinput_in0\tinput_in1\tinput_in2\tphase\tp1_score\tp2_score\twave\tlives\tsmart_bombs\tseed\thseed\tlseed\tobject_table_crc32\tprocess_table_crc32\tsuper_process_table_crc32\tshell_table_crc32\tvideo_crc32\tsound_commands\tevents"
        );
    }

    #[test]
    fn trace_scenarios_cover_phase_one_required_cases() {
        let scenarios = trace_scenarios().expect("trace scenarios parse");
        let names = scenarios
            .iter()
            .map(|scenario| scenario.scenario.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                "attract_boot",
                "start_game",
                "first_300_frames",
                "firing",
                "thrust_reverse",
                "smart_bomb",
                "hyperspace",
                "abduction",
                "death",
                "wave_advance",
                "planet_destruction",
                "high_score_entry",
            ]
        );
        assert_eq!(scenarios[0].frames, 900);
        assert_eq!(
            expanded_trace_input_text(&scenarios[0].input_program).expect("expand input program"),
            format!("{}\n", vec!["none"; 900].join(";"))
        );
    }

    #[test]
    fn trace_scenario_parser_rejects_bad_header_and_frame_drift() {
        let bad_header = "name\tframes\tinput_program\tdescription\tsource\nboot\t1\tnone\t-\t-\n";
        assert!(parse_trace_scenarios(bad_header).is_err());

        let bad_count =
            "scenario\tframes\tinput_program\tdescription\tsource\nboot\t2\tnone\t-\t-\n";
        let error = parse_trace_scenarios(bad_count).expect_err("frame count drift");
        assert!(error.contains("expands to 1"));
    }

    #[test]
    fn trace_input_program_expands_repeat_segments_and_validates_actions() {
        let text =
            expanded_trace_input_text("coin,start_one;none*2;fire,thrust*3").expect("expand");

        assert_eq!(
            text,
            "coin,start_one;none;none;fire,thrust;fire,thrust;fire,thrust\n"
        );

        let error = expanded_trace_input_text("none*0").expect_err("zero repeat");
        assert!(error.contains("greater than zero"));

        let error = expanded_trace_input_text("warp*2").expect_err("bad action");
        assert!(error.contains("unknown trace input action"));
    }

    #[test]
    #[ignore = "known unknown: unignore when object/process scheduler golden traces exist"]
    fn known_unknown_object_and_process_trace_equivalence() {
        panic!("object/process table CRCs must match red-label golden traces");
    }

    #[test]
    #[ignore = "known unknown: unignore when cabinet video golden frames exist"]
    fn known_unknown_native_video_trace_equivalence() {
        panic!("native video frame CRCs must match red-label golden traces");
    }

    #[test]
    #[ignore = "known unknown: unignore when waveform fixtures and end-to-end sound traces exist"]
    fn known_unknown_sound_trace_equivalence() {
        panic!("sound command timing and waveforms must match red-label golden traces");
    }

    #[test]
    #[ignore = "known unknown: unignore when player/world routines are translated"]
    fn known_unknown_player_world_trace_equivalence() {
        panic!("player, enemy, terrain, and collision state must match red-label golden traces");
    }

    #[test]
    #[ignore = "known unknown: unignore when cabinet session/high-score MAME golden traces exist"]
    fn known_unknown_session_and_high_score_trace_equivalence() {
        panic!(
            "coin, two-player, operator, and high-score state must match red-label golden traces"
        );
    }

    #[test]
    fn trace_frame_records_machine_output_with_red_label_ram_observed_state() {
        let mut machine = ArcadeMachine::new();
        insert_live_coin(&mut machine);
        for _ in 0..128 {
            machine.step(CabinetInput::NONE);
        }
        let input = CabinetInput {
            start_one: true,
            ..CabinetInput::NONE
        };

        let output = machine.step(input);
        let trace = TraceFrame::from_output(input, &output);

        assert_eq!(trace.frame, output.snapshot.frame);
        assert_eq!(trace.input_bits, input.bits());
        assert_eq!(trace.input_ports, input.defender_input_ports());
        assert_eq!(trace.phase, GamePhase::Playing);
        assert_eq!(
            trace.player_one_score,
            output.red_label_trace.player_one_score
        );
        assert_eq!(
            trace.player_two_score,
            output.red_label_trace.player_two_score
        );
        assert_eq!(trace.wave, output.red_label_trace.wave);
        assert_eq!(trace.lives, output.red_label_trace.lives);
        assert_eq!(trace.smart_bombs, output.red_label_trace.smart_bombs);
        assert_eq!(trace.seed, output.red_label_trace.seed);
        assert_eq!(trace.hseed, output.red_label_trace.hseed);
        assert_eq!(trace.lseed, output.red_label_trace.lseed);
        assert_eq!(trace.events, vec![MachineEvent::GameStarted]);
        assert_eq!(trace.sound_commands, Vec::new());
        assert_eq!(trace.object_table_crc32, output.object_table_crc32);
        assert_eq!(trace.process_table_crc32, output.process_table_crc32);
        assert_eq!(
            trace.super_process_table_crc32,
            output.super_process_table_crc32
        );
        assert_eq!(trace.shell_table_crc32, output.shell_table_crc32);
        assert_eq!(trace.video_crc32, output.video_crc32);
        assert_eq!(trace.video_crc32, machine.red_label_visible_video_crc32());
        assert!(trace.object_table_crc32.is_some());
        assert!(trace.process_table_crc32.is_some());
        assert!(trace.super_process_table_crc32.is_some());
        assert!(trace.shell_table_crc32.is_some());
        assert!(trace.video_crc32.is_some());

        let line = trace.to_tsv_line();
        assert!(line.contains(&format!(
            "\tplaying\t{}\t{}\t{}\t{}\t{}\t0x{:02X}\t0x{:02X}\t0x{:02X}\t",
            output.red_label_trace.player_one_score,
            output.red_label_trace.player_two_score,
            output.red_label_trace.wave,
            output.red_label_trace.lives,
            output.red_label_trace.smart_bombs,
            output.red_label_trace.seed,
            output.red_label_trace.hseed,
            output.red_label_trace.lseed
        )));
        assert!(line.contains(&format!(
            "\t0x{:08X}\t0x{:08X}\t0x{:08X}\t0x{:08X}\t0x{:08X}\t-\t",
            trace.object_table_crc32.expect("object CRC"),
            trace.process_table_crc32.expect("process CRC"),
            trace.super_process_table_crc32.expect("super-process CRC"),
            trace.shell_table_crc32.expect("shell CRC"),
            trace.video_crc32.expect("video CRC")
        )));
        assert!(line.ends_with("\tgame_started"));
    }

    #[test]
    fn trace_frame_records_source_sound_commands_from_frame_output() {
        let mut machine = ArcadeMachine::new();
        insert_live_coin(&mut machine);
        for _ in 0..128 {
            machine.step(CabinetInput::NONE);
        }
        machine.step(CabinetInput {
            start_one: true,
            ..CabinetInput::NONE
        });
        let input = CabinetInput::NONE;

        let output = machine.step(input);
        let trace = TraceFrame::from_output(input, &output);

        assert_eq!(
            trace.sound_commands,
            vec![SoundCommand::from_main_board_pia_port_b(0x35)]
        );
        assert!(trace.to_tsv_line().contains("\t0xF5\t"));
    }

    fn insert_live_coin(machine: &mut ArcadeMachine) {
        machine.step(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });
        for _ in 0..64 {
            let output = machine.step(CabinetInput::NONE);
            if output
                .events()
                .any(|event| event == MachineEvent::CreditAdded)
            {
                return;
            }
        }
        panic!("live coin process did not award credit");
    }

    fn trace_field<'a>(line: &'a str, column: &str) -> &'a str {
        let columns = trace_header().split('\t').collect::<Vec<_>>();
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), columns.len(), "trace column count drift");
        let index = columns
            .iter()
            .position(|candidate| *candidate == column)
            .unwrap_or_else(|| panic!("missing trace column {column}"));
        fields[index]
    }

    fn assert_trace_frame_state(
        line: &str,
        expected_state: &str,
        expected_object_crc32: &str,
        expected_shell_crc32: &str,
    ) {
        assert!(line.contains(expected_state));
        assert_eq!(
            trace_field(line, "object_table_crc32"),
            expected_object_crc32
        );
        assert_ne!(trace_field(line, "process_table_crc32"), "-");
        assert_ne!(trace_field(line, "super_process_table_crc32"), "-");
        assert_eq!(trace_field(line, "shell_table_crc32"), expected_shell_crc32);
    }

    #[test]
    fn attract_trace_samples_zeroed_red_label_player_and_seed_ram() {
        let mut machine = ArcadeMachine::new();
        let input = CabinetInput::NONE;

        let output = machine.step(input);
        let trace = TraceFrame::from_output(input, &output);

        assert_eq!(trace.phase, GamePhase::Attract);
        assert_eq!(trace.player_one_score, 0);
        assert_eq!(trace.player_two_score, 0);
        assert_eq!(trace.wave, 0);
        assert_eq!(trace.lives, 0);
        assert_eq!(trace.smart_bombs, 0);
        assert_eq!(trace.seed, 0x35);
        assert_eq!(trace.hseed, 0);
        assert_eq!(trace.lseed, 0);
    }

    #[test]
    fn trace_text_starts_from_cold_boot_object_ram() {
        let trace = trace_text_for_inputs(&[CabinetInput::NONE]).expect("trace text");
        let frame = trace.lines().nth(1).expect("first frame");

        assert_eq!(trace_field(frame, "object_table_crc32"), "0xE15D8394");
        assert_ne!(trace_field(frame, "process_table_crc32"), "-");
        assert_ne!(trace_field(frame, "super_process_table_crc32"), "-");
        assert_eq!(trace_field(frame, "shell_table_crc32"), "0x41D912FF");
    }

    #[test]
    fn trace_text_records_inputs_without_starting_game_during_power_up() {
        let input = CabinetInput {
            coin: true,
            start_one: true,
            ..CabinetInput::NONE
        };
        let trace = trace_text_for_inputs(&[input]).expect("trace text");
        let frame = trace.lines().nth(1).expect("first frame");

        assert!(frame.starts_with("1\t0x0003\t0x20\t0x00\t0x10\tattract\t"));
        assert!(frame.ends_with("\t-\t-"));
    }

    #[test]
    fn trace_text_with_xyzzy_disabled_is_red_label_equivalent() {
        let expanded_inputs = expanded_trace_input_text(
            "none*900;coin*3;none*9;start_one*2;none*120;fire,thrust;smart_bomb;hyperspace",
        )
        .expect("expanded trace input program");
        let inputs =
            parse_trace_input_script(expanded_inputs.trim_end()).expect("scripted trace inputs");
        let default_trace = trace_text_for_inputs(&inputs).expect("default trace text");
        let disabled_xyzzy_trace = super::trace_text_for_inputs_with_compatibility(
            &inputs,
            CompatibilityState {
                xyzzy_active: false,
                xyzzy_invincible: true,
                xyzzy_auto_fire: true,
            },
        )
        .expect("disabled xyzzy trace text");

        let comparison =
            compare_trace_text(&default_trace, &disabled_xyzzy_trace).expect("matching trace");
        assert_eq!(comparison.frames, inputs.len());
    }

    #[test]
    fn trace_text_advances_source_power_up_ram_fill_at_observed_frame_boundary() {
        let inputs = vec![CabinetInput::NONE; 747];
        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_68 = trace.lines().nth(68).expect("frame 68");
        let frame_72 = trace.lines().nth(72).expect("frame 72");
        let frame_240 = trace.lines().nth(240).expect("frame 240");
        let frame_245 = trace.lines().nth(245).expect("frame 245");
        let frame_720 = trace.lines().nth(720).expect("frame 720");
        let frame_721 = trace.lines().nth(721).expect("frame 721");
        let frame_724 = trace.lines().nth(724).expect("frame 724");
        let frame_731 = trace.lines().nth(731).expect("frame 731");
        let frame_732 = trace.lines().nth(732).expect("frame 732");
        let frame_746 = trace.lines().nth(746).expect("frame 746");

        assert_trace_frame_state(
            frame_68,
            "\tgame_over\t145607283\t60413270\t174\t159\t79\t0x44\t0xA3\t0xA2\t",
            "0xA20F8966",
            "0x7C785B90",
        );
        assert_trace_frame_state(
            frame_72,
            "\tgame_over\t145607283\t60413270\t174\t159\t79\t0x44\t0xA3\t0xA2\t",
            "0x790AE7A4",
            "0x7C785B90",
        );
        assert_trace_frame_state(
            frame_240,
            "\tgame_over\t145607283\t60413270\t174\t159\t79\t0x44\t0xA3\t0xA2\t",
            "0x790AE7A4",
            "0x67D19934",
        );
        assert_trace_frame_state(
            frame_245,
            "\tgame_over\t73233961\t142405422\t71\t88\t44\t0x9A\t0x68\t0xCD\t",
            "0x4A92B837",
            "0x67D19934",
        );
        assert_trace_frame_state(
            frame_720,
            "\tplaying\t73233961\t142405422\t71\t88\t44\t0x00\t0x00\t0x00\t",
            "0x4A92B837",
            "0x41D912FF",
        );
        assert_trace_frame_state(
            frame_721,
            "\tattract\t0\t0\t0\t0\t0\t0x00\t0x00\t0x00\t",
            "0x6EE2736A",
            "0x41D912FF",
        );
        assert_trace_frame_state(
            frame_724,
            "\tattract\t0\t0\t0\t0\t0\t0x00\t0x00\t0x00\t",
            "0xE15D8394",
            "0x41D912FF",
        );
        assert_trace_frame_state(
            frame_731,
            "\tattract\t0\t0\t0\t0\t0\t0xD9\t0xF6\t0xCC\t",
            "0xE15D8394",
            "0x41D912FF",
        );
        assert_eq!(trace_field(frame_731, "sound_commands"), "0xC0");
        assert_trace_frame_state(
            frame_732,
            "\tgame_over\t0\t0\t0\t0\t0\t0x3E\t0xB0\t0x13\t",
            "0x9075E2DD",
            "0x41D912FF",
        );
        assert_trace_frame_state(
            frame_746,
            "\tgame_over\t0\t0\t0\t0\t0\t0xFE\t0x3A\t0x21\t",
            "0x9075E2DD",
            "0x41D912FF",
        );
    }

    #[test]
    fn trace_text_does_not_hold_rand_on_credited_start_fixture_frames_without_live_process() {
        let inputs = vec![CabinetInput::NONE; 1018];
        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_1017 = trace.lines().nth(1017).expect("frame 1017");
        let frame_1018 = trace.lines().nth(1018).expect("frame 1018");

        assert_ne!(
            trace_frame_rand_state(frame_1017),
            trace_frame_rand_state(frame_1018)
        );
    }

    #[test]
    fn trace_text_advances_rand_on_first_credited_coin_frame() {
        let mut inputs = vec![CabinetInput::NONE; 900];
        inputs.push(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });

        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_900 = trace.lines().nth(900).expect("frame 900");
        let frame_901 = trace.lines().nth(901).expect("frame 901");

        assert_eq!(trace_frame_rand_state(frame_900), ("0xDB", "0x1C", "0xA3"));
        assert_eq!(trace_frame_rand_state(frame_901), ("0x81", "0x8E", "0x51"));
    }

    #[test]
    fn trace_text_aligns_delayed_coin_credit_event_with_source_sound_command() {
        let mut inputs = vec![CabinetInput::NONE; 900];
        inputs.extend(
            [CabinetInput {
                coin: true,
                ..CabinetInput::NONE
            }; 4],
        );
        inputs.extend([CabinetInput::NONE; 9]);

        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_911 = trace.lines().nth(911).expect("frame 911");
        let frame_912 = trace.lines().nth(912).expect("frame 912");

        assert!(frame_911.ends_with("\t-\t-"));
        assert!(frame_912.ends_with("\t0xE6\tcredit_added"));
    }

    #[test]
    fn trace_text_keeps_cold_boot_attract_process_cadence_after_credit() {
        let mut inputs = vec![CabinetInput::NONE; 900];
        inputs.extend(
            [CabinetInput {
                coin: true,
                ..CabinetInput::NONE
            }; 4],
        );
        inputs.extend([CabinetInput::NONE; 16]);

        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_912 = trace.lines().nth(912).expect("frame 912");
        let frame_913 = trace.lines().nth(913).expect("frame 913");
        let frame_914 = trace.lines().nth(914).expect("frame 914");

        assert!(frame_912.ends_with("\t0xE6\tcredit_added"));
        assert_ne!(
            trace_field(frame_912, "process_table_crc32"),
            trace_field(frame_913, "process_table_crc32")
        );
        assert_ne!(
            trace_field(frame_913, "process_table_crc32"),
            trace_field(frame_914, "process_table_crc32")
        );
    }

    #[test]
    fn trace_text_aligns_debounced_start_event_with_source_sound_command() {
        let mut inputs = vec![CabinetInput::NONE; 900];
        inputs.extend(
            [CabinetInput {
                coin: true,
                ..CabinetInput::NONE
            }; 4],
        );
        inputs.extend([CabinetInput::NONE; 120]);
        inputs.extend(
            [CabinetInput {
                start_one: true,
                ..CabinetInput::NONE
            }; 4],
        );

        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_1026 = trace.lines().nth(1026).expect("frame 1026");
        let frame_1027 = trace.lines().nth(1027).expect("frame 1027");

        assert!(frame_1026.ends_with("\t-\t-"));
        assert!(frame_1027.ends_with("\t0xF5\tgame_started"));
    }

    #[test]
    fn trace_text_keeps_late_post_start_gameplay_state_after_cold_boot_handoff() {
        let mut inputs = vec![CabinetInput::NONE; 900];
        inputs.extend(
            [CabinetInput {
                coin: true,
                ..CabinetInput::NONE
            }; 4],
        );
        inputs.extend([CabinetInput::NONE; 120]);
        inputs.extend(
            [CabinetInput {
                start_one: true,
                ..CabinetInput::NONE
            }; 4],
        );
        inputs.extend([CabinetInput::NONE; 300]);

        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_1328 = trace.lines().nth(1328).expect("frame 1328");
        let fields = frame_1328.split('\t').collect::<Vec<_>>();

        assert_eq!(fields[5], "playing");
        assert_eq!(fields[8], "1");
        assert_ne!(fields[9], "0");
        assert_ne!(fields[10], "0");
    }

    fn trace_frame_rand_state(line: &str) -> (&str, &str, &str) {
        let fields = line.split('\t').collect::<Vec<_>>();
        (fields[11], fields[12], fields[13])
    }

    #[test]
    fn trace_frame_can_record_raw_table_checksums_without_defining_layouts() {
        let mut machine = ArcadeMachine::new();
        let input = CabinetInput::NONE;
        let output = machine.step(input);
        let object_table = [0x10, 0x20, 0x30, 0x40];
        let process_table = [0x11, 0x22, 0x33];
        let super_process_table = [0x44, 0x55, 0x66, 0x77];
        let shell_table = [0xAA, 0x55];

        let trace = TraceFrame::from_output(input, &output)
            .with_object_table_bytes(&object_table)
            .with_process_table_bytes(&process_table)
            .with_super_process_table_bytes(&super_process_table)
            .with_shell_table_bytes(&shell_table);

        let object_crc = crc32(&object_table);
        let process_crc = crc32(&process_table);
        let super_process_crc = crc32(&super_process_table);
        let shell_crc = crc32(&shell_table);
        assert_eq!(trace.object_table_crc32, Some(object_crc));
        assert_eq!(trace.process_table_crc32, Some(process_crc));
        assert_eq!(trace.super_process_table_crc32, Some(super_process_crc));
        assert_eq!(trace.shell_table_crc32, Some(shell_crc));
        let line = trace.to_tsv_line();
        assert_eq!(
            trace_field(&line, "object_table_crc32"),
            format!("0x{object_crc:08X}")
        );
        assert_eq!(
            trace_field(&line, "process_table_crc32"),
            format!("0x{process_crc:08X}")
        );
        assert_eq!(
            trace_field(&line, "super_process_table_crc32"),
            format!("0x{super_process_crc:08X}")
        );
        assert_eq!(
            trace_field(&line, "shell_table_crc32"),
            format!("0x{shell_crc:08X}")
        );
    }

    #[test]
    fn trace_frame_can_record_native_video_checksum_without_interpreting_pixels() {
        let mut machine = ArcadeMachine::new();
        let input = CabinetInput::NONE;
        let output = machine.step(input);
        let video = RenderedImage {
            width: 2,
            height: 1,
            pixels: vec![0, 0, 0, 255, 255, 255, 255, 255],
        };

        let trace = TraceFrame::from_output_with_video(input, &output, &video);

        let expected_crc = crc32(&video.pixels);
        assert_eq!(trace.video_crc32, Some(expected_crc));
        assert!(
            trace
                .to_tsv_line()
                .contains(&format!("\t0x{expected_crc:08X}\t"))
        );
    }

    #[test]
    fn trace_output_includes_header_and_empty_event_marker() {
        let mut machine = ArcadeMachine::new();
        let input = CabinetInput::NONE;
        let output = machine.step(input);

        let trace = trace_output(input, &output);

        assert!(trace.starts_with(trace_header()));
        assert_eq!(
            TraceFrame::from_output(input, &output).video_crc32,
            output.video_crc32
        );
        assert!(trace.ends_with("\t-\t-\n"));
    }

    #[test]
    fn trace_input_script_maps_cabinet_actions_per_frame() {
        let inputs = parse_trace_input_script("coin,start_one;fire,thrust;none")
            .expect("trace input script should parse");

        assert_eq!(inputs.len(), 3);
        assert!(inputs[0].coin);
        assert!(inputs[0].start_one);
        assert!(inputs[1].fire);
        assert!(inputs[1].thrust);
        assert_eq!(inputs[2], CabinetInput::NONE);
    }

    #[test]
    fn trace_input_script_rejects_empty_or_unknown_frames() {
        assert!(parse_trace_input_script("").is_err());
        assert!(parse_trace_input_script("coin;;fire").is_err());

        let error = parse_trace_input_script("coin;warp").expect_err("unknown action");
        assert!(error.contains("unknown trace input action"));
        assert!(error.contains("frame 2"));
    }

    #[test]
    fn trace_comparison_accepts_matching_text_and_reports_frame_count() {
        let inputs = [
            CabinetInput::NONE,
            CabinetInput {
                fire: true,
                thrust: true,
                ..CabinetInput::NONE
            },
        ];
        let trace = trace_text_for_inputs(&inputs).expect("trace text");

        let comparison = compare_trace_text(&trace, &trace).expect("matching trace");

        assert_eq!(comparison.frames, 2);
    }

    #[test]
    fn trace_comparison_reports_changed_lines() {
        let inputs = [CabinetInput::NONE];
        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let changed_trace = trace.replace("\tattract\t", "\tplaying\t");

        let mismatch = compare_trace_text(&trace, &changed_trace).expect_err("trace mismatch");

        assert_eq!(mismatch.line, 2);
        assert!(
            mismatch
                .expected
                .as_deref()
                .expect("expected line")
                .contains("\tattract\t")
        );
        assert!(
            mismatch
                .actual
                .as_deref()
                .expect("actual line")
                .contains("\tplaying\t")
        );
        assert!(mismatch.to_string().contains("trace mismatch at line 2"));
    }

    #[test]
    fn trace_comparison_reports_missing_and_extra_lines() {
        let trace = trace_text_for_inputs(&[CabinetInput::NONE]).expect("trace text");
        let header_only = format!("{}\n", trace_header());

        let missing = compare_trace_text(&trace, &header_only).expect_err("missing line");
        assert_eq!(missing.line, 2);
        assert!(missing.expected.is_some());
        assert_eq!(missing.actual, None);

        let extra = compare_trace_text(&header_only, &trace).expect_err("extra line");
        assert_eq!(extra.line, 2);
        assert_eq!(extra.expected, None);
        assert!(extra.actual.is_some());
    }
}
