//! Embedded red-label memory-map and RAM-layout metadata.
//!
//! The source asset records the MAME-documented Defender main-board and
//! sound-board address ranges used by the Rust address classifiers. It also
//! records source-owned `phr6.src` RAM table labels and `romc8.src` CMOS
//! default bytes so runtime code can address player, object, process,
//! shell-list, and operator/default cells without inventing Rust-only layouts.

use std::ops::Range;

use crate::assets::{
    RED_LABEL_CMOS_DEFAULTS_TSV, RED_LABEL_CMOS_LAYOUT_TSV, RED_LABEL_LINKED_LISTS_TSV,
    RED_LABEL_MEMORY_MAP_TSV, RED_LABEL_RAM_LAYOUT_TSV, RED_LABEL_SRAM_ROUTINES_TSV,
};

pub const RED_LABEL_CMOS_BASE: u16 = 0xC400;
pub const RED_LABEL_CMOS_CELLS: u16 = 0x0100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMapCpu {
    Main,
    Sound,
}

impl MemoryMapCpu {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "main" => Ok(Self::Main),
            "sound" => Ok(Self::Sound),
            other => Err(format!("unknown memory map CPU '{other}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelMemoryMapEntry {
    pub cpu: MemoryMapCpu,
    pub start: u16,
    pub end: u16,
    pub access: String,
    pub bank_select: String,
    pub mirror_mask: Option<u16>,
    pub handler: String,
    pub source: String,
}

impl RedLabelMemoryMapEntry {
    pub fn contains(&self, address: u16) -> bool {
        (self.start..=self.end).contains(&address)
    }

    pub fn mirrored_offset(&self, address: u16) -> Option<u16> {
        let canonical = match self.mirror_mask {
            Some(mask) => address & !mask,
            None => address,
        };
        self.contains(canonical).then_some(canonical - self.start)
    }
}

pub fn red_label_memory_map() -> Result<Vec<RedLabelMemoryMapEntry>, String> {
    parse_red_label_memory_map_tsv(RED_LABEL_MEMORY_MAP_TSV)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelSramRoutine {
    pub routine: String,
    pub address: u16,
    pub register: String,
    pub width_nibbles: u8,
    pub x_advance: u8,
    pub operation: String,
    pub source: String,
}

pub fn red_label_sram_routines() -> Result<Vec<RedLabelSramRoutine>, String> {
    parse_red_label_sram_routines_tsv(RED_LABEL_SRAM_ROUTINES_TSV)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCmosLayoutEntry {
    pub symbol: String,
    pub address: u16,
    pub offset: u16,
    pub cells: u16,
    pub description: String,
    pub source: String,
}

impl RedLabelCmosLayoutEntry {
    pub fn address_range(&self) -> Option<Range<u16>> {
        checked_range(self.address, u32::from(self.cells))
    }

    pub fn offset_range(&self) -> Option<Range<u16>> {
        checked_range(self.offset, u32::from(self.cells))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelCmosDefault {
    pub symbol: String,
    pub offset: u16,
    pub cells: u16,
    pub bytes: Vec<u8>,
    pub description: String,
    pub source: String,
}

impl RedLabelCmosDefault {
    pub fn cell_range(&self) -> Option<Range<u16>> {
        checked_range(self.offset, u32::from(self.cells))
    }

    pub fn encoded_cells(&self) -> Vec<u8> {
        let mut cells = Vec::with_capacity(usize::from(self.cells));
        for byte in &self.bytes {
            let (ms_nibble, ls_nibble) = unpack_sram_byte(*byte);
            cells.push(cmos_4bit_cell_value(ms_nibble));
            cells.push(cmos_4bit_cell_value(ls_nibble));
        }
        cells
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelRamLayoutEntry {
    pub table: String,
    pub base: u16,
    pub entry_size: u16,
    pub entries: u16,
    pub field: String,
    pub offset: u16,
    pub size: u16,
    pub source: String,
}

impl RedLabelRamLayoutEntry {
    pub fn table_range(&self) -> Option<Range<u16>> {
        checked_range(
            self.base,
            u32::from(self.entry_size) * u32::from(self.entries),
        )
    }

    pub fn field_range_for_entry(&self, entry_index: u16) -> Option<Range<u16>> {
        if entry_index >= self.entries {
            return None;
        }

        let relative_start =
            u32::from(entry_index) * u32::from(self.entry_size) + u32::from(self.offset);
        let start = u32::from(self.base) + relative_start;
        checked_range_from_u32(start, u32::from(self.size))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedLabelLinkedList {
    pub list: String,
    pub head_symbol: String,
    pub head_address: u16,
    pub entry_table: String,
    pub link_field: String,
    pub source: String,
}

pub fn red_label_ram_layout() -> Result<Vec<RedLabelRamLayoutEntry>, String> {
    parse_red_label_ram_layout_tsv(RED_LABEL_RAM_LAYOUT_TSV)
}

pub fn red_label_linked_lists() -> Result<Vec<RedLabelLinkedList>, String> {
    parse_red_label_linked_lists_tsv(RED_LABEL_LINKED_LISTS_TSV)
}

pub fn red_label_cmos_layout() -> Result<Vec<RedLabelCmosLayoutEntry>, String> {
    parse_red_label_cmos_layout_tsv(RED_LABEL_CMOS_LAYOUT_TSV)
}

pub fn red_label_cmos_defaults() -> Result<Vec<RedLabelCmosDefault>, String> {
    parse_red_label_cmos_defaults_tsv(RED_LABEL_CMOS_DEFAULTS_TSV)
}

pub fn cmos_4bit_cell_value(value: u8) -> u8 {
    value | 0xF0
}

pub fn pack_sram_byte(ms_nibble: u8, ls_nibble: u8) -> u8 {
    ((ms_nibble & 0x0F) << 4) | (ls_nibble & 0x0F)
}

pub fn unpack_sram_byte(value: u8) -> (u8, u8) {
    ((value >> 4) & 0x0F, value & 0x0F)
}

pub fn pack_sram_word(nibbles: [u8; 4]) -> u16 {
    (u16::from(nibbles[0] & 0x0F) << 12)
        | (u16::from(nibbles[1] & 0x0F) << 8)
        | (u16::from(nibbles[2] & 0x0F) << 4)
        | u16::from(nibbles[3] & 0x0F)
}

pub fn unpack_sram_word(value: u16) -> [u8; 4] {
    [
        ((value >> 12) & 0x0F) as u8,
        ((value >> 8) & 0x0F) as u8,
        ((value >> 4) & 0x0F) as u8,
        (value & 0x0F) as u8,
    ]
}

pub fn parse_red_label_memory_map_tsv(text: &str) -> Result<Vec<RedLabelMemoryMapEntry>, String> {
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Err(String::from("memory map TSV is empty"));
    };
    if header != "cpu\tstart\tend\taccess\tbank_select\tmirror_mask\thandler\tsource" {
        return Err(format!("unexpected memory map header '{header}'"));
    }

    let mut entries = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        entries.push(parse_memory_map_line(index + 2, line)?);
    }
    if entries.is_empty() {
        return Err(String::from("memory map TSV has no entries"));
    }

    Ok(entries)
}

pub fn parse_red_label_cmos_layout_tsv(text: &str) -> Result<Vec<RedLabelCmosLayoutEntry>, String> {
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Err(String::from("CMOS layout TSV is empty"));
    };
    if header != "symbol\taddress\toffset\tcells\tdescription\tsource" {
        return Err(format!("unexpected CMOS layout header '{header}'"));
    }

    let mut entries = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        entries.push(parse_cmos_layout_line(index + 2, line)?);
    }
    if entries.is_empty() {
        return Err(String::from("CMOS layout TSV has no entries"));
    }

    Ok(entries)
}

pub fn parse_red_label_cmos_defaults_tsv(text: &str) -> Result<Vec<RedLabelCmosDefault>, String> {
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Err(String::from("CMOS defaults TSV is empty"));
    };
    if header != "symbol\toffset\tcells\tbytes\tdescription\tsource" {
        return Err(format!("unexpected CMOS defaults header '{header}'"));
    }

    let mut defaults = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        defaults.push(parse_cmos_default_line(index + 2, line)?);
    }
    if defaults.is_empty() {
        return Err(String::from("CMOS defaults TSV has no entries"));
    }

    Ok(defaults)
}

pub fn parse_red_label_sram_routines_tsv(text: &str) -> Result<Vec<RedLabelSramRoutine>, String> {
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Err(String::from("SRAM routines TSV is empty"));
    };
    if header != "routine\taddress\tregister\twidth_nibbles\tx_advance\toperation\tsource" {
        return Err(format!("unexpected SRAM routines header '{header}'"));
    }

    let mut routines = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        routines.push(parse_sram_routine_line(index + 2, line)?);
    }
    if routines.is_empty() {
        return Err(String::from("SRAM routines TSV has no entries"));
    }

    Ok(routines)
}

pub fn parse_red_label_ram_layout_tsv(text: &str) -> Result<Vec<RedLabelRamLayoutEntry>, String> {
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Err(String::from("RAM layout TSV is empty"));
    };
    if header != "table\tbase\tentry_size\tentries\tfield\toffset\tsize\tsource" {
        return Err(format!("unexpected RAM layout header '{header}'"));
    }

    let mut entries = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        entries.push(parse_ram_layout_line(index + 2, line)?);
    }
    if entries.is_empty() {
        return Err(String::from("RAM layout TSV has no entries"));
    }

