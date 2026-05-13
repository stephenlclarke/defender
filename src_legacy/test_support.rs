//! Shared mutation assertions for refactor-safety tests.
//!
//! These helpers intentionally capture raw red-label bytes instead of derived
//! Rust state. They are only compiled for tests and are meant to make future
//! routine translations assert source-visible mutations consistently.

use std::ops::Range;

use crate::{
    machine::ArcadeMachine,
    red_label_memory::{RedLabelRamLayoutEntry, red_label_linked_lists, red_label_ram_layout},
};

const RED_LABEL_VISIBLE_VIDEO_RAM_END: u16 = 0x9C00;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MachineByteSource {
    Ram,
    Cmos,
}

impl MachineByteSource {
    fn label(self) -> &'static str {
        match self {
            Self::Ram => "red-label RAM",
            Self::Cmos => "red-label CMOS",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MachineByteSnapshot {
    label: &'static str,
    source: MachineByteSource,
    range: Range<u16>,
    bytes: Vec<u8>,
}

impl MachineByteSnapshot {
    fn capture(
        machine: &ArcadeMachine,
        source: MachineByteSource,
        label: &'static str,
        range: Range<u16>,
    ) -> Self {
        let bytes = read_machine_bytes(machine, source, range.clone(), label).to_vec();
        Self {
            label,
            source,
            range,
            bytes,
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn range(&self) -> Range<u16> {
        self.range.clone()
    }

    pub(crate) fn assert_current_unchanged(&self, machine: &ArcadeMachine) {
        let current = self.current_bytes(machine);
        assert_eq!(
            current,
            self.bytes.as_slice(),
            "{} changed unexpectedly in {} {}",
            self.label,
            self.source.label(),
            format_range(&self.range)
        );
    }

    pub(crate) fn assert_current_changed(&self, machine: &ArcadeMachine) {
        let current = self.current_bytes(machine);
        assert_ne!(
            current,
            self.bytes.as_slice(),
            "{} did not change in {} {}",
            self.label,
            self.source.label(),
            format_range(&self.range)
        );
    }

    pub(crate) fn assert_current_changed_to(&self, machine: &ArcadeMachine, expected_after: &[u8]) {
        self.assert_current_changed(machine);
        let current = self.current_bytes(machine);
        assert_eq!(
            current,
            expected_after,
            "{} changed to unexpected bytes in {} {}",
            self.label,
            self.source.label(),
            format_range(&self.range)
        );
    }

    fn current_bytes<'a>(&self, machine: &'a ArcadeMachine) -> &'a [u8] {
        read_machine_bytes(machine, self.source, self.range.clone(), self.label)
    }
}

pub(crate) fn red_label_ram_snapshot(
    machine: &ArcadeMachine,
    label: &'static str,
    range: Range<u16>,
) -> MachineByteSnapshot {
    MachineByteSnapshot::capture(machine, MachineByteSource::Ram, label, range)
}

pub(crate) fn red_label_cmos_snapshot(
    machine: &ArcadeMachine,
    label: &'static str,
    range: Range<u16>,
) -> MachineByteSnapshot {
    MachineByteSnapshot::capture(machine, MachineByteSource::Cmos, label, range)
}

pub(crate) fn red_label_video_ram_snapshot(
    machine: &ArcadeMachine,
    label: &'static str,
    range: Range<u16>,
) -> MachineByteSnapshot {
    assert!(
        range.start <= range.end && range.end <= RED_LABEL_VISIBLE_VIDEO_RAM_END,
        "{label} video RAM range {} is outside visible video RAM 0x0000..0x{RED_LABEL_VISIBLE_VIDEO_RAM_END:04X}",
        format_range(&range)
    );
    red_label_ram_snapshot(machine, label, range)
}

pub(crate) fn red_label_object_cell_snapshot(
    machine: &ArcadeMachine,
    object_address: u16,
) -> MachineByteSnapshot {
    red_label_ram_snapshot(
        machine,
        "object cell",
        red_label_table_entry_range("object", object_address),
    )
}

pub(crate) fn red_label_process_cell_snapshot(
    machine: &ArcadeMachine,
    process_address: u16,
) -> MachineByteSnapshot {
    red_label_ram_snapshot(
        machine,
        "process cell",
        red_label_table_entry_range("process", process_address),
    )
}

pub(crate) fn red_label_super_process_cell_snapshot(
    machine: &ArcadeMachine,
    process_address: u16,
) -> MachineByteSnapshot {
    red_label_ram_snapshot(
        machine,
        "super-process cell",
        red_label_table_entry_range("super_process", process_address),
    )
}

pub(crate) fn red_label_shell_list_head_snapshot(machine: &ArcadeMachine) -> MachineByteSnapshot {
    let lists = red_label_linked_lists().expect("embedded red-label linked-list asset is valid");
    let shell_list = lists
        .iter()
        .find(|entry| entry.list == "shell_object")
        .expect("embedded red-label linked-list asset has SPTR shell list");
    red_label_ram_snapshot(
        machine,
        "SPTR shell-list head",
        shell_list.head_address..shell_list.head_address + 2,
    )
}

fn read_machine_bytes<'a>(
    machine: &'a ArcadeMachine,
    source: MachineByteSource,
    range: Range<u16>,
    label: &str,
) -> &'a [u8] {
    let range_text = format_range(&range);
    match source {
        MachineByteSource::Ram => machine.red_label_ram_range(range).unwrap_or_else(|| {
            panic!("{label} red-label RAM range {range_text} is outside machine RAM")
        }),
        MachineByteSource::Cmos => machine.red_label_cmos_range(range).unwrap_or_else(|| {
            panic!("{label} red-label CMOS range {range_text} is outside CMOS RAM")
        }),
    }
}

