//! Embedded asset manifest for the self-contained runtime.
//!
//! Source data lives under `assets/`; runtime code embeds it from here so the
//! release binary can be copied to a new machine without a local asset tree.

pub const ARCADE_LOGO_PAGE_PNG: &[u8] = include_bytes!("../assets/arcade/logo-page.png");

pub const RED_LABEL_AUDIT_ADJUSTMENTS_TSV: &str =
    include_str!("../assets/red-label/audit-adjustments.tsv");
pub const RED_LABEL_COLOR_CYCLE_TSV: &str = include_str!("../assets/red-label/color-cycle.tsv");
pub const RED_LABEL_COLOR_RAM_TSV: &str = include_str!("../assets/red-label/color-ram.tsv");
pub const RED_LABEL_CMOS_DEFAULTS_TSV: &str = include_str!("../assets/red-label/cmos-defaults.tsv");
pub const RED_LABEL_CMOS_LAYOUT_TSV: &str = include_str!("../assets/red-label/cmos-layout.tsv");
pub const RED_LABEL_DEFAULTS_TSV: &str = include_str!("../assets/red-label/defaults.tsv");
pub const RED_LABEL_HIGH_SCORES_TSV: &str = include_str!("../assets/red-label/high-scores.tsv");
pub const RED_LABEL_INPUT_PORTS_TSV: &str = include_str!("../assets/red-label/input-ports.tsv");
pub const RED_LABEL_LINKED_LISTS_TSV: &str = include_str!("../assets/red-label/linked-lists.tsv");
pub const RED_LABEL_MEMORY_MAP_TSV: &str = include_str!("../assets/red-label/memory-map.tsv");
pub const RED_LABEL_MESSAGE_GLYPHS_TSV: &str =
    include_str!("../assets/red-label/message-glyphs.tsv");
pub const RED_LABEL_MESSAGES_TSV: &str = include_str!("../assets/red-label/messages.tsv");
pub const RED_LABEL_OBJECT_IMAGES_TSV: &str = include_str!("../assets/red-label/object-images.tsv");
pub const RED_LABEL_OBJECT_PICTURES_TSV: &str =
    include_str!("../assets/red-label/object-pictures.tsv");
pub const RED_LABEL_PLAYER_DEATH_TSV: &str = include_str!("../assets/red-label/player-death.tsv");
pub const RED_LABEL_RAM_LAYOUT_TSV: &str = include_str!("../assets/red-label/ram-layout.tsv");
pub const RED_LABEL_ROM_MAP_TSV: &str = include_str!("../assets/red-label/rom-map.tsv");
pub const RED_LABEL_ROM_REGIONS_TSV: &str = include_str!("../assets/red-label/rom-regions.tsv");
pub const RED_LABEL_ROMS_TSV: &str = include_str!("../assets/red-label/roms.tsv");
pub const RED_LABEL_ROUTINE_ADDRESSES_TSV: &str =
    include_str!("../assets/red-label/routine-addresses.tsv");
pub const RED_LABEL_SCORE_DIGITS_TSV: &str = include_str!("../assets/red-label/score-digits.tsv");
pub const RED_LABEL_SCORES_TSV: &str = include_str!("../assets/red-label/scores.tsv");
pub const RED_LABEL_SHELL_IMAGES_TSV: &str = include_str!("../assets/red-label/shell-images.tsv");
pub const RED_LABEL_SOUND_DIRECT_COMMAND_SEQUENCES_TSV: &str =
    include_str!("../assets/red-label/sound-direct-command-sequences.tsv");
pub const RED_LABEL_SOUND_TABLE_COMMAND_SEQUENCES_TSV: &str =
    include_str!("../assets/red-label/sound-table-command-sequences.tsv");
pub const RED_LABEL_SOUND_TABLES_TSV: &str = include_str!("../assets/red-label/sound-tables.tsv");
pub const RED_LABEL_SOUND_TABLE_TIMELINES_TSV: &str =
    include_str!("../assets/red-label/sound-table-timelines.tsv");
pub const RED_LABEL_SOUND_THRUST_COMMAND_SEQUENCES_TSV: &str =
    include_str!("../assets/red-label/sound-thrust-command-sequences.tsv");