    Ok(entries)
}

pub fn parse_red_label_linked_lists_tsv(text: &str) -> Result<Vec<RedLabelLinkedList>, String> {
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Err(String::from("linked lists TSV is empty"));
    };
    if header != "list\thead_symbol\thead_address\tentry_table\tlink_field\tsource" {
        return Err(format!("unexpected linked lists header '{header}'"));
    }

    let mut lists = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        lists.push(parse_linked_list_line(index + 2, line)?);
    }
    if lists.is_empty() {
        return Err(String::from("linked lists TSV has no entries"));
    }

    Ok(lists)
}

fn parse_memory_map_line(line_number: usize, line: &str) -> Result<RedLabelMemoryMapEntry, String> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 8 {
        return Err(format!(
            "memory map line {line_number} has {} fields, expected 8",
            fields.len()
        ));
    }

    let start = parse_hex_u16(line_number, "start", fields[1])?;
    let end = parse_hex_u16(line_number, "end", fields[2])?;
    if end < start {
        return Err(format!(
            "memory map line {line_number} end {end:#06X} precedes start {start:#06X}"
        ));
    }

    Ok(RedLabelMemoryMapEntry {
        cpu: MemoryMapCpu::parse(fields[0])?,
        start,
        end,
        access: String::from(fields[3]),
        bank_select: String::from(fields[4]),
        mirror_mask: parse_optional_hex_u16(line_number, "mirror_mask", fields[5])?,
        handler: String::from(fields[6]),
        source: String::from(fields[7]),
    })
}

fn parse_cmos_layout_line(
    line_number: usize,
    line: &str,
) -> Result<RedLabelCmosLayoutEntry, String> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 6 {
        return Err(format!(
            "CMOS layout line {line_number} has {} fields, expected 6",
            fields.len()
        ));
    }

    let address = parse_context_hex_u16("CMOS layout", line_number, "address", fields[1])?;
    let offset = parse_context_hex_u16("CMOS layout", line_number, "offset", fields[2])?;
    let cells = parse_u16(line_number, "cells", fields[3])?;

    if address < RED_LABEL_CMOS_BASE {
        return Err(format!(
            "CMOS layout line {line_number} address {address:#06X} precedes CMOS base"
        ));
    }
    if address - RED_LABEL_CMOS_BASE != offset {
        return Err(format!(
            "CMOS layout line {line_number} address/offset mismatch"
        ));
    }
    if checked_range(address, u32::from(cells)).is_none() {
        return Err(format!("CMOS layout line {line_number} range overflows"));
    }

    Ok(RedLabelCmosLayoutEntry {
        symbol: String::from(fields[0]),
        address,
        offset,
        cells,
        description: String::from(fields[4]),
        source: String::from(fields[5]),
    })
}