fn red_label_table_entry_range(table_name: &str, address: u16) -> Range<u16> {
    let layout = red_label_ram_layout().expect("embedded red-label RAM layout asset is valid");
    let table = table_descriptor(&layout, table_name);
    let table_range = table
        .table_range()
        .expect("embedded red-label RAM table range is valid");
    assert!(
        table_range.contains(&address),
        "red-label address 0x{address:04X} is outside `{table_name}` table {}",
        format_range(&table_range)
    );
    let offset = address - table.base;
    assert!(
        offset.is_multiple_of(table.entry_size),
        "red-label address 0x{address:04X} is not aligned to `{table_name}` entry size 0x{:04X}",
        table.entry_size
    );
    let start = table.base + (offset / table.entry_size) * table.entry_size;
    let end = start
        .checked_add(table.entry_size)
        .expect("red-label table entry end is valid");
    start..end
}

fn table_descriptor<'a>(
    layout: &'a [RedLabelRamLayoutEntry],
    table_name: &str,
) -> &'a RedLabelRamLayoutEntry {
    layout
        .iter()
        .find(|entry| entry.table == table_name)
        .unwrap_or_else(|| panic!("embedded red-label RAM layout has `{table_name}` table"))
}

fn format_range(range: &Range<u16>) -> String {
    format!("0x{:04X}..0x{:04X}", range.start, range.end)
}

#[cfg(test)]
mod tests {
    use super::{
        red_label_cmos_snapshot, red_label_object_cell_snapshot, red_label_process_cell_snapshot,
        red_label_ram_snapshot, red_label_shell_list_head_snapshot,
        red_label_super_process_cell_snapshot, red_label_video_ram_snapshot,
    };
    use crate::machine::{ArcadeMachine, RED_LABEL_SYSTEM_PROCESS_TYPE, RedLabelObjectDescriptor};

    #[test]
    fn snapshots_capture_source_owned_machine_surfaces() {
        let machine = ArcadeMachine::new();

        let ram = red_label_ram_snapshot(&machine, "credit byte", 0xA037..0xA038);
        assert_eq!(ram.bytes(), &[0]);
        assert_eq!(ram.range(), 0xA037..0xA038);
        ram.assert_current_unchanged(&machine);

        let cmos = red_label_cmos_snapshot(&machine, "credit backup cells", 0x7D..0x7F);
        assert_eq!(cmos.bytes().len(), 2);
        cmos.assert_current_unchanged(&machine);

        let video = red_label_video_ram_snapshot(&machine, "first visible byte", 0x0000..0x0001);
        assert_eq!(video.bytes(), &[0]);
        video.assert_current_unchanged(&machine);

        let object = red_label_object_cell_snapshot(&machine, 0xA23C);
        assert_eq!(object.bytes().len(), 0x17);

        let process = red_label_process_cell_snapshot(&machine, 0xAAC5);
        assert_eq!(process.bytes().len(), 0x0F);

        let super_process = red_label_super_process_cell_snapshot(&machine, 0xAF2A);
        assert_eq!(super_process.bytes().len(), 0x17);

        let shell_head = red_label_shell_list_head_snapshot(&machine);
        assert_eq!(shell_head.bytes(), &[0, 0]);
        shell_head.assert_current_unchanged(&machine);
    }

    #[test]
    fn snapshots_assert_process_object_and_video_mutations() {
        let mut machine = ArcadeMachine::new();
        let active_process_head =
            red_label_ram_snapshot(&machine, "ACTIVE process head", 0xA05F..0xA061);
        let process = red_label_process_cell_snapshot(&machine, 0xAAC5);

        let created = machine
            .red_label_make_process(0x1234, RED_LABEL_SYSTEM_PROCESS_TYPE)
            .expect("make process");

        assert_eq!(created.process_address, 0xAAC5);
        active_process_head.assert_current_changed_to(&machine, &[0xAA, 0xC5]);
        process.assert_current_changed(&machine);

        let object = red_label_object_cell_snapshot(&machine, 0xA23C);
        let object_address = machine
            .red_label_init_object_cell(
                created.process_address,
                RedLabelObjectDescriptor {
                    picture_address: 0xF901,
                    collision_vector_address: 0xD5B6,
                    scanner_color: 0x0200,
                },
            )
            .expect("init object cell")
            .object_address;
        assert_eq!(object_address, 0xA23C);
        object.assert_current_changed(&machine);

        let video = red_label_video_ram_snapshot(&machine, "ASTP1 video bytes", 0x2000..0x2002);
        machine
            .red_label_write_object_picture_cwrit(0x2000, 0xF901)
            .expect("write object picture");
        video.assert_current_changed(&machine);
    }

    #[test]
    fn snapshots_assert_cmos_and_super_process_mutations() {
        let machine = ArcadeMachine::new();
        let cmos = red_label_cmos_snapshot(&machine, "credit backup cells", 0x7D..0x7F);
        let mut custom_cmos = *machine.red_label_cmos_ram();
        custom_cmos[0x7D] = 0xF1;
        custom_cmos[0x7E] = 0xF2;
        let changed_machine =
            ArcadeMachine::try_new_with_cmos(custom_cmos).expect("custom CMOS is valid");
        cmos.assert_current_changed_to(&changed_machine, &[0xF1, 0xF2]);

        let mut machine = ArcadeMachine::new();
        let super_process = red_label_super_process_cell_snapshot(&machine, 0xAF2A);
        let created = machine
            .red_label_make_super_process(0x4567, RED_LABEL_SYSTEM_PROCESS_TYPE)
            .expect("make super-process");

        assert_eq!(created.process_address, 0xAF2A);
        super_process.assert_current_changed(&machine);
    }
}
