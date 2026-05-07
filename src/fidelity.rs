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
    fn phase_ten_trace_schema_gates_core_tables_video_sound_and_events() {
        let columns = trace_header().split('\t').collect::<Vec<_>>();

        for column in [
            "object_table_crc32",
            "process_table_crc32",
            "super_process_table_crc32",
            "shell_table_crc32",
            "video_crc32",
            "sound_commands",
            "events",
        ] {
            assert!(columns.contains(&column), "missing trace column {column}");
        }
    }

    #[test]
    fn phase_ten_manifest_covers_full_playability_reference_scenarios() {
        let scenarios = trace_scenarios().expect("trace scenarios parse");
        let names = scenarios
            .iter()
            .map(|scenario| scenario.scenario.as_str())
            .collect::<Vec<_>>();

        assert_eq!(scenarios.len(), 12);
        for scenario in [
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
        ] {
            assert!(names.contains(&scenario), "missing scenario {scenario}");
        }
    }

    #[test]
    fn phase_ten_sound_requirements_cover_every_playable_reference_scenario() {
        let requirement_lines = crate::assets::RED_LABEL_TRACE_REQUIREMENTS_TSV
            .lines()
            .skip(1)
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>();

        assert_eq!(requirement_lines.len(), 12);
        for line in requirement_lines {
            let fields = line.split('\t').collect::<Vec<_>>();
            assert_eq!(fields.len(), 5);
            let scenario = fields[0];
            if scenario == "attract_boot" {
                assert_eq!(fields[1], "-");
                assert_eq!(fields[2], "-");
            } else {
                assert_eq!(
                    fields[1], "0xE6,0xF5",
                    "{scenario} must keep credited-start sound evidence"
                );
                assert_eq!(
                    fields[2], "credit_added,game_started",
                    "{scenario} must keep credited-start event evidence"
                );
            }
        }
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
    fn trace_text_keeps_video_only_rand_hold_on_credited_start_fixture_frames() {
        let inputs = vec![CabinetInput::NONE; 1018];
        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_1017 = trace.lines().nth(1017).expect("frame 1017");
        let frame_1018 = trace.lines().nth(1018).expect("frame 1018");

        assert_eq!(
            trace_frame_rand_state(frame_1017),
            trace_frame_rand_state(frame_1018)
        );
        assert_eq!(trace_field(frame_1017, "process_table_crc32"), "0x415B8220");
        assert_eq!(trace_field(frame_1018, "process_table_crc32"), "0x415B8220");
        assert_ne!(
            trace_field(frame_1017, "video_crc32"),
            trace_field(frame_1018, "video_crc32")
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
    fn trace_text_keeps_attract_pixel_cadence_on_first_credited_coin_frame() {
        let mut inputs = vec![CabinetInput::NONE; 900];
        inputs.push(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });

        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_901 = trace.lines().nth(901).expect("frame 901");

        assert_eq!(trace_field(frame_901, "process_table_crc32"), "0xDEFE9590");
        assert_eq!(trace_field(frame_901, "video_crc32"), "0x2ABF7D7D");
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
    fn trace_text_keeps_late_defender_appearance_boundaries_before_player_start_release() {
        let mut inputs = vec![CabinetInput::NONE; 900];
        inputs.extend(
            [CabinetInput {
                coin: true,
                ..CabinetInput::NONE
            }; 4],
        );
        inputs.extend([CabinetInput::NONE; 125]);
        inputs.extend(
            [CabinetInput {
                start_one: true,
                ..CabinetInput::NONE
            }; 4],
        );
        inputs.extend([CabinetInput::NONE; 125]);

        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_1094 = trace.lines().nth(1094).expect("frame 1094");
        let frame_1095 = trace.lines().nth(1095).expect("frame 1095");
        let frame_1096 = trace.lines().nth(1096).expect("frame 1096");
        let frame_1097 = trace.lines().nth(1097).expect("frame 1097");
        let frame_1098 = trace.lines().nth(1098).expect("frame 1098");
        let frame_1099 = trace.lines().nth(1099).expect("frame 1099");
        let frame_1100 = trace.lines().nth(1100).expect("frame 1100");
        let frame_1101 = trace.lines().nth(1101).expect("frame 1101");
        let frame_1102 = trace.lines().nth(1102).expect("frame 1102");
        let frame_1103 = trace.lines().nth(1103).expect("frame 1103");
        let frame_1104 = trace.lines().nth(1104).expect("frame 1104");
        let frame_1105 = trace.lines().nth(1105).expect("frame 1105");
        let frame_1106 = trace.lines().nth(1106).expect("frame 1106");
        let frame_1107 = trace.lines().nth(1107).expect("frame 1107");
        let frame_1108 = trace.lines().nth(1108).expect("frame 1108");
        let frame_1109 = trace.lines().nth(1109).expect("frame 1109");
        let frame_1110 = trace.lines().nth(1110).expect("frame 1110");
        let frame_1111 = trace.lines().nth(1111).expect("frame 1111");
        let frame_1112 = trace.lines().nth(1112).expect("frame 1112");
        let frame_1113 = trace.lines().nth(1113).expect("frame 1113");
        let frame_1114 = trace.lines().nth(1114).expect("frame 1114");
        let frame_1115 = trace.lines().nth(1115).expect("frame 1115");
        let frame_1116 = trace.lines().nth(1116).expect("frame 1116");
        let frame_1117 = trace.lines().nth(1117).expect("frame 1117");
        let frame_1118 = trace.lines().nth(1118).expect("frame 1118");
        let frame_1119 = trace.lines().nth(1119).expect("frame 1119");
        let frame_1120 = trace.lines().nth(1120).expect("frame 1120");
        let frame_1128 = trace.lines().nth(1128).expect("frame 1128");
        let frame_1138 = trace.lines().nth(1138).expect("frame 1138");
        let frame_1148 = trace.lines().nth(1148).expect("frame 1148");
        let frame_1152 = trace.lines().nth(1152).expect("frame 1152");
        let frame_1153 = trace.lines().nth(1153).expect("frame 1153");
        let frame_1158 = trace.lines().nth(1158).expect("frame 1158");

        assert_eq!(trace_frame_rand_state(frame_1094), ("0xCA", "0x93", "0x49"));
        assert_eq!(trace_field(frame_1094, "process_table_crc32"), "0xCBC566B3");
        assert_eq!(trace_field(frame_1094, "video_crc32"), "0xF7F32A16");
        assert_eq!(trace_frame_rand_state(frame_1095), ("0x5D", "0x49", "0xA4"));
        assert_eq!(trace_field(frame_1095, "process_table_crc32"), "0xB0D5079D");
        assert_eq!(trace_field(frame_1095, "video_crc32"), "0xEE15F0FF");
        assert_eq!(trace_frame_rand_state(frame_1096), ("0x1E", "0x24", "0xD2"));
        assert_eq!(trace_field(frame_1096, "process_table_crc32"), "0x7610175B");
        assert_eq!(trace_field(frame_1096, "video_crc32"), "0x4948C8AC");
        assert_eq!(trace_frame_rand_state(frame_1097), ("0x1E", "0x24", "0xD2"));
        assert_eq!(trace_field(frame_1097, "process_table_crc32"), "0xE1647BE7");
        assert_eq!(trace_field(frame_1097, "video_crc32"), "0xA9A2EA5B");
        assert_eq!(trace_frame_rand_state(frame_1098), ("0x1E", "0x24", "0xD2"));
        assert_eq!(trace_field(frame_1098, "process_table_crc32"), "0xE1647BE7");
        assert_eq!(trace_field(frame_1098, "video_crc32"), "0xCD369FA2");
        assert_eq!(trace_frame_rand_state(frame_1099), ("0xE6", "0x12", "0x69"));
        assert_eq!(trace_field(frame_1099, "process_table_crc32"), "0x924C91C1");
        assert_eq!(trace_field(frame_1099, "video_crc32"), "0x8474CD12");
        assert_eq!(trace_frame_rand_state(frame_1100), ("0xE6", "0x12", "0x69"));
        assert_eq!(trace_field(frame_1100, "process_table_crc32"), "0x924C91C1");
        assert_eq!(trace_field(frame_1100, "video_crc32"), "0x95AE6CF7");
        assert_eq!(trace_frame_rand_state(frame_1101), ("0x00", "0x09", "0x34"));
        assert_eq!(trace_field(frame_1101, "process_table_crc32"), "0xD4A341D3");
        assert_eq!(trace_field(frame_1101, "video_crc32"), "0x92119D3B");
        assert_eq!(trace_frame_rand_state(frame_1102), ("0x00", "0x09", "0x34"));
        assert_eq!(trace_field(frame_1102, "process_table_crc32"), "0xD4A341D3");
        assert_eq!(trace_field(frame_1102, "video_crc32"), "0x2385B892");
        assert_eq!(trace_frame_rand_state(frame_1103), ("0xAF", "0x04", "0x9A"));
        assert_eq!(trace_field(frame_1103, "process_table_crc32"), "0xB5D8174D");
        assert_eq!(trace_field(frame_1103, "video_crc32"), "0x8082A3E7");
        assert_eq!(trace_frame_rand_state(frame_1104), ("0xAF", "0x04", "0x9A"));
        assert_eq!(trace_field(frame_1104, "process_table_crc32"), "0xB5D8174D");
        assert_eq!(trace_field(frame_1104, "video_crc32"), "0x7241518D");
        assert_eq!(trace_frame_rand_state(frame_1105), ("0xED", "0x82", "0x4D"));
        assert_eq!(trace_field(frame_1105, "process_table_crc32"), "0xE411B7A0");
        assert_eq!(trace_field(frame_1105, "video_crc32"), "0x51B2332C");
        assert_eq!(trace_frame_rand_state(frame_1106), ("0xED", "0x82", "0x4D"));
        assert_eq!(trace_field(frame_1106, "process_table_crc32"), "0x8FC1F7B1");
        assert_eq!(trace_field(frame_1106, "video_crc32"), "0xF2F48164");
        assert_eq!(trace_frame_rand_state(frame_1107), ("0xED", "0x82", "0x4D"));
        assert_eq!(trace_field(frame_1107, "process_table_crc32"), "0x8FC1F7B1");
        assert_eq!(trace_field(frame_1107, "video_crc32"), "0xC4C21960");
        assert_eq!(trace_frame_rand_state(frame_1108), ("0x3F", "0x41", "0x26"));
        assert_eq!(trace_field(frame_1108, "process_table_crc32"), "0x12CA5B62");
        assert_eq!(trace_field(frame_1108, "video_crc32"), "0x24E560C5");
        assert_eq!(trace_frame_rand_state(frame_1109), ("0x3F", "0x41", "0x26"));
        assert_eq!(trace_field(frame_1109, "process_table_crc32"), "0x3D8F57F4");
        assert_eq!(trace_field(frame_1109, "video_crc32"), "0xEAA73796");
        assert_eq!(trace_frame_rand_state(frame_1110), ("0x3F", "0x41", "0x26"));
        assert_eq!(trace_field(frame_1110, "process_table_crc32"), "0xB03A551A");
        assert_eq!(trace_field(frame_1110, "video_crc32"), "0xBBD00335");
        assert_eq!(trace_frame_rand_state(frame_1111), ("0x82", "0x20", "0x93"));
        assert_eq!(trace_field(frame_1111, "process_table_crc32"), "0x16F30048");
        assert_eq!(trace_field(frame_1111, "video_crc32"), "0xD7E1179B");
        assert_eq!(trace_frame_rand_state(frame_1112), ("0x70", "0x90", "0x49"));
        assert_eq!(trace_field(frame_1112, "process_table_crc32"), "0xC45E3518");
        assert_eq!(trace_field(frame_1112, "video_crc32"), "0xC271DF44");
        assert_eq!(trace_frame_rand_state(frame_1113), ("0xCD", "0x48", "0x24"));
        assert_eq!(trace_field(frame_1113, "process_table_crc32"), "0x029B25DE");
        assert_eq!(trace_field(frame_1113, "video_crc32"), "0x9A55A3F3");
        assert_eq!(trace_frame_rand_state(frame_1114), ("0xCD", "0x48", "0x24"));
        assert_eq!(trace_field(frame_1114, "process_table_crc32"), "0x029B25DE");
        assert_eq!(trace_field(frame_1114, "video_crc32"), "0x0DB382A2");
        assert_eq!(trace_frame_rand_state(frame_1115), ("0xCD", "0x48", "0x24"));
        assert_eq!(trace_field(frame_1115, "process_table_crc32"), "0x029B25DE");
        assert_eq!(trace_field(frame_1115, "video_crc32"), "0x4BB94EED");
        assert_eq!(trace_frame_rand_state(frame_1116), ("0xCD", "0x48", "0x24"));
        assert_eq!(trace_field(frame_1116, "process_table_crc32"), "0x029B25DE");
        assert_eq!(trace_field(frame_1116, "video_crc32"), "0x4BB94EED");
        assert_eq!(trace_frame_rand_state(frame_1117), ("0xAE", "0x24", "0x12"));
        assert_eq!(trace_field(frame_1117, "process_table_crc32"), "0x581F39FE");
        assert_eq!(trace_field(frame_1117, "video_crc32"), "0x4BB94EED");
        assert_eq!(trace_frame_rand_state(frame_1118), ("0x36", "0x12", "0x09"));
        assert_eq!(trace_field(frame_1118, "process_table_crc32"), "0xC514952D");
        assert_eq!(trace_field(frame_1118, "video_crc32"), "0x4BB94EED");
        assert_eq!(trace_frame_rand_state(frame_1119), ("0xC0", "0x09", "0x04"));
        assert_eq!(trace_field(frame_1119, "process_table_crc32"), "0x89952A6E");
        assert_eq!(trace_field(frame_1119, "video_crc32"), "0x4BB94EED");
        assert_eq!(trace_frame_rand_state(frame_1120), ("0xD7", "0x04", "0x82"));
        assert_eq!(trace_field(frame_1120, "process_table_crc32"), "0xCF7AFA7C");
        assert_eq!(trace_field(frame_1120, "video_crc32"), "0x4BB94EED");
        assert_eq!(trace_frame_rand_state(frame_1128), ("0x5B", "0x12", "0x04"));
        assert_eq!(trace_field(frame_1128, "process_table_crc32"), "0x38485366");
        assert_eq!(trace_field(frame_1128, "video_crc32"), "0x4BB94EED");
        assert_eq!(trace_frame_rand_state(frame_1138), ("0xD8", "0x11", "0x04"));
        assert_eq!(trace_field(frame_1138, "process_table_crc32"), "0x7B4F740C");
        assert_eq!(trace_field(frame_1138, "video_crc32"), "0x4BB94EED");
        assert_eq!(trace_frame_rand_state(frame_1148), ("0xC0", "0xC9", "0x04"));
        assert_eq!(trace_field(frame_1148, "process_table_crc32"), "0x21AE8F46");
        assert_eq!(trace_field(frame_1148, "video_crc32"), "0x4BB94EED");
        assert_eq!(trace_frame_rand_state(frame_1152), ("0xBD", "0x4C", "0x90"));
        assert_eq!(trace_field(frame_1152, "process_table_crc32"), "0xB7F720AC");
        assert_eq!(trace_field(frame_1152, "video_crc32"), "0x369910B2");
        assert_eq!(trace_frame_rand_state(frame_1153), ("0xB6", "0x26", "0x48"));
        assert_eq!(trace_field(frame_1153, "process_table_crc32"), "0x2AC84733");
        assert_eq!(trace_field(frame_1153, "video_crc32"), "0x58FA3AE7");
        assert_eq!(trace_frame_rand_state(frame_1158), ("0xD8", "0x09", "0x32"));
        assert_eq!(trace_field(frame_1158, "process_table_crc32"), "0xE8C08D0D");
        assert_eq!(trace_field(frame_1158, "video_crc32"), "0x58FA3AE7");
    }

    #[test]
    fn trace_text_keeps_instruction_handoff_sample_crc_boundaries() {
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
        inputs.extend([CabinetInput::NONE; 332]);

        let trace = trace_text_for_inputs(&inputs).expect("trace text");
        let frame_1195 = trace.lines().nth(1195).expect("frame 1195");
        let frame_1204 = trace.lines().nth(1204).expect("frame 1204");
        let frame_1206 = trace.lines().nth(1206).expect("frame 1206");
        let frame_1210 = trace.lines().nth(1210).expect("frame 1210");
        let frame_1216 = trace.lines().nth(1216).expect("frame 1216");
        let frame_1222 = trace.lines().nth(1222).expect("frame 1222");
        let frame_1228 = trace.lines().nth(1228).expect("frame 1228");
        let frame_1229 = trace.lines().nth(1229).expect("frame 1229");
        let frame_1234 = trace.lines().nth(1234).expect("frame 1234");
        let frame_1235 = trace.lines().nth(1235).expect("frame 1235");
        let frame_1236 = trace.lines().nth(1236).expect("frame 1236");
        let frame_1258 = trace.lines().nth(1258).expect("frame 1258");
        let frame_1259 = trace.lines().nth(1259).expect("frame 1259");
        let frame_1266 = trace.lines().nth(1266).expect("frame 1266");
        let frame_1270 = trace.lines().nth(1270).expect("frame 1270");
        let frame_1282 = trace.lines().nth(1282).expect("frame 1282");
        let frame_1283 = trace.lines().nth(1283).expect("frame 1283");
        let frame_1289 = trace.lines().nth(1289).expect("frame 1289");
        let frame_1290 = trace.lines().nth(1290).expect("frame 1290");
        let frame_1291 = trace.lines().nth(1291).expect("frame 1291");
        let frame_1292 = trace.lines().nth(1292).expect("frame 1292");
        let frame_1293 = trace.lines().nth(1293).expect("frame 1293");
        let frame_1294 = trace.lines().nth(1294).expect("frame 1294");
        let frame_1295 = trace.lines().nth(1295).expect("frame 1295");
        let frame_1296 = trace.lines().nth(1296).expect("frame 1296");
        let frame_1297 = trace.lines().nth(1297).expect("frame 1297");
        let frame_1298 = trace.lines().nth(1298).expect("frame 1298");
        let frame_1299 = trace.lines().nth(1299).expect("frame 1299");
        let frame_1300 = trace.lines().nth(1300).expect("frame 1300");
        let frame_1301 = trace.lines().nth(1301).expect("frame 1301");
        let frame_1302 = trace.lines().nth(1302).expect("frame 1302");
        let frame_1303 = trace.lines().nth(1303).expect("frame 1303");
        let frame_1304 = trace.lines().nth(1304).expect("frame 1304");
        let frame_1305 = trace.lines().nth(1305).expect("frame 1305");
        let frame_1306 = trace.lines().nth(1306).expect("frame 1306");
        let frame_1307 = trace.lines().nth(1307).expect("frame 1307");
        let frame_1308 = trace.lines().nth(1308).expect("frame 1308");

        assert_eq!(trace_frame_rand_state(frame_1195), ("0x42", "0x8B", "0x44"));
        assert_eq!(trace_field(frame_1195, "object_table_crc32"), "0x48ED611B");
        assert_eq!(trace_field(frame_1195, "process_table_crc32"), "0x3D82A47F");
        assert_eq!(trace_field(frame_1195, "video_crc32"), "0x19C411E1");
        assert_eq!(trace_field(frame_1204, "process_table_crc32"), "0x448AD0F0");
        assert_eq!(trace_field(frame_1204, "video_crc32"), "0x172FBE2F");
        assert_eq!(trace_field(frame_1206, "process_table_crc32"), "0xB4DDF519");
        assert_eq!(trace_field(frame_1206, "video_crc32"), "0x9CEB8542");
        assert_eq!(trace_field(frame_1210, "process_table_crc32"), "0xE4078CFB");
        assert_eq!(trace_field(frame_1210, "video_crc32"), "0x22DC6057");
        assert_eq!(trace_field(frame_1216, "process_table_crc32"), "0x85A53777");
        assert_eq!(trace_field(frame_1216, "video_crc32"), "0x882DCFF2");
        assert_eq!(trace_field(frame_1222, "object_table_crc32"), "0x1DFD9317");
        assert_eq!(trace_field(frame_1222, "process_table_crc32"), "0x76FCA77D");
        assert_eq!(trace_field(frame_1222, "video_crc32"), "0x6DEB7656");
        assert_eq!(trace_field(frame_1228, "video_crc32"), "0xF2A3DE00");
        assert_eq!(trace_field(frame_1229, "video_crc32"), "0x7CF8C674");
        assert_eq!(trace_field(frame_1234, "process_table_crc32"), "0x57B145F1");
        assert_eq!(trace_field(frame_1234, "video_crc32"), "0xA2C31EAE");
        assert_eq!(trace_field(frame_1235, "video_crc32"), "0x92834217");
        assert_eq!(trace_field(frame_1236, "video_crc32"), "0x084F74A0");
        assert_eq!(trace_field(frame_1258, "process_table_crc32"), "0xEBA74B00");
        assert_eq!(trace_field(frame_1258, "video_crc32"), "0xAB882140");
        assert_eq!(trace_field(frame_1259, "process_table_crc32"), "0xCE0643A9");
        assert_eq!(trace_field(frame_1259, "video_crc32"), "0xFDAE7B4C");
        assert_eq!(trace_field(frame_1266, "video_crc32"), "0xF1EB0674");
        assert_eq!(trace_field(frame_1270, "process_table_crc32"), "0xA1B9B050");
        assert_eq!(trace_field(frame_1282, "process_table_crc32"), "0x392752CA");
        assert_eq!(trace_field(frame_1283, "video_crc32"), "0xC1BDE031");
        assert_eq!(trace_field(frame_1289, "video_crc32"), "0x62573290");
        assert_eq!(trace_field(frame_1290, "video_crc32"), "0x6B087042");
        assert_eq!(trace_field(frame_1291, "object_table_crc32"), "0xF75BF69B");
        assert_eq!(trace_field(frame_1291, "video_crc32"), "0x8FB1402E");
        assert_eq!(trace_field(frame_1292, "object_table_crc32"), "0xD70D26D6");
        assert_eq!(trace_field(frame_1292, "video_crc32"), "0xAE7B3645");
        assert_eq!(trace_field(frame_1293, "object_table_crc32"), "0x81610635");
        assert_eq!(trace_field(frame_1293, "video_crc32"), "0xC7F23178");
        assert_eq!(trace_field(frame_1294, "object_table_crc32"), "0xDA6908F3");
        assert_eq!(trace_field(frame_1294, "video_crc32"), "0x86D514F3");
        assert_eq!(trace_field(frame_1295, "video_crc32"), "0x4D7E9152");
        assert_eq!(trace_field(frame_1296, "object_table_crc32"), "0xEDF3013F");
        assert_eq!(trace_field(frame_1296, "video_crc32"), "0xACCCAED9");
        assert_eq!(trace_field(frame_1297, "video_crc32"), "0x398EFF52");
        assert_eq!(trace_field(frame_1298, "object_table_crc32"), "0x04E92E89");
        assert_eq!(trace_field(frame_1298, "video_crc32"), "0xC6A3A07F");
        assert_eq!(trace_field(frame_1299, "object_table_crc32"), "0x759EFEB7");
        assert_eq!(trace_field(frame_1299, "video_crc32"), "0xE6B4D2D6");
        assert_eq!(trace_field(frame_1300, "video_crc32"), "0xA1D90CC5");
        assert_eq!(trace_field(frame_1301, "object_table_crc32"), "0xFC10930F");
        assert_eq!(trace_field(frame_1301, "video_crc32"), "0x90D03CF5");
        assert_eq!(trace_field(frame_1302, "object_table_crc32"), "0x6F60E207");
        assert_eq!(trace_field(frame_1302, "video_crc32"), "0xD86B7C0A");
        assert_eq!(trace_field(frame_1303, "video_crc32"), "0xCECFB74C");
        assert_eq!(trace_field(frame_1304, "object_table_crc32"), "0x8503CB71");
        assert_eq!(trace_field(frame_1304, "video_crc32"), "0x5261547F");
        assert_eq!(trace_field(frame_1305, "video_crc32"), "0x6CA3D53A");
        assert_eq!(trace_field(frame_1306, "object_table_crc32"), "0x588FB88C");
        assert_eq!(trace_field(frame_1306, "process_table_crc32"), "0x3AC090F9");
        assert_eq!(trace_field(frame_1306, "video_crc32"), "0x0EBDEBA7");
        assert_eq!(trace_field(frame_1307, "object_table_crc32"), "0x8101F78D");
        assert_eq!(trace_field(frame_1307, "process_table_crc32"), "0xC0CFCFA1");
        assert_eq!(trace_field(frame_1307, "video_crc32"), "0x7633308A");
        assert_eq!(trace_field(frame_1308, "video_crc32"), "0x7E7DDFBF");

        for (frame_number, object_crc32, process_crc32, video_crc32) in [
            (1309, "0xB69BFE41", "0x7C1D32B5", "0x66426E15"),
            (1310, "0x71FC33F3", "0x409C0C0D", "0xC46B8A44"),
            (1311, "0x789A212A", "0x7C978C85", "0x55B34B70"),
            (1312, "0x508BC174", "0x6ABF26BC", "0x2DC83755"),
            (1313, "0x17DA3217", "0xB36F840E", "0x9E9445C1"),
            (1314, "0x25FE2DB6", "0x8743E040", "0x215AB95F"),
            (1315, "0x6FBFF055", "0x35E77D46", "0x637BA67B"),
            (1316, "0x29528056", "0x4D83C26A", "0xFF937191"),
            (1317, "0x99F0DE30", "0x673E6211", "0xECFF348E"),
            (1318, "0x6A014FC9", "0x351FB337", "0xF91153CC"),
            (1319, "0x2D50BCAA", "0x101DBF0F", "0x03E7518B"),
            (1320, "0xE38F2271", "0x4D0BB905", "0x72D9B81A"),
            (1321, "0xEAE930A8", "0x1BB3AE09", "0xBD2265D8"),
            (1322, "0x28EEAAC7", "0x369646F7", "0x656ED429"),
            (1323, "0x6D0F26B2", "0x79DBE5FB", "0x1AE77360"),
            (1324, "0x1A14F4D6", "0x952FA115", "0x59689022"),
            (1325, "0x4C78D435", "0x9B308E0D", "0xFF8A9969"),
            (1326, "0x1770DAF3", "0xBEB83C05", "0x084D901E"),
            (1327, "0x1E16C82A", "0x334CD60B", "0xDD54C073"),
            (1328, "0xE1E912DD", "0x77535CB1", "0xDD54C073"),
        ] {
            let frame = trace.lines().nth(frame_number).expect("frame");
            assert_eq!(trace_field(frame, "object_table_crc32"), object_crc32);
            assert_eq!(trace_field(frame, "process_table_crc32"), process_crc32);
            assert_eq!(trace_field(frame, "video_crc32"), video_crc32);
        }

        for (frame_number, object_crc32, process_crc32, video_crc32) in [
            (1329, "0xFD0DAB56", "0xAE83FE03", "0x7A0CE222"),
            (1330, "0xCF29B4F7", "0xBB7D9DDB", "0x1322D8BA"),
            (1331, "0xBE5E64C9", "0x9EDC9572", "0xA118FA15"),
            (1332, "0x87725717", "0xE6B82A5E", "0x7DE4A2BA"),
            (1333, "0x37D00971", "0x28E5389F", "0xCABF786F"),
            (1334, "0xD2CC63C3", "0xA957868F", "0x324AF8EC"),
            (1335, "0x09F253D4", "0x286F86AF", "0x9BBF0892"),
            (1336, "0xC72DCD0F", "0x02D0E38B", "0x5E69A807"),
            (1337, "0xCE4BDFD6", "0xC209CD89", "0x4F41CBB7"),
            (1338, "0x1AA1BEF2", "0xEF2C2577", "0x63C92D8E"),
            (1339, "0xC32FF1F3", "0x3600BF75", "0x86011F95"),
            (1340, "0xB4342397", "0x576D8CE9", "0xBE803FFB"),
            (1341, "0xD96E0EA9", "0xD488AB1B", "0x40CBA97C"),
            (1342, "0xFDA743B2", "0x7C996E61", "0x9A4551E8"),
            (1343, "0xF4C1516B", "0x599B6259", "0x43FA95EF"),
            (1344, "0xDCD0B135", "0x048D6453", "0xC87E9881"),
            (1345, "0x9B814256", "0x6CA2AC67", "0x11FE442F"),
            (1346, "0xA9A55DF7", "0x588EC829", "0x8E8BBFCA"),
            (1347, "0xCE3F7682", "0x0ECAE795", "0xB17B4151"),
            (1348, "0x6B7C8628", "0x2499783A", "0xF1FB68B8"),
            (1349, "0xDBDED84E", "0x0E24D841", "0x0ECCB021"),
            (1350, "0x282F49B7", "0x8F966651", "0x90E68D1F"),
            (1351, "0x6F7EBAD4", "0xEA4ED4CB", "0x12914ABB"),
            (1352, "0xA1A1240F", "0xAE515E71", "0xFEEF198D"),
            (1353, "0xA8C736D6", "0x7781FCC3", "0x3D352198"),
            (1354, "0x1CACB703", "0x627F9F1B", "0x318BB20E"),
            (1355, "0xA6CFBCCC", "0x6240D41F", "0x67017467"),
            (1356, "0xD1D46EA8", "0x032DE783", "0x152058B0"),
            (1357, "0x87B84E4B", "0x9B53F195", "0xB4EF6074"),
            (1358, "0xDCB0408D", "0xBEDB439D", "0x8C6AE956"),
            (1359, "0xD5D65254", "0x9BD94FA5", "0xDC59CD9B"),
            (1360, "0xEB2A4941", "0x4B563EDD", "0xBC10AB56"),
        ] {
            let frame = trace.lines().nth(frame_number).expect("frame");
            assert_eq!(trace_field(frame, "object_table_crc32"), object_crc32);
            assert_eq!(trace_field(frame, "process_table_crc32"), process_crc32);
            assert_eq!(trace_field(frame, "video_crc32"), video_crc32);
        }
    }

    #[test]
    fn trace_text_keeps_pre_start_input_video_boundaries_source_specific() {
        let fire = CabinetInput {
            fire: true,
            ..CabinetInput::NONE
        };
        let thrust = CabinetInput {
            thrust: true,
            ..CabinetInput::NONE
        };

        for (
            input,
            frame_1060_video_crc32,
            held_frames,
            late_frame_number,
            late_process_table_crc32,
            late_video_crc32,
        ) in [
            (fire, "0x24BE36C6", 11, 1064, "0xB061DA05", "0xDA691661"),
            (thrust, "0xF6CEDF60", 12, 1066, "0xE1FCD6B4", "0x65524E70"),
        ] {
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
            inputs.extend([CabinetInput::NONE; 30]);
            inputs.extend(vec![input; held_frames]);
            inputs.extend([CabinetInput::NONE; 100]);

            let trace = trace_text_for_inputs(&inputs).expect("trace text");
            let frame_1059 = trace.lines().nth(1059).expect("frame 1059");
            let frame_1060 = trace.lines().nth(1060).expect("frame 1060");
            let frame_1061 = trace.lines().nth(1061).expect("frame 1061");
            let late_frame = trace
                .lines()
                .nth(late_frame_number)
                .expect("late input-video boundary frame");
            let frame_1080 = trace.lines().nth(1080).expect("frame 1080");
            let frame_1084 = trace.lines().nth(1084).expect("frame 1084");
            let frame_1093 = trace.lines().nth(1093).expect("frame 1093");
            let frame_1098 = trace.lines().nth(1098).expect("frame 1098");
            let frame_1111 = trace.lines().nth(1111).expect("frame 1111");
            let frame_1164 = trace.lines().nth(1164).expect("frame 1164");

            assert_eq!(trace_frame_rand_state(frame_1059), ("0x5E", "0xA7", "0xBE"));
            assert_eq!(trace_field(frame_1059, "process_table_crc32"), "0xB87521C6");
            assert_eq!(trace_field(frame_1059, "video_crc32"), "0xB8A9D13E");
            assert_eq!(trace_frame_rand_state(frame_1060), ("0xDE", "0xD3", "0xDF"));
            assert_eq!(trace_field(frame_1060, "process_table_crc32"), "0x5F53C9A4");
            assert_eq!(
                trace_field(frame_1060, "video_crc32"),
                frame_1060_video_crc32
            );
            assert_eq!(trace_frame_rand_state(frame_1061), ("0xDE", "0xD3", "0xDF"));
            assert_eq!(trace_field(frame_1061, "process_table_crc32"), "0x5F53C9A4");
            assert_eq!(trace_field(frame_1061, "video_crc32"), "0x412AAFA0");
            assert_eq!(trace_frame_rand_state(late_frame), ("0x49", "0x34", "0xF7"));
            assert_eq!(
                trace_field(late_frame, "process_table_crc32"),
                late_process_table_crc32
            );
            assert_eq!(trace_field(late_frame, "video_crc32"), late_video_crc32);

            if late_frame_number == 1064 {
                let frame_1065 = trace.lines().nth(1065).expect("frame 1065");
                let frame_1066 = trace.lines().nth(1066).expect("frame 1066");
                let frame_1067 = trace.lines().nth(1067).expect("frame 1067");
                let frame_1068 = trace.lines().nth(1068).expect("frame 1068");
                let frame_1069 = trace.lines().nth(1069).expect("frame 1069");
                assert_eq!(trace_frame_rand_state(frame_1065), ("0x49", "0x34", "0xF7"));
                assert_eq!(trace_field(frame_1065, "process_table_crc32"), "0xE1FCD6B4");
                assert_eq!(trace_field(frame_1065, "video_crc32"), "0xACAB39B5");
                assert_eq!(trace_field(frame_1066, "process_table_crc32"), "0xE1FCD6B4");
                assert_eq!(trace_field(frame_1066, "video_crc32"), "0x344CF817");
                assert_eq!(trace_field(frame_1067, "process_table_crc32"), "0x0D38215F");
                assert_eq!(trace_field(frame_1067, "video_crc32"), "0xDCCBA212");
                assert_eq!(trace_field(frame_1068, "process_table_crc32"), "0x0D38215F");
                assert_eq!(trace_field(frame_1068, "video_crc32"), "0xD6E596B0");
                assert_eq!(trace_field(frame_1069, "process_table_crc32"), "0x42FE2F11");
                assert_eq!(trace_field(frame_1069, "video_crc32"), "0x738D075C");
            } else {
                let frame_1069 = trace.lines().nth(1069).expect("frame 1069");
                let frame_1070 = trace.lines().nth(1070).expect("frame 1070");
                assert_eq!(trace_field(frame_1069, "process_table_crc32"), "0x42FE2F11");
                assert_eq!(trace_field(frame_1069, "video_crc32"), "0x68F8962D");
                assert_eq!(trace_field(frame_1070, "process_table_crc32"), "0x42FE2F11");
                assert_eq!(trace_field(frame_1070, "video_crc32"), "0x1173B3A4");
            }

            if input == fire {
                assert_eq!(trace_field(frame_1080, "video_crc32"), "0x57AE5184");
                assert_eq!(trace_field(frame_1093, "video_crc32"), "0x2318B8F4");
                assert_eq!(trace_field(frame_1111, "video_crc32"), "0xB8DF8442");
                assert_eq!(trace_field(frame_1164, "video_crc32"), "0xF72D1B86");
            } else {
                assert_eq!(trace_field(frame_1084, "video_crc32"), "0xD8909668");
                assert_eq!(trace_field(frame_1098, "video_crc32"), "0x8F778AC6");
                assert_eq!(trace_field(frame_1111, "video_crc32"), "0xB8DF8442");
                assert_eq!(trace_field(frame_1164, "video_crc32"), "0x54E87194");
            }
        }
    }

    #[test]
    fn trace_text_keeps_delayed_smart_bomb_and_hyperspace_video_boundaries() {
        let smart_bomb = CabinetInput {
            smart_bomb: true,
            ..CabinetInput::NONE
        };
        let hyperspace = CabinetInput {
            hyperspace: true,
            ..CabinetInput::NONE
        };

        for (input, frame_1100_video_crc32) in
            [(smart_bomb, "0x1A034D42"), (hyperspace, "0x2B011907")]
        {
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
            inputs.extend([CabinetInput::NONE; 60]);
            inputs.push(input);
            inputs.extend([CabinetInput::NONE; 75]);

            let trace = trace_text_for_inputs(&inputs).expect("trace text");
            let frame_1089 = trace.lines().nth(1089).expect("frame 1089");
            let frame_1090 = trace.lines().nth(1090).expect("frame 1090");
            let frame_1091 = trace.lines().nth(1091).expect("frame 1091");
            let frame_1092 = trace.lines().nth(1092).expect("frame 1092");
            let frame_1093 = trace.lines().nth(1093).expect("frame 1093");
            let frame_1094 = trace.lines().nth(1094).expect("frame 1094");
            let frame_1095 = trace.lines().nth(1095).expect("frame 1095");
            let frame_1099 = trace.lines().nth(1099).expect("frame 1099");
            let frame_1100 = trace.lines().nth(1100).expect("frame 1100");
            let frame_1111 = trace.lines().nth(1111).expect("frame 1111");
            let frame_1164 = trace.lines().nth(1164).expect("frame 1164");

            assert_eq!(trace_frame_rand_state(frame_1089), ("0x0E", "0x4D", "0x26"));
            assert_eq!(trace_field(frame_1089, "process_table_crc32"), "0x2B97C76D");
            assert_eq!(trace_field(frame_1089, "video_crc32"), "0x4158816D");
            assert_eq!(trace_frame_rand_state(frame_1090), ("0xF4", "0x26", "0x93"));
            assert_eq!(trace_field(frame_1090, "process_table_crc32"), "0x0332C9AE");
            assert_eq!(trace_field(frame_1090, "video_crc32"), "0x1958E231");
            assert_eq!(trace_frame_rand_state(frame_1091), ("0xF4", "0x26", "0x93"));
            assert_eq!(trace_field(frame_1091, "process_table_crc32"), "0x0332C9AE");
            assert_eq!(trace_field(frame_1091, "video_crc32"), "0x8A009827");
            assert_eq!(trace_frame_rand_state(frame_1092), ("0xCA", "0x93", "0x49"));
            assert_eq!(trace_field(frame_1092, "process_table_crc32"), "0xC5F7D968");
            assert_eq!(trace_field(frame_1092, "video_crc32"), "0xB56FD48D");
            assert_eq!(trace_frame_rand_state(frame_1093), ("0xCA", "0x93", "0x49"));
            assert_eq!(trace_field(frame_1093, "process_table_crc32"), "0xCBC566B3");
            assert_eq!(trace_field(frame_1093, "video_crc32"), "0x2318B8F4");
            assert_eq!(trace_frame_rand_state(frame_1094), ("0xCA", "0x93", "0x49"));
            assert_eq!(trace_field(frame_1094, "process_table_crc32"), "0xCBC566B3");
            assert_eq!(trace_field(frame_1094, "video_crc32"), "0x9DBFF0EF");
            assert_eq!(trace_frame_rand_state(frame_1095), ("0x5D", "0x49", "0xA4"));
            assert_eq!(trace_field(frame_1095, "process_table_crc32"), "0xB0D5079D");
            assert_eq!(trace_field(frame_1095, "video_crc32"), "0xC24E2B0F");
            assert_eq!(trace_frame_rand_state(frame_1099), ("0xE6", "0x12", "0x69"));
            assert_eq!(trace_field(frame_1099, "process_table_crc32"), "0x924C91C1");
            assert_eq!(trace_field(frame_1099, "video_crc32"), "0xF7C16FEF");
            assert_eq!(trace_frame_rand_state(frame_1100), ("0xE6", "0x12", "0x69"));
            assert_eq!(trace_field(frame_1100, "process_table_crc32"), "0x924C91C1");
            assert_eq!(
                trace_field(frame_1100, "video_crc32"),
                frame_1100_video_crc32
            );
            assert_eq!(trace_field(frame_1111, "video_crc32"), "0xB8DF8442");
            assert_eq!(trace_field(frame_1164, "video_crc32"), "0xF72D1B86");
        }
    }

    #[test]
    fn trace_text_keeps_late_post_start_source_handoff_state_after_cold_boot() {
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

        assert_eq!(fields[5], "game_over");
        assert_eq!(fields[8], "0");
        assert_eq!(fields[9], "0");
        assert_eq!(fields[10], "0");
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