fn parse_cmos_default_line(line_number: usize, line: &str) -> Result<RedLabelCmosDefault, String> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 6 {
        return Err(format!(
            "CMOS defaults line {line_number} has {} fields, expected 6",
            fields.len()
        ));
    }

    let offset = parse_context_hex_u16("CMOS defaults", line_number, "offset", fields[1])?;
    let cells = parse_u16(line_number, "cells", fields[2])?;
    let bytes = parse_hex_byte_list(line_number, "bytes", fields[3])?;
    let expected_cells = bytes.len() * 2;
    if usize::from(cells) != expected_cells {
        return Err(format!(
            "CMOS defaults line {line_number} cell count {cells} does not match {} encoded cells",
            expected_cells
        ));
    }
    if u32::from(offset) + u32::from(cells) > u32::from(RED_LABEL_CMOS_CELLS) {
        return Err(format!(
            "CMOS defaults line {line_number} range exceeds CMOS cell space"
        ));
    }

    Ok(RedLabelCmosDefault {
        symbol: String::from(fields[0]),
        offset,
        cells,
        bytes,
        description: String::from(fields[4]),
        source: String::from(fields[5]),
    })
}

fn parse_ram_layout_line(line_number: usize, line: &str) -> Result<RedLabelRamLayoutEntry, String> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 8 {
        return Err(format!(
            "RAM layout line {line_number} has {} fields, expected 8",
            fields.len()
        ));
    }

    let base = parse_context_hex_u16("RAM layout", line_number, "base", fields[1])?;
    let entry_size = parse_context_hex_u16("RAM layout", line_number, "entry_size", fields[2])?;
    let entries = parse_u16(line_number, "entries", fields[3])?;
    let offset = parse_context_hex_u16("RAM layout", line_number, "offset", fields[5])?;
    let size = parse_context_hex_u16("RAM layout", line_number, "size", fields[6])?;

    if entry_size == 0 {
        return Err(format!("RAM layout line {line_number} entry_size is zero"));
    }
    if entries == 0 {
        return Err(format!("RAM layout line {line_number} entries is zero"));
    }
    if size == 0 {
        return Err(format!("RAM layout line {line_number} size is zero"));
    }
    if u32::from(offset) + u32::from(size) > u32::from(entry_size) {
        return Err(format!(
            "RAM layout line {line_number} field {} exceeds entry size",
            fields[4]
        ));
    }
    if checked_range(base, u32::from(entry_size) * u32::from(entries)).is_none() {
        return Err(format!(
            "RAM layout line {line_number} table range overflows"
        ));
    }

    Ok(RedLabelRamLayoutEntry {
        table: String::from(fields[0]),
        base,
        entry_size,
        entries,
        field: String::from(fields[4]),
        offset,
        size,
        source: String::from(fields[7]),
    })
}

fn parse_linked_list_line(line_number: usize, line: &str) -> Result<RedLabelLinkedList, String> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 6 {
        return Err(format!(
            "linked list line {line_number} has {} fields, expected 6",
            fields.len()
        ));
    }

    Ok(RedLabelLinkedList {
        list: String::from(fields[0]),
        head_symbol: String::from(fields[1]),
        head_address: parse_context_hex_u16("linked list", line_number, "head_address", fields[2])?,
        entry_table: String::from(fields[3]),
        link_field: String::from(fields[4]),
        source: String::from(fields[5]),
    })
}

fn parse_sram_routine_line(line_number: usize, line: &str) -> Result<RedLabelSramRoutine, String> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 7 {
        return Err(format!(
            "SRAM routine line {line_number} has {} fields, expected 7",
            fields.len()
        ));
    }

    Ok(RedLabelSramRoutine {
        routine: String::from(fields[0]),
        address: parse_hex_u16(line_number, "address", fields[1])?,
        register: String::from(fields[2]),
        width_nibbles: parse_u8(line_number, "width_nibbles", fields[3])?,
        x_advance: parse_u8(line_number, "x_advance", fields[4])?,
        operation: String::from(fields[5]),
        source: String::from(fields[6]),
    })
}

fn parse_optional_hex_u16(
    line_number: usize,
    field_name: &str,
    value: &str,
) -> Result<Option<u16>, String> {
    if value == "-" {
        Ok(None)
    } else {
        parse_hex_u16(line_number, field_name, value).map(Some)
    }
}

fn parse_hex_u16(line_number: usize, field_name: &str, value: &str) -> Result<u16, String> {
    parse_context_hex_u16("memory map", line_number, field_name, value)
}

fn parse_context_hex_u16(
    context: &str,
    line_number: usize,
    field_name: &str,
    value: &str,
) -> Result<u16, String> {
    let Some(hex) = value.strip_prefix("0x") else {
        return Err(format!(
            "{context} line {line_number} {field_name} '{value}' is not hex"
        ));
    };
    u16::from_str_radix(hex, 16).map_err(|error| {
        format!("{context} line {line_number} {field_name} '{value}' is invalid: {error}")
    })
}

fn parse_u16(line_number: usize, field_name: &str, value: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .map_err(|error| format!("line {line_number} {field_name} '{value}' is invalid: {error}"))
}

fn parse_u8(line_number: usize, field_name: &str, value: &str) -> Result<u8, String> {
    value
        .parse::<u8>()
        .map_err(|error| format!("line {line_number} {field_name} '{value}' is invalid: {error}"))
}

fn parse_hex_byte_list(
    line_number: usize,
    field_name: &str,
    value: &str,
) -> Result<Vec<u8>, String> {
    let bytes = value
        .split_whitespace()
        .map(|token| parse_hex_u8(line_number, field_name, token))
        .collect::<Result<Vec<_>, _>>()?;
    if bytes.is_empty() {
        return Err(format!("line {line_number} {field_name} is empty"));
    }
    Ok(bytes)
}

fn parse_hex_u8(line_number: usize, field_name: &str, value: &str) -> Result<u8, String> {
    let Some(hex) = value.strip_prefix("0x") else {
        return Err(format!(
            "line {line_number} {field_name} byte '{value}' is not hex"
        ));
    };
    u8::from_str_radix(hex, 16).map_err(|error| {
        format!("line {line_number} {field_name} byte '{value}' is invalid: {error}")
    })
}

fn checked_range(start: u16, len: u32) -> Option<Range<u16>> {
    checked_range_from_u32(u32::from(start), len)
}

fn checked_range_from_u32(start: u32, len: u32) -> Option<Range<u16>> {
    let end = start.checked_add(len)?;
    if end > u32::from(u16::MAX) {
        return None;
    }
    Some(start as u16..end as u16)
}

