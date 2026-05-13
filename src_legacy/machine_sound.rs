//! Red-label sound command and fixture helpers.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelLoadedSoundTable {
    pub address: u16,
    pub priority: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelSoundOutput {
    pub sound_number: u8,
    pub idle_port_b: u8,
    pub idle_command: SoundCommand,
    pub command_port_b: u8,
    pub command: SoundCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelSoundTableCommand {
    pub table_pointer: u16,
    pub repeat_index: u8,
    pub repeat_count: u8,
    pub timer: u8,
    pub sound_number: u8,
    pub sound_output: RedLabelSoundOutput,
    pub command: SoundCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelSoundTableTimedCommand {
    pub sequencer_tick: u32,
    pub command: RedLabelSoundTableCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelSoundTableTimeline {
    pub label: String,
    pub address: u16,
    pub priority: u8,
    pub commands: Vec<RedLabelSoundTableTimedCommand>,
    pub sequence_end_tick: u32,
    pub terminator_pointer: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelSoundTableTimelineFixtureCheck {
    pub row_count: usize,
    pub command_rows: usize,
    pub sequence_end_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelSoundTableCommandSequenceFixtureCheck {
    pub row_count: usize,
    pub idle_rows: usize,
    pub command_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelSoundDirectCommandSequenceFixtureCheck {
    pub row_count: usize,
    pub idle_rows: usize,
    pub command_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelSoundThrustCommandSequenceFixtureCheck {
    pub row_count: usize,
    pub idle_rows: usize,
    pub command_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RedLabelSoundThrustGateCommand {
    gate_event: &'static str,
    source_label: &'static str,
    status_block_mask: Option<u8>,
    thrust_flag_before: u8,
    thrust_flag_after: u8,
    sound_number: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RedLabelSoundDirectCommand {
    callsite: &'static str,
    source_file: &'static str,
    source_line: u16,
    source_label: &'static str,
    sound_number: u8,
    source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedLabelSoundSequenceSource {
    Timer,
    Table,
    SequenceEnded,
    ThrustStarted,
    ThrustStopped,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelSoundSequenceStep {
    pub source: RedLabelSoundSequenceSource,
    pub timer_before: u8,
    pub timer_after: u8,
    pub repeat_before: u8,
    pub repeat_after: u8,
    pub table_pointer_before: u16,
    pub table_pointer_after: u16,
    pub priority_after: u8,
    pub thrust_flag_before: u8,
    pub thrust_flag_after: u8,
    pub sound_number: Option<u8>,
    pub sound_output: Option<RedLabelSoundOutput>,
    pub command: Option<SoundCommand>,
}

pub(super) fn red_label_sound_table_address(label: &str) -> Result<u16, String> {
    red_label_sound_tables()?
        .iter()
        .find(|entry| entry.label == label)
        .map(|entry| entry.address)
        .ok_or_else(|| format!("red-label sound table asset has no `{label}` entry"))
}

pub(super) fn red_label_sound_table(address: u16) -> Result<&'static RedLabelSoundTable, String> {
    red_label_sound_tables()?
        .iter()
        .find(|entry| entry.address == address)
        .ok_or_else(|| format!("red-label sound table asset has no 0x{address:04X} entry"))
}

pub(super) fn red_label_sound_table_byte_required(address: u16) -> Result<u8, String> {
    red_label_sound_tables()?
        .iter()
        .find_map(|entry| {
            let offset = address.checked_sub(entry.address)?;
            entry.bytes.get(usize::from(offset)).copied()
        })
        .ok_or_else(|| format!("red-label sound table asset has no byte at 0x{address:04X}"))
}

pub(super) fn red_label_sound_table_command_plan(
    label: &str,
) -> Result<Vec<RedLabelSoundTableCommand>, String> {
    let table = red_label_sound_table(red_label_sound_table_address(label)?)?;
    red_label_sound_table_commands(table)
}

pub(super) fn red_label_sound_table_timed_command_plan(
    label: &str,
) -> Result<Vec<RedLabelSoundTableTimedCommand>, String> {
    let table = red_label_sound_table(red_label_sound_table_address(label)?)?;
    red_label_sound_table_timed_commands(table)
}

pub(super) fn red_label_sound_table_timeline(
    label: &str,
) -> Result<RedLabelSoundTableTimeline, String> {
    let table = red_label_sound_table(red_label_sound_table_address(label)?)?;
    red_label_sound_table_timeline_for_table(table)
}

pub(super) fn red_label_sound_table_timelines() -> Result<Vec<RedLabelSoundTableTimeline>, String> {
    red_label_sound_tables()?
        .iter()
        .map(red_label_sound_table_timeline_for_table)
        .collect()
}

pub(super) fn red_label_sound_table_timeline_tsv() -> Result<String, String> {
    let timelines = red_label_sound_table_timelines()?;
    Ok(red_label_sound_table_timeline_tsv_from_timelines(
        &timelines,
    ))
}

pub(super) fn red_label_sound_table_command_sequence_tsv() -> Result<String, String> {
    let timelines = red_label_sound_table_timelines()?;
    Ok(red_label_sound_table_command_sequence_tsv_from_timelines(
        &timelines,
    ))
}

pub(super) fn red_label_sound_direct_commands() -> [RedLabelSoundDirectCommand; 3] {
    [
        RedLabelSoundDirectCommand {
            callsite: "player_death_stop",
            source_file: "defa7.src",
            source_line: 1386,
            source_label: "PDTH5",
            sound_number: RED_LABEL_PLAYER_END_SOUND_STOP_NUMBER,
            source: "https://github.com/mwenge/defender/blob/master/src/defa7.src#L1384-L1386",
        },
        RedLabelSoundDirectCommand {
            callsite: "game_over_stop",
            source_file: "defa7.src",
            source_line: 1430,
            source_label: "PLE2",
            sound_number: RED_LABEL_PLAYER_END_SOUND_STOP_NUMBER,
            source: "https://github.com/mwenge/defender/blob/master/src/defa7.src#L1428-L1430",
        },
        RedLabelSoundDirectCommand {
            callsite: "lander_pull_tick",
            source_file: "defb6.src",
            source_line: 821,
            source_label: "LNDFX0",
            sound_number: RED_LABEL_LANDER_PULL_SOUND_NUMBER,
            source: "https://github.com/mwenge/defender/blob/master/src/defb6.src#L819-L821",
        },
    ]
}

pub(super) fn red_label_sound_direct_command_sequence_tsv() -> String {
    red_label_sound_direct_command_sequence_tsv_from_commands(&red_label_sound_direct_commands())
}

pub(super) fn red_label_sound_direct_command_sequence_fixture_check()
-> Result<RedLabelSoundDirectCommandSequenceFixtureCheck, String> {
    red_label_sound_direct_command_sequence_fixture_check_tsv(
        crate::assets::RED_LABEL_SOUND_DIRECT_COMMAND_SEQUENCES_TSV,
        &red_label_sound_direct_command_sequence_tsv(),
    )
}

pub(super) fn red_label_sound_direct_command_sequence_fixture_check_tsv(
    expected_tsv: &str,
    actual_tsv: &str,
) -> Result<RedLabelSoundDirectCommandSequenceFixtureCheck, String> {
    if actual_tsv != expected_tsv {
        return Err(String::from(
            "red-label direct command-sequence fixture does not match generated SNDOUT writes",
        ));
    }

    let mut lines = expected_tsv.lines();
    match lines.next() {
        Some(RED_LABEL_SOUND_DIRECT_COMMAND_SEQUENCE_TSV_HEADER) => {}
        _ => {
            return Err(String::from(
                "red-label direct command-sequence fixture header is invalid",
            ));
        }
    }

    let mut check = RedLabelSoundDirectCommandSequenceFixtureCheck {
        row_count: 0,
        idle_rows: 0,
        command_rows: 0,
    };
    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 2;
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 10 {
            return Err(format!(
                "red-label direct command-sequence fixture line {line_number} must have 10 columns"
            ));
        }

        check.row_count += 1;
        match columns[6] {
            "idle" => check.idle_rows += 1,
            "command" => check.command_rows += 1,
            write_kind => {
                return Err(format!(
                    "red-label direct command-sequence fixture line {line_number} has unknown write kind `{write_kind}`"
                ));
            }
        }
    }

    if check.row_count == 0 {
        return Err(String::from(
            "red-label direct command-sequence fixture has no data rows",
        ));
    }

    Ok(check)
}

pub(super) fn red_label_sound_thrust_command_sequence_tsv() -> String {
    red_label_sound_thrust_command_sequence_tsv_from_gates(&[
        RedLabelSoundThrustGateCommand {
            gate_event: "thrust_start",
            source_label: "SNDS01",
            status_block_mask: Some(RED_LABEL_SOUND_PLAYER_ALIVE_BLOCK_MASK),
            thrust_flag_before: 0x00,
            thrust_flag_after: RED_LABEL_THRUST_SOUND_START_NUMBER,
            sound_number: RED_LABEL_THRUST_SOUND_START_NUMBER,
        },
        RedLabelSoundThrustGateCommand {
            gate_event: "thrust_stop",
            source_label: "SNDS00",
            status_block_mask: None,
            thrust_flag_before: RED_LABEL_THRUST_SOUND_START_NUMBER,
            thrust_flag_after: 0x00,
            sound_number: RED_LABEL_THRUST_SOUND_STOP_NUMBER,
        },
    ])
}

pub(super) fn red_label_sound_thrust_command_sequence_fixture_check()
-> Result<RedLabelSoundThrustCommandSequenceFixtureCheck, String> {
    red_label_sound_thrust_command_sequence_fixture_check_tsv(
        crate::assets::RED_LABEL_SOUND_THRUST_COMMAND_SEQUENCES_TSV,
        &red_label_sound_thrust_command_sequence_tsv(),
    )
}

pub(super) fn red_label_sound_thrust_command_sequence_fixture_check_tsv(
    expected_tsv: &str,
    actual_tsv: &str,
) -> Result<RedLabelSoundThrustCommandSequenceFixtureCheck, String> {
    if actual_tsv != expected_tsv {
        return Err(String::from(
            "red-label thrust command-sequence fixture does not match generated SNDOUT writes",
        ));
    }

    let mut lines = expected_tsv.lines();
    match lines.next() {
        Some(RED_LABEL_SOUND_THRUST_COMMAND_SEQUENCE_TSV_HEADER) => {}
        _ => {
            return Err(String::from(
                "red-label thrust command-sequence fixture header is invalid",
            ));
        }
    }

    let mut check = RedLabelSoundThrustCommandSequenceFixtureCheck {
        row_count: 0,
        idle_rows: 0,
        command_rows: 0,
    };
    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 2;
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 11 {
            return Err(format!(
                "red-label thrust command-sequence fixture line {line_number} must have 11 columns"
            ));
        }

        check.row_count += 1;
        match columns[8] {
            "idle" => check.idle_rows += 1,
            "command" => check.command_rows += 1,
            write_kind => {
                return Err(format!(
                    "red-label thrust command-sequence fixture line {line_number} has unknown write kind `{write_kind}`"
                ));
            }
        }
    }

    if check.row_count == 0 {
        return Err(String::from(
            "red-label thrust command-sequence fixture has no data rows",
        ));
    }

    Ok(check)
}

pub(super) fn red_label_sound_table_command_sequence_fixture_check()
-> Result<RedLabelSoundTableCommandSequenceFixtureCheck, String> {
    red_label_sound_table_command_sequence_fixture_check_tsv(
        crate::assets::RED_LABEL_SOUND_TABLE_COMMAND_SEQUENCES_TSV,
        &red_label_sound_table_command_sequence_tsv()?,
    )
}

pub(super) fn red_label_sound_table_command_sequence_fixture_check_tsv(
    expected_tsv: &str,
    actual_tsv: &str,
) -> Result<RedLabelSoundTableCommandSequenceFixtureCheck, String> {
    if actual_tsv != expected_tsv {
        return Err(String::from(
            "red-label sound table command-sequence fixture does not match generated SNDOUT writes",
        ));
    }

    let mut lines = expected_tsv.lines();
    match lines.next() {
        Some(RED_LABEL_SOUND_TABLE_COMMAND_SEQUENCE_TSV_HEADER) => {}
        _ => {
            return Err(String::from(
                "red-label sound table command-sequence fixture header is invalid",
            ));
        }
    }

    let mut check = RedLabelSoundTableCommandSequenceFixtureCheck {
        row_count: 0,
        idle_rows: 0,
        command_rows: 0,
    };
    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 2;
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 13 {
            return Err(format!(
                "red-label sound table command-sequence fixture line {line_number} must have 13 columns"
            ));
        }

        check.row_count += 1;
        match columns[5] {
            "idle" => check.idle_rows += 1,
            "command" => check.command_rows += 1,
            write_kind => {
                return Err(format!(
                    "red-label sound table command-sequence fixture line {line_number} has unknown write kind `{write_kind}`"
                ));
            }
        }
    }

    if check.row_count == 0 {
        return Err(String::from(
            "red-label sound table command-sequence fixture has no data rows",
        ));
    }

    Ok(check)
}

pub(super) fn red_label_sound_table_timeline_fixture_check()
-> Result<RedLabelSoundTableTimelineFixtureCheck, String> {
    red_label_sound_table_timeline_fixture_check_tsv(
        crate::assets::RED_LABEL_SOUND_TABLE_TIMELINES_TSV,
        &red_label_sound_table_timeline_tsv()?,
    )
}

pub(super) fn red_label_sound_table_timeline_fixture_check_tsv(
    expected_tsv: &str,
    actual_tsv: &str,
) -> Result<RedLabelSoundTableTimelineFixtureCheck, String> {
    if actual_tsv != expected_tsv {
        return Err(String::from(
            "red-label sound table timeline fixture does not match generated SNDSEQ timeline",
        ));
    }

    let mut lines = expected_tsv.lines();
    match lines.next() {
        Some(RED_LABEL_SOUND_TABLE_TIMELINE_TSV_HEADER) => {}
        _ => {
            return Err(String::from(
                "red-label sound table timeline fixture header is invalid",
            ));
        }
    }

    let mut check = RedLabelSoundTableTimelineFixtureCheck {
        row_count: 0,
        command_rows: 0,
        sequence_end_rows: 0,
    };
    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 2;
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 16 {
            return Err(format!(
                "red-label sound table timeline fixture line {line_number} must have 16 columns"
            ));
        }

        check.row_count += 1;
        match columns[4] {
            "command" => check.command_rows += 1,
            "sequence_end" => check.sequence_end_rows += 1,
            event => {
                return Err(format!(
                    "red-label sound table timeline fixture line {line_number} has unknown event `{event}`"
                ));
            }
        }
    }

    if check.row_count == 0 {
        return Err(String::from(
            "red-label sound table timeline fixture has no data rows",
        ));
    }

    Ok(check)
}

pub(super) fn red_label_sound_table_timeline_tsv_from_timelines(
    timelines: &[RedLabelSoundTableTimeline],
) -> String {
    let mut text = String::from(RED_LABEL_SOUND_TABLE_TIMELINE_TSV_HEADER);
    text.push('\n');

    for timeline in timelines {
        for timed in &timeline.commands {
            let command = timed.command;
            let output = command.sound_output;
            text.push_str(&format!(
                "{}\t0x{:04X}\t0x{:02X}\t{}\tcommand\t0x{:04X}\t{}\t{}\t0x{:02X}\t0x{:02X}\t0x{:02X}\t{}\t0x{:02X}\t{}\t{}\t0x{:04X}\n",
                timeline.label,
                timeline.address,
                timeline.priority,
                timed.sequencer_tick,
                command.table_pointer,
                command.repeat_index,
                command.repeat_count,
                command.timer,
                command.sound_number,
                output.idle_port_b,
                output.idle_command.hex(),
                output.command_port_b,
                command.command.hex(),
                timeline.sequence_end_tick,
                timeline.terminator_pointer
            ));
        }

        text.push_str(&format!(
            "{}\t0x{:04X}\t0x{:02X}\t{}\tsequence_end\t0x{:04X}\t-\t-\t-\t-\t-\t-\t-\t-\t{}\t0x{:04X}\n",
            timeline.label,
            timeline.address,
            timeline.priority,
            timeline.sequence_end_tick,
            timeline.terminator_pointer,
            timeline.sequence_end_tick,
            timeline.terminator_pointer
        ));
    }

    text
}

pub(super) fn red_label_sound_table_command_sequence_tsv_from_timelines(
    timelines: &[RedLabelSoundTableTimeline],
) -> String {
    let mut text = String::from(RED_LABEL_SOUND_TABLE_COMMAND_SEQUENCE_TSV_HEADER);
    text.push('\n');

    for timeline in timelines {
        for timed in &timeline.commands {
            let command = timed.command;
            let output = command.sound_output;
            text.push_str(&format!(
                "{}\t0x{:04X}\t0x{:02X}\t{}\t1\tidle\t0x{:04X}\t{}\t{}\t0x{:02X}\t0x{:02X}\t0x{:02X}\t{}\n",
                timeline.label,
                timeline.address,
                timeline.priority,
                timed.sequencer_tick,
                command.table_pointer,
                command.repeat_index,
                command.repeat_count,
                command.timer,
                command.sound_number,
                output.idle_port_b,
                output.idle_command.hex()
            ));
            text.push_str(&format!(
                "{}\t0x{:04X}\t0x{:02X}\t{}\t2\tcommand\t0x{:04X}\t{}\t{}\t0x{:02X}\t0x{:02X}\t0x{:02X}\t{}\n",
                timeline.label,
                timeline.address,
                timeline.priority,
                timed.sequencer_tick,
                command.table_pointer,
                command.repeat_index,
                command.repeat_count,
                command.timer,
                command.sound_number,
                output.command_port_b,
                command.command.hex()
            ));
        }
    }

    text
}

pub(super) fn red_label_sound_direct_command_sequence_tsv_from_commands(
    commands: &[RedLabelSoundDirectCommand],
) -> String {
    let mut text = String::from(RED_LABEL_SOUND_DIRECT_COMMAND_SEQUENCE_TSV_HEADER);
    text.push('\n');

    for command in commands {
        let output = red_label_sound_output(command.sound_number);
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\t0x{:02X}\t1\tidle\t0x{:02X}\t{}\t{}\n",
            command.callsite,
            command.source_file,
            command.source_line,
            command.source_label,
            command.sound_number,
            output.idle_port_b,
            output.idle_command.hex(),
            command.source
        ));
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\t0x{:02X}\t2\tcommand\t0x{:02X}\t{}\t{}\n",
            command.callsite,
            command.source_file,
            command.source_line,
            command.source_label,
            command.sound_number,
            output.command_port_b,
            output.command.hex(),
            command.source
        ));
    }

    text
}

pub(super) fn red_label_sound_thrust_command_sequence_tsv_from_gates(
    gates: &[RedLabelSoundThrustGateCommand],
) -> String {
    let mut text = String::from(RED_LABEL_SOUND_THRUST_COMMAND_SEQUENCE_TSV_HEADER);
    text.push('\n');

    for gate in gates {
        let output = red_label_sound_output(gate.sound_number);
        let status_block_mask = gate
            .status_block_mask
            .map(|mask| format!("0x{mask:02X}"))
            .unwrap_or_else(|| String::from("-"));
        text.push_str(&format!(
            "{}\t{}\t0x{RED_LABEL_THRUST_SWITCH_BIT:02X}\t{status_block_mask}\t0x{:02X}\t0x{:02X}\t0x{:02X}\t1\tidle\t0x{:02X}\t{}\n",
            gate.gate_event,
            gate.source_label,
            gate.thrust_flag_before,
            gate.thrust_flag_after,
            gate.sound_number,
            output.idle_port_b,
            output.idle_command.hex()
        ));
        text.push_str(&format!(
            "{}\t{}\t0x{RED_LABEL_THRUST_SWITCH_BIT:02X}\t{status_block_mask}\t0x{:02X}\t0x{:02X}\t0x{:02X}\t2\tcommand\t0x{:02X}\t{}\n",
            gate.gate_event,
            gate.source_label,
            gate.thrust_flag_before,
            gate.thrust_flag_after,
            gate.sound_number,
            output.command_port_b,
            output.command.hex()
        ));
    }

    text
}

pub(super) fn red_label_sound_table_commands(
    table: &RedLabelSoundTable,
) -> Result<Vec<RedLabelSoundTableCommand>, String> {
    if table.bytes.is_empty() {
        return Err(format!("red-label sound table `{}` is empty", table.label));
    }

    let mut commands = Vec::new();
    let mut offset = 1usize;
    loop {
        let repeat_count = *table.bytes.get(offset).ok_or_else(|| {
            format!(
                "red-label sound table `{}` has no terminator after command records",
                table.label
            )
        })?;
        if repeat_count == 0 {
            return Ok(commands);
        }

        let timer = *table.bytes.get(offset + 1).ok_or_else(|| {
            format!(
                "red-label sound table `{}` record at 0x{:04X} has no timer byte",
                table.label,
                table.address.wrapping_add(offset as u16)
            )
        })?;
        let sound_number = *table.bytes.get(offset + 2).ok_or_else(|| {
            format!(
                "red-label sound table `{}` record at 0x{:04X} has no sound-number byte",
                table.label,
                table.address.wrapping_add(offset as u16)
            )
        })?;
        let table_pointer = table.address.wrapping_add(offset as u16);
        let sound_output = red_label_sound_output(sound_number);
        for repeat_index in 1..=repeat_count {
            commands.push(RedLabelSoundTableCommand {
                table_pointer,
                repeat_index,
                repeat_count,
                timer,
                sound_number,
                sound_output,
                command: sound_output.command,
            });
        }
        offset += 3;
    }
}

pub(super) fn red_label_sound_table_timed_commands(
    table: &RedLabelSoundTable,
) -> Result<Vec<RedLabelSoundTableTimedCommand>, String> {
    let mut sequencer_tick = 1u32;
    let mut timed_commands = Vec::new();
    for command in red_label_sound_table_commands(table)? {
        if command.timer == 0 {
            return Err(format!(
                "red-label sound table `{}` record at 0x{:04X} has zero timer byte",
                table.label, command.table_pointer
            ));
        }
        timed_commands.push(RedLabelSoundTableTimedCommand {
            sequencer_tick,
            command,
        });
        sequencer_tick += u32::from(command.timer);
    }
    Ok(timed_commands)
}

pub(super) fn red_label_sound_table_timeline_for_table(
    table: &RedLabelSoundTable,
) -> Result<RedLabelSoundTableTimeline, String> {
    let priority = *table
        .bytes
        .first()
        .ok_or_else(|| format!("red-label sound table `{}` is empty", table.label))?;
    let commands = red_label_sound_table_timed_commands(table)?;
    let last_command = commands.last().ok_or_else(|| {
        format!(
            "red-label sound table `{}` has no command records",
            table.label
        )
    })?;
    let sequence_end_tick = last_command
        .sequencer_tick
        .wrapping_add(u32::from(last_command.command.timer));
    let terminator_pointer = last_command.command.table_pointer.wrapping_add(3);
    Ok(RedLabelSoundTableTimeline {
        label: table.label.clone(),
        address: table.address,
        priority,
        commands,
        sequence_end_tick,
        terminator_pointer,
    })
}

pub(super) fn red_label_sound_priority_allows_load(priority: u8, current_priority: u8) -> bool {
    priority >= current_priority
}

pub(super) fn red_label_sound_tables() -> Result<&'static [RedLabelSoundTable], String> {
    static SOUND_TABLES: OnceLock<Result<Vec<RedLabelSoundTable>, String>> = OnceLock::new();
    match SOUND_TABLES.get_or_init(|| parse_sound_tables(crate::assets::RED_LABEL_SOUND_TABLES_TSV))
    {
        Ok(tables) => Ok(tables.as_slice()),
        Err(error) => Err(error.clone()),
    }
}

pub(super) fn parse_sound_tables(text: &'static str) -> Result<Vec<RedLabelSoundTable>, String> {
    let mut lines = text.lines();
    match lines.next() {
        Some("label\taddress\tbytes\tsource") => {}
        _ => {
            return Err(String::from(
                "red-label sound table asset header is invalid",
            ));
        }
    }

    let mut tables = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 2;
        if line.trim().is_empty() {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        if columns.len() != 4 {
            return Err(format!(
                "red-label sound table line {line_number} must have 4 columns"
            ));
        }
        let label = columns[0];
        if label.is_empty() {
            return Err(format!(
                "red-label sound table line {line_number} has an empty label"
            ));
        }
        if columns[3].is_empty() {
            return Err(format!(
                "red-label sound table line {line_number} has an empty source"
            ));
        }
        tables.push(RedLabelSoundTable {
            label: String::from(label),
            address: parse_asset_hex_u16("sound table address", columns[1], line_number)?,
            bytes: parse_hex_pairs("sound table bytes", columns[2], line_number)?,
        });
    }

    if tables.is_empty() {
        return Err(String::from("red-label sound table asset has no records"));
    }
    Ok(tables)
}

/// Source-shaped `defa7.src` `SNDOUT`: write the idle `0x3f` byte to `SOUND`,
/// then write the complemented six-bit sound number. MAME ORs both port-B
/// values with `0xc0`; only the second write asserts sound-board CB1.
/// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L697-L707>.
pub(super) fn red_label_sound_output(sound_number: u8) -> RedLabelSoundOutput {
    let command_port_b = !sound_number & 0x3F;
    RedLabelSoundOutput {
        sound_number,
        idle_port_b: RED_LABEL_SNDOUT_IDLE_PORT_B,
        idle_command: SoundCommand::from_main_board_pia_port_b(RED_LABEL_SNDOUT_IDLE_PORT_B),
        command_port_b,
        command: SoundCommand::from_main_board_pia_port_b(command_port_b),
    }
}

pub(super) fn red_label_sound_output_command(sound_number: u8) -> SoundCommand {
    red_label_sound_output(sound_number).command
}