pub const RED_LABEL_SRAM_ROUTINES_TSV: &str = include_str!("../assets/red-label/sram-routines.tsv");
pub const RED_LABEL_SWITCH_TABLE_TSV: &str = include_str!("../assets/red-label/switch-table.tsv");
pub const RED_LABEL_TERRAIN_DATA_TSV: &str = include_str!("../assets/red-label/terrain-data.tsv");
pub const RED_LABEL_TRACE_SCENARIOS_TSV: &str =
    include_str!("../assets/red-label/trace-scenarios.tsv");
pub const RED_LABEL_TRACE_REQUIREMENTS_TSV: &str =
    include_str!("../assets/red-label/trace-requirements.tsv");
pub const RED_LABEL_TRACE_SCHEMA_TSV: &str = include_str!("../assets/red-label/trace-schema.tsv");
pub const RED_LABEL_WAVE_TABLE_TSV: &str = include_str!("../assets/red-label/wave-table.tsv");

pub fn first_tsv_line(asset: &str) -> &str {
    asset.lines().next().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::{
        ARCADE_LOGO_PAGE_PNG, RED_LABEL_AUDIT_ADJUSTMENTS_TSV, RED_LABEL_CMOS_DEFAULTS_TSV,
        RED_LABEL_CMOS_LAYOUT_TSV, RED_LABEL_COLOR_CYCLE_TSV, RED_LABEL_COLOR_RAM_TSV,
        RED_LABEL_DEFAULTS_TSV, RED_LABEL_HIGH_SCORES_TSV, RED_LABEL_INPUT_PORTS_TSV,
        RED_LABEL_LINKED_LISTS_TSV, RED_LABEL_MEMORY_MAP_TSV, RED_LABEL_MESSAGE_GLYPHS_TSV,
        RED_LABEL_MESSAGES_TSV, RED_LABEL_OBJECT_IMAGES_TSV, RED_LABEL_OBJECT_PICTURES_TSV,
        RED_LABEL_PLAYER_DEATH_TSV, RED_LABEL_RAM_LAYOUT_TSV, RED_LABEL_ROM_MAP_TSV,
        RED_LABEL_ROM_REGIONS_TSV, RED_LABEL_ROMS_TSV, RED_LABEL_ROUTINE_ADDRESSES_TSV,
        RED_LABEL_SCORE_DIGITS_TSV, RED_LABEL_SCORES_TSV, RED_LABEL_SHELL_IMAGES_TSV,
        RED_LABEL_SOUND_DIRECT_COMMAND_SEQUENCES_TSV, RED_LABEL_SOUND_TABLE_COMMAND_SEQUENCES_TSV,
        RED_LABEL_SOUND_TABLE_TIMELINES_TSV, RED_LABEL_SOUND_TABLES_TSV,
        RED_LABEL_SOUND_THRUST_COMMAND_SEQUENCES_TSV, RED_LABEL_SRAM_ROUTINES_TSV,
        RED_LABEL_SWITCH_TABLE_TSV, RED_LABEL_TERRAIN_DATA_TSV, RED_LABEL_TRACE_REQUIREMENTS_TSV,
        RED_LABEL_TRACE_SCENARIOS_TSV, RED_LABEL_TRACE_SCHEMA_TSV, RED_LABEL_WAVE_TABLE_TSV,
        first_tsv_line,
    };

    #[test]
    fn embedded_red_label_assets_have_expected_headers() {
        assert_eq!(
            first_tsv_line(RED_LABEL_AUDIT_ADJUSTMENTS_TSV),
            "number\tsymbol\toffset\tcells\tmessage\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_COLOR_CYCLE_TSV),
            "label\taddress\tbytes\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_COLOR_RAM_TSV),
            "label\taddress\tbytes\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_CMOS_DEFAULTS_TSV),
            "symbol\toffset\tcells\tbytes\tdescription\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_CMOS_LAYOUT_TSV),
            "symbol\taddress\toffset\tcells\tdescription\tsource"
        );
        assert_eq!(first_tsv_line(RED_LABEL_DEFAULTS_TSV), "key\tvalue");
        assert_eq!(first_tsv_line(RED_LABEL_HIGH_SCORES_TSV), "initials\tscore");
        assert_eq!(
            first_tsv_line(RED_LABEL_INPUT_PORTS_TSV),
            "port\tbit\tname\tactive\tpia_port\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_LINKED_LISTS_TSV),
            "list\thead_symbol\thead_address\tentry_table\tlink_field\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_MEMORY_MAP_TSV),
            "cpu\tstart\tend\taccess\tbank_select\tmirror_mask\thandler\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_MESSAGE_GLYPHS_TSV),
            "label\tcharacter\taddress\twidth\theight\tbytes\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_MESSAGES_TSV),
            "label\tvector_address\twords\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_OBJECT_IMAGES_TSV),
            "label\taddress\tbytes\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_OBJECT_PICTURES_TSV),
            "label\taddress\twidth\theight\tprimary_image\talternate_image\toutput_routine\terase_routine\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_PLAYER_DEATH_TSV),
            "label\taddress\tbytes\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_RAM_LAYOUT_TSV),
            "table\tbase\tentry_size\tentries\tfield\toffset\tsize\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_ROM_MAP_TSV),
            "name\tregion\tregion_offset\tsize\tview\tbank\tcpu_start"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_ROM_REGIONS_TSV),
            "region\tsize\tsource"
        );
        assert_eq!(first_tsv_line(RED_LABEL_ROMS_TSV), "name\tsize\tcrc32");
        assert_eq!(
            first_tsv_line(RED_LABEL_ROUTINE_ADDRESSES_TSV),
            "label\taddress\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_SCORE_DIGITS_TSV),
            "label\tdigit\taddress\twidth\theight\tbytes\tsource"
        );
        assert_eq!(first_tsv_line(RED_LABEL_SCORES_TSV), "kind\tscore");
        assert_eq!(
            first_tsv_line(RED_LABEL_SHELL_IMAGES_TSV),
            "label\taddress\tbytes\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_SOUND_TABLES_TSV),
            "label\taddress\tbytes\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_SOUND_DIRECT_COMMAND_SEQUENCES_TSV),
            "callsite\tsource_file\tsource_line\tsource_label\tsound_number\twrite_index\twrite_kind\tport_b\tcommand\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_SOUND_TABLE_COMMAND_SEQUENCES_TSV),
            "label\ttable_address\tpriority\tsequencer_tick\twrite_index\twrite_kind\ttable_pointer\trepeat_index\trepeat_count\ttimer\tsound_number\tport_b\tcommand"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_SOUND_TABLE_TIMELINES_TSV),
            "label\ttable_address\tpriority\tsequencer_tick\tevent\ttable_pointer\trepeat_index\trepeat_count\ttimer\tsound_number\tidle_port_b\tidle_command\tcommand_port_b\tcommand\tsequence_end_tick\tterminator_pointer"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_SOUND_THRUST_COMMAND_SEQUENCES_TSV),
            "gate_event\tsource_label\tpia21_mask\tstatus_block_mask\tthrust_flag_before\tthrust_flag_after\tsound_number\twrite_index\twrite_kind\tport_b\tcommand"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_SRAM_ROUTINES_TSV),
            "routine\taddress\tregister\twidth_nibbles\tx_advance\toperation\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_SWITCH_TABLE_TSV),
            "bit\tname\troutine\tprocess_type\tstatus_mask\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_TERRAIN_DATA_TSV),
            "label\taddress\tbytes\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_TRACE_SCENARIOS_TSV),
            "scenario\tframes\tinput_program\tdescription\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_TRACE_REQUIREMENTS_TSV),
            "scenario\trequired_sound_commands\trequired_events\tdescription\tsource"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_TRACE_SCHEMA_TSV),
            "frame\tinput_bits\tinput_in0\tinput_in1\tinput_in2\tphase\tp1_score\tp2_score\twave\tlives\tsmart_bombs\tseed\thseed\tlseed\tobject_table_crc32\tprocess_table_crc32\tsuper_process_table_crc32\tshell_table_crc32\tvideo_crc32\tsound_commands\tevents"
        );
        assert_eq!(
            first_tsv_line(RED_LABEL_WAVE_TABLE_TSV),
            "key\tceiling\tfloor\tintra_delta\tinter_delta\twave1\twave2\twave3\twave4"
        );
    }

    #[test]
    fn logo_asset_is_embedded_png_data() {
        assert!(ARCADE_LOGO_PAGE_PNG.starts_with(b"\x89PNG\r\n\x1A\n"));
    }

    #[test]
    fn red_label_trace_schema_has_no_stale_docs_duplicate() {
        let duplicate =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/fidelity/trace-schema.tsv");

        assert!(
            !duplicate.exists(),
            "assets/red-label/trace-schema.tsv is the single checked-in trace schema source"
        );
    }
}