#[cfg(test)]
mod tests {
    use super::{
        MemoryMapCpu, RED_LABEL_CMOS_BASE, RED_LABEL_CMOS_CELLS, cmos_4bit_cell_value,
        pack_sram_byte, pack_sram_word, parse_red_label_cmos_defaults_tsv,
        parse_red_label_cmos_layout_tsv, parse_red_label_linked_lists_tsv,
        parse_red_label_memory_map_tsv, parse_red_label_ram_layout_tsv,
        parse_red_label_sram_routines_tsv, red_label_cmos_defaults, red_label_cmos_layout,
        red_label_linked_lists, red_label_memory_map, red_label_ram_layout,
        red_label_sram_routines, unpack_sram_byte, unpack_sram_word,
    };

    #[test]
    fn embedded_memory_map_contains_main_and_sound_boundaries() {
        let entries = red_label_memory_map().expect("memory map parses");

        assert!(entries.iter().any(|entry| {
            entry.cpu == MemoryMapCpu::Main
                && entry.handler == "ram"
                && entry.start == 0x0000
                && entry.end == 0xBFFF
        }));
        assert!(entries.iter().any(|entry| {
            entry.cpu == MemoryMapCpu::Main
                && entry.handler == "fixed_rom"
                && entry.start == 0xD000
                && entry.end == 0xFFFF
        }));
        assert!(entries.iter().any(|entry| {
            entry.cpu == MemoryMapCpu::Sound
                && entry.handler == "pia_ic4"
                && entry.mirror_mask == Some(0x8000)
        }));
    }

    #[test]
    fn memory_map_entry_reports_mirrored_offsets() {
        let entries = red_label_memory_map().expect("memory map parses");
        let pia = entries
            .iter()
            .find(|entry| entry.handler == "pia0")
            .expect("PIA0 entry");

        assert_eq!(pia.mirrored_offset(0xCC00), Some(0));
        assert_eq!(pia.mirrored_offset(0xCFE3), Some(3));
        assert_eq!(pia.mirrored_offset(0xCC04), None);
    }

    #[test]
    fn memory_map_parser_rejects_bad_header() {
        let error = parse_red_label_memory_map_tsv("wat\n").expect_err("bad header");
        assert!(error.contains("unexpected memory map header"));
    }

    #[test]
    fn memory_map_parser_rejects_bad_hex_and_ranges() {
        let text = "cpu\tstart\tend\taccess\tbank_select\tmirror_mask\thandler\tsource\nmain\t0\t0xBFFF\trw\tany\t-\tram\tMAME\n";
        let error = parse_red_label_memory_map_tsv(text).expect_err("bad hex");
        assert!(error.contains("is not hex"));

        let text = "cpu\tstart\tend\taccess\tbank_select\tmirror_mask\thandler\tsource\nmain\t0xBFFF\t0x0000\trw\tany\t-\tram\tMAME\n";
        let error = parse_red_label_memory_map_tsv(text).expect_err("bad range");
        assert!(error.contains("precedes start"));
    }

    #[test]
    fn embedded_sram_routines_capture_packed_nibble_contract() {
        let routines = red_label_sram_routines().expect("SRAM routines parse");

        assert!(routines.iter().any(|routine| {
            routine.routine == "SRAMRead"
                && routine.address == 0xF813
                && routine.register == "A"
                && routine.width_nibbles == 2
                && routine.x_advance == 2
                && routine.operation == "read_byte"
        }));
        assert!(routines.iter().any(|routine| {
            routine.routine == "SRAMWordRd"
                && routine.address == 0xF838
                && routine.register == "D"
                && routine.width_nibbles == 4
                && routine.x_advance == 4
                && routine.operation == "read_word"
        }));
        assert!(routines.iter().any(|routine| {
            routine.routine == "SRAMWrite"
                && routine.address == 0xF842
                && routine.operation == "write_byte"
        }));
    }

    #[test]
    fn sram_pack_helpers_follow_most_significant_nibble_first() {
        assert_eq!(pack_sram_byte(0x0A, 0x05), 0xA5);
        assert_eq!(pack_sram_byte(0xFA, 0xF5), 0xA5);
        assert_eq!(unpack_sram_byte(0xA5), (0x0A, 0x05));

        assert_eq!(pack_sram_word([0x01, 0x02, 0x03, 0x04]), 0x1234);
        assert_eq!(pack_sram_word([0xF1, 0xF2, 0xF3, 0xF4]), 0x1234);
        assert_eq!(unpack_sram_word(0x1234), [0x01, 0x02, 0x03, 0x04]);
        assert_eq!(cmos_4bit_cell_value(0x0A), 0xFA);
    }

    #[test]
    fn sram_routine_parser_rejects_bad_header_and_numbers() {
        let error = parse_red_label_sram_routines_tsv("wat\n").expect_err("bad header");
        assert!(error.contains("unexpected SRAM routines header"));

        let text = "routine\taddress\tregister\twidth_nibbles\tx_advance\toperation\tsource\nSRAMRead\tF813\tA\t2\t2\tread_byte\tComputer Archeology\n";
        let error = parse_red_label_sram_routines_tsv(text).expect_err("bad address");
        assert!(error.contains("is not hex"));

        let text = "routine\taddress\tregister\twidth_nibbles\tx_advance\toperation\tsource\nSRAMRead\t0xF813\tA\twat\t2\tread_byte\tComputer Archeology\n";
        let error = parse_red_label_sram_routines_tsv(text).expect_err("bad width");
        assert!(error.contains("width_nibbles"));
    }

    #[test]
    fn embedded_cmos_layout_contains_source_owned_cells_and_markers() {
        let entries = red_label_cmos_layout().expect("CMOS layout parses");

        let dip_flag = entries
            .iter()
            .find(|entry| entry.symbol == "DIPFLG")
            .expect("DIPFLG entry");
        assert_eq!(dip_flag.address, RED_LABEL_CMOS_BASE);
        assert_eq!(dip_flag.offset, 0);
        assert_eq!(dip_flag.cells, 1);
        assert_eq!(dip_flag.address_range(), Some(0xC400..0xC401));
        assert_eq!(dip_flag.offset_range(), Some(0x0000..0x0001));

        let top_high_score = entries
            .iter()
            .find(|entry| entry.symbol == "CRHSTD")
            .expect("CRHSTD entry");
        assert_eq!(top_high_score.address, 0xC41D);
        assert_eq!(top_high_score.cells, 12);

        let eighth_high_score = entries
            .iter()
            .find(|entry| entry.symbol == "CRHSTD+12*7")
            .expect("eighth high-score entry");
        assert_eq!(eighth_high_score.address_range(), Some(0xC471..0xC47D));

        let credit_backup = entries
            .iter()
            .find(|entry| entry.symbol == "CREDST")
            .expect("CREDST marker");
        assert_eq!(credit_backup.address, 0xC47D);
        assert_eq!(credit_backup.cells, 0);
        assert_eq!(credit_backup.address_range(), Some(0xC47D..0xC47D));

        let boundary_check = entries
            .iter()
            .find(|entry| entry.symbol == "CMOSCK")
            .expect("CMOSCK entry");
        assert_eq!(boundary_check.address_range(), Some(0xC47F..0xC481));

        let restore_wave = entries
            .iter()
            .find(|entry| entry.symbol == "GA1+6")
            .expect("restore wave game-adjust entry");
        assert_eq!(restore_wave.offset_range(), Some(0x009D..0x009F));
        assert_eq!(restore_wave.description, "4 RESTORE WAVE #");
    }

    #[test]
    fn embedded_cmos_defaults_match_rom_defalt_bytes() {
        let defaults = red_label_cmos_defaults().expect("CMOS defaults parse");

        assert_eq!(defaults.len(), 30);

        let top_high_score = defaults
            .iter()
            .find(|entry| entry.symbol == "CRHSTD")
            .expect("top high-score default");
        assert_eq!(top_high_score.offset, 0x1D);
        assert_eq!(top_high_score.cells, 12);
        assert_eq!(
            top_high_score.bytes.as_slice(),
            &[0x02, 0x12, 0x70, b'D', b'R', b'J']
        );
        assert_eq!(top_high_score.cell_range(), Some(0x001D..0x0029));
        assert_eq!(
            top_high_score.encoded_cells().as_slice(),
            &[
                0xF0, 0xF2, 0xF1, 0xF2, 0xF7, 0xF0, 0xF4, 0xF4, 0xF5, 0xF2, 0xF4, 0xFA
            ]
        );

        let replay = defaults
            .iter()
            .find(|entry| entry.symbol == "REPLAY")
            .expect("replay default");
        assert_eq!(replay.offset, 0x81);
        assert_eq!(replay.bytes.as_slice(), &[0x01, 0x00]);
        assert_eq!(replay.encoded_cells().as_slice(), &[0xF0, 0xF1, 0xF0, 0xF0]);

        let last = defaults.last().expect("last default");
        assert_eq!(last.symbol, "GA1+18");
        assert_eq!(last.cell_range(), Some(0x00A9..0x00AB));
        assert!(last.cell_range().expect("range").end <= RED_LABEL_CMOS_CELLS);
    }

    #[test]
    fn embedded_ram_layout_contains_source_owned_tables() {
        let fields = red_label_ram_layout().expect("RAM layout parses");

        let background_left = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "BGL")
            .expect("background-left field");
        assert_eq!(background_left.base, 0xA000);
        assert_eq!(
            background_left.field_range_for_entry(0),
            Some(0xA020..0xA022)
        );

        let previous_background_left = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "BGLX")
            .expect("previous-background-left field");
        assert_eq!(
            previous_background_left.field_range_for_entry(0),
            Some(0xA022..0xA024)
        );

        let pseudo_color_ram = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "PCRAM")
            .expect("pseudo-color RAM field");
        assert_eq!(
            pseudo_color_ram.field_range_for_entry(0),
            Some(0xA026..0xA036)
        );

        let terrain_right_pointer = fields
            .iter()
            .find(|entry| entry.table == "terrain_runtime" && entry.field == "RTPTR")
            .expect("terrain right data pointer");
        assert_eq!(
            terrain_right_pointer.field_range_for_entry(0),
            Some(0xA00B..0xA00D)
        );

        let altitude_table = fields
            .iter()
            .find(|entry| entry.table == "terrain_altitude" && entry.field == "ALTTBL")
            .expect("terrain altitude table");
        assert_eq!(
            altitude_table.field_range_for_entry(0),
            Some(0xB300..0xB700)
        );

        let terrain_flavor_0 = fields
            .iter()
            .find(|entry| entry.table == "terrain_flavor_0" && entry.field == "TERTF0")
            .expect("terrain flavor 0 table");
        assert_eq!(
            terrain_flavor_0.field_range_for_entry(0),
            Some(0xB700..0xBA90)
        );

        let terrain_flavor_1 = fields
            .iter()
            .find(|entry| entry.table == "terrain_flavor_1" && entry.field == "TERTF1")
            .expect("terrain flavor 1 table");
        assert_eq!(
            terrain_flavor_1.field_range_for_entry(0),
            Some(0xBA90..0xBE20)
        );

        let terrain_screen_table = fields
            .iter()
            .find(|entry| entry.table == "terrain_screen_table" && entry.field == "STBL")
            .expect("terrain screen table");
        assert_eq!(
            terrain_screen_table.field_range_for_entry(0),
            Some(0xBE20..0xBF50)
        );

        let map_control = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "MAPCR")
            .expect("map-control field");
        assert_eq!(map_control.field_range_for_entry(0), Some(0xA036..0xA037));

        let credit = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "CREDIT")
            .expect("credit field");
        assert_eq!(credit.field_range_for_entry(0), Some(0xA037..0xA038));

        let credit_units = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "CUNITS")
            .expect("credit-units field");
        assert_eq!(credit_units.field_range_for_entry(0), Some(0xA038..0xA039));

        let bonus_units = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "BUNITS")
            .expect("bonus-units field");
        assert_eq!(bonus_units.field_range_for_entry(0), Some(0xA039..0xA03A));

        let text_cursor = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "CURSER")
            .expect("text-cursor field");
        assert_eq!(text_cursor.field_range_for_entry(0), Some(0xA050..0xA052));

        let overload_counter = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "OVCNT")
            .expect("overload-counter field");
        assert_eq!(
            overload_counter.field_range_for_entry(0),
            Some(0xA05E..0xA05F)
        );

        let firq_timer = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "TIMER")
            .expect("FIRQ timer field");
        assert_eq!(firq_timer.field_range_for_entry(0), Some(0xA05D..0xA05E));

        let interrupt_temp = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "ITEMP")
            .expect("interrupt-temp field");
        assert_eq!(
            interrupt_temp.field_range_for_entry(0),
            Some(0xA06F..0xA071)
        );

        let interrupt_temp_2 = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "ITEMP2")
            .expect("second interrupt-temp field");
        assert_eq!(
            interrupt_temp_2.field_range_for_entry(0),
            Some(0xA071..0xA073)
        );

        let scratch_word = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "XTEMP")
            .expect("scratch-word field");
        assert_eq!(scratch_word.field_range_for_entry(0), Some(0xA073..0xA075));

        let scratch_word_2 = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "XTEMP2")
            .expect("second scratch-word field");
        assert_eq!(
            scratch_word_2.field_range_for_entry(0),
            Some(0xA075..0xA077)
        );

        let scanner_erase_end = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "SETEND")
            .expect("scanner erase end field");
        assert_eq!(
            scanner_erase_end.field_range_for_entry(0),
            Some(0xA097..0xA099)
        );

        let pia21 = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "PIA21")
            .expect("PIA21 input field");
        assert_eq!(pia21.field_range_for_entry(0), Some(0xA07B..0xA07C));

        let score_flag = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "SCRFLG")
            .expect("score-flag field");
        assert_eq!(score_flag.field_range_for_entry(0), Some(0xA08A..0xA08B));

        let current_player = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "CURPLR")
            .expect("current-player field");
        assert_eq!(
            current_player.field_range_for_entry(0),
            Some(0xA08B..0xA08C)
        );

        let current_player_index = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "PLRX")
            .expect("current-player index field");
        assert_eq!(
            current_player_index.field_range_for_entry(0),
            Some(0xA08D..0xA08F)
        );

        let irq_hook = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "IRQHK")
            .expect("IRQ hook field");
        assert_eq!(irq_hook.field_range_for_entry(0), Some(0xA08F..0xA092));

        let irq_flag = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "IFLG")
            .expect("IRQ flag field");
        assert_eq!(irq_flag.field_range_for_entry(0), Some(0xA092..0xA093));

        let shell_count = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "BMBCNT")
            .expect("shell-count field");
        assert_eq!(shell_count.field_range_for_entry(0), Some(0xA099..0xA09A));

        let smart_bomb_flag = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "SBFLG")
            .expect("smart-bomb flag field");
        assert_eq!(
            smart_bomb_flag.field_range_for_entry(0),
            Some(0xA09A..0xA09B)
        );

        let shell_temp = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "SHTEMP")
            .expect("shell-temp field");
        assert_eq!(shell_temp.field_range_for_entry(0), Some(0xA09D..0xA09F));

        for (field, range) in [
            ("XXX1", 0xA0A1..0xA0A2),
            ("XXX2", 0xA0A2..0xA0A3),
            ("XXX3", 0xA0A3..0xA0A4),
        ] {
            let screen_output_param = fields
                .iter()
                .find(|entry| entry.table == "base_page" && entry.field == field)
                .unwrap_or_else(|| panic!("{field} screen-output parameter"));
            assert_eq!(screen_output_param.field_range_for_entry(0), Some(range));
        }

        let bomb_image_pointer = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "BAX")
            .expect("bomb-image-pointer field");
        assert_eq!(
            bomb_image_pointer.field_range_for_entry(0),
            Some(0xA0A6..0xA0A8)
        );

        let fireball_pointer = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "FBX")
            .expect("fireball-pointer field");
        assert_eq!(
            fireball_pointer.field_range_for_entry(0),
            Some(0xA0A8..0xA0AA)
        );

        let target_list = fields
            .iter()
            .find(|entry| entry.table == "target_list" && entry.field == "TLIST")
            .expect("target list");
        assert_eq!(target_list.base, 0xA11A);
        assert_eq!(target_list.entry_size, 0x28);
        assert_eq!(target_list.field_range_for_entry(0), Some(0xA11A..0xA142));

        let ufo_count = fields
            .iter()
            .find(|entry| entry.table == "enemy_runtime" && entry.field == "UFOCNT")
            .expect("UFO active-count field");
        assert_eq!(ufo_count.field_range_for_entry(0), Some(0xA119..0xA11A));

        let fissle_pointer = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "FISX")
            .expect("fissle-pointer field");
        assert_eq!(
            fissle_pointer.field_range_for_entry(0),
            Some(0xA0A4..0xA0A6)
        );

        let replay = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "REPLA")
            .expect("replay field");
        assert_eq!(replay.field_range_for_entry(0), Some(0xA0AB..0xA0AD));

        let sound_index = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "SNDX")
            .expect("sound-index field");
        assert_eq!(sound_index.field_range_for_entry(0), Some(0xA0B0..0xA0B2));

        let status = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "STATUS")
            .expect("status field");
        assert_eq!(status.field_range_for_entry(0), Some(0xA0BA..0xA0BB));

        let laser_flag = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "LFLG")
            .expect("laser-flag field");
        assert_eq!(laser_flag.field_range_for_entry(0), Some(0xA0B5..0xA0B6));

        let laser_collision_x = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "LCOLRX")
            .expect("laser-collision-x field");
        assert_eq!(
            laser_collision_x.field_range_for_entry(0),
            Some(0xA0B6..0xA0B7)
        );

        let power_flag = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "PWRFLG")
            .expect("power flag field");
        assert_eq!(power_flag.field_range_for_entry(0), Some(0xA0B7..0xA0B8));

        let willi_cursor = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "WCURS")
            .expect("Willi cursor field");
        assert_eq!(willi_cursor.field_range_for_entry(0), Some(0xA0B8..0xA0BA));

        let next_player_direction = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "NPLAD")
            .expect("next-player-direction field");
        assert_eq!(
            next_player_direction.field_range_for_entry(0),
            Some(0xA0BB..0xA0BD)
        );

        let player_direction = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "PLADIR")
            .expect("player-direction field");
        assert_eq!(
            player_direction.field_range_for_entry(0),
            Some(0xA0BD..0xA0BF)
        );

        let player_upper_left = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "PLAXC")
            .expect("player-upper-left field");
        assert_eq!(
            player_upper_left.field_range_for_entry(0),
            Some(0xA0BF..0xA0C1)
        );

        let player_collision_flag = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "PCFLG")
            .expect("player-collision flag field");
        assert_eq!(
            player_collision_flag.field_range_for_entry(0),
            Some(0xA0DE..0xA0DF)
        );

        let random_seed = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "SEED")
            .expect("random-seed field");
        assert_eq!(random_seed.field_range_for_entry(0), Some(0xA0DF..0xA0E0));

        let random_seed_high = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "HSEED")
            .expect("random-seed high field");
        assert_eq!(
            random_seed_high.field_range_for_entry(0),
            Some(0xA0E0..0xA0E1)
        );

        let random_seed_low = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "LSEED")
            .expect("random-seed low field");
        assert_eq!(
            random_seed_low.field_range_for_entry(0),
            Some(0xA0E1..0xA0E2)
        );

        let last_explosion = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "LSEXPL")
            .expect("last-explosion field");
        assert_eq!(
            last_explosion.field_range_for_entry(0),
            Some(0xA0E2..0xA0E4)
        );

        let expanded_x_start = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "XSTART")
            .expect("expanded-writer x-start field");
        assert_eq!(
            expanded_x_start.field_range_for_entry(0),
            Some(0xA0E9..0xA0EB)
        );

        let data_pointer = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "DATPTR")
            .expect("expanded-erase data pointer field");
        assert_eq!(data_pointer.field_range_for_entry(0), Some(0xA0F3..0xA0F5));

        let collision_center = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "CENTMP")
            .expect("collision-center field");
        assert_eq!(
            collision_center.field_range_for_entry(0),
            Some(0xA0F8..0xA0FA)
        );

        let astronaut_count = fields
            .iter()
            .find(|entry| entry.table == "base_page" && entry.field == "ASTCNT")
            .expect("astronaut-count field");
        assert_eq!(
            astronaut_count.field_range_for_entry(0),
            Some(0xA0FA..0xA0FB)
        );

        let star_x = fields
            .iter()
            .find(|entry| entry.table == "star_map" && entry.field == "SX")
            .expect("star x field");
        assert_eq!(star_x.base, 0xAF9D);
        assert_eq!(star_x.entry_size, 0x04);
        assert_eq!(star_x.entries, 16);
        assert_eq!(star_x.table_range(), Some(0xAF9D..0xAFDD));
        assert_eq!(star_x.field_range_for_entry(0), Some(0xAF9D..0xAF9E));
        assert_eq!(star_x.field_range_for_entry(15), Some(0xAFD9..0xAFDA));

        let star_y = fields
            .iter()
            .find(|entry| entry.table == "star_map" && entry.field == "SY")
            .expect("star y field");
        assert_eq!(star_y.field_range_for_entry(0), Some(0xAF9E..0xAF9F));

        let star_color = fields
            .iter()
            .find(|entry| entry.table == "star_map" && entry.field == "SCOL")
            .expect("star color field");
        assert_eq!(star_color.field_range_for_entry(15), Some(0xAFDB..0xAFDC));

        let laser_fizzle_table = fields
            .iter()
            .find(|entry| entry.table == "laser_fizzle" && entry.field == "FISTAB")
            .expect("laser-fizzle table");
        assert_eq!(laser_fizzle_table.base, 0xA142);
        assert_eq!(laser_fizzle_table.entry_size, 0x20);
        assert_eq!(
            laser_fizzle_table.field_range_for_entry(0),
            Some(0xA142..0xA162)
        );

        let thrust_table = fields
            .iter()
            .find(|entry| entry.table == "thrust_table" && entry.field == "THTAB")
            .expect("thrust table");
        assert_eq!(thrust_table.base, 0xA162);
        assert_eq!(thrust_table.entry_size, 0x40);
        assert_eq!(thrust_table.field_range_for_entry(0), Some(0xA162..0xA1A2));

        let fireball_table = fields
            .iter()
            .find(|entry| entry.table == "fireball_table" && entry.field == "FBTAB")
            .expect("fireball table");
        assert_eq!(fireball_table.base, 0xA1A2);
        assert_eq!(fireball_table.entry_size, 0x20);
        assert_eq!(
            fireball_table.field_range_for_entry(0),
            Some(0xA1A2..0xA1C2)
        );

        let appearance_size = fields
            .iter()
            .find(|entry| entry.table == "appearance_ram" && entry.field == "RSIZE")
            .expect("appearance size field");
        assert_eq!(appearance_size.base, 0x9C00);
        assert_eq!(appearance_size.entry_size, 0x40);
        assert_eq!(appearance_size.entries, 16);
        assert_eq!(
            appearance_size.field_range_for_entry(15),
            Some(0x9FC0..0x9FC2)
        );

        let appearance_object = fields
            .iter()
            .find(|entry| entry.table == "appearance_ram" && entry.field == "OBJPTR")
            .expect("appearance object pointer field");
        assert_eq!(
            appearance_object.field_range_for_entry(0),
            Some(0x9C0A..0x9C0C)
        );

        let p2_wave = fields
            .iter()
            .find(|entry| entry.table == "player" && entry.field == "PWAV")
            .expect("player wave field");
        assert_eq!(p2_wave.base, 0xA1C2);
        assert_eq!(p2_wave.entry_size, 0x3D);
        assert_eq!(p2_wave.entries, 2);
        assert_eq!(p2_wave.offset, 0x08);
        assert_eq!(p2_wave.field_range_for_entry(1), Some(0xA207..0xA208));

        let object_data = fields
            .iter()
            .find(|entry| entry.table == "object" && entry.field == "ODATA")
            .expect("object misc data field");
        assert_eq!(object_data.base, 0xA23C);
        assert_eq!(object_data.entry_size, 0x17);
        assert_eq!(object_data.entries, 95);
        assert_eq!(object_data.offset, 0x15);
        assert_eq!(object_data.size, 0x02);

        let process_data = fields
            .iter()
            .find(|entry| entry.table == "process" && entry.field == "PDATA")
            .expect("regular process data field");
        assert_eq!(process_data.table_range(), Some(0xAAC5..0xAF2A));
        assert_eq!(process_data.size, 0x08);

        let super_process_data = fields
            .iter()
            .find(|entry| entry.table == "super_process" && entry.field == "PDATA")
            .expect("super process data field");
        assert_eq!(super_process_data.table_range(), Some(0xAF2A..0xAF9D));
        assert_eq!(super_process_data.size, 0x10);

        let mono_picture = fields
            .iter()
            .find(|entry| entry.table == "mono_picture" && entry.field == "MONOTB")
            .expect("monochrome player-death picture table");
        assert_eq!(mono_picture.field_range_for_entry(0), Some(0xAFDD..0xB05D));

        let scanner_object = fields
            .iter()
            .find(|entry| entry.table == "scanner_object_erase" && entry.field == "SETAB")
            .expect("scanner object erase table");
        assert_eq!(
            scanner_object.field_range_for_entry(0),
            Some(0xB05D..0xB125)
        );

        let scanner_terrain = fields
            .iter()
            .find(|entry| entry.table == "scanner_terrain_erase" && entry.field == "STETAB")
            .expect("scanner terrain erase table");
        assert_eq!(
            scanner_terrain.field_range_for_entry(0),
            Some(0xB125..0xB1A5)
        );
    }

    #[test]
    fn embedded_linked_lists_capture_shells_as_object_cells() {
        let lists = red_label_linked_lists().expect("linked lists parse");

        let shell_list = lists
            .iter()
            .find(|entry| entry.list == "shell_object")
            .expect("shell object list");
        assert_eq!(shell_list.head_symbol, "SPTR");
        assert_eq!(shell_list.head_address, 0xA06D);
        assert_eq!(shell_list.entry_table, "object");
        assert_eq!(shell_list.link_field, "OLINK");

        assert!(lists.iter().any(|entry| {
            entry.list == "active_process"
                && entry.head_symbol == "ACTIVE"
                && entry.entry_table == "process"
                && entry.link_field == "PLINK"
        }));
    }

    #[test]
    fn ram_layout_parser_rejects_bad_header_and_invalid_fields() {
        let error = parse_red_label_ram_layout_tsv("wat\n").expect_err("bad header");
        assert!(error.contains("unexpected RAM layout header"));

        let text = "table\tbase\tentry_size\tentries\tfield\toffset\tsize\tsource\nobject\tA23C\t0x17\t95\tOLINK\t0x00\t0x02\tphr6\n";
        let error = parse_red_label_ram_layout_tsv(text).expect_err("bad base");
        assert!(error.contains("is not hex"));

        let text = "table\tbase\tentry_size\tentries\tfield\toffset\tsize\tsource\nobject\t0xA23C\t0x17\t95\tODATA\t0x16\t0x02\tphr6\n";
        let error = parse_red_label_ram_layout_tsv(text).expect_err("bad field range");
        assert!(error.contains("exceeds entry size"));

        let text = "table\tbase\tentry_size\tentries\tfield\toffset\tsize\tsource\nobject\t0xA23C\t0x17\t0\tODATA\t0x15\t0x02\tphr6\n";
        let error = parse_red_label_ram_layout_tsv(text).expect_err("zero entries");
        assert!(error.contains("entries is zero"));
    }

    #[test]
    fn linked_list_parser_rejects_bad_header_and_addresses() {
        let error = parse_red_label_linked_lists_tsv("wat\n").expect_err("bad header");
        assert!(error.contains("unexpected linked lists header"));

        let text = "list\thead_symbol\thead_address\tentry_table\tlink_field\tsource\nshell_object\tSPTR\tA06D\tobject\tOLINK\tphr6\n";
        let error = parse_red_label_linked_lists_tsv(text).expect_err("bad address");
        assert!(error.contains("is not hex"));
    }

    #[test]
    fn cmos_layout_parser_rejects_bad_header_and_address_drift() {
        let error = parse_red_label_cmos_layout_tsv("wat\n").expect_err("bad header");
        assert!(error.contains("unexpected CMOS layout header"));

        let text = "symbol\taddress\toffset\tcells\tdescription\tsource\nDIPFLG\tC400\t0x00\t1\tDIPSW SET FLAG\tphr6\n";
        let error = parse_red_label_cmos_layout_tsv(text).expect_err("bad address");
        assert!(error.contains("is not hex"));

        let text = "symbol\taddress\toffset\tcells\tdescription\tsource\nDIPFLG\t0xC400\t0x01\t1\tDIPSW SET FLAG\tphr6\n";
        let error = parse_red_label_cmos_layout_tsv(text).expect_err("bad offset");
        assert!(error.contains("address/offset mismatch"));

        let text = "symbol\taddress\toffset\tcells\tdescription\tsource\nDIPFLG\t0xC3FF\t0x00\t1\tDIPSW SET FLAG\tphr6\n";
        let error = parse_red_label_cmos_layout_tsv(text).expect_err("precedes base");
        assert!(error.contains("precedes CMOS base"));
    }

    #[test]
    fn cmos_defaults_parser_rejects_bad_header_and_cell_drift() {
        let error = parse_red_label_cmos_defaults_tsv("wat\n").expect_err("bad header");
        assert!(error.contains("unexpected CMOS defaults header"));

        let text =
            "symbol\toffset\tcells\tbytes\tdescription\tsource\nDIPSW\t7D\t2\t0x00\tDIPSW\tromc8\n";
        let error = parse_red_label_cmos_defaults_tsv(text).expect_err("bad offset");
        assert!(error.contains("is not hex"));

        let text = "symbol\toffset\tcells\tbytes\tdescription\tsource\nDIPSW\t0x7D\t1\t0x00\tDIPSW\tromc8\n";
        let error = parse_red_label_cmos_defaults_tsv(text).expect_err("bad cells");
        assert!(error.contains("does not match 2 encoded cells"));

        let text =
            "symbol\toffset\tcells\tbytes\tdescription\tsource\nDIPSW\t0x7D\t2\t00\tDIPSW\tromc8\n";
        let error = parse_red_label_cmos_defaults_tsv(text).expect_err("bad byte");
        assert!(error.contains("byte '00' is not hex"));

        let text =
            "symbol\toffset\tcells\tbytes\tdescription\tsource\nOVER\t0xFF\t2\t0x00\tOVER\tromc8\n";
        let error = parse_red_label_cmos_defaults_tsv(text).expect_err("overflow");
        assert!(error.contains("range exceeds CMOS cell space"));
    }
}
